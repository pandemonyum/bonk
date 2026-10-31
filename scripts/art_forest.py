"""Bioma Bosco Ombroso (D-068): tileset del terreno e delle decorazioni, seed fisso.

Usato da scripts/build-assets.py. Il codice legge i tile per nome: l'ordine di
FOREST_TILE_NAMES e FOREST_DECOR_NAMES è l'indice nel PNG e va tenuto allineato con
game-data/stages.ron e docs/ART_BIBLE.md (sezione "Bosco Ombroso"). Tile nuovi solo in coda.

- assets/tiles/forest_tileset.png: terreno opaco a colonna 16x(16*41). Muschio verde scuro e
  freddo con le sue varianti, radure d'erba al sole, tappeti di foglie, muro di sottobosco
  fitto, bordi morbidi a 15 maschere di radura e muro (stessa tecnica di art_tileset.make_edge).
- assets/tiles/forest_decor.png: decorazioni calpestabili 16x(16*8) con alpha 0/255, disegnate a
  mano in assets-src/biomes/forest_decor.px e cotte dal codice sopra il terreno.

Colori: palette propria assets-src/biomes/forest.palette.txt.
"""

from __future__ import annotations

import random
import sys

from PIL import Image

from art_common import SRC, load_palette, parse_blocks, render_grid
from art_tileset import EDGE_BITS, TILE, blank, edge_name, grass_base, outside_mask, put, stamp

FOREST_SEED = 20261031
FOREST_PALETTE = SRC / "biomes" / "forest.palette.txt"
FOREST_DECOR_SOURCE = SRC / "biomes" / "forest_decor.px"

# Ordine = indice nel tileset (contratto con il codice, D-068).
FOREST_TILE_NAMES = [
    "moss_a",
    "moss_b",
    "moss_leaves",
    "moss_roots",
    "clearing",
    "thicket",
    "moss_c",
    "moss_twigs",
    "clearing_b",
    "leaves_a",
    "leaves_b",
    # bordi: radura con il muschio che entra dai lati (maschera 1..15), poi il muro
    *[edge_name("clearing", m) for m in range(1, 16)],
    *[edge_name("thicket", m) for m in range(1, 16)],
]

FOREST_DECOR_NAMES = [
    "fern",
    "mushrooms_tan",
    "mushrooms_white",
    "stones_moss",
    "ruin_slab",
    "ruin_rubble",
    "twigs",
    "flowers_shade",
]


# --- Muschio ------------------------------------------------------------------------------
# Fondo `m`, dettagli a basso contrasto (`l` luce, `M` ombra): il sottobosco resta più spento
# degli attori. Timbri avvolti sui bordi del tile come nel prato.
CUSHION = [".ll.", "lmmM", ".MM."]
CUSHION_SMALL = [".l.", "lmM", ".M."]
DOT = ["l", "M"]
FLECK = ["..ll...", ".lLLl..", "lLLLLll", ".lLLLLl", "..llLl.", "....l.."]
FLECK_SMALL = [".l.", "lLl", ".l."]
# Rametti (corteccia `D`, luce `7`, ombra `M` sul muschio) e aghi caduti.
TWIGS = (
    ["7DD...", "MMDDD.", "...MMD", ".....M"],
    ["...D", "7DD.", "MM.."],
)
NEEDLES = (["h.", ".G"], ["Gh"], ["G", "h"])
# Foglie cadute: corpo, punta chiara e lato in ombra, toni spenti (ocra o marrone, mai rosso).
# La luce (`G`, `d`) sta solo sulla punta di poche foglie: con l'ocra chiaro il tappeto sembrava
# oro e nascondeva le monete.
LEAVES_OCHRE = (["Gh.", ".h8"], [".G", "hh"], ["hG", "8h"], ["Gh", ".8"], ["hh.", ".h8"])
LEAVES_BROWN = (["DD.", ".D8"], [".D", "88"], ["dD", "D8"], ["D.", "88"], ["DD.", ".88"])


def moss_base(rng: random.Random, cushions: int, dots: int, specks: int) -> list[list[str]]:
    grid = blank("m")
    for _ in range(specks):
        put(grid, rng.randrange(TILE), rng.randrange(TILE), "M")
    for _ in range(dots):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), DOT)
    for i in range(cushions):
        art = CUSHION if i % 2 == 0 else CUSHION_SMALL
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), art)
    return grid


def inner(rng: random.Random, margin: int, size: int) -> int:
    """Coordinata casuale per un timbro largo `size` che resta a `margin` px dai bordi."""
    return rng.randrange(margin, TILE - margin - size + 1)


