# Murmuration Engine Implementation Plan

## Goal

Build a native-first Rust/WebGPU murmuration simulator that reaches a visually compelling MVP quickly, then scales into a technically impressive GPU-driven system with a browser build.

Success means:

- a polished native demo exists early
- the architecture leaves room for GPU compute and web deployment
- each milestone has clear acceptance criteria
- the project produces strong portfolio artifacts at multiple stages

## Product Strategy

Build in this order:

1. Get motion on screen fast.
2. Make it beautiful before making it perfect.
3. Add true local flocking after the visual loop feels good.
4. Move simulation to GPU only after behavior and rendering contracts are stable.
5. Ship native first, then web.

Reasoning:

- The highest risk technical work is GPU neighbor search.
- The highest value early outcome is a convincing visual demo.
- A strong M1 or M2 native build is already portfolio-worthy.

## Scope Decisions

### MVP

The first public-quality version should be:

- native desktop app
- 25k-50k V-shaped birds
- fake murmuration field plus optional partial CPU boids
- predator ripple interaction
- live parameter controls
- atmospheric sky, fog, and turn-wave shading
- screenshots/GIF-ready presentation

### Post-MVP

After MVP:

- CPU spatial grid refinement
- GPU compute update loop
- web build with quality presets
- GPU spatial grid for 100k+ birds

## Architecture

Use a Rust workspace with four crates:

```txt
murmuration-engine/
  Cargo.toml
  README.md
  crates/
    sim_core/
    renderer_wgpu/
    native_app/
    web_app/
  assets/
    shaders/
  examples/
    presets/
  docs/
```

### Responsibilities

`sim_core`

- bird data model
- flock parameters
- CPU update logic
- force composition
- spatial grid
- presets and serialization

`renderer_wgpu`

- wgpu device/surface setup
- camera and scene uniforms
- bird instancing pipeline
- sky/fog rendering
- trail/postprocess support later
- GPU buffer management

`native_app`

- window lifecycle
- input handling
- egui controls
- preset selection
- runtime stats
- screenshot/export hooks later

`web_app`

- wasm bootstrap
- WebGPU surface setup
- browser-specific input and resize handling
- quality preset selection

### Core Contracts

Keep these stable early because they cross CPU, GPU, and UI boundaries:

- `GpuBird` buffer layout
- `GpuParams` uniform layout
- preset schema
- camera uniform schema
- simulation stepping API

## Milestone Plan

## Phase 0: Repository Scaffold

Goal:
Create a clean workspace that can support native and web targets without restructuring later.

Tasks:

- create top-level Rust workspace
- add crate skeletons for `sim_core`, `renderer_wgpu`, `native_app`, `web_app`
- create shader, docs, and preset directories
- add root README with project framing
- add basic lint/test/build commands

Deliverables:

- `cargo check` passes for workspace
- native app opens a window, even if empty
- repository structure matches intended architecture

Acceptance criteria:

- new contributor can run one command and build the native target
- code layout clearly separates simulation, rendering, and app concerns

## Phase 1: Rendered Motion Prototype

Goal:
Get an alive-looking 3D flock on screen as quickly as possible.

Tasks:

- initialize `wgpu` and `winit`
- implement orbit/free camera
- add instance buffer for birds
- spawn 10k random birds
- render simple points or tiny V-shapes
- implement basic velocity integration

Deliverables:

- navigable 3D scene
- birds move every frame
- bird orientation follows velocity

Acceptance criteria:

- 10k birds visible
- native app holds roughly 60 FPS on development hardware
- camera interaction feels stable

Notes:

- Do not start with complex shading here.
- Favor simple correctness and fast iteration.

## Phase 2: Fake Murmuration MVP

Goal:
Make the flock look compelling before solving true local neighbor behavior.

Tasks:

- add global center attraction
- add soft ellipsoid/world boundary force
- add coherent noise or curl-noise-like field
- implement speed clamp
- implement turn-rate limiting
- upgrade rendering to V-shaped instanced birds
- add sunset gradient sky
- add fog and depth fade
- add turn-wave brightness approximation

Deliverables:

- convincing emergent wave motion
- screenshot-worthy lighting and atmosphere

