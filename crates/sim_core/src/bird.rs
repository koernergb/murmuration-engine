#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bird {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub fear: f32,
    pub phase: f32,
    pub density: f32,
    pub seed: f32,
}

impl Bird {
    pub fn new(position: [f32; 3], velocity: [f32; 3]) -> Self {
        Self {
            position,
            velocity,
            fear: 0.0,
            phase: 0.0,
            density: 0.0,
            seed: 0.0,
        }
    }
}
