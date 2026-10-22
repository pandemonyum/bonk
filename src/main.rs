use bevy::prelude::AppExit;
use pixel_monster_survivor::{LaunchOptions, client_app};

fn main() -> AppExit {
    client_app(LaunchOptions::from_env()).run()
}
