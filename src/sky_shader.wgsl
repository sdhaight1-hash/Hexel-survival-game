// Screen-space sky diagnostic: shares camera clock with the terrain shader.
// Intentionally one fullscreen triangle; the sky writes no depth.
struct Camera {
    view_proj: mat4x4<f32>,
    options: vec4<f32>,
    eye_position: vec4<f32>,
    surface: vec4<f32>,
    lod_near: vec4<f32>,
    lod_covered: array<vec4<f32>, 128>,
    sky: vec4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;
struct SkyVertex {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> SkyVertex {
    let corners = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    let p = corners[index];
    var out: SkyVertex;
    out.clip_position = vec4<f32>(p, 1.0, 1.0);
    out.ndc = p;
    return out;
}
@fragment fn fs_main(v: SkyVertex) -> @location(0) vec4<f32> {
    let day = camera.sky.z;
    let vertical = clamp(v.ndc.y * 0.5 + 0.5, 0.0, 1.0);
    let night = mix(vec3<f32>(0.045, 0.055, 0.11), vec3<f32>(0.008, 0.014, 0.036), vertical);
    let horizon = vec3<f32>(0.66, 0.76, 0.83);
    let zenith = vec3<f32>(0.20, 0.43, 0.77);
    var color = mix(night, mix(horizon, zenith, sqrt(vertical)), day);
    if camera.sky.w > 0.5 && day > 0.0 {
        let radial = length(v.ndc.xy - camera.sky.xy);
        let corona = 0.075 / (1.0 + radial * radial * 720.0);
        let disc = 1.0 - smoothstep(0.017, 0.023, radial);
        color += vec3<f32>(1.0, 0.72, 0.36) * corona * day;
        color = mix(color, vec3<f32>(1.0, 0.94, 0.70), disc * day);
    }
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
