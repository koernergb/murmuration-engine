# Murmuration Engine Design Constraints

## Design Intent

Murmuration Engine should feel like a cinematic nature-film simulation, not a sci-fi particle demo.

The project should present:

- a vast dusk sky
- a flock that reads as a living mass first and individual birds second
- coordinated wave-like turns with silver/dark ripples
- restrained, elegant atmosphere instead of spectacle-heavy effects

If a visual or motion choice makes the project feel synthetic, arcade-like, neon, or noisy, it is the wrong direction.

## Core Aesthetic

The default look should evoke:

- starling murmurations at dusk
- telephoto wildlife footage
- large-scale atmospheric depth
- dark silhouettes with occasional metallic turn highlights

Keywords:

- cinematic
- restrained
- organic
- atmospheric
- fluid
- high-scale

## Visual Language

### Sky and Background

- Always render against a real atmospheric backdrop, never a flat black or empty void.
- Use a warm-to-cool procedural sky gradient.
- Horizon should be warm, dusty, and slightly luminous.
- Zenith should be cooler, darker, and more subdued.
- Atmospheric haze should be visible at medium and far depth.

Preferred palette:

- dusk blue
- slate
- charcoal
- silver-gray
- muted amber

Avoid:

- purple-heavy palettes
- highly saturated sunsets
- deep-space black backgrounds
- rainbow gradients

### Bird Appearance

- Birds should be primarily read as silhouettes.
- Base bird tone should stay dark and neutral.
- Highlights should appear mainly through turn-wave shading and directional lighting.
- The flock should read as one sculptural body at distance.
- Individual birds should only become visually explicit when the camera gets closer.

Render strategy:

- start with tiny instanced V-shaped birds
- orient geometry by velocity
- avoid full mesh birds in early versions

### Lighting

- Use one low directional sun as the key light.
- Favor rim response and grazing-angle highlights over broad diffuse brightness.
- Keep ambient light soft and low-contrast.
- Exposure should support silhouettes first, highlights second.

Avoid:

- bright bloom-heavy presentation
- flashy specular materials
- multi-colored lighting rigs

### Fog and Depth

- Fog is mandatory in the default presentation.
- Depth haze should create layering and reinforce scale.
- Distant birds should soften into the atmosphere rather than remain crisp everywhere.
- The horizon should feel thick enough to support the illusion of mass and distance.

### Trails and Postprocessing

- Trails, if used, must be subtle and naturalistic.
- Motion smear should feel like air and momentum, not special effects.
- Postprocessing should be restrained and cinematic.
- Any effect that calls attention to itself more than the flock is too strong.

Avoid:

- neon trails
- arcade streaks
- heavy bloom
- aggressive chromatic aberration
- glitch aesthetics

## Motion Language

### Flock Behavior

- The flock should feel elastic, cohesive, and alive.
- Motion must prioritize ribbons, sheets, folds, pulses, and traveling disturbances.
- The simulation should feel like pressure moving through a body, not particles wandering independently.
- Local behavior should aggregate into large-scale shape changes.

### Steering Constraints

- Turn-rate limiting is mandatory.
- Speed clamping is mandatory.
- Noise should be coherent and low-frequency.
- Motion should never jitter, sparkle, or twitch.

If the movement reads as random, the simulation has failed the aesthetic target even if the math is technically correct.

### Predator Response

- Predator interactions should create sharp but graceful disruptions.
- Nearby birds should react immediately.
- Fear should propagate as a readable wavefront.
- The flock should split, fold, and recombine rather than simply explode outward.

The desired feeling is:

- tension
- compression
- release
- recomposition

## Camera Language

- Default camera behavior should feel cinematic, not game-like.
- Prefer long-lens composition and slow movement.
- Favor orbit, glide, and dolly motion.
- Keep the flock framed inside a large sky volume with negative space.
- Camera motion should support the flock’s shape and wave behavior.

Default presentation should avoid:

- frantic free-fly motion
- rapid cuts
- overly wide-angle distortion
- constant camera spinning

Interactive free camera can exist, but it should not define the project’s first impression.

## UI Presentation

- The simulation itself is the hero; the UI is secondary.
- Controls should be clean, minimal, and easy to hide.
- A cinematic mode should remove UI clutter instantly.
- Debug visualizations should never appear in the default showcase mode.

UI tone:

- quiet
- practical
- unobtrusive

Avoid:

- colorful dashboard styling
- oversized debug panels
- UI that visually competes with the scene

## Rendering Constraints

These constraints are intentional and should be treated as product rules:

- Start with LOD 1 only: tiny instanced V-birds.
- Do not begin with complex animated bird meshes.
- Do not use spectacle-driven effects to compensate for weak motion.
- Depth, shading, and turn response matter more than polygon detail.
- A strong silhouette and coherent group motion are higher priority than per-bird realism.

## What Success Looks Like

A successful frame should communicate:

- scale
- coordination
- atmospheric depth
- living motion
- restrained beauty

A successful short clip should feel like:

- a real flock captured beautifully
- an emergent system under tension
- a polished simulation artifact worth watching silently

## Anti-Goals

The project should not become:

- a cyberpunk particle system
- a neon VFX demo
- a generic boids sandbox
- a chaotic noise field
- a bright game-like spectacle piece

Specifically avoid:

- purple-on-black presentation
- glowing particles
- exaggerated bloom
- highly saturated color ramps
- noisy random motion
- birds reading like triangles in empty space

## Decision Filter

When making visual or simulation choices, prefer the option that increases:

- readability of wave behavior
- atmospheric depth
- silhouette quality
- coherence of mass motion
- cinematic restraint

Reject choices that mainly increase:

- raw flashiness
- color intensity
- effect count
- visual noise
- game-like energy
