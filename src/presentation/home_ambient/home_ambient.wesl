// Animazioni ambientali della home (D-067). Ogni nodo copre un rettangolo dell'illustrazione e
// ridisegna solo i pixel della sua maschera, partendo dai pixel dell'illustrazione stessa: dove la
// maschera è zero il fragment si scarta e resta il fondo. Le coordinate `p` sono texel
// dell'illustrazione (1 = un pixel del PNG), così tutto segue il ridimensionamento e il taglio
// del fondo. Il tempo arriva come quattro fasi 0..1 (`params.phases`): si usano solo funzioni
// periodiche di una fase o di un suo multiplo intero, e il ciclo non salta mai.
// Canali delle maschere: scripts/art_home_ambient.py (CHANNELS).
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

struct AmbientParams {
    node_uv: vec4<f32>,
    image: vec4<f32>,
    mask_rect: vec4<f32>,
    phases: vec4<f32>,
    shape: vec4<f32>,
    shape2: vec4<f32>,
}

@group(1) @binding(0) var<uniform> params: AmbientParams;
@group(1) @binding(1) var art_texture: texture_2d<f32>;
@group(1) @binding(2) var art_sampler: sampler;
@group(1) @binding(3) var mask_texture: texture_2d<f32>;
@group(1) @binding(4) var mask_sampler: sampler;
@group(1) @binding(5) var plate_texture: texture_2d<f32>;
@group(1) @binding(6) var plate_sampler: sampler;

const TAU: f32 = 6.2831853;

// Colori in spazio lineare (le texture sRGB arrivano già lineari).
const SWIRL_LIGHT: vec3<f32> = vec3<f32>(0.45, 0.95, 1.0);
const RUNE_LIGHT: vec3<f32> = vec3<f32>(0.25, 0.75, 1.0);
const HALO_LIGHT: vec3<f32> = vec3<f32>(0.15, 0.55, 1.0);
const SPARK: vec3<f32> = vec3<f32>(0.8, 1.0, 1.0);
const FOAM: vec3<f32> = vec3<f32>(0.92, 0.97, 1.0);
const GLINT: vec3<f32> = vec3<f32>(0.95, 1.0, 1.0);
const EMBER: vec3<f32> = vec3<f32>(1.0, 0.75, 0.25);
const MOTE_CORE: vec3<f32> = vec3<f32>(1.0, 1.0, 0.8);
const MOTE_GLOW: vec3<f32> = vec3<f32>(1.0, 0.78, 0.22);

fn art(p: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(art_texture, art_sampler, p / params.image.xy, 0.0).rgb;
}

fn mask_uv(p: vec2<f32>) -> vec2<f32> {
    return (p - params.mask_rect.xy) / params.mask_rect.zw;
}

fn mask(p: vec2<f32>) -> vec4<f32> {
    let uv = mask_uv(p);
    // fuori dal rettangolo niente effetto (il campionatore ripeterebbe il bordo)
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return vec4<f32>(0.0);
    }
    return textureSampleLevel(mask_texture, mask_sampler, uv, 0.0);
}

fn plate(p: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(plate_texture, plate_sampler, mask_uv(p), 0.0).rgb;
}

// Hash senza seno (precisione uguale su desktop e WebGL2).
fn hash11(x: f32) -> f32 {
    var v = fract(x * 0.1031);
    v = v * (v + 33.33);
    v = v * (v + v);
    return fract(v);
}

fn hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Rumore liscio nello spazio e periodico nel tempo: ogni nodo della griglia oscilla con la sua
// fase. Da -1 a 1.
fn wave_noise(p: vec2<f32>, phase: f32) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = cos(TAU * (phase + hash12(i)));
    let b = cos(TAU * (phase + hash12(i + vec2<f32>(1.0, 0.0))));
    let c = cos(TAU * (phase + hash12(i + vec2<f32>(0.0, 1.0))));
    let d = cos(TAU * (phase + hash12(i + vec2<f32>(1.0, 1.0))));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// --- Portale ------------------------------------------------------------------------------------
// shape: centro e semiassi del vortice; shape2.x: squadratura del superellisse, shape2.y: verso.

