"""Logo "Companion Survivors" (assets/ui/logo.png) dal font Pixelify Sans e dai timbri di
assets-src/logo/. Usato da scripts/build-assets.py; regole in docs/ART_BIBLE.md (D-063)."""

from __future__ import annotations

import sys

from PIL import Image, ImageDraw, ImageFont

from art_common import OUT, ROOT, SRC, load_palette, parse_blocks

# --- Logo "Companion Survivors" ------------------------------------------------------------
# Le lettere sono quelle di Pixelify Sans Regular riportate sulla sua griglia: un pixel del
# font misura 93 unità (1000 per em) e i contorni sono "grassi" di ~8 unità, quindi disegnando
# con 16 campioni per pixel del font e tenendo le celle coperte per più di metà si ottiene il
# carattere pixel per pixel, senza sfumature. Ogni pixel del font diventa LOGO_PIXEL pixel del
# logo, poi la lettera viene ingrassata (Bold del font variabile non sta sulla griglia).
LOGO_OUT = "ui/logo.png"
LOGO_DIR = SRC / "logo"
LOGO_FONT = OUT / "fonts" / "pixelify-sans" / "PixelifySans.ttf"
LOGO_LICENSE = (
    "original, project-owned (lettering drawn from Pixelify Sans, OFL 1.1; "
    "parts in assets-src/logo)"
)
LOGO_UNITS_PER_PIXEL = 93
LOGO_SAMPLES = 16  # campioni per pixel del font, per lato
LOGO_PIXEL = (3, 5)  # un pixel del font = 3x5 pixel del logo: lettere alte, righe larghe ~200
LOGO_EMBOLDEN = (3, 2)  # la lettera cresce a destra e in basso: aste di 6 px, tratti di 7
LOGO_TRACKING = 3  # pixel tra le lettere oltre al pixel del font di spaziatura
# Ritocchi sulla griglia del font (x da 0 = bordo sinistro della lettera, riga 0 = ultima riga
# sopra la linea di base): la C di Pixelify è chiusa tranne un pixel, qui ha la bocca aperta.
LOGO_GLYPH_EDITS = {"C": ((4, -4), (4, -2))}
# Rampe: (riflesso, fasce dall'alto al basso, ombra interna, contorno, ombra portata).
LOGO_RAMPS = {"warm": ("1", "2345", "6", "7", "8"), "cool": ("A", "BCDF", "H", "J", "K")}
# Righe: testo, posizione dell'angolo in alto a sinistra rispetto alla prima riga, rampa.
LOGO_LINES = (("Companion", (0, 0), "warm"), ("Survivors", (18, 39), "cool"))
LOGO_FLAME = (0, "i")  # la fiamma prende il posto del punto della "i" di Companion
LOGO_STAR = (1, "o")  # la stella sta al centro della "o" di Survivors
# Foglie dietro le scritte: (riga, "left" = prima lettera, "right" = ultima), timbro, specchiata,
# scostamento dall'angolo in alto a sinistra (left) o in alto a destra (right) della lettera.
LOGO_LEAVES = (
    (0, "left", "leaf_up", False, (-13, -5)),
    (0, "left", "leaf_left", False, (-15, 11)),
    (0, "left", "leaf_down", False, (-12, 25)),
    (1, "right", "leaf_up", True, (1, -7)),
    (1, "right", "leaf_left", True, (3, 9)),
    (1, "right", "leaf_down", True, (0, 23)),
)
# Contorno di 2 pixel con gli angoli arrotondati e ombra portata in basso a destra.
LOGO_OUTLINE = [(dx, dy) for dy in range(-2, 3) for dx in range(-2, 3) if abs(dx) + abs(dy) < 4]
LOGO_SHADOW = ((0, 1), (1, 1), (0, 2), (1, 2))

Cells = set[tuple[int, int]]


def logo_font() -> tuple[ImageFont.FreeTypeFont, int, int]:
    """Font alla misura di LOGO_SAMPLES campioni per pixel e scostamento (x, y) che porta la
    griglia del font su quella dei campioni (presa dai bordi sinistro e basso della 'o')."""
    font = ImageFont.truetype(str(LOGO_FONT), LOGO_SAMPLES * 1000 / LOGO_UNITS_PER_PIXEL)
    font.set_variation_by_axes([400])
    s = LOGO_SAMPLES
    img = Image.new("L", (8 * s, 16 * s), 0)
    ImageDraw.Draw(img).text((2 * s, 12 * s), "o", font=font, fill=255, anchor="ls")
    x0, _, _, y1 = img.point(lambda v: 255 if v >= 128 else 0).getbbox()
    return font, (-x0) % s, (-y1) % s


