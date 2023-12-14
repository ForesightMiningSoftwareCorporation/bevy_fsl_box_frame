use crate::{ray_map::RayMap, BoxFrame};
use bevy::{
    ecs::prelude::*,
    prelude::{Camera, GlobalTransform},
};
use bevy_mod_picking::backend::{HitData, PointerHits};
use parry3d::{na::Isometry3, query::RayCast};

/// Generates pointer hits for the box frame's AABB.
pub(crate) fn box_frame_backend(
    ray_map: Res<RayMap>,
    cameras: Query<&Camera>,
    box_frames: Query<(Entity, &BoxFrame, &GlobalTransform)>,
    mut picking_out: EventWriter<PointerHits>,
) {
    for (&ray_id, &ray) in ray_map.map().iter() {
        let Ok(camera) = cameras.get(ray_id.camera) else {
            continue;
        };

        let mut picks = Vec::new();
        for (entity, frame, transform) in &box_frames {
            let (_scale, rot, trans) = transform.to_scale_rotation_translation();
            let isometry = Isometry3::from_parts(trans.into(), rot.into());
            let ray = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());

            if let Some(hit) =
                frame
                    .aabb()
                    .cast_ray_and_get_normal(&isometry, &ray, f32::INFINITY, true)
            {
                picks.push((
                    entity,
                    HitData::new(
                        ray_id.camera,
                        hit.toi,
                        Some(ray.point_at(hit.toi).into()),
                        Some(hit.normal.into()),
                    ),
                ));
            }
        }

        if !picks.is_empty() {
            picking_out.send(PointerHits {
                pointer: ray_id.pointer,
                picks,
                order: camera.order as f32,
            });
        }
    }
}