Acceptance criteria:

- 25k birds move in coherent ribbons/sheets
- flock remains stable for long runs
- screenshots read as a murmuration rather than generic particles

Why this phase matters:

- It creates the first portfolio artifact.
- It derisks rendering, camera, and art direction before harder simulation work.

## Phase 3: UI, Presets, and Runtime Instrumentation

Goal:
Turn the demo into an interactive simulation instrument.

Tasks:

- integrate `egui`
- expose sliders for core movement and rendering parameters
- add preset loading and reset-to-default behavior
- display FPS, frame time, bird count, and current quality mode
- make fake flock mode and real boids mode switchable

Initial slider set:

- bird count
- center attraction
- noise strength
- noise scale
- boundary radius
- min/max speed
- max turn rate
- fog density
- exposure

Preset set:

- Calm Cloud
- Ribbon Sheet
- Dense Pulse
- Storm Column
- Predator Ripple

Acceptance criteria:

- preset switching is instant
- parameter updates are reflected live
- UI does not tank frame rate or destabilize stepping

## Phase 4: CPU Spatial Grid and Real Boids

Goal:
Introduce true local coordination while preserving the better-looking fake mode as a fallback.

Tasks:

- implement `Bird` and `FlockParams` in `sim_core`
- implement uniform spatial grid with cell hashing
- query 27 neighboring cells per bird
- compute separation, alignment, and cohesion
- combine local boid forces with boundary and optional noise
- track local density for shading

Technical decisions:

- grid cell size should match neighbor radius
- use structure-of-arrays only if profiling shows `Bird` AoS is insufficient
- start single-threaded, then parallelize only if necessary

Acceptance criteria:

- 5k-20k birds exhibit stable local flocking
- no exploding velocities or collapse to a point mass
- run remains stable for 5+ minutes

Exit criteria for moving on:

- local flocking behavior is visually distinct from fake mode
- CPU cost and bottlenecks are understood via simple profiling

## Phase 5: Predator and Fear-Wave Behavior

Goal:
Add the interaction that makes the system feel like a murmuration rather than a flock screensaver.

Tasks:

- add predator state and control
- add per-bird fear scalar
- increase fear near predator
- decay fear over time
- spread fear through neighbor interactions
- let fear modulate turn responsiveness and shading

Behavior targets:

- flock splits and rejoins
- visible ripple travels through the formation
- predator interaction can be triggered repeatedly without instability

Acceptance criteria:

- waves are visible from the default camera
- behavior feels responsive, not random
- predator mode is demoable with one click/toggle

## Phase 6: Native Polish Pass

Goal:
Package the native app as a strong portfolio artifact before major compute refactors.

Tasks:

- tune presets for visual variety
- refine bird shape and shading
- add subtle trails if affordable
- add cinematic UI toggle
- support screenshots
- capture README assets
- write architecture and performance notes

Acceptance criteria:

- README screenshots/GIFs exist
- app has a clean default launch state
- there is a stable, impressive 30-second demo flow

Recommendation:

- This is a good checkpoint for publishing even if GPU compute is not done yet.

## Phase 7: GPU Compute Update Loop

Goal:
Move per-frame simulation updates off the CPU while preserving existing rendering and parameter contracts.

Tasks:

- add ping-pong bird buffers
- write `update_birds.wgsl`
- move global forces, boundary, noise, predator, and integration to compute
- keep CPU boids mode available for validation during transition
- swap buffers each frame and render the latest output

Dependencies:

- `GpuBird` and `GpuParams` layouts must already be stable
- rendering pipeline must already consume bird buffers cleanly

Acceptance criteria:

- 50k+ birds web-feasible
- 100k birds native-feasible
- CPU no longer iterates over all birds in GPU mode

Risk:

- debugging compute and render synchronization can slow progress
- this is why it comes after a polished CPU/native version

## Phase 8: GPU Spatial Grid

Goal:
Reach technically impressive large-scale local flocking.

Tasks:

- add grid clear pass
- add bird-to-cell assignment pass
- add flock update pass that inspects nearby cells
- define max-per-cell behavior and overflow handling
- profile memory pressure and contention

