use crate::{drag_face::Dragging, solid_color_material::SolidColorMaterial};
use bevy::{color::palettes::css::RED, ecs::system::EntityCommands, math::FloatOrd, prelude::*};
use bevy_mod_picking::prelude::{Pickable, PointerButton};
use bevy_polyline::prelude::{Polyline, PolylineBundle, PolylineMaterial};
use parry3d::{bounding_volume::Aabb, shape::Ball};

/// The behavioral component of a box frame entity.
///
/// Users should not manually construct this type. Instead use
/// [`BoxFrame::build`].
#[derive(Component)]
pub struct BoxFrame {
    /// The button that triggers face dragging.
    pub drag_button: PointerButton,
    /// Assets and configuration for how the gizmo is rendered.
    pub visuals: BoxFrameVisuals,

    pub(crate) dragging_face: Option<Dragging>,

    faces: [f32; 6],
    face_entities: [Entity; 6],
    handle_entities: [Entity; 6],
}

/// Assets and configuration for how the gizmo is rendered.
#[derive(Clone)]
pub struct BoxFrameVisuals {
    /// Material used for frame edges.
    pub edge_material: Handle<PolylineMaterial>,
    /// Material used for highlighting frame handles.
    pub edge_highlight_material: Handle<PolylineMaterial>,
    /// Mesh used to render a face handle.
    pub handle_mesh: Handle<Mesh>,
    /// Material used to render a face handle.
    pub handle_material: Handle<SolidColorMaterial>,
    /// The scaling factor applied to a handle's [`Transform`] when there is no
    /// pointer hovering over it.
    pub handle_scale: ScalingFn,
    /// An extra scaling factor applied to a handle's [`Transform`] when there
    /// is a pointer hovering over it.
    ///
    /// For example, a factor of `2.0` would cause the handle to appear twice
    /// the size when hovering over it.
    pub handle_hover_scale: f32,
}

/// Given the box frame's current extents, returns the desired scaling factor of
/// the handle's [`Transform`].
///
/// The scaling factor should be considered equivalent to the perceived radius
/// of the default handle mesh (a sphere). Note that the picking intersection
/// test uses a unit-radius ball scaled by this function, so if you are using
/// a custom mesh, you must account for its perceived radius, and picking might
/// not work as well for non-spherical meshes.
pub type ScalingFn = fn([f32; 3]) -> f32;

#[derive(Component)]
pub(crate) struct BoxFrameHandle {
    pub base_scale: f32,
    pub hover_scale: f32,
}

impl BoxFrameVisuals {
    /// Creates default assets for rendering a box frame.
    ///
    /// This can be replaced by user-specified assets.
    pub fn new_default(
        line_materials: &mut Assets<PolylineMaterial>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<SolidColorMaterial>,
    ) -> Self {
        Self {
            edge_material: line_materials.add(PolylineMaterial {
                width: 1.0,
                ..default()
            }),
            edge_highlight_material: line_materials.add(PolylineMaterial {
                width: 3.0,
                ..default()
            }),

            handle_mesh: meshes.add(Sphere::new(1.0).mesh()),
            handle_material: materials.add(RED),
            handle_scale: |e| 0.05 * median3(e),
            handle_hover_scale: 1.2,
        }
    }
}

impl BoxFrame {
    /// Uses `commands` to build a box frame entity.
    ///
    /// `faces`: Coordinates of each face along it's normal axis. See
    /// [`FaceIndex`].
    pub fn build(
        faces: [f32; 6],
        transform: Transform,
        drag_button: PointerButton,
        visuals: BoxFrameVisuals,
        polylines: &mut Assets<Polyline>,
        commands: &mut EntityCommands,
    ) {
        let faces = sorted_faces(faces);
        let extents = box_extents(faces);
        let base_scale = (visuals.handle_scale)(extents);
        let mut face_entities = [Entity::PLACEHOLDER; 6];
        let mut handle_entities = [Entity::PLACEHOLDER; 6];
        commands
            .with_children(|builder| {
                for (face, entity) in face_polylines(faces).into_iter().zip(&mut face_entities) {
                    *entity = builder
                        .spawn(PolylineBundle {
                            polyline: polylines.add(face),
                            material: visuals.edge_material.clone(),
                            ..default()
                        })
                        .id();
                }
                for (handle_center, entity) in
                    face_centers(faces).into_iter().zip(&mut handle_entities)
                {
                    *entity = builder
                        .spawn(MaterialMeshBundle {
                            mesh: visuals.handle_mesh.clone(),
                            material: visuals.handle_material.clone(),
                            transform: Transform::default()
                                .with_translation(handle_center)
                                .with_scale(Vec3::splat((visuals.handle_scale)(extents))),
                            visibility: Visibility::Hidden,
                            ..default()
                        })
                        .insert((
                            BoxFrameHandle {
                                base_scale,
                                hover_scale: visuals.handle_hover_scale,
                            },
                            Pickable {
                                should_block_lower: false,
                                is_hoverable: true,
                            },
                        ))
                        .id();
                }
            })
            .insert((
                Self {
                    faces,
                    face_entities,
                    handle_entities,
                    drag_button,
                    visuals,
                    dragging_face: None,
                },
                SpatialBundle {
                    transform,
                    ..default()
                },
                Pickable {
                    should_block_lower: false,
                    is_hoverable: true,
                },
            ));
    }

