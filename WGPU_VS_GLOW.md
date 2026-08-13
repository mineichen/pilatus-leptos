# wgpu vs glow (egui renderer backend)

Comparison of `eframe` running on **wgpu** (WebGPU/WebGL2 via `wgpu-hal`) vs **glow** (OpenGL/WebGL via `glutin`/`glow` crate).

Measured on: Linux container, 32 cores, 63 GB RAM, rustc 1.97.1, cargo 1.97.1, egui/eframe **0.35**.

## What changed (this is a Cargo-feature-only change, zero source edits)

| File | Change |
|---|---|
| `../imanot/Cargo.toml` | `eframe = { ..., features = ["wgpu", ...] }` → `features = ["glow", "default_fonts"]` |
| `../pilatus-leptos/Cargo.toml` | `eframe = "0.35"` (default = wgpu) → `{ default-features = false, features = ["accesskit","default_fonts","glow","wayland","web_screen_reader","x11"] }` |
| `Cargo.toml` (here) | Uncommented `[patch.".../pilatus-leptos"]` so this app builds against the local glow-based `pilatus-leptos` |

Both the annotation tool (`annotation-tool`) and the embedded viewer already go through `egui::Context`/`eframe`, so the renderer is swappable without touching Rust code.

## Native build — `annotation-tool-app` (x86_64-unknown-linux-gnu, `--no-default-features --features wayland`)

Cold build time (fresh `cargo clean`, 3 runs, release profile):

| Backend | run 1 | run 2 | run 3 | median |
|---|---|---|---|---|
| wgpu | 25.8 s | 33.7 s | 33.4 s | **33.4 s** |
| glow  | 26.8 s | 26.3 s | 27.6 s | **26.8 s** |

Binary size:

| Backend | size |
|---|---|
| wgpu | 22 206 672 B (~21.2 MiB) |
| glow  | 15 053 768 B (~14.4 MiB) |
| **savings** | **−7.2 MB, −32%** |

## Wasm build — annotation tool (`wasm32-unknown-unknown`, `--no-default-features`)

Cold build time (3 runs, release):

| Backend | run 1 | run 2 | run 3 | median |
|---|---|---|---|---|
| wgpu | 28.4 s | 28.1 s | 28.8 s | **28.4 s** |
| glow  | 27.0 s | 19.8 s | 19.0 s | **19.8 s** |

`.wasm` payload sizes (both the lib cdylib and the bin; + wasm-opt `-O` / `-Oz` / gzip, matching how trunk ships it with `data-wasm-opt="z"`):

| artifact | wgpu bytes | glow bytes | diff |
|---|---|---|---|
| lib raw | 1 590 466 | 1 255 010 | −21% |
| lib `-O` | 1 212 679 | 970 636 | −20% |
| lib `-Oz` | 1 194 575 | 960 198 | −20% |
| lib `-O`+gzip | 266 616 | 216 742 | −19% |
| bin raw | 11 359 906 | 7 144 567 | −37% |
| bin `-O` | 8 482 007 | 5 603 082 | −34% |
| bin `-Oz` | 8 347 249 | 5 553 909 | −33% |
| bin `-O`+gzip | 3 146 906 | 2 152 394 | −32% |

## Shipped app — feeder-os frontend `app` binary wasm (release, cold build)

The app that actually ships (leptos frontend with embedded eframe viewport):

| metric | wgpu | glow | diff |
|---|---|---|---|
| build time | 81.7 s | 60.6 s | **−26%** |
| wasm raw | 22 161 741 | 17 857 232 | −19% |
| wasm `-Oz` | 12 768 483 | 9 915 987 | **−22%** |
| wasm `-Oz`+gzip | 4 503 065 | 3 507 176 | **−22%** |

## Interpretation

- **Binary size**: consistent, significant win for glow on every target — roughly **−30 to −35%** native, **−20 to −35%** wasm (even after `-Oz` + gzip). wgpu/naga/wgpu-core make up the difference; on wasm the WebGPU/JS glue is also heavy.
- **Compile time**: glow is **~20% faster** native and **~30% faster** wasm (single-digit seconds on this 32-core box). Incremental rebuilds after touching egui/eframe are affected similarly.
- Runtime is not measured here (no display in the container). Ones generally to check after switching: glow uses an OpenGL/WebGL context (OpenGL 2.0+/WebGL1), wgpu used WebGPU-first with WebGL fallback on web. For these apps (image viewers, 2D masks) egui+glow is the long-standing default path and is functionally equivalent. Native glow additionally requires a GL/EGL library present at link time (e.g. `libglvnd`'s `libEGL.so`/`libGL.so`; on the container I passed `LIBRARY_PATH` to the nix-store `libglvnd`).

## Caveats

- Compile numbers are cold `cargo clean` rebuilds; run-to-run variance is ±2–3 s (the 25.8 s wgpu outlier was a warm page-cache run).
- The annotation tool's default `sam` feature (onnxruntime/`ort-sys`) fails to link in this container because `ort`'s `download-binaries` feature is off — independent of wgpu vs glow (fails identically for both backends), so all renderer measurements were done with `sam` disabled to isolate the renderer.
- To revert: flip the three `eframe` lines back to `wgpu` / defaults and comment out the `[patch.".../pilatus-leptos"]` block above.