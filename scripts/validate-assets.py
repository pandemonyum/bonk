#!/usr/bin/env python3
"""Valida gli asset in assets/ contro manifest e ART_BIBLE.

Controlli: ogni PNG è nel manifest (e viceversa), dimensioni dichiarate = reali, nomi
snake_case, tileset a colonna di tile 16x16 con un nome snake_case unico per tile (il terreno
opaco; le decorazioni, tipo "decor" di D-068, con almeno un pixel per tile e il bordo di 1 px
trasparente), sprite e icone UI entro i limiti di dimensione (i fogli
animati fotogramma per fotogramma: misura multipla del fotogramma, nessun fotogramma vuoto, base
toccata se l'ancora è in basso al centro), solo
colori della palette per gli asset generati (quella propria dell'asset se il manifest ha
"palette", come il logo), licenza presente. Le illustrazioni fornite dall'utente (tipo
"illustration", D-063) non sono pixel art del progetto: niente controllo di palette e alpha, ma
limiti propri di misura e di peso e la sorgente in assets-src/. Le maschere e il vortice delle
animazioni della home (tipi "mask" e "plate", D-067) stanno dentro l'illustrazione da cui
vengono ("of", "origin"), senza trasparenza; ogni font ha accanto il suo
OFL.txt e una riga in ASSET_LICENSES.md. Avvisi (non errori) se un
upgrade di game-data/upgrades.ron non ha l'icona ui/upgrade_<id>.png o un compagno di
game-data/companions.ron non ha il ritratto ui/portrait_<id>.png.

Audio (D-070, docs/audio/AUDIO_BIBLE.md; solo libreria standard, via audio_common): in
assets/audio/ solo catalog.ron, .wav e .ogg; ogni file con nome ^[a-z0-9_]+\\.(wav|ogg)$, una
voce audio nel manifest (sfx, music, ambience: cartella e formato del tipo), una licenza
ammessa, una riga in ASSET_LICENSES.md e la sorgente se è in assets-src/. Gli effetti sono WAV
PCM 16 bit mono 44 100 Hz, al massimo 1,5 s, con il picco entro -1 dBFS, e coincidono con la
voce del manifest (durata, canali, frequenza, sha256); gli Ogg hanno l'header Vorbis. Ogni
percorso "audio/...wav|ogg" citato in assets/audio/catalog.ron è nel manifest.

Uso: python scripts/validate-assets.py   (exit code 1 se ci sono errori)
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path

from PIL import Image

# audio_common sta accanto a questo file (python -I non mette la cartella dello script nel path).
sys.path.insert(0, str(Path(__file__).resolve().parent))

import audio_common as audio  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
GAME_DATA = ROOT / "game-data"
PALETTE = ROOT / "assets-src" / "palette.txt"
NAME_RE = re.compile(r"^[a-z0-9_]+\.png$")
# Campi sprite e tileset nei file RON di game-data/ (es. sprite: "sprites/x.png").
DATA_SPRITE_RE = re.compile(r'\b(?:sprite|projectile_sprite|tileset)\s*:\s*"([^"]+)"')
TILE_SIZE = 16  # lato dei tile dei tileset (ART_BIBLE, "Griglia e dimensioni")
TILE_NAME_RE = re.compile(r"^[a-z0-9_]+$")
MAX_SPRITE = 64  # sprite singoli; vedi docs/ART_BIBLE.md
MAX_FRAME = 96  # fotogrammi dei fogli animati (il boss e' 96x80)
MAX_UI = 40  # icone e ritratti dell'interfaccia (i ritratti dei compagni sono 40x36)
MAX_LOGO = (256, 128)  # logo della home a texel nativi (il canvas va da 480x270 a 640x360)
MAX_ILLUSTRATION = 2048  # lato massimo di un'illustrazione (texture anche per WebGL2)
MAX_ILLUSTRATION_BYTES = 4 * 2**20  # peso del PNG: il build web scarica tutto assets/
DATA_ID_RE = re.compile(r'^\s*id\s*:\s*"([a-z0-9_]+)"')


def palette_colors(path: Path = PALETTE) -> set[tuple[int, int, int]]:
    colors = set()
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if line and not line.startswith("#"):
            rgb = int(line.split()[1], 16)
            colors.add(((rgb >> 16) & 0xFF, (rgb >> 8) & 0xFF, rgb & 0xFF))
    return colors


def check_sheet(path: str, entry: dict, img: Image.Image) -> list[str]:
    """Foglio animato: misura multipla intera del fotogramma, griglia coerente con il manifest,
    ogni fotogramma entro il limite dei fotogrammi (MAX_FRAME) e non vuoto; con ancora in basso al centro
    ogni fotogramma tocca la riga di base (piedi o ombra a terra)."""
    errors: list[str] = []
    fw, fh = entry["frame"]
    if fw <= 0 or fh <= 0 or img.width % fw or img.height % fh:
        return [f"{path}: {img.width}x{img.height} non è multiplo del fotogramma {fw}x{fh}"]
    if max(fw, fh) > MAX_FRAME:
        errors.append(f"{path}: fotogramma {fw}x{fh} oltre {MAX_FRAME}px")
    columns, rows = img.width // fw, img.height // fh
    if entry.get("columns", columns) != columns:
        errors.append(f"{path}: 'columns' {entry['columns']} ma la griglia ne ha {columns}")
    if "rows" in entry and len(entry["rows"]) != rows:
        errors.append(f"{path}: {len(entry['rows'])} nomi di riga ma la griglia ne ha {rows}")
    if entry.get("frames", columns * rows) != columns * rows:
        errors.append(f"{path}: 'frames' {entry['frames']} invece di {columns * rows}")
    names = entry.get("rows", [str(r) for r in range(rows)])
    alpha = img.getchannel("A")
    for r in range(rows):
        for c in range(columns):
            label = f"{names[r] if r < len(names) else r} {c}"
            box = alpha.crop((c * fw, r * fh, (c + 1) * fw, (r + 1) * fh))
            if box.getbbox() is None:
                errors.append(f"{path}: fotogramma {label} vuoto")
            elif entry.get("anchor") == "bottom_center":
                if box.crop((0, fh - 1, fw, fh)).getbbox() is None:
                    errors.append(f"{path}: fotogramma {label} non tocca la riga di base")
    return errors


def check_tileset(path: str, entry: dict, img: Image.Image) -> list[str]:
    """Tileset a colonna (D-068): tile 16x16, un nome per tile nell'ordine dei tile. Il terreno
    ("tileset") è opaco; le decorazioni ("decor") hanno almeno un pixel per tile e il bordo di
    1 px trasparente, così si cuociono sopra qualunque terreno senza toccare i tile vicini."""
    errors: list[str] = []
    t, names = entry.get("tile_size"), entry.get("tiles", [])
    if t != TILE_SIZE:
        return [f"{path}: tile_size {t}, atteso {TILE_SIZE}"]
    if img.width != t or img.height != t * len(names):
        return [f"{path}: atteso {t}x{t * len(names)} ({len(names)} tile in colonna)"]
    bad = [n for n in names if not TILE_NAME_RE.match(n)]
    if bad:
        errors.append(f"{path}: nomi di tile non snake_case {bad}")
    if len(set(names)) != len(names):
        errors.append(f"{path}: nomi di tile ripetuti")
    alpha = img.getchannel("A")
    if entry["kind"] == "tileset":
        if alpha.getextrema()[0] != 255:
            errors.append(f"{path}: il terreno deve essere opaco (alpha 255 ovunque)")
        return errors
    for i, name in enumerate(names):
        tile = alpha.crop((0, i * t, t, (i + 1) * t))
        box = tile.getbbox()
        if box is None:
            errors.append(f"{path}: tile {name} vuoto")
        elif box[0] < 1 or box[1] < 1 or box[2] > t - 1 or box[3] > t - 1:
            errors.append(f"{path}: tile {name} tocca il bordo (serve 1 px trasparente)")
    return errors


def check_overlay(path: str, entry: dict, img: Image.Image, entries: dict) -> list[str]:
    """Maschera o vortice delle animazioni della home: dentro l'illustrazione, opachi."""
    errors = []
    of = entries.get(entry.get("of", ""))
    if not of or of["kind"] != "illustration":
        return [f"{path}: illustrazione {entry.get('of')!r} assente dal manifest"]
    x, y = entry.get("origin", (-1, -1))
    w, h = img.size
    if x < 0 or y < 0 or x + w > of["size"][0] or y + h > of["size"][1]:
        errors.append(f"{path}: {w}x{h} in {x},{y} esce dall'illustrazione {of['size']}")
    if img.getchannel("A").getextrema()[0] != 255:
        errors.append(f"{path}: trasparenza in una maschera (i canali sono RGB)")
    if not (ROOT / entry.get("source", "")).is_file():
        errors.append(f"{path}: sorgente {entry.get('source')!r} assente")
    return errors


