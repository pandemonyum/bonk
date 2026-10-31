"""Costanti, lettori di WAV e Ogg e manifest condivisi dagli script audio (D-070).

Usato da audio_sfx.py (sintesi), build-assets.py (conserva le voci audio del manifest) e
validate-assets.py (controlli audio). Solo libreria standard: niente numpy, soundfile, audioop.
Regole e numeri: docs/audio/AUDIO_BIBLE.md e docs/design/AUDIO_SYSTEM.md (§ 3.13, § 4, § 5).
"""

from __future__ import annotations

import json
import math
import re
import struct
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"
AUDIO = ASSETS / "audio"
MANIFEST = ASSETS / "manifest.json"
CATALOG = AUDIO / "catalog.ron"
SFX_SRC = ROOT / "assets-src" / "audio" / "sfx"

# Effetti (bus Sfx e Ui): WAV PCM 16 bit mono 44 100 Hz, al massimo 1,5 s (AUDIO_SYSTEM § 5.3).
SAMPLE_RATE = 44_100
SFX_CHANNELS = 1
SFX_BITS = 16
MAX_SFX_S = 1.5
FULL_SCALE = 32768  # 0 dBFS di un campione PCM a 16 bit
MAX_PEAK_DBFS = -1.0  # picco di campione massimo ammesso; il bersaglio della sintesi è -3
WAVE_FORMAT_PCM = 1
WAVE_FORMAT_EXTENSIBLE = 0xFFFE

# Tipi audio del manifest, con la cartella e il formato di ognuno (AUDIO_SYSTEM § 4).
AUDIO_KINDS = {
    "sfx": ("audio/sfx/", ".wav"),
    "music": ("audio/music/", ".ogg"),
    "ambience": ("audio/ambience/", ".ogg"),
}
AUDIO_EXTENSIONS = (".wav", ".ogg")
AUDIO_NAME_RE = re.compile(r"^[a-z0-9_]+\.(wav|ogg)$")
FOLDER_RE = re.compile(r"^[a-z0-9_]+$")
# Licenze ammesse senza chiedere (D-070); CC-BY-4.0 vuole anche il campo "attribution".
LICENSES = ("original", "user-owned", "CC0-1.0", "CC-BY-4.0")
# la licenza è una di queste parole, seguita da nulla, da uno spazio, una virgola o una parentesi
LICENSE_RE = re.compile(r"^(?:" + "|".join(re.escape(x) for x in LICENSES) + r")(?=$|[ ,(])")
SFX_LICENSE = "original, project-owned (synthesized from assets-src)"
SFX_GENERATOR = "scripts/audio_sfx.py"
# Un percorso audio citato in catalog.ron (relativo ad assets/).
CATALOG_FILE_RE = re.compile(r'"(audio/[^"\s]+\.(?:wav|ogg))"')


class AudioFileError(Exception):
    """File audio illeggibile o con un header non valido."""


@dataclass(frozen=True)
class WavInfo:
    format_tag: int  # 1 = PCM (anche dentro WAVE_FORMAT_EXTENSIBLE)
    channels: int
    sample_rate: int
    bits: int
    frames: int
    peak: int  # valore assoluto del campione più alto (solo PCM 16 bit, altrimenti 0)

    @property
    def duration_s(self) -> float:
        return self.frames / self.sample_rate if self.sample_rate else 0.0

    @property
    def peak_dbfs(self) -> float:
        return dbfs(self.peak)


def dbfs(peak: float) -> float:
    """Picco in dBFS di un valore assoluto di campione a 16 bit (32768 = 0 dBFS)."""
    return -math.inf if peak <= 0 else 20 * math.log10(peak / FULL_SCALE)


