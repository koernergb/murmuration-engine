use std::error::Error;
use std::time::Instant;

use renderer_wgpu::Renderer;
use sim_core::{Bird, FlockParams, SimulationState};
use winit::{dpi::PhysicalSize, window::Window};

pub struct NativeApp<'window> {
    renderer: Renderer<'window>,
    simulation: SimulationState,
    last_frame: Instant,
}

impl<'window> NativeApp<'window> {
    pub async fn new(window: &'window Window) -> Result<Self, Box<dyn Error>> {
        let params = FlockParams::default();
        let birds = seed_birds(params.bird_count as usize);

        Ok(Self {
            renderer: Renderer::new(window, birds.len()).await?,
            simulation: SimulationState::new(birds, params),
            last_frame: Instant::now(),
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.renderer.resize(size);
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().clamp(1.0 / 240.0, 1.0 / 24.0);
        self.last_frame = now;
        self.simulation.step(dt);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.draw(
            self.simulation.birds(),
            self.simulation.params(),
            self.simulation.time_seconds(),
        )
    }

    pub fn renderer_size(&self) -> PhysicalSize<u32> {
        self.renderer.size()
    }

    pub fn summary(&self) -> String {
        format!(
            "native_app milestone: {} birds, frame {}",
            self.simulation.birds().len(),
            self.renderer.frame_index()
        )
    }
}

fn seed_birds(count: usize) -> Vec<Bird> {
    let mut birds = Vec::with_capacity(count);
    for index in 0..count {
        let t = index as f32 / count as f32;
        let angle = t * std::f32::consts::TAU * 34.0;
        let radius = 18.0 + ((index % 113) as f32 / 113.0) * 42.0;
        let height = ((index % 41) as f32 / 41.0 - 0.5) * 18.0 + (t * 11.0).sin() * 4.0;
        let x = radius * angle.cos();
        let y = height;
        let z = radius * angle.sin();
        let mut bird = Bird::new(
            [x, y, z],
            [-angle.sin() * 4.0, (t * 9.0).cos() * 0.6, angle.cos() * 4.0],
        );
        bird.seed = (index % 97) as f32 / 97.0;
        bird.phase = t * std::f32::consts::TAU;
        birds.push(bird);
    }
    birds
}
