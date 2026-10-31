//! Da `BiomeData` (nomi e pesi) a `Biome` (indici e soglie), con tutti gli errori dei dati: è la
//! validazione dei biomi (`data::validate` la riporta).

use super::*;

impl Biome {
    /// Nomi e pesi diventano indici e soglie; tutti gli errori, con file, bioma e campo.
    pub fn compile(data: &BiomeData) -> Result<Self, Vec<String>> {
        let mut c = Compiler {
            prefix: format!("stages.ron: biomes.{}", data.id),
            errors: Vec::new(),
        };
        let t = &data.terrain;
        let names = Self::terrain_names(data);
        c.non_empty("terrain.tiles", &t.tiles);
        c.unique("terrain.tiles", &names);
        for material in &t.edges {
            c.index(&t.tiles, "terrain.edges", material);
        }
        let material = |c: &mut Compiler, field: &str, name: &str| Material {
            tile: c.index(&names, field, name),
            edges: t
                .edges
                .iter()
                .position(|m| m == name)
                .map(|k| (t.tiles.len() + k * EDGE_TILES as usize) as u16),
        };
        c.non_empty("terrain.ground", &t.ground);
        let ground_tiles: Vec<u16> = (t.ground.iter().enumerate())
            .map(|(i, (name, _))| c.index(&names, &format!("terrain.ground[{i}]"), name))
            .collect();
        let weights: Vec<f64> = t.ground.iter().map(|(_, w)| *w).collect();
        if !t.ground.is_empty() && weights.iter().sum::<f64>() <= 0.0 {
            c.error("terrain.ground", "serve almeno un peso > 0");
        }
        let ground = ground_tiles
            .into_iter()
            .zip(c.thresholds("terrain.ground", &weights))
            .collect();
        let border = material(&mut c, "terrain.border", &t.border);
        let patch = t.patch.as_ref().map(|p| Patch {
            material: material(&mut c, "terrain.patch.tile", &p.tile),
            cell: c.cell("terrain.patch.cell", p.cell),
            // con 1 non ci sarebbe nessuna macchia, e le regole sulle macchie non varrebbero mai
            threshold: c.below_one("terrain.patch.threshold", p.threshold),
        });
        // Base, macchie e muro sono materiali distinti: i bordi e gli ostacoli li riconoscono
        // dal tile logico.
        for (name, _) in &t.ground {
            let clash = *name == t.border || t.patch.as_ref().is_some_and(|p| p.tile == *name);
            if clash {
                c.error(
                    "terrain.ground",
                    &format!("\"{name}\" è già muro o macchia"),
                );
            }
        }
        if t.patch.as_ref().is_some_and(|p| p.tile == t.border) {
            c.error("terrain.patch.tile", "deve essere diverso dal muro");
        }
        // Una variante o un gruppo che mostra muro, macchia o un bordo farebbe sembrare un
        // materiale quello che per la logica è un altro.
        let materials: Vec<&str> = std::iter::once(t.border.as_str())
            .chain(t.patch.as_ref().map(|p| p.tile.as_str()))
            .collect();
        let shows_material = |name: &str| {
            materials.contains(&name)
                || names[t.tiles.len().min(names.len())..]
                    .iter()
                    .any(|n| n == name)
        };
        let mut variants: Vec<Variant> = Vec::new();
        for (i, (from, to, below)) in t.variants.iter().enumerate() {
            let at = format!("terrain.variants[{i}]");
            if shows_material(to) {
                c.error(&at, &format!("\"{to}\" è un muro, una macchia o un bordo"));
            }
            // e al contrario: muro e macchie non si travestono da terreno di base
            if materials.contains(&from.as_str()) && t.ground.iter().any(|(g, _)| g == to) {
                c.error(
                    &at,
                    &format!("\"{from}\" non può sembrare il terreno di base \"{to}\""),
                );
            }
            let variant = Variant {
                from: c.index(&names, &at, from),
                to: c.index(&names, &at, to),
                below: c.unit(&at, *below),
            };
            if (variants.iter()).any(|v| v.from == variant.from && v.below >= variant.below) {
                c.error(&at, "le soglie dello stesso tile vanno in ordine crescente");
            }
            variants.push(variant);
        }
        let clusters = t.clusters.as_ref().map(|k| {
            for name in k.tiles.iter().filter(|n| shows_material(n)) {
                c.error(
                    "terrain.clusters.tiles",
                    &format!("\"{name}\" è un muro, una macchia o un bordo"),
                );
            }
            Clusters {
                tiles: c.indices(&names, "terrain.clusters.tiles", &k.tiles),
                cell: c.cell("terrain.clusters.cell", k.cell),
                threshold: c.below_one("terrain.clusters.threshold", k.threshold),
                share: c.positive_unit("terrain.clusters.share", k.share),
            }
        });
        let decorations = data.decorations.as_ref().map(|d| {
            c.non_empty("decorations.tiles", &d.tiles);
            c.unique("decorations.tiles", &d.tiles);
            let rules = (d.rules.iter().enumerate())
                .map(|(i, rule)| {
                    let at = |field: &str| format!("decorations.rules[{i}].{field}");
                    if rule.on == Surface::Patch && t.patch.is_none() {
                        c.error(&at("on"), "il terreno non ha macchie");
                    }
                    DecorRule {
                        tiles: c.indices(&d.tiles, &at("tiles"), &rule.tiles),
                        on: rule.on,
                        density: c.positive_unit(&at("density"), rule.density),
                        cluster: rule.cluster.map(|(cell, threshold)| {
                            (
                                c.cell(&at("cluster"), cell),
                                c.below_one(&at("cluster"), threshold),
                            )
                        }),
                    }
                })
                .collect();
            Decorations {
                tileset: d.tileset.clone(),
                tile_count: d.tiles.len() as u32,
                rules,
            }
        });
        let obstacles = c.obstacles(data);
        if c.errors.is_empty() {
            Ok(Self {
                id: data.id.clone(),
                terrain: Terrain {
                    tileset: t.tileset.clone(),
                    tile_count: names.len() as u32,
                    ground,
                    patch,
                    border,
                    variants,
                    clusters,
                },
                decorations,
                obstacles,
            })
        } else {
            Err(c.errors)
        }
    }
}

