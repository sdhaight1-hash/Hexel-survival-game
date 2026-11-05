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

// V5.2: Five rock-specific palette anchors in a stable 2D metre grid.
// The exact same field is sampled by all faces, eliminating per-cell tint jumps.
// The gallery offsets stage columns to make B/C/D comparable.
fn palette_id(grid: vec2<i32>, rock: u32) -> u32 {
    let q = bitcast<u32>(grid.x);
    let r = bitcast<u32>(grid.y);
    var h = q * 0x9e3779b9u ^ r * 0x85ebca6bu ^ rock * 0xc2b2ae35u ^ 42u;
    h = (h ^ (h >> 16u)) * 0x7feb352du;
    h = (h ^ (h >> 15u)) * 0x846ca68bu;
    return (h ^ (h >> 16u)) % 5u;
}
fn palette_rgb(rock: u32, id: u32) -> vec3<f32> {
    if rock == 0u {
        switch (id) {
            case 0u: { return vec3<f32>(1.02, 1.00, 0.99); }
            case 1u: { return vec3<f32>(1.20, 0.92, 0.86); }
            case 2u: { return vec3<f32>(0.88, 0.99, 1.17); }
            case 3u: { return vec3<f32>(1.20, 1.08, 0.88); }
            default: { return vec3<f32>(0.85, 0.90, 1.03); }
        }
    }
    if rock == 1u {
        switch (id) {
            case 0u: { return vec3<f32>(1.00, 1.00, 1.00); }
            case 1u: { return vec3<f32>(1.17, 1.03, 0.90); }
            case 2u: { return vec3<f32>(0.87, 1.01, 1.18); }
            case 3u: { return vec3<f32>(0.82, 0.91, 0.91); }
            default: { return vec3<f32>(1.15, 1.13, 0.96); }
        }
    }
    switch (id) {
        case 0u: { return vec3<f32>(1.00, 1.00, 1.00); }
        case 1u: { return vec3<f32>(1.18, 1.06, 0.97); }
        case 2u: { return vec3<f32>(0.88, 0.97, 1.17); }
        case 3u: { return vec3<f32>(1.18, 1.17, 1.09); }
        default: { return vec3<f32>(0.82, 0.90, 0.94); }
    }
}
// Regions are roughly 5 metres wide. Corners have discrete IDs, but the
// surface interpolates their color multipliers with cubic smoothstep.
fn palette_region(p: vec3<f32>, rock: u32) -> vec3<f32> {
    let uv = p.xz / 5.0;
    let origin = floor(uv);
    let fraction = fract(uv);
    let t = fraction * fraction * (vec2<f32>(3.0) - 2.0 * fraction);
    let grid = vec2<i32>(origin);
    let c00 = palette_rgb(rock, palette_id(grid, rock));
    let c10 = palette_rgb(rock, palette_id(grid + vec2<i32>(1, 0), rock));
    let c01 = palette_rgb(rock, palette_id(grid + vec2<i32>(0, 1), rock));
    let c11 = palette_rgb(rock, palette_id(grid + vec2<i32>(1, 1), rock));
    return mix(mix(c00, c10, t.x), mix(c01, c11, t.x), t.y);
}
fn palette_id_debug(p: vec3<f32>, rock: u32) -> vec3<f32> {
    let id = palette_id(vec2<i32>(floor(p.xz / 5.0 + vec2<f32>(0.5))), rock);
    switch (id) {
        case 0u: { return vec3<f32>(0.93, 0.20, 0.22); }
        case 1u: { return vec3<f32>(0.19, 0.81, 0.27); }
        case 2u: { return vec3<f32>(0.18, 0.46, 0.97); }
        case 3u: { return vec3<f32>(0.95, 0.80, 0.12); }
        default: { return vec3<f32>(0.80, 0.21, 0.86); }
    }
}

