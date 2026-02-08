[CmdletBinding()]
param(
    [int]$RefreshMs = 800,
    [string]$TaskName = "CodexLineBridge",
    [switch]$SkipStart
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Assert-CommandAvailable {
    param([Parameter(Mandatory = $true)][string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command '$Name' is not available in PATH."
    }
}

function Stop-BridgeProcesses {
    param([Parameter(Mandatory = $true)][string]$LoopScriptPath)

    $escaped = [System.Text.RegularExpressions.Regex]::Escape($LoopScriptPath)
    $candidates = Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" |
        Where-Object { $_.CommandLine -match $escaped }

    foreach ($proc in $candidates) {
        try {
            Stop-Process -Id $proc.ProcessId -Force -ErrorAction SilentlyContinue
        } catch {
        }
    }
}

if ($RefreshMs -lt 200) {
    throw "RefreshMs must be >= 200 ms."
}

Assert-CommandAvailable -Name "codexline"
Assert-CommandAvailable -Name "schtasks"

$bridgeDir = Join-Path $HOME ".codex/codexline"
$loopScriptPath = Join-Path $bridgeDir "bridge-loop.ps1"

New-Item -ItemType Directory -Path $bridgeDir -Force | Out-Null

$loopScriptLines = @(
    '$cacheDir = Join-Path $env:LOCALAPPDATA "codexline"',
    '$linePath = Join-Path $cacheDir "line.txt"',
    '$tmpPath = Join-Path $cacheDir "line.txt.tmp"',
    '',
    'New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null',
    '',
    'while ($true) {',
    '    try {',
    '        $line = & codexline --plain 2>$null',
    '        if ($LASTEXITCODE -eq 0 -and $line) {',
    '            Set-Content -Path $tmpPath -Value $line -NoNewline -Encoding utf8',
    '            Move-Item -Path $tmpPath -Destination $linePath -Force',
    '        }',
    '    } catch {',
    '    }',
    '',
    "    Start-Sleep -Milliseconds $RefreshMs",
    '}'
)
$loopScript = [string]::Join("`n", $loopScriptLines)
Set-Content -Path $loopScriptPath -Value $loopScript -Encoding utf8

$taskCommand = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "' + $loopScriptPath + '"'
$null = & schtasks /Create /F /TN $TaskName /SC ONLOGON /TR $taskCommand

Stop-BridgeProcesses -LoopScriptPath $loopScriptPath

if (-not $SkipStart) {
    Start-Process -FilePath "powershell.exe" -WindowStyle Hidden -ArgumentList @(
        "-NoProfile",
        "-ExecutionPolicy", "Bypass",
        "-File", $loopScriptPath
    ) | Out-Null
}

$lineFile = Join-Path $env:LOCALAPPDATA "codexline/line.txt"

Write-Host "[codexline-bridge] Installed." -ForegroundColor Green
Write-Host "Task Name : $TaskName"
Write-Host "Loop Script: $loopScriptPath"
Write-Host "Cache File : $lineFile"
Write-Host ""
Write-Host 'PowerShell prompt snippet (append to your $PROFILE):'
Write-Host 'function global:prompt {'
Write-Host '  $lineFile = Join-Path $env:LOCALAPPDATA "codexline/line.txt"'
Write-Host '  if (Test-Path $lineFile) {'
Write-Host '    $line = Get-Content -Raw $lineFile'
Write-Host '    if ($line) { Write-Host $line -ForegroundColor DarkCyan }'
Write-Host '  }'
Write-Host '  "PS $($executionContext.SessionState.Path.CurrentLocation)> "'
Write-Host '}'