struct Compiler {
    prefix: String,
    errors: Vec<String>,
}

impl Compiler {
    fn error(&mut self, field: &str, message: &str) {
        self.errors
            .push(format!("{}.{field}: {message}", self.prefix));
    }

    fn index(&mut self, names: &[String], field: &str, name: &str) -> u16 {
        match names.iter().position(|n| n == name) {
            Some(index) => index as u16,
            None => {
                self.error(field, &format!("\"{name}\" non è un tile del tileset"));
                0
            }
        }
    }

    fn indices(&mut self, names: &[String], field: &str, wanted: &[String]) -> Vec<u16> {
        self.non_empty(field, wanted);
        wanted.iter().map(|n| self.index(names, field, n)).collect()
    }

    fn non_empty<T>(&mut self, field: &str, list: &[T]) {
        if list.is_empty() {
            self.error(field, "serve almeno un tile");
        }
    }

    fn unique(&mut self, field: &str, names: &[String]) {
        for (i, name) in names.iter().enumerate() {
            if names[..i].contains(name) {
                self.error(field, &format!("\"{name}\" duplicato"));
            }
        }
    }

    fn unit(&mut self, field: &str, value: f32) -> f32 {
        if !(0.0..=1.0).contains(&value) {
            self.error(field, &format!("deve essere tra 0 e 1, trovato {value}"));
        }
        value
    }

    /// Quota che deve dare qualcosa: 0 sarebbe una regola che non fa nulla.
    fn positive_unit(&mut self, field: &str, value: f32) -> f32 {
        if !(value > 0.0 && value <= 1.0) {
            self.error(field, &format!("deve essere > 0 e <= 1, trovato {value}"));
        }
        value
    }

    /// Soglia del rumore che qualche cella supera (il rumore è sotto 1).
    fn below_one(&mut self, field: &str, value: f32) -> f32 {
        if !(0.0..1.0).contains(&value) {
            self.error(
                field,
                &format!("soglia tra 0 e 1 (esclusa), trovato {value}"),
            );
        }
        value
    }

    fn cell(&mut self, field: &str, value: u32) -> i32 {
        if !(1..=64).contains(&value) {
            self.error(
                field,
                &format!("lato della cella tra 1 e 64 tile, trovato {value}"),
            );
        }
        value.clamp(1, 64) as i32
    }

    /// Soglie cumulate di pesi non negativi, normalizzate a 1 (tutte 0 se i pesi lo sono:
    /// quel tiro non sceglie mai). Sommate in f64 e convertite alla fine: pesi come 0,42 + 0,42 +
    /// 0,09 danno esattamente le soglie f32 0,84 e 0,93.
    fn thresholds(&mut self, field: &str, weights: &[f64]) -> Vec<f32> {
        if weights.iter().any(|w| !(w.is_finite() && *w >= 0.0)) {
            self.error(field, "i pesi devono essere >= 0");
            return vec![0.0; weights.len()];
        }
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return vec![0.0; weights.len()];
        }
        let mut sum = 0.0;
        (weights.iter())
            .map(|w| {
                sum += w;
                (sum / total) as f32
            })
            .collect()
    }

    fn obstacles(&mut self, data: &BiomeData) -> Vec<ObstacleKind> {
        let kinds = &data.obstacles;
        let ids: Vec<String> = kinds.iter().map(|k| k.id.clone()).collect();
        if !kinds.iter().any(|k| k.ground > 0.0) {
            self.error(
                "obstacles",
                "serve un tipo con peso sul terreno di base (ground > 0)",
            );
        }
        self.unique("obstacles", &ids);
        let weights = |pick: fn(&crate::data::schema::ObstacleKindData) -> f64| -> Vec<f64> {
            kinds.iter().map(pick).collect()
        };
        let ground = self.thresholds("obstacles.ground", &weights(|k| k.ground));
        // sulle macchie tutti i pesi a 0 vanno bene: radure senza ostacoli
        let patch = self.thresholds("obstacles.patch", &weights(|k| k.patch));
        (kinds.iter().enumerate())
            .map(|(i, k)| {
                let radius = k.footprint_radius;
                if !(MIN_FOOTPRINT_RADIUS..=MAX_FOOTPRINT_RADIUS).contains(&radius) {
                    self.error(
                        &format!("obstacles.{}.footprint_radius", k.id),
                        &format!(
                            "tra {MIN_FOOTPRINT_RADIUS} e {MAX_FOOTPRINT_RADIUS} px (varchi \
                             garantiti e passo dei corpi), trovato {radius}"
                        ),
                    );
                }
                ObstacleKind {
                    id: k.id.clone(),
                    sprite: k.sprite.clone(),
                    radius,
                    ground: ground[i],
                    patch: patch[i],
                }
            })
            .collect()
    }
}
