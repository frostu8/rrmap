//! Ring Racers map and WAD tools.

pub mod editor;
pub mod format;
pub mod map;
pub mod ui;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

/// Editor plugins for [`bevy`].
pub struct EditorPlugins;

impl PluginGroup for EditorPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(bevy_egui::EguiPlugin)
            .add(ui::UiPlugin)
    }
}
