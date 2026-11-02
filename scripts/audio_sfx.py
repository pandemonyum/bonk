#!/usr/bin/env python3
"""Sintetizza gli effetti sonori dalle patch testuali di assets-src/audio/sfx/*.json (D-070).

Come i .px degli sprite: una patch descrive il suono, lo script scrive il WAV in assets/ e la
voce nel manifest. Suoni originali del progetto, riproducibili, licenza "original". Solo
libreria standard (math, struct, json, random con seed fissa): niente numpy, campioni o IA.

Una patch (JSON; i campi sconosciuti sono un errore, come deny_unknown_fields nei .ron):
  {
    "cue": "ui.tick",                 nome del cue in assets/audio/catalog.ron (documentazione)
    "out": "audio/sfx/ui/tick.wav",   uscita relativa ad assets/
    "notes": "...",                   facoltativo: a cosa serve, com'è fatto
    "peak_dbfs": -3.0,                facoltativo: picco di campione finale, mai oltre -1
    "fade_in_ms": 2.0,                facoltativi: dissolvenze a coseno, almeno 1 ms
    "fade_out_ms": 5.0,
    "tail_db": -60.0,                 facoltativo: il silenzio finale sotto questa soglia
                                      (relativa al picco) si taglia
    "lowpass_hz": 7000,               facoltativo: passa-basso a un polo su tutto il suono
    "layers": [                       strati sommati, ognuno con il suo inizio
      {
        "wave": "square",             sine | square | triangle | noise
        "duty": 0.25,                 solo square: frazione del periodo in alto (0,05-0,95)
        "seed": 7,                    solo noise (obbligatoria): random.Random(seed)
        "freq_hz": 1320,              altezza; per noise la frequenza dei valori nuovi
                                      (sample and hold), assente = rumore bianco
        "start_ms": 0, "length_ms": 40,
        "gain": 1.0,
        "env": {"attack_ms": 0.5, "decay_ms": 30, "sustain": 0.0, "release_ms": 4,
                "curve": 2.0},        ADSR: attacco lineare; decadimento e rilascio con la
                                      curva (1 - x) ** curve (1 lineare, >1 più secco)
        "sweep": {"to_hz": 990, "time_ms": 30, "shape": "exp"},  exp | linear, poi fermo
        "vibrato": {"rate_hz": 6, "depth_cents": 20, "delay_ms": 0}
      }
    ]
  }

Catena: strati -> passa-basso -> taglio del silenzio finale -> dissolvenze -> picco a
peak_dbfs -> PCM 16 bit mono 44 100 Hz con l'header canonico di 44 byte. Il quadrato usa la
correzione PolyBLEP (meno aliasing) ed è a media nulla. Deterministico: stessa patch, stesso
file al byte (sulla stessa piattaforma; il rumore usa solo Random.random(), la cui sequenza
Python garantisce stabile tra le versioni). Un file uguale non si riscrive.

Uso: python -I scripts/audio_sfx.py           rigenera e aggiorna assets/manifest.json
     python -I scripts/audio_sfx.py --check   non scrive; exit code 1 se qualcosa cambierebbe
Poi: python -I scripts/validate-assets.py
"""

from __future__ import annotations

import hashlib
import json
import math
import random
import re
import sys
from pathlib import Path

# audio_common sta accanto a questo file (python -I non mette la cartella dello script nel path).
sys.path.insert(0, str(Path(__file__).resolve().parent))

from audio_common import (  # noqa: E402
    ASSETS,
    FULL_SCALE,
    MANIFEST,
    MAX_PEAK_DBFS,
    MAX_SFX_S,
    ROOT,
    SAMPLE_RATE,
    SFX_BITS,
    SFX_CHANNELS,
    SFX_GENERATOR,
    SFX_LICENSE,
    SFX_SRC,
    TRUE_PEAK_TAPS,
    dbfs,
    is_audio,
    load_manifest,
    manifest_text,
    parse_wav,
    true_peak_kernels,
    wav_bytes,
)