fn level(q: vec2<f32>, n: f32) -> f32 {
    let a = max(abs(q), vec2<f32>(1e-5));
    return pow(pow(a.x, n) + pow(a.y, n), 1.0 / n);
}

// Particelle che escono ogni tanto dal bordo del vortice verso l'alto e svaniscono. Ognuna
// rinasce `k` volte per giro della fase in punti diversi, sempre gli stessi a ogni giro.
fn sparkles(cell: vec2<f32>, center: vec2<f32>, radii: vec2<f32>, n: f32, phase: f32) -> f32 {
    var light = 0.0;
    for (var i = 0; i < 12; i += 1) {
        let fi = f32(i);
        let k = 2.0 + floor(hash11(fi * 7.13 + 1.0) * 3.0);
        let life = phase * k + hash11(fi * 3.71 + 2.0);
        let s = fract(life);
        let seed = fi * 13.0 + (floor(life) % k);
        if hash11(seed * 1.37 + 5.0) < 0.4 {
            continue;
        }
        // metà alta e fianchi: in basso ci sono le rovine davanti al portale
        let angle = mix(-3.0, -0.15, hash11(seed * 2.11 + 9.0));
        let dir = vec2<f32>(cos(angle), sin(angle));
        let start = center + dir * (0.9 / level(dir, n)) * radii;
        let travel = 24.0 * (1.0 - (1.0 - s) * (1.0 - s));
        let at = floor(start + dir * travel + vec2<f32>(0.0, -10.0 * s * s));
        let fade = sin(3.14159 * s);
        let d = abs(cell - at);
        if d.x + d.y < 0.5 {
            light = max(light, fade);
        } else if d.x + d.y < 1.5 && hash11(seed * 4.3 + 3.0) > 0.5 {
            light = max(light, fade * 0.45);
        }
    }
    return light;
}

fn portal(p: vec2<f32>) -> vec4<f32> {
    let m = mask(p);
    let base = art(p);
    let center = params.shape.xy;
    let radii = params.shape.zw;
    let n = params.shape2.x;
    let spin = params.shape2.y;
    let ph = params.phases;
    let pulse = 0.5 + 0.5 * sin(TAU * ph.z);
    var color = base;

    if m.r > 0.001 {
        // il vortice dipinto gira attorno all'occhio lungo le curve del superellisse (ogni texel
        // resta alla sua distanza dal centro); il bordo resta fermo
        let q = (p - center) / radii;
        let rho = level(q, n);
        let theta = atan2(q.y, q.x) - spin * TAU * ph.x;
        let u = vec2<f32>(cos(theta), sin(theta));
        let turned = plate(center + u * (rho / level(u, n)) * radii);
        let rim = smoothstep(0.78, 1.0, rho);
        color = mix(turned, base, rim) * (0.94 + 0.12 * pulse);
        // bracci di luce che scendono a spirale verso il centro, avvolti come quelli dipinti
        // (log r = spin k theta + ...), a texel interi
        let qc = (floor(p) + 0.5 - center) / radii;
        let rc = max(level(qc, n), 0.02);
        let arm = fract(1.6 * log(rc) - 2.0 * spin * atan2(qc.y, qc.x) / TAU + ph.y);
        let band = smoothstep(0.86, 0.98, abs(2.0 * arm - 1.0));
        let fade = (1.0 - rim) * smoothstep(0.08, 0.35, rc);
        color = color + SWIRL_LIGHT * band * fade * 0.16;
    }
    // rune: un'onda di luce che sale lungo l'arco
    let rune_pulse = 0.5 + 0.5 * sin(TAU * (ph.z + (center.y - p.y) / 160.0));
    color = color + RUNE_LIGHT * m.g * (0.08 + 0.42 * rune_pulse);
    // alone ciano attorno all'apertura
    color = color + HALO_LIGHT * m.b * (0.06 + 0.2 * pulse);

    let spark = sparkles(floor(p), center, radii, n, ph.w);
    color = mix(color, SPARK, spark);
    return vec4<f32>(color, max(max(m.r, m.g), max(m.b, spark)));
}

// --- Cascate ------------------------------------------------------------------------------------
// shape: lip, base, velocità, texel di un giro della fase (FLOW_PERIOD in material.rs);
// shape2: pozza (cx, cy, rx, ry).

