#!/usr/bin/env python3
"""Controlli INGEST delle consegne di musica e ambienti in .audio-inbox/ (D-070).

Procedura e criteri: docs/audio/AUDIO_HANDOFF_WORKFLOW.md (§ 3, "Controlli"). I numeri di ogni
richiesta (docs/audio/requests/<ID>.md) sono riportati nella tabella REQUESTS qui sotto; per un
file fuori tabella si passano da riga di comando. Solo libreria standard e nessuna decodifica
Vorbis: picco, loudness, estremi, continuità e stereo si misurano sul master WAV, se c'è.
Sull'Ogg restano "non misurato" fino al test di decodifica (AUDIO-3, AUDIO-4).

Ogg, con le stesse righe del comando della procedura: pagine, CRC, BOS ed EOS, un solo flusso,
header di identificazione, commenti, granule finale, bitrate medio; in più peso ed encoder.

Master WAV (PCM 16, 24 o 32 bit, oppure float 32):
- formato e frame contro i campioni attesi;
- picco di campione e stima del true peak con interpolazione 4x (gli stessi filtri di
  audio_sfx.py, da audio_common); sui loop l'interpolazione gira attorno al punto di giro;
- loudness integrata BS.1770-4: filtro K a due stadi ricalcolato per la frequenza del file,
  blocchi da 400 ms con passo di 100 ms (75 %), gate assoluto a -70 LUFS e relativo a -10 LU;
- loop: primo e ultimo campione con |x| <= 0,01 in ogni canale; RMS dei primi e degli ultimi
  50 ms non oltre 6 dB sotto il secondo vicino; salto ultimo -> primo campione entro il 99,9°
  percentile delle differenze fra campioni vicini, per canale;
- stinger: primo campione |x| <= 0,01, oltre -40 dBFS entro 10 ms, ultimi 10 ms sotto -60 dBFS;
- loop con BPM, informativo: le stesse misure degli estremi a ogni stanghetta interna, per
  distinguere una dissolvenza al giro dal modo in cui finiscono tutte le battute;
- continuità, informativo: anche la seconda differenza a cavallo del giro (vedi jump_stats);
- stereo: RMS dei canali, destro meno sinistro e correlazione (criterio solo per gli ambienti);
- banda grave, informativa: potenza del mono fino a 30 e fino a 100 Hz (vedi band_power);
- note gravi tenute (euristica, solo dove la tabella lo chiede): vedi low_notes().

Uso:
  python -I scripts/audio_inbox.py MUSIC-HOME-001 --save     ultima versione di ogni parte
  python -I scripts/audio_inbox.py AMB-HOME-001 --version 1
  python -I scripts/audio_inbox.py <file.ogg o .wav> ... [--kind loop --samples N ...]
  python -I scripts/audio_inbox.py assets/audio/music/home.ogg   file già promosso: i criteri
      vengono dalla richiesta scritta in "request" della sua voce del manifest
--save scrive l'uscita anche in .audio-inbox/<ID>/check/v<N>.txt (UTF-8), --out in un file a
scelta. Exit code: 0 se nessun NO (i "non misurato" non contano), 1 con almeno un NO, 2 se un
file manca o non si legge.
"""

from __future__ import annotations

import argparse
import datetime
import heapq
import json
import math
import operator
import re
import struct
import sys
import time
from array import array
from dataclasses import dataclass, replace
from itertools import repeat
from pathlib import Path

# audio_common sta accanto a questo file (python -I non mette la cartella dello script nel path).
sys.path.insert(0, str(Path(__file__).resolve().parent))

from audio_common import (  # noqa: E402
    ASSETS,
    LICENSE_RE,
    ROOT,
    SAMPLE_RATE,
    TRUE_PEAK_TAPS,
    WAVE_FORMAT_PCM,
    AudioFileError,
    true_peak_kernels,
    wav_parts,
)

INBOX = ROOT / ".audio-inbox"
WAVE_FORMAT_FLOAT = 3
NAME_RE = re.compile(r"^(?P<id>(?:MUSIC|AMB)-[A-Z0-9]+-\d{3})(?:_(?P<part>[a-z0-9]+))?"
                     r"_v(?P<ver>\d+)\.(?P<ext>ogg|wav)$")

# --- Criteri comuni (procedura, tabella "Controlli") ----------------------------------------
MASTER_MAX_TP = -1.5  # dBTP sul master; sull'Ogg il limite è -1, ma serve decodificarlo
EDGE_MAX = 0.01  # |x| del primo e dell'ultimo campione (-40 dBFS)
EDGE_S, EDGE_REF_S, EDGE_MAX_DROP_DB = 0.05, 1.0, 6.0  # 50 ms contro il secondo vicino
JUMP_PERCENTILE = 99.9
STING_ATTACK_S, STING_ATTACK_DBFS = 0.010, -40.0
STING_TAIL_S, STING_TAIL_DBFS = 0.010, -60.0
ABS_GATE_LUFS, REL_GATE_LU = -70.0, -10.0
TP_BLOCK = 1024  # campioni per blocco nella ricerca del true peak

# --- Registro grave (euristica, vedi low_notes) ---------------------------------------------
LOW_MIN_HZ, LOW_MAX_HZ = 30.0, 100.0
NOTE_NAMES = ("Do", "Do#", "Re", "Re#", "Mi", "Fa", "Fa#", "Sol", "Sol#", "La", "La#", "Si")
LOW_ALLOWED = ("Fa", "Do")
LOW_REL_DB = 20.0  # una nota "suona" se è entro 20 dB dalla nota grave più forte del file
LOW_ABS_DBFS = -50.0  # ... e sopra -50 dBFS
LOW_HELD = 2  # "tenuta": suona in almeno 2 finestre consecutive
DECIM_TAPS = 1023  # FIR passa-basso (Blackman) prima della decimazione


@dataclass(frozen=True)
class Spec:
    """Criteri di un file. None = nessun criterio: la misura si stampa senza giudizio."""
    kind: str = "loop"  # loop (musica) | ambience (ambiente in loop) | stinger
    samples: int | None = None  # lunghezza attesa (stinger: valore centrale della fascia)
    tol: int = 1  # tolleranza in campioni
    max_bytes: int | None = None  # tetto del peso dell'Ogg
    lufs: float | None = None
    lufs_tol: float = 1.0
    bpm: float | None = None  # finestra di un battito per le note gravi (senza: 1 s)
    beats_per_bar: int = 4  # tutte le richieste sono in 4/4
    right_over_left: tuple[float, float] | None = None  # ambienti: RMS destro - sinistro, dB
    low_notes: str = ""  # "criterio" | "info" | "" (non calcolate)


