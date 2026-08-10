# Web MVP Plan

## Implementation Status

The browser MVP described here was implemented in August 2026. The delivered version includes:

- shared Rust application state used by both native and browser shells
- WebAssembly startup through Trunk and `wasm-bindgen`
- current-browser WebGPU support through `wgpu` 26
- responsive canvas sizing with device-pixel-ratio handling
- pointer repulsion, keyboard shortcuts, presets, and quality tiers
- pause, reset, diagnostics, loading, and failure states
- optimized production packaging for static hosting and OpenAI Sites
- native workspace tests, WASM compile checks, optimized bundle verification, live WebGPU startup testing, control testing, and a 390 × 844 responsive layout check

GPU compute simulation, a GPU spatial grid, broad cross-device profiling, touch-specific behavior, and a WebGL fallback remain post-MVP work.

## Goal

Ship Murmuration Engine as an interactive browser experience while preserving the existing Rust simulation, `wgpu` renderer, and WGSL shaders.

The web MVP should:

- load from a normal web page without local setup
- render an animated flock through WebGPU
- respond to cursor movement and window resizing
- provide a small set of understandable controls and presets
- choose a safe default bird count for laptop-class hardware
- explain unsupported-browser and initialization failures clearly

The simulation and rendering should run locally in the browser. A server-side Rust application is not required for the MVP; the web server only needs to host static files.

## Proposed Architecture

```txt
HTML/CSS
  page layout, loading state, controls, compatibility messages

JavaScript bootstrap
  load the WebAssembly module and report startup failures

Rust compiled to WebAssembly
  application lifecycle, simulation state, input, presets

wgpu + WebGPU + WGSL
  sky and instanced bird rendering
```

Existing workspace responsibilities should remain intact:

```txt
crates/sim_core       shared simulation and parameters
crates/renderer_wgpu  shared WebGPU renderer and camera
crates/native_app     native window and event-loop shell
crates/web_app        browser bootstrap and browser event-loop shell
assets/shaders        shared WGSL shaders
```

## MVP Scope

### Included

- WebAssembly build of the Rust application
- WebGPU rendering into an HTML canvas
- existing procedural flock behavior
- pointer interaction with the flock
- responsive canvas resizing
- four existing presets
- quality or bird-count selection
- pause/resume and reset controls
- FPS and bird-count diagnostics
- loading, unsupported-browser, and fatal-error states
- production static build suitable for HTTPS hosting

### Deferred

- server-side simulation
- accounts and cloud preset storage
- multiplayer or synchronized sessions
- GPU compute simulation
- GPU spatial grid
- full mobile optimization
- WebGL fallback
- exporting video or screenshots
- accessibility work beyond basic keyboard-operable controls and readable status text

## Implementation Tasks

### 1. Establish the WebAssembly Toolchain

- [ ] Add the `wasm32-unknown-unknown` Rust compilation target.
- [ ] Select a build tool. Trunk is recommended for the MVP because it can build the Rust crate, process the HTML entry point, and serve the result locally.
- [ ] Add target-specific browser dependencies to `crates/web_app/Cargo.toml`:
  - `wasm-bindgen`
  - `wasm-bindgen-futures`
  - `web-sys` with only the required browser APIs
  - `console_error_panic_hook`
  - a browser-compatible logging implementation
- [ ] Confirm `wgpu` and `winit` features are configured for the WebGPU/WASM target.
- [ ] Add reproducible development and production build commands to the README.

Target commands should eventually look like:

```bash
rustup target add wasm32-unknown-unknown
trunk serve crates/web_app/index.html
trunk build crates/web_app/index.html --release
```

Exact commands may change based on the final Trunk workspace configuration.

### 2. Create the Browser Entry Point

- [ ] Replace `bootstrap_message` in `crates/web_app/src/lib.rs` with an asynchronous `#[wasm_bindgen(start)]` entry point.
- [ ] Install the panic hook and browser logger before initializing the renderer.
- [ ] Locate the page's canvas element or create one through `winit`.
- [ ] Attach the `winit` window to that canvas.
- [ ] Initialize `Renderer` asynchronously.
- [ ] Seed `SimulationState` with a browser-safe bird count.
- [ ] Start the browser-compatible `winit` event loop.
- [ ] Surface startup errors in the page instead of relying only on the developer console.