// L'acqua `d` texel più in alto: sopra il bordo della cascata (roccia) resta il pixel fermo.
fn falling(p: vec2<f32>, d: f32, still: vec3<f32>) -> vec3<f32> {
    let src = p - vec2<f32>(0.0, d);
    return mix(still, art(src), mask(src).r);
}

fn waterfall(p: vec2<f32>) -> vec4<f32> {
    let m = mask(p);
    let base = art(p);
    let ph = params.phases;
    var color = base;

    if m.r > 0.001 {
        let period = params.shape.w;
        // due letture sfalsate di mezzo periodo; ognuna sparisce quando ricomincia
        let near = ph.x * period;
        let far = fract(ph.x + 0.5) * period;
        let w = 1.0 - abs(2.0 * ph.x - 1.0);
        let flow = mix(falling(p, far, base), falling(p, near, base), w);
        // strisce chiare che scendono, colonna per colonna, a texel interi
        let column = floor(p.x);
        let h = hash11(column * 1.7 + 3.0);
        let lane = fract((floor(p.y) - params.shape.x) / period - ph.x + h);
        let dash = 0.12 + 0.18 * hash11(column * 2.3 + 1.0);
        // le strisce si sciolgono nella schiuma sopra la base
        let landing = 1.0 - smoothstep(params.shape.y - 10.0, params.shape.y, floor(p.y));
        let streak = select(0.0, 1.0, lane < dash && h > 0.45) * landing;
        color = mix(base, flow, m.r);
        color = mix(color, FOAM, streak * 0.3 * m.r);
    }
    if m.g > 0.001 {
        // la schiuma ribolle: piccoli spostamenti e scintille che cambiano forma
        let wobble = vec2<f32>(wave_noise(p / 3.0, ph.y), wave_noise(p / 3.0 + 17.0, ph.y));
        let src = p + wobble * vec2<f32>(1.4, 0.9);
        let moved = mix(base, art(src), mask(src).g);
        let glitter = max(wave_noise(floor(p) / 2.0 + 5.0, ph.w), 0.0);
        color = mix(color, moved, m.g) + FOAM * glitter * 0.14 * m.g;
    }
    if m.b > 0.001 && params.shape2.z > 0.0 {
        // cerchi che si allargano dalla caduta, schiacciati dalla prospettiva
        let e = length((floor(p) + 0.5 - params.shape2.xy) / params.shape2.zw);
        let ring = fract(3.0 * e - ph.z);
        let line = 1.0 - smoothstep(0.0, 0.18, ring);
        color = color + FOAM * line * m.b * 0.3 * (1.0 - e);
    }
    return vec4<f32>(color, max(m.r, max(m.g, m.b)));
}

// --- Fiume --------------------------------------------------------------------------------------

fn river(p: vec2<f32>) -> vec4<f32> {
    let m = mask(p);
    if m.r <= 0.001 {
        return vec4<f32>(0.0);
    }
    let base = art(p);
    let ph = params.phases;
    // l'acqua ondeggia appena, solo lontano dalle rive
    let sway = sin(TAU * (ph.y + floor(p.y) / 9.0)) * 0.6 * m.g;
    var color = mix(base, art(p + vec2<f32>(sway, 0.0)), m.g);
    // le creste chiare dipinte si accendono e si spengono a onde
    let crest = 0.5 + 0.5 * sin(TAU * (ph.z + p.x / 23.0 + p.y / 7.0));
    color = color + FOAM * m.b * (crest - 0.35) * 0.18;
    // riflessi: brevi trattini di luce, ognuno al suo momento
    let size = vec2<f32>(9.0, 4.0);
    let cell = floor(p / size);
    let h = hash12(cell);
    let k = 1.0 + floor(hash12(cell + 7.3) * 2.0);
    let shine = smoothstep(0.9, 1.0, cos(TAU * (ph.x * k + h)));
    let local = floor(p - cell * size);
    let spot = vec2<f32>(1.0 + floor(h * 5.0), 1.0 + floor(fract(h * 7.0) * 2.0));
    let on_line = local.y == spot.y && local.x >= spot.x && local.x < spot.x + 3.0;
    color = mix(color, GLINT, select(0.0, shine, on_line) * 0.6 * m.g);
    return vec4<f32>(color, m.r);
}

