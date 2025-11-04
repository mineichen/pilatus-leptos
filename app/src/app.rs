use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
};
use pilatus_examples_leptos::{Greeter, ManualTick};
use pilatus_leptos::{DeviceView, ProvideDeviceContext, RecipeView};
use thaw::{Anchor, AnchorLink, ConfigProvider, Layout, LayoutHeader, LayoutSider};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Title text="FeederOS"/>
        <ConfigProvider>
            <ProvideDeviceContext>
                <Layout has_sider=true>
                    <LayoutSider attr:style="background-color: #0078ff99; padding: 20px;">
                    <Anchor>
                    <AnchorLink title="Home" href="/" />
                            <AnchorLink title="Greeter" href="/device/bc505bd1-1818-4d97-a4cb-2b698b657000/greeter" />
                            <AnchorLink title="ManualTick" href="/device/e8e8eb2d-2325-4a40-aba7-7d223d39fe83/manual_tick" />
                            </Anchor>
                            </LayoutSider>
                            <Layout>
                            <LayoutHeader attr:style="background-color: #0078ffaa; padding: 20px;">
                            <h1>"Welcome to Leptos!"</h1>
                            </LayoutHeader>
                            <Layout attr:style="background-color: #0078ff88; padding: 20px;">
                            <Router>
                                <Routes fallback=|| "Page not found.".into_view()>
                                    <Route path=StaticSegment("") view=RecipeView/>
                                    <ParentRoute path=leptos_router::path!("/device/:device_id") view=DeviceView>
                                        <Route path=StaticSegment("greeter") view=Greeter/>
                                        <Route path=StaticSegment("manual_tick") view=ManualTick />
                                    </ParentRoute>
                                </Routes>
                            </Router>
                        </Layout>
                    </Layout>
                </Layout>
             </ProvideDeviceContext>
        </ConfigProvider>
    }
}
