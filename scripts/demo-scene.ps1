<#
.SYNOPSIS
    Spins up throwaway listeners on recognisable developer ports.

.DESCRIPTION
    Used for screenshots and manual testing. Two reasons this exists rather than
    "just take a screenshot of your machine":

      * A real desktop listing is mostly corporate agents, VPN clients and
        svchost, plus internal IP addresses. None of that belongs in a public
        README.
      * The scene is reproducible, so the next release's screenshots look like
        the last one's.

    Every listener binds to 127.0.0.1 only and holds the port until stopped, so
    nothing is reachable from the network and nothing outlives the script.

.PARAMETER Collateral
    Also start one process holding three ports at once, to demonstrate the
    warning Portify shows before a kill takes several ports down together.

.PARAMETER Stop
    Terminate everything this script started.

.EXAMPLE
    .\scripts\demo-scene.ps1 -Collateral
    .\scripts\demo-scene.ps1 -Stop

.NOTES
    Requires Node on PATH — it is only used as a convenient process that holds a
    socket open and shows up under a name a developer recognises.
#>
[CmdletBinding()]
param(
    [switch]$Collateral,
    [switch]$Stop
)

$ErrorActionPreference = 'Stop'

# Ports chosen because Portify's service catalogue knows them all, so the
# listing reads as a plausible development machine.
$ScenePorts = @(3000, 5173, 8080, 9229)
$SharedPorts = @(7001, 7002, 7003)
$StateFile = Join-Path $env:TEMP 'portify-demo-scene.json'

function Start-Listener {
    param([int[]]$Port)

    $binds = ($Port | ForEach-Object { "require('net').createServer().listen($_,'127.0.0.1')" }) -join ';'
    # The interval keeps the event loop alive without burning CPU.
    $script = "$binds;setInterval(()=>{},1<<30)"

    (Start-Process node -PassThru -WindowStyle Hidden -ArgumentList @('-e', $script)).Id
}

if ($Stop) {
    if (-not (Test-Path $StateFile)) {
        Write-Host 'No demo scene is running.'
        return
    }

    foreach ($processId in (Get-Content $StateFile | ConvertFrom-Json)) {
        Stop-Process -Id $processId -Force -ErrorAction SilentlyContinue
    }
    Remove-Item $StateFile
    Write-Host 'Demo scene stopped.'
    return
}

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    throw 'Node is not on PATH. Install it, or edit this script to use another runtime.'
}

if (Test-Path $StateFile) {
    Write-Warning 'A demo scene is already running. Stopping it first.'
    & $PSCommandPath -Stop
}

$started = @()
foreach ($port in $ScenePorts) {
    $started += Start-Listener -Port $port
}
if ($Collateral) {
    $started += Start-Listener -Port $SharedPorts
}

# ConvertTo-Json collapses a single-element array to a scalar; the wrapper keeps
# the shape stable for -Stop.
ConvertTo-Json @($started) | Set-Content $StateFile

Start-Sleep -Milliseconds 400
Write-Host "Listening on $($ScenePorts -join ', ')" -ForegroundColor Green
if ($Collateral) {
    Write-Host "One process holding $($SharedPorts -join ', ') together" -ForegroundColor Green
}
Write-Host 'Stop with: .\scripts\demo-scene.ps1 -Stop'
