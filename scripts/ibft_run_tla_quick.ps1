$ErrorActionPreference = "Stop"

$tla2tools = Get-ChildItem -Path "$env:USERPROFILE\.vscode\extensions" -Recurse -Filter "tla2tools.jar" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notlike "*\out\*" } |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $tla2tools) {
    Write-Host "tla2tools.jar not found. Install the TLA+ VS Code extension." -ForegroundColor Red
    exit 1
}

$specDir = Join-Path $PSScriptRoot "..\specs\ibft"

function Run-TLCCheck {
    param([string]$Label, [string]$Module)
    Write-Host "`n$Label..." -ForegroundColor Cyan
    Push-Location $specDir
    try {
        java -jar $tla2tools -config IBFTConsensus.cfg -workers auto -nowarning $Module
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAILED (exit $LASTEXITCODE)" -ForegroundColor Red
            exit $LASTEXITCODE
        }
        Write-Host "Passed." -ForegroundColor Green
    } finally {
        Pop-Location
    }
}

Write-Host "IBFT quick model checks" -ForegroundColor White
Write-Host "  Check 1: Byzantine resistance (F=1, round 0 only)" -ForegroundColor DarkCyan
Write-Host "    Proves Agreement/LockConsistency/CommitImpliesPrepareQuorum cannot be violated" -ForegroundColor DarkCyan
Write-Host "    by a Byzantine validator injecting arbitrary Prepare / Commit messages." -ForegroundColor DarkCyan
Write-Host "  Check 2: Honest model with view changes (MaxRound=1)" -ForegroundColor DarkCyan
Write-Host "    Proves all four invariants including locked-block re-propose safety." -ForegroundColor DarkCyan

Run-TLCCheck "Check 1 — Byzantine commit-phase attack (MaxRound=0, ByzValidators={3})" "MC_IBFTConsensus_Quick"
Run-TLCCheck "Check 2 — Honest + view-change coverage (MaxRound=1, ByzValidators={})" "MC_IBFTConsensus_CI"

Write-Host "`nAll IBFT quick model checks passed." -ForegroundColor Green
exit 0
