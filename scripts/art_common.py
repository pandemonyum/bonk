"""Percorsi e lettori condivisi dagli script degli asset (build-assets.py e moduli art_*)."""

from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "assets-src"
OUT = ROOT / "assets"


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


def parse_blocks(path: Path) -> dict[str, list[str]]:
    """Timbri '@ <nome>' seguiti dalla loro griglia ('#' commento), come in assets-src/logo/."""
    blocks: dict[str, list[str]] = {}
    current: str | None = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        if raw.lstrip().startswith("#") or not raw.strip():
            continue
        if raw.startswith("@"):
            current = raw[1:].strip()
            if current in blocks:
                sys.exit(f"{path.name}: timbro ripetuto {current!r}")
            blocks[current] = []
        elif current is None:
            sys.exit(f"{path.name}: griglia prima del primo '@ <nome>'")
        else:
            blocks[current].append(raw.rstrip())
    for name, rows in blocks.items():
        if not rows or len({len(r) for r in rows}) != 1:
            sys.exit(f"{path.name}: {name}: righe vuote o di lunghezza diversa")
    return blocks


def render_grid(rows: list[str], palette, name: str) -> Image.Image:
    img = Image.new("RGBA", (len(rows[0]), len(rows)))
    for y, row in enumerate(rows):
        for x, char in enumerate(row):
            if char not in palette:
                sys.exit(f"{name}: carattere {char!r} non in palette (riga {y})")
            img.putpixel((x, y), palette[char])
    return img
