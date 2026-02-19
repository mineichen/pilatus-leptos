# Project Summary: pilatus-leptos-app

## Project Type
**Rust WASM Frontend Application** using the Leptos framework (v0.8) with Client-Side Rendering (CSR). Part of the "Pilatus" workspace - an industrial camera control application.

## Tech Stack
| Technology | Details |
|------------|---------|
| Language | Rust (Edition 2024) |
| Framework | Leptos v0.8 (CSR mode) |
| Target | `wasm32-unknown-unknown` |
| Build Tool | Trunk (WASM bundler) |
| UI Library | Thaw (git dependency) |
| Styling | Tailwind CSS v4 + SCSS |
| Routing | leptos_router v0.8 |
| Testing | Playwright (e2e) |

## Key Dependencies
- `pilatus` - Core domain library (git)
- `pilatus-leptos` - Shared Leptos utilities (workspace member)
- `thaw` / `thaw_utils` - UI components
- `gloo-net` / `gloo-timers` - Web APIs for WASM
- `serde` / `serde_json` - Serialization

## Directory Structure
```
src/
├── main.rs              # Entry point - mounts app to body
├── lib.rs               # Module exports
├── app.rs               # Main App component with routing
├── home.rs              # Home page view
├── nav.rs               # Navigation sidebar
├── point.rs             # Demo Point component
├── busy_button.rs       # Demo async button
└── recipe_management/   # Recipe feature module
    ├── recipe_row.rs    # Recipe table row component
    └── recipe_tags.rs   # Tag management component

end2end/                 # Playwright e2e tests
public/                  # Static assets
dist/                    # Build output
```

## Build & Development Commands

```bash
# Development (via justfile)
just dev      # Frontend + backend in parallel
just devf     # Frontend only: trunk serve --features examples
just devb     # Backend only

# Manual
trunk serve                    # Dev server with hot reload
trunk serve --features examples # With demo components
trunk build                    # Production build

# Testing
cd end2end && npx playwright test
```

## Architecture Patterns

### Component Pattern
```rust
#[component]
pub fn ComponentName(props: Type) -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();
    let action = Action::new_local(move |_: &()| async move { /* ... */ });
    view! { /* JSX-like template */ }
}
```

### State Management
- **RecipeContext**: Central state via Leptos context system
- **Signals**: `RwSignal`, `Memo`, `Signal` for reactivity
- **WebSocket**: Real-time sync with debounced auto-save (250ms)

### Routing
```rust
<Router>
  <Routes>
    <Route path="" view=HomeView/>
    <Route path="recipes" view=RecipeManagement/>
    <ParentRoute path="/device/:device_id" view=DeviceView>
      // Dynamic child routes based on device_type
    </ParentRoute>
  </Routes>
</Router>
```

## Code Conventions

- **Files**: snake_case (`recipe_management.rs`)
- **Components**: PascalCase (`RecipeManagement`)
- **Functions/Variables**: snake_case
- **One component per file**
- **Inline styles**: `attr:style="..."`
- **Tailwind classes**: `class="mt-4"`
- **NO comments** in code unless explicitly requested

## Feature Flags

| Flag | Purpose |
|------|---------|
| `aravis` | Aravis camera support |
| `emulation-camera` | Camera emulation (default) |
| `examples` | Demo components |

## Backend Connection

- API: `http://localhost:4123`
- WebSocket: `/api/recipe/stream`, `/api/image/subscribe`
- Proxied via Trunk in development

## Key Entry Points

| File | Purpose |
|------|---------|
| `src/main.rs` | App entry - mounts to body |
| `src/app.rs` | Main component, routing setup |
| `src/recipe_management.rs` | Recipe management feature |
| `pilatus-leptos/src/recipe_context.rs` | Context provider |

## Routes

| Route | Component |
|-------|-----------|
| `/` | `HomeView` |
| `/recipes` | `RecipeManagement` |
| `/device/:device_id/:device_type` | `JsonDeviceView` |
| `/device/:device_id/pilatus-aravis` | `AravisView` |
| `/device/:device_id/pilatus-emulation-camera` | `EmulationCameraView` |

## Workspace Structure

This project is part of a larger workspace at `/workspace/pilatus-leptos/`:
- `pilatus-leptos-app/` - This frontend app
- `pilatus-leptos/` - Shared Leptos utilities
- `pilatus-aravis-leptos/` - Aravis camera UI
- `pilatus-emulation-camera-leptos/` - Camera emulation UI
- `pilatus-engineering-leptos/` - Engineering tools UI
- `app-backend/` - Backend API server
