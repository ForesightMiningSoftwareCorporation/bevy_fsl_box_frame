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
    transforms: Query<&GlobalTransform>,
    mut picking_out: EventWriter<PointerHits>,
) {
    for (&ray_id, &ray) in ray_map.map().iter() {
        let Ok(camera) = cameras.get(ray_id.camera) else {
            continue;
        };

        let ray = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());

        let mut picks = Vec::new();
        for (entity, frame, frame_transform) in &box_frames {
            let world_frame_center = frame_transform.transform_point(frame.center());
            // Check handle intersections first, they always take priority.
            let ball = frame.handle_ball();
            if let Some((toi, world_handle_center)) = frame
                .handle_entities()
                .into_iter()
                .filter_map(|handle_entity| {
                    let transform = transforms.get(handle_entity).ok()?;
                    let isometry = isometry_from_transform(transform);
                    ball.cast_ray(&isometry, &ray, f32::INFINITY, true)
                        .map(|toi| {
                            let world_handle_center = transform.translation();
                            (toi, world_handle_center)
                        })
                })
                .reduce(|t1, t2| if t1.0 < t2.0 { t1 } else { t2 })
            {
                let world_normal = (world_handle_center - world_frame_center).normalize();
                picks.push((
                    entity,
                    HitData::new(
                        ray_id.camera,
                        toi,
                        Some(ray.point_at(toi).into()),
                        Some(world_normal),
                    ),
                ));
                continue;
            }

            // No handle intersections, check for AABB intersection.
            let isometry = isometry_from_transform(frame_transform);
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

fn isometry_from_transform(tfm: &GlobalTransform) -> Isometry3<f32> {
    let (_scale, rot, trans) = tfm.to_scale_rotation_translation();
    Isometry3::from_parts(trans.into(), rot.into())
}
