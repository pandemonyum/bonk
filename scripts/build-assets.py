#!/usr/bin/env python3
"""Genera gli asset pixel art di gioco da sorgenti testuali in assets-src/.

- Sprite: file assets-src/sprites/*.px (griglia di caratteri della palette).
- Fogli animati: un .px con `frame:`, `columns:` e `rows:` nell'intestazione e un blocco
  '@ <riga> <colonna>' per fotogramma (vedi assemble_sheet); diventano un solo PNG a griglia.
- Intestazione facoltativa `palette: <file>` (accanto al .px, stesso formato di palette.txt):
  l'asset usa la sua palette invece di quella del progetto; `license:` sostituisce la licenza
  di default nel manifest (per esempio un asset fornito dall'utente e ridisegnato qui).
- Icone e ritratti dell'interfaccia: file assets-src/ui/*.px, stesso formato.
- Tileset dell'arena: generato proceduralmente con seed fisso (riproducibile).
- Logo ui/logo.png: scritte dal font Pixelify Sans portate sulla sua griglia di pixel (niente
  antialiasing), con la palette e le decorazioni di assets-src/logo/ (vedi build_logo).
- Illustrazioni fornite dall'utente: assets-src/illustrations/<nome>.png|.webp ->
  assets/ui/<nome>.png a piena risoluzione, a 256 colori se non si vede la differenza (D-063).
- Scrive assets/manifest.json con dimensioni, sorgente e licenza di ogni PNG.

Moduli accanto a questo script: art_common (percorsi, palette, griglie), art_tileset,
art_logo, art_illustration.

Uso: python scripts/build-assets.py
Richiede: Python 3.10+ e Pillow (10.1+ per la misura frazionaria del font).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

# I moduli art_* stanno accanto a questo file (serve anche se lo si carica da un altro script).
sys.path.insert(0, str(Path(__file__).resolve().parent))

from art_common import OUT, ROOT, SRC, load_palette, render_grid  # noqa: E402
from art_illustration import build_illustrations  # noqa: E402
from art_logo import LOGO_OUT, build_logo  # noqa: E402
from art_tileset import TILE, TILE_NAMES, TILESET_SEED, build_tileset  # noqa: E402

LICENSE = "original, project-owned (generated from assets-src)"
# Sorgenti .px per cartella, con il tipo scritto nel manifest.
PX_KINDS = (("sprites", "sprite"), ("ui", "ui"))


def parse_px(path: Path) -> tuple[dict[str, str], list[str]]:
    header: dict[str, str] = {}
    rows: list[str] = []
    in_grid = False
    for raw in path.read_text(encoding="utf-8").splitlines():
        if not in_grid:
            line = raw.strip()
            if line == "---":
                in_grid = True
            elif line and not line.startswith("#"):
                key, _, value = line.partition(":")
                header[key.strip()] = value.strip()
        elif raw.strip():
            rows.append(raw.rstrip())
    if "out" not in header:
        sys.exit(f"{path.name}: manca 'out:' nell'intestazione")
    if "frame" in header:
        rows = assemble_sheet(path, header, rows)
    elif header.get("mirror") == "true":
        rows = [row + row[::-1] for row in rows]
    widths = {len(row) for row in rows}
    if len(widths) != 1:
        sys.exit(f"{path.name}: righe di lunghezza diversa {sorted(widths)}")
    return header, rows


def sheet_layout(header: dict[str, str]) -> tuple[int, int, int, list[str]]:
    """(larghezza, altezza del fotogramma, colonne, nomi delle righe) di un foglio animato."""
    fw, _, fh = header["frame"].partition("x")
    return int(fw), int(fh), int(header["columns"]), header["rows"].split()


def assemble_sheet(path: Path, header: dict[str, str], lines: list[str]) -> list[str]:
    """Foglio animato: un blocco per fotogramma, aperto da '@ <riga> <colonna>'.

    Le righe del foglio sono quelle di `rows:` nell'ordine scritto, le colonne vanno da 0 a
    `columns:` - 1. Nella griglia le righe che iniziano con '#' sono commenti e un '|' nel mezzo
    della riga (tra le colonne fw/2 - 1 e fw/2) è solo la guida del centro e viene tolto.
    Restituisce la griglia dell'intero foglio, fotogrammi affiancati.
    """
    fw, fh, columns, names = sheet_layout(header)
    frames: dict[tuple[str, int], list[str]] = {}
    current: tuple[str, int] | None = None
    for line in lines:
        if line.lstrip().startswith("#"):
            continue
        if line.startswith("@"):
            parts = line[1:].split()
            if (
                len(parts) != 2
                or parts[0] not in names
                or not parts[1].isdigit()
                or int(parts[1]) >= columns
            ):
                sys.exit(f"{path.name}: fotogramma non valido {line!r} (righe {names})")
            current = (parts[0], int(parts[1]))
            if current in frames:
                sys.exit(f"{path.name}: fotogramma ripetuto {line!r}")
            frames[current] = []
            continue
        if current is None:
            sys.exit(f"{path.name}: griglia prima del primo '@ <riga> <colonna>'")
        if "|" in line and (line.count("|") > 1 or line.index("|") != fw // 2):
            sys.exit(f"{path.name}: {current}: la guida '|' va solo dopo la colonna {fw // 2 - 1}")
        frames[current].append(line.replace("|", ""))
    grid: list[str] = []
    for name in names:
        band = [""] * fh
        for col in range(columns):
            rows = frames.get((name, col))
            if rows is None:
                sys.exit(f"{path.name}: manca il fotogramma {name} {col}")
            if len(rows) != fh or any(len(r) != fw for r in rows):
                found = f"{sorted({len(r) for r in rows})}x{len(rows)}"
                sys.exit(f"{path.name}: {name} {col} deve essere {fw}x{fh}, trovato {found}")
            band = [b + r for b, r in zip(band, rows)]
        grid += band
    extra = sorted(set(frames) - {(n, c) for n in names for c in range(columns)})
    if extra:
        sys.exit(f"{path.name}: fotogrammi fuori griglia {extra}")
    return grid


def main() -> None:
    palette = load_palette()
    manifest: list[dict] = []

    # Logo e illustrazioni prima di scrivere qualcosa: un'uscita contesa ferma il build.
    logo, logo_entry = build_logo()
    illustrations = build_illustrations()
    taken = {LOGO_OUT}
    for path, _, entry in illustrations:
        if path in taken:
            sys.exit(f"{entry['source']}: {path} è già l'uscita di un altro asset")
        taken.add(path)

    for folder, kind in PX_KINDS:
        for src in sorted((SRC / folder).glob("*.px")):
            header, rows = parse_px(src)
            if header["out"] in taken:
                sys.exit(f"{src.name}: {header['out']} è già l'uscita di un altro asset")
            taken.add(header["out"])
            own_palette = (src.parent / header["palette"]).resolve() if "palette" in header else None
            if own_palette and not own_palette.is_file():
                sys.exit(f"{src.name}: palette {header['palette']} non trovata")
            img = render_grid(rows, load_palette(own_palette) if own_palette else palette, src.name)
            dest = OUT / header["out"]
            dest.parent.mkdir(parents=True, exist_ok=True)
            img.save(dest)
            entry = {
                "path": header["out"],
                "kind": kind,
                "size": [img.width, img.height],
                "frames": 1,
            }
            if "frame" in header:
                fw, fh, columns, names = sheet_layout(header)
                entry["frame"] = [fw, fh]
                entry["frames"] = columns * len(names)
                entry["columns"] = columns
                entry["rows"] = names
                if "anchor" in header:
                    entry["anchor"] = header["anchor"]
            if own_palette:
                entry["palette"] = own_palette.relative_to(ROOT).as_posix()
            entry["source"] = src.relative_to(ROOT).as_posix()
            entry["license"] = header.get("license", LICENSE)
            manifest.append(entry)

    tileset = build_tileset(palette)
    tileset_path = "tiles/meadow_tileset.png"
    (OUT / "tiles").mkdir(parents=True, exist_ok=True)
    tileset.save(OUT / tileset_path)
    manifest.append(
        {
            "path": tileset_path,
            "kind": "tileset",
            "size": [tileset.width, tileset.height],
            "tile_size": TILE,
            "tiles": TILE_NAMES,
            "source": "scripts/build-assets.py (procedural, seed %d)" % TILESET_SEED,
            "license": LICENSE,
        }
    )

    (OUT / "ui").mkdir(parents=True, exist_ok=True)
    logo.save(OUT / LOGO_OUT)
    manifest.append(logo_entry)
    for path, data, entry in illustrations:
        (OUT / path).write_bytes(data)
        manifest.append(entry)

    (OUT / "manifest.json").write_text(
        json.dumps({"version": 1, "assets": manifest}, indent=2) + "\n", encoding="utf-8"
    )
    for entry in manifest:
        size = f"{entry['size'][0]}x{entry['size'][1]}"
        if "frame" in entry:
            size += f" ({entry['frames']} fotogrammi {entry['frame'][0]}x{entry['frame'][1]})"
        print(f"  {entry['path']:32} {size}")
    print(f"OK: {len(manifest)} asset generati in {OUT.relative_to(ROOT)}/")


if __name__ == "__main__":
    main()
