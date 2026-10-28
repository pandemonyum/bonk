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
- Scrive assets/manifest.json con dimensioni, sorgente e licenza di ogni PNG.

Uso: python scripts/build-assets.py
Richiede: Python 3.10+ e Pillow.
"""

from __future__ import annotations

import json
import random
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "assets-src"
OUT = ROOT / "assets"
TILE = 16
TILESET_SEED = 20261022
LICENSE = "original, project-owned (generated from assets-src)"
# Sorgenti .px per cartella, con il tipo scritto nel manifest.
PX_KINDS = (("sprites", "sprite"), ("ui", "ui"))

# Lati di un tile di bordo: il bit è acceso se il vicino in quella direzione è di un altro
# materiale (nord = riga 0 dell'immagine). Tenere allineato con src/world/tiles.rs.
EDGE_BITS = ((1, "n"), (2, "e"), (4, "s"), (8, "w"))


def edge_name(prefix: str, mask: int) -> str:
    return prefix + "_" + "".join(side for bit, side in EDGE_BITS if mask & bit)


# Ordine = indice nel tileset (TileData::from_tileset_index). Tenere allineato con
# src/world/tiles.rs e docs/ART_BIBLE.md. I tile nuovi vanno solo in coda.
TILE_NAMES = [
    "grass_a",
    "grass_b",
    "grass_tuft",
    "flowers_white",
    "flowers_pink",
    "dirt",
    "hedge",
    # v1: varianti decorative calpestabili
    "grass_c",
    "grass_clover",
    "flowers_yellow",
    "grass_pebbles",
    "grass_tall",
    # v1: bordi morbidi terra/erba (maschera 1..15) e siepe/erba (maschera 1..15)
    *[edge_name("dirt", m) for m in range(1, 16)],
    *[edge_name("hedge", m) for m in range(1, 16)],
]


def load_palette(path: Path | None = None) -> dict[str, tuple[int, int, int, int]]:
    """Palette del progetto, o quella propria di un asset (`palette:` nell'intestazione .px)."""
    path = path or SRC / "palette.txt"
    palette: dict[str, tuple[int, int, int, int]] = {".": (0, 0, 0, 0)}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        char, hexcode = line.split()[:2]
        if len(char) != 1 or char in palette:
            sys.exit(f"{path.name}: carattere non valido o duplicato: {char!r}")
        rgb = int(hexcode, 16)
        palette[char] = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF, 255)
    return palette


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


def render_grid(rows: list[str], palette, name: str) -> Image.Image:
    img = Image.new("RGBA", (len(rows[0]), len(rows)))
    for y, row in enumerate(rows):
        for x, char in enumerate(row):
            if char not in palette:
                sys.exit(f"{name}: carattere {char!r} non in palette (riga {y})")
            img.putpixel((x, y), palette[char])
    return img


# --- Tileset procedurale ---------------------------------------------------------------
# Luce da nord-ovest come negli sprite. Tutti i timbri si avvolgono sui bordi del tile
# (coordinate modulo 16), così un tile ripetuto non mostra cuciture.


def blank(fill: str) -> list[list[str]]:
    return [[fill] * TILE for _ in range(TILE)]


def put(grid, x: int, y: int, char: str) -> None:
    grid[y % TILE][x % TILE] = char


def stamp(grid, x: int, y: int, art: list[str]) -> None:
    """Timbro di caratteri (con '.' trasparente) con l'angolo in alto a sinistra in x, y."""
    for dy, row in enumerate(art):
        for dx, char in enumerate(row):
            if char != ".":
                put(grid, x + dx, y + dy, char)


# Timbri d'erba, stile del player (forme tozze, tre toni, luce da nord-ovest): ciuffi a tre e a
# due fili con la punta chiara e la base in ombra, macchie di sole tonde, zolle in ombra.
TUFT_3 = ["b.b.b", "AbAbA", ".AAA."]
TUFT_2 = ["b.b", "AbA", ".A."]
BLADE = ["b", "A"]
CLUMP = [".bb.", "bAAb", ".AA."]
SUN = [".bbb.", "bbbbb", ".bbb."]


def grass_base(rng: random.Random, tufts: int, blades: int, dashes: int) -> list[list[str]]:
    grid = blank("a")
    for _ in range(dashes):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), CLUMP)
    for _ in range(dashes // 2):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "B")
    for _ in range(blades):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), BLADE)
    for i in range(tufts):
        art = TUFT_3 if i % 2 == 0 else TUFT_2
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), art)
    return grid


