"""Tileset procedurale dell'arena (assets/tiles/meadow_tileset.png), seed fisso.

Usato da scripts/build-assets.py. Gli indici dei tile (TILE_NAMES) vanno tenuti allineati con
src/world/tiles.rs e docs/ART_BIBLE.md.
"""

from __future__ import annotations

import random
import sys

from PIL import Image

from art_common import render_grid

TILE = 16
TILESET_SEED = 20261022

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


# Timbri d'erba, stile del player (forme tozze, tre toni, luce da nord-ovest): ciuffi a tre e a
# due fili con la punta chiara e la base in ombra, macchie di sole tonde, zolle in ombra.
TUFT_3 = ["b.b.b", "AbAbA", ".AAA."]
TUFT_2 = ["b.b", "AbA", ".A."]
BLADE = ["b", "A"]
CLUMP = [".bb.", "bAAb", ".AA."]
SUN = [".bbb.", "bbbbb", ".bbb."]


def grass_base(rng: random.Random, tufts: int, blades: int, dashes: int) -> list[list[str]]:
    grid = blank("a")
    for _ in range(dashes):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), CLUMP)
    for _ in range(dashes // 2):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "B")
    for _ in range(blades):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), BLADE)
    for i in range(tufts):
        art = TUFT_3 if i % 2 == 0 else TUFT_2
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), art)
    return grid


def flower(grid, x: int, y: int, petal: str, shade: str, center: str) -> None:
    """Fiore tozzo a quattro petali: petalo in basso in ombra, bordo scuro e ombra a terra."""
    stamp(grid, x - 2, y - 2, [
        ".BpB.".replace("p", petal),
        "B" + petal + center + petal + "B",
        "." + petal + shade + "BA",
        "..BA.",
    ])


def pebble(grid, x: int, y: int) -> None:
    stamp(grid, x, y, [".AA.", "A6tA", "AtTA", ".AA."])


def make_grass(name: str, rng: random.Random) -> list[list[str]]:
    if name == "grass_a":
        return grass_base(rng, tufts=3, blades=3, dashes=3)
    if name == "grass_b":
        return grass_base(rng, tufts=5, blades=4, dashes=4)
    if name == "grass_c":
        grid = grass_base(rng, tufts=2, blades=2, dashes=1)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), SUN)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), SUN)
        return grid
    if name == "grass_tuft":
        grid = grass_base(rng, tufts=1, blades=2, dashes=2)
        stamp(grid, 3, 4, ["..b.b.b..", ".bAbAbAb.", "bAAbAbAAb", "AABAAABAA", ".BBBBBBB."])
        return grid
    if name == "grass_tall":
        grid = grass_base(rng, tufts=1, blades=1, dashes=2)
        stamp(grid, 1, 2, ["b...b", "Ab.bA", "AAbAA", "BAAAB", ".BBB."])
        stamp(grid, 7, 7, ["..b.b.b..", "b.A.b.Ab.", "AbAbAbAAb", "AAAAAAAAA", ".BBBBBBB."])
        return grid
    if name == "grass_clover":
        grid = grass_base(rng, tufts=1, blades=2, dashes=1)
        for x, y in ((3, 3), (10, 8), (4, 11)):
            stamp(grid, x, y, [".bb.", "bAbb", "bbAb", ".bA."])
        return grid
    if name == "grass_pebbles":
        grid = grass_base(rng, tufts=2, blades=2, dashes=1)
        pebble(grid, 2, 3)
        pebble(grid, 9, 9)
        stamp(grid, 6, 12, ["6t", "AA"])
        return grid
    if name in ("flowers_white", "flowers_pink", "flowers_yellow"):
        grid = grass_base(rng, tufts=1, blades=2, dashes=2)
        petal, shade, center = {
            "flowers_white": ("w", "t", "y"),
            "flowers_pink": ("m", "M", "y"),
            "flowers_yellow": ("y", "G", "f"),
        }[name]
        # un ciuffo di tre fiori, non sparsi su tutto il tile
        for x, y in ((5, 5), (11, 7), (6, 11)):
            flower(grid, x, y, petal, shade, center)
        return grid
    sys.exit(f"tile sconosciuto: {name}")


def make_dirt(rng: random.Random) -> list[list[str]]:
    """Terra battuta: crepe, granelli chiari, sassolini tozzi con contorno e ombra."""
    grid = blank("d")
    for _ in range(6):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["DD", ".D"])
    for _ in range(5):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), ["7", "D"])
    for _ in range(3):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "8")
    for _ in range(2):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), [".7.", "7dD", ".D8"])
    return grid


# Cespugli della siepe: ciuffi tondi e tozzi con luce in alto a sinistra e bordo verde scuro,
# sfalsati su due file e avvolti sui bordi del tile.
HEDGE_CLUMP = [
    "..NNN..",
    ".NbaaN.",
    "NbaaaAN",
    "NaaaAAN",
    "NAAABBN",
    ".NBBBN.",
]


def make_hedge(rng: random.Random) -> list[list[str]]:
    grid = blank("B")
    for x, y in ((0, 0), (8, 0), (4, 5), (12, 5), (0, 10), (8, 10), (4, 15), (12, 15)):
        stamp(grid, x + rng.randrange(2) - 1, y - 1, HEDGE_CLUMP)
    for _ in range(3):
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
                # contorno quasi nero del cespuglio dove confina con l'erba (come gli attori),
                # bordo di luce dentro il contorno a nord e a ovest, faccia in ombra a sud
                if any(grass_at(x + dx, y + dy) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                    grid[y][x] = "k"
                elif grass_at(x, y - 2) or grass_at(x - 2, y):
                    grid[y][x] = "a"
                elif grass_at(x, y + 2):
                    grid[y][x] = "N"
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
