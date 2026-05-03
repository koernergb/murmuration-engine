pub mod bird;
pub mod forces;
pub mod integrator;
pub mod params;
pub mod spatial_grid;

pub use bird::Bird;
pub use params::FlockParams;

#[derive(Debug, Clone)]
pub struct SimulationState {
    birds: Vec<Bird>,
    params: FlockParams,
    time_seconds: f32,
}

impl SimulationState {
    pub fn new(birds: Vec<Bird>, params: FlockParams) -> Self {
        Self {
            birds,
            params,
            time_seconds: 0.0,
        }
    }

    pub fn birds(&self) -> &[Bird] {
        &self.birds
    }

    pub fn params(&self) -> &FlockParams {
        &self.params
    }

    pub fn time_seconds(&self) -> f32 {
        self.time_seconds
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        self.time_seconds += dt;
        let boundary_radius = self.params.boundary_radius.max(25.0);

        for (index, bird) in self.birds.iter_mut().enumerate() {
            let phase = self.time_seconds * (0.55 + bird.seed * 0.35) + bird.phase;
            let swirl = normalize_or_zero([-bird.position[2], 0.0, bird.position[0]]);
            let center_pull = normalize_or_zero([
                -bird.position[0],
                -bird.position[1] * 0.35,
                -bird.position[2],
            ]);
            let vertical_wave = [
                (phase * 0.7).sin() * 0.35,
                (phase * 1.6).cos() * 0.9,
                (phase * 0.9).sin() * 0.35,
            ];
            let index_bias = ((index % 29) as f32 / 29.0) - 0.5;
            let steering = add3(
                scale3(swirl, 2.4 + index_bias * 0.3),
                add3(scale3(center_pull, 0.85), vertical_wave),
            );

            bird.velocity = add3(bird.velocity, scale3(steering, dt * 0.9));
            integrator::integrate_linear(bird, dt, self.params.min_speed, self.params.max_speed);

            let distance = length3(bird.position);
            if distance > boundary_radius {
                let inward = scale3(normalize_or_zero(scale3(bird.position, -1.0)), 4.0);
                bird.velocity = add3(bird.velocity, scale3(inward, dt));
            }

            bird.density = ((boundary_radius - distance) / boundary_radius).clamp(0.0, 1.0);
            bird.fear = (phase * 0.35).sin().abs() * 0.08;
        }
    }
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize_or_zero(v: [f32; 3]) -> [f32; 3] {
    let length = length3(v);
    if length <= f32::EPSILON {
        [0.0; 3]
    } else {
        [v[0] / length, v[1] / length, v[2] / length]
    }
}
