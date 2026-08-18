#[cfg(not(target_arch = "wasm32"))]
pub fn bootstrap_message() -> &'static str {
    "web_app is intended for the wasm32-unknown-unknown target"
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::{Cell, RefCell};
    use std::rc::{Rc, Weak};
    use std::sync::Arc;

    use app_core::{preset_name, AppCommand, MurmurationApp};
    use renderer_wgpu::RenderPalette;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{
        Document, EventTarget, HtmlButtonElement, HtmlCanvasElement, HtmlSelectElement,
        PointerEvent,
    };
    use winit::dpi::{PhysicalPosition, PhysicalSize};
    use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::platform::web::{EventLoopExtWebSys, WindowBuilderExtWebSys};
    use winit::window::{Window, WindowBuilder};

    thread_local! {
        static ACTIVE_RUNTIME: RefCell<Option<Weak<RuntimeControl>>> = const { RefCell::new(None) };
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum EngineMode {
        Standalone,
        Background,
    }

    #[derive(Debug, Clone)]
    struct MountConfig {
        mode: EngineMode,
        bird_count: u32,
        preset: usize,
        pointer_enabled: bool,
        reduced_motion: bool,
        palette: RenderPalette,
    }

    impl MountConfig {
        fn standalone() -> Self {
            Self {
                mode: EngineMode::Standalone,
                bird_count: 25_000,
                preset: 0,
                pointer_enabled: true,
                reduced_motion: false,
                palette: RenderPalette::STANDALONE,
            }
        }

        fn background(options: &JsValue) -> Self {
            let mut palette = RenderPalette::WARM_EDITORIAL;
            if option_bool(options, "transparent").unwrap_or(false) {
                palette = palette.with_background_alpha(0.0);
            }
            if let Some(value) = option_value(options, "palette") {
                palette = palette_from_js(&value, palette);
            }

            Self {
                mode: EngineMode::Background,
                bird_count: quality_bird_count(
                    option_string(options, "quality")
                        .as_deref()
                        .unwrap_or("low"),
                ),
                preset: option_string(options, "preset")
                    .as_deref()
                    .and_then(preset_index)
                    .unwrap_or(0),
                pointer_enabled: option_bool(options, "pointerInteraction").unwrap_or(true),
                reduced_motion: option_bool(options, "reducedMotion").unwrap_or(false),
                palette,
            }
        }
    }

    enum EngineCommand {
        SetPaused(bool),
        Resize(PhysicalSize<u32>),
        SetQuality(u32),
        SetPreset(usize),
        Reset,
        SetPointerEnabled(bool),
        SetPalette(RenderPalette),
        Cursor(Option<PhysicalPosition<f64>>),
        GuideTarget(Option<(PhysicalPosition<f64>, f32)>),
        Dispose,
    }

    struct ListenerRegistration {
        target: EventTarget,
        event_name: &'static str,
        callback: Closure<dyn FnMut(web_sys::Event)>,
    }

    impl ListenerRegistration {
        fn remove(self) {
            let _ = self.target.remove_event_listener_with_callback(
                self.event_name,
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }

    struct RuntimeControl {
        window: Arc<Window>,
        canvas: HtmlCanvasElement,
        commands: RefCell<Vec<EngineCommand>>,
        listeners: RefCell<Vec<ListenerRegistration>>,
        pointer_enabled: Cell<bool>,
        disposed: Cell<bool>,
    }

    impl RuntimeControl {
        fn enqueue(&self, command: EngineCommand) {
            if !self.disposed.get() {
                self.commands.borrow_mut().push(command);
                self.window.request_redraw();
            }
        }

        fn dispose(&self) {
            if self.disposed.replace(true) {
                return;
            }
            for listener in self.listeners.borrow_mut().drain(..) {
                listener.remove();
            }
            self.commands.borrow_mut().push(EngineCommand::Dispose);
            self.window.request_redraw();
        }
    }

    #[derive(Clone)]
    struct EngineHandle {
        runtime: Rc<RuntimeControl>,
    }

    impl EngineHandle {
        fn resize(&self, width: u32, height: u32) {
            let size = PhysicalSize::new(width.max(1), height.max(1));
            self.runtime.canvas.set_width(size.width);
            self.runtime.canvas.set_height(size.height);
            self.runtime.enqueue(EngineCommand::Resize(size));
        }

        fn pause(&self) {
            self.runtime.enqueue(EngineCommand::SetPaused(true));
        }

        fn resume(&self) {
            self.runtime.enqueue(EngineCommand::SetPaused(false));
        }

        fn set_quality(&self, quality: &str) {
            self.runtime
                .enqueue(EngineCommand::SetQuality(quality_bird_count(quality)));
        }

        fn set_preset(&self, preset: &str) -> Result<(), JsValue> {
            let index = preset_index(preset)
                .ok_or_else(|| JsValue::from_str("Unknown Murmuration Engine preset."))?;
            self.runtime.enqueue(EngineCommand::SetPreset(index));
            Ok(())
        }

        fn set_pointer_enabled(&self, enabled: bool) {
            self.runtime.pointer_enabled.set(enabled);
            self.runtime
                .enqueue(EngineCommand::SetPointerEnabled(enabled));
        }

        fn set_palette(&self, value: &JsValue) {
            let palette = palette_from_js(value, RenderPalette::WARM_EDITORIAL);
            self.runtime.enqueue(EngineCommand::SetPalette(palette));
        }

        fn set_guide_target(&self, x: f64, y: f64, strength: f32) {
            let position = PhysicalPosition::new(
                x.clamp(0.0, 1.0) * self.runtime.canvas.width() as f64,
                y.clamp(0.0, 1.0) * self.runtime.canvas.height() as f64,
            );
            self.runtime.enqueue(EngineCommand::GuideTarget(Some((
                position,
                strength.clamp(0.0, 1.0),
            ))));
        }

        fn dispose(&self) {
            self.runtime.dispose();
        }
    }

    #[wasm_bindgen]
    pub struct WasmMurmurationController {
        handle: EngineHandle,
    }

    #[wasm_bindgen]
    impl WasmMurmurationController {
        #[wasm_bindgen(js_name = resize)]
        pub fn resize(&self, width: u32, height: u32) {
            self.handle.resize(width, height);
        }

        #[wasm_bindgen(js_name = pause)]
        pub fn pause(&self) {
            self.handle.pause();
        }

        #[wasm_bindgen(js_name = resume)]
        pub fn resume(&self) {
            self.handle.resume();
        }

        #[wasm_bindgen(js_name = setQuality)]
        pub fn set_quality(&self, quality: &str) {
            self.handle.set_quality(quality);
        }

        #[wasm_bindgen(js_name = setPreset)]
        pub fn set_preset(&self, preset: &str) -> Result<(), JsValue> {
            self.handle.set_preset(preset)
        }

        #[wasm_bindgen(js_name = setPointerInteraction)]
        pub fn set_pointer_interaction(&self, enabled: bool) {
            self.handle.set_pointer_enabled(enabled);
        }

        #[wasm_bindgen(js_name = setPalette)]
        pub fn set_palette(&self, palette: JsValue) {
            self.handle.set_palette(&palette);
        }

        #[wasm_bindgen(js_name = setGuideTarget)]
        pub fn set_guide_target(&self, x: f64, y: f64, strength: f32) {
            self.handle.set_guide_target(x, y, strength);
        }

        #[wasm_bindgen(js_name = dispose)]
        pub fn dispose(&self) {
            self.handle.dispose();
        }

        #[wasm_bindgen(getter, js_name = disposed)]
        pub fn disposed(&self) -> bool {
            self.handle.runtime.disposed.get()
        }
    }

    #[wasm_bindgen(js_name = mountBackground)]
    pub async fn mount_background(
        canvas: HtmlCanvasElement,
        options: JsValue,
    ) -> Result<WasmMurmurationController, JsValue> {
        let handle = mount_engine(canvas, MountConfig::background(&options))
            .await
            .map_err(|error| JsValue::from_str(&error))?;
        Ok(WasmMurmurationController { handle })
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);

        let Ok(document) = document() else {
            return Ok(());
        };
        let Some(element) = document.get_element_by_id("murmuration-canvas") else {
            return Ok(());
        };
        let Ok(canvas) = element.dyn_into::<HtmlCanvasElement>() else {
            return Ok(());
        };

        spawn_local(async move {
            set_status("Connecting to your GPU…");
            match mount_engine(canvas, MountConfig::standalone()).await {
                Ok(handle) => {
                    if let Err(error) = install_standalone_controls(&document, &handle) {
                        show_startup_error(&error);
                        handle.dispose();
                        return;
                    }
                    set_ready();
                }
                Err(error) => {
                    show_startup_error(&error);
                    log::error!("web startup failed: {error}");
                }
            }
        });

        Ok(())
    }

    async fn mount_engine(
        canvas: HtmlCanvasElement,
        config: MountConfig,
    ) -> Result<EngineHandle, String> {
        if !webgpu_available() {
            return Err("WebGPU is unavailable in this browser.".into());
        }

        let already_active = ACTIVE_RUNTIME
            .with(|active| active.borrow().as_ref().and_then(Weak::upgrade).is_some());
        if already_active {
            return Err("Only one Murmuration Engine instance may be mounted at a time.".into());
        }

        let document = document()?;
        let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
        let initial_size = canvas_physical_size(&canvas);
        let window = WindowBuilder::new()
            .with_title("Murmuration Engine")
            .with_canvas(Some(canvas.clone()))
            .with_inner_size(initial_size)
            .build(&event_loop)
            .map_err(|error| error.to_string())?;
        let window = Arc::new(window);

        if config.mode == EngineMode::Standalone {
            set_status("Gathering the flock…");
        }
        let mut app = MurmurationApp::new_owned_with_bird_count(window.clone(), config.bird_count)
            .await
            .map_err(|error| error.to_string())?;
        if config.mode == EngineMode::Background {
            app.configure_low_power_background();
        }
        if config.preset != 0 {
            app.apply_preset(config.preset, true);
        }
        app.set_palette(config.palette);
        app.set_paused(config.reduced_motion);
        sync_canvas_size(&canvas, &mut app);

        let runtime = Rc::new(RuntimeControl {
            window: window.clone(),
            canvas: canvas.clone(),
            commands: RefCell::new(Vec::new()),
            listeners: RefCell::new(Vec::new()),
            pointer_enabled: Cell::new(config.pointer_enabled),
            disposed: Cell::new(false),
        });
        install_pointer_listener(&document, &runtime)?;
        install_visibility_listener(&document, &runtime)?;
        ACTIVE_RUNTIME.with(|active| *active.borrow_mut() = Some(Rc::downgrade(&runtime)));

        let runtime_for_loop = runtime.clone();
        let mode = config.mode;
        let mut requested_paused = config.reduced_motion;
        let mut rendered_frames = 0_u64;

        event_loop.spawn(move |event, target| {
            let document_hidden = document.hidden();

            match event {
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::Resized(size) => app.resize(size),
                        WindowEvent::KeyboardInput { event, .. }
                            if mode == EngineMode::Standalone =>
                        {
                            if let Some(command) = keyboard_command(&event) {
                                apply_command(
                                    command,
                                    &mut app,
                                    &runtime_for_loop,
                                    &mut requested_paused,
                                );
                            }
                        }
                        WindowEvent::RedrawRequested if !document_hidden => {
                            app.set_paused(requested_paused);
                            app.update();
                            match app.render() {
                                Ok(()) => {
                                    rendered_frames = rendered_frames.saturating_add(1);
                                    if mode == EngineMode::Standalone && rendered_frames % 12 == 0 {
                                        update_stats(&app);
                                    }
                                }
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    let size = app.renderer_size();
                                    app.resize(size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    if mode == EngineMode::Standalone {
                                        show_runtime_error("The GPU ran out of memory. Try the Low quality setting, then reload the page.");
                                    }
                                    runtime_for_loop.dispose();
                                }
                                Err(wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Other) => {}
                            }
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    let commands: Vec<_> = runtime_for_loop.commands.borrow_mut().drain(..).collect();
                    let mut should_exit = false;
                    let mut needs_render = false;
                    for command in commands {
                        if matches!(command, EngineCommand::Dispose) {
                            should_exit = true;
                        } else {
                            apply_command(
                                command,
                                &mut app,
                                &runtime_for_loop,
                                &mut requested_paused,
                            );
                            needs_render = true;
                        }
                    }

                    if should_exit {
                        ACTIVE_RUNTIME.with(|active| *active.borrow_mut() = None);
                        target.exit();
                        return;
                    }

                    let idle = document_hidden || requested_paused;
                    target.set_control_flow(if idle {
                        ControlFlow::Wait
                    } else {
                        ControlFlow::Poll
                    });
                    if !document_hidden && (!idle || needs_render) {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        });

        Ok(EngineHandle { runtime })
    }

    fn apply_command(
        command: EngineCommand,
        app: &mut MurmurationApp<'static>,
        runtime: &RuntimeControl,
        requested_paused: &mut bool,
    ) {
        match command {
            EngineCommand::SetPaused(paused) => {
                *requested_paused = paused;
                app.set_paused(paused);
            }
            EngineCommand::Resize(size) => app.resize(size),
            EngineCommand::SetQuality(count) => {
                app.handle_command(AppCommand::SetBirdCount(count));
            }
            EngineCommand::SetPreset(index) => app.apply_preset(index, true),
            EngineCommand::Reset => app.handle_command(AppCommand::ResetFlock),
            EngineCommand::SetPointerEnabled(enabled) => {
                runtime.pointer_enabled.set(enabled);
                if !enabled {
                    app.set_cursor_position(None);
                }
            }
            EngineCommand::SetPalette(palette) => app.set_palette(palette),
            EngineCommand::Cursor(position) => app.set_cursor_position(position),
            EngineCommand::GuideTarget(position) => app.set_guide_position(position),
            EngineCommand::Dispose => {}
        }

        if runtime.canvas.id() == "murmuration-canvas" {
            update_controls(app);
        }
    }

    fn install_pointer_listener(
        document: &Document,
        runtime: &Rc<RuntimeControl>,
    ) -> Result<(), String> {
        let target: EventTarget = document.clone().unchecked_into();
        let runtime_for_pointer = runtime.clone();
        let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            if !runtime_for_pointer.pointer_enabled.get() {
                return;
            }
            let Ok(pointer) = event.dyn_into::<PointerEvent>() else {
                return;
            };
            let rect = runtime_for_pointer.canvas.get_bounding_client_rect();
            let x = pointer.client_x() as f64 - rect.left();
            let y = pointer.client_y() as f64 - rect.top();
            let inside = x >= 0.0 && y >= 0.0 && x <= rect.width() && y <= rect.height();
            let position = inside.then(|| {
                let scale_x = runtime_for_pointer.canvas.width() as f64 / rect.width().max(1.0);
                let scale_y = runtime_for_pointer.canvas.height() as f64 / rect.height().max(1.0);
                PhysicalPosition::new(x * scale_x, y * scale_y)
            });
            runtime_for_pointer.enqueue(EngineCommand::Cursor(position));
        });
        target
            .add_event_listener_with_callback("pointermove", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        runtime.listeners.borrow_mut().push(ListenerRegistration {
            target,
            event_name: "pointermove",
            callback,
        });
        Ok(())
    }

    fn install_visibility_listener(
        document: &Document,
        runtime: &Rc<RuntimeControl>,
    ) -> Result<(), String> {
        let target: EventTarget = document.clone().unchecked_into();
        let runtime_for_visibility = runtime.clone();
        let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |_event| {
            runtime_for_visibility.window.request_redraw();
        });
        target
            .add_event_listener_with_callback("visibilitychange", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        runtime.listeners.borrow_mut().push(ListenerRegistration {
            target,
            event_name: "visibilitychange",
            callback,
        });
        Ok(())
    }

    fn install_standalone_controls(
        document: &Document,
        handle: &EngineHandle,
    ) -> Result<(), String> {
        bind_click(document, "pause-button", handle, |handle| {
            handle
                .runtime
                .enqueue(EngineCommand::SetPaused(!button_says_resume()));
        })?;
        bind_click(document, "reset-button", handle, |handle| {
            handle.runtime.enqueue(EngineCommand::Reset);
        })?;
        bind_click(document, "stats-button", handle, |_handle| {
            toggle_stats();
        })?;

        let quality = element::<HtmlSelectElement>(document, "quality-select")?;
        let handle_for_quality = handle.clone();
        let quality_for_callback = quality.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            handle_for_quality.set_quality(&quality_for_callback.value());
        });
        quality
            .add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        callback.forget();

        let presets = element::<HtmlSelectElement>(document, "preset-select")?;
        let handle_for_preset = handle.clone();
        let presets_for_callback = presets.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
            if let Ok(index) = presets_for_callback.value().parse::<usize>() {
                handle_for_preset
                    .runtime
                    .enqueue(EngineCommand::SetPreset(index));
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
        handle: &EngineHandle,
        action: F,
    ) -> Result<(), String>
    where
        F: Fn(&EngineHandle) + 'static,
    {
        let button = element::<HtmlButtonElement>(document, id)?;
        let handle = handle.clone();
        let callback = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| action(&handle));
        button
            .add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())
            .map_err(js_error)?;
        callback.forget();
        Ok(())
    }

    fn keyboard_command(event: &KeyEvent) -> Option<EngineCommand> {
        if event.state != ElementState::Pressed || event.repeat {
            return None;
        }
        match event.physical_key {
            PhysicalKey::Code(KeyCode::Space) => {
                Some(EngineCommand::SetPaused(!button_says_resume()))
            }
            PhysicalKey::Code(KeyCode::KeyR) => Some(EngineCommand::Reset),
            PhysicalKey::Code(KeyCode::Digit1) => Some(EngineCommand::SetPreset(0)),
            PhysicalKey::Code(KeyCode::Digit2) => Some(EngineCommand::SetPreset(1)),
            PhysicalKey::Code(KeyCode::Digit3) => Some(EngineCommand::SetPreset(2)),
            PhysicalKey::Code(KeyCode::Digit4) => Some(EngineCommand::SetPreset(3)),
            _ => None,
        }
    }

    fn sync_canvas_size(canvas: &HtmlCanvasElement, app: &mut MurmurationApp<'static>) {
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

    fn update_stats(app: &MurmurationApp<'_>) {
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

    fn button_says_resume() -> bool {
        document()
            .ok()
            .and_then(|document| document.get_element_by_id("pause-button"))
            .and_then(|button| button.text_content())
            .is_some_and(|text| text == "Resume")
    }

    fn toggle_stats() {
        if let Ok(document) = document() {
            if let Some(stats) = document.get_element_by_id("stats") {
                let hidden = stats.class_name().contains("is-hidden");
                stats.set_class_name(if hidden { "stats" } else { "stats is-hidden" });
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

    fn quality_bird_count(quality: &str) -> u32 {
        match quality.to_ascii_lowercase().as_str() {
            "high" | "50000" | "50k" => 50_000,
            "medium" | "25000" | "25k" => 25_000,
            _ => 10_000,
        }
    }

    fn preset_index(preset: &str) -> Option<usize> {
        match preset
            .to_ascii_lowercase()
            .replace([' ', '-', '_'], "")
            .as_str()
        {
            "0" | "calm" | "calmcloud" | "background" | "portfolio" | "classic" => Some(0),
            "1" | "predator" | "predatorripple" => Some(1),
            "2" | "ribbon" | "ribbonsheet" => Some(2),
            "3" | "storm" | "stormcolumn" | "vortex" => Some(3),
            _ => None,
        }
    }

    fn palette_from_js(value: &JsValue, fallback: RenderPalette) -> RenderPalette {
        let mut palette = fallback;
        palette.fog = option_color(value, "fog").unwrap_or(palette.fog);
        palette.ivory = option_color(value, "ivory").unwrap_or(palette.ivory);
        palette.ink = option_color(value, "ink").unwrap_or(palette.ink);
        palette.brass = option_color(value, "brass").unwrap_or(palette.brass);
        palette.rust = option_color(value, "rust").unwrap_or(palette.rust);
        if let Some(alpha) = option_number(value, "backgroundAlpha") {
            palette.background_alpha = alpha.clamp(0.0, 1.0) as f32;
        }
        palette
    }

    fn option_color(value: &JsValue, key: &str) -> Option<[f32; 4]> {
        option_string(value, key).and_then(|color| parse_hex_color(&color))
    }

    fn parse_hex_color(value: &str) -> Option<[f32; 4]> {
        let value = value.strip_prefix('#').unwrap_or(value);
        if value.len() != 6 {
            return None;
        }
        let number = u32::from_str_radix(value, 16).ok()?;
        Some([
            ((number >> 16) & 0xff) as f32 / 255.0,
            ((number >> 8) & 0xff) as f32 / 255.0,
            (number & 0xff) as f32 / 255.0,
            1.0,
        ])
    }

    fn option_value(value: &JsValue, key: &str) -> Option<JsValue> {
        let result = js_sys::Reflect::get(value, &JsValue::from_str(key)).ok()?;
        (!result.is_undefined() && !result.is_null()).then_some(result)
    }

    fn option_string(value: &JsValue, key: &str) -> Option<String> {
        option_value(value, key)?.as_string()
    }

    fn option_bool(value: &JsValue, key: &str) -> Option<bool> {
        option_value(value, key)?.as_bool()
    }

    fn option_number(value: &JsValue, key: &str) -> Option<f64> {
        option_value(value, key)?.as_f64()
    }

    fn webgpu_available() -> bool {
        web_sys::window().is_some_and(|window| {
            js_sys::Reflect::has(&window.navigator(), &JsValue::from_str("gpu")).unwrap_or(false)
        })
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
