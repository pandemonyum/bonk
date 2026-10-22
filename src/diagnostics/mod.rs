//! Diagnostica (solo client): overlay F3, screenshot, autoplay e uscita automatica.

pub mod automation;
pub mod overlay;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            overlay::OverlayPlugin,
            automation::AutomationPlugin,
        ));
    }
}