### 3. Share Application Logic

The current lifecycle and input-to-simulation wiring live in `native_app`. The web shell should not create a second independent copy of that behavior.

- [ ] Extract platform-neutral application state from `crates/native_app/src/app.rs` into a shared module or new crate.
- [ ] Keep the following logic shared:
  - simulation ownership
  - camera ownership
  - frame update
  - cursor screen-to-world projection
  - preset application
  - reset and parameter changes
  - renderer draw call
- [ ] Keep native-only window-title formatting and native keyboard handling in `native_app`.
- [ ] Keep DOM manipulation and browser-specific input handling in `web_app`.
- [ ] Ensure the native application still builds and behaves as before after extraction.

A small `app_core` crate is reasonable if sharing through an existing crate would create circular dependencies.

### 4. Make Timing Browser-Compatible

- [ ] Verify the current timing implementation compiles and behaves correctly under WASM.
- [ ] Prefer the animation-frame-driven redraw lifecycle provided by the browser event loop.
- [ ] Keep the existing delta-time clamp to prevent large simulation jumps after a hidden tab resumes.
- [ ] Pause or reduce work when the page is not visible if the event loop does not already do so.
- [ ] Avoid blocking calls, threads, and synchronous waits in browser code.

### 5. Adapt Rendering for WebGPU

- [ ] Verify adapter and device creation in Chrome, Edge, Firefox, and Safari versions that advertise WebGPU support.
- [ ] Use limits supported by browser WebGPU implementations rather than assuming native defaults.
- [ ] Verify canvas surface formats, alpha mode, and present mode.
- [ ] Handle zero-sized canvases and resize events without recreating unnecessary resources.
- [ ] Confirm depth textures are recreated after resize.
- [ ] Confirm all WGSL shaders pass browser validation.
- [ ] Add readable handling for:
  - no compatible adapter
  - device request failure
  - surface loss
  - out-of-memory failure
  - uncaught shader validation errors

The MVP should target WebGPU directly. A WebGL fallback would add a second rendering path and is outside the initial scope.

### 6. Add Browser Input

- [ ] Map pointer coordinates relative to the canvas, accounting for device pixel ratio.
- [ ] Feed pointer movement into the existing screen-to-world cursor projection.
- [ ] Clear the active cursor target when the pointer leaves the canvas.
- [ ] Decide the initial interaction behavior:
  - hover repels birds, matching the current application; or
  - hover attracts birds, if cursor attraction is implemented first
- [ ] Add pointer-down state if the force should only apply while pressing.
- [ ] Add touch/pointer-event support where it is inexpensive to do so.
- [ ] Prevent browser scrolling or text selection only when interaction with the canvas requires it.

### 7. Build the Web UI

- [ ] Update `crates/web_app/index.html` with:
  - full-window or hero-sized canvas
  - loading indicator
  - compatibility/error panel
  - compact control panel
  - short interaction instructions
- [ ] Expose the four existing presets.
- [ ] Add quality selection:
  - Low: 10,000 birds
  - Medium: 25,000 birds
  - High: 50,000 birds
- [ ] Default to Low or Medium until browser profiling supports a more aggressive choice.
- [ ] Add pause/resume and reset controls.
- [ ] Display FPS and active bird count behind a diagnostics toggle.
- [ ] Keep control changes in Rust or pass typed values cleanly across the WASM boundary; avoid per-bird data crossing that boundary.
- [ ] Make controls usable by keyboard and ensure status messages are readable by assistive technology.

### 8. Tune CPU and Upload Performance

The current simulation updates every bird on the CPU and uploads instance data each frame. This is acceptable for the first web version only if profiling confirms stable performance at the selected quality levels.