AUTHOR = "Bonk project (synthesized, pixel-art-director)"
OUT_RE = re.compile(r"^audio/sfx/[a-z0-9_]+/[a-z0-9_]+\.wav$")
WAVES = ("sine", "square", "triangle", "noise")
PATCH_KEYS = {"cue", "out", "notes", "peak_dbfs", "fade_in_ms", "fade_out_ms", "tail_db",
              "lowpass_hz", "layers"}
LAYER_KEYS = {"wave", "duty", "seed", "freq_hz", "start_ms", "length_ms", "gain", "env",
              "sweep", "vibrato"}
ENV_KEYS = {"attack_ms", "decay_ms", "sustain", "release_ms", "curve"}
SWEEP_KEYS = {"to_hz", "time_ms", "shape"}
VIBRATO_KEYS = {"rate_hz", "depth_cents", "delay_ms"}
DEFAULT_PEAK_DBFS = -3.0
MIN_FADE_MS = 1.0


class PatchError(Exception):
    pass


# --- Lettura e controllo delle patch ------------------------------------------------------


def number(obj: dict, key: str, where: str, default=None, low=None, high=None,
           low_open=False) -> float:
    value = obj.get(key, default)
    if value is None:
        raise PatchError(f"{where}.{key}: obbligatorio")
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise PatchError(f"{where}.{key} = {value!r}: serve un numero")
    if low is not None and (value <= low if low_open else value < low):
        raise PatchError(f"{where}.{key} = {value}: deve essere {'>' if low_open else '>='} {low}")
    if high is not None and value > high:
        raise PatchError(f"{where}.{key} = {value}: deve essere <= {high}")
    return float(value)


def only_keys(obj, allowed: set[str], where: str) -> dict:
    if not isinstance(obj, dict):
        raise PatchError(f"{where}: serve un oggetto")
    unknown = sorted(set(obj) - allowed)
    if unknown:
        raise PatchError(f"{where}: campi sconosciuti {unknown} (ammessi {sorted(allowed)})")
    return obj


def read_layer(raw, where: str) -> dict:
    only_keys(raw, LAYER_KEYS, where)
    wave = raw.get("wave")
    if wave not in WAVES:
        raise PatchError(f"{where}.wave = {wave!r}: uno di {list(WAVES)}")
    layer = {
        "wave": wave,
        "start": number(raw, "start_ms", where, 0.0, low=0) / 1000,
        "length": number(raw, "length_ms", where, low=0, low_open=True) / 1000,
        "gain": number(raw, "gain", where, 1.0, low=0, high=4, low_open=True),
    }
    if wave == "noise":
        seed = raw.get("seed")
        if isinstance(seed, bool) or not isinstance(seed, int):
            raise PatchError(f"{where}.seed: serve un intero (il rumore è riproducibile)")
        layer["seed"] = seed
        layer["freq"] = number(raw, "freq_hz", where, 0.0, low=0, high=SAMPLE_RATE)
    else:
        for key in ("seed",) + (() if wave == "square" else ("duty",)):
            if key in raw:
                raise PatchError(f"{where}.{key}: non vale per wave {wave!r}")
        layer["freq"] = number(raw, "freq_hz", where, low=20, high=SAMPLE_RATE / 2)
        if wave == "square":
            layer["duty"] = number(raw, "duty", where, 0.5, low=0.05, high=0.95)
    env = only_keys(raw.get("env", {}), ENV_KEYS, f"{where}.env")
    layer["env"] = (
        number(env, "attack_ms", f"{where}.env", 1.0, low=0) / 1000,
        number(env, "decay_ms", f"{where}.env", 0.0, low=0) / 1000,
        number(env, "sustain", f"{where}.env", 1.0, low=0, high=1),
        number(env, "release_ms", f"{where}.env", 5.0, low=0) / 1000,
        number(env, "curve", f"{where}.env", 1.0, low=0.1, high=8),
    )
    if layer["env"][3] > layer["length"]:
        raise PatchError(f"{where}.env.release_ms: più lungo dello strato")
    if "sweep" in raw:
        sweep = only_keys(raw["sweep"], SWEEP_KEYS, f"{where}.sweep")
        shape = sweep.get("shape", "exp")
        if shape not in ("exp", "linear"):
            raise PatchError(f"{where}.sweep.shape = {shape!r}: exp o linear")
        high = SAMPLE_RATE if wave == "noise" else SAMPLE_RATE / 2
        layer["sweep"] = (
            number(sweep, "to_hz", f"{where}.sweep", low=20, high=high),
            number(sweep, "time_ms", f"{where}.sweep", low=0, low_open=True) / 1000,
            shape,
        )
        if wave == "noise" and layer["freq"] == 0:
            raise PatchError(f"{where}.sweep: il rumore bianco non ha una frequenza da spostare")
    if "vibrato" in raw:
        vib = only_keys(raw["vibrato"], VIBRATO_KEYS, f"{where}.vibrato")
        layer["vibrato"] = (
            number(vib, "rate_hz", f"{where}.vibrato", low=0, high=40, low_open=True),
            number(vib, "depth_cents", f"{where}.vibrato", low=0, high=1200),
            number(vib, "delay_ms", f"{where}.vibrato", 0.0, low=0) / 1000,
        )
    return layer


