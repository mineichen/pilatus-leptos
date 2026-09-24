use leptos::prelude::*;
use tw_merge::tw_merge;

use crate::InputNumberValue;

/// Fill fraction (0..1) of `v` within `[min_f, max_f]` for the track
/// gradient in `style.scss`. Out-of-range values saturate, degenerate
/// ranges and non-finite math fall back to 0.
pub(crate) fn fill_frac(v: f64, min_f: f64, max_f: f64) -> f64 {
    if max_f > min_f {
        let frac = ((v - min_f) / (max_f - min_f)).clamp(0.0, 1.0);
        if frac.is_finite() {
            return frac;
        }
    }
    0.0
}

/// A range slider bound directly to a numeric signal, mirroring
/// [`crate::InputNumber`]: it accepts any [`InputNumberValue`] (`u32`,
/// `f32`, …) instead of only `f64`, so callers bind their signal directly
/// instead of maintaining an `f64` mirror plus sync effects.
///
/// Unlike the RustUI registry `Slider`
/// (<https://github.com/rust-ui/leptos-ui/blob/main/app_crates/registry/src/ui/slider.rs>),
/// which renders an unbound `<input type="range">`, this component is
/// controlled: it syncs the DOM via `prop:value` / `on:input`. Values are
/// clamped to the required `[min, max]` range.
///
/// The look lives in `style.scss`: a white handle with a `--primary` ring
/// on a thin track with a value fill. All paint is static; the only dynamic
/// piece is the `--slider-fill` fraction below, which positions the fill.
///
/// The optional children render as the label above the slider:
///
/// ```ignore
/// <Slider value=my_value min=0 max=100 step=1>"Threshold"</Slider>
/// ```
#[component]
pub fn Slider<T, V>(
    // Controlled value in slider units.
    value: V,

    // Range bounds (required); passed through to the native input and used
    // for clamping. Plain `T` (no `into`) so numeric literals infer to the
    // value type (`min=0` with a `u32` signal, `min=0.0` with an `f32`
    // one); `impl Into<T>` would pin literals to `i32`/`f64` instead.
    min: T,
    max: T,

    // Step; passed through to the native input.
    #[prop(optional)] step: Option<T>,

    // Styling of the field wrapper (label + slider).
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] style: String,

    // Label rendered above the slider.
    #[prop(optional)] children: Option<Children>,

    #[prop(into, optional)] disabled: Signal<bool>,
) -> impl IntoView
where
    T: InputNumberValue,
    V: Get<Value = T> + Set<Value = T> + Clone + Send + Sync + 'static,
{
    let min_f = min.to_f64();
    let max_f = max.to_f64();
    let min_attr = min.format();
    let max_attr = max.format();
    let step_attr = step.map(T::format);

    let wrap_class = tw_merge!("flex min-w-0 flex-1 flex-col gap-1", class);

    let input_class = "w-full cursor-pointer disabled:cursor-not-allowed disabled:opacity-50";

    let value_for_prop = value.clone();
    let value_for_fill = value.clone();
    let value_for_input = value;

    // The only dynamic styling on the element: the fill fraction for the
    // track gradient (Gecko uses its native progress instead and ignores
    // the var). All paint stays in `style.scss`.
    let fill_style = move || {
        format!(
            "--slider-fill: {:.4};",
            fill_frac(value_for_fill.get().to_f64(), min_f, max_f)
        )
    };

    view! {
        <div data-name="SliderField" class=wrap_class style=style>
            <input
                data-name="Slider"
                type="range"
                class=input_class
                style=fill_style
                min=min_attr
                max=max_attr
                step=step_attr
                disabled=disabled
                prop:value=move || value_for_prop.get().to_f64()
                on:input=move |event| {
                    let Ok(raw) = event_target_value(&event).parse::<f64>() else {
                        return;
                    };
                    if !raw.is_finite() {
                        return;
                    }
                    let clamped = if raw < min_f {
                        min_f
                    } else if raw > max_f {
                        max_f
                    } else {
                        raw
                    };
                    if let Some(v) = T::from_f64(clamped) {
                        value_for_input.set(v);
                    }
                }
            />
            {children.map(|label| view! {
                <label style="color: var(--muted-foreground); font-size: 13px;">{label()}</label>
            })}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::fill_frac;

    #[test]
    fn fill_fraction_tracks_value() {
        assert_eq!(fill_frac(60.0, 0.0, 100.0), 0.6);
        assert_eq!(fill_frac(0.0, 0.0, 100.0), 0.0);
        assert_eq!(fill_frac(100.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn fill_fraction_saturates_and_falls_back() {
        assert_eq!(fill_frac(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(fill_frac(150.0, 0.0, 100.0), 1.0);
        assert_eq!(fill_frac(f64::INFINITY, 0.0, 100.0), 1.0);
        assert_eq!(fill_frac(f64::NAN, 0.0, 100.0), 0.0);
        assert_eq!(fill_frac(50.0, 100.0, 100.0), 0.0);
    }
}
