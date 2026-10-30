"""Illustrazioni fornite dall'utente: assets-src/illustrations/ -> assets/ui/ (D-063).
Usato da scripts/build-assets.py; regole in docs/ART_BIBLE.md."""

from __future__ import annotations

import io
import sys
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter, ImageStat

from art_common import ROOT, SRC

# --- Illustrazioni fornite dall'utente (D-063) -------------------------------------------------
# Non sono pixel art del progetto: niente palette, niente griglia. Il PNG resta a piena
# risoluzione e proporzione (il codice fa un "cover" sulla finestra). Per il peso del build web
# si prova una versione a 256 colori con dithering Floyd-Steinberg e la si tiene solo se,
# guardata da vicino come si vede a schermo (media su 9x9 pixel), resta uguale all'originale.
ILLUSTRATIONS = SRC / "illustrations"
ILLUSTRATION_EXTS = (".png", ".webp")
ILLUSTRATION_LICENSE = "provided by the user, rights declared by the user (D-063)"
ILLUSTRATION_PALETTE_SAMPLE = 3  # palette calcolata su una copia a 1/3: 0,8 s invece di 8
ILLUSTRATION_MAX_MEAN_ERROR = 2.0  # errore medio per canale (0-255) dopo la media 9x9
ILLUSTRATION_MAX_PEAK_ERROR = 32  # errore massimo per canale dopo la media 9x9


def png_bytes(img: Image.Image) -> bytes:
    buffer = io.BytesIO()
    img.save(buffer, "PNG", optimize=True)
    return buffer.getvalue()


def seen_error(a: Image.Image, b: Image.Image) -> tuple[float, int]:
    """Differenza dopo una media 9x9: il dithering conta per il colore che si vede."""
    blur = ImageFilter.BoxBlur(4)
    diff = ImageChops.difference(a.convert("RGB").filter(blur), b.convert("RGB").filter(blur))
    return max(ImageStat.Stat(diff).mean), max(high for _, high in diff.getextrema())


def build_illustrations() -> list[tuple[str, bytes, dict]]:
    if not ILLUSTRATIONS.is_dir():
        return []
    by_name: dict[str, list[Path]] = {}
    for path in sorted(ILLUSTRATIONS.iterdir()):
        if path.suffix.lower() in ILLUSTRATION_EXTS:
            by_name.setdefault(path.stem, []).append(path)
    built = []
    for name, paths in sorted(by_name.items()):
        if len(paths) > 1:
            sys.exit(f"illustrations: più sorgenti per {name}: {[p.name for p in paths]}")
        src = paths[0]
        with Image.open(src) as opened:
            img = opened.convert("RGBA")
        if img.getchannel("A").getextrema()[0] == 255:
            img = img.convert("RGB")
        full = png_bytes(img)
        chosen, encoding = full, "rgb" if img.mode == "RGB" else "rgba"
        report = f"{img.mode} {len(full) / 2**20:.2f} MiB"
        if img.mode == "RGB":  # con trasparenza resta piena
            n = ILLUSTRATION_PALETTE_SAMPLE
            sample_size = (max(1, img.width // n), max(1, img.height // n))
            sample = img.resize(sample_size, Image.Resampling.BOX)
            colors = sample.quantize(colors=256, method=Image.Quantize.MAXCOVERAGE)
            indexed = img.quantize(palette=colors, dither=Image.Dither.FLOYDSTEINBERG)
            small = png_bytes(indexed)
            mean, peak = seen_error(img, indexed)
            report += f", 256 colori {len(small) / 2**20:.2f} MiB"
            report += f" (errore medio {mean:.2f}, max {peak})"
            unseen = mean <= ILLUSTRATION_MAX_MEAN_ERROR and peak <= ILLUSTRATION_MAX_PEAK_ERROR
            if unseen and len(small) < len(full):
                chosen, encoding = small, "indexed-256"
        print(f"  {src.name}: {report} -> {encoding}")
        entry = {
            "path": f"ui/{name}.png",
            "kind": "illustration",
            "size": [img.width, img.height],
            "frames": 1,
            "encoding": encoding,
            "source": src.relative_to(ROOT).as_posix(),
            "license": ILLUSTRATION_LICENSE,
        }
        built.append((entry["path"], chosen, entry))
    return built
