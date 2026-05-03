use sim_core::{Bird, FlockParams};

#[derive(Debug, Default)]
pub struct Renderer {
    frame_index: u64,
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub fn draw(&mut self, _birds: &[Bird], _params: &FlockParams) {
        self.frame_index = self.frame_index.saturating_add(1);
    }
}

