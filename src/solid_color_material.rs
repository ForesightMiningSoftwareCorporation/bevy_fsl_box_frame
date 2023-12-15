use bevy::{
    prelude::{AlphaMode, Color, Material},
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(AsBindGroup, Clone, Debug, TypePath, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct SolidColorMaterial {
    #[uniform(0)]
    pub color: Color,
    pub alpha_mode: AlphaMode,
}

impl Material for SolidColorMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/solid_color.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

impl From<Color> for SolidColorMaterial {
    fn from(color: Color) -> Self {
        Self {
            color,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}
