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
    @location(4) palette_id: u32,
    @location(5) pattern_seed: u32,
    @location(6) micro_mask: f32,
}
struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) material: u32,
    @location(3) world_position: vec3<f32>,
    @location(4) @interpolate(flat) palette_id: u32,
    @location(5) @interpolate(flat) pattern_seed: u32,
    @location(6) micro_mask: f32,
}
@vertex fn vs_main(v: VertexIn) -> VertexOut {
    var out: VertexOut;
    out.clip_position = camera.view_proj * vec4<f32>(v.position, 1.0);
    out.color = v.color;
    out.uv = v.uv;
    out.material = v.material;
    out.palette_id = v.palette_id;
    out.pattern_seed = v.pattern_seed;
    out.micro_mask = v.micro_mask;
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

// V5.5 per-hexel palette choices and edge blends come from CPU mesh colors.
// Geological structures below remain world-space 3D fields.

// V5.6: four close tones per assigned palette family, composed from two
// related 3D signals. The same signals also create shallow relief.
// The per-hexel seed changes the internal pattern while the V5.5 mesh
// keeps the parent palette tint and 14cm solid-contact blending.
struct RockMicro {
    pigment: vec3<f32>,
}
fn v56_micro(material: u32, seed: u32, position: vec3<f32>) -> RockMicro {
    let rock = (material / 3u) / 4u;
    let offset = vec3<f32>(
        f32(seed & 1023u),
        f32((seed >> 10u) & 1023u),
        f32((seed >> 20u) & 1023u)
    ) * 0.037;
    var basis = position * 2.25 + offset;
    if rock == 1u {
        // Shale: compressed lamination, not isotropic granite speckles.
        basis = position * vec3<f32>(2.4, 5.0, 2.3) + offset;
    } else if rock == 2u {
        // Basalt: dense volcanic groundmass with restrained breakup.
        basis = position * 3.0 + offset;
    }
    let coarse = v5_noise(basis);
    let small = v5_noise(basis * 2.1 + vec3<f32>(4.0, 7.0, 2.0));
    let a = smoothstep(0.22, 0.78, coarse);
    let b = smoothstep(0.28, 0.72, small);
    var t0 = vec3<f32>(0.976, 0.981, 0.987);
    var t1 = vec3<f32>(1.021, 0.992, 0.975);
    var t2 = vec3<f32>(0.989, 1.014, 1.025);
    var t3 = vec3<f32>(1.028, 1.018, 0.993);
    if rock == 1u {
        t0 = vec3<f32>(0.973, 0.979, 0.989);
        t1 = vec3<f32>(1.016, 1.008, 0.989);
        t2 = vec3<f32>(0.984, 1.002, 1.016);
        t3 = vec3<f32>(1.027, 1.020, 1.011);
    } else if rock == 2u {
        t0 = vec3<f32>(0.972, 0.982, 0.986);
        t1 = vec3<f32>(1.008, 1.004, 0.992);
        t2 = vec3<f32>(0.992, 1.008, 1.014);
        t3 = vec3<f32>(1.025, 1.017, 1.011);
    }
    var result: RockMicro;
    result.pigment = mix(mix(t0, t1, a), mix(t2, t3, a), b);
    return result;
}

// V5.7: Grayscale identity first. These three *different* 3D structures
// replace the old universal soft-mottled base. No normal perturbation.
// This is procedural prototype art, not a claimed baked texture asset.
// All samples use coherent gallery-local metres, independent of hexels.
fn v57_granite(p: vec3<f32>) -> f32 {
    // Interlocking coarse mineral grains. A 3D cellular partition
    // defines hard-ish grain domains; secondary within-grain speckle
    // reads as feldspar/quartz/mica instead of broad cloudy noise.
    let cell_pos = p * 5.8;
    let cell_base = floor(cell_pos);
    let local = fract(cell_pos);
    var nearest = 9.0;
    var next_nearest = 9.0;
    var mineral_id = 0.5;
    for (var z = -1; z <= 1; z = z + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            for (var x = -1; x <= 1; x = x + 1) {
                let offset = vec3<f32>(f32(x), f32(y), f32(z));
                let candidate = cell_base + offset;
                let jitter = vec3<f32>(
                    v5_hash(candidate + vec3<f32>(11.0, 2.0, 5.0)),
                    v5_hash(candidate + vec3<f32>(3.0, 7.0, 13.0)),
                    v5_hash(candidate + vec3<f32>(19.0, 17.0, 2.0))
                );
                let displacement = offset + 0.16 + jitter * 0.68 - local;
                let distance2 = dot(displacement, displacement);
                if distance2 < nearest {
                    next_nearest = nearest;
                    nearest = distance2;
                    mineral_id = v5_hash(candidate + vec3<f32>(29.0, 4.0, 8.0));
                } else if distance2 < next_nearest {
                    next_nearest = distance2;
                }
            }
        }
    }
    let boundary = 1.0 - smoothstep(0.0, 0.13, next_nearest - nearest);
    let feldspar = smoothstep(0.35, 0.66, mineral_id);
    let mica = 1.0 - smoothstep(0.10, 0.22, mineral_id);
    let micro = v5_noise(p * 15.0 + vec3<f32>(2.0, 17.0, 4.0));
    let grain = 0.45 + 0.33 * feldspar - 0.28 * mica
        + (micro - 0.5) * 0.20 - boundary * 0.075;
    return clamp(grain, 0.0, 1.0);
}
fn v57_shale(p: vec3<f32>) -> f32 {
    // Laminated and interrupted sediment planes, with short secondary
    // cleavage. Unlike granite, the dominant feature is *direction*.
    let warp = v5_noise(p * 0.62 + vec3<f32>(7.0, 3.0, 12.0)) - 0.5;
    let bedding_coord = p.y + p.x * 0.15 - p.z * 0.075 + warp * 0.11;
    let major = cos(bedding_coord * 21.0);
    let minor = cos(bedding_coord * 56.0 + warp * 3.0);
    let disruption = v5_noise(p * 1.2 + vec3<f32>(14.0, 2.0, 4.0));
    let strata = smoothstep(-0.22, 0.38, major);
    let fine_lamination = smoothstep(0.42, 0.92, minor) * (0.35 + disruption * 0.65);
    let cleavage = smoothstep(0.57, 0.79,
        v5_noise(p * vec3<f32>(1.9, 7.0, 2.1) + vec3<f32>(3.0, 8.0, 6.0)));
    return clamp(0.31 + strata * 0.38 + fine_lamination * 0.14
        - cleavage * 0.14 + (disruption - 0.5) * 0.06, 0.0, 1.0);
}
fn v57_basalt(p: vec3<f32>) -> f32 {
    // Dense fine groundmass and genuinely dark sparse round-ish vesicles,
    // not shale stripes or granite's contrasting grain domains.
    let ground = v5_noise(p * 13.0 + vec3<f32>(7.0, 4.0, 2.0));
    let vesicle_point = p * 4.2;
    let base = floor(vesicle_point);
    let local = fract(vesicle_point);
    var pore = 0.0;
    for (var z = -1; z <= 1; z = z + 1) {
        for (var y = -1; y <= 1; y = y + 1) {
            for (var x = -1; x <= 1; x = x + 1) {
                let offset = vec3<f32>(f32(x), f32(y), f32(z));
                let source = base + offset;
                let present = v5_hash(source + vec3<f32>(17.0, 9.0, 4.0));
                if present > 0.87 {
                    let jitter = vec3<f32>(
                        v5_hash(source + vec3<f32>(2.0, 3.0, 10.0)),
                        v5_hash(source + vec3<f32>(13.0, 5.0, 6.0)),
                        v5_hash(source + vec3<f32>(8.0, 19.0, 7.0))
                    );
                    let relative = offset + 0.22 + jitter * 0.56 - local;
                    let d = length(relative);
                    let radius = 0.20 + 0.13 * v5_hash(source + vec3<f32>(3.0, 11.0, 18.0));
                    pore = max(pore, 1.0 - smoothstep(radius - 0.035, radius + 0.045, d));
                }
            }
        }
    }
    let mottling = v5_noise(p * 2.2 + vec3<f32>(2.0, 6.0, 9.0));
    return clamp(0.49 + (ground - 0.5) * 0.18
        + (mottling - 0.5) * 0.11 - pore * 0.44, 0.0, 1.0);
}