def read_patch(path: Path) -> dict:
    where = path.name
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        raise PatchError(f"{where}: JSON non valido: {e}") from e
    only_keys(raw, PATCH_KEYS, where)
    out = raw.get("out")
    if not isinstance(out, str) or not OUT_RE.match(out):
        raise PatchError(f"{where}.out = {out!r}: serve audio/sfx/<cartella>/<nome>.wav "
                         "(minuscole, cifre e _)")
    layers = raw.get("layers")
    if not isinstance(layers, list) or not layers:
        raise PatchError(f"{where}.layers: serve almeno uno strato")
    patch = {
        "source": path.relative_to(ROOT).as_posix(),
        "out": out,
        "cue": raw.get("cue"),
        "peak": number(raw, "peak_dbfs", where, DEFAULT_PEAK_DBFS, low=-24, high=MAX_PEAK_DBFS),
        "fade_in": number(raw, "fade_in_ms", where, 2.0, low=MIN_FADE_MS, high=50) / 1000,
        "fade_out": number(raw, "fade_out_ms", where, 5.0, low=MIN_FADE_MS, high=200) / 1000,
        "tail_db": number(raw, "tail_db", where, -60.0, low=-96, high=-30),
        "lowpass": number(raw, "lowpass_hz", where, 0.0, low=0, high=SAMPLE_RATE / 2),
        "layers": [read_layer(layer, f"{where}.layers[{i}]") for i, layer in enumerate(layers)],
    }
    end = max(layer["start"] + layer["length"] for layer in patch["layers"])
    if end > MAX_SFX_S:
        raise PatchError(f"{where}: {end:.3f} s, gli effetti stanno entro {MAX_SFX_S} s")
    return patch


# --- Sintesi -------------------------------------------------------------------------------


def envelope(t: float, length: float, env: tuple) -> float:
    attack, decay, sustain, release, curve = env

    def ads(t: float) -> float:
        if t < attack:
            return t / attack
        t -= attack
        if t < decay:
            return sustain + (1 - sustain) * (1 - t / decay) ** curve
        return sustain

    release_start = length - release
    if t < release_start:
        return ads(t)
    x = (t - release_start) / release if release > 0 else 1.0
    return ads(release_start) * (1 - min(x, 1.0)) ** curve


def frequency(t: float, layer: dict) -> float:
    f = layer["freq"]
    if "sweep" in layer:
        to, time, shape = layer["sweep"]
        x = min(t / time, 1.0)
        f = f * (to / f) ** x if shape == "exp" else f + (to - f) * x
    if "vibrato" in layer:
        rate, cents, delay = layer["vibrato"]
        if t >= delay:
            f *= 2 ** (cents / 1200 * math.sin(2 * math.pi * rate * (t - delay)))
    return f


def poly_blep(phase: float, dt: float) -> float:
    """Correzione di un gradino in phase = 0 (PolyBLEP a due campioni)."""
    if phase < dt:
        x = phase / dt
        return x + x - x * x - 1
    if phase > 1 - dt:
        x = (phase - 1) / dt
        return x * x + x + x + 1
    return 0.0


