"""Animazioni ambientali della home (D-067): maschere e layout ricavati dall'illustrazione.

Da assets-src/illustrations/home_background.png, per ogni effetto delle tabelle qui sotto:
- assets/ui/home_ambient/<nome>.png: maschera RGB ritagliata sul rettangolo dell'effetto, agli
  stessi pixel dell'illustrazione (nessun ridimensionamento); i canali dipendono dal tipo
  (CHANNELS);
- assets/ui/home_ambient/portal_vortex.png: il vortice del portale completo, da far ruotare, con i
  pixel dell'illustrazione come la mostra il gioco (assets/ui/home_background.png, anche a 256
  colori): fermo coincide col fondo. Dove l'illustrazione non lo mostra (dietro le rovine, negli angoli) ci va il colore dei texel
  visibili più vicini sulla stessa orbita di rotazione;
- assets/ui/home_ambient.ron: il layout che il gioco incorpora (src/presentation/home_ambient/):
  rettangolo, maschera e parametri di ogni effetto, nell'ordine in cui il gioco li disegna.
Le zone sono scritte qui in pixel dell'illustrazione; dentro ogni zona i pixel giusti li sceglie
il colore (vortice, rune, acqua, schiuma, fiamma). Le maschere dicono solo *dove* il gioco anima i
pixel dell'illustrazione: nessun pixel dipinto nuovo. Un pixel è al più di un effetto (il fiume
lascia alle cascate i loro) e i personaggi davanti all'acqua non sono mai di nessuno.

Uso: chiamato da scripts/build-assets.py; da solo, `python scripts/art_home_ambient.py --preview
<png>` disegna le maschere sopra l'illustrazione (R rosso, G verde, B blu) per controllarle.
"""

from __future__ import annotations

import io
import math
import sys
from collections.abc import Callable
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

from art_common import OUT, ROOT, SRC

ILLUSTRATION = SRC / "illustrations" / "home_background.png"
ILLUSTRATION_OUT = "ui/home_background.png"
MASK_DIR = "ui/home_ambient"
LAYOUT_OUT = "ui/home_ambient.ron"
PLATE_NAME = "portal_vortex"
MASK_LICENSE = (
    "derived from ui/home_background.png by scripts/art_home_ambient.py (masks only, no new "
    "pixels): same rights as the illustration (D-065, D-067)"
)
CHANNELS = {
    "portal": "R vortice, G rune, B alone",
    "waterfall": "R acqua che cade, G schiuma, B pozza delle increspature",
    "river": "R acqua, G interno (lontano dalla riva), B creste chiare",
    "flame": "R fiamma e margine in cui guizza, G nucleo chiaro",
}

Box = tuple[int, int, int, int]  # x0, y0, x1, y1 (x1 e y1 esclusi), pixel dell'illustrazione
Pixel = tuple[int, int, int]

