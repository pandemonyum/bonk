//! Prove automatiche senza mani sulla tastiera: autoplay, screenshot programmati, uscita.
//! Es.: `cargo run -- --autoplay --screenshot screenshots/s0.png --screenshot-after 4 --exit-after 5`

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};

use crate::LaunchOptions;
use crate::gameplay::player::MoveIntent;

/// Se lo screenshot non arriva entro questo margine dopo `--exit-after`, si esce comunque.
const SCREENSHOT_GRACE_SECS: f32 = 10.0;

#[derive(Resource, Default)]
struct ScheduledShot {
    requested: bool,
    saved: bool,
}

pub struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScheduledShot>().add_systems(
            Update,
            (
                autoplay_intent.run_if(|o: Res<LaunchOptions>| o.autoplay),
                take_scheduled_screenshot,
                take_manual_screenshot,
                exit_when_done,
            ),
        );
    }
}

/// Direzione dell'autoplay al tempo `t`: una curva di Lissajous che attraversa l'arena
/// vicino al centro, così i nemici inseguono in modo visibile.
pub fn autoplay_direction(t: f32) -> Vec2 {
    Vec2::new((t * 0.9).cos(), (t * 1.3).sin()).normalize_or_zero()
}

fn autoplay_intent(time: Res<Time>, mut intent: ResMut<MoveIntent>) {
    intent.0 = autoplay_direction(time.elapsed_secs());
}

fn take_scheduled_screenshot(
    mut commands: Commands,
    time: Res<Time<Real>>,
    options: Res<LaunchOptions>,
    mut shot: ResMut<ScheduledShot>,
) {
    let Some(path) = &options.screenshot else {
        return;
    };
    if shot.requested || time.elapsed_secs() < options.screenshot_after_secs {
        return;
    }
    shot.requested = true;
    spawn_screenshot(&mut commands, path.clone())
        .observe(|_: On<ScreenshotCaptured>, mut shot: ResMut<ScheduledShot>| shot.saved = true);
}

fn take_manual_screenshot(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
) {
    if keys.just_pressed(KeyCode::F12) {
        let millis = time.elapsed().as_millis();
        spawn_screenshot(
            &mut commands,
            PathBuf::from(format!("screenshots/shot-{millis}.png")),
        );
    }
}

fn spawn_screenshot<'a>(commands: &'a mut Commands, path: PathBuf) -> EntityCommands<'a> {
    ensure_parent_dir(&path);
    info!("screenshot -> {}", path.display());
    let mut entity = commands.spawn(Screenshot::primary_window());
    entity.observe(save_to_disk(path));
    entity
}

fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
        && let Err(err) = std::fs::create_dir_all(parent)
    {
        warn!("impossibile creare {}: {err}", parent.display());
    }
}

fn exit_when_done(
    time: Res<Time<Real>>,
    options: Res<LaunchOptions>,
    shot: Res<ScheduledShot>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(limit) = options.exit_after_secs else {
        return;
    };
    let elapsed = time.elapsed_secs();
    let waiting_for_shot = options.screenshot.is_some() && !shot.saved;
    // Lo screenshot può essere programmato dopo `--exit-after`: si attende comunque.
    let give_up_at = limit.max(options.screenshot_after_secs) + SCREENSHOT_GRACE_SECS;
    if elapsed >= limit && (!waiting_for_shot || elapsed >= give_up_at) {
        if waiting_for_shot {
            warn!("uscita senza screenshot: non catturato entro {SCREENSHOT_GRACE_SECS} s");
        }
        info!("uscita automatica dopo {elapsed:.1} s");
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autoplay_moves_at_unit_speed_and_changes_direction() {
        let start = autoplay_direction(0.0);
        assert!((start.length() - 1.0).abs() < 1e-5);
        assert!((autoplay_direction(2.0) - start).length() > 0.5);
    }
}
