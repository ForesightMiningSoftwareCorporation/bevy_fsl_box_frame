#import bevy_pbr::forward_io::VertexOutput

struct SolidColorMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0)
var<uniform> material: SolidColorMaterial;

@fragment
fn fragment(
    _mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return material.color;
}
