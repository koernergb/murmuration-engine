use sim_core::FlockParams;

#[derive(Debug, Clone)]
pub struct UiState {
    pub params: FlockParams,
    pub show_stats: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            params: FlockParams::default(),
            show_stats: true,
        }
    }
}

