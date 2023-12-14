use crate::drag_face::Dragging;
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_polyline::prelude::{Polyline, PolylineBundle, PolylineMaterial};
use parry3d::bounding_volume::Aabb;

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

    pub(crate) dragging_face: Option<Dragging>,

    faces: [f32; 6],
    face_entities: [Entity; 6],
}

impl BoxFrame {
    /// Uses `commands` to build a box frame entity.
    ///
    /// `faces`: Coordinates of each face along it's normal axis. See [`FaceId`].
    ///
    /// `highlight_material` is used for edges of faces being highlighted,
    /// otherwise `material` is used.
    pub fn build(
        faces: [f32; 6],
        transform: Transform,
        material: Handle<PolylineMaterial>,
        highlight_material: Handle<PolylineMaterial>,
        polylines: &mut Assets<Polyline>,
        commands: &mut EntityCommands,
    ) {
        let faces = sorted_faces(faces);
        let mut face_entities = [Entity::PLACEHOLDER; 6];
        commands
            .with_children(|builder| {
                for (i, face) in face_polylines(faces).into_iter().enumerate() {
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
                    faces,
                    face_entities,
                    material,
                    highlight_material,
                    dragging_face: None,
                },
                SpatialBundle {
                    transform,
                    ..default()
                },
            ));
    }

    pub fn faces(&self) -> [f32; 6] {
        self.faces
    }

    pub fn aabb(&self) -> Aabb {
        aabb_from_faces(self.faces)
    }

    pub(crate) fn set_face_during_drag(&mut self, face: usize, coord: f32) {
        // NOTE: We aren't sorting the faces until the drag ends, because this
        // allows them to pass through each other.
        self.faces[face] = coord;
    }

    pub(crate) fn on_drag_end(
        &mut self,
        line_handles: &mut Query<&mut Handle<Polyline>>,
        polylines: &mut Assets<Polyline>,
    ) {
        self.dragging_face = None;
        // Sort faces so we can pick the correct face on the next picking event.
        self.faces = sorted_faces(self.faces);
        self.reset_lines(line_handles, polylines)
    }

    pub(crate) fn reset_lines(
        &self,
        line_handles: &mut Query<&mut Handle<Polyline>>,
        polylines: &mut Assets<Polyline>,
    ) {
        let new_lines = face_polylines(self.faces);
        for (face_entity, new_line) in self.face_entities.into_iter().zip(new_lines) {
            let Ok(mut line_handle) = line_handles.get_mut(face_entity) else {
                continue;
            };
            *line_handle = polylines.add(new_line);
        }
    }

    pub(crate) fn clear_highlights(
        &self,
        material_handles: &mut Query<&mut Handle<PolylineMaterial>>,
    ) {
        for face_entity in self.face_entities {
            if let Ok(mut line_handle) = material_handles.get_mut(face_entity) {
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
/// 0 = -X
/// 1 = -Y
/// 2 = -Z
/// 3 = +X
/// 4 = +Y
/// 5 = +Z
/// ```
pub type FaceIndex = usize;

pub const NEG_X: FaceIndex = 0;
pub const NEG_Y: FaceIndex = 1;
pub const NEG_Z: FaceIndex = 2;
pub const POS_X: FaceIndex = 3;
pub const POS_Y: FaceIndex = 4;
pub const POS_Z: FaceIndex = 5;

const FACE_NORMALS: [Vec3; 6] = [
    Vec3::NEG_X,
    Vec3::NEG_Y,
    Vec3::NEG_Z,
    Vec3::X,
    Vec3::Y,
    Vec3::Z,
];

pub(crate) fn face_sign(face: FaceIndex) -> f32 {
    if face < 3 {
        -1.0
    } else {
        1.0
    }
}

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
    [0b000, 0b010, 0b110, 0b100], // -X
    [0b000, 0b100, 0b101, 0b001], // -Y
    [0b000, 0b001, 0b011, 0b010], // -Z
    [0b001, 0b101, 0b111, 0b011], // +X
    [0b010, 0b011, 0b111, 0b110], // +Y
    [0b100, 0b110, 0b111, 0b101], // +Z
];

fn sorted_faces(faces: [f32; 6]) -> [f32; 6] {
    let [x1, y1, z1, x2, y2, z2] = faces;
    [
        x1.min(x2),
        y1.min(y2),
        z1.min(z2),
        x1.max(x2),
        y1.max(y2),
        z1.max(z2),
    ]
}

fn aabb_from_faces(faces: [f32; 6]) -> Aabb {
    let [x1, y1, z1, x2, y2, z2] = sorted_faces(faces);
    Aabb::new([x1, y1, z1].into(), [x2, y2, z2].into())
}

fn corner_vertices(faces: [f32; 6]) -> [Vec3; 8] {
    CUBE_CORNERS.map(|[x, y, z]| Vec3::new(faces[x], faces[y], faces[z]))
}

/// A polyline of 4 edges for each face.
fn face_polylines(faces: [f32; 6]) -> [Polyline; 6] {
    let verts = corner_vertices(faces);
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
