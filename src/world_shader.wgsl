struct Camera {
    view_proj: mat4x4<f32>,
    // x: checker, y: checker scale, z: F8 candidate preview active
    options: vec4<f32>,
    eye_position: vec4<f32>,
    // Independent *gallery* conditions: wetness, moss, frost, metal glints.
    surface: vec4<f32>,
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
    @location(3) world_position: vec3<f32>,
}
@vertex fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = camera.view_proj * vec4<f32>(v.position, 1.0);
    out.color = v.color;
    out.uv = v.uv;
    out.material = v.material;
    out.world_position = v.position;
    return out;
}

// Fixed-in-world procedural patches for this small gallery. These are NOT
// world-owned moss, weather, snow or erosion states.
fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn patch_noise(p: vec2<f32>) -> f32 {
    let cell = floor(p);
    let fraction = fract(p);
    let t = fraction * fraction * (vec2<f32>(3.0, 3.0) - 2.0 * fraction);
    let low = mix(hash21(cell), hash21(cell + vec2<f32>(1.0, 0.0)), t.x);
    let high = mix(
        hash21(cell + vec2<f32>(0.0, 1.0)),
        hash21(cell + vec2<f32>(1.0, 1.0)),
        t.x
    );
    return mix(low, high, t.y);
}

// V5 proof of concept. All macro features use a shared 3D position,
// not per-face UVs, so a band or fracture traverses adjacent hexel surfaces.
// Intentionally low-frequency and low contrast, without extra texture reads.
fn v5_hash(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}
fn v5_noise(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let t0 = fract(p);
    let t = t0 * t0 * (vec3<f32>(3.0) - 2.0 * t0);
    let x00 = mix(v5_hash(cell), v5_hash(cell + vec3<f32>(1.0,0.0,0.0)), t.x);
    let x10 = mix(v5_hash(cell + vec3<f32>(0.0,1.0,0.0)), v5_hash(cell + vec3<f32>(1.0,1.0,0.0)), t.x);
    let x01 = mix(v5_hash(cell + vec3<f32>(0.0,0.0,1.0)), v5_hash(cell + vec3<f32>(1.0,0.0,1.0)), t.x);
    let x11 = mix(v5_hash(cell + vec3<f32>(0.0,1.0,1.0)), v5_hash(cell + vec3<f32>(1.0,1.0,1.0)), t.x);
    return mix(mix(x00,x10,t.y), mix(x01,x11,t.y), t.z);
}

fn v5_color(material: u32, position: vec3<f32>) -> vec3<f32> {
    let pad = material / 3u;
    let rock = pad / 4u;
    let stage = pad % 4u;

    // Each column repeats the same local pattern across comparison stages.
    let p = position - vec3<f32>(12.0 * f32(stage), 0.0, 0.0);
    let fine = v5_noise(p * 4.0);
    let grain = clamp((fine - 0.5) * 0.15, -0.075, 0.075);
    var dark = vec3<f32>(0.35, 0.32, 0.32);
    var light = vec3<f32>(0.57, 0.52, 0.48);
    if rock == 1u {
        dark = vec3<f32>(0.23, 0.26, 0.29);
        light = vec3<f32>(0.38, 0.40, 0.41);
    } else if rock == 2u {
        dark = vec3<f32>(0.16, 0.18, 0.20);
        light = vec3<f32>(0.30, 0.32, 0.34);
    }
    let tone = clamp(0.5 + grain, 0.0, 1.0);
    var color = mix(dark, light, tone);

    if stage >= 1u {
        // Ten-metre-scale gentle palette field, never random cell tint.
        let regional = (v5_noise(p * 0.11 + vec3<f32>(11.0,2.0,4.0)) - 0.5) * 0.26;
        color *= 1.0 + regional;
        color += vec3<f32>(regional * 0.025, -regional * 0.015, -regional * 0.035);
    }
    if stage >= 2u {
        var structural = 0.0;
        if rock == 0u {
            // Granite: quiet, irregular interlocking mineral aggregates.
            structural = smoothstep(0.63, 0.78, v5_noise(p * 1.45 + vec3<f32>(1.0,3.0,5.0))) * 0.075;
        } else if rock == 1u {
            // Shale: tilted, slightly warped bedding through full 3D volume.
            let phase = (p.y + p.x * 0.14 + p.z * 0.09) * 6.0
                + (v5_noise(p * 0.31) - 0.5) * 1.8;
            structural = smoothstep(0.85, 0.98, sin(phase)) * 0.075;
        } else {
            // Basalt: rare cooling-fracture tracers, not wall-covering grids.
            let a = abs(sin(p.x * 0.79 + p.y * 0.24 + v5_noise(p * 0.33)));
            let b = abs(sin(p.z * 0.64 - p.y * 0.32));
            structural = (1.0 - smoothstep(0.02, 0.09, min(a,b))) * 0.065;
        }
        color *= 1.0 - structural;
    }
    if stage >= 3u {
        // One locally coherent sparse accent mask in 3D, not per-hexel RNG.
        let accent = smoothstep(0.83, 0.92, v5_noise(p * 0.95 + vec3<f32>(23.0,11.0,5.0)));
        if rock == 0u {
            color = mix(color, vec3<f32>(0.58,0.55,0.52), accent * 0.32);
        } else if rock == 1u {
            color = mix(color, vec3<f32>(0.47,0.46,0.43), accent * 0.22);
        } else {
            color = mix(color, vec3<f32>(0.38,0.39,0.38), accent * 0.18);
        }
    }
    return color;
}

