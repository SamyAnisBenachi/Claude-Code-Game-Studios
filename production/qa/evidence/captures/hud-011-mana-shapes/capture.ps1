param(
    [string]$Url = "http://127.0.0.1:8081/hud-mana-shapes-harness.html?fixture=hud_011_mana_shapes",
    [string]$ChromePath = "C:\Program Files\Google\Chrome\Application\chrome.exe",
    [int]$DebugPort = 9223,
    [int]$ReadyTimeoutSeconds = 45
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..\..\..\..")
$relativeOutputDir = "production/qa/evidence/captures/hud-011-mana-shapes"
$outputDir = Join-Path $repoRoot ($relativeOutputDir -replace "/", "\")
$summaryPath = Join-Path $outputDir "capture-summary.json"
$summaryRelativePath = "$relativeOutputDir/capture-summary.json"

if (-not (Test-Path -LiteralPath $ChromePath)) {
    throw "Chrome executable not found: $ChromePath"
}

New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

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

function Get-HarnessState {
    $response = Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = "JSON.stringify(globalThis.__hudManaShapesEvidence ?? null)"
        returnByValue = $true
        awaitPromise = $false
    }

    $value = $response.result.result.value
    if ([string]::IsNullOrWhiteSpace($value) -or $value -eq "null") {
        return $null
    }

    $value | ConvertFrom-Json
}

function Get-RuntimeValue {
    param([string]$Expression)

    $response = Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = $Expression
        returnByValue = $true
        awaitPromise = $false
    }

    if ($response.result.result.PSObject.Properties.Name -contains "value") {
        return $response.result.result.value
    }

    return $null
}

function Get-CdpEventDiagnostics {
    @(
        $script:CdpEvents |
            Where-Object {
                $_.method -eq "Runtime.consoleAPICalled" -or
                $_.method -eq "Runtime.exceptionThrown" -or
                $_.method -eq "Log.entryAdded"
            } |
            ForEach-Object {
                $_ | ConvertTo-Json -Depth 20 -Compress
            }
    )
}

function Capture-Screenshot {
    param([string]$Path)

    $screenshot = Send-CdpCommand -Method "Page.captureScreenshot" -Params @{
        format = "png"
        fromSurface = $true
    }
    [IO.File]::WriteAllBytes($Path, [Convert]::FromBase64String($screenshot.result.data))
}

function Capture-Viewport {
    param(
        [int]$Width,
        [int]$Height
    )

    Send-CdpCommand -Method "Emulation.setDeviceMetricsOverride" -Params @{
        width = $Width
        height = $Height
        deviceScaleFactor = 1
        mobile = $false
    } | Out-Null

    Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = "globalThis.__hudManaShapesEvidence = null"
        returnByValue = $true
    } | Out-Null

    $cacheBust = [Guid]::NewGuid().ToString("N")
    $separator = if ($Url.Contains("?")) { "&" } else { "?" }
    $viewportUrl = "$Url${separator}viewport=${Width}x${Height}&capture=$cacheBust"
    Send-CdpCommand -Method "Page.navigate" -Params @{ url = $viewportUrl } | Out-Null

    $readyDeadline = (Get-Date).AddSeconds($ReadyTimeoutSeconds)
    $state = $null
    do {
        Start-Sleep -Milliseconds 500
        $state = Get-HarnessState
        if ($null -ne $state -and $state.ready_for_capture -eq $true) {
            break
        }
    } while ((Get-Date) -lt $readyDeadline)

    if ($null -eq $state -or $state.ready_for_capture -ne $true) {
        $timeoutBaseName = "hud-011-mana-shapes-${Width}x${Height}-timeout"
        $timeoutScreenshot = Join-Path $outputDir "$timeoutBaseName.png"
        $timeoutDiagnosticsPath = Join-Path $outputDir "$timeoutBaseName-diagnostics.json"
        Capture-Screenshot -Path $timeoutScreenshot
        $diagnostics = [ordered]@{
            viewport = [ordered]@{ width = $Width; height = $Height }
            url = $viewportUrl
            state = $state
            title = Get-RuntimeValue -Expression "document.title"
            evidenceJson = Get-RuntimeValue -Expression "JSON.stringify(globalThis.__hudManaShapesEvidence ?? null)"
            bodyText = Get-RuntimeValue -Expression "document.body?.innerText ?? ''"
            cdpEvents = Get-CdpEventDiagnostics
            timeoutScreenshot = "$relativeOutputDir/$timeoutBaseName.png"
            capturedAt = (Get-Date).ToUniversalTime().ToString("o")
        }
        ($diagnostics | ConvertTo-Json -Depth 80) + "`n" |
            Set-Content -LiteralPath $timeoutDiagnosticsPath -Encoding ascii
        throw "HUD-011 harness did not become ready at ${Width}x${Height} within $ReadyTimeoutSeconds seconds."
    }

    Start-Sleep -Seconds 1

    $baseName = "hud-011-mana-shapes-${Width}x${Height}"
    $colorPath = Join-Path $outputDir "$baseName.png"
    $grayPath = Join-Path $outputDir "$baseName-grayscale.png"
    Capture-Screenshot -Path $colorPath

    Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = "document.documentElement.style.filter = 'grayscale(1)'; document.body.style.filter = 'grayscale(1)'; const canvas = document.querySelector('canvas'); if (canvas) { canvas.style.filter = 'grayscale(1)'; }"
        returnByValue = $true
    } | Out-Null
    Start-Sleep -Milliseconds 250
    Capture-Screenshot -Path $grayPath

    [ordered]@{
        viewport = [ordered]@{ width = $Width; height = $Height }
        url = $viewportUrl
        fixture = $state
        colorScreenshot = "$relativeOutputDir/$baseName.png"
        grayscaleScreenshot = "$relativeOutputDir/$baseName-grayscale.png"
        colorBytes = (Get-Item -LiteralPath $colorPath).Length
        grayscaleBytes = (Get-Item -LiteralPath $grayPath).Length
    }
}

$profileDir = Join-Path ([IO.Path]::GetTempPath()) ("hud-011-cdp-" + [Guid]::NewGuid().ToString("N"))
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
        "--disable-renderer-backgrounding",
        "--autoplay-policy=no-user-gesture-required",
        "--enable-webgl",
        "--enable-webgl2",
        "--ignore-gpu-blocklist",
        "--disable-gpu-sandbox",
        "--enable-unsafe-swiftshader",
        "--use-gl=angle",
        "--use-angle=swiftshader",
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

    $captures = @(
        Capture-Viewport -Width 1366 -Height 768
        Capture-Viewport -Width 1920 -Height 1080
    )

    $chromeVersion = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($ChromePath).ProductVersion
    $result = [ordered]@{
        url = $Url
        captureTool = "PowerShell Chrome DevTools Protocol"
        chromePath = $ChromePath
        chromeVersion = $chromeVersion
        summaryPath = $summaryRelativePath
        captures = $captures
        verdict = [ordered]@{
            artifactProduction = "PASS"
            colorCaptureCount = 2
            grayscaleCaptureCount = 2
            fixtureValues = "current_mana=6, mana_cap=10, reserve_mana=2"
        }
        capturedAt = (Get-Date).ToUniversalTime().ToString("o")
    }

    ($result | ConvertTo-Json -Depth 80) + "`n" | Set-Content -LiteralPath $summaryPath -Encoding ascii
    $result | ConvertTo-Json -Depth 80
    exit 0
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
