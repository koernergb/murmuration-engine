use app_core::AppCommand;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub fn should_exit(event: &WindowEvent) -> bool {
    matches!(event, WindowEvent::CloseRequested)
        || matches!(
            event,
            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            }
        )
}

pub fn action_for_event(event: &WindowEvent) -> Option<AppCommand> {
    let WindowEvent::KeyboardInput { event, .. } = event else {
        return None;
    };

    if event.state != ElementState::Pressed || event.repeat {
        return None;
    }

    match event.physical_key {
        PhysicalKey::Code(KeyCode::Digit1) => Some(AppCommand::ApplyPreset(0)),
        PhysicalKey::Code(KeyCode::Digit2) => Some(AppCommand::ApplyPreset(1)),
        PhysicalKey::Code(KeyCode::Digit3) => Some(AppCommand::ApplyPreset(2)),
        PhysicalKey::Code(KeyCode::Digit4) => Some(AppCommand::ApplyPreset(3)),
        PhysicalKey::Code(KeyCode::Equal) => Some(AppCommand::AdjustBirdCount(5_000)),
        PhysicalKey::Code(KeyCode::Minus) => Some(AppCommand::AdjustBirdCount(-5_000)),
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(AppCommand::AdjustNoise(0.08)),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(AppCommand::AdjustNoise(-0.08)),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(AppCommand::AdjustBoundary(8.0)),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(AppCommand::AdjustBoundary(-8.0)),
        PhysicalKey::Code(KeyCode::Period) => Some(AppCommand::AdjustFog(0.003)),
        PhysicalKey::Code(KeyCode::Comma) => Some(AppCommand::AdjustFog(-0.003)),
        PhysicalKey::Code(KeyCode::KeyR) => Some(AppCommand::ResetFlock),
        PhysicalKey::Code(KeyCode::Space) => Some(AppCommand::TogglePaused),
        _ => None,
    }
}