def check_audio_file(rel: str, entry: dict, licenses: str) -> list[str]:
    """Un file di assets/audio/ con la sua voce del manifest (tipo audio)."""
    errors: list[str] = []
    kind = entry["kind"]
    folder, extension = audio.AUDIO_KINDS[kind]
    if not rel.startswith(folder) or not rel.endswith(extension):
        errors.append(f"{rel}: il tipo {kind!r} vuole un file {extension} in assets/{folder}")
    license = entry.get("license", "")
    if not license:
        errors.append(f"{rel}: licenza mancante")
    elif not audio.LICENSE_RE.match(license):
        errors.append(f"{rel}: licenza {license!r} fuori elenco {list(audio.LICENSES)}")
    elif license.startswith("CC-BY-4.0") and not entry.get("attribution"):
        errors.append(f"{rel}: CC-BY-4.0 senza 'attribution'")
    source = entry.get("source", "")
    if source.startswith("assets-src/") and not (ROOT / source).is_file():
        errors.append(f"{rel}: sorgente {source!r} assente")
    if f"assets/{rel}" not in licenses:
        errors.append(f"{rel}: file audio senza riga in ASSET_LICENSES.md")

    path = ASSETS / rel
    if rel.endswith(".ogg"):
        try:
            channels, rate = audio.read_ogg_vorbis(path)
        except audio.AudioFileError as e:
            return errors + [f"{rel}: Ogg Vorbis non valido: {e}"]
        found = {"channels": channels, "sample_rate": rate}
    else:
        try:
            info = audio.read_wav(path)
        except audio.AudioFileError as e:
            return errors + [f"{rel}: WAV non leggibile: {e}"]
        if info.format_tag != audio.WAVE_FORMAT_PCM:
            errors.append(f"{rel}: WAV non PCM (formato {info.format_tag})")
        if info.bits != audio.SFX_BITS:
            errors.append(f"{rel}: {info.bits} bit, gli effetti sono PCM {audio.SFX_BITS} bit")
        if info.channels != audio.SFX_CHANNELS:
            errors.append(f"{rel}: {info.channels} canali, gli effetti sono mono")
        if info.sample_rate != audio.SAMPLE_RATE:
            errors.append(f"{rel}: {info.sample_rate} Hz, gli effetti sono a {audio.SAMPLE_RATE} Hz")
        if info.frames == 0:
            errors.append(f"{rel}: nessun campione")
        elif info.duration_s > audio.MAX_SFX_S:
            errors.append(f"{rel}: {info.duration_s:.3f} s, gli effetti stanno entro "
                          f"{audio.MAX_SFX_S} s")
        if info.peak and info.peak_dbfs > audio.MAX_PEAK_DBFS:
            errors.append(f"{rel}: picco {info.peak_dbfs:.2f} dBFS, oltre {audio.MAX_PEAK_DBFS} dBFS")
        found = {"channels": info.channels, "sample_rate": info.sample_rate}
        if "duration_s" in entry and abs(entry["duration_s"] - info.duration_s) > 0.0005:
            errors.append(f"{rel}: durata {info.duration_s:.4f} s diversa dal manifest "
                          f"{entry['duration_s']}")
        if "sha256" in entry and hashlib.sha256(path.read_bytes()).hexdigest() != entry["sha256"]:
            errors.append(f"{rel}: contenuto diverso dallo sha256 del manifest "
                          "(rigenera con scripts/audio_sfx.py)")
    for field, value in found.items():
        if field in entry and entry[field] != value:
            errors.append(f"{rel}: {field} {value} diverso dal manifest {entry[field]}")
    return errors