# Numeri delle richieste in docs/audio/requests/. Chiave della parte: "" se il file è uno solo.
REQUESTS: dict[str, dict[str, Spec]] = {
    "MUSIC-HOME-001": {"": Spec("loop", 3_175_200, 1, 1_152_000, -16.0, 1.0, bpm=80,
                                low_notes="criterio")},
    "MUSIC-RUN-001": {"": Spec("loop", 4_233_600, 1, 1_536_000, -16.0, 1.0, bpm=120)},
    "MUSIC-HORDE-001": {"": Spec("loop", 2_116_800, 1, 768_000, -16.0, 1.0, bpm=120)},
    "MUSIC-BOSS-001": {"": Spec("loop", 3_528_000, 1, 1_280_000, -16.0, 1.0, bpm=144)},
    "MUSIC-STING-001": {
        "boss": Spec("stinger", 110_250, 22_050, 48_000, -16.0, 1.5),
        "victory": Spec("stinger", 176_400, 44_100, 80_000, -16.0, 1.5),
        "defeat": Spec("stinger", 132_300, 44_100, 64_000, -16.0, 1.5),
    },
    "AMB-HOME-001": {
        "portal": Spec("ambience", 1_411_200, 22_050, 512_000, -24.0, 1.5,
                       right_over_left=(6.0, 10.0), low_notes="info"),
        "falls": Spec("ambience", 1_190_700, 22_050, 432_000, -22.0, 1.5,
                      right_over_left=(3.0, 6.0)),
    },
}
PROVENANCE_FILE_KEYS = ("nome", "autore", "strumento", "licenza", "url", "data", "attribuzione",
                        "termini", "master", "note")
PROVENANCE_SOURCE_KEYS = ("per", "descrizione", "autore", "licenza", "url", "data",
                          "licenza salvata")


class Out:
    """Righe dell'uscita (stampate subito e tenute per --save) ed esiti per criterio."""

    def __init__(self) -> None:
        self.lines: list[str] = []
        self.results: list[tuple[str, str, str]] = []
        self.no = 0

    def __call__(self, text: str = "") -> None:
        print(text, flush=True)
        self.lines.append(text)

    def result(self, what: str, ok: bool | str | None, detail: str = "") -> str:
        """ok: True/False = OK/NO; None = non misurato; una stringa = esito a parole."""
        verdict = ok if isinstance(ok, str) else {True: "OK", False: "NO", None: "non misurato"}[ok]
        self.no += verdict == "NO"
        self.results.append((what, verdict, detail))
        return verdict


def db(ratio: float) -> float:
    return 20 * math.log10(ratio) if ratio > 0 else -math.inf


def fmt_db(value: float, digits: int = 2) -> str:
    return "-inf" if value == -math.inf else f"{value:+.{digits}f}"


# --- Ogg ------------------------------------------------------------------------------------


def _crc_table() -> list[int]:
    table = []
    for i in range(256):
        c = i << 24
        for _ in range(8):
            c = ((c << 1) ^ 0x04C11DB7) if c & 0x80000000 else c << 1
        table.append(c & 0xFFFFFFFF)
    return table


CRC_TABLE = _crc_table()


def ogg_crc(data: bytes) -> int:
    """CRC delle pagine Ogg: polinomio 0x04C11DB7, non riflesso, valore iniziale 0."""
    c, table = 0, CRC_TABLE
    for x in data:
        c = ((c << 8) & 0xFFFFFFFF) ^ table[(c >> 24) ^ x]
    return c


def check_ogg(path: Path, spec: Spec, out: Out) -> int | None:
    """Le righe del comando della procedura, poi peso ed encoder. Rende il granule finale."""
    d = path.read_bytes()
    pos, pages, bad = 0, [], []
    while pos + 27 <= len(d):
        if d[pos:pos + 4] != b"OggS":
            bad.append(f"byte {pos}: manca OggS")
            break
        typ, gran, serial, seq, stored, nseg = struct.unpack_from("<xBqIIIB", d, pos + 4)
        size = 27 + nseg + sum(d[pos + 27:pos + 27 + nseg])
        page = bytearray(d[pos:pos + size])
        if len(page) < size:
            bad.append(f"byte {pos}: pagina troncata")
            break
        page[22:26] = bytes(4)
        if ogg_crc(page) != stored:
            bad.append(f"pagina {seq} (byte {pos}): CRC errato")
        pages.append((typ, gran, serial, seq, pos + 27 + nseg, pos + size))
        pos += size
    if pos != len(d):
        bad.append(f"{len(d) - pos} byte fuori dalle pagine")
    container_bad = list(bad)

    ch = rate = nom = 0
    vendor, tags = "?", []
    ident = d[pages[0][4]:pages[0][4] + 30] if pages else b""
    ident_ok = ident[:7] == b"\x01vorbis" and len(ident) >= 28
    if ident_ok:
        ch, rate, _bmax, nom, _bmin, _bs = struct.unpack_from("<IBIiiiB", ident, 7)[1:]
    else:
        bad.append("prima pagina senza header di identificazione Vorbis")
    # l'header dei commenti comincia sulla seconda pagina e può continuare sulle successive
    rest = b"".join(d[p[4]:p[5]] for p in pages[1:8])
    if rest[:7] == b"\x03vorbis":
        try:
            m = struct.unpack_from("<I", rest, 7)[0]
            vendor = rest[11:11 + m].decode("utf-8", "replace")
            j = 11 + m
            count = struct.unpack_from("<I", rest, j)[0]
            j += 4
            for _ in range(min(count, 1000)):
                m = struct.unpack_from("<I", rest, j)[0]
                tags.append(rest[j + 4:j + 4 + m].decode("utf-8", "replace"))
                j += 4 + m
        except struct.error:
            bad.append("header dei commenti troncato")
    elif pages:
        bad.append("seconda pagina senza header dei commenti Vorbis")

    serials = sorted({p[2] for p in pages})
    grans = [p[1] for p in pages if p[1] >= 0]
    samples = grans[-1] if grans else 0
    format_bad = []
    if ch != 2:
        format_bad.append(f"{ch} canali invece di 2")
    if rate != SAMPLE_RATE:
        format_bad.append(f"{rate} Hz invece di {SAMPLE_RATE}")
    bad += format_bad
    more = []
    if len(serials) != 1:
        more.append(f"{len(serials)} flussi logici (file concatenati)")
    if pages and not pages[0][0] & 2:
        more.append("prima pagina senza BOS")
    if pages and not pages[-1][0] & 4:
        more.append("ultima pagina senza EOS")
    if [p[3] for p in pages] != list(range(len(pages))):
        more.append("numeri di pagina non consecutivi")
    if grans != sorted(grans):
        more.append("granule non crescenti")
    bad += more
    container_bad += more + [b for b in bad if b.startswith(("header dei", "seconda pagina"))]

    dur = samples / rate if rate else 0.0
    kbps = len(d) * 8 / dur / 1000 if dur else 0.0
    out(f"{path.name}: {len(d)} byte ({len(d) / 1048576:.2f} MiB), {len(pages)} pagine")
    out(f"  Vorbis {ch} canali, {rate} Hz, nominale {nom / 1000:g} kbps, encoder: {vendor}")
    out("  commenti: " + ("; ".join(t[:60] for t in tags[:20]) if tags else "nessuno"))
    out(f"  campioni per canale (granule finale): {samples} = {dur:.4f} s, "
        f"bitrate medio {kbps:.1f} kbps")
    length_ok = None
    if spec.samples is not None:
        length_ok = abs(samples - spec.samples) <= spec.tol
        out(f"  attesi {spec.samples} (tolleranza {spec.tol}): scarto {samples - spec.samples:+d}"
            f" -> {'OK' if length_ok else 'NO'}")
    out("  problemi: " + ("; ".join(bad) if bad else "nessuno"))

    weight_ok = None if spec.max_bytes is None else len(d) <= spec.max_bytes
    if weight_ok is not None:
        out(f"  peso {len(d)} byte, tetto {spec.max_bytes} "
            f"({len(d) / spec.max_bytes:.0%} del tetto) -> {'OK' if weight_ok else 'NO'}")
    encoders = [t.split("=", 1)[1] for t in tags if t.lower().startswith("encoder=")]
    if "libvorbis" in vendor.lower():
        encoder_ok, why = True, f"fornitore {vendor!r}"
    elif any("libvorbis" in e.lower() for e in encoders):
        encoder_ok = True
        why = (f"tag ENCODER {encoders[0]!r}: libvorbis dentro FFmpeg; il fornitore {vendor!r} "
               "è la firma del muxer di FFmpeg")
    else:
        encoder_ok = False
        why = f"fornitore {vendor!r}, tag ENCODER {encoders or 'assente'}: libVorbis non risulta"
    out(f"  encoder: {why} -> {'OK' if encoder_ok else 'NO'}")

    tag = f"[{path.name}]"
    out.result(f"{tag} contenitore", not container_bad,
               f"{len(pages)} pagine, {len(serials)} flusso, CRC "
               f"{'giusti' if not any('CRC' in b for b in bad) else 'errati'}"
               + (f"; {'; '.join(container_bad)}" if container_bad else ""))
    out.result(f"{tag} codec, canali, frequenza", ident_ok and not format_bad,
               f"Vorbis {ch} canali, {rate} Hz")
    out.result(f"{tag} lunghezza (granule)", length_ok,
               f"{samples}" + (f", attesi {spec.samples} +-{spec.tol}" if spec.samples else ""))
    out.result(f"{tag} peso e bitrate", weight_ok,
               f"{len(d)} byte" + (f" <= {spec.max_bytes}" if weight_ok else
                                   f", tetto {spec.max_bytes}" if spec.max_bytes else "")
               + f", {kbps:.1f} kbps medi, nominale {nom / 1000:g}")
    out.result(f"{tag} encoder", encoder_ok, encoders[0] if encoders else vendor)
    out.result(f"{tag} picco, loudness, estremi, giro, stereo", None,
               "serve decodificare l'Ogg: test di decodifica dopo la promozione")
    return samples if pages else None


