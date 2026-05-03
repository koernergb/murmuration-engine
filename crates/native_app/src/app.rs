use renderer_wgpu::Renderer;
use sim_core::{Bird, FlockParams, SimulationState};

pub struct NativeApp {
    renderer: Renderer,
    simulation: SimulationState,
}

impl NativeApp {
    pub fn new() -> Self {
        let params = FlockParams::default();
        let birds = seed_birds(params.bird_count as usize);

        Self {
            renderer: Renderer::new(),
            simulation: SimulationState::new(birds, params),
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.simulation.step(dt);
        self.renderer
            .draw(self.simulation.birds(), self.simulation.params());
    }

    pub fn summary(&self) -> String {
        format!(
            "native_app scaffold: {} birds, frame {}",
            self.simulation.birds().len(),
            self.renderer.frame_index()
        )
    }
}

fn seed_birds(count: usize) -> Vec<Bird> {
    let mut birds = Vec::with_capacity(count);
    for index in 0..count {
        let x = (index % 100) as f32 - 50.0;
        let y = ((index / 100) % 100) as f32 * 0.1 - 5.0;
        let z = (index / 10_000) as f32 - 1.0;
        birds.push(Bird::new([x, y, z], [3.0, 0.0, 0.5]));
    }
    birds
}