    /// The coordinates of each face. See [`FaceIndex`].
    ///
    /// WARNING: While dragging a face, the minimum and maximum values along one
    /// axis may swap places in order to stay consistent with the face indices
    /// used at the start of the drag. This is intentional, but if you don't
    /// want this, use `self.sorted_faces()`.
    pub fn faces(&self) -> [f32; 6] {
        self.faces
    }

    /// Same as `self.faces()`, but the relative order of minimum and
    /// maximum values along each axis is always `min < max`, as specified by
    /// [`FaceIndex`].
    pub fn sorted_faces(&self) -> [f32; 6] {
        sorted_faces(self.faces)
    }

    /// The center of the box's AABB in local coordinates.
    pub fn center(&self) -> Vec3 {
        self.aabb().center().into()
    }

    /// The full extents of the box along each axis.
    pub fn extents(&self) -> [f32; 3] {
        box_extents(self.faces)
    }

    pub(crate) fn aabb(&self) -> Aabb {
        aabb_from_faces(self.faces)
    }

    pub(crate) fn face_centers(&self) -> [Vec3; 6] {
        face_centers(self.faces)
    }

    pub(crate) fn handle_ball(&self) -> Ball {
        let radius = (self.visuals.handle_scale)(self.extents());
        Ball::new(radius)
    }

    pub(crate) fn handle_entities(&self) -> [Entity; 6] {
        self.handle_entities
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
        self.faces = self.sorted_faces();
        self.reset_lines(line_handles, polylines)
    }

    pub(crate) fn transform_handles(
        &mut self,
        handles: &mut Query<(&mut BoxFrameHandle, &mut Transform)>,
    ) {
        let handle_scale = (self.visuals.handle_scale)(self.extents());
        for (face_center, handle_entity) in
            self.face_centers().into_iter().zip(self.handle_entities)
        {
            let Ok((mut handle, mut handle_tfm)) = handles.get_mut(handle_entity) else {
                return;
            };
            handle.base_scale = handle_scale;
            handle_tfm.translation = face_center;
            handle_tfm.scale = Vec3::splat(handle_scale);
        }
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
                *line_handle = self.visuals.edge_material.clone();
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
            *line_handle = self.visuals.edge_highlight_material.clone();
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

const NEG_X: FaceIndex = 0;
const NEG_Y: FaceIndex = 1;
const NEG_Z: FaceIndex = 2;
const POS_X: FaceIndex = 3;
const POS_Y: FaceIndex = 4;
const POS_Z: FaceIndex = 5;

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

fn face_centers(faces: [f32; 6]) -> [Vec3; 6] {
    let [x1, y1, z1, x2, y2, z2] = sorted_faces(faces);
    let c = 0.5 * Vec3::new(x1 + x2, y1 + y2, z1 + z2);
    [
        Vec3::new(x1, c.y, c.z),
        Vec3::new(x2, c.y, c.z),
        Vec3::new(c.x, y1, c.z),
        Vec3::new(c.x, y2, c.z),
        Vec3::new(c.x, c.y, z1),
        Vec3::new(c.x, c.y, z2),
    ]
}

fn box_extents(faces: [f32; 6]) -> [f32; 3] {
    let [x1, y1, z1, x2, y2, z2] = sorted_faces(faces);
    [(x2 - x1).abs(), (y2 - y1).abs(), (z2 - z1).abs()]
}

/// The median of three values.
pub fn median3(mut extents: [f32; 3]) -> f32 {
    extents.sort_unstable_by_key(|&x| FloatOrd(x));
    extents[1]
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
