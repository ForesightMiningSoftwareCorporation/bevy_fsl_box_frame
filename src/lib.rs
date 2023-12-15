//! 3D box frame with pointer-based manipulation features.
//!
//! We say "frame" because only the 12 edges of the box are rendered via
//! `bevy_polyline`.
//!
//! Faces of the box can be dragged by the pointer to manipulate the box
//! extents. As the pointer hovers over each face, visual feedback is provided
//! (highlight material).
//!
//! Depends on [`bevy_mod_picking::DefaultPickingPlugins`] and
//! [`bevy_polyline::PolylinePlugin`].

mod box_frame;
mod drag_face;
mod handle_visibility;
mod highlight;
mod picking_backend;

// TODO: remove this for bevy_mod_picking 0.17.1
mod ray_map;

pub use box_frame::*;

use bevy::prelude::{IntoSystemConfigs, Plugin, PreUpdate, Update};
use bevy_mod_picking::picking_core::PickSet;
use drag_face::*;
use handle_visibility::*;
use highlight::*;
use picking_backend::box_frame_backend;
use ray_map::RayMap;

/// Enables pointer interactions for [`BoxFrame`] entities.
pub struct BoxFramePlugin;

impl Plugin for BoxFramePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<RayMap>()
            .add_systems(PreUpdate, RayMap::repopulate.in_set(PickSet::ProcessInput))
            .add_systems(PreUpdate, box_frame_backend.in_set(PickSet::Backend))
            .add_systems(Update, handle_visibility)
            // Correct highlighting updates depend on the state of dragging.
            .add_systems(Update, (drag_face, highlight_face).chain());
    }
}