- [ ] Start with 10,000 birds for initial browser bring-up.
- [ ] Profile simulation time, buffer upload time, render time, memory use, and WASM download size.
- [ ] Test 25,000 and 50,000 birds separately rather than assuming native performance transfers to browsers.
- [ ] Avoid allocations inside the per-frame update path.
- [ ] Reuse GPU buffers unless a quality change requires increased capacity.
- [ ] Ensure changing presets does not accidentally return to the native default of 50,000 birds.
- [ ] Automatically step down the default tier if startup capability checks justify it.

Moving simulation updates into `update_birds.wgsl` is the preferred post-MVP optimization. It should not block the first browser release unless the CPU implementation cannot sustain the Low tier.

### 9. Package and Host

- [ ] Produce a release build with optimized WASM.
- [ ] Optionally run a WASM size optimizer as part of the production build.
- [ ] Verify generated files use correct MIME types, especially `.wasm`.
- [ ] Host over HTTPS, which browser GPU features may require outside local development.
- [ ] Configure long-lived caching for content-hashed assets and conservative caching for the HTML entry point.
- [ ] Add a favicon, page title, social preview metadata, and concise project description.
- [ ] Document a repeatable deployment process.

The MVP can be hosted as static assets on services such as GitHub Pages, Cloudflare Pages, Netlify, or another static host. No always-running Rust server is required.

## Suggested Delivery Order

### Milestone 1: Canvas Bring-Up

- WebAssembly module loads.
- A browser canvas is created.
- `wgpu` obtains a WebGPU device and renders a clear color or sky.
- Initialization failures appear in the page.

### Milestone 2: Flock Rendering

- Shared renderer draws seeded birds.
- Resize behavior is correct.
- Low quality runs at a stable interactive frame rate.

### Milestone 3: Interaction

- Pointer coordinates map correctly into world space.
- Birds respond to the cursor.
- Reset, pause, presets, and quality controls work.

### Milestone 4: Production Polish

- Loading and unsupported-browser states are present.
- Release build and static deployment are documented.
- Multiple desktop browsers and representative hardware have been tested.
- The default experience is stable without opening developer tools.

## Testing Matrix

At minimum, test:

- macOS with Safari and Chrome
- Windows with Edge or Chrome
- one Firefox release with WebGPU enabled by default
- integrated graphics on a laptop
- high-DPI and standard-DPI displays
- window resize and device rotation where applicable
- tab backgrounding and restoration
- pointer leaving and re-entering the canvas
- WebGPU unavailable or permission/device initialization failure

For each supported browser, record:

- startup success
- initial load time
- FPS at Low, Medium, and High
- memory use where browser tooling exposes it
- rendering or shader validation warnings
- whether pointer interaction tracks the visible cursor accurately

## MVP Acceptance Criteria

The web MVP is complete when:

- [ ] A user can open an HTTPS URL and see the simulation without installing anything.
- [ ] The browser runs the Rust code through WebAssembly and renders through WebGPU.
- [ ] The default quality tier stays responsive on a representative laptop with integrated graphics.
- [ ] Pointer movement produces an obvious, stable flock response.
- [ ] Resize, pause, reset, presets, and quality selection work.
- [ ] Unsupported WebGPU and initialization failures produce a useful on-page message.
- [ ] No per-frame bird data is sent to or received from a backend.
- [ ] The native application still builds and runs.
- [ ] Development, production build, and deployment commands are documented.

## Post-MVP Priorities

1. Move global forces and integration into a WebGPU compute shader.
2. Keep bird buffers resident on the GPU between update and render passes.
3. Add automatic quality selection informed by measured frame time.
4. Improve touch interaction and mobile layout.
5. Add GPU spatial partitioning if true local boid behavior is required at high counts.
6. Add shareable preset URLs or a backend only when persistence or collaboration becomes a product requirement.

## Key Risks

- Browser WebGPU capabilities and limits differ across devices.
- The CPU simulation and per-frame instance upload may cap the practical bird count.
- Native and web event loops have different lifecycle requirements.
- A high initial bird count can make the page appear broken on weaker machines.
- Shader errors are less discoverable for users unless surfaced in the UI.
- Duplicating native application logic in `web_app` would create ongoing behavior drift.

The risk-reducing strategy is to bring up the existing renderer at 10,000 birds first, preserve shared Rust logic, and increase quality only after profiling real browsers.
