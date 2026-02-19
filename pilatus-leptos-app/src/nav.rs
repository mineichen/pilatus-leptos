use leptos::prelude::*;
use leptos_router::components::A;
use pilatus_leptos::RecipeContext;

#[component]
pub fn Nav() -> impl IntoView {
    let ctx = expect_context::<RecipeContext>();

    view! {
        <nav class="flex-1 overflow-y-auto">
            <div class="px-6 pt-4 pb-2">
                <span class="text-xs font-semibold text-slate-500 uppercase tracking-wider">"Devices"</span>
            </div>
            <div class="px-3 space-y-1">
                <For
                    each=move || ctx.list_devices().get()
                    key=|x| x.device_id
                    let(x)>
                    <A
                        href=format!("/device/{}/{}", x.device_id, x.device_type)
                        attr:class="flex items-center gap-3 px-4 py-3 rounded-lg text-slate-300 hover:text-white hover:bg-slate-800 transition-colors"
                    >
                        <span class="w-2.5 h-2.5 rounded-full bg-emerald-500 shrink-0"></span>
                        <span class="truncate">{x.name.to_string()}</span>
                    </A>
                </For>
            </div>
        </nav>
    }
}
