struct CameraUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    atmosphere: vec4<f32>,
    ivory_color: vec4<f32>,
    brass_color: vec4<f32>,
    rust_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) local_position: vec2<f32>,
    @location(1) bird_position: vec4<f32>,
    @location(2) bird_velocity: vec4<f32>,
    @location(3) bird_aux: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) center_world_position: vec3<f32>,
    @location(2) local_uv: vec2<f32>,
    @location(3) density: f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let to_eye = normalize(camera.eye.xyz - input.bird_position.xyz);
    let provisional_right = cross(vec3<f32>(0.0, 1.0, 0.0), to_eye);
    let right = normalize(select(vec3<f32>(1.0, 0.0, 0.0), provisional_right, length(provisional_right) > 0.0001));
    let up = normalize(cross(to_eye, right));
    let scale = 0.14 + input.bird_aux.z * 0.09;
    let world_position =
        input.bird_position.xyz
        + right * input.local_position.x * scale
        + up * input.local_position.y * scale;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    out.world_position = world_position;
    out.center_world_position = input.bird_position.xyz;
    out.local_uv = input.local_position;
    out.density = input.bird_aux.z;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let radius2 = dot(input.local_uv, input.local_uv);
    if (radius2 > 1.0) {
        discard;
    }

    let fog_density = camera.atmosphere.x;
    let exposure = camera.atmosphere.y;
    let sphere_z = sqrt(max(0.0, 1.0 - radius2));
    let local_normal = vec3<f32>(input.local_uv, sphere_z);
    let sun_direction = normalize(camera.sun_direction.xyz);
    let view_direction = normalize(camera.eye.xyz - input.center_world_position);
    let provisional_right = cross(vec3<f32>(0.0, 1.0, 0.0), view_direction);
    let right = normalize(select(vec3<f32>(1.0, 0.0, 0.0), provisional_right, length(provisional_right) > 0.0001));
    let up = normalize(cross(view_direction, right));
    let forward = -view_direction;
    let basis = mat3x3<f32>(right, up, forward);
    let normal = normalize(basis * local_normal);
    let reflected = reflect(-view_direction, normal);
    let sky_mix = clamp(reflected.y * 0.5 + 0.5, 0.0, 1.0);
    let horizon_band = exp(-abs(reflected.y) * 24.0);
    let ground_color = mix(camera.rust_color.xyz, camera.horizon_color.xyz, 0.56);
    let sky_color = mix(camera.horizon_color.xyz, camera.zenith_color.xyz, sky_mix);
    let env = mix(ground_color, sky_color, smoothstep(0.06, 0.94, sky_mix));
    let banded_env = env + camera.horizon_color.xyz * horizon_band * 0.92;
    let highlight = pow(max(0.0, dot(reflected, sun_direction)), 32.0);
    let secondary_highlight = pow(max(0.0, dot(reflected, normalize(sun_direction + vec3<f32>(0.0, 0.35, 0.0)))), 18.0);
    let fresnel = pow(1.0 - max(0.0, dot(normal, view_direction)), 4.6);
    let diffuse = max(0.0, dot(normal, sun_direction));
    let core_shadow = smoothstep(0.0, 1.0, input.density);
    let cloud_radius = 30.0;
    let cloud_normal = normalize(vec3<f32>(
        input.center_world_position.x / cloud_radius,
        input.center_world_position.y / (cloud_radius * 0.35),
        input.center_world_position.z / cloud_radius
    ));
    let cloud_light = clamp(dot(-cloud_normal, sun_direction) * 0.5 + 0.5, 0.0, 1.0);
    let cloud_shadow = mix(0.42, 1.18, cloud_light);
    let chrome = banded_env * (0.28 + fresnel * 1.05)
        + camera.ivory_color.xyz * highlight * 1.05
        + camera.brass_color.xyz * secondary_highlight * 0.42;
    let base = mix(camera.zenith_color.xyz, camera.zenith_color.xyz * 0.16, core_shadow);
    let silhouette = smoothstep(1.0, 0.22, radius2);
    let rim = pow(1.0 - max(0.0, dot(normal, view_direction)), 1.6);
    let lit = (mix(base, chrome, 0.78) + vec3<f32>(0.008) * diffuse + vec3<f32>(0.05) * rim * 0.14) * cloud_shadow;
    let distance_to_eye = distance(camera.eye.xyz, input.center_world_position);
    let fog_factor = 1.0 - exp(-distance_to_eye * fog_density * 0.045);
    let fog_color = mix(camera.horizon_color.xyz, camera.zenith_color.xyz, 0.62);
    let edge = smoothstep(1.0, 0.78, radius2);
    let core = smoothstep(1.0, 0.08, radius2);
    let alpha = edge * (0.18 + input.density * 0.24) * (0.60 + core * 0.40);
    let color = mix(lit, fog_color, clamp(fog_factor, 0.0, 0.66)) * exposure;
    let final_color = mix(color * 0.86, color, silhouette);
    return vec4<f32>(final_color, alpha);
}
