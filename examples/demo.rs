use bevy::prelude::*;
use bevy_fsc_box_frame::{BoxFrame, BoxFramePlugin};
use bevy_mod_picking::prelude::*;
use bevy_polyline::prelude::{Polyline, PolylineMaterial};
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_polyline::PolylinePlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(BoxFramePlugin)
        .add_startup_system(setup)
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

    let extents = [0.5; 6];
    let min_extent = 0.05;
    BoxFrame::build(
        extents,
        min_extent,
        transform,
        material,
        highlight_material,
        &mut polylines,
        &mut commands.spawn(()),
    );

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RapierPickSource::default(),
    ));
}
