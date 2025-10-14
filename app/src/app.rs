use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use pilatus_leptos::RecipeView;
use thaw::{ConfigProvider, Layout, LayoutHeader, LayoutSider, ssr::SSRMountStyleProvider};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <AutoReload options=options.clone() />
                    <HydrationScripts options/>
                    <MetaTags/>
                </head>
                <body>
                    <App/>
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/app.css"/>



        // sets the document title
        <Title text="FeederOS"/>

        // content for this welcome page
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
