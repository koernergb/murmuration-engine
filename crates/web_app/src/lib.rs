use renderer_wgpu::Renderer;
use sim_core::FlockParams;

pub fn bootstrap_message() -> String {
    let renderer = Renderer::new();
    let params = FlockParams::default();

    format!(
        "web_app scaffold: renderer frame {}, default bird count {}",
        renderer.frame_index(),
        params.bird_count
    )
}

