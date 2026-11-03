# Plan: Reduce dependencies

Status: **plan only, nothing implemented yet.**

## Baseline (wasm32-unknown-unknown, `cargo tree -e normal`)

| Crate | Unique crates |
|---|---|
| `pilatus-leptos-components` | ~200 |
| `pilatus-engineering-leptos` | ~302 (+~100: egui/eframe, imask/imanot) |
| `pilatus-leptos-app` | ~410 |
| `pilatus-engineering-leptos` on host (linux x86_64) | ~456 (+~65 native eframe) |

Measure again after each step with:
`cargo tree -p pilatus-leptos-app --target wasm32-unknown-unknown --prefix none -e normal | cut -d' ' -f1 | sort -u | wc -l`

## Decisions

- **Do not deduplicate** versions (`gloo-net` 0.6/0.7, `thiserror`, `syn`, ...). Leptos 0.9 (rc2 released) is expected to
  bring up-to-date deps, so this resolves itself with the upgrade.
- **Keep `pilatus`** as a dependency of the UI crates.
- **Drop native eframe entirely.** Nothing works outside leptos/web.

## Steps

### 1. Remove native eframe (~65 crates on host builds, 0 on wasm)
- Delete the `[target.'cfg(all(target_os = "linux", target_arch = "x86_64"))'.dependencies]` block in
  `pilatus-engineering-leptos/Cargo.toml` (it enables `x11` -> `winit`, `wayland-*`, `x11rb`, `glutin`, `arboard`, ...).
- Remove the "Important to enable native build for winit" comment.
- Check that nothing in the code uses native-only eframe APIs (`eframe::run_native`, `NativeOptions`, ...).
  Only `WebRunner`/`WebOptions` should remain (`image_viewer/app.rs`).
- Verify: `cargo check` on host still works (host check only needs the web API stubs eframe provides without `x11`),
  otherwise document that the engineering crate is checked with `--target wasm32-unknown-unknown`.

### 2. Trim egui/eframe features (expected: tens of crates)
Currently enabled: `eframe` = `default_fonts`, `glow`; `egui` = `default`, `default_fonts`, `bytemuck`
(the `default` on egui is enabled by `imanot`, which depends on egui itself).
- Workspace `Cargo.toml`: `egui = { version = "0.36", default-features = false }`; add only what the code needs.
- `imanot` (own crate, `../imanot`): set `default-features = false` on its egui dependency, otherwise it re-enables defaults.
- Candidates to drop (verify each by building): egui `accesskit` (-> `accesskit`), `default_fonts` if a custom/system font is
  acceptable (-> `epaint_default_fonts`), `log`, `bytemuck` unless imanot needs it.
- eframe: keep only `glow` (or `wgpu`, see `WGPU_VS_GLOW.md`). Check whether `default_fonts` is needed.
- `image` + `png` (-> `png`, `flate2`, `fdeflate`, `crc32fast`, `miniz_oxide`, `moxcms`) is pulled in via `eframe`.
  Find which eframe feature enables it (`cargo tree -e features -i image`) and drop it if possible. The
  workspace's own `image` dep already has `default-features = false`.
- Check what pulls in `vello_cpu`/`peniko`/`skrifa`/`kurbo`/`harfrust` (via `epaint`). These might be
  gated behind a font/text feature in egui 0.36.

### 3. Leptos: hydration and server_fn are NOT accidentally enabled
Findings on leptos 0.8.21 for the app (features enabled: `csr`, `default`, `nightly`):
- `server_fn`, `hydration_context`, `leptos_server`, `leptos_config`, `serde_qs`, `url` are **unconditional** dependencies of
  `leptos` 0.8 / `leptos_router`. No feature of ours turns them on and we cannot switch them off.
- `server_fn` is built with `browser` + `default` + `form-redirects`, all set by leptos itself.
- Action: none for 0.8. Re-check after the 0.9 upgrade whether they became optional.
- Small check: who enables the `leptos` feature `nightly`? (`cargo tree -e features -i leptos`). It is not in our manifests;
  find the source and drop it if unintended (needs a nightly toolchain).

### 4. Upgrade to Leptos 0.9 (rc2)
- Bump `leptos`, `leptos_router`, `leptos_meta`, `leptos_axum`, `reactive_graph`, `reactive_stores` together.
- Re-measure; check `leptos_ui`/`tw_merge` compatibility.

### 5. Own crates (`imask`, `imanot`)
- `imask`: `rkyv` (+ `munge`, `rancor`, `ptr_meta`, `bytecheck`, `rend`, `seahash`...) and `range-set-blaze`; `nalgebra` via both.
  Check whether rkyv is behind a feature the viewer doesn't need, and whether `nalgebra` can be feature-gated or replaced by
  the few operations used (`approx`/`simba`/`matrixmultiply`/`num-*` come with it).
- `pilatus-engineering-leptos` itself: `chrono`, `gloo-timers`, `gloo-events`, `pin-project-lite`: confirm each is actually used
  (`cargo machete` / `cargo udeps`).

### 6. General hygiene
- Run `cargo machete` (or `cargo +nightly udeps`) across the workspace for unused declared dependencies.
- `pilatus-leptos-app`: `tracing-log` + `tracing-subscriber` (registry only) are small; keep unless unused.

## Expected result
Host builds: -~65 crates (step 1). wasm: -a few dozen from egui features (step 2), possibly more from the Leptos 0.9 upgrade
and trimming `imask`/`imanot`. The remaining ~200-crate Leptos baseline is not reducible by us.
