$ErrorActionPreference = "Stop"

$tla2tools = Get-ChildItem -Path "$env:USERPROFILE\.vscode\extensions" -Recurse -Filter "tla2tools.jar" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notlike "*\out\*" } |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $tla2tools) {
    Write-Host "tla2tools.jar not found. Install the TLA+ VS Code extension." -ForegroundColor Red
    exit 1
}

$specDir = Join-Path $PSScriptRoot "..\specs\ibft"

Write-Host "IBFT full model check (Byzantine, MaxRound=2)..." -ForegroundColor Cyan
Write-Host "Explores two full view-change cycles with one Byzantine validator." -ForegroundColor DarkCyan
Write-Host "This performs a complete BFS over a large state space and may take 30+ minutes." -ForegroundColor Yellow

Push-Location $specDir
try {
    java -jar $tla2tools -config IBFTConsensus.cfg -workers auto -nowarning MC_IBFTConsensus
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nModel check FAILED (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

Write-Host "`nIBFT full model check passed." -ForegroundColor Green
exit 0
