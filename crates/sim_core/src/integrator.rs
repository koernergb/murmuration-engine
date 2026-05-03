use crate::Bird;

pub fn integrate_linear(bird: &mut Bird, dt: f32, min_speed: f32, max_speed: f32) {
    let dt = dt.max(0.0);
    let speed = magnitude(bird.velocity).clamp(min_speed, max_speed);

    if speed > 0.0 {
        let direction = normalize_or_zero(bird.velocity);
        bird.velocity = scale(direction, speed);
    }

    bird.position = add(bird.position, scale(bird.velocity, dt));
    bird.phase += dt;
}

fn magnitude(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize_or_zero(v: [f32; 3]) -> [f32; 3] {
    let mag = magnitude(v);
    if mag <= f32::EPSILON {
        [0.0; 3]
    } else {
        [v[0] / mag, v[1] / mag, v[2] / mag]
    }
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

