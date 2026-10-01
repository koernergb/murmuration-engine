# Murmuration Engine

An interactive, high-density flocking simulation built in Rust and rendered with WebGPU.

Murmuration Engine explores how simple local rules can produce coordinated motion at scale. The same simulation and rendering code runs as a native desktop application and in the browser through WebAssembly, with interactive presets, pointer-driven disturbances, and quality tiers of up to 50,000 birds.

## Why this project

This project is a practical study in real-time graphics and simulation architecture. It demonstrates:

- a multi-crate Rust workspace with clear simulation, rendering, and platform boundaries;
- a shared application core used by native and browser front ends;
- instanced WebGPU rendering with custom WGSL shaders;
- Rust-to-WebAssembly delivery using Trunk and `wasm-bindgen`;
- simulation data structures and spatial-grid groundwork for future local-neighbor updates;
- responsive input, runtime quality controls, diagnostics, and graceful startup failures; and
- a reusable background module for embedding the simulation in another site.

## Features

- Real-time procedural field motion with layered anchors, curl-like noise, boundary steering, and cursor repulsion
- Four distinct presets: Calm Cloud, Predator Ripple, Ribbon Sheet, and Storm Column
- Pointer interaction that introduces a local pressure wave through the flock
- 10k, 25k, and 50k bird quality tiers
- Native desktop shell and a browser-based WebAssembly shell
- Responsive WebGPU canvas with device-pixel-ratio handling
- Pause, reset, preset, quality, and optional performance controls
- Loading and unsupported-browser states
- Standalone and embeddable production bundles

## Technology

| Area | Tools |
| --- | --- |
| Language | Rust 2021 |
| Graphics | `wgpu` 26, WebGPU, WGSL |
| Windowing | `winit` |
| Math and data | `glam`, `bytemuck`, `serde` |
| Browser runtime | WebAssembly, `wasm-bindgen`, `web-sys` |
| Web build | Trunk |

## Architecture

```text
Native shell ─┐
              ├── app_core ── sim_core
Browser shell ┘       │
                      └── renderer_wgpu ── WGSL shaders ── WebGPU
```

| Path | Responsibility |
| --- | --- |
| `crates/sim_core` | Bird state, procedural motion, simulation parameters, integration, and spatial-grid groundwork |
| `crates/renderer_wgpu` | Camera, GPU buffers, instanced bird rendering, sky, depth, and render passes |
| `crates/app_core` | Shared lifecycle, presets, commands, input mapping, and runtime statistics |
| `crates/native_app` | Desktop window and native event loop |
| `crates/web_app` | Browser entry point, DOM controls, and WebAssembly lifecycle |
| `assets/shaders` | Shared WGSL render and compute shader sources |
| `examples/presets` | JSON configurations for the included flock behaviors |

The browser build performs the simulation locally. No backend is required; the server only delivers static HTML, JavaScript, WebAssembly, and rendering assets.

## Run the browser experience

### Prerequisites

- [Rust and rustup](https://rustup.rs/)
- A browser with WebGPU support

Install the browser target and Trunk once:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Start the development server:

```bash
cd crates/web_app
trunk serve --open
```

Open the `http://localhost:...` address printed by Trunk. Opening `index.html` directly with a `file://` URL will not correctly initialize the WebAssembly application.

### Controls

| Input | Action |
| --- | --- |
| Pointer movement | Disturb the flock near the cursor |
| `1`–`4` | Select a preset |
| `Space` | Pause or resume |
| `R` | Reset the simulation |
| Settings control | Change preset, quality, and diagnostics |

## Run the native application

```bash
cargo run -p native_app
```

## Build and validate

Check the complete Rust workspace:

```bash
cargo check --workspace --all-targets
```

Create an optimized browser bundle:

```bash
./scripts/build-web.sh
```

The build produces:

- `dist/client` — standalone browser experience
- `dist/embed` — copy-ready embeddable module
- `dist/server` — minimal static asset worker

See [`docs/background_integration.md`](docs/background_integration.md) for embedding instructions.

## Engineering notes

The current browser MVP uses a procedural field-motion model rather than a full local-neighbor boids solver. Simulation updates run on the CPU and upload instance data for rendering each frame. This keeps the architecture portable and debuggable while supporting the project’s current quality tiers. The next major performance step is to move integration and global forces into WebGPU compute, keep bird buffers resident on the GPU, and expand device-level profiling.

The project deliberately separates platform-specific code from shared simulation and rendering concerns. That structure makes it possible to evolve the native and browser experiences independently without maintaining two simulation implementations.

## Current status

The native and browser MVPs are implemented. The browser version includes WebGPU startup, live flock rendering, pointer interaction, presets, quality selection, responsive resizing, diagnostics, and production packaging.

Planned follow-up work includes:

- GPU compute-based simulation updates
- GPU spatial partitioning for larger local-neighborhood simulations
- automatic quality selection based on measured frame time
- broader cross-browser and cross-device profiling
- improved touch interaction and mobile controls
- a WebGL fallback for browsers without WebGPU

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — workspace boundaries and stable contracts
- [`docs/simulation.md`](docs/simulation.md) — simulation model and planned layers
- [`docs/performance.md`](docs/performance.md) — profiling priorities
- [`docs/web_mvp.md`](docs/web_mvp.md) — browser architecture, scope, and validation plan
- [`docs/background_integration.md`](docs/background_integration.md) — embedding the simulation in another site

## License

MIT
