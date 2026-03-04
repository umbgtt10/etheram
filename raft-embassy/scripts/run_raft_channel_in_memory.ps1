$ErrorActionPreference = 'Continue'
Set-Location "$PSScriptRoot\.."
Write-Host "Running Raft Embassy with Channel Transport and In-Memory Storage..."
cargo run --release --no-default-features --features "channel-transport,in-memory-storage,channel-external-interface"
