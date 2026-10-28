//! Test d'integrazione headless in un solo eseguibile: ogni file in `tests/` diventerebbe un
//! eseguibile a sé con dentro tutta Bevy (~0,75 GB di simboli ciascuno, vedi D-027).
//!
//! Esecuzione: `cargo test --test sim`, oppure `cargo test --test sim combat` per un modulo.

mod balance;
mod bot;
mod chain;
mod combat;
mod common;
mod damage;
mod formulas;
mod movement;
mod party;
mod patterns;
mod progression;
mod recap;
mod run;
mod spawning;
mod threats;
mod upgrades;
mod world;
