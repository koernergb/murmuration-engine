use serde::Deserialize;
use sim_core::FlockParams;

const PRESET_FILES: [(&str, &str); 4] = [
    ("Calm Cloud", include_str!("../../../examples/presets/calm.json")),
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

pub fn base_window_title() -> &'static str {
    "Murmuration Engine | Phase 3"
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub show_stats: bool,
    pub active_preset: usize,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_stats: true,
            active_preset: 0,
        }
    }
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

pub fn preset_count() -> usize {
    PRESET_FILES.len()
}

pub fn preset_name(index: usize) -> &'static str {
    PRESET_FILES
        .get(index)
        .map(|(name, _)| *name)
        .unwrap_or("Unknown")
}

pub fn apply_preset(params: &mut FlockParams, preset_index: usize) -> Result<(), serde_json::Error> {
    let (_, json) = PRESET_FILES[preset_index];
    let patch: PresetPatch = serde_json::from_str(json)?;
    patch.apply(params);
    Ok(())
}

pub fn window_title(ui: &UiState, stats: &RuntimeStats) -> String {
    if !ui.show_stats {
        return base_window_title().to_string();
    }

    format!(
        "{} | {} | {:>5.1} FPS | {:>5.1} ms | {:>5} birds | noise {:.2} | boundary {:.0} | fog {:.3} | keys: 1-4 presets, +/- birds, arrows tune, </> fog, R reset, H hide",
        base_window_title(),
        preset_name(ui.active_preset),
        stats.fps,
        stats.frame_time_ms,
        stats.bird_count,
        stats.noise_weight,
        stats.boundary_radius,
        stats.fog_density,
    )
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
        if let Some(bird_count) = self.bird_count {
            params.bird_count = bird_count;
        }
        if let Some(alignment_weight) = self.alignment_weight {
            params.alignment_weight = alignment_weight;
        }
        if let Some(boundary_radius) = self.boundary_radius {
            params.boundary_radius = boundary_radius;
        }
        if let Some(cohesion_weight) = self.cohesion_weight {
            params.cohesion_weight = cohesion_weight;
        }
        if let Some(fear_spread) = self.fear_spread {
            params.fear_spread = fear_spread;
        }
        if let Some(noise_weight) = self.noise_weight {
            params.noise_weight = noise_weight;
        }
        if let Some(predator_weight) = self.predator_weight {
            params.predator_weight = predator_weight;
        }
        let _ = self.name;
    }
}
