#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub eye: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: [0.0, 25.0, 80.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
        }
    }
}

