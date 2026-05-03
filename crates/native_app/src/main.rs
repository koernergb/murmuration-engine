mod app;
mod input;
mod ui;

use app::NativeApp;
use input::{action_for_event, should_exit};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title(ui::base_window_title())
        .with_inner_size(winit::dpi::LogicalSize::new(1440.0, 900.0))
        .build(&event_loop)?;
    let mut app = pollster::block_on(NativeApp::new(&window))?;
    window.set_title(&app.window_title());

    println!("{}", app.summary());

    event_loop.run(|event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                if should_exit(&event) {
                    target.exit();
                    return;
                }

                if let Some(action) = action_for_event(&event) {
                    app.handle_action(action);
                    window.set_title(&app.window_title());
                    return;
                }

                match event {
                    WindowEvent::Resized(size) => {
                        app.resize(size);
                        window.set_title(&app.window_title());
                    }
                    WindowEvent::RedrawRequested => {
                        app.update();
                        window.set_title(&app.window_title());
                        match app.render() {
                            Ok(()) => {}
                            Err(wgpu::SurfaceError::Lost) => app.resize(app.renderer_size()),
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Timeout) => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    })?;

    Ok(())
}
