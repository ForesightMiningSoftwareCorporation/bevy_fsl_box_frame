use crate::{face_index_from_world_normal, BoxFrame, FaceIndex};
use approx::relative_eq;
use bevy::prelude::*;
use bevy_mod_picking::{
    events::PointerEvent,
    pointer::Location,
    prelude::{DragEnd, DragStart, PointerId, PointerLocation, PointerMap, RapierPickSource},
};
use bevy_polyline::prelude::Polyline;
use bevy_rapier3d::prelude::{Collider, Real};

// This data is constant while dragging is occurring.
pub(crate) struct Dragging {
    // The pointer doing the drag.
    pointer_id: PointerId,
    // The face being dragged.
    face: FaceIndex,
    // The face's extent at time of DragStart.
    initial_extent: Real,
    // The ray along which the face is translated during dragging. In world
    // coordinates.
    drag_ray: Ray,
}

impl Dragging {
    pub fn face(&self) -> FaceIndex {
        self.face
    }
}

pub(crate) fn drag_face(
    mut drag_start_events: EventReader<PointerEvent<DragStart>>,
    mut drag_end_events: EventReader<PointerEvent<DragEnd>>,
    pointer_map: Res<PointerMap>,
    picking_cameras: Query<(&Camera, &GlobalTransform), With<RapierPickSource>>,
    pointer_locations: Query<&PointerLocation>,
    mut box_frames: Query<(&mut BoxFrame, &mut Collider, &GlobalTransform)>,
    mut line_handles: Query<&mut Handle<Polyline>>,
    mut polylines: ResMut<Assets<Polyline>>,
) {
    // TODO bevy_mod_picking: For now we just assume this was the camera used to
    // generate all picking rays, but it's possible that we have multiple
    // cameras. We need more information from the events to know which camera
    // the backend used to generate the ray.
    let Ok((camera, camera_transform)) = picking_cameras.get_single() else { return };

    // Start or stop the dragging state machine based on events.
    for drag_start in drag_start_events.iter() {
        let Ok((mut frame, _, transform)) = box_frames.get_mut(drag_start.target())
            else { continue };
        let pick_data = drag_start.event_data().pick_data;
        let (Some(world_position), Some(world_normal)) = (pick_data.position, pick_data.normal)
            else { continue };
        let face = face_index_from_world_normal(world_normal, transform);
        frame.dragging_face = Some(Dragging {
            pointer_id: drag_start.pointer_id(),
            face,
            initial_extent: frame.extents[face],
            drag_ray: Ray {
                origin: world_position,
                direction: world_normal,
            },
        });
    }
    for drag_end in drag_end_events.iter() {
        let Ok(mut frame) = box_frames.get_component_mut::<BoxFrame>(drag_end.target())
            else { continue };
        frame.dragging_face = None;
    }

    // For all frames currently in the "dragging" state, we need to calculate
    // the new desired position of the face being dragged and update the box
    // frame to reflect that.
    for (mut frame, mut collider, _) in box_frames.iter_mut() {
        let Some(Dragging { pointer_id, face, initial_extent, drag_ray }) = &frame.dragging_face
            else { continue };

        let Some(pointer_ray) = get_pointer_ray(
            *pointer_id,
            &pointer_map,
            &pointer_locations,
            camera,
            camera_transform,
        ) else { continue };

        // Determine the new frame extents based on the desired position of the
        // dragging face.
        let Some((drag_delta, _)) = closest_points_on_two_rays(drag_ray, &pointer_ray)
            else { continue };
        let mut new_extents = frame.extents;
        new_extents[*face] = (initial_extent + drag_delta).max(frame.min_extent);

        frame.update_extents(
            new_extents,
            &mut collider,
            &mut line_handles,
            &mut polylines,
        )
    }
}

// TODO bevy_mod_picking: it should be much easier to construct a pointer ray
// than this.
fn get_pointer_ray(
    pointer_id: PointerId,
    pointer_map: &Res<PointerMap>,
    pointer_locations: &Query<&PointerLocation>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray> {
    let pointer_entity = pointer_map.get_entity(pointer_id)?;
    let Ok(PointerLocation {
        location: Some(Location { position, .. }),
    }) = pointer_locations.get(pointer_entity)
        else { return None };
    let (view_min, _) = camera.logical_viewport_rect()?;
    let view_position = *position - view_min;
    camera.viewport_to_world(camera_transform, view_position)
}

/// Find the closest pair of points `(p1, p2)` where `p1` is on ray `r1` and
/// `p2` is on ray `r2`. Returns `(t1, t2)` such that `p_n =
/// r_n.get_point(t_n)`.
fn closest_points_on_two_rays(r1: &Ray, r2: &Ray) -> Option<(Real, Real)> {
    // If the rays are parallel, then there are infinitely many solutions.
    if vectors_are_parallel(r1.direction, r2.direction) {
        return None;
    }

    // The key insight is that the vector between the two points must be
    // perpendicular to both rays. So we end up with this linear system:
    //
    // t1 * V1 - t2 * V2 + t3 * (V1 x V2) = P2 - P1
    let col1 = r1.direction;
    let col2 = -r2.direction;
    let col3 = r1.direction.cross(r2.direction);
    let rhs = r2.origin - r1.origin;
    let mat = Mat3::from_cols(col1, col2, col3);
    let t = mat.inverse() * rhs;

    Some((t.x, t.y))
}

fn vectors_are_parallel(v1: Vec3, v2: Vec3) -> bool {
    relative_eq!(v1.angle_between(v2), 0.0)
}
