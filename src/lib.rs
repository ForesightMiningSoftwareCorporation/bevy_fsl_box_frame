//! 3D box frame with pointer-based manipulation features.
//!
//! We say "frame" because only the 12 edges of the box are rendered via
//! `bevy_polyline`.
//!
//! Faces of the box can be dragged by the pointer to manipulate the box
//! extents. As the pointer hovers over each face, visual feedback is provided
//! (highlight material).
//!
//! Depends on [`bevy_mod_picking::plugins::DefaultPickingPlugins`] and the
//! `rapier` backend.

mod box_frame;
mod drag_face;
mod highlight;

use bevy::prelude::Plugin;
pub use box_frame::*;
use drag_face::*;
use highlight::*;

/// Enables pointer interactions for [`BoxFrame`] entities.
pub struct BoxFramePlugin;

impl Plugin for BoxFramePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems((highlight_face, drag_face));
    }
}
