#[cfg(not(target_arch = "wasm32"))]
pub fn bootstrap_message() -> &'static str {
    "web_app is intended for the wasm32-unknown-unknown target"
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;
    use std::rc::Rc;

    use app_core::{preset_name, AppCommand, MurmurationApp};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{Document, HtmlButtonElement, HtmlCanvasElement, HtmlSelectElement};
    use winit::dpi::PhysicalSize;
    use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::platform::web::{EventLoopExtWebSys, WindowBuilderExtWebSys};
    use winit::window::{Window, WindowBuilder};

    const DEFAULT_BIRD_COUNT: u32 = 25_000;

    enum WebCommand {
        Core(AppCommand),
        Preset(usize),
        ToggleStats,
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);

        spawn_local(async {
            if let Err(error) = run().await {
                show_startup_error(&error);
                log::error!("web startup failed: {error}");
            }
        });

        Ok(())
    }

    async fn run() -> Result<(), String> {
        if !webgpu_available() {
            return Err("WebGPU is unavailable in this browser. Try a current version of Chrome, Edge, Firefox, or Safari with hardware acceleration enabled.".into());
        }

        set_status("Connecting to your GPU…");
        let document = document()?;
        let canvas = element::<HtmlCanvasElement>(&document, "murmuration-canvas")?;
        let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
        let initial_size = canvas_physical_size(&canvas);
        let window = WindowBuilder::new()
            .with_title("Murmuration Engine")
            .with_canvas(Some(canvas.clone()))
            .with_inner_size(initial_size)
            .build(&event_loop)
            .map_err(|error| error.to_string())?;
        let window: &'static Window = Box::leak(Box::new(window));

        set_status("Gathering the flock…");
        let app = MurmurationApp::new_with_bird_count(window, DEFAULT_BIRD_COUNT)
            .await
            .map_err(|error| error.to_string())?;
        let app = Rc::new(RefCell::new(app));
        let commands = Rc::new(RefCell::new(Vec::<WebCommand>::new()));
        let show_stats = Rc::new(RefCell::new(true));
        install_controls(&document, &commands)?;
        sync_canvas_size(&canvas, &mut app.borrow_mut());
        set_ready();

        let app_for_loop = app.clone();
        let commands_for_loop = commands.clone();
        let stats_for_loop = show_stats.clone();
        let canvas_for_loop = canvas.clone();
        let mut rendered_frames = 0_u64;

        event_loop.spawn(move |event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::CursorMoved { position, .. } => {
                            app_for_loop.borrow_mut().set_cursor_position(Some(position));
                        }
                        WindowEvent::CursorLeft { .. } => {
                            app_for_loop.borrow_mut().set_cursor_position(None);
                        }
                        WindowEvent::Resized(size) => {
                            app_for_loop.borrow_mut().resize(size);
                        }
                        WindowEvent::KeyboardInput { event, .. } => {
                            if let Some(command) = keyboard_command(&event) {
                                commands_for_loop.borrow_mut().push(command);
                            }
                        }
                        WindowEvent::RedrawRequested => {
                            let mut app = app_for_loop.borrow_mut();
                            app.update();
                            match app.render() {
                                Ok(()) => {
                                    rendered_frames = rendered_frames.saturating_add(1);
                                    if rendered_frames % 12 == 0 {
                                        update_stats(&app, *stats_for_loop.borrow());
                                    }
                                }
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    let size = app.renderer_size();
                                    app.resize(size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    show_runtime_error("The GPU ran out of memory. Try the Low quality setting, then reload the page.");
                                    target.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Other) => {}
                            }
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    if canvas_physical_size(&canvas_for_loop) != app_for_loop.borrow().renderer_size() {
                        sync_canvas_size(&canvas_for_loop, &mut app_for_loop.borrow_mut());
                    }

                    for command in commands_for_loop.borrow_mut().drain(..) {
                        let mut app = app_for_loop.borrow_mut();
                        match command {
                            WebCommand::Core(command) => app.handle_command(command),
                            WebCommand::Preset(index) => app.apply_preset(index, true),
                            WebCommand::ToggleStats => {
                                let mut visible = stats_for_loop.borrow_mut();
                                *visible = !*visible;
                                set_stats_visible(*visible);
                            }
                        }
                        update_controls(&app);
                    }
                    window.request_redraw();
                }
                _ => {}
            }
        });

        Ok(())
    }

    fn install_controls(
        document: &Document,
        commands: &Rc<RefCell<Vec<WebCommand>>>,
    ) -> Result<(), String> {
        bind_click(document, "pause-button", commands, || {
            WebCommand::Core(AppCommand::TogglePaused)
        })?;
        bind_click(document, "reset-button", commands, || {
            WebCommand::Core(AppCommand::ResetFlock)
        })?;
        bind_click(document, "stats-button", commands, || {
            WebCommand::ToggleStats
        })?;

        let quality = element::<HtmlSelectElement>(document, "quality-select")?;
        let queue = commands.clone();
        let quality_for_callback = quality.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            let count = match quality_for_callback.value().as_str() {
                "low" => 10_000,
                "high" => 50_000,
                _ => 25_000,
            };
            queue
                .borrow_mut()
                .push(WebCommand::Core(AppCommand::SetBirdCount(count)));
        });
        quality
            .add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        callback.forget();

        let presets = element::<HtmlSelectElement>(document, "preset-select")?;
        let queue = commands.clone();
        let presets_for_callback = presets.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            if let Ok(index) = presets_for_callback.value().parse::<usize>() {
                queue.borrow_mut().push(WebCommand::Preset(index));
            }
        });
        presets
            .add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        callback.forget();

        Ok(())
    }

    fn bind_click<F>(
        document: &Document,
        id: &str,
        commands: &Rc<RefCell<Vec<WebCommand>>>,
        command: F,
    ) -> Result<(), String>
    where
        F: Fn() -> WebCommand + 'static,
    {
        let button = element::<HtmlButtonElement>(document, id)?;
        let queue = commands.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            queue.borrow_mut().push(command());
        });
        button
            .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        callback.forget();
        Ok(())
    }

    fn keyboard_command(event: &KeyEvent) -> Option<WebCommand> {
        if event.state != ElementState::Pressed || event.repeat {
            return None;
        }

        match event.physical_key {
            PhysicalKey::Code(KeyCode::Space) => Some(WebCommand::Core(AppCommand::TogglePaused)),
            PhysicalKey::Code(KeyCode::KeyR) => Some(WebCommand::Core(AppCommand::ResetFlock)),
            PhysicalKey::Code(KeyCode::Digit1) => Some(WebCommand::Preset(0)),
            PhysicalKey::Code(KeyCode::Digit2) => Some(WebCommand::Preset(1)),
            PhysicalKey::Code(KeyCode::Digit3) => Some(WebCommand::Preset(2)),
            PhysicalKey::Code(KeyCode::Digit4) => Some(WebCommand::Preset(3)),
            _ => None,
        }
    }

    fn sync_canvas_size(canvas: &HtmlCanvasElement, app: &mut MurmurationApp<'_>) {
        let size = canvas_physical_size(canvas);
        canvas.set_width(size.width);
        canvas.set_height(size.height);
        let _ = canvas.set_attribute("style", "width: 100%; height: 100%;");
        app.resize(size);
    }

    fn canvas_physical_size(canvas: &HtmlCanvasElement) -> PhysicalSize<u32> {
        let ratio = web_sys::window()
            .map(|window| window.device_pixel_ratio())
            .unwrap_or(1.0);
        PhysicalSize::new(
            ((canvas.client_width() as f64 * ratio).round() as u32).max(1),
            ((canvas.client_height() as f64 * ratio).round() as u32).max(1),
        )
    }

    fn update_stats(app: &MurmurationApp<'_>, visible: bool) {
        if !visible {
            return;
        }
        let stats = app.stats();
        set_text("fps-value", &format!("{:.0}", stats.fps));
        set_text("birds-value", &format_count(stats.bird_count));
        set_text("preset-value", preset_name(app.active_preset()));
    }

    fn update_controls(app: &MurmurationApp<'_>) {
        if let Ok(document) = document() {
            if let Ok(button) = element::<HtmlButtonElement>(&document, "pause-button") {
                button.set_text_content(Some(if app.is_paused() { "Resume" } else { "Pause" }));
            }
            if let Ok(select) = element::<HtmlSelectElement>(&document, "preset-select") {
                select.set_value(&app.active_preset().to_string());
            }
        }
    }

    fn set_ready() {
        if let Ok(document) = document() {
            if let Some(shell) = document.get_element_by_id("experience") {
                shell.set_class_name("experience is-ready");
            }
            if let Some(loading) = document.get_element_by_id("loading-state") {
                loading.set_class_name("loading-state is-hidden");
            }
        }
    }

    fn set_status(message: &str) {
        set_text("loading-message", message);
    }

    fn set_stats_visible(visible: bool) {
        if let Ok(document) = document() {
            if let Some(stats) = document.get_element_by_id("stats") {
                stats.set_class_name(if visible { "stats" } else { "stats is-hidden" });
            }
        }
    }

    fn show_startup_error(message: &str) {
        show_error("Unable to start the murmuration", message);
    }

    fn show_runtime_error(message: &str) {
        show_error("The murmuration stopped", message);
    }

    fn show_error(title: &str, message: &str) {
        if let Ok(document) = document() {
            set_text("error-title", title);
            set_text("error-message", message);
            if let Some(panel) = document.get_element_by_id("error-state") {
                panel.set_class_name("error-state is-visible");
            }
            if let Some(loading) = document.get_element_by_id("loading-state") {
                loading.set_class_name("loading-state is-hidden");
            }
        }
    }

    fn set_text(id: &str, value: &str) {
        if let Ok(document) = document() {
            if let Some(element) = document.get_element_by_id(id) {
                element.set_text_content(Some(value));
            }
        }
    }

    fn webgpu_available() -> bool {
        js_sys::Reflect::has(
            &web_sys::window().unwrap().navigator(),
            &JsValue::from_str("gpu"),
        )
        .unwrap_or(false)
    }

    fn document() -> Result<Document, String> {
        web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| "The page document is unavailable.".into())
    }

    fn element<T: JsCast>(document: &Document, id: &str) -> Result<T, String> {
        document
            .get_element_by_id(id)
            .ok_or_else(|| format!("Required page element #{id} is missing."))?
            .dyn_into::<T>()
            .map_err(|_| format!("Page element #{id} has the wrong type."))
    }

    fn js_error(error: JsValue) -> String {
        error
            .as_string()
            .unwrap_or_else(|| "A browser API call failed.".into())
    }

    fn format_count(count: usize) -> String {
        if count >= 1_000 {
            format!("{}k", count / 1_000)
        } else {
            count.to_string()
        }
    }
}
