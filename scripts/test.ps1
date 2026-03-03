$failed = @()

Write-Host "Formatting..." -ForegroundColor Cyan
cargo fmt

$crates = @(
    "barechain-etheram",
    "barechain-etheram-variants",
    "barechain-etheram-validation"
)

foreach ($crate in $crates) {
    Write-Host "Testing $crate..." -ForegroundColor Cyan
    cargo nextest run -p $crate
    if ($LASTEXITCODE -ne 0) {
        $failed += $crate
    }
}

Write-Host "Checking barechain-etheram-variants (no_std gate)..." -ForegroundColor Cyan
cargo check -p barechain-etheram-variants --no-default-features
if ($LASTEXITCODE -ne 0) {
    $failed += "barechain-etheram-variants (no_std gate)"
}

Write-Host "Running barechain-etheram-embassy (channel-transport)..." -ForegroundColor Cyan
powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_channel_in_memory.ps1"
if ($LASTEXITCODE -ne 0) {
    $failed += "barechain-etheram-embassy (channel-transport)"
}

Write-Host "Running barechain-etheram-embassy (udp-transport)..." -ForegroundColor Cyan
powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_udp_semihosting.ps1"
if ($LASTEXITCODE -ne 0) {
    $failed += "barechain-etheram-embassy (udp-transport)"
}

if ($failed.Count -gt 0) {
    Write-Host "`nFailed:" -ForegroundColor Red
    foreach ($crate in $failed) {
        Write-Host "  - $crate" -ForegroundColor Red
    }
    exit 1
} else {
    Write-Host "`nAll tests passed." -ForegroundColor Green
    exit 0
}
