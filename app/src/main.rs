use app::*;

#[cfg(feature = "ssr")]
fn main() {
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use leptos_meta::MetaTags;
    use thaw::ssr::SSRMountStyleProvider;

    pilatus_rt::Runtime::with_root("data")
        .register(pilatus_axum_rt::register)
        .register(register)
        .run();

    fn shell(options: LeptosOptions) -> impl IntoView {
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

    extern "C" fn register(c: &mut minfac::ServiceCollection) {
        use minfac::Registered;

        c.with::<Registered<pilatus_axum::Stats>>().register(|s| {
            Box::new(|x: axum::Router| {
                let leptos_routes = generate_route_list(App);
                let conf = get_configuration(None).unwrap();
                axum::Router::new()
                    .leptos_routes(&conf.leptos_options, leptos_routes, {
                        let leptos_options = conf.leptos_options.clone();
                        move || shell(leptos_options.clone())
                    })
                    .fallback(leptos_axum::file_and_error_handler(shell))
                    .with_state(conf.leptos_options)
                    .merge(x)
            }) as Box<dyn FnOnce(axum::Router) -> axum::Router>
        });
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    use leptos::{logging, mount};

    console_error_panic_hook::set_once();
    logging::log!("csr mode - mounting to body");
    mount::mount_to_body(App);
}
