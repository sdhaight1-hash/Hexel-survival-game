struct VertexIn {
    @location(0) position: vec2<f32>,
}
@vertex fn vs_main(v: VertexIn) -> @builtin(position) vec4<f32> {
    return vec4<f32>(v.position, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