def make_moss(name: str, rng: random.Random) -> list[list[str]]:
    if name == "moss_a":
        return moss_base(rng, cushions=4, dots=3, specks=4)
    if name == "moss_b":
        return moss_base(rng, cushions=6, dots=4, specks=5)
    if name == "moss_c":
        # chiazze di luce filtrata dalle chiome
        grid = moss_base(rng, cushions=3, dots=2, specks=3)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), FLECK)
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), FLECK_SMALL)
        return grid
    if name == "moss_twigs":
        grid = moss_base(rng, cushions=3, dots=2, specks=3)
        for art in TWIGS:
            stamp(grid, inner(rng, 1, len(art[0])), inner(rng, 1, len(art)), art)
        for _ in range(2):
            stamp(grid, rng.randrange(TILE), rng.randrange(TILE), rng.choice(NEEDLES))
        return grid
    if name == "moss_leaves":
        grid = moss_base(rng, cushions=3, dots=2, specks=3)
        for art in (["Gh.", ".h8"], [".dD", "D8."]):
            stamp(grid, inner(rng, 1, 3), inner(rng, 1, 2), art)
        return grid
    if name == "moss_roots":
        grid = moss_base(rng, cushions=3, dots=2, specks=3)
        # radice che affiora: luce `d`/`7` in alto, corpo `D`, lato in ombra `8`, ombra sul
        # muschio `n`; le punte rientrano nel muschio prima dei bordi del tile
        stamp(grid, 2, 5, [
            "..dDD.......",
            "dD888DDd....",
            "8n.nn888DDd.",
            "......nn88Dd",
            ".........n8D",
            "..........n.",
        ])
        stamp(grid, 4, 11, ["dDd.", "8n8D", "...n"])
        return grid
    sys.exit(f"tile del bosco sconosciuto: {name}")


# --- Radura -------------------------------------------------------------------------------
# Erba al sole con la rampa del prato (b/a/A): più chiara del muschio, gli attori ci si leggono
# come sul prato.
CLOVER = [".bb.", "bAbb", "bbAb", ".bA."]
TINY_FLOWER = [".w.", "wyw", ".A."]


def make_clearing(name: str, rng: random.Random) -> list[list[str]]:
    grid = grass_base(rng, tufts=2, blades=3, dashes=2)
    stamp(grid, rng.randrange(TILE), rng.randrange(TILE), [".bbb.", "bbbbb", ".bbb."])
    if name == "clearing_b":
        for x, y in ((2, 3), (10, 9)):
            stamp(grid, x, y, CLOVER)
        for x, y in ((9, 2), (12, 5), (3, 10), (6, 12)):
            stamp(grid, x, y, TINY_FLOWER)
    return grid


# --- Tappeti di foglie --------------------------------------------------------------------


def make_leaves(name: str, rng: random.Random) -> list[list[str]]:
    """Foglie sparse sul muschio, più fitte al centro e più rade verso i bordi; l'ultimo pixel
    di ogni lato resta muschio. Si raccorda al muschio da tutti i lati e due tappeti vicini si
    toccano con le foglie rade dei margini. Un tappeto pieno faceva quadrati (o pallini, se
    tondo) ripetuti a griglia dove il codice ne mette tanti vicini."""
    grid = moss_base(rng, cushions=2, dots=1, specks=2)
    main, other = LEAVES_OCHRE, LEAVES_BROWN
    if name == "leaves_b":
        main, other = other, main
    for _ in range(46):
        art = rng.choice(main if rng.random() < 0.7 else other)
        w, h = len(art[0]), len(art)
        x, y = rng.randrange(1, TILE - w), rng.randrange(1, TILE - h)
        # distanza dal centro del tile, 0 al centro e 1 sul bordo, in norma "rotonda"
        cx, cy = (x + w / 2 - TILE / 2) / (TILE / 2), (y + h / 2 - TILE / 2) / (TILE / 2)
        r = (cx * cx + cy * cy) ** 0.5
        if rng.random() < 1.3 - 0.8 * r:
            stamp(grid, x, y, art)
    return grid


# --- Muro di sottobosco fitto -------------------------------------------------------------
# Cespugli di rovo tondi con le punte delle foglie, luce in alto a sinistra, bordo `n` sul fondo
# profondo `Z`; fra i cespugli qualche ramo di rovo. Più scuro del muschio: si legge come muro.
THICKET_CLUMP = [
    "..n.nn...",
    ".nlLlln..",
    "nlLLllmn.",
    "nlLlmmmMn",
    "nlmmmmMMn",
    ".nmmMMMn.",
    "..nnMnn..",
]
BRAMBLE = (["8h.", ".8h"], ["h8", ".8"], [".h", "8."])


def make_thicket(rng: random.Random) -> list[list[str]]:
    grid = blank("Z")
    for _ in range(5):
        stamp(grid, rng.randrange(TILE), rng.randrange(TILE), rng.choice(BRAMBLE))
    for x, y in ((0, 0), (8, 0), (4, 5), (12, 5), (0, 10), (8, 10), (4, 15), (12, 15)):
        stamp(grid, x + rng.randrange(3) - 1, y - 1 + rng.randrange(2), THICKET_CLUMP)
    return grid


