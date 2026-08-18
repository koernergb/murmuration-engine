use app_core::{preset_name, RuntimeStats};

pub fn base_window_title() -> &'static str {
    "Murmuration Engine | Web MVP"
}

pub fn window_title(active_preset: usize, stats: &RuntimeStats, paused: bool) -> String {
    format!(
        "{} | {}{} | {:>5.1} FPS | {:>5.1} ms | {:>5} birds | noise {:.2} | boundary {:.0} | fog {:.3} | keys: 1-4 presets, +/- birds, arrows tune, </> fog, R reset, Space pause",
        base_window_title(),
        preset_name(active_preset),
        if paused { " | PAUSED" } else { "" },
        stats.fps,
        stats.frame_time_ms,
        stats.bird_count,
        stats.noise_weight,
        stats.boundary_radius,
        stats.fog_density,
    )
}
