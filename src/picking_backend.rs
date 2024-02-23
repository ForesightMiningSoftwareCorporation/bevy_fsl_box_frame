use crate::{ray_map::RayMap, BoxFrame};
use bevy::{
    ecs::prelude::*,
    prelude::{Camera, GlobalTransform},
    render::view::RenderLayers,
};
use bevy_mod_picking::backend::{HitData, PointerHits};
use parry3d::{na::Isometry3, query::RayCast};

/// Generates pointer hits for the box frame's AABB and handles.
pub(crate) fn box_frame_backend(
    ray_map: Res<RayMap>,
    cameras: Query<(&Camera, Option<&RenderLayers>)>,
    box_frames: Query<(Entity, &BoxFrame, &GlobalTransform, Option<&RenderLayers>)>,
    transforms: Query<&GlobalTransform>,
    mut picking_out: EventWriter<PointerHits>,
) {
    for (&ray_id, &ray) in ray_map.map().iter() {
        let Ok((camera, view_mask)) = cameras.get(ray_id.camera) else {
            continue;
        };

        let cam_view_mask = view_mask.copied().unwrap_or_default();

        let ray = parry3d::query::Ray::new(ray.origin.into(), ray.direction.into());

        let mut picks = Vec::new();
        for (frame_entity, frame, frame_transform, frame_view_mask) in &box_frames {
            let frame_view_mask = frame_view_mask.copied().unwrap_or_default();
            if !frame_view_mask.intersects(&cam_view_mask) {
                continue;
            }

            let world_frame_center = frame_transform.transform_point(frame.center());
            // Check handle intersections first, they always take priority.
            let ball = frame.handle_ball();
            if let Some((toi, handle_entity, world_handle_center)) = frame
                .handle_entities()
                .into_iter()
                .filter_map(|handle_entity| {
                    let transform = transforms.get(handle_entity).ok()?;
                    let isometry = isometry_from_transform(transform);
                    ball.cast_ray(&isometry, &ray, f32::INFINITY, true)
                        .map(|toi| {
                            let world_handle_center = transform.translation();
                            (toi, handle_entity, world_handle_center)
                        })
                })
                .reduce(|t1, t2| if t1.0 < t2.0 { t1 } else { t2 })
            {
                let world_normal = (world_handle_center - world_frame_center).normalize();
                let intersect_p = ray.point_at(toi);
                // HACK: bevy_mod_picking seems to have a bug where equal depth
                // values alias and one hit gets dropped
                let fudge = 0.001;
                picks.push((
                    handle_entity,
                    HitData::new(ray_id.camera, toi - fudge, Some(intersect_p.into()), None),
                ));
                picks.push((
                    frame_entity,
                    HitData::new(
                        ray_id.camera,
                        toi,
                        Some(intersect_p.into()),
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
                    frame_entity,
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
