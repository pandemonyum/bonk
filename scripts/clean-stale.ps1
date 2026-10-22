# Ripulisce le copie vecchie del nostro crate in target/ (eseguibili e .pdb di gioco e test).
# Cargo ne crea una nuova a ogni cambio di dipendenze, feature o profilo e non cancella le
# precedenti. Bevy e le altre dipendenze restano: niente ricompilazione lunga.
# Uso: ./scripts/clean-stale.ps1          Pulizia totale (ricompila tutto, ~20 min): cargo clean
Set-Location (Split-Path -Parent $PSScriptRoot)

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path $cargoBin)) {
    $env:Path = "$cargoBin;$env:Path"
}

function TargetGB {
    if (-not (Test-Path target)) { return 0 }
    $bytes = (Get-ChildItem target -Recurse -File -Force -ErrorAction SilentlyContinue |
        Measure-Object Length -Sum).Sum
    [math]::Round($bytes / 1GB, 2)
}

$before = TargetGB
cargo clean -p pixel-monster-survivor
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$after = TargetGB
Write-Host "target/: $before GB -> $after GB (liberati $([math]::Round($before - $after, 2)) GB)"
