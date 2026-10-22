#!/usr/bin/env python3
"""Verifica nearest-neighbor su uno screenshot: ogni texel del canvas 480x270 deve essere un
blocco uniforme di NxN pixel fisici (niente righe/colonne duplicate, niente sfumature).

Uso: python scripts/check-pixel-grid.py screenshots/s.png [--virtual 480x270]
Exit code 1 se meno del 100% dei blocchi è uniforme. Il testo HUD/overlay non è pixel art:
usare una finestra con bande nere in cui il testo cada fuori dal canvas (es. `--window
1101x701`) e senza `--overlay`. A 960x540 il promemoria comandi copre il canvas e il
controllo scende a ~99% anche con scala corretta.
"""

from __future__ import annotations

import argparse
import math
import sys

from PIL import Image


def uniform_ratio(img: Image.Image, x0: int, y0: int, n: int, vw: int, vh: int) -> float:
    px = img.load()
    uniform = 0
    for ty in range(vh):
        for tx in range(vw):
            bx, by = x0 + tx * n, y0 + ty * n
            ref = px[bx, by]
            if all(px[bx + dx, by + dy] == ref for dy in range(n) for dx in range(n)):
                uniform += 1
    return uniform / (vw * vh)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("screenshot")
    parser.add_argument("--virtual", default="480x270")
    args = parser.parse_args()
    vw, vh = (int(v) for v in args.virtual.lower().split("x"))

    img = Image.open(args.screenshot).convert("RGB")
    w, h = img.size
    n = min(w // vw, h // vh)
    if n < 1:
        print(f"finestra {w}x{h} più piccola del canvas: niente scala intera da verificare")
        return 1
    # Il canvas è centrato: con bande dispari l'origine può cadere su floor o ceil.
    cx, cy = (w - vw * n) / 2, (h - vh * n) / 2
    best = max(
        (uniform_ratio(img, x0, y0, n, vw, vh), x0, y0)
        for x0 in {math.floor(cx), math.ceil(cx)}
        for y0 in {math.floor(cy), math.ceil(cy)}
    )
    ratio, x0, y0 = best
    ok = ratio == 1.0
    print(
        f"{'OK' if ok else 'FALLITO'}: {w}x{h}, scala {n}x, canvas in ({x0},{y0}), "
        f"blocchi uniformi {ratio:.4%}"
    )
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
