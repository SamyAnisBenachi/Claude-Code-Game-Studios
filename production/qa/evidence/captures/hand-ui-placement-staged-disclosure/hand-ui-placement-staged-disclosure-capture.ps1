param(
    [string]$ChromePath = "C:\Program Files\Google\Chrome\Application\chrome.exe",
    [int]$TrunkPort = 8081,
    [int]$DebugPort = 9224,
    [int]$ReadyTimeoutSeconds = 120
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..\..\..\..")
$clientRoot = Join-Path $repoRoot "client"
$captureRoot = Join-Path $repoRoot "production\qa\evidence\captures\hand-ui-placement-staged-disclosure"
$url = "http://127.0.0.1:$TrunkPort/"
$tracePath = Join-Path $captureRoot "hand-ui-placement-staged-disclosure-trace.json"
$relativeCaptureRoot = "production/qa/evidence/captures/hand-ui-placement-staged-disclosure"

if (-not (Test-Path -LiteralPath $ChromePath)) {
    throw "Chrome executable not found: $ChromePath"
}

New-Item -ItemType Directory -Force -Path $captureRoot | Out-Null

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
    } | ConvertTo-Json -Depth 80 -Compress
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

function Get-HarnessReportJson {
    $response = Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = "JSON.stringify(globalThis.__handUiPlacementStagedDisclosure?.report ?? null)"
        returnByValue = $true
        awaitPromise = $false
    }

    $response.result.result.value
}

