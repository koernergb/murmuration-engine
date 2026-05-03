mod app;
mod input;
mod ui;

use app::NativeApp;
use input::InputState;
use ui::UiState;

fn main() {
    let input = InputState::default();
    let ui = UiState::default();
    let mut app = NativeApp::new();
    app.tick(1.0 / 60.0);
    println!(
        "{} | orbit_enabled={} | show_stats={} | default_bird_count={}",
        app.summary(),
        input.orbit_enabled,
        ui.show_stats,
        ui.params.bird_count
    );
}
