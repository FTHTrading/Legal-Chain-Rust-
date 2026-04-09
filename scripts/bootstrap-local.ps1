#!/usr/bin/env pwsh
# bootstrap-local.ps1 — Start a local LEGAL-CHAIN devnet with a single validator (Alice).
#
# Usage:
#   .\scripts\bootstrap-local.ps1          # Dev mode (single node)
#   .\scripts\bootstrap-local.ps1 -Chain local   # Local testnet spec
#
# Prerequisites:
#   - Rust toolchain 1.81+ with wasm32-unknown-unknown target
#   - cargo build --release completed

param(
    [ValidateSet("dev", "local")]
    [string]$Chain = "dev",
    [switch]$Purge
)

$ErrorActionPreference = "Stop"
$NodeBin = Join-Path $PSScriptRoot "..\target\release\legal-chain-node.exe"

if (-not (Test-Path $NodeBin)) {
    Write-Host "[*] Node binary not found. Building in release mode..." -ForegroundColor Yellow
    Push-Location (Join-Path $PSScriptRoot "..")
    cargo build --release
    Pop-Location
}

if ($Purge) {
    Write-Host "[*] Purging chain data for '$Chain'..." -ForegroundColor Cyan
    & $NodeBin purge-chain --chain $Chain -y
}

Write-Host "[*] Starting LEGAL-CHAIN node (chain=$Chain)..." -ForegroundColor Green
$args = @(
    "--chain", $Chain,
    "--alice",
    "--tmp",
    "--rpc-cors", "all",
    "--rpc-methods", "unsafe",
    "--rpc-port", "9944",
    "--port", "30333",
    "--validator"
)

if ($Chain -eq "dev") {
    $args += "--force-authoring"
}

& $NodeBin @args
