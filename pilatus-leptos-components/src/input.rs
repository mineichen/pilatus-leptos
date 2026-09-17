use leptos::html;
use leptos::prelude::*;
use strum::AsRefStr;
use tw_merge::tw_merge;

#[derive(Default, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum InputType {
    #[default]
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
    Search,
    Date,
    Time,
    #[strum(serialize = "datetime-local")]
    DatetimeLocal,
    Month,
    Week,
    Color,
    File,
    Hidden,
}

#[component]
pub fn Input<V>(
    // Controlled value: anything readable and writable (`RwSignal`,
    // `LeafRwSignal`, `MapRwSignal`, `FrozenSignal`, ...). No local
    // `RwSignal` is created; the DOM syncs directly via `prop:value`.
    value: V,

    // Styling
    #[prop(into, optional)] class: String,

    // Common HTML attributes
    #[prop(default = InputType::default())] r#type: InputType,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] autocomplete: Option<String>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] readonly: Signal<bool>,
    #[prop(into, optional)] required: Signal<bool>,
    #[prop(into, optional)] autofocus: Signal<bool>,
    #[prop(optional)] minlength: Option<u16>,

    // Number input attributes
    #[prop(into, optional)] min: Option<String>,
    #[prop(into, optional)] max: Option<String>,
    #[prop(into, optional)] step: Option<String>,

    // Ref for direct DOM access
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView
where
    V: Get<Value = String> + Set<Value = String> + Clone + Send + Sync + 'static,
{
    let merged_class = tw_merge!(
        "text-foreground file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
        "focus-visible:border-ring focus-visible:ring-ring/50",
        "focus-visible:ring-2",
        "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
        "read-only:bg-muted",
        class
    );

    let type_str = r#type.as_ref().to_owned();
    let value_for_prop = value.clone();
    let value_for_input = value;

    view! {
        <input
            data-name="Input"
            type=type_str
            class=merged_class
            placeholder=placeholder
            name=name
            id=id
            title=title
            autocomplete=autocomplete
            disabled=disabled
            readonly=readonly
            required=required
            autofocus=autofocus
            minlength=minlength
            min=min
            max=max
            step=step
            prop:value=move || value_for_prop.get()
            on:input=move |event| value_for_input.set(event_target_value(&event))
            node_ref=node_ref
        />
    }
}
