#!/usr/bin/env python3
"""Anteprime dello stile grafico composte con Pillow dagli asset in assets/.

Serve a iterare sulla pixel art senza compilare il gioco: compone scene 480x270 (la
risoluzione virtuale del gioco) con tile, ostacoli, player, compagni, orda, gemme, foglie,
anello di gocce e un HUD finto con le icone di assets/ui/, poi le salva ingrandite x3 con
nearest-neighbor in docs/evidence/art-preview-*.png.

Non è il gioco: la mappa è disegnata qui (stesse regole dei bordi di src/world/generation.rs)
e i testi usano un carattere 3x5 provvisorio solo per l'anteprima, non una proposta di font.

Uso: python scripts/build-assets.py && python scripts/preview-scene.py
Richiede: Python 3.10+ e Pillow.
"""

from __future__ import annotations

import json
import math
import random
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
EVIDENCE = ROOT / "docs" / "evidence"
W, H = 480, 270
TILE = 16
SCALE = 3


def palette() -> dict[str, tuple[int, int, int, int]]:
    colors = {}
    for raw in (ROOT / "assets-src" / "palette.txt").read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line and not line.startswith("#"):
            char, hexcode = line.split()[:2]
            rgb = int(hexcode, 16)
            colors[char] = ((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF, 255)
    return colors


PAL = palette()
_cache: dict[str, Image.Image] = {}


def art(path: str) -> Image.Image:
    if path not in _cache:
        _cache[path] = Image.open(ASSETS / path).convert("RGBA")
    return _cache[path]


# --- Carattere 3x5 provvisorio (solo anteprima) ------------------------------------------

# Una riga per glifo: 5 righe da 3 pixel separate da "/".
FONT = {
    "A": ".#./#.#/###/#.#/#.#", "B": "##./#.#/##./#.#/##.", "C": ".##/#../#../#../.##",
    "D": "##./#.#/#.#/#.#/##.", "E": "###/#../##./#../###", "F": "###/#../##./#../#..",
    "G": ".##/#../#.#/#.#/.##", "H": "#.#/#.#/###/#.#/#.#", "I": "###/.#./.#./.#./###",
    "J": "..#/..#/..#/#.#/.#.", "K": "#.#/#.#/##./#.#/#.#", "L": "#../#../#../#../###",
    "M": "#.#/###/###/#.#/#.#", "N": "##./#.#/#.#/#.#/#.#", "O": ".#./#.#/#.#/#.#/.#.",
    "P": "##./#.#/##./#../#..", "Q": ".#./#.#/#.#/##./.##", "R": "##./#.#/##./#.#/#.#",
    "S": ".##/#../.#./..#/##.", "T": "###/.#./.#./.#./.#.", "U": "#.#/#.#/#.#/#.#/###",
    "V": "#.#/#.#/#.#/#.#/.#.", "W": "#.#/#.#/###/###/#.#", "X": "#.#/#.#/.#./#.#/#.#",
    "Y": "#.#/#.#/.#./.#./.#.", "Z": "###/..#/.#./#../###", "0": "###/#.#/#.#/#.#/###",
    "1": ".#./##./.#./.#./###", "2": "##./..#/.#./#../###", "3": "##./..#/.#./..#/##.",
    "4": "#.#/#.#/###/..#/..#", "5": "###/#../##./..#/##.", "6": ".##/#../###/#.#/###",
    "7": "###/..#/.#./.#./.#.", "8": "###/#.#/###/#.#/###", "9": "###/#.#/###/..#/##.",
    ".": ".../.../.../.../.#.", ":": ".../.#./.../.#./...", "+": ".../.#./###/.#./...",
    "-": ".../.../###/.../...", "%": "#.#/..#/.#./#../#.#", "/": "..#/..#/.#./#../#..",
    "!": ".#./.#./.#./.../.#.", "'": ".#./.#./.../.../...", " ": ".../.../.../.../...",
    ",": ".../.../.../.#./#..", "?": "##./..#/.#./.../.#.", "[": "##./#../#../#../##.",
    "]": ".##/..#/..#/..#/.##", "(": ".#./#../#../#../.#.", ")": ".#./..#/..#/..#/.#.",
}
GLYPHS = {char: rows.replace("/", "") for char, rows in FONT.items()}
assert all(len(g) == 15 for g in GLYPHS.values())


def text(img: Image.Image, x: int, y: int, s: str, color: str = "w", shadow: str = "k") -> int:
    """Scrive `s` in maiuscolo con ombra di 1 px; restituisce la x finale."""
    for char in s.upper():
        glyph = GLYPHS.get(char, GLYPHS["?"])
        for i, bit in enumerate(glyph):
            if bit == "#":
                gx, gy = x + i % 3, y + i // 3
                if shadow:
                    img.putpixel((gx + 1, gy + 1), PAL[shadow])
        for i, bit in enumerate(glyph):
            if bit == "#":
                img.putpixel((x + i % 3, y + i // 3), PAL[color])
        x += 4
    return x


def text_width(s: str) -> int:
    return len(s) * 4 - 1


# --- Mappa di tile -----------------------------------------------------------------------

EDGE_BITS = ((1, (0, -1)), (2, (1, 0)), (4, (0, 1)), (8, (-1, 0)))


def tile_names() -> dict[str, int]:
    manifest = json.loads((ASSETS / "manifest.json").read_text(encoding="utf-8"))
    sheet = next(e for e in manifest["assets"] if e["kind"] == "tileset")
    return {name: i for i, name in enumerate(sheet["tiles"])}


def edge_tile(names: dict[str, int], material: str, mask: int) -> int:
    if mask == 0:
        return names[material]
    sides = "".join(s for (bit, _), s in zip(EDGE_BITS, "nesw") if mask & bit)
    return names[f"{material}_{sides}"]


def value_noise(x: float, y: float, cell: float, salt: str) -> float:
    cx, cy = math.floor(x / cell), math.floor(y / cell)
    fx, fy = x / cell - cx, y / cell - cy
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)

    def corner(dx: int, dy: int) -> float:
        return random.Random(f"{salt}:{cx + dx},{cy + dy}").random()

    top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * fx
    bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * fx
    return top + (bottom - top) * fy


def grass_tile(names: dict[str, int], x: int, y: int) -> int:
    """Varianti d'erba con fiori a macchie, come `visual_tile_at` in src/world/generation.rs."""
    r = random.Random(f"grass:{x},{y}").random()
    if value_noise(x, y, 6, "flowers") > 0.76 and r < 0.3:
        return names[("flowers_white", "flowers_pink", "flowers_yellow")[min(2, int(r / 0.3 * 3))]]
    for limit, name in ((0.32, "grass_a"), (0.62, "grass_b"), (0.74, "grass_c"),
                        (0.82, "grass_tuft"), (0.87, "grass_tall"), (0.92, "grass_clover"),
                        (0.955, "grass_pebbles"), (0.975, "flowers_white"),
                        (0.99, "flowers_pink")):
        if r < limit:
            return names[name]
    return names["flowers_yellow"]


def draw_map(material_at, cols: int, rows: int) -> Image.Image:
    """`material_at(x, y)` -> "grass" | "dirt" | "hedge"; fuori mappa = stesso materiale."""
    names = tile_names()
    sheet = art("tiles/meadow_tileset.png")
    img = Image.new("RGBA", (cols * TILE, rows * TILE))
    for y in range(rows):
        for x in range(cols):
            material = material_at(x, y)
            if material == "grass":
                index = grass_tile(names, x, y)
            else:
                mask = 0
                for bit, (dx, dy) in EDGE_BITS:
                    nx, ny = x + dx, y + dy
                    inside = 0 <= nx < cols and 0 <= ny < rows
                    if inside and material_at(nx, ny) != material:
                        mask |= bit
                index = edge_tile(names, material, mask)
            tile = sheet.crop((0, index * TILE, TILE, (index + 1) * TILE))
            img.paste(tile, (x * TILE, y * TILE))
    return img


# --- Composizione ------------------------------------------------------------------------


class Layer:
    """Sprite da disegnare in ordine di y (i piedi): chi sta più in basso copre chi è dietro."""

    def __init__(self) -> None:
        self.items: list[tuple[float, int, Image.Image, int, int]] = []

    def actor(self, path: str, x: int, y: int, depth: int = 0) -> None:
        img = art(path)
        self.items.append((y + depth * 10000, len(self.items), img, x - img.width // 2,
                           y - img.height))

    def centered(self, path: str, x: int, y: int, depth: int = 1) -> None:
        img = art(path)
        self.items.append((y + depth * 10000, len(self.items), img, x - img.width // 2,
                           y - img.height // 2))

    def ground(self, path: str, x: int, y: int) -> None:
        img = art(path)
        self.items.append((-10000 + y, len(self.items), img, x - img.width // 2,
                           y - img.height))

    def draw(self, canvas: Image.Image) -> None:
        for _, _, img, x, y in sorted(self.items, key=lambda t: (t[0], t[1])):
            canvas.alpha_composite(img, (x, y))


def rect(img: Image.Image, x0: int, y0: int, x1: int, y1: int, color: str) -> None:
    for y in range(max(0, y0), min(img.height, y1)):
        for x in range(max(0, x0), min(img.width, x1)):
            img.putpixel((x, y), PAL[color])


def panel(img, x0, y0, x1, y1, border: str, fill: str = "1", top: str = "2") -> None:
    """Pannello blu notte: contorno scuro, bordo colorato di 1 px, riga di luce in alto."""
    rect(img, x0, y0, x1, y1, "k")
    rect(img, x0 + 1, y0 + 1, x1 - 1, y1 - 1, border)
    rect(img, x0 + 2, y0 + 2, x1 - 2, y1 - 2, fill)
    rect(img, x0 + 2, y0 + 2, x1 - 2, y0 + 3, top)
    for cx, cy in ((x0, y0), (x1 - 1, y0), (x0, y1 - 1), (x1 - 1, y1 - 1)):
        img.putpixel((cx, cy), (0, 0, 0, 0))


def bar(img, x, y, w, frac: float, fill: str, light: str, dark: str) -> None:
    """Barra da 5 px: contorno, fondo scuro, riempimento con riga di luce e d'ombra."""
    rect(img, x, y, x + w, y + 5, "k")
    rect(img, x + 1, y + 1, x + w - 1, y + 4, "0")
    filled = int((w - 2) * frac)
    rect(img, x + 1, y + 1, x + 1 + filled, y + 4, fill)
    rect(img, x + 1, y + 1, x + 1 + filled, y + 2, light)
    rect(img, x + 1, y + 3, x + 1 + filled, y + 4, dark)


def ui(path: str) -> Image.Image:
    return art(f"ui/{path}.png")


def hud(canvas: Image.Image, level: int, kills: int, clock: str) -> None:
    for i, state in enumerate(("full", "full", "full", "half", "empty")):
        canvas.alpha_composite(ui(f"heart_{state}"), (6 + i * 11, 5))
    bar(canvas, 6, 16, 80, 0.62, "u", "W", "U")
    for i, element in enumerate(("fire", "water", "nature")):
        canvas.alpha_composite(ui(f"icon_{element}"), (6 + i * 18, 24))
    clock_icon = ui("clock")
    total = clock_icon.width + 3 + text_width(clock)
    x0 = (W - total) // 2
    canvas.alpha_composite(clock_icon, (x0, 4))
    text(canvas, x0 + clock_icon.width + 3, 6, clock)
    label = f"LV {level}"
    text(canvas, W - 6 - text_width(label), 5, label, "g")
    skull = ui("skull")
    count = f"{kills}"
    canvas.alpha_composite(skull, (W - 6 - text_width(count) - skull.width - 3, 13))
    text(canvas, W - 6 - text_width(count), 15, count)


def meadow_map() -> Image.Image:
    def material(x: int, y: int) -> str:
        if x >= 28 or y >= 16:
            return "hedge"
        d1 = ((x - 6.5) / 4.2) ** 2 + ((y - 11.5) / 2.6) ** 2
        d2 = ((x - 10) / 3.0) ** 2 + ((y - 13.5) / 2.0) ** 2
        d3 = ((x - 21) / 2.2) ** 2 + ((y - 3.5) / 1.6) ** 2
        return "dirt" if min(d1, d2, d3) <= 1.0 else "grass"

    return draw_map(material, 30, 17)


def scene_gameplay() -> Image.Image:
    canvas = Image.new("RGBA", (W, H))
    canvas.alpha_composite(meadow_map().crop((0, 0, W, H)))
    layer = Layer()
    for x, y in ((40, 70), (120, 40), (420, 60), (380, 120), (70, 250), (300, 250)):
        layer.actor("sprites/tree.png", x, y)
    for x, y in ((180, 70), (330, 200), (100, 150)):
        layer.actor("sprites/rock.png", x, y)
    px, py = 240, 150
    layer.actor("sprites/summoner.png", px, py)
    layer.actor("sprites/ember_companion.png", px - 18, py - 12)
    layer.actor("sprites/tide_companion.png", px + 19, py - 12)
    layer.actor("sprites/sprout_companion.png", px, py - 22)
    rng = random.Random(7)
    kinds = ["blob_enemy"] * 11 + ["swarm_enemy"] * 6 + ["armored_enemy"] * 3
    placed: list[tuple[int, int]] = [(px, py)]
    while len(placed) < 75:
        angle = rng.uniform(0, math.tau)
        dist = rng.uniform(64, 250)
        x = int(px + math.cos(angle) * dist * 1.25)
        y = int(py + math.sin(angle) * dist * 0.75)
        if not (12 < x < W - 12 and 40 < y < H - 2):
            continue
        if any((x - ox) ** 2 + (y - oy) ** 2 < 150 for ox, oy in placed):
            continue
        placed.append((x, y))
        layer.actor(f"sprites/{rng.choice(kinds)}.png", x, y)
    layer.actor("sprites/elite_enemy.png", 352, 168)
    for _ in range(16):
        layer.ground("sprites/xp_gem.png", rng.randrange(80, 400), rng.randrange(70, 250))
    cx, cy = px, py - 8
    for i in range(3):
        a = math.radians(30 + i * 120)
        layer.centered("sprites/leaf.png", int(cx + math.cos(a) * 34), int(cy + math.sin(a) * 34))
    for i in range(28):
        a = math.tau * i / 28
        layer.centered("sprites/droplet.png", int(cx + math.cos(a) * 46),
                       int(cy + math.sin(a) * 46))
    for x, y in ((196, 118), (176, 104), (268, 186), (292, 200)):
        layer.centered("sprites/fireball.png", x, y)
    for i in range(8):
        a = math.tau * i / 8
        layer.centered("sprites/telegraph.png", int(352 + math.cos(a) * 16),
                       int(160 + math.sin(a) * 10), depth=-1)
    layer.draw(canvas)
    hud(canvas, 18, 1283, "08:24")
    return canvas


def big_text(img: Image.Image, x: int, y: int, s: str, color: str, shadow: str = "k") -> None:
    """Titoli del mock: glifi 3x5 a pixel doppi (nel gioco sarà un font vero)."""
    tmp = Image.new("RGBA", (text_width(s) + 2, 7))
    text(tmp, 0, 0, s, color, shadow)
    tmp = tmp.resize((tmp.width * 2, tmp.height * 2), Image.NEAREST)
    img.alpha_composite(tmp, (x, y))


def wrap(s: str, width: int) -> list[str]:
    lines, line = [], ""
    for word in s.split():
        if line and len(line) + 1 + len(word) > width:
            lines.append(line)
            line = word
        else:
            line = f"{line} {word}".strip()
    return lines + [line]


def dim(img: Image.Image, amount: float = 0.62) -> Image.Image:
    """Velo blu notte sopra la scena (nel gioco: nodo UI semitrasparente)."""
    return Image.blend(img, Image.new("RGBA", img.size, PAL["0"]), amount)


# (id upgrade, titolo, descrizione, bordo = colore dell'elemento, livello)
CARDS = (
    ("ember_damage", "FIAMMA VIVA", "DANNO DEI COMPAGNI +15%", "o", "LV. 2/5"),
    ("join_tide", "NUOVO COMPAGNO: GOCCI",
     "ONDA D'ACQUA ATTORNO A TE CHE FERISCE E RALLENTA", "c", "LV. 1/1"),
    ("regen", "GERMOGLIO", "RIGENERA 0,5 HP AL SECONDO", "e", "LV. 1/3"),
)


def scene_draft() -> Image.Image:
    canvas = dim(scene_gameplay())
    title = "LIVELLO 18!"
    big_text(canvas, (W - text_width(title) * 2) // 2, 28, title, "g", "h")
    sub = "SCEGLI UN POTENZIAMENTO"
    text(canvas, (W - text_width(sub)) // 2, 50, sub, "w")
    card_w, card_h, gap = 116, 100, 14
    x0 = (W - 3 * card_w - 2 * gap) // 2
    for i, (uid, name, desc, border, level) in enumerate(CARDS):
        x, y = x0 + i * (card_w + gap), 70
        selected = i == 1
        if selected:
            rect(canvas, x - 2, y - 2, x + card_w + 2, y + card_h + 2, "y")
        panel(canvas, x, y, x + card_w, y + card_h, "g" if selected else border)
        icon = ui(f"upgrade_{uid}")
        canvas.alpha_composite(icon, (x + (card_w - icon.width) // 2, y + 10))
        cy = y + 34
        for line in wrap(name, 27):
            text(canvas, x + (card_w - text_width(line)) // 2, cy, line, "w")
            cy += 8
        text(canvas, x + (card_w - text_width(level)) // 2, cy, level, "4")
        cy += 12
        for line in wrap(desc, 26):
            text(canvas, x + 8, cy, line, "4")
            cy += 8
        key = f"[{i + 1}]"
        text(canvas, x + (card_w - text_width(key)) // 2, y + card_h - 12, key, "3")
    hint = "1-3 O CLIC PER SCEGLIERE"
    text(canvas, (W - text_width(hint)) // 2, 184, hint, "4")
    return canvas


# (id, nome, barre ATT/VEL/AREA da 0 a 1: valori indicativi per il mock, bordo)
COMPANIONS = (
    ("ember", "BRASA", (0.8, 0.6, 0.3), "o"),
    ("tide", "GOCCI", (0.5, 0.4, 0.9), "c"),
    ("sprout", "FROND", (0.4, 0.7, 0.6), "e"),
)


def scene_select() -> Image.Image:
    canvas = Image.new("RGBA", (W, H), PAL["0"])
    for y in range(0, H, 4):
        rect(canvas, 0, y, W, y + 1, "1")
    title = "SCEGLI IL TUO COMPAGNO"
    big_text(canvas, (W - text_width(title) * 2) // 2, 24, title, "w")
    card_w, card_h, gap = 88, 120, 12
    x0 = (W - 4 * card_w - 3 * gap) // 2
    for i in range(4):
        x, y = x0 + i * (card_w + gap), 60
        if i == 3:
            panel(canvas, x, y, x + card_w, y + card_h, "3", "0", "1")
            lock = ui("lock")
            canvas.alpha_composite(lock, (x + (card_w - lock.width) // 2, y + 40))
            text(canvas, x + (card_w - text_width("???")) // 2, y + 64, "???", "3")
            continue
        cid, name, stats, border = COMPANIONS[i]
        selected = i == 0
        if selected:
            rect(canvas, x - 2, y - 2, x + card_w + 2, y + card_h + 2, "y")
        panel(canvas, x, y, x + card_w, y + card_h, "g" if selected else border)
        canvas.alpha_composite(ui(f"portrait_{cid}"), (x + (card_w - 32) // 2, y + 12))
        text(canvas, x + (card_w - text_width(name)) // 2, y + 52, name, "w")
        for j, (label, value, fill) in enumerate(zip(("ATT", "VEL", "AREA"), stats, "fle")):
            by = y + 68 + j * 12
            text(canvas, x + 8, by, label, "4")
            light, dark = {"f": ("y", "o"), "l": ("L", "c"), "e": ("i", "E")}[fill]
            bar(canvas, x + 30, by, 50, value, fill, light, dark)
        canvas.alpha_composite(ui(f"icon_{('fire', 'water', 'nature')[i]}"),
                               (x + card_w - 22, y + 6))
    hint = "INVIO PER INIZIARE"
    text(canvas, (W - text_width(hint)) // 2, 200, hint, "4")
    return canvas


def scene_boss() -> Image.Image:
    canvas = Image.new("RGBA", (W, H))
    canvas.alpha_composite(meadow_map().crop((0, 0, W, H)))
    layer = Layer()
    for x, y in ((60, 90), (420, 70), (90, 250)):
        layer.actor("sprites/tree.png", x, y)
    layer.actor("sprites/boss_enemy.png", 240, 120)
    px, py = 240, 214
    layer.actor("sprites/summoner.png", px, py)
    layer.actor("sprites/ember_companion.png", px - 18, py + 2)
    layer.actor("sprites/tide_companion.png", px + 19, py + 2)
    layer.actor("sprites/sprout_companion.png", px, py + 12)
    for x, y in ((170, 120), (318, 128)):
        layer.actor("sprites/elite_enemy.png", x, y)
    rng = random.Random(3)
    for _ in range(26):
        x, y = rng.randrange(20, W - 20), rng.randrange(60, 200)
        if abs(x - 240) < 50 and y < 140:
            continue
        kind = rng.choice(["blob_enemy", "swarm_enemy", "blob_enemy"])
        layer.actor(f"sprites/{kind}.png", x, y)
    for i in range(14):
        a = math.tau * i / 14
        layer.centered("sprites/telegraph.png", int(px + math.cos(a) * 30),
                       int(py - 8 + math.sin(a) * 18), depth=-1)
    for x, y in ((232, 180), (246, 160), (226, 150)):
        layer.centered("sprites/fireball.png", x, y)
    layer.draw(canvas)
    hud(canvas, 21, 2410, "09:47")
    skull = ui("skull")
    name = "BOSS (NOME DAI DATI)"
    x0 = (W - 200) // 2
    canvas.alpha_composite(skull, (x0 - skull.width - 4, 34))
    text(canvas, (W - text_width(name)) // 2, 27, name, "w")
    bar(canvas, x0, 36, 200, 0.7, "r", "X", "R")
    return canvas


def scene_tiles() -> Image.Image:
    canvas = Image.new("RGBA", (W, H), PAL["0"])

    def material(x: int, y: int) -> str:
        if x < 1 or y < 1 or x >= 17 or y >= 15:
            return "hedge"
        if ((x - 5) / 3.2) ** 2 + ((y - 5) / 2.3) ** 2 <= 1 or (x == 5 and 6 <= y < 12):
            return "dirt"
        if ((x - 12) / 2.6) ** 2 + ((y - 10) / 2.6) ** 2 <= 1 or (x, y) in ((8, 11), (9, 11)):
            return "dirt"
        return "grass"

    canvas.alpha_composite(draw_map(material, 18, 16), (4, 7))
    sheet = art("tiles/meadow_tileset.png")
    count = sheet.height // TILE
    for i in range(count):
        tile = sheet.crop((0, i * TILE, TILE, (i + 1) * TILE))
        canvas.alpha_composite(tile, (300 + (i % 9) * 19, 7 + (i // 9) * 19))
    text(canvas, 300, 108, f"{count} TILE: 0-11 ERBA, TERRA, SIEPE", "4")
    text(canvas, 300, 116, "12-26 BORDI TERRA, 27-41 BORDI SIEPE", "4")
    return canvas


def save(img: Image.Image, name: str) -> None:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    big = img.resize((img.width * SCALE, img.height * SCALE), Image.NEAREST)
    big.convert("RGB").save(EVIDENCE / name)
    print(f"  docs/evidence/{name}  {img.width}x{img.height} x{SCALE}")


def main() -> None:
    save(scene_gameplay(), "art-preview-scene.png")
    save(scene_draft(), "art-preview-draft.png")
    save(scene_select(), "art-preview-select.png")
    save(scene_boss(), "art-preview-boss.png")
    save(scene_tiles(), "art-preview-tiles.png")


if __name__ == "__main__":
    main()