# --- Zone (pixel dell'illustrazione 1672x941) -------------------------------------------------
# rect = (x, y, larghezza, altezza): l'area del nodo del gioco, con margine per particelle e guizzi.
PORTAL = {
    "name": "portal",
    "rect": (1330, 205, 195, 240),
    "center": (1427.0, 371.0),  # occhio della spirale: il vortice ruota attorno a questo punto
    "radii": (38.0, 57.0),  # semiassi del vortice, fin dove ruota
    "squareness": 2.0,  # 2 = ellisse (la spirale dipinta è ellittica); di più = rettangolo stondato
    "spin": -1.0,  # verso della rotazione: quello in cui i bracci dipinti scendono verso l'occhio
    "vortex_box": (1384, 306, 1470, 432),
    # pietra dell'arco e della torre: fuori resta il cielo, azzurro quanto le rune
    "rune_region": [
        (1336, 432), (1336, 296), (1352, 282), (1375, 272), (1405, 268), (1405, 212),
        (1452, 212), (1452, 268), (1480, 278), (1500, 286), (1516, 298), (1520, 432),
    ],
}
# lip/base: righe dove l'acqua comincia a cadere e tocca la pozza; pool: (cx, cy, rx, ry) della
# pozza con le increspature (rx 0 = niente pozza); speed: texel al secondo.
WATERFALLS = [
    {
        "name": "falls_castle",
        "rect": (1155, 390, 65, 58),
        "column": (1160, 394, 1217, 446),
        "lip": 396,
        "base": 442,
        "foam": None,
        "pool": (0, 0, 0, 0),
        "speed": 26.0,
    },
    {
        "name": "falls_left",
        "rect": (998, 445, 72, 80),
        "column": (1005, 449, 1063, 506),
        "lip": 451,
        "base": 503,
        "foam": (999, 492, 1069, 514),
        "pool": (1033, 513, 33, 9),
        "speed": 30.0,
    },
    {
        "name": "falls_mid",
        "rect": (1128, 476, 66, 82),
        "column": (1133, 480, 1192, 546),
        "lip": 484,
        "base": 540,
        "foam": (1128, 530, 1194, 553),
        "pool": (1151, 551, 20, 6),
        "speed": 30.0,
    },
    {
        "name": "falls_arch",
        "rect": (1232, 565, 63, 86),
        "column": (1247, 568, 1291, 619),
        "lip": 571,
        "base": 616,
        "foam": (1232, 600, 1295, 633),
        "pool": (1264, 637, 30, 9),
        "speed": 28.0,
    },
    {
        "name": "falls_bridge",
        "rect": (1295, 543, 45, 98),
        "column": (1311, 545, 1328, 604),
        "lip": 548,
        "base": 600,
        "foam": (1295, 590, 1340, 631),
        "pool": (1318, 635, 20, 7),
        "speed": 32.0,
    },
    {
        "name": "falls_right",
        "rect": (1482, 598, 88, 176),
        "column": (1493, 603, 1554, 724),
        "lip": 612,
        "base": 716,
        "foam": (1484, 702, 1569, 745),
        "pool": (1526, 753, 40, 13),
        "speed": 36.0,
    },
]
RIVER = {"name": "river", "rect": (550, 392, 690, 340)}
FLAME = {
    "name": "flame",
    "rect": (1418, 672, 64, 92),
    "base": 757.0,  # riga dove la fiamma esce dalla coda: sopra guizza sempre di più
    "region": [(1426, 674), (1482, 674), (1482, 758), (1441, 758), (1430, 742)],
}
# Personaggi e bolle davanti all'acqua: nessuna maschera li tocca mai (rettangoli o cerchi).
OBSTRUCTIONS: list[tuple] = [
    ("rect", (998, 522, 1138, 941)),  # player
    ("rect", (1140, 640, 1306, 941)),  # Ondino
    ("circle", (1270, 657, 24)),  # bolla di Ondino
    ("circle", (1163, 742, 13)),  # bolla piccola
    ("rect", (824, 612, 992, 941)),  # Foglin
    ("rect", (1305, 682, 1426, 941)),  # Fiammetto (la fiamma della coda è un effetto suo)
]


# --- Classificazione dei colori ---------------------------------------------------------------
def is_vortex(r: int, g: int, b: int) -> bool:
    return b > 200 and g > r + 30


def is_white(r: int, g: int, b: int) -> bool:
    return min(r, g, b) >= 200 and b >= 235


def is_rune(r: int, g: int, b: int) -> bool:
    return b >= 235 and g >= 150 and g - r >= 40


def is_falling_water(r: int, g: int, b: int) -> bool:
    return b >= 228 and g >= r


def is_foam(r: int, g: int, b: int) -> bool:
    return min(r, g, b) >= 195 or (b >= 245 and g >= 215)


def is_pool_water(r: int, g: int, b: int) -> bool:
    return (b >= 200 and g > r + 20) or min(r, g, b) >= 195


def is_lake(r: int, g: int, b: int) -> bool:
    # chiaro o profondo; le rocce azzurrine hanno il verde e il blu troppo vicini al rosso
    return b >= 190 and g >= 120 and g - r >= 50 and b - r >= 120


