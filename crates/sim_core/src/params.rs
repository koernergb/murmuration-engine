#[derive(Debug, Clone)]
pub struct FlockParams {
    pub bird_count: u32,
    pub neighbor_radius: f32,
    pub separation_radius: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub boundary_weight: f32,
    pub boundary_radius: f32,
    pub noise_weight: f32,
    pub noise_scale: f32,
    pub noise_speed: f32,
    pub predator_weight: f32,
    pub predator_radius: f32,
    pub fear_decay: f32,
    pub fear_spread: f32,
    pub fear_turn_boost: f32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub max_turn_rate: f32,
    pub trail_decay: f32,
    pub exposure: f32,
    pub fog_density: f32,
}

impl Default for FlockParams {
    fn default() -> Self {
        Self {
            bird_count: 10_000,
            neighbor_radius: 8.0,
            separation_radius: 3.0,
            separation_weight: 1.2,
            alignment_weight: 0.8,
            cohesion_weight: 0.6,
            boundary_weight: 0.5,
            boundary_radius: 150.0,
            noise_weight: 0.8,
            noise_scale: 0.015,
            noise_speed: 0.2,
            predator_weight: 1.0,
            predator_radius: 12.0,
            fear_decay: 0.3,
            fear_spread: 0.4,
            fear_turn_boost: 0.5,
            min_speed: 2.0,
            max_speed: 12.0,
            max_turn_rate: 1.0,
            trail_decay: 0.92,
            exposure: 1.0,
            fog_density: 0.02,
        }
    }
}

