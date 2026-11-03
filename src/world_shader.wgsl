struct Camera {
    view_proj: mat4x4<f32>,
    // x: checker debug toggle; y: texel density relative to 128px/m
    options: vec4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var material_tiles: texture_2d_array<f32>;
@group(1) @binding(1) var tile_sampler: sampler;

struct VertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) material: u32,
}
struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) material: u32,
}
@vertex fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = camera.view_proj * vec4<f32>(v.position, 1.0);
    out.color = v.color;
    out.uv = v.uv;
    out.material = v.material;
    return out;
}
@fragment fn fs_main(v: VertexOut) -> @location(0) vec4<f32> {
    // All vertex UVs are in physical metres. Material images currently
    // contain one repeat per metre at the 128px/m benchmark.
    let checker = camera.options.x > 0.5;
    let layer = select(v.material, 11u, checker);
    // Scale only the diagnostic grid. Scaling the pre-baked soil/grass
    // transition vertically would incorrectly repeat turf below the rim.
    let scale = select(1.0, camera.options.y, checker);
    let albedo = textureSample(material_tiles, tile_sampler, v.uv * scale, i32(layer));
    return vec4<f32>(albedo.rgb * v.color, 1.0);
}
