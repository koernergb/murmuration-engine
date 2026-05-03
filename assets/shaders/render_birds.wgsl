struct CameraUniform {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    tint: vec4<f32>,
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
    @location(0) color: vec4<f32>,
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
    let turn_highlight = 0.25 + 0.75 * abs(dot(forward, to_camera));
    let density_shadow = 1.0 - input.bird_aux.z * 0.22;
    let fear_tint = input.bird_aux.x * 0.3;
    let base = vec3<f32>(0.14, 0.17, 0.22) * density_shadow;
    let highlight = vec3<f32>(0.68, 0.72, 0.80) * turn_highlight;
    let color = mix(base, highlight, 0.45 + fear_tint);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    out.color = vec4<f32>(color * camera.tint.xyz, 0.92);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
