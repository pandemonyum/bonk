# Gate qualità: formattazione, clippy senza warning, test, validazione asset.
# Uso: ./scripts/check.ps1        Esce con codice != 0 al primo controllo fallito.
# Niente $ErrorActionPreference = 'Stop': in PowerShell 5.1 trasformerebbe in errore fatale
# il normale output di cargo su stderr quando viene rediretto. Contano gli exit code, e un
# comando non trovato conta come fallimento (non lascia l'exit code del passo precedente).
Set-Location (Split-Path -Parent $PSScriptRoot)

$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and (Test-Path $cargoBin)) {
    $env:Path = "$cargoBin;$env:Path"
}
# Su Windows spesso c'è solo il launcher `py`.
$python = if (Get-Command python -ErrorAction SilentlyContinue) { 'python' } else { 'py' }

$steps = @(
    @{ Name = 'fmt';     Cmd = { cargo fmt --all -- --check } },
    @{ Name = 'clippy';  Cmd = { cargo clippy --all-targets --locked -- -D warnings } },
    @{ Name = 'test';    Cmd = { cargo test --locked } },
    @{ Name = 'assets';  Cmd = { & $python scripts/validate-assets.py } }
)

foreach ($step in $steps) {
    Write-Host "==> $($step.Name)" -ForegroundColor Cyan
    $global:LASTEXITCODE = $null
    try {
        & $step.Cmd
    } catch {
        Write-Host "FALLITO: $($step.Name): $_" -ForegroundColor Red
        exit 1
    }
    $code = $global:LASTEXITCODE
    if ($null -eq $code -or $code -ne 0) {
        Write-Host "FALLITO: $($step.Name) (exit $code)" -ForegroundColor Red
        if ($null -eq $code -or $code -eq 0) { exit 1 } else { exit $code }
    }
}
Write-Host "OK: tutti i controlli passati" -ForegroundColor Green
