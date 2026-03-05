$ErrorActionPreference = "Stop"

function Invoke-Step {
    param([string]$Label, [scriptblock]$Command)
    Write-Host "$Label..." -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed: $Label (exit code $LASTEXITCODE)" -ForegroundColor Red
        exit 1
    }
}

Invoke-Step "Tests" { powershell -File "$PSScriptRoot\run_tests.ps1" }

Invoke-Step "Apps" { powershell -File "$PSScriptRoot\run_apps.ps1" }

Write-Host "`nFull success!" -ForegroundColor Green
exit 0