@fragment fn fs_main(v: VertexOut) -> @location(0) vec4<f32> {
    // Hexel UVs are in metres, with 1m repeats for gallery candidates.
    let checker = camera.options.x > 0.5;
    let v5 = camera.options.w > 1.5;
    let layer = select(select(v.material, 11u, checker), 0u, v5);
    let scale = select(1.0, camera.options.y, checker);
    let albedo = textureSample(material_tiles, tile_sampler, v.uv * scale, i32(layer));
    var color = albedo.rgb;
    if v5 && !checker {
        color = select(v5_color(v.material, v.world_position), vec3<f32>(0.48), v.material == 69u);
    }
    // In V4 gallery mode, the UV checker is procedural: layer 11 is a rock,
    // not the legacy checker image. Magenta/green reveals UV discontinuities.
    if (checker && camera.options.w > 0.5) {
        let grid = floor(v.uv * 8.0 * camera.options.y);
        let alternating = (i32(grid.x) + i32(grid.y)) % 2 == 0;
        color = select(vec3<f32>(0.10, 0.65, 0.24), vec3<f32>(0.83, 0.12, 0.58), alternating);
    }

    // Derivatives are calculated unconditionally for uniformity in WGSL.
    let dx = dpdx(v.world_position);
    let dy = dpdy(v.world_position);
    let view_dir = normalize(camera.eye_position.xyz - v.world_position);
    var normal = normalize(cross(dx, dy));
    if (dot(normal, view_dir) < 0.0) {
        normal = -normal;
    }

    // P0 and F6 checker remain byte-for-byte the same visual path.
    let is_rock = v.material >= 7u && v.material <= 10u;
    if (!checker && camera.options.z > 0.5 && is_rock) {
        let key_light = normalize(vec3<f32>(0.40, 0.80, 0.43));
        let half_dir = normalize(key_light + view_dir);
        let specular = max(dot(normal, half_dir), 0.0);
        let top_exposure = max(normal.y, 0.0);

        // A wet mineral surface is darker and less rough, but never
        // automatically metallic. Weather comes from independent state.
        let wet_patch = 0.65 + 0.35 * patch_noise(v.uv * 5.0);
        let wet = camera.surface.x * wet_patch;
        color *= 1.0 - 0.28 * wet;
        color += vec3<f32>(0.14, 0.17, 0.18) * pow(specular, 14.0) * wet;

        // Stylized pigment/cover masks only; no moss geometry or growth.
        let moss_pattern = 0.65 * patch_noise(v.uv * 5.0 + vec2<f32>(6.2, 2.4))
            + 0.35 * patch_noise(v.uv * 13.0);
        let moss = camera.surface.y
            * smoothstep(0.38, 0.72, moss_pattern)
            * (0.45 + 0.55 * top_exposure);
        color = mix(color, vec3<f32>(0.12, 0.22, 0.10) + color * 0.22, moss * 0.78);

        let frost_pattern = 0.65 * patch_noise(v.uv * 6.0 + vec2<f32>(1.6, 8.1))
            + 0.35 * patch_noise(v.uv * 17.0 + vec2<f32>(3.1, 4.4));
        let frost = camera.surface.z
            * smoothstep(0.43, 0.72, frost_pattern)
            * (0.25 + 0.75 * top_exposure);
        color = mix(color, vec3<f32>(0.64, 0.72, 0.77), frost * 0.80);

        // F8 ore-composite alpha stores the authored copper-only footprint.
        // All other layers (including P0) have no metallic response. The
        // host is still visible; moss/frost can physically cover the ore.
        let is_copper = v.material == 9u || v.material == 10u;
        if (is_copper && camera.surface.w > 0.5) {
            let exposed = albedo.a * (1.0 - moss) * (1.0 - frost);
            let metal_sheen = pow(specular, 22.0) * 0.35;

            // Rare stable world-UV facets: light/view-dependent, never
            // full-bright emission, animated flicker, or a glow texture.
            let fleck_uv = v.uv * 48.0;
            let fleck_cell = floor(fleck_uv);
            let rare = smoothstep(0.976, 0.996, hash21(fleck_cell));
            let centre = length(fract(fleck_uv) - vec2<f32>(0.5, 0.5));
            let fleck = 1.0 - smoothstep(0.06, 0.29, centre);
            let sparkle = rare * fleck * pow(specular, 8.0) * 0.75;
            color += exposed * (metal_sheen + sparkle) * vec3<f32>(1.0, 0.69, 0.41);
        }
    }
    // F5 test view: exaggerated palette tint (not actual material albedo).
    // Gray = neutral, colored patches = deterministic palette influence.
    // The underlying vertex interpolation is unchanged.
    if v5 && camera.options.w > 2.5 && !checker {
        let exaggerated = clamp((v.color - vec3<f32>(1.0)) * 4.0 + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));
        return vec4<f32>(exaggerated, 1.0);
    }
    return vec4<f32>(clamp(color * v.color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
