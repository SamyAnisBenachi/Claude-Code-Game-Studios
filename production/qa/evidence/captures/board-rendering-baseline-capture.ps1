param(
    [string]$Url = "http://127.0.0.1:8080/board-rendering-perf-harness.html?fixture=board_rendering_baseline&seed=board-rendering-baseline-v1",
    [string]$ChromePath = "C:\Program Files\Google\Chrome\Application\chrome.exe",
    [int]$DebugPort = 9222,
    [int]$ReadyTimeoutSeconds = 60
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..\..\..")
$screenshotPath = Join-Path $repoRoot "production\qa\evidence\captures\board-rendering-baseline-1920x1080.png"
$tracePath = Join-Path $repoRoot "production\qa\evidence\captures\board-rendering-baseline-timing.json"
$screenshotRelativePath = "production/qa/evidence/captures/board-rendering-baseline-1920x1080.png"
$traceRelativePath = "production/qa/evidence/captures/board-rendering-baseline-timing.json"

if (-not (Test-Path -LiteralPath $ChromePath)) {
    throw "Chrome executable not found: $ChromePath"
}

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $screenshotPath) | Out-Null

function ConvertTo-ArgumentString {
    param([string[]]$Arguments)

    ($Arguments | ForEach-Object {
        if ($_ -match '[\s"]') {
            '"' + ($_ -replace '"', '\"') + '"'
        } else {
            $_
        }
    }) -join " "
}

function Invoke-CdpHttpJson {
    param(
        [string]$Endpoint,
        [string]$Method = "Get"
    )

    $response = Invoke-WebRequest -Uri $Endpoint -UseBasicParsing -Method $Method -TimeoutSec 5
    $response.Content | ConvertFrom-Json
}

function Receive-CdpMessage {
    $buffer = New-Object byte[] 1048576
    $segment = [ArraySegment[byte]]::new($buffer)
    $stream = [System.IO.MemoryStream]::new()

    do {
        $received = $script:Socket.ReceiveAsync($segment, [Threading.CancellationToken]::None).GetAwaiter().GetResult()
        if ($received.MessageType -eq [System.Net.WebSockets.WebSocketMessageType]::Close) {
            throw "Chrome DevTools websocket closed unexpectedly."
        }
        $stream.Write($buffer, 0, $received.Count)
    } while (-not $received.EndOfMessage)

    $json = [Text.Encoding]::UTF8.GetString($stream.ToArray())
    $json | ConvertFrom-Json
}

function Send-CdpCommand {
    param(
        [string]$Method,
        [hashtable]$Params = @{}
    )

    $script:NextCdpId += 1
    $id = $script:NextCdpId
    $payload = @{
        id = $id
        method = $Method
        params = $Params
    } | ConvertTo-Json -Depth 50 -Compress
    $bytes = [Text.Encoding]::UTF8.GetBytes($payload)
    $segment = [ArraySegment[byte]]::new($bytes)
    [void]$script:Socket.SendAsync($segment, [System.Net.WebSockets.WebSocketMessageType]::Text, $true, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

    while ($true) {
        $message = Receive-CdpMessage
        if ($message.PSObject.Properties.Name -contains "id" -and $message.id -eq $id) {
            if ($message.PSObject.Properties.Name -contains "error") {
                throw ($message.error | ConvertTo-Json -Depth 20)
            }
            return $message
        }
        [void]$script:CdpEvents.Add($message)
    }
}

function Get-HarnessPerf {
    $expression = "JSON.stringify(globalThis.__boardRenderingPerf ?? null)"
    $response = Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = $expression
        returnByValue = $true
        awaitPromise = $false
    }

    $value = $response.result.result.value
    if ([string]::IsNullOrWhiteSpace($value) -or $value -eq "null") {
        return $null
    }

    $value | ConvertFrom-Json
}

$profileDir = Join-Path ([IO.Path]::GetTempPath()) ("board-rendering-cdp-" + [Guid]::NewGuid().ToString("N"))
$chrome = $null
$script:Socket = $null
$script:NextCdpId = 0
$script:CdpEvents = [System.Collections.Generic.List[object]]::new()

