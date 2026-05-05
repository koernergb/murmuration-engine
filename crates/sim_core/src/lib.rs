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
    cursor_repulsor: Option<[f32; 3]>,
}

impl SimulationState {
    pub fn new(birds: Vec<Bird>, params: FlockParams) -> Self {
        Self {
            birds,
            params,
            time_seconds: 0.0,
            cursor_repulsor: None,
        }
    }

    pub fn birds(&self) -> &[Bird] {
        &self.birds
    }

    pub fn params(&self) -> &FlockParams {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut FlockParams {
        &mut self.params
    }

    pub fn time_seconds(&self) -> f32 {
        self.time_seconds
    }

    pub fn replace_birds(&mut self, birds: Vec<Bird>) {
        self.birds = birds;
    }

    pub fn set_cursor_repulsor(&mut self, cursor_repulsor: Option<[f32; 3]>) {
        self.cursor_repulsor = cursor_repulsor;
    }

    pub fn step(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        self.time_seconds += dt;
        let boundary_radius = self.params.boundary_radius.max(25.0);
        let noise_scale = self.params.noise_scale.max(0.001);
        let anchors = cloud_anchors(self.time_seconds, boundary_radius);
        let cloud_center = centroid(&anchors);

        for (index, bird) in self.birds.iter_mut().enumerate() {
            let phase = self.time_seconds * (0.32 + bird.seed * 0.24) + bird.phase;
            let anchor_mix = bird.seed * (anchors.len() as f32 - 0.01);
            let base_anchor_index = anchor_mix.floor() as usize;
            let next_anchor_index = (base_anchor_index + 1) % anchors.len();
            let anchor_blend = anchor_mix.fract() * 0.65 + 0.15;
            let primary_anchor = lerp3(
                anchors[base_anchor_index],
                anchors[next_anchor_index],
                anchor_blend,
            );
            let relative = sub3(bird.position, primary_anchor);
            let to_center = sub3(cloud_center, bird.position);
            let swirl = normalize_or_zero([
                -relative[2] * 0.7 + to_center[0] * 0.2,
                relative[1] * 0.15,
                relative[0] * 0.7 + to_center[2] * 0.2,
            ]);
            let center_pull = normalize_or_zero(scale3(relative, -1.0));
            let boundary_force = soft_boundary_force(relative, boundary_radius);
            let lobe_pull = normalize_or_zero(to_center);
            let vertical_wave = [
                (phase * 0.48 + relative[2] * 0.028).sin() * 0.08,
                (phase * 0.86 + relative[0] * 0.021).cos() * 0.42,
                (phase * 0.44 - relative[1] * 0.024).sin() * 0.08,
            ];
            let curlish_noise = fake_curl_noise(relative, phase, noise_scale);
            let compression = [
                (relative[1] * 0.12 + phase * 0.55).sin() * 0.2,
                (relative[0] * 0.08 + phase * 0.4).cos() * 0.08,
                (relative[0] * 0.12 - phase * 0.58).cos() * 0.2,
            ];
            let lane_bias = (((index % 31) as f32 / 31.0) - 0.5) * 0.08;

            let steering = add3(
                scale3(swirl, 1.05 + lane_bias),
                add3(
                    scale3(center_pull, 0.9),
                    add3(
                        scale3(boundary_force, self.params.boundary_weight * 1.8),
                        add3(
                            scale3(lobe_pull, 0.42),
                            add3(
                                scale3(vertical_wave, 0.55),
                                add3(
                                    scale3(curlish_noise, self.params.noise_weight * 0.75),
                                    scale3(compression, 0.25),
                                ),
                            ),
                        ),
                    ),
                ),
            );

            bird.velocity = add3(bird.velocity, scale3(steering, dt));
            if let Some(cursor_repulsor) = self.cursor_repulsor {
                let offset = sub3(bird.position, cursor_repulsor);
                let cursor_radius = boundary_radius * 0.26;
                let distance = length3(offset);
                if distance < cursor_radius && distance > 0.001 {
                    let repel_strength = ((cursor_radius - distance) / cursor_radius).powi(2) * 22.0;
                    bird.velocity = add3(
                        bird.velocity,
                        scale3(normalize_or_zero(offset), repel_strength * dt),
                    );
                    bird.fear = bird.fear.max(0.18 * (1.0 - distance / cursor_radius));
                }
            }
            integrator::integrate_linear(
                bird,
                dt,
                self.params.min_speed,
                self.params.max_speed,
                self.params.max_turn_rate,
            );

            let lobe_distance = length3(relative) / (boundary_radius * 0.72);
            let cloud_distance = length3(sub3(bird.position, cloud_center)) / boundary_radius;
            let dark_core = (1.0 - lobe_distance).clamp(0.0, 1.0);
            let outer_falloff = (1.0 - cloud_distance * 0.85).clamp(0.0, 1.0);
            let pulse = ((phase * 0.42 + bird.seed * 8.0).sin() * 0.5) + 0.5;
            bird.density = (dark_core * 0.72 + outer_falloff * 0.28).clamp(0.05, 1.0);
            bird.density *= 0.82 + pulse * 0.18;
            bird.fear = 0.0;
        }
    }
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    add3(scale3(a, 1.0 - t), scale3(b, t))
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

fn cloud_anchors(time_seconds: f32, boundary_radius: f32) -> [[f32; 3]; 3] {
    let spread = boundary_radius * 0.32;
    [
        [
            (time_seconds * 0.28).sin() * spread,
            (time_seconds * 0.36).cos() * spread * 0.22,
            (time_seconds * 0.22).cos() * spread * 0.8,
        ],
        [
            (time_seconds * 0.24 + 1.8).sin() * spread * 0.92,
            (time_seconds * 0.31 + 0.6).sin() * spread * 0.16,
            (time_seconds * 0.29 + 1.2).cos() * spread * 0.72,
        ],
        [
            (time_seconds * 0.33 + 3.2).sin() * spread * 0.84,
            (time_seconds * 0.27 + 0.9).cos() * spread * 0.18,
            (time_seconds * 0.25 + 2.4).cos() * spread * 0.76,
        ],
    ]
}

fn centroid(points: &[[f32; 3]]) -> [f32; 3] {
    let mut sum = [0.0; 3];
    for point in points {
        sum = add3(sum, *point);
    }
    scale3(sum, 1.0 / points.len() as f32)
}