Implementation notes:

- prefer fixed-size buffers over dynamic structures
- accept approximate neighbors if behavior remains convincing
- add debug visualization for cell occupancy if needed

Acceptance criteria:

- 100k+ birds with local flocking in native mode
- stable performance without CPU neighbor search
- no catastrophic artifacts from cell overflow

## Phase 9: Web Target

Goal:
Ship a browser version that is reduced in scale but strong in presentation.

Tasks:

- add wasm target bootstrap
- initialize WebGPU in browser
- adapt input and resize handling
- expose quality presets
- provide unsupported-browser messaging
- tune default counts for laptop-class devices

Quality tiers:

- Low: 10k birds
- Medium: 25k-30k birds
- High: 50k-75k birds

Acceptance criteria:

- browser build runs without local setup beyond hosting
- demo loads into an interactive default scene
- quality settings prevent unusable defaults on weaker machines

## Workstreams

These can overlap once the repo is scaffolded.

### Simulation

- fake field
- boid forces
- predator/fear
- CPU grid
- GPU compute

### Rendering

- instanced birds
- sky/fog
- density/turn-wave shading
- trails
- postprocessing

### Product Surface

- controls
- presets
- README/storytelling
- screenshots/GIFs
- browser deployment

## Recommended 4-Week Schedule

### Week 1

- scaffold workspace
- ship native renderer
- render 10k-25k moving birds
- implement fake murmuration field

Exit target:

- first alive-looking video clip

### Week 2

- add sky/fog/shading
- integrate UI
- add presets and stats
- tune fake mode into a portfolio-quality visual

Exit target:

- first shareable native demo

### Week 3

- implement CPU spatial grid
- add separation/alignment/cohesion
- add predator and fear wave behavior
- compare fake vs real modes

Exit target:

- interactive demo with true local flocking and ripple behavior

### Week 4

- move update loop to GPU
- stabilize buffer contracts
- begin web target
- document performance tradeoffs

Exit target:

- native GPU-updated build and rough browser prototype

## Engineering Checklist

## Initial Dependencies

- `wgpu`
- `winit`
- `egui`
- `egui-wgpu`
- `egui-winit`
- `glam`
- `bytemuck`
- `rand`
- `serde`
- `serde_json`
- `tracing`
- `pollster` for native startup

Optional later:

- `noise`
- `rayon`
- `wasm-bindgen`
- `web-sys`
- `console_error_panic_hook`

## Non-Functional Requirements

- deterministic enough stepping for repeatable presets
- no frame-to-frame NaN propagation
- clean fallback when WebGPU is unavailable
- performance stats visible during tuning
- preset files simple enough to edit by hand

## Risks and Mitigations

### Risk: GPU grid complexity stalls the project

Mitigation:

- delay GPU grid until a polished native demo exists
- keep fake and CPU modes usable independently

### Risk: Visual output feels like particles, not birds

Mitigation:

- prioritize turn-rate limiting and velocity-oriented rendering early
- add turn-wave shading before overcomplicating simulation

### Risk: CPU boids become too slow too early

Mitigation:

- keep fake mode for higher counts
- tune for 5k-20k in CPU mode and reserve higher counts for GPU mode

### Risk: Web target consumes time before core experience is strong

Mitigation:

- treat web as a deployment phase, not the first milestone

## Definition of Done by Stage

### Done for MVP

- native app launches cleanly
- flock is visually convincing
- controls and presets work
- predator ripple is demoable
- README can show polished media

### Done for Technical Showcase

- GPU compute path is stable
- 100k+ birds are practical in native mode
- architecture is documented clearly

### Done for Public Web Release

- browser build is stable on supported machines
- quality presets prevent overload
- demo is easy to open and explore

## Immediate Next Tasks

The next concrete implementation sequence should be:

1. create the Rust workspace and crate skeletons
2. open a native window with `wgpu` and `winit`
3. render 10k instanced birds with simple motion
4. add fake-field forces and turn-rate limiting
5. add sky/fog/shading
6. add UI and presets

If progress stalls, do not jump to GPU neighbor search. Get the native MVP beautiful first.