def is_crest(r: int, g: int, b: int) -> bool:
    return is_white(r, g, b)


def is_flame(r: int, g: int, b: int) -> bool:
    return r >= 235 and b <= 215 and g >= 35


def is_flame_core(r: int, g: int, b: int) -> bool:
    return r >= 240 and g >= 190


# --- Maschere (immagini "L" grandi quanto il rettangolo dell'effetto) --------------------------
def blank(rect: tuple[int, int, int, int]) -> Image.Image:
    return Image.new("L", rect[2:], 0)


def classify(
    art: Image.Image, rect: tuple, box: Box, test: Callable[[int, int, int], bool]
) -> Image.Image:
    """255 dove `test` accetta il colore, dentro `box` e dentro il rettangolo."""
    x, y, w, h = rect
    mask = blank(rect)
    px, out = art.load(), mask.load()
    for yy in range(max(box[1], y), min(box[3], y + h)):
        for xx in range(max(box[0], x), min(box[2], x + w)):
            if test(*px[xx, yy]):
                out[xx - x, yy - y] = 255
    return mask


def obstruction_mask(rect: tuple) -> Image.Image:
    x, y, _, _ = rect
    mask = blank(rect)
    draw = ImageDraw.Draw(mask)
    for shape, data in OBSTRUCTIONS:
        if shape == "rect":
            x0, y0, x1, y1 = data
            draw.rectangle((x0 - x, y0 - y, x1 - 1 - x, y1 - 1 - y), fill=255)
        else:
            cx, cy, r = data
            draw.ellipse((cx - r - x, cy - r - y, cx + r - x, cy + r - y), fill=255)
    return mask


def without(mask: Image.Image, other: Image.Image) -> Image.Image:
    return ImageChops.subtract(mask, other)


def grow(mask: Image.Image, size: int) -> Image.Image:
    return mask.filter(ImageFilter.MaxFilter(size))


def shrink(mask: Image.Image, size: int) -> Image.Image:
    return mask.filter(ImageFilter.MinFilter(size))


def scaled(mask: Image.Image, factor: float) -> Image.Image:
    return mask.point(lambda v: min(255, round(v * factor)))


def component(mask: Image.Image, seed: tuple[int, int]) -> Image.Image:
    """Solo la parte connessa a `seed`, con i buchi riempiti."""
    work = mask.point(lambda v: 255 if v else 0)
    ImageDraw.floodfill(work, seed, 128)
    part = work.point(lambda v: 255 if v == 128 else 0)
    # buchi: tutto ciò che il fondo esterno non raggiunge
    outside = Image.new("L", (part.width + 2, part.height + 2), 0)
    outside.paste(part, (1, 1))
    ImageDraw.floodfill(outside, (0, 0), 128)
    return outside.crop((1, 1, part.width + 1, part.height + 1)).point(
        lambda v: 0 if v == 128 else 255
    )


def ellipse_falloff(rect: tuple, pool: tuple[int, int, int, int]) -> Image.Image:
    """1 al centro della pozza, 0 sul bordo dell'ellisse (quadratico)."""
    x, y, _, _ = rect
    cx, cy, rx, ry = pool
    mask = blank(rect)
    if rx <= 0 or ry <= 0:
        return mask
    out = mask.load()
    for yy in range(mask.height):
        for xx in range(mask.width):
            d = ((xx + x - cx) / rx) ** 2 + ((yy + y - cy) / ry) ** 2
            if d < 1.0:
                out[xx, yy] = round(255 * (1.0 - d))
    return mask


def polygon_mask(rect: tuple, points: list[tuple[int, int]]) -> Image.Image:
    x, y, _, _ = rect
    mask = blank(rect)
    ImageDraw.Draw(mask).polygon([(px - x, py - y) for px, py in points], fill=255)
    return mask


