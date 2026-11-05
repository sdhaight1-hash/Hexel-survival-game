struct Camera {
    view_proj: mat4x4<f32>,
    // x: checker, y: checker scale, z: F8 candidate preview active
    options: vec4<f32>,
    eye_position: vec4<f32>,
    // Independent *gallery* conditions: wetness, moss, frost, metal glints.
    surface: vec4<f32>,
    // near chunk Q, R, horizontal radius; used only by distant LOD pass
    lod_near: vec4<f32>,
    // Complete surface columns only; incomplete near sections retain LOD.
    lod_covered: array<vec4<f32>, 128>,
    sky: vec4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var material_tiles: texture_2d_array<f32>;
@group(1) @binding(1) var tile_sampler: sampler;
@group(2) @binding(0) var shadow_depth: texture_depth_2d;
@group(2) @binding(1) var shadow_compare: sampler_comparison;
@group(2) @binding(2) var<uniform> sun_view_proj: mat4x4<f32>;

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

fn shade(v: VertexOut) -> vec4<f32> {
    // Hexel UVs are in metres, with 1m repeats for gallery candidates.
    let checker = camera.options.x > 0.5;
    var layer = select(v.material, 11u, checker);
    let scale = select(1.0, camera.options.y, checker);
    // The six *actual baked image layers* are appended after the 12
    // unchanged P0 layers. Never change the authored material identity:
    // this only selects alternate preview artwork in opt-in LOD mode.
    if camera.options.w > 0.5 && !checker {
        if v.material == 3u {
            // Rich grass is a real seed-owned surface identity. The dry
            // and sparse variants already carry their own material IDs.
            layer = 12u;
        } else if v.material == 13u || v.material == 14u {
            layer = v.material;
        } else if v.material == 2u {
            layer = 14u;
        } else if v.material == 0u {
            layer = 15u;
        } else if v.material == 1u {
            layer = 16u;
        } else if v.material == 8u {
            layer = 17u;
        }
    }
    let albedo = textureSample(material_tiles, tile_sampler, v.uv * scale, i32(layer));
    var color = albedo.rgb;

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
    let phase = camera.eye_position.w;
    let sun_direction = normalize(vec3<f32>(cos(phase) * 0.75, sin(phase), 0.35));
    let daylight = smoothstep(-0.12, 0.30, sun_direction.y);
    let sun_factor = max(dot(normal, sun_direction), 0.0);
    let ambient = 0.10 + 0.38 * daylight;
    let direct = 0.78 * max(sun_direction.y, 0.0) * sun_factor;
    var lit = ambient + direct;
    if camera.options.w > 0.5 && daylight > 0.01 {
        // Real cast shadows for the bounded near cascade. Shadow texture is
        // rendered from the directional sun each frame. Distant terrain
        // outside this cascade retains diffuse lighting.
        let light_clip = sun_view_proj * vec4<f32>(v.world_position + normal * 0.04, 1.0);
        let light_ndc = light_clip.xyz / light_clip.w;
        if abs(light_ndc.x) < 1.0 && abs(light_ndc.y) < 1.0
            && light_ndc.z > 0.0 && light_ndc.z < 1.0 {
            let uv = vec2<f32>(light_ndc.x * 0.5 + 0.5, 0.5 - light_ndc.y * 0.5);
            let visible = textureSampleCompareLevel(
                shadow_depth, shadow_compare, uv, light_ndc.z - 0.0015
            );
            lit = ambient + direct * visible;
        }
    }
    return vec4<f32>(clamp(color * v.color * lit, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}


@fragment fn fs_main(v: VertexOut) -> @location(0) vec4<f32> {
    return shade(v);
}

// Distant terrain never draws inside the live full-resolution streaming
// footprint. Pixel-accurate axial rounding avoids the square-grid overlay
// piling up against nearby hex-prism cliffs.
@fragment fn fs_lod(v: VertexOut) -> @location(0) vec4<f32> {
    // Evaluate implicit texture derivatives before the divergent discard.
    let shaded = shade(v);
    let rf = v.world_position.z / 0.8660254;
    let qf = v.world_position.x - rf * 0.5;
    let yf = -qf - rf;
    var q = round(qf);
    var r = round(rf);
    let ry = round(yf);
    let dq = abs(q - qf);
    let dr = abs(r - rf);
    let dy = abs(ry - yf);
    if dq > dy && dq > dr {
        q = -ry - r;
    } else if dr > dy {
        r = -q - ry;
    }
    let cq = floor(q / 16.0);
    let cr = floor(r / 16.0);
    let oq = cq - camera.lod_near.x;
    let orr = cr - camera.lod_near.y;
    let dist = max(abs(oq), max(abs(orr), abs(oq + orr)));
    if dist <= camera.lod_near.z {
        // Do NOT mask an entire requested R2/R5 radius. Only complete
        // full-resolution resident columns may replace the distant surface.
        for (var i: i32 = 0; i < 128; i = i + 1) {
            if f32(i) >= camera.lod_near.w {
                break;
            }
            let column = camera.lod_covered[i].xy;
            if cq == column.x && cr == column.y {
                discard;
            }
        }
    }
    return shaded;
}
