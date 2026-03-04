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

foreach ($crate in @("etheram-etheram", "etheram-etheram-variants", "etheram-etheram-validation")) {
    Invoke-Step "Testing $crate" { cargo nextest run -p $crate }
}

Invoke-Step "Checking etheram-etheram-variants (no_std gate)" {
    cargo check -p etheram-etheram-variants --no-default-features
}

Invoke-Step "Running etheram-etheram-embassy (channel-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_channel_in_memory.ps1"
}

Invoke-Step "Running etheram-etheram-embassy (udp-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_udp_semihosting.ps1"
}

Write-Host "`nFull success!" -ForegroundColor Green
exit 0