# --- Master WAV: lettura --------------------------------------------------------------------


def read_pcm(data: bytes) -> tuple[dict, list[array], float]:
    """Formato, canali separati (array 'i' o 'f') e fondo scala. PCM 16 e 24 bit si allargano a
    32 bit spostando i byte (valori x65536 e x256, segno compreso): il fondo scala è sempre
    2^31, così un 16 bit si misura come un 24 bit."""
    tag, channels, rate, align, bits, pcm = wav_parts(data)
    frames = len(pcm) // align
    info = {"tag": tag, "channels": channels, "rate": rate, "bits": bits, "frames": frames}
    if channels < 1 or align != channels * bits // 8:
        raise AudioFileError(f"block align {align} incoerente con {channels} canali a {bits} bit")
    raw = bytes(pcm[: frames * align])
    if tag == WAVE_FORMAT_PCM and bits in (16, 24, 32):
        if bits == 32:
            buf = raw
        else:
            width = bits // 8
            buf = bytearray(4 * frames * channels)
            for j in range(width):
                buf[4 - width + j::4] = raw[j::width]
        values = array("i")
        full = 2.0 ** 31
    elif tag == WAVE_FORMAT_FLOAT and bits == 32:
        buf, values, full = raw, array("f"), 1.0
    else:
        raise AudioFileError(f"formato {tag} a {bits} bit: servono PCM 16, 24 o 32 bit o float 32")
    if values.itemsize != 4:
        raise AudioFileError(f"array('{values.typecode}') di {values.itemsize} byte su questa "
                             "piattaforma, ne servono 4")
    values.frombytes(buf)
    if sys.byteorder != "little":
        values.byteswap()
    return info, [values[c::channels] for c in range(channels)], full


# --- Master WAV: misure ---------------------------------------------------------------------


def true_peak(x: array, circular: bool) -> float:
    """Massimo assoluto fra i campioni e i valori interpolati 4x, nelle unità del file.

    Stessi filtri di audio_sfx.py. Per non interpolare 3 milioni di posizioni si guardano i
    blocchi dal più forte: |valore interpolato| <= somma dei |coefficienti| x massimo della
    finestra, quindi quando quel limite scende sotto il picco trovato ci si ferma (esatto, non
    un'approssimazione). Loop: la finestra gira attorno al punto di giro; altrimenti zeri."""
    taps = TRUE_PEAK_TAPS
    width = 2 * taps
    n = len(x)
    if circular and n >= width:
        pad = x[n - taps:] + x + x[:taps]
        positions = n  # da n - 1 si torna a 0
    else:
        pad = array("d", bytes(8 * taps)) + array("d", x) + array("d", bytes(8 * (taps + 1)))
        positions = n + 1
    k1, k2, k3 = true_peak_kernels(taps)
    gain = max(sum(map(abs, k)) for k in (k1, k2, k3))
    peak = float(max(max(x), -min(x)))
    blocks = []
    for lo in range(0, positions, TP_BLOCK):
        seg = pad[lo + 1:min(lo + TP_BLOCK, positions) + width]
        blocks.append((max(max(seg), -min(seg)), lo))
    blocks.sort(reverse=True)
    sumprod = math.sumprod
    for top, lo in blocks:
        if top * gain <= peak:
            break
        for i in range(lo, min(lo + TP_BLOCK, positions)):
            w = pad[i + 1:i + 1 + width]
            a, b, c = sumprod(w, k1), sumprod(w, k2), sumprod(w, k3)
            m = max(a, b, c, -a, -b, -c)
            if m > peak:
                peak = m
    return peak


def k_filter(rate: int) -> tuple[tuple[float, ...], tuple[float, float]]:
    """Filtro K di BS.1770-4 ricalcolato per `rate` dai prototipi analogici (come libebur128):
    mensola alta (+4 dB, b0 b1 b2 a1 a2) e passa-alto RLB (numeratore 1, -2, 1; a1 a2).
    A 48 kHz dà i coefficienti della norma."""
    f0, gain_db, q = 1681.974450955533, 3.999843853973347, 0.7071752369554196
    k = math.tan(math.pi * f0 / rate)
    vh = 10 ** (gain_db / 20)
    vb = vh ** 0.4996667741545416
    a0 = 1 + k / q + k * k
    shelf = ((vh + vb * k / q + k * k) / a0, 2 * (k * k - vh) / a0,
             (vh - vb * k / q + k * k) / a0, 2 * (k * k - 1) / a0, (1 - k / q + k * k) / a0)
    f0, q = 38.13547087602444, 0.5003270373238773
    k = math.tan(math.pi * f0 / rate)
    a0 = 1 + k / q + k * k
    return shelf, (2 * (k * k - 1) / a0, (1 - k / q + k * k) / a0)


