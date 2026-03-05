$ErrorActionPreference = "Stop"

$tla2tools = Get-ChildItem -Path "$env:USERPROFILE\.vscode\extensions" -Recurse -Filter "tla2tools.jar" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notlike "*\out\*" } |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $tla2tools) {
    Write-Host "tla2tools.jar not found. Install the TLA+ VS Code extension." -ForegroundColor Red
    exit 1
}

$specDir = Join-Path $PSScriptRoot "..\specs\raft"

Write-Host "Raft full model check (N=3, MaxTerm=3)..." -ForegroundColor Cyan
Write-Host "Explores three full election terms including cascaded step-downs and split-vote scenarios." -ForegroundColor DarkCyan
Write-Host "This performs a complete BFS over a large state space and may take 10-30 minutes." -ForegroundColor Yellow

Push-Location $specDir
try {
    java -jar $tla2tools -config RaftConsensus.cfg -workers auto -nowarning MC_RaftConsensus
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nModel check FAILED (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

Write-Host "`nRaft full model check passed." -ForegroundColor Green
exit 0