# --- Bordi --------------------------------------------------------------------------------
# Profondità (px) del muschio che entra in un tile di bordo, per lato: profili irregolari che
# valgono 2 agli estremi, così due bordi affiancati si raccordano senza scalini.
CLEARING_PROFILE = {
    "n": [2, 2, 3, 4, 3, 2, 2, 3, 3, 4, 3, 2, 2, 3, 2, 2],
    "e": [2, 2, 3, 3, 4, 3, 2, 2, 3, 4, 4, 3, 2, 3, 2, 2],
    "s": [2, 3, 3, 2, 2, 3, 4, 4, 3, 2, 2, 3, 3, 2, 2, 2],
    "w": [2, 2, 3, 2, 2, 3, 4, 3, 2, 2, 3, 4, 3, 3, 2, 2],
}
# Il muro ha gobbe più larghe (cespugli tondi), come la siepe del prato.
THICKET_PROFILE = {
    "n": [2, 3, 3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 3, 4, 3, 2],
    "e": [2, 3, 4, 4, 3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 3, 2],
    "s": [2, 2, 3, 4, 4, 3, 2, 2, 2, 3, 4, 4, 3, 2, 2, 2],
    "w": [2, 2, 2, 3, 4, 4, 3, 2, 2, 3, 4, 4, 3, 2, 2, 2],
}


def make_forest_edge(material, floor, mask: int, wall: bool) -> list[list[str]]:
    """Tile di bordo: `floor` (il muschio) entra dai lati di `mask` dentro `material`."""
    profile = THICKET_PROFILE if wall else CLEARING_PROFILE
    out = [[outside_mask(mask, profile, x, y) for x in range(TILE)] for y in range(TILE)]

    def floor_at(x: int, y: int) -> bool:
        return 0 <= x < TILE and 0 <= y < TILE and out[y][x]

    def touches_floor(x: int, y: int) -> bool:
        return any(floor_at(x + dx, y + dy) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))

    def touches_material(x: int, y: int) -> bool:
        return any(
            0 <= x + dx < TILE and 0 <= y + dy < TILE and not out[y + dy][x + dx]
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))
        )

    grid = [row[:] for row in material]
    for y in range(TILE):
        for x in range(TILE):
            if out[y][x]:
                char = floor[y][x]
                if wall:
                    # ombra portata del muro sul muschio, verso sud ed est (luce da NO)
                    if y > 0 and not floor_at(x, y - 1):
                        char = "n"
                    elif y > 1 and not floor_at(x, y - 2):
                        char = "M"
                    elif x > 0 and not floor_at(x - 1, y):
                        char = "M"
                elif touches_material(x, y):
                    # il muschio prende luce dove confina con la radura al sole
                    char = "l"
                grid[y][x] = char
            elif wall:
                # contorno quasi nero dove il muro confina con il pavimento (come gli attori),
                # bordo di luce dentro a nord e a ovest, faccia in ombra a sud
                if touches_floor(x, y):
                    grid[y][x] = "k"
                elif floor_at(x, y - 2) or floor_at(x - 2, y):
                    grid[y][x] = "l"
                elif floor_at(x, y + 2) or floor_at(x + 2, y):
                    grid[y][x] = "Z"
            elif touches_floor(x, y):
                # l'erba della radura passa in ombra dove la copre il bosco
                grid[y][x] = "A"
    return grid


# --- Build --------------------------------------------------------------------------------


def to_image(grids: dict[str, list[list[str]]], names: list[str], palette) -> Image.Image:
    sheet = Image.new("RGBA", (TILE, TILE * len(names)))
    for i, name in enumerate(names):
        rows = ["".join(r) for r in grids[name]]
        if len(rows) != TILE or any(len(r) != TILE for r in rows):
            sys.exit(f"{name}: il tile deve essere {TILE}x{TILE}")
        sheet.paste(render_grid(rows, palette, name), (0, i * TILE))
    return sheet


def build_forest_tileset() -> Image.Image:
    palette = load_palette(FOREST_PALETTE)
    rng = random.Random(FOREST_SEED)
    grids: dict[str, list[list[str]]] = {}
    for name in FOREST_TILE_NAMES:
        if name.startswith("moss_"):
            grids[name] = make_moss(name, rng)
        elif name in ("clearing", "clearing_b"):
            grids[name] = make_clearing(name, rng)
        elif name in ("leaves_a", "leaves_b"):
            grids[name] = make_leaves(name, rng)
        elif name == "thicket":
            grids[name] = make_thicket(rng)
        elif name.startswith(("clearing_", "thicket_")):
            prefix, sides = name.split("_")
            mask = sum(bit for bit, side in EDGE_BITS if side in sides)
            wall = prefix == "thicket"
            grids[name] = make_forest_edge(grids[prefix], grids["moss_a"], mask, wall)
        else:
            sys.exit(f"tile del bosco senza generatore: {name}")
    return to_image(grids, FOREST_TILE_NAMES, palette)


def build_forest_decor() -> Image.Image:
    palette = load_palette(FOREST_PALETTE)
    blocks = parse_blocks(FOREST_DECOR_SOURCE)
    if list(blocks) != FOREST_DECOR_NAMES:
        sys.exit(f"{FOREST_DECOR_SOURCE.name}: timbri {list(blocks)}, attesi {FOREST_DECOR_NAMES}")
    grids = {name: [list(row) for row in rows] for name, rows in blocks.items()}
    return to_image(grids, FOREST_DECOR_NAMES, palette)
