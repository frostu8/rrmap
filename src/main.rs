use rrmap::editor::EditorCamera;
use rrmap::format::wad::Wad;
use rrmap::map::Map;

use bevy::prelude::*;

fn main() {
    let file = std::env::args()
        .nth(1)
        .expect("Pass wad file as first argument!");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(rrmap::EditorPlugins)
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        EditorCamera,
        // PickRaycastSource,
    ));
}