def box_mask(rect: tuple, box: Box | None) -> Image.Image:
    mask = blank(rect)
    if box:
        x, y, _, _ = rect
        ImageDraw.Draw(mask).rectangle((box[0] - x, box[1] - y, box[2] - 1 - x, box[3] - 1 - y), 255)
    return mask


def merge(r: Image.Image, g: Image.Image, b: Image.Image) -> Image.Image:
    return Image.merge("RGB", (r, g, b))


# --- Effetti ----------------------------------------------------------------------------------
def portal_vortex(art: Image.Image) -> Image.Image:
    """255 sul vortice che si vede (classificato, connesso all'occhio, senza buchi)."""
    p = PORTAL
    rect = p["rect"]
    cx, cy = p["center"]
    if not is_vortex(*art.getpixel((round(cx), round(cy)))):
        # il riempimento partirebbe dal fondo e prenderebbe pietra e rune
        sys.exit(f"home_ambient: l'occhio {p['center']} non è sul vortice: aggiorna PORTAL")
    raw = classify(art, rect, p["vortex_box"], is_vortex)
    filled = component(raw, (round(cx) - rect[0], round(cy) - rect[1]))
    # dei buchi riempiti restano solo le strisce bianche: non la cima delle rovine che il bordo
    # illuminato del vortice chiude da ogni lato
    light = classify(art, rect, p["vortex_box"], lambda r, g, b: is_vortex(r, g, b) or is_white(r, g, b))
    return ImageChops.multiply(filled, light)


def vortex_core(vortex: Image.Image) -> Image.Image:
    """Il vortice senza il texel di bordo: sul profilo delle rovine è scuro e girando lascerebbe
    una scia. Il vortice completo copia questi texel così come sono; quelli tolti li ricostruisce."""
    return shrink(vortex, 3)


def level(qx: float, qy: float, n: float) -> float:
    return (abs(qx) ** n + abs(qy) ** n) ** (1.0 / n)


def portal_plate(art: Image.Image, vortex: Image.Image) -> Image.Image:
    """Il vortice completo fino al bordo del superellisse in cui ruota (vedi docstring)."""
    p = PORTAL
    x0, y0, w, h = p["rect"]
    (cx, cy), (rx, ry), n = p["center"], p["radii"], p["squareness"]
    plate = art.crop((x0, y0, x0 + w, y0 + h))
    seen, out, src = vortex_core(vortex).load(), plate.load(), art.load()

    def visible(theta: float, rho: float) -> tuple[int, int, int] | None:
        ux, uy = math.cos(theta), math.sin(theta)
        k = rho / level(ux, uy, n)
        xx, yy = math.floor(cx + ux * k * rx) - x0, math.floor(cy + uy * k * ry) - y0
        if 0 <= xx < w and 0 <= yy < h and seen[xx, yy]:
            return src[xx + x0, yy + y0]
        return None

    for yy in range(h):
        for xx in range(w):
            qx, qy = (xx + x0 + 0.5 - cx) / rx, (yy + y0 + 0.5 - cy) / ry
            rho = level(qx, qy, n)
            if seen[xx, yy] or rho > 1.05 or rho < 1e-3:
                continue
            # il colore dei texel visibili più vicini sulla stessa orbita, da una parte e
            # dall'altra, pesati con la distanza: il vortice gira lungo queste orbite
            theta, step = math.atan2(qy, qx), 0.7 / (rho * max(rx, ry))
            found = []
            for sign in (1, -1):
                for k in range(1, int(math.pi / step)):
                    color = visible(theta + sign * k * step, rho)
                    if color:
                        found.append((k, color))
                        break
            if len(found) == 2:
                (k1, c1), (k2, c2) = found
                out[xx, yy] = tuple(round((a * k2 + b * k1) / (k1 + k2)) for a, b in zip(c1, c2))
            elif found:
                out[xx, yy] = found[0][1]
    return plate


