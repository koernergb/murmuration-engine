use crate::Bird;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceSample {
    pub separation: [f32; 3],
    pub alignment: [f32; 3],
    pub cohesion: [f32; 3],
    pub boundary: [f32; 3],
    pub noise: [f32; 3],
}

impl ForceSample {
    pub fn zero() -> Self {
        Self {
            separation: [0.0; 3],
            alignment: [0.0; 3],
            cohesion: [0.0; 3],
            boundary: [0.0; 3],
            noise: [0.0; 3],
        }
    }
}

pub fn placeholder_force(_bird: &Bird) -> ForceSample {
    ForceSample::zero()
}