def check_audio(entries: dict, licenses: str) -> tuple[list[str], list[str], int]:
    """Errori, file audio su disco e numero di percorsi citati in catalog.ron."""
    errors: list[str] = []
    voices = {p: e for p, e in entries.items() if audio.is_audio(e)}
    found: list[str] = []
    for file in sorted(p for p in audio.AUDIO.rglob("*") if p.is_file()):
        rel = file.relative_to(ASSETS).as_posix()
        if file == audio.CATALOG:
            continue
        if file.suffix.lower() not in audio.AUDIO_EXTENSIONS:
            errors.append(f"{rel}: formato non abilitato (in assets/audio/ solo .wav e .ogg)")
            continue
        found.append(rel)
        folders = rel.split("/")[1:-1]
        if not audio.AUDIO_NAME_RE.match(file.name) or not all(
            audio.FOLDER_RE.match(f) for f in folders
        ):
            errors.append(f"{rel}: nome fuori da {audio.AUDIO_NAME_RE.pattern} "
                          "(anche le cartelle: minuscole, cifre e _)")
        if rel not in voices:
            what = "voce non audio" if rel in entries else "nessuna voce"
            errors.append(f"{rel}: file audio senza voce audio nel manifest ({what})")
            continue
        errors += check_audio_file(rel, voices[rel], licenses)
    for missing in sorted(voices.keys() - set(found)):
        errors.append(f"{missing}: nel manifest ma file assente")

    cited = []
    if audio.CATALOG.is_file():
        cited = audio.catalog_files(audio.CATALOG.read_text(encoding="utf-8"))
    for line_no, path in cited:
        if path not in voices:
            errors.append(f"assets/audio/catalog.ron:{line_no}: \"{path}\" assente dal manifest")
    return errors, found, len(cited)


