# Murmuration Engine

Murmuration Engine is a Rust/WebGPU real-time simulation project for high-density flocking behavior, built to explore emergent motion, local coordination, and perturbation-wave propagation in large multi-agent systems.

## Workspace

```txt
crates/sim_core       core simulation types and stepping logic
crates/renderer_wgpu  rendering and GPU infrastructure
crates/native_app     desktop entrypoint and app shell
crates/web_app        browser entrypoint and wasm shell
assets/shaders        WGSL shader sources
examples/presets      simulation presets
docs                  design and implementation notes
```

## Current status

This repository is scaffolded for Phase 0. The next implementation step is opening a native window and rendering the first moving birds.

## Planned milestones

- native window and renderer
- fake murmuration MVP
- UI and presets
- CPU boids with spatial grid
- predator fear waves
- GPU compute simulation
- web build

## Development

Install Rust through [rustup](https://rustup.rs/), then run the native application:

```bash
cargo check
cargo run -p native_app
```

## Web MVP

The browser experience reuses the Rust simulation and `wgpu` renderer through WebAssembly and WebGPU. It includes pointer interaction, four presets, 10k/25k/50k quality tiers, pause/reset controls, live diagnostics, responsive resizing, and an unsupported-browser state.

Install the browser build tools once:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Run the local web experience:

```bash
cd crates/web_app
trunk serve --open
```

Create an optimized static bundle:

```bash
cd crates/web_app
trunk build --release
```

The production files are written to `crates/web_app/dist` and can be deployed to any HTTPS static host. WebGPU must be available in the visitor's browser.

See [docs/web_mvp.md](docs/web_mvp.md) for architecture, scope, testing, and follow-up work.
