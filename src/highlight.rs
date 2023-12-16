use crate::{face_index_from_world_normal, BoxFrame, BoxFrameHandle};
use bevy::prelude::*;
use bevy_mod_picking::{
    events::Pointer,
    prelude::{DragEnd, Move, Out, Over},
};
use bevy_polyline::prelude::PolylineMaterial;

pub(crate) fn highlight_face(
    mut over_events: EventReader<Pointer<Over>>,
    mut move_events: EventReader<Pointer<Move>>,
    mut out_events: EventReader<Pointer<Out>>,
    mut drag_end_events: EventReader<Pointer<DragEnd>>,
    box_frames: Query<(&BoxFrame, &GlobalTransform)>,
    mut line_handles: Query<&mut Handle<PolylineMaterial>>,
) {
    // Prioritize highlighting based on faces being dragged.
    for (frame, _) in &box_frames {
        if let Some(dragging) = &frame.dragging_face {
            frame.clear_highlights(&mut line_handles);
            frame.highlight_face(dragging.face(), &mut line_handles);
        }
    }

    let normalized_over = over_events
        .iter()
        .map(|e| (e.target, Some(e.event.hit.clone())));
    let normalized_move = move_events
        .iter()
        .map(|e| (e.target, Some(e.event.hit.clone())));
    let normalized_out = out_events.iter().map(|e| (e.target, None));
    let normalized_drag_end = drag_end_events.iter().map(|e| (e.target, None));

    // Highlight faces intersecting a pointer ray. "Out" events will clear all
    // highlights.
    for (target, maybe_pick_data) in normalized_over
        .chain(normalized_move)
        .chain(normalized_out)
        .chain(normalized_drag_end)
    {
        let Ok((frame, transform)) = box_frames.get(target) else {
            continue;
        };

        // Ignore events for entities that are already highlighted based on a
        // dragging face.
        if frame.dragging_face.is_some() {
            continue;
        }

        frame.clear_highlights(&mut line_handles);
        if let Some(pick_data) = maybe_pick_data {
            if let Some(world_normal) = pick_data.normal {
                let picked_face = face_index_from_world_normal(world_normal, transform);
                frame.highlight_face(picked_face, &mut line_handles);
            }
        }
    }
}

pub(crate) fn highlight_handles(
    mut over_events: EventReader<Pointer<Over>>,
    mut out_events: EventReader<Pointer<Out>>,
    mut handles: Query<(&BoxFrameHandle, &mut Transform)>,
) {
    for over in over_events.iter() {
        let Ok((handle, mut tfm)) = handles.get_mut(over.target) else {
            continue;
        };
        tfm.scale = Vec3::splat(handle.base_scale * handle.hover_scale);
    }
    for out in out_events.iter() {
        let Ok((handle, mut tfm)) = handles.get_mut(out.target) else {
            continue;
        };
        tfm.scale = Vec3::splat(handle.base_scale);
    }
}
