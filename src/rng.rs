//! Random deterministico con seed esplicito.
//!
//! Implementazione propria (PCG32 + SplitMix64, poche righe) invece di `rand`: le sequenze di
//! interi restano identiche tra versioni delle dipendenze e tra nativo e WASM. Stesso seed
//! sulla stessa piattaforma = stessa partita. Tra piattaforme diverse le posizioni possono
//! differire nell'ultimo bit perché `sin`/`cos` (es. `Vec2::from_angle`) usano la libreria
//! matematica del sistema; per il determinismo cross-platform servirebbe `libm` (backlog).

/// SplitMix64: mescola un `u64`. Usato per derivare seed e per gli hash di coordinate.
pub const fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hash deterministico di una cella di griglia `(x, y)` per un dato seed.
pub const fn hash2(seed: u64, x: i32, y: i32) -> u64 {
    let cell = ((x as u32 as u64) << 32) | (y as u32 as u64);
    splitmix64(seed ^ splitmix64(cell))
}

/// Converte un hash in un float uniforme in `[0, 1)`.
pub fn unit_f32(hash: u64) -> f32 {
    (hash >> 40) as f32 / (1u64 << 24) as f32
}

/// Generatore PCG32 (O'Neill, variante XSH-RR).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcg32 {
    state: u64,
    inc: u64,
}

impl Pcg32 {
    const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

    /// `stream` separa sequenze indipendenti con lo stesso seed (es. spawn vs drop).
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(Self::MULTIPLIER).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }

    /// Float uniforme in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Float uniforme in `[min, max)`.
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcg32_matches_reference_sequence() {
        // Valori del pcg32-demo ufficiale (pcg-random.org) con seed 42, stream 54.
        let mut rng = Pcg32::new(42, 54);
        let got: Vec<u32> = (0..6).map(|_| rng.next_u32()).collect();
        assert_eq!(
            got,
            [
                0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e
            ]
        );
    }

    #[test]
    fn same_seed_same_sequence_different_seed_differs() {
        let a: Vec<u32> = {
            let mut r = Pcg32::new(7, 1);
            (0..16).map(|_| r.next_u32()).collect()
        };
        let b: Vec<u32> = {
            let mut r = Pcg32::new(7, 1);
            (0..16).map(|_| r.next_u32()).collect()
        };
        let c: Vec<u32> = {
            let mut r = Pcg32::new(8, 1);
            (0..16).map(|_| r.next_u32()).collect()
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn floats_stay_in_range() {
        let mut rng = Pcg32::new(123, 0);
        for _ in 0..10_000 {
            let f = rng.next_f32();
            assert!((0.0..1.0).contains(&f));
            let r = rng.range_f32(-5.0, 5.0);
            assert!((-5.0..5.0).contains(&r));
        }
        for i in 0..1_000 {
            let u = unit_f32(hash2(9, i, -i));
            assert!((0.0..1.0).contains(&u));
        }
    }

    #[test]
    fn hash2_depends_on_all_inputs() {
        assert_eq!(hash2(1, 2, 3), hash2(1, 2, 3));
        assert_ne!(hash2(1, 2, 3), hash2(1, 3, 2));
        assert_ne!(hash2(1, 2, 3), hash2(2, 2, 3));
        assert_ne!(hash2(1, -1, 0), hash2(1, 0, -1));
    }
}
