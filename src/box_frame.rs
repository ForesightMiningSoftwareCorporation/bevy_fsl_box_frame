use crate::drag_face::Dragging;
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_polyline::prelude::{Polyline, PolylineBundle, PolylineMaterial};
use bevy_rapier3d::prelude::{Collider, Real};

/// The behavioral component of a box frame entity.
///
/// Users should not manually construct this type. Instead use
/// [`BoxFrame::build`].
#[derive(Component)]
pub struct BoxFrame {
    /// Material used when there are no pointers over the box.
    pub material: Handle<PolylineMaterial>,
    /// Material used for a face polyline when there is a pointer over it.
    pub highlight_material: Handle<PolylineMaterial>,

    pub(crate) extents: [Real; 6],
    pub(crate) min_extent: Real,
    pub(crate) face_entities: [Entity; 6],
    pub(crate) dragging_face: Option<Dragging>,
}

impl BoxFrame {
    /// Uses `commands` to build a box frame entity.
    ///
    /// `extents`: Determines the shape of the box. Each value is the
    /// corresponding face's distance from the center of the box. Indexed by
    /// [`FaceIndex`].
    ///
    /// `min_extent`: A lower bound for any components of `extents`. This
    /// prevents invalid configurations when dragging a face.
    ///
    /// `highlight_material` is used for edges of faces being highlighted,
    /// otherwise `material` is used.
    pub fn build(
        extents: [Real; 6],
        min_extent: Real,
        transform: Transform,
        material: Handle<PolylineMaterial>,
        highlight_material: Handle<PolylineMaterial>,
        polylines: &mut Assets<Polyline>,
        commands: &mut EntityCommands,
    ) {
        // All extents must be positive.
        let min_extent = min_extent.max(0.0);
        let extents = extents.map(|e| e.max(min_extent));

        let mut face_entities = [Entity::PLACEHOLDER; 6];
        commands
            .with_children(|builder| {
                for (i, face) in box_frame_polylines(extents).into_iter().enumerate() {
                    face_entities[i] = builder
                        .spawn(PolylineBundle {
                            polyline: polylines.add(face),
                            material: material.clone(),
                            ..default()
                        })
                        .id();
                }
            })
            .insert((
                Self {
                    extents,
                    min_extent,
                    face_entities,
                    material,
                    highlight_material,
                    dragging_face: default(),
                },
                SpatialBundle {
                    transform,
                    ..default()
                },
                box_frame_collider(extents),
            ));
    }

    pub(crate) fn update_extents(
        &mut self,
        new_extents: [Real; 6],
        collider: &mut Collider,
        line_handles: &mut Query<&mut Handle<Polyline>>,
        polylines: &mut Assets<Polyline>,
    ) {
        self.extents = new_extents;
        *collider = box_frame_collider(new_extents);
        let new_lines = box_frame_polylines(new_extents);
        for (face_entity, new_line) in self.face_entities.into_iter().zip(new_lines) {
            let Ok(mut line_handle) = line_handles.get_mut(face_entity) else {
                continue;
            };
            *line_handle = polylines.add(new_line);
        }
    }

    pub(crate) fn clear_highlights(&self, line_handles: &mut Query<&mut Handle<PolylineMaterial>>) {
        for face_entity in self.face_entities {
            if let Ok(mut line_handle) = line_handles.get_mut(face_entity) {
                *line_handle = self.material.clone();
            }
        }
    }

    pub(crate) fn highlight_face(
        &self,
        face: FaceIndex,
        line_handles: &mut Query<&mut Handle<PolylineMaterial>>,
    ) {
        // Highlight the picked face.
        if let Ok(mut line_handle) = line_handles.get_mut(self.face_entities[face]) {
            *line_handle = self.highlight_material.clone();
        }
    }
}

/// ```text
/// 0 = +X
/// 1 = -X
/// 2 = +Y
/// 3 = -Y
/// 4 = +Z
/// 5 = -Z
/// ```
pub type FaceIndex = usize;

pub const POS_X: FaceIndex = 0;
pub const NEG_X: FaceIndex = 1;
pub const POS_Y: FaceIndex = 2;
pub const NEG_Y: FaceIndex = 3;
pub const POS_Z: FaceIndex = 4;
pub const NEG_Z: FaceIndex = 5;

const FACE_NORMALS: [Vec3; 6] = [
    Vec3::X,
    Vec3::NEG_X,
    Vec3::Y,
    Vec3::NEG_Y,
    Vec3::Z,
    Vec3::NEG_Z,
];

/// Encoded as `0bZYX`.
type CornerIndex = usize;

/// Indexed by [`CornerIndex`].
const CUBE_CORNERS: [[FaceIndex; 3]; 8] = [
    [NEG_X, NEG_Y, NEG_Z],
    [POS_X, NEG_Y, NEG_Z],
    [NEG_X, POS_Y, NEG_Z],
    [POS_X, POS_Y, NEG_Z],
    [NEG_X, NEG_Y, POS_Z],
    [POS_X, NEG_Y, POS_Z],
    [NEG_X, POS_Y, POS_Z],
    [POS_X, POS_Y, POS_Z],
];

/// Indexed by [`FaceIndex`].
const FACE_QUADS: [[CornerIndex; 4]; 6] = [
    [0b001, 0b101, 0b111, 0b011], // +X
    [0b000, 0b010, 0b110, 0b100], // -X
    [0b010, 0b011, 0b111, 0b110], // +Y
    [0b000, 0b100, 0b101, 0b001], // -Y
    [0b100, 0b110, 0b111, 0b101], // +Z
    [0b000, 0b001, 0b011, 0b010], // -Z
];

fn box_frame_vertices([px, nx, py, ny, pz, nz]: [Real; 6]) -> [Vec3; 8] {
    let signed = [px, -nx, py, -ny, pz, -nz];
    CUBE_CORNERS.map(|[x, y, z]| Vec3::new(signed[x], signed[y], signed[z]))
}

fn box_frame_collider(extents: [Real; 6]) -> Collider {
    Collider::convex_hull(&box_frame_vertices(extents)).unwrap()
}

/// A polyline of 4 edges for each face.
fn box_frame_polylines(extents: [Real; 6]) -> [Polyline; 6] {
    let verts = box_frame_vertices(extents);
    [0, 1, 2, 3, 4, 5].map(|face| {
        let [i0, i1, i2, i3] = FACE_QUADS[face];
        Polyline {
            vertices: [i0, i1, i2, i3, i0].map(|corner| verts[corner]).to_vec(),
        }
    })
}

pub(crate) fn face_index_from_world_normal(
    world_normal: Vec3,
    transform: &GlobalTransform,
) -> FaceIndex {
    let (_, rot, _) = transform.to_scale_rotation_translation();
    let model_normal = rot.inverse() * world_normal;
    face_index_from_model_normal(model_normal)
}

pub(crate) fn face_index_from_model_normal(model_normal: Vec3) -> FaceIndex {
    // Choose the face whose normal maximizes the dot product with the input
    // normal.
    let mut max_prod = f32::NEG_INFINITY;
    let mut max_prod_face = 0;
    for (i, face_n) in FACE_NORMALS.into_iter().enumerate() {
        let prod = face_n.dot(model_normal);
        if prod > max_prod {
            max_prod = prod;
            max_prod_face = i;
        }
    }
    max_prod_face
}