fn v5_color(material: u32, position: vec3<f32>) -> vec3<f32> {
    let pad = material / 3u;
    let rock = pad / 4u;
    let stage = pad % 4u;

    // Each column repeats the same local pattern across comparison stages.
    let p = position - vec3<f32>(12.0 * f32(stage), 0.0, 0.0);
    // The base stays quiet: two grain scales, with room for flat material.
    // Shared 3D positions make the grain continuous across exposed hex faces.
    let fine = v5_noise(p * 4.3 + vec3<f32>(8.0, 2.0, 3.0));
    let mid = v5_noise(p * 1.6 + vec3<f32>(2.0, 9.0, 5.0));
    var dark = vec3<f32>(0.35, 0.32, 0.32);
    var light = vec3<f32>(0.57, 0.52, 0.48);
    if rock == 1u {
        dark = vec3<f32>(0.23, 0.26, 0.29);
        light = vec3<f32>(0.38, 0.40, 0.41);
    } else if rock == 2u {
        dark = vec3<f32>(0.16, 0.18, 0.20);
        light = vec3<f32>(0.30, 0.32, 0.34);
    }
    // V5.4: modest increase in inherent rock read, not broad contrast.
    let grain_strength = select(0.41, 0.28, rock == 2u);
    let tone = clamp(0.50 + (fine - 0.5) * grain_strength
        + (mid - 0.5) * 0.25, 0.0, 1.0);
    var color = mix(dark, light, tone);

    if stage >= 1u {
        // Same V5.2 five-palette field; do NOT randomize per hexel.
        color *= palette_region(p, rock);
    }
    if stage >= 2u {
        if rock == 0u {
            // Granite: small, irregularly grouped feldspar and mica minerals.
            // No dark polygon outlines and no repeating rectangular 'stones'.
            let mineral = v5_noise(p * 2.0 + vec3<f32>(3.7, 8.2, 1.5));
            let feldspar = smoothstep(0.59, 0.79, mineral);
            let mica = smoothstep(0.73, 0.88,
                v5_noise(p * 3.7 + vec3<f32>(14.0, 2.0, 9.0)));
            color = mix(color, vec3<f32>(0.56, 0.47, 0.43), feldspar * 0.26);
            color *= 1.0 - 0.11 * mica;
        } else if rock == 1u {
            // Shale: warped bedding in a coherent 3D geological volume.
            // Interrupted bands are less regular than V5.2 sine stripes.
            let warp = v5_noise(p * 0.44 + vec3<f32>(4.0, 6.0, 3.0));
            let phase = 3.8 * (p.y + 0.12 * p.x + 0.07 * p.z)
                + 1.8 * (warp - 0.5);
            let bedding = smoothstep(0.65, 0.94, cos(phase));
            let continuity = smoothstep(0.32, 0.62,
                v5_noise(p * 0.83 + vec3<f32>(9.0, 2.0, 1.0)));
            color *= 1.0 - 0.14 * bedding * continuity;
        } else {
            // Basalt: nearly quiet volcanic groundmass with occasional
            // irregular pores and thin disrupted dark fractures.
            let porous = v5_noise(p * 2.4 + vec3<f32>(7.0, 4.0, 11.0));
            let pores = smoothstep(0.70, 0.87, porous);
            let fracture_field = v5_noise(p * 0.74 + vec3<f32>(2.0, 7.0, 13.0));
            let crack = 1.0 - smoothstep(0.015, 0.06,
                abs(fracture_field - 0.39));
            let fracture_mask = smoothstep(0.56, 0.76, mid);
            color *= 1.0 - 0.09 * pores - 0.10 * crack * fracture_mask;
        }
    }
    if stage >= 3u {
        // V5.4: rare, rock-specific *disruptions* in a shared geological
        // coordinate volume. These overlays are not repeated per hex face.
        // Wide low-frequency gates prevent every metre from having a crack.
        if rock == 0u {
            // Granite: localized joint planes, with very rare pale quartz
            // traces displaced from the joint. Avoid polygon outlines.
            let joint_warp = (v5_noise(p * 0.32 + vec3<f32>(13.0, 3.0, 7.0)) - 0.5) * 0.50;
            let joint_phase = p.x * 0.61 + p.z * 0.24 + p.y * 0.08 + joint_warp;
            let joint_distance = abs(sin(joint_phase * 3.14159265));
            let joint_gate = smoothstep(0.57, 0.76,
                v5_noise(p * 0.47 + vec3<f32>(17.0, 6.0, 2.0)));
            let joint_line = 1.0 - smoothstep(0.014, 0.052, joint_distance);
            color *= 1.0 - 0.19 * joint_line * joint_gate;
            let quartz_gate = smoothstep(0.77, 0.88,
                v5_noise(p * 0.65 + vec3<f32>(4.0, 18.0, 6.0)));
            let quartz_line = 1.0 - smoothstep(0.040, 0.105,
                abs(joint_distance - 0.085));
            color = mix(color, vec3<f32>(0.65, 0.63, 0.61),
                quartz_line * quartz_gate * 0.25);
        } else if rock == 1u {
            // Shale: a rare displaced bedding/slip zone running through
            // the volume, not another perfectly regular band on each face.
            let shear_warp = (v5_noise(p * 0.36 + vec3<f32>(3.0, 15.0, 8.0)) - 0.5) * 0.42;
            let shear_plane = abs(sin((p.y * 0.70 + p.x * 0.26
                - p.z * 0.17 + shear_warp) * 3.14159265));
            let shear_gate = smoothstep(0.59, 0.78,
                v5_noise(p * 0.54 + vec3<f32>(10.0, 4.0, 20.0)));
            let slip_trace = (1.0 - smoothstep(0.018, 0.09, shear_plane))
                * shear_gate;
            let bruised = smoothstep(0.12, 0.35, shear_plane)
                * (1.0 - smoothstep(0.35, 0.65, shear_plane)) * shear_gate;
            color *= 1.0 - 0.20 * slip_trace - 0.045 * bruised;
        } else {
            // Basalt: sparse irregular cooling-joint corridors with a
            // local oxidation tint. These do not form a repeated hex grid.
            let cool_warp = (v5_noise(p * 0.40 + vec3<f32>(2.0, 7.0, 19.0)) - 0.5) * 0.75;
            let cool_plane = abs(sin((p.x * 0.48 - p.z * 0.37
                + p.y * 0.09 + cool_warp) * 3.14159265));
            let cool_gate = smoothstep(0.64, 0.82,
                v5_noise(p * 0.52 + vec3<f32>(9.0, 6.0, 12.0)));
            let cooling_crack = (1.0 - smoothstep(0.012, 0.075, cool_plane))
                * cool_gate;
            color *= 1.0 - 0.24 * cooling_crack;
            let oxidation = (1.0 - smoothstep(0.06, 0.24, cool_plane))
                * cool_gate * 0.14;
            color = mix(color, vec3<f32>(0.34, 0.28, 0.24), oxidation);
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
    if v5 && camera.options.w > 2.5 && !checker && v.material != 69u {
        let pad = v.material / 3u;
        if pad % 4u == 0u {
            return vec4<f32>(vec3<f32>(0.5), 1.0);
        }
        let rock = pad / 4u;
        let p = v.world_position - vec3<f32>(12.0 * f32(pad % 4u), 0.0, 0.0);
        // F5 stage 1 = raw five-color region IDs, stage 2 = blended weights.
        if camera.options.w < 3.5 {
            return vec4<f32>(palette_id_debug(p, rock), 1.0);
        }
        let blended = palette_region(p, rock);
        return vec4<f32>(clamp((blended - vec3<f32>(1.0)) * 2.4 + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
    }
    return vec4<f32>(clamp(color * v.color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
