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
        let noise_scale = self.params.noise_scale.max(0.001);

        for (index, bird) in self.birds.iter_mut().enumerate() {
            let phase = self.time_seconds * (0.45 + bird.seed * 0.35) + bird.phase;
            let center = [
                (self.time_seconds * 0.13).sin() * boundary_radius * 0.08,
                (self.time_seconds * 0.09).cos() * 6.5,
                (self.time_seconds * 0.11).cos() * boundary_radius * 0.06,
            ];
            let relative = sub3(bird.position, center);
            let swirl = normalize_or_zero([-relative[2], 0.18 * relative[1], relative[0]]);
            let center_pull = normalize_or_zero(scale3(relative, -1.0));
            let boundary_force = soft_boundary_force(relative, boundary_radius);
            let wave_lift = [
                (phase * 0.7 + relative[2] * 0.018).sin() * 0.2,
                (phase * 1.15 + relative[0] * 0.015).cos() * 0.95,
                (phase * 0.65 - relative[1] * 0.02).sin() * 0.22,
            ];
            let curlish_noise = fake_curl_noise(relative, phase, noise_scale);
            let ribbon_shear = [
                (relative[1] * 0.055 + phase * 0.8).sin() * 0.7,
                (relative[0] * 0.025 + phase).cos() * 0.1,
                (relative[0] * 0.055 - phase * 0.75).cos() * 0.7,
            ];
            let lane_bias = (((index % 31) as f32 / 31.0) - 0.5) * 0.25;

            let steering = add3(
                scale3(swirl, 1.8 + lane_bias),
                add3(
                    scale3(center_pull, 0.62),
                    add3(
                        scale3(boundary_force, self.params.boundary_weight * 1.6),
                        add3(
                            scale3(wave_lift, 0.95),
                            add3(
                                scale3(curlish_noise, self.params.noise_weight * 1.3),
                                scale3(ribbon_shear, 0.42),
                            ),
                        ),
                    ),
                ),
            );

            bird.velocity = add3(bird.velocity, scale3(steering, dt));
            integrator::integrate_linear(
                bird,
                dt,
                self.params.min_speed,
                self.params.max_speed,
                self.params.max_turn_rate,
            );

            let horizontal_distance =
                (relative[0] * relative[0] + relative[2] * relative[2]).sqrt() / boundary_radius;
            let vertical_band = (relative[1].abs() / (boundary_radius * 0.32)).clamp(0.0, 1.0);
            let fold = ((phase * 0.55 + horizontal_distance * 8.0).sin() * 0.5) + 0.5;
            bird.density = (1.0 - horizontal_distance * 0.78) * (1.0 - vertical_band * 0.55);
            bird.density = bird.density.clamp(0.08, 1.0) * (0.75 + fold * 0.25);
            bird.fear = ((phase * 0.25 + horizontal_distance * 6.0).sin().abs() * 0.05).min(0.12);
        }
    }
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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

fn soft_boundary_force(position: [f32; 3], radius: f32) -> [f32; 3] {
    let ellipsoid = [
        position[0] / radius,
        position[1] / (radius * 0.34),
        position[2] / radius,
    ];
    let extent = length3(ellipsoid);

    if extent <= 0.72 {
        return [0.0; 3];
    }

    let pressure = ((extent - 0.72) / 0.28).clamp(0.0, 1.6);
    scale3(normalize_or_zero(scale3(position, -1.0)), pressure * pressure * 3.2)
}

fn fake_curl_noise(position: [f32; 3], phase: f32, scale: f32) -> [f32; 3] {
    let px = position[0] * scale;
    let py = position[1] * scale * 1.8;
    let pz = position[2] * scale;

    [
        (py + phase * 0.8).sin() - (pz - phase * 0.45).cos(),
        (pz + phase * 0.6).sin() - (px + phase * 0.5).cos(),
        (px - phase * 0.7).sin() - (py + phase * 0.35).cos(),
    ]
}
