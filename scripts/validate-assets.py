#!/usr/bin/env python3
"""Valida gli asset in assets/ contro manifest e ART_BIBLE.

Controlli: ogni PNG è nel manifest (e viceversa), dimensioni dichiarate = reali, nomi
snake_case, tileset multipli di 16 px, sprite entro i limiti di dimensione, solo colori della
palette per gli asset generati, licenza presente.

Uso: python scripts/validate-assets.py   (exit code 1 se ci sono errori)
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
PALETTE = ROOT / "assets-src" / "palette.txt"
NAME_RE = re.compile(r"^[a-z0-9_]+\.png$")
MAX_SPRITE = 64  # boss inclusi; vedi docs/ART_BIBLE.md


def palette_colors() -> set[tuple[int, int, int]]:
    colors = set()
    for raw in PALETTE.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line and not line.startswith("#"):
            rgb = int(line.split()[1], 16)
            colors.add(((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF))
    return colors


def main() -> int:
    errors: list[str] = []
    manifest = json.loads((ASSETS / "manifest.json").read_text(encoding="utf-8"))
    entries = {e["path"]: e for e in manifest["assets"]}
    on_disk = {p.relative_to(ASSETS).as_posix() for p in ASSETS.rglob("*.png")}

    for missing in sorted(on_disk - entries.keys()):
        errors.append(f"{missing}: PNG non presente nel manifest")
    for missing in sorted(entries.keys() - on_disk):
        errors.append(f"{missing}: nel manifest ma file assente")

    colors = palette_colors()
    for path in sorted(on_disk & entries.keys()):
        entry = entries[path]
        img = Image.open(ASSETS / path).convert("RGBA")
        if not NAME_RE.match(Path(path).name):
            errors.append(f"{path}: nome non snake_case")
        if list(img.size) != entry["size"]:
            errors.append(f"{path}: dimensione {img.size} diversa dal manifest {entry['size']}")
        if not entry.get("license"):
            errors.append(f"{path}: licenza mancante")
        if entry["kind"] == "tileset":
            t = entry["tile_size"]
            if img.width != t or img.height != t * len(entry["tiles"]):
                errors.append(f"{path}: tileset atteso {t}x{t * len(entry['tiles'])}")
        elif entry["kind"] == "sprite" and max(img.size) > MAX_SPRITE:
            errors.append(f"{path}: sprite oltre {MAX_SPRITE}px")
        if entry["license"].startswith("original"):
            # get_flattened_data da Pillow 12; getdata e' deprecato (rimosso in Pillow 14)
            flat = getattr(img, "get_flattened_data", img.getdata)
            pixels = list(flat())
            stray = {px[:3] for px in pixels if px[3] != 0 and px[:3] not in colors}
            if stray:
                errors.append(f"{path}: {len(stray)} colori fuori palette")
            if any(0 < px[3] < 255 for px in pixels):
                errors.append(f"{path}: alpha semitrasparente (pixel art: solo 0 o 255)")

    for e in errors:
        print("ERRORE:", e)
    print(f"{'FALLITO' if errors else 'OK'}: {len(on_disk)} PNG controllati, {len(errors)} errori")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
