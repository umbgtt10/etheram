$ErrorActionPreference = 'Continue'
Set-Location "$PSScriptRoot\.."
Write-Host "Running EtheRAM Embassy with UDP Transport and Semihosting Storage..."
cargo run --release --no-default-features --features "udp-transport,semihosting-storage,udp-external-interface"
