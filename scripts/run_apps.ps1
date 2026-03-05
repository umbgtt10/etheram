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

Invoke-Step "Running etheram-embassy (channel-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_channel_in_memory.ps1"
}

Invoke-Step "Running etheram-embassy (udp-transport)" {
    powershell -File "$PSScriptRoot\..\etheram-embassy\scripts\run_udp_semihosting.ps1"
}

Invoke-Step "Running raft-embassy (channel-transport)" {
    powershell -File "$PSScriptRoot\..\raft-embassy\scripts\run_raft_channel_in_memory.ps1"
}

Invoke-Step "Running raft-embassy (udp-transport)" {
    powershell -File "$PSScriptRoot\..\raft-embassy\scripts\run_raft_udp_semihosting.ps1"
}

Write-Host "`nAll apps passed!" -ForegroundColor Green
exit 0
