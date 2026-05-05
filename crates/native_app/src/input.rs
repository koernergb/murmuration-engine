use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppAction {
    ApplyPreset(usize),
    AdjustBirdCount(i32),
    AdjustNoise(f32),
    AdjustBoundary(f32),
    AdjustFog(f32),
    ResetFlock,
    ToggleStats,
}

pub fn should_exit(event: &WindowEvent) -> bool {
    matches!(event, WindowEvent::CloseRequested)
        || matches!(
            event,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            }
        )
}

pub fn action_for_event(event: &WindowEvent) -> Option<AppAction> {
    let WindowEvent::KeyboardInput { event, .. } = event else {
        return None;
    };

    if event.state != ElementState::Pressed || event.repeat {
        return None;
    }

    match event.physical_key {
        PhysicalKey::Code(KeyCode::Digit1) => Some(AppAction::ApplyPreset(0)),
        PhysicalKey::Code(KeyCode::Digit2) => Some(AppAction::ApplyPreset(1)),
        PhysicalKey::Code(KeyCode::Digit3) => Some(AppAction::ApplyPreset(2)),
        PhysicalKey::Code(KeyCode::Digit4) => Some(AppAction::ApplyPreset(3)),
        PhysicalKey::Code(KeyCode::Equal) => Some(AppAction::AdjustBirdCount(5_000)),
        PhysicalKey::Code(KeyCode::Minus) => Some(AppAction::AdjustBirdCount(-5_000)),
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(AppAction::AdjustNoise(0.08)),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(AppAction::AdjustNoise(-0.08)),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(AppAction::AdjustBoundary(8.0)),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(AppAction::AdjustBoundary(-8.0)),
        PhysicalKey::Code(KeyCode::Period) => Some(AppAction::AdjustFog(0.003)),
        PhysicalKey::Code(KeyCode::Comma) => Some(AppAction::AdjustFog(-0.003)),
        PhysicalKey::Code(KeyCode::KeyR) => Some(AppAction::ResetFlock),
        PhysicalKey::Code(KeyCode::KeyH) => Some(AppAction::ToggleStats),
        _ => None,
    }
}
