use leptos::context::Provider;
use leptos::ev::keydown;
use leptos::prelude::*;
use leptos_ui::clx;
use tw_merge::*;

clx! {DialogBody, div, "flex flex-col gap-4"}
clx! {DialogHeader, div, "flex flex-col gap-2 text-center sm:text-left"}
clx! {DialogTitle, h3, "text-lg leading-none font-semibold"}
clx! {DialogDescription, p, "text-muted-foreground text-sm"}
clx! {DialogFooter, footer, "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end"}

#[derive(Clone, Copy)]
struct DialogContext {
    show: RwSignal<bool>,
}

/// Controlled dialog root. `show` drives visibility, scroll lock and ESC handling.
/// Render `<DialogTrigger>` / `<DialogContent>` inside.
#[component]
pub fn Dialog(
    children: Children,
    show: RwSignal<bool>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let body_style = || {
        web_sys::window()
            .expect("Window exists")
            .document()
            .expect("Document exists")
            .body()
            .expect("Body exists")
            .style()
    };

    Effect::new(move |_| {
        if show.get() {
            body_style().set_property("overflow", "hidden").ok();
        } else {
            body_style().remove_property("overflow").ok();
        }
    });
    on_cleanup(move || {
        let _ = body_style().remove_property("overflow");
    });
    window_event_listener(keydown, move |event| {
        if event.key() == "Escape" {
            show.set(false);
        }
    });

    let ctx = DialogContext { show };
    let merged_class = tw_merge!("w-fit", class);

    view! {
        <Provider value=ctx>
            <div class=merged_class>
                {children()}
            </div>
        </Provider>
    }
}

/// Centered modal content + backdrop. Mounted only while `show` is true.
/// No JS involved, purely signal-driven.
#[component]
pub fn DialogContent(
    children: ChildrenFn,
    #[prop(optional, into)] class: String,
    #[prop(default = true)] show_close_button: bool,
    #[prop(default = true)] close_on_backdrop_click: bool,
) -> impl IntoView {
    let ctx = expect_context::<DialogContext>();
    let show = ctx.show;
    let merged_class = tw_merge!(
        "relative bg-background border rounded-2xl shadow-lg p-6 w-full max-w-[calc(100%-2rem)] max-h-[85vh] overflow-hidden flex flex-col gap-4 fixed top-[50%] left-[50%] translate-x-[-50%] translate-y-[-50%] z-100",
        class
    );

    let close_class = if show_close_button {
        "absolute top-4 right-4 p-1 rounded-sm cursor-pointer focus:ring-2 focus:ring-offset-2 focus:outline-none [&_svg:not([class*='size-'])]:size-4 focus:ring-ring".to_string()
    } else {
        "absolute top-4 right-4 p-1 rounded-sm cursor-pointer focus:ring-2 focus:ring-offset-2 focus:outline-none [&_svg:not([class*='size-'])]:size-4 focus:ring-ring hidden".to_string()
    };

    view! {
        {move || {
            let children_inner = children.clone();
            let merged_inner = merged_class.clone();
            let close_inner = close_class.clone();
            show.get().then(move || {
                view! {
                    <div
                        class="fixed inset-0 z-60 bg-black/50"
                        on:click=move |_| {
                            if close_on_backdrop_click {
                                show.set(false);
                            }
                        }
                    />
                    <div
                        class=merged_inner
                        role="dialog"
                        aria-modal="true"
                    >
                        <button
                            type="button"
                            class=close_inner
                            on:click=move |_| show.set(false)
                            aria-label="Close dialog"
                        >
                            <span class="hidden">"Close Dialog"</span>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                width="24"
                                height="24"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                class="size-4"
                            >
                                <path d="M18 6 6 18" />
                                <path d="m6 6 12 12" />
                            </svg>
                        </button>
                        {children_inner()}
                    </div>
                }
            })
        }}
    }
}
