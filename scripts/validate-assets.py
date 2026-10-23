#!/usr/bin/env python3
"""Valida gli asset in assets/ contro manifest e ART_BIBLE.

Controlli: ogni PNG è nel manifest (e viceversa), dimensioni dichiarate = reali, nomi
snake_case, tileset multipli di 16 px, sprite e icone UI entro i limiti di dimensione, solo
colori della palette per gli asset generati, licenza presente. Avvisi (non errori) se un
upgrade di game-data/upgrades.ron non ha l'icona ui/upgrade_<id>.png o un compagno di
game-data/companions.ron non ha il ritratto ui/portrait_<id>.png.

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
GAME_DATA = ROOT / "game-data"
PALETTE = ROOT / "assets-src" / "palette.txt"
NAME_RE = re.compile(r"^[a-z0-9_]+\.png$")
# Campi sprite nei file RON di game-data/ (es. sprite: "sprites/x.png").
DATA_SPRITE_RE = re.compile(r'\b(?:sprite|projectile_sprite)\s*:\s*"([^"]+)"')
MAX_SPRITE = 64  # boss inclusi; vedi docs/ART_BIBLE.md
MAX_UI = 32  # icone e ritratti dell'interfaccia
DATA_ID_RE = re.compile(r'^\s*id\s*:\s*"([a-z0-9_]+)"')


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
        elif entry["kind"] == "ui" and max(img.size) > MAX_UI:
            errors.append(f"{path}: icona UI oltre {MAX_UI}px")
        if entry["license"].startswith("original"):
            # get_flattened_data da Pillow 12; getdata e' deprecato (rimosso in Pillow 14)
            flat = getattr(img, "get_flattened_data", img.getdata)
            pixels = list(flat())
            stray = {px[:3] for px in pixels if px[3] != 0 and px[:3] not in colors}
            if stray:
                errors.append(f"{path}: {len(stray)} colori fuori palette")
            if any(0 < px[3] < 255 for px in pixels):
                errors.append(f"{path}: alpha semitrasparente (pixel art: solo 0 o 255)")

    # Gli sprite citati dai dati di gioco devono esistere ed essere nel manifest: il gioco li
    # carica per percorso e un file mancante sarebbe solo uno sprite invisibile.
    referenced = 0
    for data_file in sorted(GAME_DATA.glob("*.ron")):
        for line_no, line in enumerate(data_file.read_text(encoding="utf-8").splitlines(), 1):
            if line.lstrip().startswith("//"):
                continue
            for path in DATA_SPRITE_RE.findall(line):
                referenced += 1
                if path not in entries:
                    errors.append(
                        f"game-data/{data_file.name}:{line_no}: sprite \"{path}\" "
                        "assente da assets/ o dal manifest"
                    )

    # Icone attese dall'interfaccia per ogni upgrade e ritratti per ogni compagno: solo avvisi,
    # così un upgrade nuovo nei dati non blocca nessuno finché l'icona non è disegnata.
    warnings: list[str] = []
    for data_file, prefix in (("upgrades.ron", "upgrade"), ("companions.ron", "portrait")):
        for line in (GAME_DATA / data_file).read_text(encoding="utf-8").splitlines():
            match = DATA_ID_RE.match(line)
            if match and f"ui/{prefix}_{match.group(1)}.png" not in entries:
                warnings.append(f"game-data/{data_file}: manca ui/{prefix}_{match.group(1)}.png")

    for w in warnings:
        print("AVVISO:", w)
    for e in errors:
        print("ERRORE:", e)
    print(
        f"{'FALLITO' if errors else 'OK'}: {len(on_disk)} PNG controllati, "
        f"{referenced} sprite citati in game-data/, {len(errors)} errori"
    )
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
