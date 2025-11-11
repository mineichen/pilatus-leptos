use leptos::prelude::*;
use leptos_meta::{Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
};
use pilatus_examples_leptos::{Greeter, ManualTick};
use pilatus_leptos::{DeviceView, JsonDeviceView, ProvideDeviceContext};
use thaw::{ConfigProvider, Layout, LayoutHeader, LayoutSider};

use crate::{home::HomeView, nav::Nav};

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

                        <Nav />
                    </LayoutSider>
                    <Layout>
                        <LayoutHeader attr:style="background-color: #0078ffaa; padding: 20px;">
                            <h1>"Welcome to Leptos!"</h1>
                        </LayoutHeader>
                        <Layout attr:style="background-color: #0078ff88; padding: 20px;">
                            <Router>
                                <Routes fallback=|| "Page not found.".into_view()>
                                    <Route path=StaticSegment("") view=HomeView/>
                                    <ParentRoute path=leptos_router::path!("/device/:device_id") view=DeviceView>
                                        <Route path=StaticSegment("greeter") view=Greeter/>
                                        <Route path=StaticSegment("manual_tick") view=ManualTick />
                                        <Route path=leptos_router::path!("/:device_type") view=JsonDeviceView/>
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
