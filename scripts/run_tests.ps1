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

$env:RUSTFLAGS = "-D warnings"

Invoke-Step "Formatting" { cargo fmt }

Invoke-Step "Clippy" { cargo clippy --workspace -- -D warnings }

foreach ($crate in @("etheram-core", "embassy-core", "etheram-node", "raft-node")) {
    Invoke-Step "no_std check $crate" { cargo check -p $crate --no-default-features }
}

foreach ($crate in @("etheram-core", "etheram-node", "etheram-validation", "etheram-desktop", "etheram-node-process", "raft-node", "raft-validation")) {
    Invoke-Step "Testing $crate" { cargo nextest run -p $crate }
}

Write-Host "`nAll tests passed!" -ForegroundColor Green
exit 0