def logo_glyph(font: ImageFont.FreeTypeFont, shift: tuple[int, int], char: str) -> Cells:
    """Pixel del font di una lettera (x da 0, riga 0 = ultima sopra la linea di base)."""
    s = LOGO_SAMPLES
    img = Image.new("L", (12 * s, 16 * s), 0)
    origin = (2 * s + shift[0], 12 * s + shift[1])
    ImageDraw.Draw(img).text(origin, char, font=font, fill=255, anchor="ls")
    small = img.resize((img.width // s, img.height // s), Image.Resampling.BOX)
    px = small.load()
    cells: Cells = set()
    for y in range(small.height):
        for x in range(small.width):
            if 77 < px[x, y] < 178:  # coperta a metà: il font non è più sulla sua griglia
                sys.exit(f"logo: '{char}' non sta sulla griglia del font (LOGO_UNITS_PER_PIXEL?)")
            if px[x, y] >= 128:
                cells.add((x, y - 12))
    if not cells:
        sys.exit(f"logo: il font non disegna '{char}'")
    left = min(x for x, _ in cells)
    return {(x - left, y) for x, y in cells} - set(LOGO_GLYPH_EDITS.get(char, ()))


def logo_letters(text: str, origin: tuple[int, int]) -> tuple[list[Cells], int]:
    """Lettere di una riga in pixel del logo e riga della linea di base (ultima riga piena)."""
    font, ox, oy = logo_font()
    sx, sy = LOGO_PIXEL
    bx, by = LOGO_EMBOLDEN
    top = 8  # righe del font sopra la linea di base
    letters: list[Cells] = []
    pen = origin[0]
    for char in text:
        glyph = logo_glyph(font, (ox, oy), char)
        cells: Cells = set()
        for x, y in glyph:
            for dy in range(sy + by):
                for dx in range(sx + bx):
                    cells.add((pen + x * sx + dx, origin[1] + (y + top) * sy + dy))
        letters.append(cells)
        pen += (max(x for x, _ in glyph) + 2) * sx + LOGO_TRACKING
    return letters, origin[1] + (top + 1) * sy - 1 + by


def box(cells) -> tuple[int, int, int, int]:
    xs = [x for x, _ in cells]
    ys = [y for _, y in cells]
    return min(xs), min(ys), max(xs), max(ys)


def logo_line(letters: list[Cells], baseline: int, ramp: str) -> dict[tuple[int, int], str]:
    """Colori di una riga: sfumatura verticale a fasce per lettera, riflesso di 1 pixel in
    alto, ombra interna in basso, contorno e ombra portata attorno a tutta la riga."""
    light, bands, inner, outline, shadow = LOGO_RAMPS[ramp]
    mask = set().union(*letters)
    fill = {}
    for cells in letters:
        top = box(cells)[1]
        for x, y in cells:
            band = int((y - top) / max(1, baseline - top) * len(bands))
            char = bands[min(len(bands) - 1, band)]
            if (x, y - 1) not in mask:
                char = light
            elif (x, y + 1) not in mask:
                char = inner
            fill[(x, y)] = char
    ring = {(x + dx, y + dy) for x, y in mask for dx, dy in LOGO_OUTLINE} - mask
    body = mask | ring
    drop = {(x + dx, y + dy) for x, y in body for dx, dy in LOGO_SHADOW} - body
    layer = dict.fromkeys(drop, shadow)
    layer.update(dict.fromkeys(ring, outline))
    layer.update(fill)
    return layer


def put_stamp(canvas: dict, rows: list[str], x0: int, y0: int, flip: bool = False) -> None:
    for dy, row in enumerate(rows):
        for dx, char in enumerate(row[::-1] if flip else row):
            if char != ".":
                canvas[(x0 + dx, y0 + dy)] = char


def build_logo() -> tuple[Image.Image, dict]:
    """ui/logo.png: due righe di scritte, foglie dietro, fiamma e stella sopra."""
    parts = parse_blocks(LOGO_DIR / "logo_parts.px")
    lines = []
    for text, origin, ramp in LOGO_LINES:
        letters, baseline = logo_letters(text, origin)
        lines.append((text, letters, baseline, ramp))
    # la fiamma sostituisce il punto della "i": tolgo le righe sopra il primo vuoto
    row, char = LOGO_FLAME
    text, letters = lines[row][0], lines[row][1]
    stem = letters[text.index(char)]
    rows = sorted({y for _, y in stem})
    gap = next((y for y in range(rows[0], rows[-1]) if y not in rows), None)
    if gap is None:
        sys.exit(f"logo: la '{char}' di {text} non ha il punto")
    dot = {c for c in stem if c[1] < gap}
    stem -= dot

    canvas: dict[tuple[int, int], str] = {}
    for row, side, name, flip, (dx, dy) in LOGO_LEAVES:
        text, letters = lines[row][0], lines[row][1]
        x0, y0, x1, _ = box(letters[0] if side == "left" else letters[-1])
        put_stamp(canvas, parts[name], (x0 if side == "left" else x1) + dx, y0 + dy, flip)
    for _, letters, baseline, ramp in lines:  # la seconda riga copre la coda della "p"
        canvas.update(logo_line(letters, baseline, ramp))
    flame = parts["flame"]
    x0, _, x1, _ = box(stem)
    put_stamp(canvas, flame, (x0 + x1 + 1) // 2 - len(flame[0]) // 2, box(dot)[3] + 2 - len(flame))
    row, char = LOGO_STAR
    text, letters = lines[row][0], lines[row][1]
    x0, y0, x1, y1 = box(letters[text.index(char)])
    star = parts["star"]
    sx, sy = (x0 + x1 + 1) // 2 - len(star[0]) // 2, (y0 + y1 + 1) // 2 - len(star) // 2
    put_stamp(canvas, star, sx, sy)

    palette_path = LOGO_DIR / "logo.palette.txt"
    palette = load_palette(palette_path)
    x0, y0, x1, y1 = box(canvas)
    # misure pari: l'interfaccia ha 2 unità per texel e il logo si centra a texel interi
    width, height = x1 - x0 + 1, y1 - y0 + 1
    img = Image.new("RGBA", (width + width % 2, height + height % 2))
    for (x, y), char in canvas.items():
        if char not in palette:
            sys.exit(f"logo: carattere {char!r} non in {palette_path.name}")
        img.putpixel((x - x0, y - y0), palette[char])
    entry = {
        "path": LOGO_OUT,
        "kind": "logo",
        "size": [img.width, img.height],
        "frames": 1,
        "palette": palette_path.relative_to(ROOT).as_posix(),
        "font": LOGO_FONT.relative_to(ROOT).as_posix(),
        "source": (LOGO_DIR / "logo_parts.px").relative_to(ROOT).as_posix(),
        "license": LOGO_LICENSE,
    }
    return img, entry
