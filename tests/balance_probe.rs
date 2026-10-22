//! Sonda di bilanciamento (non un test di correttezza): stampa le statistiche di 60 s di
//! simulazione con autoplay per alcuni seed. Eseguire con
//! `cargo test --test balance_probe -- --ignored --nocapture`.

mod common;

use common::run_steps;
use pixel_monster_survivor::diagnostics::automation::autoplay_direction;
use pixel_monster_survivor::gameplay::combat::RunStats;
use pixel_monster_survivor::gameplay::player::MoveIntent;
use pixel_monster_survivor::headless_app;

#[test]
#[ignore = "sonda manuale di bilanciamento"]
fn print_hit_rate_over_one_minute() {
    for seed in [1, 2, 3, 4, 5] {
        for autoplay in [false, true] {
            let mut app = headless_app(seed);
            for step in 0..60 * 60 {
                if autoplay {
                    let t = step as f32 / 60.0;
                    app.insert_resource(MoveIntent(autoplay_direction(t)));
                }
                run_steps(&mut app, 1);
            }
            let s = app.world().resource::<RunStats>().clone();
            println!(
                "seed {seed} autoplay {autoplay:5}: sparati {:3} colpi {:3} ({:4.0}%) sconfitti {:3}",
                s.projectiles_fired,
                s.hits,
                100.0 * s.hits as f32 / s.projectiles_fired.max(1) as f32,
                s.defeated
            );
        }
    }
}
