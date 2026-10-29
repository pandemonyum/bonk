# Build web del gioco in dist/ (P1): wasm-release per wasm32, wasm-bindgen alla versione del
# Cargo.lock, wasm-opt se installato, poi pagina, asset e dati. Uso: ./scripts/build-web.ps1
# Prova: python -m http.server 8080 -d dist  e apri http://localhost:8080 (mai file://).
# niente ErrorActionPreference = Stop: su PowerShell 5.1 lo stderr di cargo (anche solo i
# messaggi di avanzamento) diventerebbe un errore; si controllano gli exit code
Set-Location (Split-Path $PSScriptRoot -Parent)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

$lock = Get-Content Cargo.lock -Raw
$want = [regex]::Match($lock, 'name = "wasm-bindgen"\r?\nversion = "([^"]+)"').Groups[1].Value
$have = (wasm-bindgen --version) -replace 'wasm-bindgen ', ''
if ($have -ne $want) {
    throw "wasm-bindgen $have, il Cargo.lock vuole ${want}: cargo install wasm-bindgen-cli --version ${want} --locked"
}

# versione di asset e dati (contenuto dei file): il gioco li cerca in cartelle con questo nome,
# così il browser non usa dalla cache quelli di una build vecchia (src/web.rs)
$sha = [Security.Cryptography.SHA256]::Create()
$files = Get-ChildItem assets, game-data -Recurse -File | Sort-Object FullName
$buffer = New-Object IO.MemoryStream
foreach ($f in $files) {
    $rel = [Text.Encoding]::UTF8.GetBytes(($f.FullName.Substring((Get-Location).Path.Length) -replace '\\', '/'))
    $buffer.Write($rel, 0, $rel.Length)
    $bytes = [IO.File]::ReadAllBytes($f.FullName)
    $buffer.Write($bytes, 0, $bytes.Length)
}
$version = ([BitConverter]::ToString($sha.ComputeHash($buffer.ToArray())) -replace '-', '').Substring(0, 10).ToLower()
$env:PMS_WEB_VERSION = $version
Write-Host "==> cargo build (wasm-release, asset e dati $version)"
cargo build --profile wasm-release --target wasm32-unknown-unknown --bin pixel-monster-survivor
if ($LASTEXITCODE -ne 0) { throw "build wasm fallita" }
$raw = "target/wasm32-unknown-unknown/wasm-release/pixel-monster-survivor.wasm"

# dist/ è generata: si ricrea da zero (è in .gitignore)
if (Test-Path dist) { Remove-Item dist -Recurse -Force -ErrorAction Stop }
New-Item -ItemType Directory dist -ErrorAction Stop | Out-Null

Write-Host "==> wasm-bindgen"
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section --out-dir dist --out-name game $raw
if ($LASTEXITCODE -ne 0) { throw "wasm-bindgen fallito" }

$wasm = "dist/game_bg.wasm"
if (Get-Command wasm-opt -ErrorAction SilentlyContinue) {
    Write-Host "==> wasm-opt -Os"
    wasm-opt -Os --strip-debug -o "$wasm.opt" $wasm
    if ($LASTEXITCODE -ne 0) { throw "wasm-opt fallito" }
    Move-Item -Force "$wasm.opt" $wasm
} else {
    Write-Host "(wasm-opt non installato: .wasm non ottimizzato oltre il profilo)"
}

Write-Host "==> pagina, asset e dati"
$hash = (Get-FileHash $wasm -Algorithm SHA256).Hash.Substring(0, 12).ToLower()
# in UTF-8 senza BOM in lettura e scrittura (PowerShell 5.1 di suo legge in ANSI)
$utf8 = New-Object System.Text.UTF8Encoding($false)
$page = [IO.File]::ReadAllText((Resolve-Path web/index.html), $utf8).Replace("__VERSION__", $hash)
[IO.File]::WriteAllText((Join-Path (Resolve-Path dist) "index.html"), $page, $utf8)
Copy-Item assets "dist/assets-$version" -Recurse -ErrorAction Stop
Copy-Item game-data "dist/game-data-$version" -Recurse -ErrorAction Stop

# dimensioni: grezza, dopo bindgen/opt, compressa gzip (quello che il browser scarica con un
# server che comprime)
$size = { param($p) "{0:N1} MiB" -f ((Get-Item $p).Length / 1MB) }
$gz = "$env:TEMP/game_bg.wasm.gz"
$in = [IO.File]::OpenRead((Resolve-Path $wasm))
$out = [IO.File]::Create($gz)
$zip = New-Object IO.Compression.GZipStream($out, [IO.Compression.CompressionLevel]::Optimal)
$in.CopyTo($zip); $zip.Dispose(); $out.Dispose(); $in.Dispose()
Write-Host ("wasm: grezzo {0}, in dist {1}, gzip {2}" -f (& $size $raw), (& $size $wasm), (& $size $gz))
Remove-Item $gz
$total = (Get-ChildItem dist -Recurse -File | Measure-Object Length -Sum).Sum / 1MB
$biggest = Get-ChildItem dist -Recurse -File | Sort-Object Length -Descending | Select-Object -First 1
Write-Host ("dist: {0:N1} MiB in tutto, il file piu grande {1} ({2:N1} MiB)" -f $total, $biggest.Name, ($biggest.Length / 1MB))
if ($biggest.Length -gt 25MB) { throw "un file di dist supera 25 MiB" }
Write-Host "OK: dist/ pronta (python -m http.server 8080 -d dist)"
