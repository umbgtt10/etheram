$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Push-Location $repoRoot

try {
    $env:ETHERAM_NODE_PROCESS_BIN = Join-Path $repoRoot "target\debug\etheram-node-process.exe"
    $env:ETHERAM_DESKTOP_NODE_STEP_LIMIT = "0"
    $env:ETHERAM_NODE_PROCESS_TRANSPORT_BACKEND = "grpc"
    $clusterConfig = Join-Path $repoRoot "etheram-desktop\cluster.local.toml"

    Write-Host "Starting EtheRAM desktop GUI..."
    Write-Host "Node process binary: $env:ETHERAM_NODE_PROCESS_BIN"
    Write-Host "Node process step limit: $env:ETHERAM_DESKTOP_NODE_STEP_LIMIT"
    Write-Host "Using cluster config: $clusterConfig"

    cargo run -p etheram-desktop -- --gui "$clusterConfig"
}
finally {
    Pop-Location
}
