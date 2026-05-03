struct CameraUniform {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    horizon_color: vec4<f32>,
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,
    atmosphere: vec4<f32>,
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
    let env = mix(camera.horizon_color.xyz, camera.zenith_color.xyz, sky_mix);
    let highlight = pow(max(0.0, dot(reflected, sun_direction)), 18.0);
    let fresnel = pow(1.0 - max(0.0, dot(normal, view_direction)), 3.4);
    let diffuse = max(0.0, dot(normal, sun_direction));
    let core_shadow = smoothstep(0.0, 1.0, input.density);
    let chrome = env * (0.10 + fresnel * 0.26) + vec3<f32>(1.0) * highlight * 0.22;
    let base = mix(vec3<f32>(0.0005, 0.0007, 0.001), vec3<f32>(0.014, 0.016, 0.018), core_shadow);
    let silhouette = smoothstep(1.0, 0.22, radius2);
    let rim = pow(1.0 - max(0.0, dot(normal, view_direction)), 1.6);
    let lit = mix(base, chrome, 0.22) + vec3<f32>(0.010) * diffuse + vec3<f32>(0.035) * rim * 0.08;
    let distance_to_eye = distance(camera.eye.xyz, input.center_world_position);
    let fog_factor = 1.0 - exp(-distance_to_eye * fog_density * 0.045);
    let fog_color = mix(camera.horizon_color.xyz, camera.zenith_color.xyz, 0.62);
    let edge = smoothstep(1.0, 0.0, radius2);
    let core = smoothstep(1.0, 0.12, radius2);
    let alpha = edge * (0.16 + input.density * 0.26) * (0.58 + core * 0.42);
    let color = mix(lit, fog_color, clamp(fog_factor, 0.0, 0.72)) * exposure;
    let final_color = mix(color * 0.78, color, silhouette);
    return vec4<f32>(final_color, alpha);
}