def render_layer(layer: dict, buffer: list[float]) -> None:
    first = round(layer["start"] * SAMPLE_RATE)
    count = round(layer["length"] * SAMPLE_RATE)
    wave, gain, env = layer["wave"], layer["gain"], layer["env"]
    rng = random.Random(layer["seed"]) if wave == "noise" else None
    held = 2 * rng.random() - 1 if rng else 0.0
    phase = 0.0
    for i in range(count):
        t = i / SAMPLE_RATE
        f = frequency(t, layer)
        dt = f / SAMPLE_RATE
        if wave == "sine":
            value = math.sin(2 * math.pi * phase)
        elif wave == "triangle":
            value = 1 - 4 * abs((phase + 0.25) % 1.0 - 0.5)  # parte da 0 salendo, come il seno
        elif wave == "square":
            duty = layer["duty"]
            value = 1.0 if phase < duty else -1.0
            value += poly_blep(phase, dt) - poly_blep((phase - duty) % 1.0, dt)
            value -= 2 * duty - 1  # media nulla: nessun salto di continua sotto l'inviluppo
        elif f <= 0:
            value = 2 * rng.random() - 1  # rumore bianco
        else:
            value = held  # sample and hold: un valore nuovo a ogni giro della fase
        buffer[first + i] += gain * envelope(t, layer["length"], env) * value
        phase += dt
        if phase >= 1.0:
            phase -= math.floor(phase)
            if rng and f > 0:
                held = 2 * rng.random() - 1


def lowpass(signal: list[float], cutoff: float) -> list[float]:
    a = 1 - math.exp(-2 * math.pi * cutoff / SAMPLE_RATE)
    out, y = [], 0.0
    for x in signal:
        y += a * (x - y)
        out.append(y)
    return out


def raised_cosine(i: int, n: int) -> float:
    """Peso della dissolvenza: 0 al campione 0, 1 dal campione n in poi."""
    return 0.5 - 0.5 * math.cos(math.pi * i / n) if i < n else 1.0


def synthesize(patch: dict) -> list[int]:
    end = max(layer["start"] + layer["length"] for layer in patch["layers"])
    buffer = [0.0] * (round(end * SAMPLE_RATE) + 1)
    for layer in patch["layers"]:
        render_layer(layer, buffer)
    if patch["lowpass"] > 0:
        buffer = lowpass(buffer, patch["lowpass"])
    peak = max(abs(x) for x in buffer)
    if peak == 0:
        raise PatchError(f"{patch['source']}: il suono è silenzio")
    # taglio del silenzio finale, poi dissolvenze (il primo e l'ultimo campione valgono 0)
    floor = peak * 10 ** (patch["tail_db"] / 20)
    last = max(i for i, x in enumerate(buffer) if abs(x) > floor)
    fade_in = round(patch["fade_in"] * SAMPLE_RATE)
    fade_out = round(patch["fade_out"] * SAMPLE_RATE)
    signal = buffer[: max(last + 1, fade_in + fade_out) + 1]
    n = len(signal)
    signal = [x * raised_cosine(i, fade_in) * raised_cosine(n - 1 - i, fade_out)
              for i, x in enumerate(signal)]
    scale = 10 ** (patch["peak"] / 20) * FULL_SCALE / max(abs(x) for x in signal)
    return [max(-FULL_SCALE, min(FULL_SCALE - 1, math.floor(x * scale + 0.5))) for x in signal]


def true_peak_dbfs(samples: list[int]) -> float:
    """Stima del picco reale tra i campioni: interpolazione 4x con un sinc a finestra di Hann."""
    taps = TRUE_PEAK_TAPS
    kernels = true_peak_kernels(taps)
    padded = [0] * taps + samples + [0] * taps
    peak = max(abs(s) for s in samples)
    for n in range(len(samples) + 1):
        window = padded[n + 1 : n + 2 * taps + 1]  # campioni da n - taps + 1 a n + taps
        for kernel in kernels:
            peak = max(peak, abs(sum(w * h for w, h in zip(window, kernel))))
    return dbfs(peak)


# --- Uscite e manifest ---------------------------------------------------------------------


