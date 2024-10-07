use bevy::{
    asset::Asset,
    color::{LinearRgba, Srgba},
    prelude::{AlphaMode, Handle, Material, Shader},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub(crate) const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(7825413687727800356);

/// A mesh material that only outputs a single color.
#[allow(missing_docs)]
#[derive(Asset, AsBindGroup, Clone, Debug, TypePath)]
pub struct SolidColorMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
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

impl From<Srgba> for SolidColorMaterial {
    fn from(color: Srgba) -> Self {
        Self {
            color: color.into(),
            alpha_mode: AlphaMode::Opaque,
        }
    }
}
