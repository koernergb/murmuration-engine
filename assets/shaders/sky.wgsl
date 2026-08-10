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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );

    var out: VertexOutput;
    out.ndc = positions[vertex_index];
    out.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let clip = vec4<f32>(input.ndc, 1.0, 1.0);
    let world = camera.inverse_view_proj * clip;
    let ray = normalize((world.xyz / world.w) - camera.eye.xyz);
    let up = clamp(ray.y * 0.5 + 0.5, 0.0, 1.0);

    let horizon_mix = smoothstep(0.0, 0.78, up);
    let upper = mix(camera.ivory_color.xyz, camera.zenith_color.xyz, pow(up, 1.35));
    let base = mix(camera.horizon_color.xyz, upper, horizon_mix);

    let sun_direction = normalize(camera.sun_direction.xyz);
    let sun_amount = pow(max(0.0, dot(ray, sun_direction)), 28.0);
    let warm_haze = pow(1.0 - abs(ray.y), 3.0) * 0.25;
    let sun_glow = camera.brass_color.xyz * sun_amount * 0.55;
    let haze = mix(camera.horizon_color.xyz, camera.rust_color.xyz, 0.32) * warm_haze * 0.22;
    let background_alpha = camera.atmosphere.w;

    return vec4<f32>((base + haze + sun_glow) * background_alpha, background_alpha);
}
