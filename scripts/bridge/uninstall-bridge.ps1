[CmdletBinding()]
param(
    [string]$TaskName = "CodexLineBridge",
    [switch]$RemoveLoopScript
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$bridgeDir = Join-Path $HOME ".codex/codexline"
$loopScriptPath = Join-Path $bridgeDir "bridge-loop.ps1"

if (Get-Command schtasks -ErrorAction SilentlyContinue) {
    $null = & schtasks /Delete /F /TN $TaskName 2>$null
}

$escaped = [System.Text.RegularExpressions.Regex]::Escape($loopScriptPath)
$candidates = Get-CimInstance Win32_Process -Filter "Name = 'powershell.exe'" |
    Where-Object { $_.CommandLine -match $escaped }

foreach ($proc in $candidates) {
    try {
        Stop-Process -Id $proc.ProcessId -Force -ErrorAction SilentlyContinue
    } catch {
    }
}

if ($RemoveLoopScript -and (Test-Path $loopScriptPath)) {
    Remove-Item -Path $loopScriptPath -Force
}

Write-Host "[codexline-bridge] Uninstalled scheduled task and stopped running bridge processes." -ForegroundColor Green
if ($RemoveLoopScript) {
    Write-Host "Removed loop script: $loopScriptPath"
}
Write-Host "If you added prompt hooks manually, remove them from your shell profile."
