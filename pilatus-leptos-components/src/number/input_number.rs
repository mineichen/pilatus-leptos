use std::ops::RangeInclusive;

use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

use super::InputNumberValue;

/// Fill fraction (0..1) of a display-space `value` within
/// `[start, end]`. Degenerate ranges and non-finite inputs fall back to 0.
fn slider_fill(value: f64, start: f64, end: f64) -> f64 {
    if value.is_finite() && start.is_finite() && end.is_finite() && end > start {
        ((value - start) / (end - start)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Value at a pointer fraction (0..1) along `[start, end]`, clamped to the
/// range. Returns `None` for degenerate ranges and non-finite fractions.
fn value_at_fraction<T: InputNumberValue>(fraction: f64, start: T, end: T) -> Option<T> {
    if !fraction.is_finite() {
        return None;
    }
    let (lo, hi) = (start.to_f64(), end.to_f64());
    if !(lo.is_finite() && hi.is_finite() && hi > lo) {
        return None;
    }
    T::from_f64(lo + fraction.clamp(0.0, 1.0) * (hi - lo)).map(|v| {
        if v < start {
            start
        } else if v > end {
            end
        } else {
            v
        }
    })
}

/// Numeric slider card: a number field with `-` / `+` stepper buttons on
/// top and — when `range` is set — a value track with fill and thumb along
/// the bottom edge. Paint lives in `style.scss` (`.inr-*` rules ported 1:1
/// from the reference mockup); no paint rules here, only structure.
///
/// Typing commits on `change` (clamped to `min`/`max`, falling back to the
/// last valid value); the steppers nudge by `step`; the track drags and
/// arrow-keys through `range` (clamped to the range, `Home`/`End` jump to
/// its ends). `value` is controlled: anything readable and writable
/// (`RwSignal`, `LeafRwSignal`, `MapRwSignal`, `FrozenSignal`, ...). No
/// local `RwSignal` is created for the value; the DOM is the source of
/// intermediate text.
#[component]
pub fn InputNumber<T, V>(
    // Controlled value: anything readable and writable (`RwSignal`,
    // `LeafRwSignal`, `MapRwSignal`, `FrozenSignal`, ...). No local
    // `RwSignal` is created; the DOM is the source of intermediate text and
    // commits on `change` (plus stepper clicks).
    value: V,

    // Styling
    #[prop(into, optional)] class: String,

    // Common HTML attributes
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] name: Option<String>,
    #[prop(into, optional)] id: Option<String>,
    #[prop(into, optional)] title: Option<String>,
    #[prop(into, optional)] autocomplete: Option<String>,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] readonly: bool,
    #[prop(optional)] required: bool,
    #[prop(optional)] autofocus: bool,

    // Numeric bounds and step; the steppers only render when `step` is set.
    #[prop(optional)] min: Option<T>,
    #[prop(optional)] max: Option<T>,
    #[prop(optional)] step: Option<T>,

    // Slider travel. The slider only shows up when this is set explicitly
    // (`Some`); it drags through the same `value`, clamped to the range.
    // Empty ranges (`start > end`) render no slider.
    #[prop(optional)] range: Option<RangeInclusive<T>>,

    // Ref for direct DOM access
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView
where
    T: InputNumberValue,
    V: Get<Value = T> + Set<Value = T> + Clone + Send + Sync + 'static,
{
    // Hide the browser's native number-input spinners (the reference uses
    // a plain text input for the same reason). `textfield` covers Firefox,
    // the `::-webkit-*` selectors cover Chromium/Safari.
    const NO_NATIVE_SPINNERS: &str = "[-moz-appearance:textfield] [appearance:textfield] \
        [&::-webkit-outer-spin-button]:m-0 [&::-webkit-outer-spin-button]:appearance-none \
        [&::-webkit-inner-spin-button]:m-0 [&::-webkit-inner-spin-button]:appearance-none";

    let min_attr = min.map(T::format);
    let max_attr = max.map(T::format);
    let step_attr = step.map(T::format);
    let has_steppers = step.is_some();

    // Slider travel. The track only shows up when a non-empty range is set
    // explicitly (`Some`); it drags through the same `value`, clamped to
    // the range. `T: Copy`, so splitting the bounds out here still leaves
    // `range` usable below.
    let travel = range
        .filter(|r| r.start() <= r.end())
        .map(RangeInclusive::into_inner);
    let has_slider = travel.is_some();

    let card_class = if has_slider {
        tw_merge!("inr-card", class)
    } else {
        tw_merge!("inr-card inr-card-plain", class)
    };
    let top_class = if has_steppers {
        "inr-top"
    } else {
        "inr-top inr-top-single"
    };
    let input_class = tw_merge!("inr-input", NO_NATIVE_SPINNERS);

    let clamp = move |next: T| -> T {
        let lo = min.unwrap_or(T::MIN);
        let hi = max.unwrap_or(T::MAX);
        if next < lo {
            lo
        } else if next > hi {
            hi
        } else {
            next
        }
    };

    // Browser API (`valueAsNumber`) carries the locale-aware parse, so
    // float separators (`.` vs `,`) are handled by the browser instead of
    // Rust-side text parsing. Ints parse the text themselves.
    let number_from_dom = move || {
        node_ref
            .get()
            .map(|input| input.value_as_number())
            .filter(|value| value.is_finite())
    };

    let value_for_prop = value.clone();
    let value_for_commit = value.clone();
    let value_for_nudge = value.clone();

    let commit_text = move |text: String| {
        let revert = || {
            if let Some(input) = node_ref.get() {
                // Non-numeric text: fall back to the last valid value.
                input.set_value(&value_for_commit.get().format());
            }
        };
        match T::parse_input(&text, number_from_dom()) {
            Some(parsed) => {
                let clamped = clamp(parsed);
                if clamped == value_for_commit.get() {
                    // Normalize display (e.g. "1.50" -> "1.5").
                    revert();
                } else {
                    value_for_commit.set(clamped);
                }
            }
            None => revert(),
        }
    };

    let nudge = move |direction: f64| {
        let Some(step) = step else {
            return;
        };
        let current = number_from_dom().unwrap_or_else(|| value_for_nudge.get().to_f64());
        let raw = current + direction * step.to_f64();
        // Finite check lives here, outside the trait: non-finite stepper
        // arithmetic saturates to the nearest limit.
        let next = if raw.is_finite() {
            match T::from_f64(raw) {
                Some(parsed) => parsed,
                None => return,
            }
        } else if raw.is_sign_positive() {
            T::MAX
        } else {
            T::MIN
        };
        value_for_nudge.set(clamp(next));
    };

    let nudge_down = nudge.clone();
    let step_down = move |_| nudge_down(-1.0);
    let step_up = move |_| nudge(1.0);

    // Value track along the card's bottom edge (reference `.track` with
    // `.track-fill`) plus the `.thumb`: a natively focusable
    // `role="slider"` div with pointer drag (captured, like the
    // reference) and arrow / `Home` / `End` keys. The thumb is a sibling
    // of the track (not a child) so the track can clip the fill to its
    // rounded corners while the thumb overhangs the card unclipped (the
    // card is `overflow: visible`). Fill width and thumb position arrive
    // as inline percentages; all paint lives in `style.scss`.
    let slider = travel.map(|(start, end)| {
        let (lo, hi) = (start.to_f64(), end.to_f64());
        // The slider always needs a step for the arrow keys; fall back to
        // 1 (the native range default) when the field has no steppers.
        let slider_step = step.or_else(|| T::from_f64(1.0)).unwrap_or(start);

        let value_for_fill = value.clone();
        let fill_style = move || {
            format!(
                "width: {:.4}%;",
                slider_fill(value_for_fill.get().to_f64(), lo, hi) * 100.0
            )
        };
        let value_for_thumb = value.clone();
        let thumb_style = move || {
            format!(
                "left: {:.4}%;",
                slider_fill(value_for_thumb.get().to_f64(), lo, hi) * 100.0
            )
        };
        let value_for_aria = value.clone();
        let aria_now = move || value_for_aria.get().format();

        // Pointer drag state; moves are only honored between down and up.
        let dragging = RwSignal::new(false);

        let value_for_down = value.clone();
        let on_pointer_down = move |ev: leptos::ev::PointerEvent| {
            if disabled {
                return;
            }
            dragging.set(true);
            let track: web_sys::Element = event_target(&ev);
            // Keep receiving moves outside the track while dragging.
            let _ = track.set_pointer_capture(ev.pointer_id());
            let rect = track.get_bounding_client_rect();
            if rect.width() > 0.0 {
                let fraction = (f64::from(ev.client_x()) - rect.left()) / rect.width();
                if let Some(next) = value_at_fraction(fraction, start, end)
                    && next != value_for_down.get()
                {
                    value_for_down.set(next);
                }
            }
        };

        let value_for_move = value.clone();
        let on_pointer_move = move |ev: leptos::ev::PointerEvent| {
            if disabled || !dragging.get() {
                return;
            }
            let track: web_sys::Element = event_target(&ev);
            let rect = track.get_bounding_client_rect();
            if rect.width() <= 0.0 {
                return;
            }
            let fraction = (f64::from(ev.client_x()) - rect.left()) / rect.width();
            if let Some(next) = value_at_fraction(fraction, start, end)
                && next != value_for_move.get()
            {
                value_for_move.set(next);
            }
        };
        let on_pointer_up = move |_| dragging.set(false);

        // Arrow keys nudge by one slider step, `Home`/`End` jump to the
        // range ends (the reference moves integer values by 1, which is
        // its step).
        let value_for_keys = value.clone();
        let nudge_track = move |direction: f64| {
            let raw = value_for_keys.get().to_f64() + direction * slider_step.to_f64();
            if !raw.is_finite() {
                return;
            }
            if let Some(next) = T::from_f64(raw) {
                let clamped = if next < start {
                    start
                } else if next > end {
                    end
                } else {
                    next
                };
                if clamped != value_for_keys.get() {
                    value_for_keys.set(clamped);
                }
            }
        };
        let value_for_ends = value.clone();
        let on_key_down = move |ev: leptos::ev::KeyboardEvent| {
            if disabled {
                return;
            }
            match ev.key().as_str() {
                "ArrowRight" | "ArrowUp" => {
                    ev.prevent_default();
                    nudge_track(1.0);
                }
                "ArrowLeft" | "ArrowDown" => {
                    ev.prevent_default();
                    nudge_track(-1.0);
                }
                "Home" => {
                    ev.prevent_default();
                    if start != value_for_ends.get() {
                        value_for_ends.set(start);
                    }
                }
                "End" => {
                    ev.prevent_default();
                    if end != value_for_ends.get() {
                        value_for_ends.set(end);
                    }
                }
                _ => {}
            }
        };

        let track_class = if disabled {
            "inr-track inr-track-disabled"
        } else {
            "inr-track"
        };
        // A disabled track leaves the tab order.
        let track_tabindex = if disabled { -1 } else { 0 };

        view! {
            <div
                class=track_class
                role="slider"
                tabindex=track_tabindex
                aria-label="Value"
                aria-valuemin=start.format()
                aria-valuemax=end.format()
                aria-valuenow=aria_now
                on:pointerdown=on_pointer_down
                on:pointermove=on_pointer_move
                on:pointerup=on_pointer_up
                on:pointercancel=on_pointer_up
                on:keydown=on_key_down
            >
                <div class="inr-track-fill" style=fill_style></div>
            </div>
            <div class="inr-thumb" style=thumb_style></div>
        }
    });

    view! {
        <div class=card_class role="group" aria-label="Numeric slider" data-name="InputNumber">
            <div class=top_class>
                <input
                    data-name="InputNumberField"
                    type="number"
                    class=input_class
                    prop:value=move || value_for_prop.get().format()
                    on:change=move |event| commit_text(event_target_value(&event))
                    placeholder=placeholder
                    name=name
                    id=id
                    title=title
                    autocomplete=autocomplete
                    disabled=disabled
                    readonly=readonly
                    required=required
                    autofocus=autofocus
                    min=min_attr
                    max=max_attr
                    step=step_attr
                    node_ref=node_ref
                />
                {has_steppers
                    .then(|| {
                        view! {
                            <button
                                type="button"
                                class="inr-step inr-minus"
                                disabled=disabled
                                aria-label="Decrease value"
                                on:click=step_down
                            >
                                <span class="inr-minus-mark"></span>
                            </button>
                            <button
                                type="button"
                                class="inr-step inr-plus"
                                disabled=disabled
                                aria-label="Increase value"
                                on:click=step_up
                            >
                                <span class="inr-plus-mark"></span>
                            </button>
                        }
                    })}
            </div>
            {slider}
        </div>
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use crate::InputNumberValue;

    #[test]
    fn text_parsing_trims_and_rejects_garbage() {
        assert_eq!(i32::parse_input("  -7 ", None), Some(-7));
        assert_eq!(i32::parse_input("abc", None), None);
        assert_eq!(i32::parse_input("", None), None);
        assert_eq!(i32::parse_input("2.5", Some(2.5)), Some(3));
        assert_eq!(f64::parse_input("0,5", Some(0.5)), Some(0.5));
        assert_eq!(f64::parse_input("0.5", None), None);
    }

    #[test]
    fn floats_format_with_at_most_three_decimals() {
        assert_eq!(1.0f64.format(), "1");
        assert_eq!(1.5f32.format(), "1.5");
        assert_eq!(0.005f32.format(), "0.005");
        assert_eq!(0.1239f64.format(), "0.124");
        assert_eq!(80.10f64.format(), "80.1");
    }

    #[test]
    fn integers_format_exactly() {
        assert_eq!(42u32.format(), "42");
        assert_eq!((-7i32).format(), "-7");
    }

    #[test]
    fn percentage_uses_display_units_with_one_decimal_max() {
        let pct = pilatus::Percentage::new(0.8).unwrap();
        assert_eq!(pct.to_f64(), 80.0);
        assert_eq!(pct.format(), "80");
        let half = pilatus::Percentage::new(0.805).unwrap();
        assert_eq!(half.format(), "80.5");
        assert_eq!(
            pilatus::Percentage::from_f64(80.5).unwrap(),
            pilatus::Percentage::new(0.805).unwrap()
        );
        assert_eq!(
            pilatus::Percentage::parse_input("", Some(80.0)).unwrap(),
            pilatus::Percentage::new(0.8).unwrap()
        );
        assert_eq!(pilatus::Percentage::MIN.format(), "0");
        assert_eq!(pilatus::Percentage::MAX.format(), "100");
        // Stepper/display space saturates at the bounds.
        assert_eq!(
            pilatus::Percentage::from_f64(150.0).unwrap(),
            pilatus::Percentage::max()
        );
        assert_eq!(
            pilatus::Percentage::from_f64(-5.0).unwrap(),
            pilatus::Percentage::min()
        );
    }

    #[test]
    fn slider_fill_tracks_and_clamps_value() {
        use super::slider_fill;
        assert_eq!(slider_fill(50.0, 0.0, 100.0), 0.5);
        assert_eq!(slider_fill(0.0, 0.0, 100.0), 0.0);
        assert_eq!(slider_fill(100.0, 0.0, 100.0), 1.0);
        assert_eq!(slider_fill(-5.0, 0.0, 100.0), 0.0);
        assert_eq!(slider_fill(150.0, 0.0, 100.0), 1.0);
    }

    #[test]
    fn slider_fill_falls_back_for_degenerate_ranges() {
        use super::slider_fill;
        assert_eq!(slider_fill(50.0, 100.0, 100.0), 0.0);
        assert_eq!(slider_fill(50.0, 100.0, 0.0), 0.0);
        assert_eq!(slider_fill(f64::NAN, 0.0, 100.0), 0.0);
        assert_eq!(slider_fill(50.0, f64::NAN, 100.0), 0.0);
    }

    #[test]
    fn pointer_fraction_maps_to_range_value() {
        use super::value_at_fraction;
        assert_eq!(value_at_fraction(0.5, 0u32, 100), Some(50));
        assert_eq!(value_at_fraction(0.0, 0u32, 100), Some(0));
        assert_eq!(value_at_fraction(1.0, 0u32, 100), Some(100));
        // Out-of-track fractions clamp to the range.
        assert_eq!(value_at_fraction(-0.2, 0u32, 100), Some(0));
        assert_eq!(value_at_fraction(1.5, 0u32, 100), Some(100));
        // Display units for percentages: halfway is 50%.
        assert_eq!(
            value_at_fraction(0.5, pilatus::Percentage::min(), pilatus::Percentage::max()),
            pilatus::Percentage::new(0.5)
        );
    }

    #[test]
    fn pointer_fraction_rejects_degenerate_geometry() {
        use super::value_at_fraction;
        assert_eq!(value_at_fraction(0.5, 100u32, 100), None);
        assert_eq!(value_at_fraction(0.5, 100u32, 0), None);
        assert_eq!(value_at_fraction(f64::NAN, 0u32, 100), None);
        assert_eq!(value_at_fraction(f64::INFINITY, 0u32, 100), None);
    }
}
