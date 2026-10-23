#!/usr/bin/env python3
"""Genera gli asset pixel art di gioco da sorgenti testuali in assets-src/.

- Sprite: file assets-src/sprites/*.px (griglia di caratteri della palette).
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


def load_palette() -> dict[str, tuple[int, int, int, int]]:
    palette: dict[str, tuple[int, int, int, int]] = {".": (0, 0, 0, 0)}
    for raw in (SRC / "palette.txt").read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        char, hexcode = line.split()[:2]
        if len(char) != 1 or char in palette:
            sys.exit(f"palette.txt: carattere non valido o duplicato: {char!r}")
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
    if header.get("mirror") == "true":
        rows = [row + row[::-1] for row in rows]
    widths = {len(row) for row in rows}
    if len(widths) != 1:
        sys.exit(f"{path.name}: righe di lunghezza diversa {sorted(widths)}")
    if "out" not in header:
        sys.exit(f"{path.name}: manca 'out:' nell'intestazione")
    return header, rows


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


# Timbri d'erba: ciuffetto a "v" (punte chiare, base scura), filo, macchia d'ombra.
TUFT_V = ["b.b", ".A."]
TUFT_W = ["b.b.b", "A.A.A", ".A.A."]
BLADE = ["b", "A"]
DASH = ["AA"]
SUN = [".bb", "bbb", "bb."]


def grass_base(rng: random.Random, tufts: int, blades: int, dashes: int) -> list[list[str]]:
    grid = blank("a")
    for _ in range(dashes):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), DASH)
    for _ in range(dashes // 2):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "B")
    for _ in range(blades):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), BLADE)
    for _ in range(tufts):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), TUFT_V)
    return grid


def flower(grid, x: int, y: int, petal: str, shade: str, center: str) -> None:
    """Fiore a quattro petali con il petalo in basso in ombra e un'ombra a terra."""
    stamp(grid, x - 1, y - 1, [f".{petal}.", f"{petal}{center}{petal}", f".{shade}.", "..A"])


def pebble(grid, x: int, y: int) -> None:
    stamp(grid, x, y, ["6t", "TT", "AA"])


def make_grass(name: str, rng: random.Random) -> list[list[str]]:
    if name == "grass_a":
        return grass_base(rng, tufts=4, blades=5, dashes=4)
    if name == "grass_b":
        return grass_base(rng, tufts=6, blades=6, dashes=6)
    if name == "grass_c":
        grid = grass_base(rng, tufts=2, blades=2, dashes=1)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), SUN)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), SUN)
        return grid
    if name == "grass_tuft":
        grid = grass_base(rng, tufts=2, blades=2, dashes=2)
        stamp(grid, 4, 5, ["..b.b..", ".bAbAb.", "bAAbAAb", "AABABAA", ".BBBBB."])
        return grid
    if name == "grass_tall":
        grid = grass_base(rng, tufts=1, blades=2, dashes=2)
        stamp(grid, 1, 3, ["b..b", "Ab.A", "AAbA", "BAAB", ".BB."])
        stamp(grid, 8, 8, ["..b.b..", "b.A.Ab.", "AbAbAAb", "AAAAAAA", ".BBBBB."])
        return grid
    if name == "grass_clover":
        grid = grass_base(rng, tufts=2, blades=2, dashes=1)
        for x, y in ((3, 4), (10, 9), (5, 12)):
            stamp(grid, x, y, [".b.", "bAb", ".A."])
        return grid
    if name == "grass_pebbles":
        grid = grass_base(rng, tufts=2, blades=3, dashes=2)
        pebble(grid, 3, 4)
        pebble(grid, 10, 9)
        put(grid, 6, 12, "t")
        put(grid, 6, 13, "A")
        return grid
    if name in ("flowers_white", "flowers_pink", "flowers_yellow"):
        grid = grass_base(rng, tufts=2, blades=2, dashes=2)
        petal, shade, center = {
            "flowers_white": ("w", "t", "y"),
            "flowers_pink": ("m", "M", "y"),
            "flowers_yellow": ("y", "G", "f"),
        }[name]
        # un ciuffo di tre fiori, non sparsi su tutto il tile
        for x, y in ((5, 5), (9, 7), (6, 10)):
            flower(grid, x, y, petal, shade, center)
        return grid
    sys.exit(f"tile sconosciuto: {name}")


def make_dirt(rng: random.Random) -> list[list[str]]:
    grid = blank("d")
    for _ in range(9):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["DD"])
    for _ in range(4):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["D", "D"])
    for _ in range(3):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["7", "D"])
    for _ in range(2):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "8")
    return grid


# Cespugli della siepe: cerchi a ciuffi con luce in alto a sinistra, su fondo scuro.
HEDGE_CLUMP = [
    ".aaA.",
    "abaAA",
    "aAAAB",
    "AAABN",
    ".BBN.",
]


def make_hedge(rng: random.Random) -> list[list[str]]:
    grid = blank("B")
    for x, y in ((0, 0), (8, 0), (4, 5), (12, 5), (0, 10), (8, 10), (4, 15), (12, 15)):
        stamp(grid, x + rng.randrange(2), y, HEDGE_CLUMP)
    for _ in range(4):
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
                # contorno scuro del cespuglio dove confina con l'erba; faccia frontale a sud
                if any(grass_at(x + dx, y + dy) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    grid[y][x] = "N"
                elif grass_at(x, y + 2):
                    grid[y][x] = "B"
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
            img = render_grid(rows, palette, src.name)
            dest = OUT / header["out"]
            dest.parent.mkdir(parents=True, exist_ok=True)
            img.save(dest)
            manifest.append(
                {
                    "path": header["out"],
                    "kind": kind,
                    "size": [img.width, img.height],
                    "frames": 1,
                    "source": src.relative_to(ROOT).as_posix(),
                    "license": LICENSE,
                }
            )

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
        print(f"  {entry['path']:32} {entry['size'][0]}x{entry['size'][1]}")
    print(f"OK: {len(manifest)} asset generati in {OUT.relative_to(ROOT)}/")


if __name__ == "__main__":
    main()