def flower(grid, x: int, y: int, petal: str, shade: str, center: str) -> None:
    """Fiore tozzo a quattro petali: petalo in basso in ombra, bordo scuro e ombra a terra."""
    stamp(grid, x - 2, y - 2, [
        ".BpB.".replace("p", petal),
        "B" + petal + center + petal + "B",
        "." + petal + shade + "BA",
        "..BA.",
    ])


def pebble(grid, x: int, y: int) -> None:
    stamp(grid, x, y, [".AA.", "A6tA", "AtTA", ".AA."])


def make_grass(name: str, rng: random.Random) -> list[list[str]]:
    if name == "grass_a":
        return grass_base(rng, tufts=3, blades=3, dashes=3)
    if name == "grass_b":
        return grass_base(rng, tufts=5, blades=4, dashes=4)
    if name == "grass_c":
        grid = grass_base(rng, tufts=2, blades=2, dashes=1)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), SUN)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), SUN)
        return grid
    if name == "grass_tuft":
        grid = grass_base(rng, tufts=1, blades=2, dashes=2)
        stamp(grid, 3, 4, ["..b.b.b..", ".bAbAbAb.", "bAAbAbAAb", "AABAAABAA", ".BBBBBBB."])
        return grid
    if name == "grass_tall":
        grid = grass_base(rng, tufts=1, blades=1, dashes=2)
        stamp(grid, 1, 2, ["b...b", "Ab.bA", "AAbAA", "BAAAB", ".BBB."])
        stamp(grid, 7, 7, ["..b.b.b..", "b.A.b.Ab.", "AbAbAbAAb", "AAAAAAAAA", ".BBBBBBB."])
        return grid
    if name == "grass_clover":
        grid = grass_base(rng, tufts=1, blades=2, dashes=1)
        for x, y in ((3, 3), (10, 8), (4, 11)):
            stamp(grid, x, y, [".bb.", "bAbb", "bbAb", ".bA."])
        return grid
    if name == "grass_pebbles":
        grid = grass_base(rng, tufts=2, blades=2, dashes=1)
        pebble(grid, 2, 3)
        pebble(grid, 9, 9)
        stamp(grid, 6, 12, ["6t", "AA"])
        return grid
    if name in ("flowers_white", "flowers_pink", "flowers_yellow"):
        grid = grass_base(rng, tufts=1, blades=2, dashes=2)
        petal, shade, center = {
            "flowers_white": ("w", "t", "y"),
            "flowers_pink": ("m", "M", "y"),
            "flowers_yellow": ("y", "G", "f"),
        }[name]
        # un ciuffo di tre fiori, non sparsi su tutto il tile
        for x, y in ((5, 5), (11, 7), (6, 11)):
            flower(grid, x, y, petal, shade, center)
        return grid
    sys.exit(f"tile sconosciuto: {name}")


def make_dirt(rng: random.Random) -> list[list[str]]:
    """Terra battuta: crepe, granelli chiari, sassolini tozzi con contorno e ombra."""
    grid = blank("d")
    for _ in range(6):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["DD", ".D"])
    for _ in range(5):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["7", "D"])
    for _ in range(3):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "8")
    for _ in range(2):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), [".7.", "7dD", ".D8"])
    return grid


# Cespugli della siepe: ciuffi tondi e tozzi con luce in alto a sinistra e bordo verde scuro,
# sfalsati su due file e avvolti sui bordi del tile.
HEDGE_CLUMP = [
    "..NNN..",
    ".NbaaN.",
    "NbaaaAN",
    "NaaaAAN",
    "NAAABBN",
    ".NBBBN.",
]


def make_hedge(rng: random.Random) -> list[list[str]]:
    grid = blank("B")
    for x, y in ((0, 0), (8, 0), (4, 5), (12, 5), (0, 10), (8, 10), (4, 15), (12, 15)):
        stamp(grid, x + rng.randrange(2) - 1, y - 1, HEDGE_CLUMP)
    for _ in range(3):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "N")
    return grid


