//! Main editor components and systems.

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_prototype_lyon::{draw::Stroke, entity::Path};

use crate::map::{self, Map};

/// The root editor component.
#[derive(Component)]
pub struct Editor {
    map: Map,
}

impl Editor {
    /// The map that the `Editor` contains.
    pub fn map(&self) -> &Map {
        &self.map
    }

    /// Gets the vertex at index `i`.
    pub fn vertex(&self, idx: usize) -> Option<&map::Vertex> {
        self.map.vertices.get(idx)
    }
}

/// Tag for the editor camera.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EditorCamera;

/// Represents a vertex.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vertex(pub usize);

/// A bundle for spawning a linedef entity.
#[derive(Bundle)]
pub struct LineDefBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
    pub path: Path,
    pub mesh_2d_handle: Mesh2dHandle,
    pub material_handle: Handle<ColorMaterial>,
    pub stroke: Stroke,
    pub line_def: LineDef,
}

impl LineDefBundle {
    pub fn new(idx: usize) -> LineDefBundle {
        LineDefBundle {
            transform: default(),
            global_transform: default(),
            visibility: default(),
            view_visibility: default(),
            inherited_visibility: default(),
            path: default(),
            mesh_2d_handle: default(),
            material_handle: default(),
            stroke: Stroke::new(Color::WHITE, 1.0),
            line_def: LineDef(idx),
        }
    }
}

/// Represents a linedef.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LineDef(pub usize);
