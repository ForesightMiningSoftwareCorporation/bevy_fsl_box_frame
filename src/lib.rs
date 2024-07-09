#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

mod box_frame;
mod drag_face;
mod handle_visibility;
mod highlight;
mod picking_backend;
mod solid_color_material;

pub use box_frame::*;
pub use solid_color_material::*;

use bevy::{
    asset::load_internal_asset,
    prelude::{IntoSystemConfigs, MaterialPlugin, Plugin, PreUpdate, Shader, Update},
};
use bevy_mod_picking::picking_core::PickSet;
use drag_face::*;
use handle_visibility::*;
use highlight::*;
use picking_backend::box_frame_backend;

/// Enables pointer interactions for [`BoxFrame`] entities.
pub struct BoxFramePlugin;

impl Plugin for BoxFramePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            SHADER_HANDLE,
            "shaders/solid_color.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<SolidColorMaterial>::default())
            .add_systems(PreUpdate, box_frame_backend.in_set(PickSet::Backend))
            .add_systems(Update, (handle_visibility, highlight_handles))
            // Correct highlighting updates depend on the state of dragging.
            .add_systems(Update, (drag_face, highlight_face).chain());
    }
}
