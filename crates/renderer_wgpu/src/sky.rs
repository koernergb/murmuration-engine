#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkySettings {
    pub horizon_tint: [f32; 3],
    pub zenith_tint: [f32; 3],
}

impl Default for SkySettings {
    fn default() -> Self {
        Self {
            horizon_tint: [1.0, 0.65, 0.4],
            zenith_tint: [0.1, 0.2, 0.35],
        }
    }
}
