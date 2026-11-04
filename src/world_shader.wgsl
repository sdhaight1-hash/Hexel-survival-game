struct Camera {
    view_proj: mat4x4<f32>,
    // x: checker, y: checker scale, z: F8 candidate preview, w: F11 gray copper
    options: vec4<f32>,
    eye_position: vec4<f32>,
    // Independent *gallery* conditions: wetness, moss, frost, metal glints.
    surface: vec4<f32>,
    // x: F12 mask 0..5 / world-space vein 6, y: P palette index.
    family: vec4<f32>,
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

// Deterministic continuous 3D value noise in *metres of world position*.
// No per-face UV mapping or texture tile is involved. Rendering detail
// only; this is not the world-owned mineral deposit or mining quantity.
fn mineral_hash3(cell: vec3<f32>) -> f32 {
    return fract(sin(dot(cell, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}
fn mineral_noise3(point: vec3<f32>) -> f32 {
    let c = floor(point);
    let f = fract(point);
    let t = f * f * (vec3<f32>(3.0) - 2.0 * f);
    let x00 = mix(mineral_hash3(c), mineral_hash3(c + vec3<f32>(1.0, 0.0, 0.0)), t.x);
    let x10 = mix(mineral_hash3(c + vec3<f32>(0.0, 1.0, 0.0)),
                  mineral_hash3(c + vec3<f32>(1.0, 1.0, 0.0)), t.x);
    let x01 = mix(mineral_hash3(c + vec3<f32>(0.0, 0.0, 1.0)),
                  mineral_hash3(c + vec3<f32>(1.0, 0.0, 1.0)), t.x);
    let x11 = mix(mineral_hash3(c + vec3<f32>(0.0, 1.0, 1.0)),
                  mineral_hash3(c + vec3<f32>(1.0, 1.0, 1.0)), t.x);
    return mix(mix(x00, x10, t.y), mix(x01, x11, t.y), t.z);
}

// Returns (grayscale tone, ore alpha). A few bent diagonal vein sheets,
// interrupted by irregular 3D patches and secondary crossing fractures.
// The source, pigment, and host rock remain independently selectable.
// Two noise octaves cost more than a texture read; measure on native GPU.
fn world_mineral_vein(world_pos: vec3<f32>) -> vec2<f32> {
    let bend = mineral_noise3(world_pos * 0.42 + vec3<f32>(3.4, 5.9, 12.7));
    let body = mineral_noise3(world_pos * 0.19 + vec3<f32>(17.1, -4.3, 8.2));
    let warped = world_pos + vec3<f32>(0.85, 0.44, -0.52) * (bend - 0.5);
    let main_axis = dot(warped, normalize(vec3<f32>(0.62, 0.37, -0.69)))
        + 0.12 * sin(dot(world_pos, vec3<f32>(0.46, 0.37, 1.11)));
    let branch_axis = dot(warped, normalize(vec3<f32>(-0.32, 0.84, 0.44)))
        + 0.17 * sin(dot(world_pos, vec3<f32>(0.91, 0.31, -0.46)));
    let main_distance = abs(sin(main_axis * 0.84 + 0.58));
    let branch_distance = abs(sin(branch_axis * 1.06 - 0.41));
    let cluster_mask = smoothstep(0.32, 0.69, body);
    let primary_width = mix(0.030, 0.11, body);
    let primary = (1.0 - smoothstep(primary_width, primary_width + 0.10, main_distance))
        * (0.22 + 0.78 * cluster_mask);
    let secondary = (1.0 - smoothstep(0.028, 0.12, branch_distance))
        * cluster_mask * 0.67;
    let coverage = clamp(max(primary, secondary), 0.0, 1.0);
    let tone = clamp(0.22 + 0.59 * (0.55 * bend + 0.45 * body), 0.0, 1.0);
    return vec2<f32>(tone, coverage);
}

@fragment fn fs_main(v: VertexOut) -> @location(0) vec4<f32> {
    // Hexel UVs are in metres, with 1m repeats for gallery candidates.
    let checker = camera.options.x > 0.5;
    let is_copper = v.material == 9u || v.material == 10u;
    let gray_copper = !checker && camera.options.z > 0.5
        && camera.options.w > 0.5 && is_copper;
    // Source layers 12..17 remain unchanged. Pattern 6 is generated from
    // interpolated WORLD position, so it needs no new material texture layer.
    let pattern = u32(clamp(camera.family.x, 0.0, 6.0));
    let palette = u32(clamp(camera.family.y, 0.0, 2.0));
    let source_layer = select(v.material, 12u + min(pattern, 5u), gray_copper);
    let layer = select(source_layer, 11u, checker);
    let scale = select(1.0, camera.options.y, checker);
    let uv = v.uv * scale;
    let albedo = textureSample(material_tiles, tile_sampler, uv, i32(layer));
    var color = albedo.rgb;
    var ore_coverage = albedo.a;

    // Explicit gradients keep the extra host lookup valid inside a
    // material-dependent branch; all derivative operations are outside it.
    let uv_dx = dpdx(uv);
    let uv_dy = dpdy(uv);
    if (gray_copper) {
        let host_layer = select(7u, 8u, v.material == 10u);
        let host = textureSampleGrad(
            material_tiles, tile_sampler, uv, i32(host_layer), uv_dx, uv_dy
        );
        // Two distinct fields: source RGB is grayscale tonal detail,
        // while source alpha is mineral coverage (even for dark details).
        var tone = clamp(albedo.r, 0.0, 1.0);
        if (pattern == 6u) {
            let formation = world_mineral_vein(v.world_position);
            tone = formation.x;
            ore_coverage = formation.y;
        }
        var dark = vec3<f32>(0.13, 0.045, 0.018);
        var mid = vec3<f32>(0.39, 0.15, 0.060);
        var light = vec3<f32>(0.76, 0.40, 0.16);
        if (palette == 1u) {
            // Iron-oxide/hematite visual study, mostly matte.
            dark = vec3<f32>(0.080, 0.045, 0.050);
            mid = vec3<f32>(0.25, 0.11, 0.10);
            light = vec3<f32>(0.53, 0.30, 0.25);
        } else if (palette == 2u) {
            // Fantasy moonsilver, no inferred gameplay mineral ID.
            dark = vec3<f32>(0.035, 0.090, 0.13);
            mid = vec3<f32>(0.18, 0.33, 0.43);
            light = vec3<f32>(0.67, 0.84, 0.91);
        }
        let pigment = select(
            mix(dark, mid, min(tone * 2.0, 1.0)),
            mix(mid, light, min(tone * 2.0 - 1.0, 1.0)),
            tone > 0.5
        );
        color = mix(host.rgb, pigment, ore_coverage);
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
        if (is_copper && camera.surface.w > 0.5) {
            let exposed = ore_coverage * (1.0 - moss) * (1.0 - frost);
            let metal_sheen = pow(specular, 22.0) * 0.35;

            // Rare stable world-UV facets: light/view-dependent, never
            // full-bright emission, animated flicker, or a glow texture.
            let fleck_uv = v.uv * 48.0;
            let fleck_cell = floor(fleck_uv);
            let rare = smoothstep(0.976, 0.996, hash21(fleck_cell));
            let centre = length(fract(fleck_uv) - vec2<f32>(0.5, 0.5));
            let fleck = 1.0 - smoothstep(0.06, 0.29, centre);
            let sparkle = rare * fleck * pow(specular, 8.0) * 0.75;
            var sheen_tint = vec3<f32>(1.0, 0.69, 0.41);
            var response = 1.0;
            if (gray_copper && palette == 1u) {
                sheen_tint = vec3<f32>(0.69, 0.56, 0.51);
                response = 0.18; // hematite/oxide should not gleam like copper
            } else if (gray_copper && palette == 2u) {
                sheen_tint = vec3<f32>(0.72, 0.92, 1.0);
            }
            color += exposed * (metal_sheen + sparkle) * sheen_tint * response;
        }
    }
    return vec4<f32>(clamp(color * v.color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
