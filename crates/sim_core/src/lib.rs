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
        self.time_seconds += dt.max(0.0);

        for bird in &mut self.birds {
            integrator::integrate_linear(bird, dt, self.params.min_speed, self.params.max_speed);
        }
    }
}