// --- Fiamma -------------------------------------------------------------------------------------
// shape.y: riga dove la fiamma esce dalla coda.

fn flame(p: vec2<f32>) -> vec4<f32> {
    let m = mask(p);
    if m.r <= 0.001 {
        return vec4<f32>(0.0);
    }
    let ph = params.phases;
    // più guizzi in cima che alla base; si legge il pixel più in basso: la fiamma sale
    let height = clamp((params.shape.y - p.y) / 70.0, 0.0, 1.0);
    let cell = vec2<f32>(p.x / 4.0, p.y / 5.0);
    let nx = wave_noise(cell, ph.x);
    let ny = wave_noise(cell + 9.0, ph.y);
    let offset = vec2<f32>(nx * 1.6, 0.8 + ny * 1.2) * (0.25 + 0.75 * height);
    var color = art(p + offset);
    color = color + EMBER * m.g * (0.05 + 0.12 * (0.5 + 0.5 * sin(TAU * ph.z)));
    return vec4<f32>(color, m.r);
}

// --- Lucciole ----------------------------------------------------------------------------------
// shape.x: lato della cella in texel, shape.y: quota delle celle con una lucciola.

// Al più una lucciola per cella: gira piano attorno al suo punto, si accende e si spegne, e si
// vede solo dove la maschera dice che ci sono foglie vicine. Stellina di pixel 5x5 (centro e
// croce pieni, diagonali e punte più tenui): pixel art, non un alone sfumato.
fn motes(p: vec2<f32>) -> vec4<f32> {
    let m = mask(p);
    if m.r <= 0.001 {
        return vec4<f32>(0.0);
    }
    let size = params.shape.x;
    let share = params.shape.y;
    let ph = params.phases;
    let texel = floor(p);
    let home = floor(p / size);
    // centro chiaro e braccia color miele, separati: sul verde chiaro si staccano meglio
    var core = 0.0;
    var glow = 0.0;
    for (var j = -1; j <= 1; j += 1) {
        for (var i = -1; i <= 1; i += 1) {
            let cell = home + vec2<f32>(f32(i), f32(j));
            let h = hash12(cell + 0.37);
            if h > share {
                continue;
            }
            let k = 1.0 + floor(hash12(cell + 3.1) * 2.0);
            let a = hash12(cell + 5.7);
            let b = hash12(cell + 9.2);
            let center = (cell + vec2<f32>(0.25 + 0.5 * a, 0.25 + 0.5 * b)) * size;
            let drift = vec2<f32>(sin(TAU * (ph.x * k + a)), cos(TAU * (ph.y * k + b)));
            let at = floor(center + drift * size * vec2<f32>(0.22, 0.15));
            if mask(at + 0.5).r < 0.5 {
                continue;
            }
            let on = clamp(0.4 + 0.9 * sin(TAU * (ph.z * k + 7.0 * h)), 0.0, 1.0);
            let d = abs(texel - at);
            let s = d.x + d.y;
            if s < 0.5 {
                core = max(core, on);
            } else if s < 1.5 {
                glow = max(glow, 0.8 * on);
            } else if d.x < 1.5 && d.y < 1.5 {
                glow = max(glow, 0.35 * on);
            } else if s < 2.5 && min(d.x, d.y) < 0.5 {
                glow = max(glow, 0.3 * on);
            }
        }
    }
    if max(core, glow) <= 0.002 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(mix(mix(art(p), MOTE_GLOW, glow), MOTE_CORE, core), 1.0);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = params.node_uv.xy + in.uv * params.node_uv.zw;
    let p = uv * params.image.xy;
    let kind = params.image.z;
    var out: vec4<f32>;
    if kind < 0.5 {
        out = portal(p);
    } else if kind < 1.5 {
        out = waterfall(p);
    } else if kind < 2.5 {
        out = river(p);
    } else if kind < 3.5 {
        out = flame(p);
    } else {
        out = motes(p);
    }
    let alpha = clamp(out.a * params.image.w, 0.0, 1.0);
    if alpha <= 0.002 {
        discard;
    }
    return vec4<f32>(out.rgb, alpha);
}