def portal_mask(art: Image.Image, vortex: Image.Image) -> Image.Image:
    p = PORTAL
    rect = p["rect"]
    glyphs = ImageChops.multiply(
        classify(art, rect, (0, 0, 10**4, 10**4), is_rune), polygon_mask(rect, p["rune_region"])
    )
    glyphs = without(glyphs, grow(vortex, 9))
    runes = ImageChops.lighter(glyphs, scaled(grow(glyphs, 3), 0.4))
    halo = scaled(vortex.filter(ImageFilter.GaussianBlur(6)), 2.2)
    # gira solo l'interno (`vortex_core`): il texel di bordo resta quello fermo del fondo
    return merge(vortex_core(vortex), runes, without(halo, vortex))


def waterfall_mask(art: Image.Image, fall: dict, blocked: Image.Image) -> Image.Image:
    rect = fall["rect"]
    column = classify(art, rect, fall["column"], is_falling_water)
    column = shrink(grow(column, 3), 3)  # chiude i puntini di roccia dentro il getto
    column = ImageChops.multiply(column, box_mask(rect, fall["column"]))
    foam = classify(art, rect, fall["foam"], is_foam) if fall["foam"] else blank(rect)
    foam = ImageChops.lighter(foam, scaled(grow(foam, 3), 0.5))
    foam = ImageChops.multiply(foam, box_mask(rect, fall["foam"]))
    water = classify(art, rect, (0, 0, 10**4, 10**4), is_pool_water)
    pool = ImageChops.multiply(ellipse_falloff(rect, fall["pool"]), water)
    pool = without(without(pool, grow(column, 3)), foam)
    channels = [without(c, blocked) for c in (column, foam, pool)]
    return merge(*channels)


def river_mask(art: Image.Image, claimed: Image.Image) -> Image.Image:
    rect = RIVER["rect"]
    everywhere = (0, 0, 10**4, 10**4)
    lake = classify(art, rect, everywhere, is_lake)
    crest = ImageChops.multiply(classify(art, rect, everywhere, is_crest), grow(lake, 5))
    blocked = ImageChops.lighter(obstruction_mask(rect), claimed)
    water = without(ImageChops.lighter(lake, crest), blocked)
    interior = shrink(water, 5).filter(ImageFilter.BoxBlur(1))
    return merge(water, interior, without(crest, blocked))


def flame_mask(art: Image.Image) -> Image.Image:
    rect = FLAME["rect"]
    region = polygon_mask(rect, FLAME["region"])
    everywhere = (0, 0, 10**4, 10**4)
    flame = ImageChops.multiply(classify(art, rect, everywhere, is_flame), region)
    core = ImageChops.multiply(classify(art, rect, everywhere, is_flame_core), flame)
    return merge(ImageChops.multiply(grow(flame, 5), region), core, blank(rect))


def claimed_in(rect: tuple, masks: list[tuple[tuple, Image.Image]]) -> Image.Image:
    """Pixel già usati da altri effetti, riportati nel rettangolo `rect`."""
    out = blank(rect)
    for other, mask in masks:
        used = ImageChops.lighter(
            ImageChops.lighter(mask.getchannel("R"), mask.getchannel("G")), mask.getchannel("B")
        ).point(lambda v: 255 if v else 0)
        layer = blank(rect)
        layer.paste(used, (other[0] - rect[0], other[1] - rect[1]))
        out = ImageChops.lighter(out, layer)
    return out


# --- Layout per il gioco ----------------------------------------------------------------------
def ron_float(v: float) -> str:
    return f"{float(v):.1f}"


def ron_effect(name: str, rect: tuple, kind: str, plate: str | None = None) -> str:
    x, y, w, h = rect
    extra = f'            plate: Some("{MASK_DIR}/{plate}.png"),\n' if plate else ""
    return (
        "        (\n"
        f'            name: "{name}",\n'
        f'            mask: "{MASK_DIR}/{name}.png",\n'
        f"            rect: ({x}, {y}, {w}, {h}),\n" + extra + f"            kind: {kind},\n"
        "        ),\n"
    )