def k_weighted_energy(x: array, rate: int, step: int) -> list[float]:
    """Somma dei quadrati del segnale pesato K per ogni sotto-blocco di `step` campioni."""
    (b0, b1, b2, a1, a2), (c1, c2) = k_filter(rate)
    s1 = s2 = t1 = t2 = 0.0
    sums = []
    for start in range(0, len(x) - step + 1, step):
        acc = 0.0
        for v in x[start:start + step]:
            y = b0 * v + s1
            s1 = b1 * v - a1 * y + s2
            s2 = b2 * v - a2 * y
            z = y + t1
            t1 = -2.0 * y - c1 * z + t2
            t2 = y - c2 * z
            acc += z * z
        sums.append(acc)
    return sums


def block_lufs(power: float) -> float:
    return -0.691 + 10 * math.log10(power) if power > 0 else -math.inf


def loudness(chans: list[array], rate: int, full: float) -> tuple[float, int, int, float]:
    """(LUFS integrati, blocchi, blocchi sopra i due gate, gate relativo) secondo BS.1770-4.
    Peso 1 per ogni canale (L e R); i blocchi da 400 ms sono 4 sotto-blocchi da 100 ms."""
    step = rate // 10
    energy = [k_weighted_energy(c, rate, step) for c in chans]
    count = len(energy[0]) - 3
    scale = 1 / (4 * step * full * full)
    powers = [sum(sum(e[j:j + 4]) for e in energy) * scale for j in range(max(count, 0))]
    above_abs = [p for p in powers if block_lufs(p) > ABS_GATE_LUFS]
    if not above_abs:
        return -math.inf, len(powers), 0, -math.inf
    gate = block_lufs(sum(above_abs) / len(above_abs)) + REL_GATE_LU
    kept = [p for p in above_abs if block_lufs(p) > gate]
    return block_lufs(sum(kept) / len(kept)), len(powers), len(kept), gate


def rms(x: array, lo: int, hi: int) -> float:
    seg = x[lo:hi]
    return math.sqrt(math.sumprod(seg, seg) / len(seg)) if len(seg) else 0.0


def percentile_top(values: array, percentile: float) -> float:
    """Percentile a rango più vicino, cercato fra i valori più grandi."""
    m = len(values)
    return heapq.nlargest(m - math.ceil(percentile / 100 * m) + 1, values)[-1]


def jump_stats(x: array) -> tuple[float, float, float, float, float]:
    """Continuità del giro di un canale:
    - criterio della procedura: salto |x[0] - x[-1]| contro il 99,9° percentile delle
      differenze fra campioni vicini; più la quota delle differenze più piccole del salto;
    - controllo di riserva, informativo: la seconda differenza a cavallo del giro (quanto il
      primo campione si scosta dalla retta degli ultimi due, e così via) contro il 99,9°
      percentile delle seconde differenze. Un segnale liscio con il giro nel punto più ripido
      passa questo e può fallire il primo; un clic li fallisce tutti e due."""
    typecode = "q" if x.typecode == "i" else "d"
    sub = operator.sub
    diffs = array(typecode, map(abs, map(sub, x[1:], x[:-1])))
    threshold = percentile_top(diffs, JUMP_PERCENTILE)
    gap = abs(x[0] - x[-1])
    below = sum(map(operator.lt, diffs, repeat(gap))) / len(diffs)
    steps = map(sub, x[1:], x[:-1])
    curves = array(typecode, map(abs, map(sub, map(sub, x[2:], x[1:-1]), steps)))
    curve_threshold = percentile_top(curves, JUMP_PERCENTILE)
    curve_gap = max(abs((x[0] - x[-1]) - (x[-1] - x[-2])), abs((x[1] - x[0]) - (x[0] - x[-1])))
    return gap, threshold, below, curve_gap, curve_threshold


def hann(n: int) -> list[float]:
    return [0.5 - 0.5 * math.cos(2 * math.pi * i / n) for i in range(n)]


def band_power(mono: array, rate: float, window: int, cutoffs: tuple[float, ...]) -> dict:
    """Potenza media del mono fra 1 bin e ogni frequenza di `cutoffs` (DC esclusa): somma delle
    potenze dei bin di una DFT con finestra di Hann, finestre consecutive di `window` campioni,
    con la correzione della finestra (Parseval). Selettiva: una nota a 14 bin dal taglio pesa
    meno di -80 dB, mentre un filtro Butterworth del 4° ordine a 30 Hz lascia passare il Fa1
    a 43,65 Hz a -13 dB. Incertezza: un bin al bordo (1 Hz con finestre di 1 s)."""
    w = hann(window)
    scale = 2 / (window * math.sumprod(w, w))
    bins = range(1, int(max(cutoffs) * window / rate) + 1)
    tables = [([v * math.cos(2 * math.pi * k * i / window) for i, v in enumerate(w)],
               [v * math.sin(2 * math.pi * k * i / window) for i, v in enumerate(w)])
              for k in bins]
    count = len(mono) // window
    totals = [0.0] * len(tables)
    for j in range(count):
        seg = mono[j * window:(j + 1) * window]
        for b, (c, s) in enumerate(tables):
            totals[b] += math.sumprod(seg, c) ** 2 + math.sumprod(seg, s) ** 2
    return {cut: sum(t for k, t in zip(bins, totals) if k * rate / window <= cut)
            * scale / max(count, 1) for cut in cutoffs}


def decimate_mono(chans: list[array], factor: int, circular: bool) -> array:
    """Mono (media dei canali) passato in un FIR passa-basso di Blackman con taglio a metà della
    frequenza finale e calcolato solo ogni `factor` campioni. Loop: estensione circolare."""
    half = DECIM_TAPS // 2
    fc = 0.5 / factor  # cicli per campione
    h = []
    for j in range(DECIM_TAPS):
        t = j - half
        s = 2 * fc if t == 0 else math.sin(2 * math.pi * fc * t) / (math.pi * t)
        w = (0.42 - 0.5 * math.cos(2 * math.pi * j / (DECIM_TAPS - 1))
             + 0.08 * math.cos(4 * math.pi * j / (DECIM_TAPS - 1)))
        h.append(s * w)
    norm = sum(h) * len(chans)
    h = [v / norm for v in h]
    n = len(chans[0])
    if circular and n > DECIM_TAPS:
        exts = [c[n - half:] + c + c[:half] for c in chans]
    else:
        zeros = array("d", bytes(8 * half))
        exts = [zeros + c + zeros for c in chans]
    sumprod = math.sumprod
    return array("d", (sum(sumprod(e[m:m + DECIM_TAPS], h) for e in exts)
                       for m in range(0, n, factor)))


