// Depth-only sun-space rasterization for nearby real cast shadows.
// The shadow map is bounded; this is one near cascade, not global CSM.
@group(0) @binding(0) var<uniform> sun_view_proj: mat4x4<f32>;

struct VertexIn {
    @location(0) position: vec3<f32>,
}
@vertex fn vs_main(v: VertexIn) -> @builtin(position) vec4<f32> {
    return sun_view_proj * vec4<f32>(v.position, 1.0);
}
