use bevy::{
    asset::Asset,
    prelude::{AlphaMode, Color, Handle, Material, Shader},
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub(crate) const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(7825413687727800356);

/// A mesh material that only outputs a single color.
#[allow(missing_docs)]
#[derive(Asset, AsBindGroup, Clone, Debug, TypePath, TypeUuid)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct SolidColorMaterial {
    #[uniform(0)]
    pub color: Color,
    pub alpha_mode: AlphaMode,
}

impl Material for SolidColorMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_HANDLE.into()
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
