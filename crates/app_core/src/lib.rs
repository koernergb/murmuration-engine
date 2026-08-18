use std::error::Error;
use std::sync::Arc;

use renderer_wgpu::{camera::Camera, RenderPalette, Renderer};
use serde::Deserialize;
use sim_core::{Bird, FlockParams, SimulationState};
use web_time::Instant;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::Window,
};

const PRESET_FILES: [(&str, &str); 4] = [
    (
        "Calm Cloud",
        include_str!("../../../examples/presets/calm.json"),
    ),
    (
        "Predator Ripple",
        include_str!("../../../examples/presets/predator_wave.json"),
    ),
    (
        "Ribbon Sheet",
        include_str!("../../../examples/presets/ribbon_flock.json"),
    ),
    (
        "Storm Column",
        include_str!("../../../examples/presets/storm_column.json"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppCommand {
    ApplyPreset(usize),
    AdjustBirdCount(i32),
    SetBirdCount(u32),
    AdjustNoise(f32),
    AdjustBoundary(f32),
    AdjustFog(f32),
    ResetFlock,
    TogglePaused,
}

#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub bird_count: usize,
    pub noise_weight: f32,
    pub boundary_radius: f32,
    pub fog_density: f32,
}

pub struct MurmurationApp<'window> {
    renderer: Renderer<'window>,
    simulation: SimulationState,
    last_frame: Instant,
    stats: RuntimeStats,
    cursor_position: Option<PhysicalPosition<f64>>,
    guide_position: Option<(PhysicalPosition<f64>, f32)>,
    camera: Camera,
    active_preset: usize,
    paused: bool,
}

impl<'window> MurmurationApp<'window> {
    pub async fn new(window: &'window Window) -> Result<Self, Box<dyn Error>> {
        Self::new_with_bird_count(window, FlockParams::default().bird_count).await
    }

    pub async fn new_with_bird_count(
        window: &'window Window,
        bird_count: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let mut params = FlockParams::default();
        params.bird_count = bird_count.clamp(1_000, 100_000);
        let birds = seed_birds(params.bird_count as usize);
        let stats = stats_for(&params, birds.len());

        Ok(Self {
            renderer: Renderer::new(window, birds.len()).await?,
            simulation: SimulationState::new(birds, params),
            last_frame: Instant::now(),
            stats,
            cursor_position: None,
            guide_position: None,
            camera: Camera::default(),
            active_preset: 0,
            paused: false,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.renderer.resize(size);
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame)
            .as_secs_f32()
            .clamp(1.0 / 240.0, 1.0 / 24.0);
        self.last_frame = now;

        if !self.paused {
            let cursor_world = self.cursor_position.and_then(|cursor| {
                self.camera
                    .screen_to_focus_point(
                        self.renderer.size(),
                        cursor,
                        self.simulation.time_seconds(),
                    )
                    .map(|world| world.to_array())
            });
            self.simulation.set_cursor_repulsor(cursor_world);
            let guide_world = self.guide_position.and_then(|(position, strength)| {
                self.camera
                    .screen_to_focus_point(
                        self.renderer.size(),
                        position,
                        self.simulation.time_seconds(),
                    )
                    .map(|world| {
                        let mut target = world.to_array();
                        target[0] *= strength;
                        target[1] *= strength;
                        target[2] *= strength;
                        target
                    })
            });
            self.simulation.set_guide_target(guide_world);
            self.simulation.step(dt);
        }

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

    pub fn set_cursor_position(&mut self, cursor_position: Option<PhysicalPosition<f64>>) {
        self.cursor_position = cursor_position;
    }

    pub fn set_guide_position(&mut self, guide_position: Option<(PhysicalPosition<f64>, f32)>) {
        self.guide_position = guide_position;
    }

    pub fn handle_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::ApplyPreset(index) => self.apply_preset(index, false),
            AppCommand::AdjustBirdCount(delta) => {
                let count = (self.simulation.params().bird_count as i32 + delta)
                    .clamp(1_000, 100_000) as u32;
                self.set_bird_count(count);
            }
            AppCommand::SetBirdCount(count) => self.set_bird_count(count),
            AppCommand::AdjustNoise(delta) => {
                let params = self.simulation.params_mut();
                params.noise_weight = (params.noise_weight + delta).clamp(0.0, 2.5);
            }
            AppCommand::AdjustBoundary(delta) => {
                let params = self.simulation.params_mut();
                params.boundary_radius = (params.boundary_radius + delta).clamp(16.0, 56.0);
            }
            AppCommand::AdjustFog(delta) => {
                let params = self.simulation.params_mut();
                params.fog_density = (params.fog_density + delta).clamp(0.0, 0.08);
            }
            AppCommand::ResetFlock => self.reseed_birds(),
            AppCommand::TogglePaused => self.paused = !self.paused,
        }

        self.refresh_stats_from_params();
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        self.last_frame = Instant::now();
    }

    pub fn set_palette(&mut self, palette: RenderPalette) {
        self.renderer.set_palette(palette);
    }

    pub fn configure_low_power_background(&mut self) {
        self.apply_preset(0, true);
        let params = self.simulation.params_mut();
        params.noise_weight = 0.18;
        params.noise_speed = 0.12;
        params.cursor_weight = 6.0;
        params.cursor_radius_ratio = 0.18;
        params.max_speed = params.max_speed.min(3.2);
        params.max_turn_rate = params.max_turn_rate.min(2.1);
        params.fog_density = 0.045;
        self.refresh_stats_from_params();
    }

    pub fn apply_preset(&mut self, index: usize, preserve_bird_count: bool) {
        if index >= PRESET_FILES.len() {
            return;
        }

        let bird_count = self.simulation.params().bird_count;
        if let Ok(patch) = serde_json::from_str::<PresetPatch>(PRESET_FILES[index].1) {
            patch.apply(self.simulation.params_mut());
            if preserve_bird_count {
                self.simulation.params_mut().bird_count = bird_count;
            }
            self.active_preset = index;
            self.reseed_birds();
        }
    }

    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }

    pub fn active_preset(&self) -> usize {
        self.active_preset
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn summary(&self) -> String {
        format!(
            "murmuration engine: {} birds, frame {}",
            self.simulation.birds().len(),
            self.renderer.frame_index()
        )
    }

    fn set_bird_count(&mut self, count: u32) {
        self.simulation.params_mut().bird_count = count.clamp(1_000, 100_000);
        self.reseed_birds();
    }

    fn reseed_birds(&mut self) {
        let birds = seed_birds(self.simulation.params().bird_count as usize);
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

impl MurmurationApp<'static> {
    pub async fn new_owned_with_bird_count(
        window: Arc<Window>,
        bird_count: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let mut params = FlockParams::default();
        params.bird_count = bird_count.clamp(1_000, 100_000);
        let birds = seed_birds(params.bird_count as usize);
        let stats = stats_for(&params, birds.len());
        let renderer = Renderer::new_owned(window, birds.len()).await?;

        Ok(Self {
            renderer,
            simulation: SimulationState::new(birds, params),
            last_frame: Instant::now(),
            stats,
            cursor_position: None,
            guide_position: None,
            camera: Camera::default(),
            active_preset: 0,
            paused: false,
        })
    }
}

pub fn preset_count() -> usize {
    PRESET_FILES.len()
}

pub fn preset_name(index: usize) -> &'static str {
    PRESET_FILES
        .get(index)
        .map(|(name, _)| *name)
        .unwrap_or("Unknown")
}

fn stats_for(params: &FlockParams, bird_count: usize) -> RuntimeStats {
    RuntimeStats {
        fps: 0.0,
        frame_time_ms: 0.0,
        bird_count,
        noise_weight: params.noise_weight,
        boundary_radius: params.boundary_radius,
        fog_density: params.fog_density,
    }
}

#[derive(Debug, Deserialize)]
struct PresetPatch {
    bird_count: Option<u32>,
    alignment_weight: Option<f32>,
    boundary_radius: Option<f32>,
    cohesion_weight: Option<f32>,
    fear_spread: Option<f32>,
    name: Option<String>,
    noise_weight: Option<f32>,
    predator_weight: Option<f32>,
}

impl PresetPatch {
    fn apply(self, params: &mut FlockParams) {
        if let Some(value) = self.bird_count {
            params.bird_count = value;
        }
        if let Some(value) = self.alignment_weight {
            params.alignment_weight = value;
        }
        if let Some(value) = self.boundary_radius {
            params.boundary_radius = value;
        }
        if let Some(value) = self.cohesion_weight {
            params.cohesion_weight = value;
        }
        if let Some(value) = self.fear_spread {
            params.fear_spread = value;
        }
        if let Some(value) = self.noise_weight {
            params.noise_weight = value;
        }
        if let Some(value) = self.predator_weight {
            params.predator_weight = value;
        }
        let _ = self.name;
    }
}

fn seed_birds(count: usize) -> Vec<Bird> {
    let mut birds = Vec::with_capacity(count);
    for index in 0..count {
        let t = index as f32 / count as f32;
        let radius = 2.0 + hash01(index as u32, 11).powf(2.2) * 13.0;
        let theta = hash01(index as u32, 23) * std::f32::consts::TAU;
        let phi = (hash01(index as u32, 37) * 2.0 - 1.0).acos();
        let dir = [phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin()];
        let position = [dir[0] * radius, dir[1] * radius * 0.18, dir[2] * radius];
        let tangent = normalize3([-dir[2], 0.35 + dir[1] * 0.2, dir[0]]);
        let mut bird = Bird::new(
            position,
            [tangent[0] * 2.4, tangent[1] * 2.4, tangent[2] * 2.4],
        );
        bird.seed = (index % 97) as f32 / 97.0;
        bird.phase = t * std::f32::consts::TAU;
        birds.push(bird);
    }
    birds
}

fn hash01(index: u32, salt: u32) -> f32 {
    let n = index
        .wrapping_mul(747_796_405)
        .wrapping_add(salt.wrapping_mul(289_133_645));
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
