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
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let forward = normalize(select(vec3<f32>(0.0, 0.0, 1.0), input.bird_velocity.xyz, length(input.bird_velocity.xyz) > 0.0001));
    let provisional_right = cross(vec3<f32>(0.0, 1.0, 0.0), forward);
    let right = normalize(select(vec3<f32>(1.0, 0.0, 0.0), provisional_right, length(provisional_right) > 0.0001));
    let wing_up = normalize(cross(forward, right));

    let scale = 0.85 + input.bird_aux.z * 0.45;
    let world_position =
        input.bird_position.xyz
        + right * input.local_position.x * scale
        + wing_up * input.local_position.y * scale;

    let to_camera = normalize(camera.eye.xyz - world_position);
    let sun_direction = normalize(camera.sun_direction.xyz);
    let turn_highlight = 0.2 + 0.8 * abs(dot(forward, to_camera));
    let sun_grazing = 0.3 + 0.7 * max(0.0, dot(-forward, sun_direction));
    let density_shadow = 1.0 - input.bird_aux.z * 0.25;
    let fear_tint = input.bird_aux.x * 0.22;
    let base = vec3<f32>(0.14, 0.17, 0.22) * density_shadow;
    let highlight = vec3<f32>(0.72, 0.74, 0.79) * turn_highlight * sun_grazing;
    let color = mix(base, highlight, 0.42 + fear_tint);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    out.world_position = world_position;
    out.color = vec4<f32>(color, 0.92);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let fog_density = camera.atmosphere.x;
    let exposure = camera.atmosphere.y;
    let distance_to_eye = distance(camera.eye.xyz, input.world_position);
    let fog_factor = 1.0 - exp(-distance_to_eye * fog_density * 0.032);
    let fog_color = mix(camera.horizon_color.xyz, camera.zenith_color.xyz, 0.58);
    let color = mix(input.color.xyz, fog_color, clamp(fog_factor, 0.0, 0.92)) * exposure;
    return vec4<f32>(color, input.color.a);
}
