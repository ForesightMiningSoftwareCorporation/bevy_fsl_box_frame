use crate::{face_index_from_world_normal, BoxFrame};
use bevy::{prelude::*, utils::HashSet};
use bevy_mod_picking::{
    events::PointerEvent,
    prelude::{Move, Out, Over},
};
use bevy_polyline::prelude::PolylineMaterial;

pub(crate) fn highlight_face(
    mut dragging_entities: Local<HashSet<Entity>>,
    mut over_events: EventReader<PointerEvent<Over>>,
    mut move_events: EventReader<PointerEvent<Move>>,
    mut out_events: EventReader<PointerEvent<Out>>,
    box_frames: Query<(Entity, &BoxFrame, &GlobalTransform)>,
    mut line_handles: Query<&mut Handle<PolylineMaterial>>,
) {
    // Prioritize highlighting based on faces being dragged.
    dragging_entities.clear();
    for (entity, box_frame, _) in box_frames.iter() {
        if let Some(dragging) = &box_frame.dragging_face {
            box_frame.clear_highlights(&mut line_handles);
            box_frame.highlight_face(dragging.face(), &mut line_handles);
            dragging_entities.insert(entity);
        }
    }

    let normalized_over = over_events
        .iter()
        .map(|e| (e.target(), Some(e.event_data().pick_data)));
    let normalized_move = move_events
        .iter()
        .map(|e| (e.target(), Some(e.event_data().pick_data)));
    let normalized_out = out_events.iter().map(|e| (e.target(), None));

    // Highlight faces intersecting a pointer ray. "Out" events will clear all
    // highlights.
    for (target, maybe_pick_data) in normalized_over.chain(normalized_move).chain(normalized_out) {
        // Ignore events for entities that are already highlighted based on a
        // dragging face.
        if dragging_entities.contains(&target) {
            continue;
        }

        let Ok((_, box_frame, transform)) = box_frames.get(target)
            else { continue };

        box_frame.clear_highlights(&mut line_handles);
        if let Some(pick_data) = maybe_pick_data {
            if let Some(world_normal) = pick_data.normal {
                let picked_face = face_index_from_world_normal(world_normal, transform);
                box_frame.highlight_face(picked_face, &mut line_handles);
            }
        }
    }
}