fn v5_color(material: u32, position: vec3<f32>) -> vec3<f32> {
    let pad = material / 3u;
    let rock = pad / 4u;
    let stage = pad % 4u;
    // Same local coordinates on A/B/C/D pads: meaningful visual comparison.
    let p = position - vec3<f32>(12.0 * f32(stage), 0.0, 0.0);
    var gray = 0.5;
    var dark = 0.31;
    var light = 0.64;
    if rock == 0u {
        gray = v57_granite(p);
    } else if rock == 1u {
        gray = v57_shale(p);
        dark = 0.25;
        light = 0.56;
    } else {
        gray = v57_basalt(p);
        dark = 0.17;
        light = 0.39;
    }
    var value = mix(dark, light, gray);
    if stage >= 2u {
        // Original V5.3 structures remain separate from rock identity.
        if rock == 0u {
            let mineral = v5_noise(p * 2.0 + vec3<f32>(3.7, 8.2, 1.5));
            let feldspar = smoothstep(0.59, 0.79, mineral);
            value += feldspar * 0.022;
        } else if rock == 1u {
            let bedding = cos(3.8 * (p.y + 0.12 * p.x + 0.07 * p.z));
            value *= 1.0 - 0.065 * smoothstep(0.65, 0.94, bedding);
        } else {
            let porous = v5_noise(p * 2.4 + vec3<f32>(7.0, 4.0, 11.0));
            value *= 1.0 - 0.05 * smoothstep(0.70, 0.87, porous);
        }
    }
    var color = vec3<f32>(value);
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
            color = mix(color, vec3<f32>(0.64),
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
            color = mix(color, vec3<f32>(0.29), oxidation);
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

    // V5.7 retains the V5.6 related-tone per-hexel pigment and the
    // V5.5 edge fade. No bump height, derivatives or normal perturbation.
    if v5 && !checker && v.material != 69u && (v.material / 3u) % 4u != 0u {
        let pad = v.material / 3u;
        let p = v.world_position - vec3<f32>(12.0 * f32(pad % 4u), 0.0, 0.0);
        let micro = v56_micro(v.material, v.pattern_seed, p);
        color *= mix(vec3<f32>(1.0), micro.pigment,
            clamp(v.micro_mask, 0.0, 1.0));
    }
    let dx = dpdx(v.world_position);
    let dy = dpdy(v.world_position);
    let view_dir = normalize(camera.eye_position.xyz - v.world_position);
    var normal = normalize(cross(dx, dy));
    if dot(normal, view_dir) < 0.0 {
        normal = -normal;
    }
    if v5 && !checker && v.material != 69u {
        let sun = normalize(vec3<f32>(0.39, 0.78, 0.48));
        let diffuse = max(dot(normal, sun), 0.0);
        color *= 0.72 + 0.28 * diffuse;
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
        // F5 ID view shows each hexel's deterministic palette assignment.
        // F5 blend view reads the *actual* meshed edge interpolation.
        if camera.options.w < 3.5 {
            switch (v.palette_id) {
                case 0u: { return vec4<f32>(0.93, 0.20, 0.22, 1.0); }
                case 1u: { return vec4<f32>(0.19, 0.81, 0.27, 1.0); }
                case 2u: { return vec4<f32>(0.18, 0.46, 0.97, 1.0); }
                case 3u: { return vec4<f32>(0.95, 0.80, 0.12, 1.0); }
                default: { return vec4<f32>(0.80, 0.21, 0.86, 1.0); }
            }
        }
        return vec4<f32>(clamp((v.color - vec3<f32>(1.0)) * 2.4
            + vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
    }
    return vec4<f32>(clamp(color * v.color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
