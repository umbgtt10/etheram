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

foreach ($crate in @("barechain-etheram", "barechain-etheram-variants", "barechain-etheram-validation")) {
    Invoke-Step "Testing $crate" { cargo nextest run -p $crate }
}

Invoke-Step "Checking barechain-etheram-variants (no_std gate)" {
    cargo check -p barechain-etheram-variants --no-default-features
}

Invoke-Step "Running barechain-etheram-embassy (channel-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_channel_in_memory.ps1"
}

Invoke-Step "Running barechain-etheram-embassy (udp-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_udp_semihosting.ps1"
}

Write-Host "`nFull success!" -ForegroundColor Green
exit 0
