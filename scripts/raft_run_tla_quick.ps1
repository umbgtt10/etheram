$ErrorActionPreference = "Stop"

$tla2tools = Get-ChildItem -Path "$env:USERPROFILE\.vscode\extensions" -Recurse -Filter "tla2tools.jar" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notlike "*\out\*" } |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $tla2tools) {
    Write-Host "tla2tools.jar not found. Install the TLA+ VS Code extension." -ForegroundColor Red
    exit 1
}

$specDir = Join-Path $PSScriptRoot "..\specs\raft"

function Run-TLCCheck {
    param([string]$Label, [string]$Module)
    Write-Host "`n$Label..." -ForegroundColor Cyan
    Push-Location $specDir
    try {
        java -jar $tla2tools -config RaftConsensus.cfg -workers auto -nowarning $Module
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAILED (exit $LASTEXITCODE)" -ForegroundColor Red
            exit $LASTEXITCODE
        }
        Write-Host "Passed." -ForegroundColor Green
    } finally {
        Pop-Location
    }
}

Write-Host "Raft quick model checks" -ForegroundColor White
Write-Host "  Check 1: Log safety + election safety, single term (N=3, MaxTerm=1, MaxEntries=1)" -ForegroundColor DarkCyan
Write-Host "    Verifies ElectionSafety, VoteOnce, LeaderTermOK, LogSafety, LeaderCompleteness in the minimal case." -ForegroundColor DarkCyan
Write-Host "  Check 2: Log safety + election safety, two terms (N=3, MaxTerm=2, MaxEntries=1)" -ForegroundColor DarkCyan
Write-Host "    Covers cross-term log consistency, stale-leader scenarios, and cascaded step-downs (~283K states)." -ForegroundColor DarkCyan

Run-TLCCheck "Check 1 - Single-term log + election safety (MaxTerm=1, ~1175 states)" "MC_RaftConsensus_Quick"
Run-TLCCheck "Check 2 - Two-term log + election safety (MaxTerm=2, ~283K states)" "MC_RaftConsensus_CI"

Write-Host "`nAll Raft quick model checks passed." -ForegroundColor Green
exit 0