def layout_text(size: tuple[int, int]) -> str:
    p = PORTAL
    effects = [
        ron_effect(
            p["name"],
            p["rect"],
            f"Portal(center: ({ron_float(p['center'][0])}, {ron_float(p['center'][1])}), "
            f"radii: ({ron_float(p['radii'][0])}, {ron_float(p['radii'][1])}), "
            f"squareness: {ron_float(p['squareness'])}, spin: {ron_float(p['spin'])})",
            plate=PLATE_NAME,
        )
    ]
    for fall in WATERFALLS:
        pool = ", ".join(ron_float(v) for v in fall["pool"])
        effects.append(
            ron_effect(
                fall["name"],
                fall["rect"],
                f"Waterfall(lip: {ron_float(fall['lip'])}, base: {ron_float(fall['base'])}, "
                f"speed: {ron_float(fall['speed'])}, pool: ({pool}))",
            )
        )
    effects.append(ron_effect(RIVER["name"], RIVER["rect"], "River"))
    effects.append(
        ron_effect(FLAME["name"], FLAME["rect"], f"Flame(base: {ron_float(FLAME['base'])})")
    )
    return (
        "// Generato da scripts/art_home_ambient.py (D-067): non modificare a mano.\n"
        "// Animazioni ambientali della home: rettangoli in pixel dell'illustrazione.\n"
        "(\n"
        f"    image: ({size[0]}, {size[1]}),\n"
        "    effects: [\n" + "".join(effects) + "    ],\n)\n"
    )


def png_bytes(img: Image.Image) -> bytes:
    buffer = io.BytesIO()
    img.save(buffer, "PNG", optimize=True)
    return buffer.getvalue()


def build_masks(art: Image.Image) -> list[tuple[str, str, tuple, Image.Image]]:
    """(nome, tipo, rettangolo, maschera) di ogni effetto, nell'ordine del layout; il vortice
    completo del portale è a parte (`portal_plate`)."""
    for rect in [PORTAL["rect"], RIVER["rect"], FLAME["rect"], *(f["rect"] for f in WATERFALLS)]:
        x, y, w, h = rect
        if x < 0 or y < 0 or x + w > art.width or y + h > art.height:
            sys.exit(f"home_ambient: rettangolo {rect} fuori dall'illustrazione {art.size}")
    built = [(PORTAL["name"], "portal", PORTAL["rect"], portal_mask(art, portal_vortex(art)))]
    falls = []
    for fall in WATERFALLS:
        mask = waterfall_mask(art, fall, obstruction_mask(fall["rect"]))
        falls.append((fall["rect"], mask))
        built.append((fall["name"], "waterfall", fall["rect"], mask))
    # i pixel delle cascate, anche appena fuori dal rettangolo del fiume, più un margine: i due
    # nodi non si toccano nemmeno con la maschera letta in bilineare
    pad = 3
    x, y, w, h = RIVER["rect"]
    wide = claimed_in((x - pad, y - pad, w + 2 * pad, h + 2 * pad), falls)
    claimed = grow(wide, 2 * pad + 1).crop((pad, pad, pad + w, pad + h))
    built.append((RIVER["name"], "river", RIVER["rect"], river_mask(art, claimed)))
    built.append((FLAME["name"], "flame", FLAME["rect"], flame_mask(art)))
    check_disjoint(built)
    return built


def check_disjoint(built: list[tuple[str, str, tuple, Image.Image]]) -> None:
    """Nessun pixel animato da due effetti: si disegnerebbero uno sopra l'altro."""
    owner: dict[tuple[int, int], str] = {}
    for name, _, rect, mask in built:
        r, g, b = (mask.getchannel(c).load() for c in "RGB")
        for yy in range(mask.height):
            for xx in range(mask.width):
                if r[xx, yy] or g[xx, yy] or b[xx, yy]:
                    at = (xx + rect[0], yy + rect[1])
                    if at in owner:
                        sys.exit(f"home_ambient: il pixel {at} è sia di {owner[at]} sia di {name}")
                    owner[at] = name


