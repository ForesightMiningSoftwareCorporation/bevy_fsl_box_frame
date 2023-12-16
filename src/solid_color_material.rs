use bevy::{
    prelude::{AlphaMode, Color, HandleUntyped, Material, Shader},
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub const SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7825413687727800356);

#[derive(AsBindGroup, Clone, Debug, TypePath, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct SolidColorMaterial {
    #[uniform(0)]
    pub color: Color,
    pub alpha_mode: AlphaMode,
}

impl Material for SolidColorMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_HANDLE.typed().into()
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
