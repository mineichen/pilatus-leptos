use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use pilatus_leptos::RecipeView;
use thaw::{ConfigProvider, Layout, LayoutHeader, LayoutSider};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/app.css"/>
        <Title text="FeederOS"/>
        <ConfigProvider>
            <Layout has_sider=true>
                <LayoutSider attr:style="background-color: #0078ff99; padding: 20px;">
                    "Sider"
                </LayoutSider>
                <Layout>
                    <LayoutHeader attr:style="background-color: #0078ffaa; padding: 20px;">
                        <h1>"Welcome to Leptos!"</h1>
                    </LayoutHeader>
                    <Layout attr:style="background-color: #0078ff88; padding: 20px;">
                        <Router>
                            <Routes fallback=|| "Page not found.".into_view()>
                                <Route path=StaticSegment("") view=RecipeView/>
                            </Routes>
                        </Router>
                    </Layout>
                </Layout>
            </Layout>
        </ConfigProvider>
    }
}