def wav_bytes(samples: list[int], sample_rate: int = SAMPLE_RATE) -> bytes:
    """WAV PCM 16 bit mono con l'header canonico di 44 byte (RIFF, fmt da 16, data)."""
    data = struct.pack(f"<{len(samples)}h", *samples)
    block = SFX_CHANNELS * SFX_BITS // 8
    header = struct.pack(
        "<4sI4s4sIHHIIHH4sI",
        b"RIFF", 36 + len(data), b"WAVE",
        b"fmt ", 16, WAVE_FORMAT_PCM, SFX_CHANNELS, sample_rate, sample_rate * block, block,
        SFX_BITS,
        b"data", len(data),
    )
    return header + data


def parse_wav(data: bytes) -> WavInfo:
    """Header e picco di un WAV letto chunk per chunk (non dipende dal modulo wave)."""
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise AudioFileError("non è un file RIFF/WAVE")
    fmt = pcm = None
    pos = 12
    while pos + 8 <= len(data):
        chunk, size = data[pos : pos + 4], struct.unpack_from("<I", data, pos + 4)[0]
        body = data[pos + 8 : pos + 8 + size]
        if len(body) < size:
            raise AudioFileError(f"chunk {chunk.decode('latin-1')!r} troncato")
        if chunk == b"fmt ":
            fmt = body
        elif chunk == b"data":
            pcm = body
        pos += 8 + size + (size & 1)
    if fmt is None or len(fmt) < 16:
        raise AudioFileError("manca il chunk 'fmt '")
    if pcm is None:
        raise AudioFileError("manca il chunk 'data'")
    tag, channels, rate, _, align, bits = struct.unpack_from("<HHIIHH", fmt)
    if tag == WAVE_FORMAT_EXTENSIBLE and len(fmt) >= 26:
        tag = struct.unpack_from("<H", fmt, 24)[0]  # primi 2 byte del GUID del sottoformato
    if align == 0:
        raise AudioFileError("block align 0")
    frames = len(pcm) // align
    peak = 0
    if tag == WAVE_FORMAT_PCM and bits == 16 and pcm:
        samples = struct.unpack(f"<{len(pcm) // 2}h", pcm[: len(pcm) // 2 * 2])
        peak = max(max(samples), -min(samples))
    return WavInfo(tag, channels, rate, bits, frames, peak)


def read_wav(path: Path) -> WavInfo:
    return parse_wav(path.read_bytes())


def read_ogg_vorbis(path: Path) -> tuple[int, int]:
    """(canali, frequenza) dall'header di identificazione Vorbis della prima pagina Ogg."""
    data = path.read_bytes()[:4096]
    if len(data) < 28 or data[:4] != b"OggS":
        raise AudioFileError("non è un file Ogg (manca 'OggS')")
    start = 27 + data[26]  # header della pagina + tabella dei segmenti
    packet = data[start : start + 30]
    if len(packet) < 16 or packet[:7] != b"\x01vorbis":
        raise AudioFileError("la prima pagina non è l'header di identificazione Vorbis")
    _, channels, rate = struct.unpack_from("<IBI", packet, 7)
    return channels, rate


def load_manifest() -> dict:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def manifest_text(assets: list[dict]) -> str:
    """Lo stesso formato di build-assets.py: `path` prima chiave di ogni voce (lo legge anche
    il test dei biomi, src/world/biome/tests.rs)."""
    return json.dumps({"version": 1, "assets": assets}, indent=2) + "\n"


def is_audio(entry: dict) -> bool:
    return entry.get("kind") in AUDIO_KINDS


def audio_entries(path: Path = MANIFEST) -> list[dict]:
    """Voci audio del manifest già scritto (le scrive audio_sfx.py; il build delle immagini le
    conserva in coda, nello stesso ordine)."""
    if not path.is_file():
        return []
    return [e for e in json.loads(path.read_text(encoding="utf-8"))["assets"] if is_audio(e)]


def catalog_files(text: str) -> list[tuple[int, str]]:
    """(riga, percorso) dei file audio citati in catalog.ron, commenti `//` esclusi."""
    found = []
    for line_no, line in enumerate(text.splitlines(), 1):
        code = line.split("//", 1)[0]
        found += [(line_no, path) for path in CATALOG_FILE_RE.findall(code)]
    return found
