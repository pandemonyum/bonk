# Licenze degli asset

Ogni asset distribuito con il gioco deve comparire qui: origine, autore, licenza, condizioni.
`python scripts/validate-assets.py` verifica che ogni PNG in `assets/` abbia una licenza nel
manifest `assets/manifest.json`.

## Asset originali del progetto

| Asset | Origine | Licenza |
|---|---|---|
| `assets/sprites/fireball.png` | `assets-src/sprites/fireball.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/rock.png` (20×16) | `assets-src/sprites/rock.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano (forme ombreggiate rifinite nella griglia) |
| `assets/sprites/tree.png` (40×48) | `assets-src/sprites/tree.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano (forme ombreggiate rifinite nella griglia) |
| `assets/sprites/xp_gem.png` | `assets-src/sprites/xp_gem.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/leaf.png` | `assets-src/sprites/leaf.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/droplet.png` | `assets-src/sprites/droplet.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/spark.png` (scintilla del fulmine di Tuonny, 6×6) | `assets-src/sprites/spark.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/telegraph.png` | `assets-src/sprites/telegraph.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/sprites/coin.png` (moneta a terra, foglio 4×1 da 10×10) | `assets-src/sprites/coin.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano nella griglia, senza IA né asset esterni |
| `assets/sprites/chest_sheet.png` (forziere, foglio 4×2 da 24×22) | `assets-src/sprites/chest_sheet.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano nella griglia, senza IA né asset esterni |
| `assets/sprites/blob_sheet.png` (Gelatina, foglio 4×2 da 28×24) | `assets-src/sprites/blob_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/swarm_sheet.png` (Sciame, 4×2 da 24×22) | `assets-src/sprites/swarm_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/armored_sheet.png` (Corazzato, 4×2 da 32×28) | `assets-src/sprites/armored_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/elite_sheet.png` (Gelatina Gigante, 4×4 da 44×38) | `assets-src/sprites/elite_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/sprites/boss_sheet.png` (Gran Corazzato, 4×4 da 96×80) | `assets-src/sprites/boss_sheet.px` con `enemies.palette.txt` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnato a mano per il progetto (forme ombreggiate rifinite nella griglia), senza IA né asset esterni |
| `assets/ui/icon_fire.png` | `assets-src/ui/icon_fire.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/icon_water.png` | `assets-src/ui/icon_water.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/icon_nature.png` | `assets-src/ui/icon_nature.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/heart_full.png` | `assets-src/ui/heart_full.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/heart_half.png` | `assets-src/ui/heart_half.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/heart_empty.png` | `assets-src/ui/heart_empty.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/skull.png` | `assets-src/ui/skull.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/clock.png` | `assets-src/ui/clock.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/lock.png` | `assets-src/ui/lock.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_damage.png` | `assets-src/ui/upgrade_ember_damage.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_rate.png` | `assets-src/ui/upgrade_ember_rate.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_multishot.png` | `assets-src/ui/upgrade_ember_multishot.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_pierce.png` | `assets-src/ui/upgrade_ember_pierce.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_range.png` | `assets-src/ui/upgrade_ember_range.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_ember_speed.png` | `assets-src/ui/upgrade_ember_speed.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_vitality.png` | `assets-src/ui/upgrade_vitality.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_regen.png` | `assets-src/ui/upgrade_regen.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_swift.png` | `assets-src/ui/upgrade_swift.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_magnet.png` | `assets-src/ui/upgrade_magnet.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_snack.png` | `assets-src/ui/upgrade_snack.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_join_tide.png` | `assets-src/ui/upgrade_join_tide.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_join_ember.png` | `assets-src/ui/upgrade_join_ember.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_join_sprout.png` | `assets-src/ui/upgrade_join_sprout.px` → `scripts/build-assets.py` | originale, proprietà del progetto (testina dei compagni di D-052 disegnata nel progetto) |
| `assets/ui/upgrade_join_spark.png` | `assets-src/ui/upgrade_join_spark.px` → `scripts/build-assets.py` | originale, proprietà del progetto (testina dei compagni di D-052 disegnata nel progetto) |
| `assets/ui/icon_electric.png` | `assets-src/ui/icon_electric.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/coin.png` (moneta dell'HUD e del prezzo dei forzieri, 16×16) | `assets-src/ui/coin.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/icon_chest.png` (indicatore del forziere fuori schermo, 16×16) | `assets-src/ui/icon_chest.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/upgrade_relic_courage.png` (carta reliquia Pietra del coraggio, 16×16) | `assets-src/ui/upgrade_relic_courage.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_hourglass.png` (carta reliquia Clessidra svelta, 16×16) | `assets-src/ui/upgrade_relic_hourglass.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_nest.png` (carta reliquia Nido di scintille, 16×16) | `assets-src/ui/upgrade_relic_nest.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_spyglass.png` (carta reliquia Cannocchiale di corteccia, 16×16) | `assets-src/ui/upgrade_relic_spyglass.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_oak.png` (carta reliquia Cuore di quercia, 16×16) | `assets-src/ui/upgrade_relic_oak.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_dew.png` (carta reliquia Rugiada perenne, 16×16) | `assets-src/ui/upgrade_relic_dew.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_sandals.png` (carta reliquia Sandali di vento, 16×16) | `assets-src/ui/upgrade_relic_sandals.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_piggy.png` (carta reliquia Salvadanaio di coccio, 16×16) | `assets-src/ui/upgrade_relic_piggy.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/upgrade_relic_feast.png` (carta reliquia Banchetto del bosco, 16×16) | `assets-src/ui/upgrade_relic_feast.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_a1.png` (talento Brace che scoppia, 16×16) | `assets-src/ui/talent_ember_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_a2.png` (talento Scoppio largo, 16×16) | `assets-src/ui/talent_ember_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_a3.png` (talento Brace che cova, 16×16) | `assets-src/ui/talent_ember_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_b1.png` (talento Soffio rovente, 16×16) | `assets-src/ui/talent_ember_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_b2.png` (talento Fiato lungo, 16×16) | `assets-src/ui/talent_ember_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_b3.png` (talento Soffio ardente, 16×16) | `assets-src/ui/talent_ember_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_a1.png` (talento Onda di ritorno, 16×16) | `assets-src/ui/talent_tide_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_a2.png` (talento Marea lenta, 16×16) | `assets-src/ui/talent_tide_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_a3.png` (talento Marea alta, 16×16) | `assets-src/ui/talent_tide_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_b1.png` (talento Acqua che ristora, 16×16) | `assets-src/ui/talent_tide_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_b2.png` (talento Getto dal profondo, 16×16) | `assets-src/ui/talent_tide_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_b3.png` (talento Sorgente viva, 16×16) | `assets-src/ui/talent_tide_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_a1.png` (talento Linfa urticante, 16×16) | `assets-src/ui/talent_sprout_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_a2.png` (talento Quarta foglia, 16×16) | `assets-src/ui/talent_sprout_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_a3.png` (talento Rovo largo, 16×16) | `assets-src/ui/talent_sprout_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_b1.png` (talento Radici che afferrano, 16×16) | `assets-src/ui/talent_sprout_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_b2.png` (talento Radici fitte, 16×16) | `assets-src/ui/talent_sprout_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_b3.png` (talento Radici spinose, 16×16) | `assets-src/ui/talent_sprout_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_a1.png` (talento Eco del lampo, 16×16) | `assets-src/ui/talent_spark_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_a2.png` (talento Catena salda, 16×16) | `assets-src/ui/talent_spark_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_a3.png` (talento Salto in piu', 16×16) | `assets-src/ui/talent_spark_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_b1.png` (talento Lampo dal cielo, 16×16) | `assets-src/ui/talent_spark_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_b2.png` (talento Cielo carico, 16×16) | `assets-src/ui/talent_spark_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_b3.png` (talento Lampo grande, 16×16) | `assets-src/ui/talent_spark_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/branch_ember_el.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_ember_el.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_a1.png` (talento v2 Brace che scoppia, 16×16) | `assets-src/ui/talent_ember_el_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_a2.png` (talento v2 Fuoco che cresce, 16×16) | `assets-src/ui/talent_ember_el_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_a3.png` (talento v2 Brace che cova, 16×16) | `assets-src/ui/talent_ember_el_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_b1.png` (talento v2 Soffio rovente, 16×16) | `assets-src/ui/talent_ember_el_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_b2.png` (talento v2 Tiro lungo, 16×16) | `assets-src/ui/talent_ember_el_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_b3.png` (talento v2 Brace viva, 16×16) | `assets-src/ui/talent_ember_el_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_c1.png` (talento v2 Brace e soffio, 16×16) | `assets-src/ui/talent_ember_el_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_el_c2.png` (talento v2 Soffio del vulcano, 16×16) | `assets-src/ui/talent_ember_el_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_ember_su.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_ember_su.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_a1.png` (talento v2 Calore amico, 16×16) | `assets-src/ui/talent_ember_su_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_a2.png` (talento v2 Fiamma nel petto, 16×16) | `assets-src/ui/talent_ember_su_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_a3.png` (talento v2 Lume, 16×16) | `assets-src/ui/talent_ember_su_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_b1.png` (talento v2 Luccichio, 16×16) | `assets-src/ui/talent_ember_su_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_b2.png` (talento v2 Fiuto per l'oro, 16×16) | `assets-src/ui/talent_ember_su_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_b3.png` (talento v2 Occhio d'oro, 16×16) | `assets-src/ui/talent_ember_su_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_c1.png` (talento v2 Focolare, 16×16) | `assets-src/ui/talent_ember_su_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_su_c2.png` (talento v2 Fiamma custode, 16×16) | `assets-src/ui/talent_ember_su_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_ember_ev.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_ember_ev.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_a1.png` (talento v2 Artiglio caldo, 16×16) | `assets-src/ui/talent_ember_ev_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_a2.png` (talento v2 Occhio di brace, 16×16) | `assets-src/ui/talent_ember_ev_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_a3.png` (talento v2 Fiato pronto, 16×16) | `assets-src/ui/talent_ember_ev_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_b1.png` (talento v2 Sguardo lontano, 16×16) | `assets-src/ui/talent_ember_ev_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_b2.png` (talento v2 Scaglie calde, 16×16) | `assets-src/ui/talent_ember_ev_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_b3.png` (talento v2 Cuore acceso, 16×16) | `assets-src/ui/talent_ember_ev_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_c1.png` (talento v2 Fiammetto adulto, 16×16) | `assets-src/ui/talent_ember_ev_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_ember_ev_c2.png` (talento v2 Pioggia di braci, 16×16) | `assets-src/ui/talent_ember_ev_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_tide_el.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_tide_el.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_a1.png` (talento v2 Getto dal profondo, 16×16) | `assets-src/ui/talent_tide_el_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_a2.png` (talento v2 Acqua pesante, 16×16) | `assets-src/ui/talent_tide_el_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_a3.png` (talento v2 Marea alta, 16×16) | `assets-src/ui/talent_tide_el_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_b1.png` (talento v2 Onda piena, 16×16) | `assets-src/ui/talent_tide_el_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_b2.png` (talento v2 Marea larga, 16×16) | `assets-src/ui/talent_tide_el_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_b3.png` (talento v2 Corrente, 16×16) | `assets-src/ui/talent_tide_el_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_c1.png` (talento v2 Onda e getto, 16×16) | `assets-src/ui/talent_tide_el_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_el_c2.png` (talento v2 Getto possente, 16×16) | `assets-src/ui/talent_tide_el_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_tide_su.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_tide_su.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_a1.png` (talento v2 Acqua che ristora, 16×16) | `assets-src/ui/talent_tide_su_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_a2.png` (talento v2 Sorsi d'acqua, 16×16) | `assets-src/ui/talent_tide_su_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_a3.png` (talento v2 Velo d'acqua, 16×16) | `assets-src/ui/talent_tide_su_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_b1.png` (talento v2 Bolle leggere, 16×16) | `assets-src/ui/talent_tide_su_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_b2.png` (talento v2 Pelle di rugiada, 16×16) | `assets-src/ui/talent_tide_su_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_b3.png` (talento v2 Acqua amica, 16×16) | `assets-src/ui/talent_tide_su_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_c1.png` (talento v2 Fonte calma, 16×16) | `assets-src/ui/talent_tide_su_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_su_c2.png` (talento v2 Sorgente viva, 16×16) | `assets-src/ui/talent_tide_su_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_tide_ev.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_tide_ev.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_a1.png` (talento v2 Acqua che trattiene, 16×16) | `assets-src/ui/talent_tide_ev_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_a2.png` (talento v2 Onda di ritorno, 16×16) | `assets-src/ui/talent_tide_ev_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_a3.png` (talento v2 Goccia dura, 16×16) | `assets-src/ui/talent_tide_ev_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_b1.png` (talento v2 Acqua gelida, 16×16) | `assets-src/ui/talent_tide_ev_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_b2.png` (talento v2 Risacca, 16×16) | `assets-src/ui/talent_tide_ev_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_b3.png` (talento v2 Corrente fredda, 16×16) | `assets-src/ui/talent_tide_ev_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_c1.png` (talento v2 Marea profonda, 16×16) | `assets-src/ui/talent_tide_ev_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_tide_ev_c2.png` (talento v2 Marea che avvolge, 16×16) | `assets-src/ui/talent_tide_ev_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_sprout_el.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_sprout_el.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_a1.png` (talento v2 Linfa urticante, 16×16) | `assets-src/ui/talent_sprout_el_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_a2.png` (talento v2 Foglia robusta, 16×16) | `assets-src/ui/talent_sprout_el_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_a3.png` (talento v2 Giro largo, 16×16) | `assets-src/ui/talent_sprout_el_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_b1.png` (talento v2 Radici che afferrano, 16×16) | `assets-src/ui/talent_sprout_el_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_b2.png` (talento v2 Vento tra le foglie, 16×16) | `assets-src/ui/talent_sprout_el_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_b3.png` (talento v2 Rami lunghi, 16×16) | `assets-src/ui/talent_sprout_el_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_c1.png` (talento v2 Rovo e radice, 16×16) | `assets-src/ui/talent_sprout_el_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_el_c2.png` (talento v2 Radici spinose, 16×16) | `assets-src/ui/talent_sprout_el_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_sprout_su.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_sprout_su.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_a1.png` (talento v2 Scorza di corteccia, 16×16) | `assets-src/ui/talent_sprout_su_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_a2.png` (talento v2 Linfa che nutre, 16×16) | `assets-src/ui/talent_sprout_su_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_a3.png` (talento v2 Radice salda, 16×16) | `assets-src/ui/talent_sprout_su_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_b1.png` (talento v2 Foglia larga, 16×16) | `assets-src/ui/talent_sprout_su_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_b2.png` (talento v2 Rami che raccolgono, 16×16) | `assets-src/ui/talent_sprout_su_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_b3.png` (talento v2 Seme di riserva, 16×16) | `assets-src/ui/talent_sprout_su_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_c1.png` (talento v2 Bosco fitto, 16×16) | `assets-src/ui/talent_sprout_su_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_su_c2.png` (talento v2 Corteccia viva, 16×16) | `assets-src/ui/talent_sprout_su_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_sprout_ev.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_sprout_ev.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_a1.png` (talento v2 Foglie svelte, 16×16) | `assets-src/ui/talent_sprout_ev_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_a2.png` (talento v2 Linfa forte, 16×16) | `assets-src/ui/talent_sprout_ev_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_a3.png` (talento v2 Nervature, 16×16) | `assets-src/ui/talent_sprout_ev_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_b1.png` (talento v2 Giro veloce, 16×16) | `assets-src/ui/talent_sprout_ev_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_b2.png` (talento v2 Orbita ampia, 16×16) | `assets-src/ui/talent_sprout_ev_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_b3.png` (talento v2 Nodo del ramo, 16×16) | `assets-src/ui/talent_sprout_ev_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_c1.png` (talento v2 Foglin cresciuto, 16×16) | `assets-src/ui/talent_sprout_ev_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_sprout_ev_c2.png` (talento v2 Quarta foglia, 16×16) | `assets-src/ui/talent_sprout_ev_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_spark_el.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_spark_el.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_a1.png` (talento v2 Eco del lampo, 16×16) | `assets-src/ui/talent_spark_el_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_a2.png` (talento v2 Lampo vivo, 16×16) | `assets-src/ui/talent_spark_el_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_a3.png` (talento v2 Coda di stelle, 16×16) | `assets-src/ui/talent_spark_el_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_b1.png` (talento v2 Lampo dal cielo, 16×16) | `assets-src/ui/talent_spark_el_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_b2.png` (talento v2 Aria tesa, 16×16) | `assets-src/ui/talent_spark_el_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_b3.png` (talento v2 Nube bassa, 16×16) | `assets-src/ui/talent_spark_el_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_c1.png` (talento v2 Lampo e catena, 16×16) | `assets-src/ui/talent_spark_el_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_el_c2.png` (talento v2 Temporale, 16×16) | `assets-src/ui/talent_spark_el_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_spark_su.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_spark_su.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_a1.png` (talento v2 Passo elettrico, 16×16) | `assets-src/ui/talent_spark_su_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_a2.png` (talento v2 Pelo che attira, 16×16) | `assets-src/ui/talent_spark_su_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_a3.png` (talento v2 Scossa amica, 16×16) | `assets-src/ui/talent_spark_su_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_b1.png` (talento v2 Monete attratte, 16×16) | `assets-src/ui/talent_spark_su_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_b2.png` (talento v2 Cuore che batte, 16×16) | `assets-src/ui/talent_spark_su_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_b3.png` (talento v2 Calamita viva, 16×16) | `assets-src/ui/talent_spark_su_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_c1.png` (talento v2 Corrente amica, 16×16) | `assets-src/ui/talent_spark_su_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_su_c2.png` (talento v2 Lampo nei piedi, 16×16) | `assets-src/ui/talent_spark_su_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/branch_spark_ev.png` (intestazione del ramo, talenti v2, 16×16) | `assets-src/ui/branch_spark_ev.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_a1.png` (talento v2 Catena salda, 16×16) | `assets-src/ui/talent_spark_ev_a1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_a2.png` (talento v2 Salto lungo, 16×16) | `assets-src/ui/talent_spark_ev_a2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_a3.png` (talento v2 Pelo carico, 16×16) | `assets-src/ui/talent_spark_ev_a3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_b1.png` (talento v2 Coda tesa, 16×16) | `assets-src/ui/talent_spark_ev_b1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_b2.png` (talento v2 Scintille lontane, 16×16) | `assets-src/ui/talent_spark_ev_b2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_b3.png` (talento v2 Pelo dritto, 16×16) | `assets-src/ui/talent_spark_ev_b3.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_c1.png` (talento v2 Tuonny cresciuto, 16×16) | `assets-src/ui/talent_spark_ev_c1.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/talent_spark_ev_c2.png` (talento v2 Salto in più, 16×16) | `assets-src/ui/talent_spark_ev_c2.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata nella griglia, senza IA né asset esterni |
| `assets/ui/menu_new_run.png` (bottone "Nuova partita" della home, spada d'oro, 16×16) | `assets-src/ui/menu_new_run.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/menu_companions.png` (bottone "Compagni", testa di creaturina generica, 16×16) | `assets-src/ui/menu_companions.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/menu_talents.png` (bottone "Talenti", libro aperto, 16×16) | `assets-src/ui/menu_talents.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/menu_options.png` (bottone "Opzioni", ingranaggio, 16×16) | `assets-src/ui/menu_options.px` → `scripts/build-assets.py` | originale, proprietà del progetto: abbozzata con una forma geometrica (otto denti su un anello) e rifinita nella griglia, senza IA né asset esterni |
| `assets/ui/menu_quit.png` (bottone "Esci", porta socchiusa, 16×16) | `assets-src/ui/menu_quit.px` → `scripts/build-assets.py` | originale, proprietà del progetto: disegnata a mano nella griglia, senza IA né asset esterni |
| `assets/ui/menu_arrow.png` (cursore del menu della home, 7×11) | `assets-src/ui/menu_arrow.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/frame_corner.png` (angolo della cornice del pannello della home, 10×10) | `assets-src/ui/frame_corner.px` → `scripts/build-assets.py` | originale, proprietà del progetto |
| `assets/ui/logo.png` (logo "Companion Survivors" della home, 222×90) | generato da `scripts/build-assets.py` (`build_logo` in `scripts/art_logo.py`): lettere dal font Pixelify Sans Regular (`assets/fonts/pixelify-sans/`, OFL 1.1) portate sulla sua griglia e ingrassate, con la sfumatura, i contorni e l'ombra del progetto; foglie, fiamma e stella disegnate a mano in `assets-src/logo/logo_parts.px` | originale, proprietà del progetto (lettering originale, nessun logo di altri giochi preso a modello; l'immagine dell'utente ha dato solo colori e disposizione). Il font non è incluso nel PNG: l'OFL copre il font, non le immagini disegnate con esso |
| Palette del logo (`assets-src/logo/logo.palette.txt`, 28 colori) | definita per il progetto: rampe calde e fredde delle scritte, foglie; fiamma e oro come la palette "Germoglio" | originale, proprietà del progetto |
| `assets/tiles/meadow_tileset.png` | generato da `scripts/build-assets.py` (seed 20261022, 42 tile dalla v1, ridisegnati nello stile del player il 2026-10-28) | originale, proprietà del progetto |
| Palette "Germoglio" v1 (`assets-src/palette.txt`, 56 colori) | definita per il progetto | originale, proprietà del progetto |
| Palette dei nemici (`assets-src/sprites/enemies.palette.txt`, 18 colori) | definita per il progetto: viola e rossi della palette "Germoglio", contorno come il player | originale, proprietà del progetto |

## Terze parti distribuite in `assets/`

| Asset | Autore | Licenza | Fonte | Note |
|---|---|---|---|---|
| `assets/fonts/tiny5/Tiny5.ttf` (variabile, istanza di default) | Stefan Schmidt, "The Tiny5 Project Authors" | SIL Open Font License 1.1, testo in `assets/fonts/tiny5/OFL.txt` | Google Fonts, `github.com/google/fonts` `ofl/tiny5/Tiny5[BLED,JITT,ROND,wdth,wght].ttf` (upstream `github.com/Gissio/font_tiny5`), scaricato il 2026-10-23, SHA-256 `588626f2…d8fa` | file rinominato, contenuto non modificato; HUD (D-044) |
| `assets/fonts/pixelify-sans/PixelifySans.ttf` (variabile, istanza di default) | Stefie Justprince, "The Pixelify Sans Project Authors" | SIL Open Font License 1.1, testo in `assets/fonts/pixelify-sans/OFL.txt` | Google Fonts, `github.com/google/fonts` `ofl/pixelifysans/PixelifySans[wght].ttf` (upstream `github.com/eifetx/Pixelify-Sans`), scaricato il 2026-10-23, SHA-256 `9ba86cd0…e8a5` | file rinominato, contenuto non modificato; menu, draft, risultati (D-044) |

Nessuno dei due dichiara un Reserved Font Name. L'OFL permette di distribuirli con il gioco
anche a pagamento, purché il file di licenza li accompagni e il font non sia venduto da solo.

## Terze parti incluse nel binario

| Asset | Autore | Licenza | Note |
|---|---|---|---|
| Font FiraMono (subset), incorporato da Bevy (`bevy_text`, feature `default_font`) | Mozilla / Carrois Apostrophe | SIL Open Font License 1.1 | resta solo nell'overlay di debug (F3); l'interfaccia usa Tiny5 e Pixelify Sans (D-044) |

Motore e librerie Rust: Bevy e dipendenze sono MIT/Apache-2.0; l'elenco completo si genera
con `cargo tree` (da affiancare a un report licenze prima della release, P3.5).

Stile v1 (2026-10-23): tutti gli sprite e le icone sopra sono disegnati a mano come griglie
`.px` (i più grandi abbozzati con forme geometriche e rifiniti a mano), senza generatori IA né
asset scaricati. L'immagine di riferimento dell'utente ha guidato solo lo stile: nessun pixel,
forma, personaggio o nome ne è stato copiato. Le anteprime `docs/evidence/art-preview-*.png`
sono composte da `scripts/preview-scene.py` con questi stessi asset (non vanno nel gioco).

## Asset forniti dall'utente

| Asset | Origine | Diritti e licenza |
|---|---|---|
| `assets/sprites/ember_sheet.png` (Fiammetto, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/ember_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_ember.png` (Fiammetto, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_ember.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/tide_sheet.png` (Ondino, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/tide_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_tide.png` (Ondino, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_tide.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/sprout_sheet.png` (Foglin, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/sprout_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_sprout.png` (Foglin, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_sprout.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/spark_sheet.png` (Tuonny, foglio 4×5 da 40×36) | disegno fornito dall'utente (foglio dei compagni, 6×6 pose), tagliato a risoluzione nativa e ripulito; animazioni ricavate dalle sue pose: `assets-src/sprites/spark_sheet.px` con `companions.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/ui/portrait_spark.png` (Tuonny, 40×36) | la sua posa di fronte senza ombra: `assets-src/ui/portrait_spark.px` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| Palette `assets-src/sprites/companions.palette.txt` (36 colori) | colori del foglio dell'utente, sfumature quasi uguali unite | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica (D-052) |
| `assets/sprites/summoner_sheet.png` (player, foglio 4×8 da 36×38) | foglio di animazioni generato e fornito dall'utente, tagliato a risoluzione nativa, poi ridisegnato e pulito nel progetto: `assets-src/sprites/summoner_sheet.px` con la palette propria `assets-src/sprites/summoner_sheet.palette.txt` → `scripts/build-assets.py` | l'utente dichiara di avere i diritti sul foglio e ne ha chiesto la riproduzione identica; il progetto l'ha ridisegnato e ripulito. Rischio di somiglianza con personaggi Nintendo accettato dall'utente (D-050). Nel manifest: `user-provided (rights declared by the user, D-050), redrawn and cleaned in assets-src` |

Cosa è stato fatto sul foglio dell'utente (2026-10-28, D-050):
- **Personaggio:** design, colori, proporzioni e dettagli restano quelli dell'utente.
- **Colori:** le sfumature quasi identiche del generatore sono unite in 24 colori tutti presi
  dal suo foglio, più l'ombra a terra del progetto.
- **Master:** il fermo 0 di fronte, di schiena e di profilo è il master. Schiena e profilo sono
  portati all'altezza del fronte togliendo righe uniformi, senza ricampionare i dettagli.
- **Fotogrammi:** fermo, cammino e colpito 0 e 3 derivano dai master. Colpito 1–2 e sconfitto
  0–3 sono le pose dell'utente ripulite (il colpito 1 ridotto all'altezza del master).

Il foglio originale e il ritaglio restano fuori dal repository.

Illustrazione della home (2026-10-30, D-063):

| Asset | Origine | Diritti e licenza |
|---|---|---|
| `assets/ui/home_background.png` (sfondo della home, 1448×1086) | immagine fornita dall'utente in chat, salvata come `assets-src/illustrations/home_background.webp`; `scripts/build-assets.py` la converte in PNG a piena risoluzione, a 256 colori con dithering se a schermo non si vede differenza (oggi 0,80 MiB contro 2,41 MiB in RGB). Nessun ritocco del contenuto | fornita dall'utente, che ne dichiara i diritti (D-063). Nel manifest: `provided by the user, rights declared by the user (D-063)`. Ritrae il player e tre compagni dell'utente (D-050, D-052): vale lo stesso rischio di somiglianza accettato dall'utente |

La versione attuale è **provvisoria**: ha ancora logo e menu dipinti sopra. La versione pulita,
nativa 16:9, è la richiesta ChatGPT ART-HOME-001 (D-065, oggi `REQUESTED`): approvata e
promossa, prende il posto del `.webp` con lo stesso nome e si registra nella sezione "Asset da
richieste ChatGPT".

Compagni piccoli (2026-10-28, approvati dall'utente, D-060): `assets-src/sprites/<id>_sheet.px`
ridisegna a mano a 28×24 le quattro pose approvate dei fogli da 40×36 (stessi personaggi
dell'utente, stessa palette `companions.palette.txt`, D-052) e ha preso il posto del foglio da
40×36, con la stessa voce di licenza qui sopra. I fogli da 40×36 restano nella storia di git.

## Asset da richieste ChatGPT (D-065)

Immagini che l'utente genera con ChatGPT da una richiesta `docs/art/requests/<ID>.md` e approva
(procedura: `docs/art/CHATGPT_ART_WORKFLOW.md`). Si registrano solo alla PROMOTE, una riga per
asset; le bozze non approvate non entrano nel repository e non si registrano.

| Asset | Richiesta e versione | Generata / approvata | Strumento | Reference usate | Cambiato nel progetto | Diritti |
|---|---|---|---|---|---|---|

Nessuna voce: nessuna richiesta è ancora promossa (ART-HOME-001 è `REQUESTED`).

Come si compila:
- **Asset:** il file in `assets/` e la sorgente in `assets-src/` (per uno sprite, il `.px`
  ridisegnato).
- **Richiesta e versione:** `<ID>` e la `v<N>` scelta.
- **Generata / approvata:** le due date; l'approvazione è la frase dell'utente nel registro della
  richiesta.
- **Strumento:** "ChatGPT, account dell'utente".
- **Reference usate:** i file del repo allegati (REF-n), anche se allegati come copia di `ref/`.
- **Cambiato nel progetto:** ritaglio, conversione del profilo colore, codifica del build,
  ridisegno nella griglia; "nessun ritocco del contenuto" se è così.
- **Diritti:** dichiarati dall'utente, che ha generato l'immagine con il suo account secondo i
  termini di OpenAI che ha accettato (D-065). Se ritrae il player o i compagni vale il rischio di
  somiglianza accettato dall'utente (D-050).
- **Manifest:** la stessa provenienza in breve e in inglese, nella prima riga di
  `<nome>.license.txt` (illustrazioni) o in `license:` del `.px` (sprite ridisegnati).

## Asset vietati

Nessuno sprite, suono, font, logo o UI estratto da Pokémon o da altri giochi. La concept
image in `references/` (se presente) è solo moodboard e non va ritagliata.
