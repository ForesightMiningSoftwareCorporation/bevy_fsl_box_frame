use bevy::prelude::*;
use bevy_fsl_box_frame::{BoxFrame, BoxFramePlugin, BoxFrameVisuals, SolidColorMaterial};
use bevy_polyline::prelude::{Polyline, PolylineMaterial};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            bevy_polyline::PolylinePlugin,
            BoxFramePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut line_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SolidColorMaterial>>,
) {
    let visuals = BoxFrameVisuals::new_default(&mut line_materials, &mut meshes, &mut materials);

    // Rotate the box frame to test our surface normal calculations.
    let transform =
        Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_4));

    BoxFrame::build(
        [-0.5, -0.5, -0.5, 0.5, 0.5, 0.5],
        transform,
        PointerButton::Primary,
        visuals,
        &mut polylines,
        &mut commands.spawn(()),
    );

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
