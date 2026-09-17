use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Textarea<V>(
    // Controlled value: anything readable and writable (`RwSignal`,
    // `LeafRwSignal`, `MapRwSignal`, `FrozenSignal`, ...). No local
    // `RwSignal` is created; the DOM syncs directly via `prop:value`.
    value: V,

    // Styling
    #[prop(into, optional)] class: String,

    // Common HTML attributes
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] readonly: bool,
    #[prop(optional)] required: bool,
    #[prop(optional)] autofocus: bool,
    #[prop(optional)] rows: Option<u32>,

    // Ref for direct DOM access
    #[prop(optional)] node_ref: NodeRef<html::Textarea>,
) -> impl IntoView
where
    V: Get<Value = String> + Set<Value = String> + Clone + Send + Sync + 'static,
{
    let merged_class = tw_merge!(
        "border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-2 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
        class
    );

    let value_for_prop = value.clone();
    let value_for_input = value;

    view! {
        <textarea
            data-name="Textarea"
            class=merged_class
            placeholder=placeholder
            name=name
            id=id
            disabled=disabled
            readonly=readonly
            required=required
            autofocus=autofocus
            rows=rows
            prop:value=move || value_for_prop.get()
            on:input=move |event| value_for_input.set(event_target_value(&event))
            node_ref=node_ref
        />
    }
}