def manifest_entry(patch: dict, data: bytes, previous: dict | None) -> dict:
    info = parse_wav(data)
    digest = hashlib.sha256(data).hexdigest()
    # l'approvazione dell'utente vale per quel file: se il suono cambia torna da approvare
    approved = previous.get("approved") if previous and previous.get("sha256") == digest else None
    return {
        "path": patch["out"],
        "kind": "sfx",
        "channels": info.channels,
        "sample_rate": info.sample_rate,
        "bits_per_sample": info.bits,
        "samples": info.frames,
        "duration_s": round(info.duration_s, 4),
        "peak_dbfs": round(info.peak_dbfs, 2),
        "sha256": digest,
        "cue": patch["cue"],
        "source": patch["source"],
        "generator": SFX_GENERATOR,
        "author": AUTHOR,
        "license": SFX_LICENSE,
        "approved": approved,
    }


def main(argv: list[str]) -> int:
    check = "--check" in argv
    patches, errors = [], []
    for path in sorted(SFX_SRC.glob("*.json")):
        try:
            patches.append(read_patch(path))
        except PatchError as e:
            errors.append(str(e))
    outs = [p["out"] for p in patches]
    errors += [f"{out}: uscita di più patch" for out in sorted({o for o in outs if outs.count(o) > 1})]
    if errors:
        for e in errors:
            print("ERRORE:", e)
        return 1

    manifest = load_manifest()["assets"]
    previous = {e["path"]: e for e in manifest if e.get("kind") == "sfx"}
    # voci sfx di altre origini (per esempio file dell'utente) restano come sono
    kept = [e for e in manifest if is_audio(e) and not (
        e.get("kind") == "sfx" and e.get("source", "").startswith("assets-src/audio/sfx/"))]
    entries, changed = [], []
    for patch in patches:
        samples = synthesize(patch)
        data = wav_bytes(samples)
        info = parse_wav(data)
        if (info.channels, info.sample_rate, info.bits) != (SFX_CHANNELS, SAMPLE_RATE, SFX_BITS):
            raise AssertionError(f"{patch['out']}: header inatteso {info}")
        true_peak = true_peak_dbfs(samples)
        if info.peak_dbfs > MAX_PEAK_DBFS or true_peak > MAX_PEAK_DBFS:
            print(f"ERRORE: {patch['out']}: picco {info.peak_dbfs:.2f} dBFS, "
                  f"reale stimato {true_peak:.2f} dBFS (massimo {MAX_PEAK_DBFS})")
            return 1
        dest = ASSETS / patch["out"]
        same = dest.is_file() and dest.read_bytes() == data
        if not same:
            changed.append(patch["out"])
            if not check:
                dest.parent.mkdir(parents=True, exist_ok=True)
                dest.write_bytes(data)
        entry = manifest_entry(patch, data, previous.get(patch["out"]))
        entries.append(entry)
        print(f"  {patch['out']:32} {info.duration_s * 1000:6.1f} ms  {info.frames:5} campioni  "
              f"{len(data):6} byte  picco {info.peak_dbfs:6.2f} dBFS  reale ~{true_peak:6.2f} dBFS"
              f"  sha256 {entry['sha256'][:12]}  "
              f"{'invariato' if same else 'da scrivere' if check else 'scritto'}")

    audio = sorted(kept + entries, key=lambda e: e["path"])
    text = manifest_text([e for e in manifest if not is_audio(e)] + audio)
    if MANIFEST.read_text(encoding="utf-8") != text:
        changed.append("manifest.json")
        if not check:
            MANIFEST.write_text(text, encoding="utf-8")

    stray = sorted(p.relative_to(ASSETS).as_posix() for p in (ASSETS / "audio" / "sfx").rglob("*")
                   if p.is_file() and p.relative_to(ASSETS).as_posix() not in
                   {e["path"] for e in audio})
    for path in stray:
        print(f"AVVISO: {path}: nessuna patch né voce nel manifest (non lo cancello)")
    if check:
        print(f"{'DA RIGENERARE' if changed else 'OK'}: {len(patches)} patch, "
              f"{len(changed)} file diversi {changed if changed else ''}".rstrip())
        return 1 if changed else 0
    print(f"OK: {len(patches)} effetti da {SFX_SRC.relative_to(ROOT).as_posix()}/, "
          f"{len(changed)} file scritti {changed if changed else ''}".rstrip())
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