try {
    New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

    $chromeArgs = @(
        "--headless=new",
        "--disable-dev-shm-usage",
        "--disable-background-timer-throttling",
        "--disable-backgrounding-occluded-windows",
        "--disable-frame-rate-limit",
        "--disable-renderer-backgrounding",
        "--no-first-run",
        "--no-default-browser-check",
        "--hide-scrollbars",
        "--remote-debugging-port=$DebugPort",
        "--user-data-dir=$profileDir",
        "--window-size=1920,1080",
        "about:blank"
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $ChromePath
    $startInfo.Arguments = ConvertTo-ArgumentString -Arguments $chromeArgs
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $chrome = [System.Diagnostics.Process]::Start($startInfo)

    $versionEndpoint = "http://127.0.0.1:$DebugPort/json/version"
    $deadline = (Get-Date).AddSeconds(20)
    do {
        try {
            $null = Invoke-CdpHttpJson -Endpoint $versionEndpoint
            break
        } catch {
            Start-Sleep -Milliseconds 250
        }
    } while ((Get-Date) -lt $deadline)

    $targetEndpoint = "http://127.0.0.1:$DebugPort/json/new?about%3Ablank"
    $target = Invoke-CdpHttpJson -Endpoint $targetEndpoint -Method "Put"
    $script:Socket = [System.Net.WebSockets.ClientWebSocket]::new()
    [void]$script:Socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

    Send-CdpCommand -Method "Page.enable" | Out-Null
    Send-CdpCommand -Method "Runtime.enable" | Out-Null
    Send-CdpCommand -Method "Log.enable" | Out-Null
    Send-CdpCommand -Method "Emulation.setDeviceMetricsOverride" -Params @{
        width = 1920
        height = 1080
        deviceScaleFactor = 1
        mobile = $false
    } | Out-Null
    Send-CdpCommand -Method "Page.navigate" -Params @{ url = $Url } | Out-Null

    $readyDeadline = (Get-Date).AddSeconds($ReadyTimeoutSeconds)
    $perf = $null
    do {
        Start-Sleep -Milliseconds 500
        $perf = Get-HarnessPerf
        if ($null -ne $perf -and $null -ne $perf.harnessReport -and $perf.harnessReport.ready_for_capture -eq $true) {
            break
        }
    } while ((Get-Date) -lt $readyDeadline)

    if ($null -eq $perf -or $null -eq $perf.harnessReport -or $perf.harnessReport.ready_for_capture -ne $true) {
        throw "BOARD-012 harness did not become ready for capture within $ReadyTimeoutSeconds seconds."
    }

    Start-Sleep -Seconds 2
    $perf = Get-HarnessPerf

    $screenshot = Send-CdpCommand -Method "Page.captureScreenshot" -Params @{
        format = "png"
        fromSurface = $true
    }
    [IO.File]::WriteAllBytes($screenshotPath, [Convert]::FromBase64String($screenshot.result.data))

    $harnessConsole = @(
        $script:CdpEvents |
            Where-Object { $_.method -eq "Runtime.consoleAPICalled" } |
            ForEach-Object {
                ($_.params.args | ForEach-Object {
                    if ($_.PSObject.Properties.Name -contains "value") { [string]$_.value }
                }) -join " "
            } |
            Where-Object { $_ -like "*BOARD-012 harness*" }
    )

    $chromeVersion = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($ChromePath).ProductVersion
    $result = [ordered]@{
        url = $Url
        viewport = [ordered]@{ width = 1920; height = 1080 }
        seed = "board-rendering-baseline-v1"
        screenshotPath = $screenshotRelativePath
        tracePath = $traceRelativePath
        browserFrameTiming = $perf
        harnessReport = $perf.harnessReport
        harnessConsole = $harnessConsole
        captureTool = "PowerShell Chrome DevTools Protocol"
        chromePath = $ChromePath
        chromeVersion = $chromeVersion
        capturedAt = (Get-Date).ToUniversalTime().ToString("o")
    }

    ($result | ConvertTo-Json -Depth 80) + "`n" | Set-Content -LiteralPath $tracePath -Encoding ascii
    $result | ConvertTo-Json -Depth 80
} finally {
    if ($null -ne $script:Socket) {
        $script:Socket.Dispose()
    }
    if ($null -ne $chrome -and -not $chrome.HasExited) {
        $chrome.Kill()
        $chrome.WaitForExit()
    }
    if (Test-Path -LiteralPath $profileDir) {
        Remove-Item -LiteralPath $profileDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