def build_home_ambient(
    license_text: str, shown: bytes
) -> tuple[list[tuple[str, bytes, dict]], str]:
    """Maschere (percorso, PNG, voce del manifest) e testo del layout. `shown` è il PNG
    dell'illustrazione come esce dal build: i colori del vortice vengono da lì."""
    if not ILLUSTRATION.is_file():
        sys.exit(f"home_ambient: manca {ILLUSTRATION.relative_to(ROOT)}")
    with Image.open(ILLUSTRATION) as opened:
        art = opened.convert("RGB")
    with Image.open(io.BytesIO(shown)) as opened:
        colors = opened.convert("RGB")
    if colors.size != art.size:
        sys.exit(f"home_ambient: illustrazione del build {colors.size}, sorgente {art.size}")
    images = [(name, "mask", kind, rect, mask) for name, kind, rect, mask in build_masks(art)]
    plate = portal_plate(colors, portal_vortex(art))
    images.append((PLATE_NAME, "plate", "portal", PORTAL["rect"], plate))
    out = []
    for name, role, kind, rect, image in images:
        path = f"{MASK_DIR}/{name}.png"
        entry = {
            "path": path,
            "kind": role,
            "size": [rect[2], rect[3]],
            "frames": 1,
            "origin": [rect[0], rect[1]],
            "of": ILLUSTRATION_OUT,
            "effect": kind,
            "channels": CHANNELS[kind] if role == "mask" else "colore (sRGB)",
            "source": ILLUSTRATION.relative_to(ROOT).as_posix(),
            "generator": "scripts/art_home_ambient.py",
            "license": license_text,
        }
        out.append((path, png_bytes(image), entry))
    return out, layout_text(art.size)


def write_home_ambient(masks: list[tuple[str, bytes, dict]], layout: str) -> None:
    """Scrive maschere e layout; toglie solo i PNG di MASK_DIR che non escono più dal build."""
    folder = OUT / MASK_DIR
    folder.mkdir(parents=True, exist_ok=True)
    wanted = {Path(path).name for path, _, _ in masks}
    for stale in sorted(folder.glob("*.png")):
        if stale.name not in wanted:
            print(f"  tolta la maschera che non esce più: {stale.relative_to(ROOT).as_posix()}")
            stale.unlink()
    for path, data, _ in masks:
        (OUT / path).write_bytes(data)
    (OUT / LAYOUT_OUT).write_text(layout, encoding="utf-8", newline="\n")


def preview(dest: Path) -> None:
    with Image.open(ILLUSTRATION) as opened:
        art = opened.convert("RGB")
    view = art.copy()
    for _, _, rect, mask in build_masks(art):
        crop = view.crop((rect[0], rect[1], rect[0] + rect[2], rect[1] + rect[3]))
        for channel, color in zip("RGB", [(255, 40, 40), (40, 255, 40), (60, 120, 255)]):
            alpha = scaled(mask.getchannel(channel), 0.7)
            crop = Image.composite(Image.new("RGB", crop.size, color), crop, alpha)
        view.paste(crop, rect[:2])
        ImageDraw.Draw(view).rectangle(
            (rect[0], rect[1], rect[0] + rect[2] - 1, rect[1] + rect[3] - 1), outline=(255, 255, 0)
        )
    view.save(dest)
    plate_dest = dest.with_name(dest.stem + "_vortex" + dest.suffix)
    shown = OUT / ILLUSTRATION_OUT
    with Image.open(shown if shown.is_file() else ILLUSTRATION) as opened:
        colors = opened.convert("RGB")
    portal_plate(colors, portal_vortex(art)).save(plate_dest)
    print(f"anteprima delle maschere: {dest}; vortice completo: {plate_dest}")


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--preview":
        preview(Path(sys.argv[2]))
    else:
        sys.exit("uso: python scripts/art_home_ambient.py --preview <png> (il build è build-assets.py)")
