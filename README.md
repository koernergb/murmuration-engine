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

```bash
cargo check
cargo run -p native_app
```

