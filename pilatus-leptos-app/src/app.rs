use leptos::children::ToChildren;
use leptos::prelude::*;
use leptos_meta::{Html, Title, provide_meta_context};
use leptos_router::{MatchNestedRoutes, NestedRoute};
use leptos_router::{
    StaticSegment,
    components::{A, ParentRoute, Route, Router, Routes},
};
use pilatus_leptos::{
    DeviceView, JsonDeviceView, Notifications, ProvideDeviceContext, ProvidePilatusContext,
};
use pilatus_leptos_components::{ThemeMode, ThemeToggle};

use crate::{home::HomeView, nav::Nav, recipe_management::RecipeManagement};

#[component]
pub fn App<Extra>(extra_device_routes: Extra) -> impl IntoView
where
    Extra: MatchNestedRoutes + Send + Clone + 'static,
{
    provide_meta_context();
    let theme_mode = ThemeMode::init();

    view! {
        <Title text="Pilatus Control Panel"/>
        <Html {..} class=move || if theme_mode.is_dark() { "dark" } else { "" } />
        <ProvidePilatusContext>
            <ProvideDeviceContext>
                <Router>
                <div class="flex h-screen bg-background text-foreground">
                    <aside class="w-64 bg-card border-r border-border flex flex-col shrink-0 py-4">
                        <div class="px-6 mb-4">
                            <h1 class="text-xl font-bold text-foreground">"Pilatus"</h1>
                            <p class="text-xs text-muted-foreground mt-1">"Industrial Control"</p>
                            <img src=move || if theme_mode.is_dark() { "/api/logo?theme=dark" } else { "/api/logo?theme=light" } alt="Logo" class="mt-2" />
                        </div>
                        <Nav />
                        <div class="px-3 mt-auto pt-4 border-t border-border mx-3">
                            <A href="/recipes" attr:class="flex items-center gap-3 px-4 py-3 rounded-lg text-foreground hover:bg-accent transition-colors">
                                <span>"⚙️"</span>
                                <span>"Recipe Management"</span>
                            </A>
                        </div>
                    </aside>
                    <main class="flex-1 flex flex-col min-w-0">
                        <header class="h-14 px-6 flex items-center justify-between border-b border-border bg-card">
                            <span class="text-muted-foreground text-sm">"Control Panel"</span>
                            <ThemeToggle/>
                        </header>
                        <div class="flex-1 overflow-auto p-6">
                            <Routes fallback=|| "Page not found.".into_view()>
                                <Route path=StaticSegment("") view=HomeView/>
                                <Route path=StaticSegment("recipes") view=RecipeManagement/>
                                <ParentRoute
                                    path=leptos_router::path!("/device/:device_id")
                                    view=DeviceView
                                    children=ToChildren::to_children(move || (
                                        #[cfg(feature = "examples")]
                                        NestedRoute::new(StaticSegment("pilatus-greeter"), pilatus_examples_leptos::Greeter),
                                        #[cfg(feature = "examples")]
                                        NestedRoute::new(
                                            StaticSegment("pilatus-manual-tick"),
                                            pilatus_examples_leptos::ManualTick,
                                        ),
                                        #[cfg(feature = "emulation-camera")]
                                        NestedRoute::new(
                                            StaticSegment("pilatus-emulation-camera"),
                                            pilatus_emulation_camera_leptos::EmulationCameraView,
                                        ),
                                        #[cfg(feature = "aravis")]
                                        NestedRoute::new(
                                            StaticSegment("pilatus-aravis"),
                                            pilatus_aravis_leptos::AravisView,
                                        ),
                                        extra_device_routes.clone(),
                                        NestedRoute::new(leptos_router::path!("/:device_type"), || view! {<JsonDeviceView/> }),
                                    ))
                                />
                            </Routes>
                        </div>
                    </main>
                </div>
            </Router>
            </ProvideDeviceContext>
            <Notifications />
        </ProvidePilatusContext>
    }
}
