//! Prove automatiche senza mani sulla tastiera: autoplay, screenshot programmati, uscita.
//! Es.: `cargo run -- --autoplay --screenshot screenshots/s0.png --screenshot-after 4 --exit-after 5`

use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};

use crate::LaunchOptions;
use crate::gameplay::player::MoveIntent;

/// Se gli screenshot non arrivano entro questo margine, si esce comunque.
const SCREENSHOT_GRACE_SECS: f32 = 10.0;
/// Distanza tra due screenshot di una raffica (`--screenshot-burst`).
pub const BURST_INTERVAL_SECS: f32 = 0.1;

#[derive(Resource, Default)]
struct ScheduledShots {
    requested: u32,
    saved: u32,
}

pub struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScheduledShots>().add_systems(
            Update,
            (
                autoplay_intent.run_if(|o: Res<LaunchOptions>| o.autoplay),
                take_scheduled_screenshots,
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

/// File dello screenshot `index` di una raffica di `count`: `s.png` → `s-0.png`, `s-1.png`, ...
/// Con un solo screenshot il percorso resta quello chiesto.
pub fn burst_path(path: &Path, index: u32, count: u32) -> PathBuf {
    if count <= 1 {
        return path.to_path_buf();
    }
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let name = match path.extension() {
        Some(ext) => format!("{stem}-{index}.{}", ext.to_string_lossy()),
        None => format!("{stem}-{index}"),
    };
    path.with_file_name(name)
}

fn autoplay_intent(time: Res<Time>, mut intent: ResMut<MoveIntent>) {
    intent.0 = autoplay_direction(time.elapsed_secs());
}

fn take_scheduled_screenshots(
    mut commands: Commands,
    time: Res<Time<Real>>,
    options: Res<LaunchOptions>,
    mut shots: ResMut<ScheduledShots>,
) {
    let Some(path) = &options.screenshot else {
        return;
    };
    let index = shots.requested;
    let due = options.screenshot_after_secs + index as f32 * BURST_INTERVAL_SECS;
    if index >= options.screenshot_burst || time.elapsed_secs() < due {
        return;
    }
    shots.requested += 1;
    spawn_screenshot(
        &mut commands,
        burst_path(path, index, options.screenshot_burst),
    )
    .observe(|_: On<ScreenshotCaptured>, mut shots: ResMut<ScheduledShots>| shots.saved += 1);
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
    shots: Res<ScheduledShots>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(limit) = options.exit_after_secs else {
        return;
    };
    let elapsed = time.elapsed_secs();
    let waiting_for_shots = options.screenshot.is_some() && shots.saved < options.screenshot_burst;
    // Gli screenshot possono essere programmati dopo `--exit-after`: si attendono comunque.
    let last_shot_at = options.screenshot_after_secs
        + options.screenshot_burst.saturating_sub(1) as f32 * BURST_INTERVAL_SECS;
    let give_up_at = limit.max(last_shot_at) + SCREENSHOT_GRACE_SECS;
    if elapsed >= limit && (!waiting_for_shots || elapsed >= give_up_at) {
        if waiting_for_shots {
            warn!(
                "uscita con {}/{} screenshot salvati dopo {SCREENSHOT_GRACE_SECS} s di attesa",
                shots.saved, options.screenshot_burst
            );
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

    #[test]
    fn burst_paths_get_an_index_suffix() {
        let path = Path::new("screenshots/p02.png");
        assert_eq!(burst_path(path, 0, 1), PathBuf::from("screenshots/p02.png"));
        assert_eq!(
            burst_path(path, 0, 3),
            PathBuf::from("screenshots/p02-0.png")
        );
        assert_eq!(
            burst_path(path, 2, 3),
            PathBuf::from("screenshots/p02-2.png")
        );
        assert_eq!(burst_path(Path::new("shot"), 1, 2), PathBuf::from("shot-1"));
    }
}
