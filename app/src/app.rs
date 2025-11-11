use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{A, ParentRoute, Route, Router, Routes},
};
use pilatus_examples_leptos::{Greeter, ManualTick};
use pilatus_leptos::{DeviceView, JsonDeviceView, ProvideDeviceContext};
use thaw::{Button, ButtonSize, ConfigProvider, Layout, LayoutHeader, LayoutSider};

use crate::{home::HomeView, nav::Nav, recipe_management::RecipeManagement};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Title text="FeederOS"/>
        <ConfigProvider>
            <ProvideDeviceContext>
                <Router>
                    <Layout has_sider=true>
                        <LayoutSider attr:style="background-color: #0078ff99; padding: 20px;">
                            <Nav />
                        </LayoutSider>
                        <Layout>
                            <LayoutHeader attr:style="background-color: #0078ffaa; padding: 20px; display: flex; justify-content: space-between; align-items: center;">
                                <h1>"Welcome to Leptos!"</h1>
                                <A href="/recipes" attr:style="text-decoration: none;">
                                    <Button size=ButtonSize::Large attr:style="font-size: 24px; padding: 8px 16px;">
                                        "⚙️"
                                    </Button>
                                </A>
                            </LayoutHeader>
                            <Layout attr:style="background-color: #0078ff88; padding: 20px;">
                                <Routes fallback=|| "Page not found.".into_view()>
                                    <Route path=StaticSegment("") view=HomeView/>
                                    <Route path=StaticSegment("recipes") view=RecipeManagement/>
                                    <ParentRoute path=leptos_router::path!("/device/:device_id") view=DeviceView>
                                        <Route path=StaticSegment("greeter") view=Greeter/>
                                        <Route path=StaticSegment("manual_tick") view=ManualTick />
                                        <Route path=leptos_router::path!("/:device_type") view=JsonDeviceView/>
                                    </ParentRoute>
                                </Routes>
                            </Layout>
                        </Layout>
                    </Layout>
                </Router>
             </ProvideDeviceContext>
        </ConfigProvider>
    }
}
