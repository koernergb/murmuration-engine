# Background Mode Integration

Murmuration Engine's background mode is a client-side WebGPU module. It does not need a
Rust server or backend API. The same Rust simulation and renderer power both the standalone
demo and the embeddable mode; the JavaScript wrapper owns DOM mounting and cleanup.

## Build and generated files

From the repository root:

```sh
./scripts/build-web.sh
```

The optimized embed bundle is written to `dist/embed/`:

- `murmuration-background.js` — the public JavaScript API
- `web_app.js` — wasm-bindgen's JavaScript glue
- `web_app_bg.wasm` — the optimized Rust/WebGPU module

Keep all three files together. `dist/client/` contains the complete standalone demo plus
`background-test.html`, a foreground-content integration harness. The embed bundle has no
standalone title, statistics, instructions, or control panel.

## Initialization

```js
import { mount } from "/murmuration/murmuration-background.js";

const result = await mount(document.querySelector("#murmuration-background"), {
  quality: "low",
  preset: "portfolio",
  pointerInteraction: true,
  transparent: false,
});

if (!result.ok) {
  // Leave the site's normal CSS background visible.
  console.info("Murmuration unavailable:", result.code, result.error);
} else {
  const engine = result.controller;
  // Keep `engine` and call `await engine.dispose()` during teardown.
}
```

The target may be a selector, an `HTMLElement`, or an existing `HTMLCanvasElement`. When
given a container, `mount` creates an absolutely positioned canvas that fills it. The host
should establish the desired positioning context, typically `position: fixed; inset: 0` or
`position: absolute; inset: 0`. The canvas has `pointer-events: none`; links, controls, and
text above it remain interactive. Pointer coordinates are observed at the document level
only while pointer interaction is enabled.

`mount` resolves rather than throwing for expected initialization failures:

```ts
type MountResult =
  | { ok: true; controller: MurmurationController }
  | { ok: false; code: string; error: unknown };
```

Common failure codes are `unsupported-webgpu`, `already-mounted`, `invalid-target`,
`wasm-load-failed`, and `initialization-failed`. No error overlay is added in background
mode. Only one controller, animation loop, and WebGPU instance may be active at once.

## Public API

```ts
interface MurmurationController {
  readonly canvas: HTMLCanvasElement;
  readonly disposed: boolean;
  resize(width?: number, height?: number): void;
  pause(): void;
  resume(): void;
  setQuality(quality: "low" | "medium" | "high"): void;
  setPreset(preset: "portfolio" | "classic" | "storm" | "vortex" | number): void;
  setPointerInteraction(enabled: boolean): void;
  setPalette(palette: Partial<Palette>): void;
  setGuideTarget(target: { x: number; y: number; strength?: number }): void;
  dispose(): Promise<void>;
}

interface Palette {
  fog: string;
  ivory: string;
  ink: string;
  brass: string;
  rust: string;
  backgroundAlpha: number;
}
```

`setGuideTarget` accepts normalized viewport coordinates from `0` to `1`. It shifts the
flock's formation toward that screen-space point without replacing the cursor repulsor.
`strength` is clamped from `0` to `1`; hosts should smooth changing targets before sending
them when motion is driven by scrolling or other discrete input.

Calling `resize()` without dimensions measures the target. An internal `ResizeObserver`
also keeps the canvas synchronized. `dispose()` removes listeners and observers, exits the
event loop, releases the active controller, and removes a canvas created by `mount`. It does
not remove a canvas supplied by the host. Await disposal before mounting another instance.

The `portfolio` preset defaults to approximately 10,000 birds, calm motion, restrained
pointer response, and Low quality. The warm editorial palette is:

```js
{
  fog: "#A89C93",
  ivory: "#EEE9E1",
  ink: "#211F1D",
  brass: "#C69A5B",
  rust: "#945F4A",
  backgroundAlpha: 1,
}
```

Pass `transparent: true` or `backgroundAlpha: 0` for a transparent clear color. The renderer
requests a premultiplied-alpha WebGPU surface when the browser exposes one. Transparency is
therefore feasible on current implementations, but the most predictable presentation is an
opaque engine background or a matching CSS background on the host element.

The wrapper reads `prefers-reduced-motion` by default and starts paused when it is enabled.
Pass `reducedMotion: false` only when the containing product has its own explicit motion
preference control. Rendering also pauses while the document is hidden and resumes according
to the controller's prior paused/running state when visible again.

## Next.js integration

Copy `dist/embed/` to a versioned directory under the Next.js app's `public/` directory, for
example `public/vendor/murmuration/v1/`. Mount it from a Client Component; do not import or
initialize it during server rendering:

```tsx
"use client";

import { useEffect, useRef } from "react";

export function MurmurationBackground() {
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let disposed = false;
    let controller;

    void import("/vendor/murmuration/v1/murmuration-background.js").then(async ({ mount }) => {
      if (disposed || !hostRef.current) return;
      const result = await mount(hostRef.current, { preset: "portfolio", quality: "low" });
      if (result.ok) controller = result.controller;
    });

    return () => {
      disposed = true;
      void controller?.dispose();
    };
  }, []);

  return <div ref={hostRef} aria-hidden="true" className="murmurationBackground" />;
}
```

Give that element a fixed or absolute full-page layout and a lower stacking order than the
site content. If the application uses a `basePath` or asset CDN, keep the three files at the
same URL depth because the wrapper and wasm glue use relative sibling URLs. A versioned
directory is recommended because the filenames are deliberately stable.

## Hosting, MIME, and CSP

- Serve `.js` files as `text/javascript` (or `application/javascript`).
- Serve `.wasm` as `application/wasm`; streaming compilation depends on this MIME type.
- Same-origin hosting is simplest. A separate asset origin must return the appropriate CORS
  headers and must be allowed by the page's CSP.
- A typical policy needs `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'`. Add the
  asset origin when applicable. Some browsers/CSP deployments still require
  `'unsafe-eval'` for WebAssembly; prefer `'wasm-unsafe-eval'` where supported and verify the
  exact production browser matrix.
- The module creates no workers and makes no application-data requests. The browser fetches
  the JavaScript and neighboring wasm asset only.

WebGPU availability remains browser/device dependent. Keep a normal CSS background on the
host page as the graceful fallback and treat a failed mount as an enhancement being absent.