def low_notes(mono: array, rate: float, window: int, full: float, circular: bool) -> dict:
    """Note temperate (La = 440 Hz) fra 30 e 100 Hz, finestra per finestra.

    Per ogni finestra (un battito, o 1 s senza BPM) e ogni nota: DFT a quella sola frequenza
    con finestra di Hann (lo stesso numero di Goertzel), ampiezza del seno equivalente in dBFS
    sul mono. Una nota "suona" in una finestra se è un massimo locale fra le note vicine, entro
    LOW_REL_DB dalla nota grave più forte del file e sopra LOW_ABS_DBFS; è "tenuta" se suona in
    almeno LOW_HELD finestre consecutive (sui loop anche a cavallo del giro)."""
    notes = []  # (nome, classe, frequenza)
    for midi in range(12, 60):
        f = 440 * 2 ** ((midi - 69) / 12)
        if LOW_MIN_HZ <= f <= LOW_MAX_HZ:
            notes.append((f"{NOTE_NAMES[midi % 12]}{midi // 12 - 1}", NOTE_NAMES[midi % 12], f))
    w = hann(window)
    norm = 2 / sum(w) / full
    tables = [([v * math.cos(2 * math.pi * f * i / rate) for i, v in enumerate(w)],
               [v * math.sin(2 * math.pi * f * i / rate) for i, v in enumerate(w)])
              for _, _, f in notes]
    count = len(mono) // window
    levels = []  # levels[finestra][nota]
    for k in range(count):
        seg = mono[k * window:(k + 1) * window]
        levels.append([db(math.hypot(math.sumprod(seg, c), math.sumprod(seg, s)) * norm)
                       for c, s in tables])
    loudest = max((v for row in levels for v in row), default=-math.inf)
    floor = max(loudest - LOW_REL_DB, LOW_ABS_DBFS)
    rows = []
    for j, (name, pitch_class, f) in enumerate(notes):
        on, peaks = [], [-math.inf]
        for row in levels:
            left = row[j - 1] if j > 0 else -math.inf
            right = row[j + 1] if j + 1 < len(row) else -math.inf
            if row[j] > left and row[j] > right:
                peaks.append(row[j])
            on.append(row[j] >= floor and row[j] > left and row[j] > right)
        run = best = 0
        for flag in (on + on if circular else on):
            run = run + 1 if flag else 0
            best = max(best, run)
        best = min(best, count)
        rows.append({"name": name, "class": pitch_class, "hz": f, "windows": sum(on),
                     "run": best, "peak_db": max(peaks),
                     "max_db": max(row[j] for row in levels) if levels else -math.inf})
    held = [r for r in rows if r["run"] >= LOW_HELD and r["class"] not in LOW_ALLOWED]
    return {"rows": rows, "count": count, "loudest": loudest, "floor": floor, "held": held}


@dataclass
class Master:
    """Un master letto: canali come interi del file (ints) e come float (chans)."""
    tag: str
    spec: Spec
    rate: int
    frames: int
    full: float
    ints: list[array]
    chans: list[array]
    names: list[str]

    @property
    def loop(self) -> bool:
        return self.spec.kind in ("loop", "ambience")


def analyze_master(path: Path, spec: Spec, out: Out, ogg_samples: int | None) -> None:
    started = time.perf_counter()
    data = path.read_bytes()
    info, ints, full = read_pcm(data)
    rate, frames, nch = info["rate"], info["frames"], info["channels"]
    kind = {WAVE_FORMAT_PCM: "PCM", WAVE_FORMAT_FLOAT: "float"}.get(info["tag"],
                                                                     f"formato {info['tag']}")
    tag = f"[{path.name}]"
    out(f"{path.name}: {len(data)} byte, {kind} {info['bits']} bit, {nch} canali, {rate} Hz, "
        f"{frames} frame = {frames / rate:.4f} s")
    usual = kind == "PCM" and info["bits"] in (16, 24)
    note = "" if usual else " (la richiesta chiede PCM 24 o 16 bit)"
    out.result(f"{tag} formato", nch == 2 and rate == SAMPLE_RATE,
               f"{kind} {info['bits']} bit, {nch} canali, {rate} Hz{note}")
    if spec.samples is not None:
        ok = abs(frames - spec.samples) <= spec.tol
        out(f"  attesi {spec.samples} (tolleranza {spec.tol}): scarto {frames - spec.samples:+d}"
            f" -> {'OK' if ok else 'NO'}")
        out.result(f"{tag} lunghezza (frame)", ok, f"{frames}, attesi {spec.samples} +-{spec.tol}")
    if ogg_samples is not None:
        same = ogg_samples == frames
        out(f"  stessi campioni dell'Ogg ({ogg_samples}): {'OK' if same else 'NO'}")
        out.result(f"{tag} stessa lunghezza dell'Ogg", same, f"{frames} contro {ogg_samples}")
    if frames < rate // 2:
        out("  file più corto di 0,5 s: misure saltate")
        out.result(f"{tag} misure del master", None, "file troppo corto")
        return
    names = ["L", "R"] if nch == 2 else [str(i + 1) for i in range(nch)]
    m = Master(tag, spec, rate, frames, full, ints, [array("d", c) for c in ints], names)
    report_peak(m, out)
    report_loudness(m, out)
    if spec.kind == "stinger":
        report_stinger_edges(m, out)
    else:
        report_loop_edges(m, out)
        report_loop_jump(m, out)
    mono_power = report_stereo(m, out)
    report_low_register(m, out, mono_power)
    out(f"  tempo dell'analisi: {time.perf_counter() - started:.1f} s")


def report_peak(m: Master, out: Out) -> None:
    peaks = [max(max(c), -min(c)) / m.full for c in m.ints]
    tp = max(true_peak(c, m.loop) for c in m.chans) / m.full
    ok = db(tp) <= MASTER_MAX_TP
    out("  picco di campione: " + ", ".join(f"{n} {fmt_db(db(p))} dBFS"
                                           for n, p in zip(m.names, peaks))
        + f"; true peak stimato (4x{', circolare' if m.loop else ''}) {fmt_db(db(tp))} dBTP, "
        f"limite {MASTER_MAX_TP} -> {'OK' if ok else 'NO'}")
    out.result(f"{m.tag} picco", ok, f"true peak {fmt_db(db(tp))} dBTP (limite master "
               f"{MASTER_MAX_TP}), campione {fmt_db(db(max(peaks)))} dBFS")


def report_loudness(m: Master, out: Out) -> None:
    spec = m.spec
    if m.rate % 10:
        out.result(f"{m.tag} loudness", None, f"{m.rate} Hz: i blocchi da 100 ms non sono interi")
        return
    lufs, blocks, kept, gate = loudness(m.chans, m.rate, m.full)
    ok = None
    if spec.lufs is not None:
        ok = lufs != -math.inf and abs(lufs - spec.lufs) <= spec.lufs_tol + 1e-9
    target = f", attesi {spec.lufs:g} +-{spec.lufs_tol:g}" if spec.lufs is not None else ""
    out(f"  loudness integrata {fmt_db(lufs)} LUFS ({blocks} blocchi da 400 ms, {kept} sopra "
        f"i gate; gate relativo {fmt_db(gate, 1)} LUFS){target}"
        + (f" -> {'OK' if ok else 'NO'}" if ok is not None else ""))
    out.result(f"{m.tag} loudness", "info" if ok is None else ok, f"{fmt_db(lufs)} LUFS{target}")


