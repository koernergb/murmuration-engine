use std::error::Error;
use std::time::Instant;

use renderer_wgpu::Renderer;
use sim_core::{Bird, FlockParams, SimulationState};
use winit::{dpi::PhysicalSize, window::Window};

use crate::input::AppAction;
use crate::ui::{self, RuntimeStats, UiState};

pub struct NativeApp<'window> {
    renderer: Renderer<'window>,
    simulation: SimulationState,
    last_frame: Instant,
    stats: RuntimeStats,
    ui: UiState,
}

impl<'window> NativeApp<'window> {
    pub async fn new(window: &'window Window) -> Result<Self, Box<dyn Error>> {
        let params = FlockParams::default();
        let birds = seed_birds(params.bird_count as usize);
        let bird_count = birds.len();

        Ok(Self {
            renderer: Renderer::new(window, birds.len()).await?,
            simulation: SimulationState::new(birds, params),
            last_frame: Instant::now(),
            stats: RuntimeStats {
                fps: 0.0,
                frame_time_ms: 0.0,
                bird_count,
                noise_weight: FlockParams::default().noise_weight,
                boundary_radius: FlockParams::default().boundary_radius,
                fog_density: FlockParams::default().fog_density,
            },
            ui: UiState::default(),
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
        self.update_stats(dt);
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

    pub fn handle_action(&mut self, action: AppAction) {
        match action {
            AppAction::ApplyPreset(index) => {
                if index < ui::preset_count() {
                    let params = self.simulation.params_mut();
                    if ui::apply_preset(params, index).is_ok() {
                        self.ui.active_preset = index;
                        self.reseed_birds();
                    }
                }
            }
            AppAction::AdjustBirdCount(delta) => {
                let new_count = (self.simulation.params().bird_count as i32 + delta).clamp(600, 6_000);
                self.simulation.params_mut().bird_count = new_count as u32;
                self.reseed_birds();
            }
            AppAction::AdjustNoise(delta) => {
                let params = self.simulation.params_mut();
                params.noise_weight = (params.noise_weight + delta).clamp(0.0, 2.5);
            }
            AppAction::AdjustBoundary(delta) => {
                let params = self.simulation.params_mut();
                params.boundary_radius = (params.boundary_radius + delta).clamp(16.0, 56.0);
            }
            AppAction::AdjustFog(delta) => {
                let params = self.simulation.params_mut();
                params.fog_density = (params.fog_density + delta).clamp(0.0, 0.08);
            }
            AppAction::ResetFlock => self.reseed_birds(),
            AppAction::ToggleStats => self.ui.show_stats = !self.ui.show_stats,
        }

        self.refresh_stats_from_params();
    }

    pub fn window_title(&self) -> String {
        ui::window_title(&self.ui, &self.stats)
    }

    pub fn summary(&self) -> String {
        format!(
            "native_app milestone: {} birds, frame {}",
            self.simulation.birds().len(),
            self.renderer.frame_index()
        )
    }

    fn reseed_birds(&mut self) {
        let bird_count = self.simulation.params().bird_count as usize;
        let birds = seed_birds(bird_count);
        self.simulation.replace_birds(birds);
        self.refresh_stats_from_params();
    }

    fn update_stats(&mut self, dt: f32) {
        let frame_time_ms = dt * 1_000.0;
        let fps = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        self.stats.frame_time_ms = self.stats.frame_time_ms * 0.88 + frame_time_ms * 0.12;
        self.stats.fps = self.stats.fps * 0.88 + fps * 0.12;
        self.refresh_stats_from_params();
    }

    fn refresh_stats_from_params(&mut self) {
        let params = self.simulation.params();
        self.stats.bird_count = self.simulation.birds().len();
        self.stats.noise_weight = params.noise_weight;
        self.stats.boundary_radius = params.boundary_radius;
        self.stats.fog_density = params.fog_density;
    }
}

fn seed_birds(count: usize) -> Vec<Bird> {
    let mut birds = Vec::with_capacity(count);
    for index in 0..count {
        let t = index as f32 / count as f32;
        let radius = 2.0 + hash01(index as u32, 11).powf(2.2) * 13.0;
        let theta = hash01(index as u32, 23) * std::f32::consts::TAU;
        let phi = (hash01(index as u32, 37) * 2.0 - 1.0).acos();
        let dir = [
            phi.sin() * theta.cos(),
            phi.cos(),
            phi.sin() * theta.sin(),
        ];
        let x = dir[0] * radius;
        let y = dir[1] * radius * 0.18;
        let z = dir[2] * radius;
        let tangent = normalize3([-dir[2], 0.35 + dir[1] * 0.2, dir[0]]);
        let mut bird = Bird::new(
            [x, y, z],
            [tangent[0] * 2.4, tangent[1] * 2.4, tangent[2] * 2.4],
        );
        bird.seed = (index % 97) as f32 / 97.0;
        bird.phase = t * std::f32::consts::TAU;
        birds.push(bird);
    }
    birds
}

fn hash01(index: u32, salt: u32) -> f32 {
    let n = index.wrapping_mul(747_796_405).wrapping_add(salt.wrapping_mul(289_133_645));
    let x = ((n >> 8) ^ n) as f32;
    (x.sin() * 43_758.547).fract().abs()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= f32::EPSILON {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}
