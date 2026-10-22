#!/usr/bin/env python3
"""Genera gli asset pixel art di gioco da sorgenti testuali in assets-src/.

- Sprite: file assets-src/sprites/*.px (griglia di caratteri della palette).
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

# Ordine = indice nel tileset (TileData::from_tileset_index). Tenere allineato con
# src/world/tiles.rs e docs/ART_BIBLE.md.
TILE_NAMES = [
    "grass_a",
    "grass_b",
    "grass_tuft",
    "flowers_white",
    "flowers_pink",
    "dirt",
    "hedge",
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


def blank(fill: str) -> list[list[str]]:
    return [[fill] * TILE for _ in range(TILE)]


def scatter_blades(grid, rng: random.Random, count: int) -> None:
    """Fili d'erba di 1x2 px: luce sopra, ombra sotto."""
    for _ in range(count):
        x, y = rng.randrange(TILE), rng.randrange(TILE - 1)
        grid[y][x] = "b"
        grid[y + 1][x] = "A"


def scatter(grid, rng: random.Random, char: str, count: int) -> None:
    for _ in range(count):
        grid[rng.randrange(TILE)][rng.randrange(TILE)] = char


def flower(grid, x: int, y: int, petal: str, center: str) -> None:
    for dx, dy in ((0, -1), (-1, 0), (1, 0), (0, 1)):
        grid[y + dy][x + dx] = petal
    grid[y][x] = center


def make_tile(name: str, rng: random.Random) -> list[list[str]]:
    if name in ("grass_a", "grass_b"):
        grid = blank("a")
        scatter_blades(grid, rng, 6 if name == "grass_a" else 9)
        scatter(grid, rng, "A", 4)
    elif name == "grass_tuft":
        grid = blank("a")
        scatter_blades(grid, rng, 3)
        # ciuffo centrale a ventaglio
        for x, top in ((5, 7), (6, 5), (7, 4), (8, 4), (9, 5), (10, 7)):
            for y in range(top, 11):
                grid[y][x] = "b" if y == top else "A"
    elif name in ("flowers_white", "flowers_pink"):
        grid = blank("a")
        scatter_blades(grid, rng, 4)
        petal = "w" if name == "flowers_white" else "m"
        for x, y in ((4, 4), (11, 6), (6, 11)):
            flower(grid, x, y, petal, "y")
    elif name == "dirt":
        grid = blank("d")
        scatter(grid, rng, "D", 18)
        for _ in range(3):
            x, y = rng.randrange(1, TILE - 1), rng.randrange(1, TILE - 1)
            grid[y][x] = "t"
            grid[y + 1][x] = "T"
    elif name == "hedge":
        grid = blank("B")
        for _ in range(10):
            x, y = rng.randrange(TILE - 2), rng.randrange(TILE - 2)
            grid[y][x] = grid[y][x + 1] = grid[y + 1][x] = "A"
            grid[y + 1][x + 1] = "A"
            grid[y][x] = "a"
        scatter(grid, rng, "K", 6)
    else:
        sys.exit(f"tile sconosciuto: {name}")
    return grid


def build_tileset(palette) -> Image.Image:
    rng = random.Random(TILESET_SEED)
    sheet = Image.new("RGBA", (TILE, TILE * len(TILE_NAMES)))
    for i, name in enumerate(TILE_NAMES):
        rows = ["".join(r) for r in make_tile(name, rng)]
        sheet.paste(render_grid(rows, palette, name), (0, i * TILE))
    return sheet


def main() -> None:
    palette = load_palette()
    manifest: list[dict] = []

    for src in sorted((SRC / "sprites").glob("*.px")):
        header, rows = parse_px(src)
        img = render_grid(rows, palette, src.name)
        dest = OUT / header["out"]
        dest.parent.mkdir(parents=True, exist_ok=True)
        img.save(dest)
        manifest.append(
            {
                "path": header["out"],
                "kind": "sprite",
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