function Invoke-HarnessStepRender {
    param([string]$StepId)

    $escaped = $StepId.Replace("\", "\\").Replace("'", "\'")
    Send-CdpCommand -Method "Runtime.evaluate" -Params @{
        expression = "globalThis.__handUiPlacementStagedDisclosureRender('$escaped')"
        returnByValue = $false
        awaitPromise = $false
    } | Out-Null
}

function Capture-Screenshot {
    param([string]$Path)

    $screenshot = Send-CdpCommand -Method "Page.captureScreenshot" -Params @{
        format = "png"
        fromSurface = $true
    }
    [IO.File]::WriteAllBytes($Path, [Convert]::FromBase64String($screenshot.result.data))
}

$stepOrder = @(
    @{ id = "placement-entry"; file = "01-placement-entry.png" },
    @{ id = "card-selected"; file = "02-card-selected.png" },
    @{ id = "lane-cell-target-guidance"; file = "03-lane-cell-target-guidance.png" },
    @{ id = "valid-target-highlight"; file = "04-valid-target-highlight.png" },
    @{ id = "valid-stage"; file = "05-valid-stage.png" },
    @{ id = "reserve-current-split-adjustment"; file = "06-reserve-current-split-adjustment.png" },
    @{ id = "invalid-submit"; file = "07-invalid-submit.png" },
    @{ id = "correction-successful-submit"; file = "08-correction-successful-submit.png" }
)

$profileDir = Join-Path ([IO.Path]::GetTempPath()) ("hand-ui-014-cdp-" + [Guid]::NewGuid().ToString("N"))
$trunk = $null
$chrome = $null
$script:Socket = $null
$script:NextCdpId = 0
$script:CdpEvents = [System.Collections.Generic.List[object]]::new()

try {
    New-Item -ItemType Directory -Force -Path $profileDir | Out-Null

    $trunkStartInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $trunkStartInfo.FileName = "trunk"
    $trunkStartInfo.Arguments = ConvertTo-ArgumentString -Arguments @(
        "serve",
        "hand-ui-placement-staged-disclosure-harness.html",
        "--address",
        "127.0.0.1",
        "--port",
        "$TrunkPort"
    )
    $trunkStartInfo.WorkingDirectory = $clientRoot
    $trunkStartInfo.UseShellExecute = $false
    $trunkStartInfo.CreateNoWindow = $true
    $trunk = [System.Diagnostics.Process]::Start($trunkStartInfo)

    $trunkDeadline = (Get-Date).AddSeconds($ReadyTimeoutSeconds)
    do {
        if ($trunk.HasExited) {
            throw "Trunk exited before serving HAND-UI-014 harness. ExitCode=$($trunk.ExitCode)"
        }
        try {
            $response = Invoke-WebRequest -Uri $url -UseBasicParsing -Method Get -TimeoutSec 5
            if ($response.StatusCode -eq 200) {
                break
            }
        } catch {
            Start-Sleep -Milliseconds 500
        }
    } while ((Get-Date) -lt $trunkDeadline)

    if ((Get-Date) -ge $trunkDeadline) {
        throw "Trunk did not serve HAND-UI-014 harness within $ReadyTimeoutSeconds seconds."
    }

    $chromeArgs = @(
        "--headless=new",
        "--disable-dev-shm-usage",
        "--disable-background-timer-throttling",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--no-first-run",
        "--no-default-browser-check",
        "--hide-scrollbars",
        "--remote-debugging-port=$DebugPort",
        "--user-data-dir=$profileDir",
        "--window-size=1366,768",
        "about:blank"
    )

    $chromeStartInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $chromeStartInfo.FileName = $ChromePath
    $chromeStartInfo.Arguments = ConvertTo-ArgumentString -Arguments $chromeArgs
    $chromeStartInfo.UseShellExecute = $false
    $chromeStartInfo.CreateNoWindow = $true
    $chrome = [System.Diagnostics.Process]::Start($chromeStartInfo)

    $versionEndpoint = "http://127.0.0.1:$DebugPort/json/version"
    $chromeDeadline = (Get-Date).AddSeconds(20)
    do {
        try {
            $null = Invoke-CdpHttpJson -Endpoint $versionEndpoint
            break
        } catch {
            Start-Sleep -Milliseconds 250
        }
    } while ((Get-Date) -lt $chromeDeadline)

    $targetEndpoint = "http://127.0.0.1:$DebugPort/json/new?about%3Ablank"
    $target = Invoke-CdpHttpJson -Endpoint $targetEndpoint -Method "Put"
    $script:Socket = [System.Net.WebSockets.ClientWebSocket]::new()
    [void]$script:Socket.ConnectAsync([Uri]$target.webSocketDebuggerUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()

    Send-CdpCommand -Method "Page.enable" | Out-Null
    Send-CdpCommand -Method "Runtime.enable" | Out-Null
    Send-CdpCommand -Method "Log.enable" | Out-Null
    Send-CdpCommand -Method "Emulation.setDeviceMetricsOverride" -Params @{
        width = 1366
        height = 768
        deviceScaleFactor = 1
        mobile = $false
    } | Out-Null
    Send-CdpCommand -Method "Page.navigate" -Params @{ url = $url } | Out-Null

    $readyDeadline = (Get-Date).AddSeconds($ReadyTimeoutSeconds)
    $reportJson = $null
    do {
        Start-Sleep -Milliseconds 500
        $reportJson = Get-HarnessReportJson
        if (-not [string]::IsNullOrWhiteSpace($reportJson) -and $reportJson -ne "null") {
            $stateReady = Send-CdpCommand -Method "Runtime.evaluate" -Params @{
                expression = "globalThis.__handUiPlacementStagedDisclosure?.ready_for_capture === true"
                returnByValue = $true
                awaitPromise = $false
            }
            if ($stateReady.result.result.value -eq $true) {
                break
            }
        }
    } while ((Get-Date) -lt $readyDeadline)

    if ([string]::IsNullOrWhiteSpace($reportJson) -or $reportJson -eq "null") {
        throw "HAND-UI-014 harness did not publish a report within $ReadyTimeoutSeconds seconds."
    }

    $report = $reportJson | ConvertFrom-Json
    $artifacts = @()
    foreach ($step in $stepOrder) {
        Invoke-HarnessStepRender -StepId $step.id
        Start-Sleep -Milliseconds 250
        $path = Join-Path $captureRoot $step.file
        Capture-Screenshot -Path $path
        $artifacts += [ordered]@{
            step = $step.id
            path = "$relativeCaptureRoot/$($step.file)"
        }
    }

    $chromeVersion = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($ChromePath).ProductVersion
    $commit = (& git -C $repoRoot rev-parse HEAD).Trim()
    $branch = (& git -C $repoRoot rev-parse --abbrev-ref HEAD).Trim()
    $workingTreeStatus = @(& git -C $repoRoot status --short)
    $result = [ordered]@{
        story = "HAND-UI-014"
        qaCondition = "QA-COND-0005"
        url = $url
        viewport = [ordered]@{ width = 1366; height = 768 }
        uiScale = "100%"
        inputMethod = "deterministic mouse drag and click sequence"
        captureTool = "PowerShell Chrome DevTools Protocol + Trunk WASM harness"
        chromePath = $ChromePath
        chromeVersion = $chromeVersion
        sourceCommit = $commit
        sourceBranch = $branch
        sourceNote = "sourceCommit is HEAD at capture start; the final repair commit contains this harness, trace, and screenshots."
        workingTreeStatusAtCapture = $workingTreeStatus
        artifacts = $artifacts
        harnessReport = $report
        tracePath = "$relativeCaptureRoot/hand-ui-placement-staged-disclosure-trace.json"
        capturedAt = (Get-Date).ToUniversalTime().ToString("o")
    }

    ($result | ConvertTo-Json -Depth 100) + "`n" | Set-Content -LiteralPath $tracePath -Encoding ascii
    $result | ConvertTo-Json -Depth 100
} finally {
    if ($null -ne $script:Socket) {
        $script:Socket.Dispose()
    }
    if ($null -ne $chrome -and -not $chrome.HasExited) {
        $chrome.Kill()
        $chrome.WaitForExit()
    }
    if ($null -ne $trunk -and -not $trunk.HasExited) {
        $trunk.Kill()
        $trunk.WaitForExit()
    }
    if (Test-Path -LiteralPath $profileDir) {
        Remove-Item -LiteralPath $profileDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