# Profondità (px) dell'erba che entra in un tile di bordo, per lato. I due estremi valgono 2
# così due tile di bordo affiancati si raccordano senza scalini.
EDGE_PROFILE = {
    "n": [2, 2, 3, 3, 2, 2, 3, 4, 4, 3, 2, 2, 3, 3, 2, 2],
    "e": [2, 3, 3, 2, 2, 3, 4, 3, 2, 2, 3, 3, 2, 2, 3, 2],
    "s": [2, 3, 3, 2, 2, 2, 3, 3, 4, 3, 3, 2, 2, 3, 2, 2],
    "w": [2, 2, 3, 4, 3, 2, 2, 3, 3, 2, 2, 3, 4, 3, 2, 2],
}
# Per la siepe l'erba entra di più e con gobbe più larghe: i cespugli sono tondi.
HEDGE_PROFILE = {
    "n": [3, 3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 2, 3, 4, 4, 3],
    "e": [3, 4, 4, 3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 2, 3, 3],
    "s": [3, 3, 4, 4, 3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 2, 3],
    "w": [3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 2, 3, 4, 4, 3, 3],
}


def outside_mask(mask: int, profile: dict[str, list[int]], x: int, y: int) -> bool:
    """True se il pixel (x, y) del tile di bordo è erba (fuori dal materiale)."""
    n, e, s, w = (bool(mask & bit) for bit, _ in EDGE_BITS)
    if (n and y < profile["n"][x]) or (s and y >= TILE - profile["s"][x]):
        return True
    if (w and x < profile["w"][y]) or (e and x >= TILE - profile["e"][y]):
        return True
    # angoli esterni arrotondati
    cx, cy = min(x, TILE - 1 - x), min(y, TILE - 1 - y)
    corner = (n if y < TILE // 2 else s) and (w if x < TILE // 2 else e)
    return corner and cx + cy < 6


def make_edge(material: list[list[str]], grass: list[list[str]], mask: int, hedge: bool):
    profile = HEDGE_PROFILE if hedge else EDGE_PROFILE
    is_grass = [[outside_mask(mask, profile, x, y) for x in range(TILE)] for y in range(TILE)]

    def grass_at(x: int, y: int) -> bool:
        if 0 <= x < TILE and 0 <= y < TILE:
            return is_grass[y][x]
        return False

    grid = [row[:] for row in material]
    for y in range(TILE):
        for x in range(TILE):
            if is_grass[y][x]:
                char = grass[y][x]
                if hedge:
                    # ombra portata della siepe sull'erba: verso sud ed est (luce da NO)
                    if not grass_at(x, y - 1) and y > 0:
                        char = "9"
                    elif not grass_at(x - 1, y) and x > 0:
                        char = "A"
                else:
                    # bordo dell'erba sopra la terra: punte scure sotto, chiare sopra
                    if not grass_at(x, y + 1) and y < TILE - 1:
                        char = "A"
                    elif not grass_at(x, y - 1) and y > 0:
                        char = "b"
                grid[y][x] = char
            elif hedge:
                # contorno quasi nero del cespuglio dove confina con l'erba (come gli attori),
                # bordo di luce dentro il contorno a nord e a ovest, faccia in ombra a sud
                if any(grass_at(x + dx, y + dy) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    grid[y][x] = "k"
                elif grass_at(x, y - 2) or grass_at(x - 2, y):
                    grid[y][x] = "a"
                elif grass_at(x, y + 2):
                    grid[y][x] = "N"
            else:
                # gradino della terra: sotto l'erba a nord e a ovest c'è la parete in ombra
                if grass_at(x, y - 1):
                    grid[y][x] = "8"
                elif grass_at(x, y - 2) or grass_at(x - 1, y):
                    grid[y][x] = "D"
    return grid


def build_tileset(palette) -> Image.Image:
    rng = random.Random(TILESET_SEED)
    grids: dict[str, list[list[str]]] = {}
    for name in TILE_NAMES:
        if name == "dirt":
            grids[name] = make_dirt(rng)
        elif name == "hedge":
            grids[name] = make_hedge(rng)
        elif name.startswith(("dirt_", "hedge_")):
            prefix, sides = name.split("_")
            mask = sum(bit for bit, side in EDGE_BITS if side in sides)
            grids[name] = make_edge(grids[prefix], grids["grass_a"], mask, prefix == "hedge")
        else:
            grids[name] = make_grass(name, rng)
    sheet = Image.new("RGBA", (TILE, TILE * len(TILE_NAMES)))
    for i, name in enumerate(TILE_NAMES):
        rows = ["".join(r) for r in grids[name]]
        sheet.paste(render_grid(rows, palette, name), (0, i * TILE))
    return sheet


def main() -> None:
    palette = load_palette()
    manifest: list[dict] = []

    for folder, kind in PX_KINDS:
        for src in sorted((SRC / folder).glob("*.px")):
            header, rows = parse_px(src)
            own_palette = src.parent / header["palette"] if "palette" in header else None
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