def main() -> int:
    errors: list[str] = []
    manifest = json.loads((ASSETS / "manifest.json").read_text(encoding="utf-8"))
    entries = {e["path"]: e for e in manifest["assets"]}
    # le voci audio (D-070) hanno i loro controlli, in fondo
    images = {p: e for p, e in entries.items() if not audio.is_audio(e)}
    on_disk = {p.relative_to(ASSETS).as_posix() for p in ASSETS.rglob("*.png")}

    for missing in sorted(on_disk - images.keys()):
        errors.append(f"{missing}: PNG non presente nel manifest")
    for missing in sorted(images.keys() - on_disk):
        errors.append(f"{missing}: nel manifest ma file assente")

    colors = palette_colors()
    for path in sorted(on_disk & images.keys()):
        entry = entries[path]
        img = Image.open(ASSETS / path).convert("RGBA")
        if not NAME_RE.match(Path(path).name):
            errors.append(f"{path}: nome non snake_case")
        if list(img.size) != entry["size"]:
            errors.append(f"{path}: dimensione {img.size} diversa dal manifest {entry['size']}")
        if not entry.get("license"):
            errors.append(f"{path}: licenza mancante")
        if entry["kind"] in ("tileset", "decor"):
            errors += check_tileset(path, entry, img)
        elif entry["kind"] == "sprite" and "frame" in entry:
            errors += check_sheet(path, entry, img)
        elif entry["kind"] == "sprite" and max(img.size) > MAX_SPRITE:
            errors.append(f"{path}: sprite oltre {MAX_SPRITE}px")
        elif entry["kind"] == "ui" and max(img.size) > MAX_UI:
            errors.append(f"{path}: icona UI oltre {MAX_UI}px")
        elif entry["kind"] == "logo" and (img.width > MAX_LOGO[0] or img.height > MAX_LOGO[1]):
            errors.append(f"{path}: logo oltre {MAX_LOGO[0]}x{MAX_LOGO[1]}px")
        elif entry["kind"] == "illustration":
            # Fornita dall'utente: colori e sfumature sono i suoi, si controllano solo misura,
            # peso e sorgente (niente palette né alpha 0/255).
            if max(img.size) > MAX_ILLUSTRATION:
                errors.append(f"{path}: illustrazione oltre {MAX_ILLUSTRATION}px per lato")
            weight = (ASSETS / path).stat().st_size
            if weight > MAX_ILLUSTRATION_BYTES:
                limit = MAX_ILLUSTRATION_BYTES / 2**20
                errors.append(f"{path}: {weight / 2**20:.2f} MiB, oltre {limit:g} MiB")
            if not (ROOT / entry.get("source", "")).is_file():
                errors.append(f"{path}: sorgente {entry.get('source')!r} assente")
            continue
        elif entry["kind"] in ("mask", "plate"):
            errors += check_overlay(path, entry, img, entries)
            continue
        # Asset generati da assets-src (anche quelli con licenza propria, per esempio forniti
        # dall'utente e ridisegnati qui): solo colori della loro palette, alpha 0 o 255.
        generated = entry["license"].startswith("original") or entry.get("source", "").startswith(
            "assets-src/"
        )
        if generated:
            own = entry.get("palette")
            if own and not (ROOT / own).is_file():
                errors.append(f"{path}: palette {own} assente")
                continue
            allowed = palette_colors(ROOT / own) if own else colors
            # get_flattened_data da Pillow 12; getdata e' deprecato (rimosso in Pillow 14)
            flat = getattr(img, "get_flattened_data", img.getdata)
            pixels = list(flat())
            stray = {px[:3] for px in pixels if px[3] != 0 and px[:3] not in allowed}
            if stray:
                where = own or "assets-src/palette.txt"
                errors.append(f"{path}: {len(stray)} colori fuori palette ({where})")
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

    # Font: la licenza OFL va distribuita insieme al file e il font va elencato tra le terze parti.
    licenses = (ROOT / "ASSET_LICENSES.md").read_text(encoding="utf-8")
    fonts = sorted(p for p in ASSETS.rglob("*") if p.suffix.lower() in (".ttf", ".otf"))
    for font in fonts:
        rel = font.relative_to(ASSETS).as_posix()
        if not (font.parent / "OFL.txt").is_file():
            errors.append(f"{rel}: manca OFL.txt accanto al font")
        if f"assets/{rel}" not in licenses:
            errors.append(f"{rel}: font senza riga in ASSET_LICENSES.md")

    audio_errors, audio_files, cited = check_audio(entries, licenses)
    errors += audio_errors
    wavs = sum(1 for f in audio_files if f.endswith(".wav"))

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
        f"{'FALLITO' if errors else 'OK'}: {len(on_disk)} PNG e {len(fonts)} font controllati, "
        f"{referenced} sprite citati in game-data/, {len(audio_files)} file audio "
        f"({wavs} WAV, {len(audio_files) - wavs} Ogg), {cited} citati in catalog.ron, "
        f"{len(errors)} errori"
    )
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
