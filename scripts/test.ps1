function Invoke-Step {
    param([string]$Label, [scriptblock]$Command)
    Write-Host "$Label..." -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "`nFailed: $Label" -ForegroundColor Red
        exit 1
    }
}

Invoke-Step "Formatting" { cargo fmt }

foreach ($crate in @("etheram-etheram", "etheram-etheram-variants", "etheram-etheram-validation", "raft-raft-node", "raft-raft-variants", "raft-raft-validation")) {
    Invoke-Step "Testing $crate" { cargo nextest run -p $crate }
}

foreach ($crate in @("etheram-etheram-variants", "raft-raft-node", "raft-raft-variants")) {
    Invoke-Step "no_std gate: $crate" { cargo check -p $crate --no-default-features }
}


Invoke-Step "Running etheram-etheram-embassy (channel-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_channel_in_memory.ps1"
}

Invoke-Step "Running etheram-etheram-embassy (udp-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_udp_semihosting.ps1"
}

Write-Host "`nFull success!" -ForegroundColor Green
exit 0
