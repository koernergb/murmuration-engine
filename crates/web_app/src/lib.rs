use sim_core::FlockParams;

pub fn bootstrap_message() -> String {
    let params = FlockParams::default();

    format!(
        "web_app scaffold: default bird count {}",
        params.bird_count,
    )
}
