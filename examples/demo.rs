use bevy::prelude::*;
use bevy_fsl_box_frame::{BoxFrame, BoxFramePlugin};
use bevy_mod_picking::prelude::*;
use bevy_polyline::prelude::{Polyline, PolylineMaterial};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins,
            bevy_polyline::PolylinePlugin,
            BoxFramePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    let material = materials.add(PolylineMaterial {
        width: 1.0,
        ..default()
    });
    let highlight_material = materials.add(PolylineMaterial {
        width: 3.0,
        ..default()
    });

    // Rotate the box frame to test our surface normal calculations.
    let transform =
        Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4));

    BoxFrame::build(
        [-0.5, -0.5, -0.5, 0.5, 0.5, 0.5],
        transform,
        material,
        highlight_material,
        &mut polylines,
        &mut commands.spawn(()),
    );

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