def report_stinger_edges(m: Master, out: Out) -> None:
    limit = EDGE_MAX * m.full
    first_ok = all(abs(c[0]) <= limit for c in m.ints)
    onset = next((i for i in range(m.frames) if any(abs(c[i]) > limit for c in m.ints)), None)
    attack_ok = first_ok and onset is not None and onset <= round(STING_ATTACK_S * m.rate)
    when = f"{onset / m.rate * 1000:.2f} ms" if onset is not None else "mai"
    tail_n = round(STING_TAIL_S * m.rate)
    tail = max(max(max(c[-tail_n:]), -min(c[-tail_n:])) for c in m.ints) / m.full
    tail_ok = db(tail) < STING_TAIL_DBFS
    out("  attacco: primo campione " + ", ".join(f"{n} {c[0] / m.full:+.4f}"
                                               for n, c in zip(m.names, m.ints))
        + f"; oltre {STING_ATTACK_DBFS:g} dBFS dopo {when} -> {'OK' if attack_ok else 'NO'}")
    out(f"  coda: picco degli ultimi {STING_TAIL_S * 1000:g} ms {fmt_db(db(tail))} dBFS, "
        f"limite {STING_TAIL_DBFS:g} -> {'OK' if tail_ok else 'NO'}")
    out.result(f"{m.tag} attacco", attack_ok, f"oltre {STING_ATTACK_DBFS:g} dBFS dopo {when}")
    out.result(f"{m.tag} coda", tail_ok, f"ultimi 10 ms {fmt_db(db(tail))} dBFS")


