import initWasm, { mountBackground as mountWasmBackground } from "./web_app.js";

let wasmInitialization;
let activeController;

const WARM_EDITORIAL_PALETTE = Object.freeze({
  fog: "#A89C93",
  ivory: "#EEE9E1",
  ink: "#211F1D",
  brass: "#C69A5B",
  rust: "#945F4A",
  backgroundAlpha: 1,
});

function initializeWasm() {
  wasmInitialization ??= initWasm();
  return wasmInitialization;
}

function resolveTarget(target) {
  if (typeof target === "string") {
    return document.querySelector(target);
  }
  return target;
}

function createCanvas(target) {
  const resolved = resolveTarget(target);
  if (!(resolved instanceof Element)) {
    throw new TypeError("Murmuration Engine mount target must be a DOM element, canvas, or selector.");
  }

  if (resolved instanceof HTMLCanvasElement) {
    return { canvas: resolved, created: false };
  }

  const canvas = document.createElement("canvas");
  canvas.setAttribute("aria-hidden", "true");
  resolved.append(canvas);
  return { canvas, created: true };
}

function styleBackgroundCanvas(canvas) {
  Object.assign(canvas.style, {
    position: "absolute",
    inset: "0",
    display: "block",
    width: "100%",
    height: "100%",
    pointerEvents: "none",
  });
}

function physicalSize(canvas) {
  const rect = canvas.getBoundingClientRect();
  const ratio = window.devicePixelRatio || 1;
  return {
    width: Math.max(1, Math.round(rect.width * ratio)),
    height: Math.max(1, Math.round(rect.height * ratio)),
  };
}

function errorResult(code, error) {
  const normalized = error instanceof Error ? error : new Error(String(error));
  return { ok: false, code, error: normalized };
}

export async function mount(target, options = {}) {
  if (activeController && !activeController.disposed) {
    return errorResult("already-mounted", new Error("Only one Murmuration Engine instance may be mounted at a time."));
  }

  if (!("gpu" in navigator)) {
    return errorResult("unsupported-webgpu", new Error("WebGPU is unavailable in this browser."));
  }

  let canvas;
  let created = false;
  try {
    ({ canvas, created } = createCanvas(target));
    styleBackgroundCanvas(canvas);
    try {
      await initializeWasm();
    } catch (error) {
      if (created) canvas.remove();
      return errorResult("wasm-load-failed", error);
    }

    const prefersReducedMotion = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
    const reducedMotion = options.reducedMotion ?? prefersReducedMotion;
    const wasmController = await mountWasmBackground(canvas, {
      quality: options.quality ?? "low",
      preset: options.preset ?? "portfolio",
      pointerInteraction: options.pointerInteraction ?? true,
      reducedMotion,
      transparent: options.transparent ?? false,
      palette: { ...WARM_EDITORIAL_PALETTE, ...options.palette },
    });
    // winit initializes the canvas style while creating the WebGPU window.
    // Reassert the host-facing background contract after that initialization.
    styleBackgroundCanvas(canvas);

    let disposed = false;
    const resize = (width, height) => {
      if (disposed) return;
      const size = width && height ? { width, height } : physicalSize(canvas);
      wasmController.resize(Math.max(1, Math.round(size.width)), Math.max(1, Math.round(size.height)));
    };

    const resizeObserver = new ResizeObserver(() => resize());
    resizeObserver.observe(canvas);
    resize();

    const controller = {
      canvas,
      get disposed() {
        return disposed || wasmController.disposed;
      },
      resize,
      pause() {
        if (!disposed) wasmController.pause();
      },
      resume() {
        if (!disposed) wasmController.resume();
      },
      setQuality(quality) {
        if (!disposed) wasmController.setQuality(quality);
      },
      setPreset(preset) {
        if (!disposed) wasmController.setPreset(preset);
      },
      setPointerInteraction(enabled) {
        if (!disposed) wasmController.setPointerInteraction(Boolean(enabled));
      },
      setPalette(palette) {
        if (!disposed) wasmController.setPalette({ ...WARM_EDITORIAL_PALETTE, ...palette });
      },
      async dispose() {
        if (disposed) return;
        disposed = true;
        resizeObserver.disconnect();
        wasmController.dispose();
        if (created) canvas.remove();
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        if (activeController === controller) activeController = undefined;
      },
    };

    activeController = controller;
    return { ok: true, controller };
  } catch (error) {
    if (created) canvas?.remove();
    const message = String(error?.message ?? error);
    const code = message.includes("Only one") ? "already-mounted" : "initialization-failed";
    return errorResult(code, error);
  }
}

export function getActiveController() {
  return activeController && !activeController.disposed ? activeController : undefined;
}

export { WARM_EDITORIAL_PALETTE };
