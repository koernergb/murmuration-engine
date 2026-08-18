use crate::Bird;

#[derive(Debug, Clone)]
pub struct SpatialGrid {
    pub cell_size: f32,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size }
    }

    pub fn cell_for(&self, bird: &Bird) -> [i32; 3] {
        [
            (bird.position[0] / self.cell_size).floor() as i32,
            (bird.position[1] / self.cell_size).floor() as i32,
            (bird.position[2] / self.cell_size).floor() as i32,
        ]
    }
}