def report_loop_edges(m: Master, out: Out) -> None:
    edge_vals = [(c[0] / m.full, c[-1] / m.full) for c in m.ints]
    zero_ok = all(abs(a) <= EDGE_MAX and abs(b) <= EDGE_MAX for a, b in edge_vals)
    out("  estremi: primo campione " + ", ".join(f"{n} {a:+.4f}"
                                               for n, (a, _) in zip(m.names, edge_vals))
        + "; ultimo " + ", ".join(f"{n} {b:+.4f}" for n, (_, b) in zip(m.names, edge_vals))
        + f"; limite |x| <= {EDGE_MAX:g} -> {'OK' if zero_ok else 'NO'}")
    en, rn, frames = round(EDGE_S * m.rate), round(EDGE_REF_S * m.rate), m.frames

    def drop(c: array, lo: int, ref_lo: int) -> float:
        edge, ref = rms(c, lo, lo + en), rms(c, ref_lo, ref_lo + rn)
        if ref > 0:
            return db(edge / ref)
        return 0.0 if edge == 0 else math.inf

    heads: list[float] = []
    tails: list[float] = []
    fade_ok = None
    if frames >= en + rn:
        heads = [drop(c, 0, en) for c in m.chans]
        tails = [drop(c, frames - en, frames - en - rn) for c in m.chans]
        fade_ok = all(v >= -EDGE_MAX_DROP_DB for v in heads + tails)
        out(f"  RMS dei primi {EDGE_S * 1000:g} ms contro il secondo dopo: "
            + ", ".join(f"{n} {fmt_db(v, 1)} dB" for n, v in zip(m.names, heads))
            + f"; ultimi {EDGE_S * 1000:g} ms contro il secondo prima: "
            + ", ".join(f"{n} {fmt_db(v, 1)} dB" for n, v in zip(m.names, tails))
            + f"; minimo -{EDGE_MAX_DROP_DB:g} dB -> {'OK' if fade_ok else 'NO'}")
        # la stessa misura a ogni stanghetta interna dice se gli estremi del loop sono un
        # passaggio di battuta come gli altri o una dissolvenza solo al giro
        bar = round(m.rate * 60 / m.spec.bpm) * m.spec.beats_per_bar if m.spec.bpm else 0
        inner = []
        if bar:
            inner = [b * bar for b in range(1, frames // bar)
                     if b * bar >= en + rn and b * bar + en + rn <= frames]
        if inner:
            ranges = []
            for n, c, head, tail in zip(m.names, m.chans, heads, tails):
                ends = [drop(c, e - en, e - en - rn) for e in inner]
                starts = [drop(c, e, e + en) for e in inner]
                inside = min(ends) - 1 <= tail and min(starts) - 1 <= head
                ranges.append(f"{n} fine {fmt_db(min(ends), 1)}...{fmt_db(max(ends), 1)}, "
                              f"inizio {fmt_db(min(starts), 1)}...{fmt_db(max(starts), 1)} "
                              f"(loop: fine {fmt_db(tail, 1)}, inizio {fmt_db(head, 1)}: "
                              f"{'dentro la fascia' if inside else 'fuori dalla fascia'})")
            out(f"  le stesse misure alle {len(inner)} stanghette interne (informativo, dB): "
                + "; ".join(ranges))
            out.result(f"{m.tag} estremi contro le stanghette interne", "info", "; ".join(ranges))
    edge_ok = False if not zero_ok or fade_ok is False else fade_ok
    worst = max(max(abs(a), abs(b)) for a, b in edge_vals)
    fade = f"{fmt_db(min(heads + tails), 1)} dB al peggio" if heads else "non misurato"
    out.result(f"{m.tag} estremi", edge_ok, f"|x| max {worst:.4f}; RMS 50 ms {fade}")


def report_loop_jump(m: Master, out: Out) -> None:
    full = m.full
    jumps = [jump_stats(c) for c in m.ints]
    jump_ok = all(j[0] <= j[1] for j in jumps)
    curve_ok = all(j[3] <= j[4] for j in jumps)
    out("  giro: salto ultimo -> primo campione "
        + "; ".join(f"{n} {gap / full:.5f} (99,9° percentile {thr / full:.5f}, "
                    f"più grande del {below:.1%} delle differenze)"
                    for n, (gap, thr, below, _, _) in zip(m.names, jumps))
        + f" -> {'OK' if jump_ok else 'NO'}")
    curves = "; ".join(f"{n} {cg / full:.5f} (99,9° percentile {ct / full:.5f})"
                       for n, (_, _, _, cg, ct) in zip(m.names, jumps))
    within = "entro" if curve_ok else "OLTRE"
    out(f"  giro, seconda differenza (riserva, informativo): {curves} -> {within}")
    out.result(f"{m.tag} continuità del giro", jump_ok,
               "; ".join(f"{n} {gap / full:.5f} {'<=' if gap <= thr else '>'} {thr / full:.5f}"
                         for n, (gap, thr, _, _, _) in zip(m.names, jumps)))
    out.result(f"{m.tag} giro, seconda differenza", "info",
               f"{within} il 99,9° percentile: {curves}")


def report_stereo(m: Master, out: Out) -> float:
    """Stampa RMS, differenza e correlazione; rende la potenza media del mono (L + R) / 2."""
    n, full, chans = m.frames, m.full, m.chans
    sums = [math.sumprod(c, c) for c in chans]
    if len(chans) != 2:
        return sum(sums) / n / len(chans) ** 2
    means = [sum(c) / n for c in chans]
    rms_db = [db(math.sqrt(s / n) / full) for s in sums]
    cross = math.sumprod(chans[0], chans[1])
    var_l, var_r = sums[0] / n - means[0] ** 2, sums[1] / n - means[1] ** 2
    corr = 0.0
    if var_l > 0 and var_r > 0:
        corr = (cross / n - means[0] * means[1]) / math.sqrt(var_l * var_r)
    diff = rms_db[1] - rms_db[0] if min(rms_db) > -math.inf else \
        (0.0 if rms_db[0] == rms_db[1] else math.copysign(math.inf, rms_db[1] - rms_db[0]))
    detail = f"R - L {fmt_db(diff)} dB, correlazione {corr:+.3f}"
    verdict = ""
    if m.spec.right_over_left is not None:
        lo, hi = m.spec.right_over_left
        ok = lo <= diff <= hi and corr > 0
        verdict = (f"; attesi R - L fra {lo:g} e {hi:g} dB e correlazione > 0 -> "
                   f"{'OK' if ok else 'NO'}")
        out.result(f"{m.tag} stereo", ok, f"{detail} (attesi {lo:g}-{hi:g} dB)")
    else:
        out.result(f"{m.tag} stereo", "info", detail)
    out(f"  stereo: RMS L {fmt_db(rms_db[0])} dBFS, R {fmt_db(rms_db[1])} dBFS, {detail}, "
        f"continua L {means[0] / full:+.6f} R {means[1] / full:+.6f}{verdict}")
    return (sums[0] + sums[1] + 2 * cross) / (4 * n)


def report_low_register(m: Master, out: Out, mono_power: float) -> None:
    """Banda grave (sempre) e note gravi tenute (se la tabella lo chiede), sul mono decimato."""
    spec = m.spec
    window = round(m.rate * 60 / spec.bpm) if spec.bpm else m.rate
    factor = max((f for f in range(16, 71) if window % f == 0), default=0)
    if not factor or window // factor < 64:
        out.result(f"{m.tag} banda grave", None, f"finestra di {window} campioni non decimabile")
        return
    mono = decimate_mono(m.chans, factor, m.loop)
    low_rate = m.rate / factor
    bands = band_power(mono, low_rate, window // factor, (30.0, 100.0))
    shares = {cut: 10 * math.log10(p / mono_power) if p > 0 and mono_power else -math.inf
              for cut, p in bands.items()}
    detail = (f"fino a 30 Hz {fmt_db(shares[30.0], 1)} dB, fino a 100 Hz "
              f"{fmt_db(shares[100.0], 1)} dB rispetto alla potenza del mono (DC esclusa)")
    out(f"  banda grave: {detail} (informativo)")
    out.result(f"{m.tag} banda grave", "info", detail)
    if not spec.low_notes:
        return
    res = low_notes(mono, low_rate, window // factor, m.full, m.loop)
    out(f"  note gravi fra {LOW_MIN_HZ:g} e {LOW_MAX_HZ:g} Hz ({res['count']} finestre di un "
        f"{'battito' if spec.bpm else 'secondo'}, mono decimato a {low_rate:g} Hz; soglia "
        f"{fmt_db(res['floor'], 1)} dBFS = la più forte {fmt_db(res['loudest'], 1)} - "
        f"{LOW_REL_DB:g} dB):")
    for r in res["rows"]:
        if r["windows"]:
            out(f"    {r['name']:5} {r['hz']:6.2f} Hz: suona in {r['windows']:3} finestre, al "
                f"massimo {r['run']:3} di fila, picco {fmt_db(r['max_db'], 1)} dBFS")
    others = sorted((r for r in res["rows"] if r["class"] not in LOW_ALLOWED
                     and r["peak_db"] > -math.inf), key=lambda r: -r["peak_db"])[:3]
    strongest = ", ".join(f"{r['name']} {fmt_db(r['peak_db'], 1)} dBFS" for r in others)
    out(f"  fuori da Fa e Do, i massimi locali più forti (anche sotto soglia): "
        f"{strongest or 'nessuno'}")
    held = ", ".join(f"{r['name']} ({r['run']} di fila)" for r in res["held"])
    criterion = spec.low_notes == "criterio"
    ok = not res["held"]
    verdict = f" -> {'OK' if ok else 'NO'}" if criterion else " (informativo)"
    out(f"  note gravi tenute (>= {LOW_HELD} finestre) diverse da Fa e Do: "
        f"{held or 'nessuna'}{verdict}")
    out.result(f"{m.tag} note gravi tenute fuori da Fa e Do (euristica)",
               ok if criterion else "info", held or "nessuna")


# --- PROVENANCE.txt -------------------------------------------------------------------------


def check_provenance(folder: Path, files: list[Path], out: Out) -> None:
    """Forma di PROVENANCE.txt: una sezione [file] completa per ogni Ogg, licenza nella forma
    del manifest, master citato, fonti. Canale e diritti li giudica l'utente, non lo script."""
    path = folder / "PROVENANCE.txt"
    if not path.is_file():
        out("PROVENANCE.txt: assente")
        out.result("[PROVENANCE.txt] forma", False, "file assente")
        return
    sections: list[tuple[str, dict[str, str]]] = []
    for raw in path.read_text(encoding="utf-8-sig", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        head = re.fullmatch(r"\[(\w+)\]", line)
        if head:
            sections.append((head.group(1), {}))
        elif sections and ":" in line:
            key, value = line.split(":", 1)
            sections[-1][1][key.strip().lower()] = value.strip()
    problems = []
    for ogg in (f for f in files if f.suffix == ".ogg"):
        entry = next((s for kind, s in sections if kind == "file" and s.get("nome") == ogg.name),
                     None)
        if entry is None:
            problems.append(f"nessuna sezione [file] con nome: {ogg.name}")
            out(f"PROVENANCE.txt, {ogg.name}: sezione [file] assente")
            continue
        missing = [k for k in PROVENANCE_FILE_KEYS if not entry.get(k)]
        licence = entry.get("licenza", "")
        licence_ok = bool(LICENSE_RE.match(licence))
        master = entry.get("master", "")
        master_ok = master == "nessuno" or (folder / master).is_file()
        sources = [s for kind, s in sections if kind == "fonte" and s.get("per") == ogg.name]
        source_missing = sorted({k for s in sources for k in PROVENANCE_SOURCE_KEYS
                                 if not s.get(k)})
        out(f"PROVENANCE.txt, {ogg.name}: chiavi mancanti {missing or 'nessuna'}; licenza "
            f"{licence!r} ({'forma ammessa' if licence_ok else 'forma NON ammessa'}); master "
            f"{master!r} ({'presente' if master_ok else 'assente'}); sezioni [fonte]: "
            f"{len(sources)}" + (f", chiavi mancanti {source_missing}" if source_missing else ""))
        for s in sources:
            saved = s.get("licenza salvata", "")
            out(f"  fonte: licenza {s.get('licenza', '?')!r}, licenza salvata {saved!r}")
            if s.get("licenza", "").startswith(("CC0", "CC-BY")) and \
                    not (folder / saved).is_file():
                problems.append(f"{ogg.name}: licenza salvata {saved!r} assente")
        problems += [f"{ogg.name}: manca {k}" for k in missing]
        problems += [f"{ogg.name}: licenza {licence!r}"] if not licence_ok else []
        problems += [f"{ogg.name}: master {master!r} assente"] if not master_ok else []
    out("  canale, piano e termini: li conferma l'utente (lo script controlla solo la forma)")
    out.result("[PROVENANCE.txt] forma", "a vista" if not problems else False,
               "; ".join(problems) or "sezioni complete; canale e diritti da confermare "
                                      "dall'utente")


# --- Scelta dei file e uscita ---------------------------------------------------------------


def find_group(request: str, version: int | None) -> tuple[list[tuple[str, Spec, int, Path | None,
                                                                       Path | None]], list[str]]:
    """Per ogni parte della richiesta: (parte, criteri, versione, Ogg, WAV) dall'inbox."""
    folder = INBOX / request
    found: dict[str, dict[int, dict[str, Path]]] = {}
    strays = []
    for f in sorted(folder.iterdir()) if folder.is_dir() else []:
        m = NAME_RE.match(f.name)
        if m and m["id"] == request:
            found.setdefault(m["part"] or "", {}).setdefault(int(m["ver"]), {})[m["ext"]] = f
        elif f.is_file() and f.name != "PROVENANCE.txt":
            strays.append(f.name)
    groups = []
    for part, spec in REQUESTS[request].items():
        versions = found.get(part, {})
        ver = version if version is not None else max(versions, default=None)
        files = versions.get(ver, {}) if ver is not None else {}
        groups.append((part, spec, ver, files.get("ogg"), files.get("wav")))
    strays += [f"{p or '(parte unica)'}: parte non prevista" for p in found
               if p not in REQUESTS[request]]
    return groups, strays


def promoted_spec(path: Path) -> tuple[Spec | None, str]:
    """Criteri di un file già promosso in assets/audio/ (nome senza ID): la voce del manifest
    dice la richiesta ("request": "MUSIC-HOME-001 v1"), la tabella REQUESTS i numeri. Una
    richiesta con più parti vuole la parte in fondo al nome (stinger_boss.ogg -> "boss")."""
    try:
        rel = path.resolve().relative_to(ASSETS.resolve()).as_posix()
        entries = json.loads((ASSETS / "manifest.json").read_text(encoding="utf-8"))["assets"]
    except (ValueError, OSError, KeyError, json.JSONDecodeError):
        return None, ""
    entry = next((e for e in entries if e.get("path") == rel), None)
    request = (entry or {}).get("request") or ""
    parts = REQUESTS.get(request.split(" ")[0], {})
    part = "" if "" in parts else next((p for p in parts if path.stem.endswith("_" + p)), None)
    if part is None or part not in parts:
        return None, f"{rel}: nessuna richiesta della tabella REQUESTS nella voce del manifest"
    return parts[part], (f"criteri di {request.split(' ')[0]}{'_' + part if part else ''} "
                         f"(voce del manifest di {rel}: request '{request}')")


def spec_overrides(args: argparse.Namespace) -> dict:
    keys = ("kind", "samples", "tol", "max_bytes", "lufs", "lufs_tol", "bpm")
    values = {k: getattr(args, k) for k in keys if getattr(args, k) is not None}
    if args.right_over_left:
        values["right_over_left"] = tuple(args.right_over_left)
    return values


def run_group(label: str, spec: Spec, ogg: Path | None, wav: Path | None, out: Out) -> None:
    out()
    out(f"== {label} ({spec.kind}) ==")
    samples = None
    if ogg is None:
        out("Ogg: assente")
        out.result(f"[{label}] Ogg", False, "file assente")
    else:
        samples = check_ogg(ogg, spec, out)
    if wav is None:
        out("Master WAV: assente; picco, loudness, estremi, giro e stereo: non misurato")
        out.result(f"[{label}] master", None, "nessun master WAV")
    else:
        try:
            analyze_master(wav, spec, out, samples)
        except AudioFileError as e:
            out(f"{wav.name}: illeggibile: {e}")
            out.result(f"[{wav.name}] master", False, f"illeggibile: {e}")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Controlli INGEST di .audio-inbox/ (D-070)")
    parser.add_argument("targets", nargs="+", help="ID della richiesta oppure file .ogg/.wav")
    parser.add_argument("--version", type=int, help="versione da controllare (predefinita: "
                        "l'ultima di ogni parte)")
    parser.add_argument("--save", action="store_true",
                        help="scrive l'uscita in .audio-inbox/<ID>/check/v<N>.txt")
    parser.add_argument("--out", type=Path, help="scrive l'uscita anche in questo file")
    parser.add_argument("--kind", choices=("loop", "ambience", "stinger"))
    parser.add_argument("--samples", type=int)
    parser.add_argument("--tol", type=int)
    parser.add_argument("--max-bytes", type=int)
    parser.add_argument("--lufs", type=float)
    parser.add_argument("--lufs-tol", type=float)
    parser.add_argument("--bpm", type=float)
    parser.add_argument("--right-over-left", type=float, nargs=2, metavar=("MIN_DB", "MAX_DB"))
    parser.add_argument("--low-notes", choices=("criterio", "info"))
    args = parser.parse_args(argv)
    # in una pipe Windows userebbe cp1252; la console resta Unicode comunque
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    overrides = spec_overrides(args)
    if args.low_notes:
        overrides["low_notes"] = args.low_notes

    out = Out()
    started = time.perf_counter()
    out(f"audio_inbox.py {' '.join(argv)}")
    out(f"{datetime.datetime.now():%Y-%m-%d %H:%M}, Python {sys.version.split()[0]}")
    status = 0
    saves = []
    for target in args.targets:
        if target in REQUESTS:
            groups, strays = find_group(target, args.version)
            folder = INBOX / target
            out()
            out(f"#### {target} ({folder.relative_to(ROOT).as_posix()}/)")
            if strays:
                out("file con nome fuori schema o parte non prevista: " + ", ".join(strays))
            present = [f for g in groups for f in g[3:] if f is not None]
            if not present:
                out("nessun file consegnato: la richiesta resta REQUESTED")
                status = max(status, 2)
                continue
            check_provenance(folder, present, out)
            for part, spec, ver, ogg, wav in groups:
                label = f"{target}{'_' + part if part else ''}" + \
                    (f" v{ver}" if ver is not None else ": nessuna versione")
                run_group(label, replace(spec, **overrides), ogg, wav, out)
            vers = sorted({g[2] for g in groups if g[2] is not None})
            tag = f"v{vers[0]}" if len(vers) == 1 else \
                "_".join(f"{g[0]}_v{g[2]}" for g in groups if g[2] is not None)
            saves.append(folder / "check" / f"{tag}.txt")
            continue
        path = Path(target)
        if not path.is_file():
            out(f"{target}: né un ID della tabella né un file")
            status = max(status, 2)
            continue
        m = NAME_RE.match(path.name)
        spec = REQUESTS.get(m["id"], {}).get(m["part"] or "") if m else None
        if spec is None and not m:
            spec, note = promoted_spec(path)
            if note:
                out()
                out(note)
        spec = replace(spec or Spec(), **overrides)
        if path.suffix.lower() == ".ogg":
            out()
            check_ogg(path, spec, out)
        else:
            try:
                out()
                analyze_master(path, spec, out, None)
            except AudioFileError as e:
                out(f"{path.name}: illeggibile: {e}")
                status = max(status, 2)

    out()
    out("Riepilogo (OK / NO / non misurato / a vista / info):")
    for what, verdict, detail in out.results:
        out(f"  {verdict:12} {what}: {detail}")
    out(f"Tempo totale: {time.perf_counter() - started:.1f} s")
    status = max(status, 1 if out.no else 0)
    text = "\n".join(out.lines) + "\n"
    if args.save:
        for dest in saves:
            dest.parent.mkdir(exist_ok=True)
            dest.write_text(text, encoding="utf-8")
            print(f"scritto {dest.relative_to(ROOT).as_posix()}")
    if args.out:
        args.out.write_text(text, encoding="utf-8")
        print(f"scritto {args.out}")
    return status


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
