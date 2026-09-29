use std::ops::RangeInclusive;

use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

use crate::InputNumberValue;
use crate::slider::fill_frac;

/// Clamp a raw slider reading to `[min_f, max_f]` and convert it to `T`.
/// Returns `None` for non-finite input or inconvertible values, in which
/// case the event is ignored.
fn clamp_to_bounds<T: InputNumberValue>(raw: f64, min_f: f64, max_f: f64) -> Option<T> {
    if !raw.is_finite() {
        return None;
    }
    let clamped = if raw < min_f {
        min_f
    } else if raw > max_f {
        max_f
    } else {
        raw
    };
    T::from_f64(clamped)
}

/// Apply one handle drag to the current `(min, max)` pair, keeping the
/// `min <= max` invariant: dragging a handle past the other one swaps
/// their roles (min becomes max, max becomes min), so the dragged handle
/// keeps following the pointer.
fn apply_drag<T: InputNumberValue>(is_min: bool, v: T, cur_lo: T, cur_hi: T) -> (T, T) {
    if is_min {
        if v <= cur_hi {
            (v, cur_hi)
        } else {
            (cur_hi, v)
        }
    } else if v >= cur_lo {
        (cur_lo, v)
    } else {
        (v, cur_lo)
    }
}

/// One bound field of [`InputRange`]: a numeric input in the
/// [`crate::InputNumber`] card style (`.inr-input` with its `-` / `+`
/// steppers), rendered as a bare fragment so the parent's `.inr-range-top`
/// grid places all six controls of both bounds in one strip
/// (`min-field/−/+`-divider-`max-field/−/+`). Driven by a read signal plus
/// a commit callback. The parent order-normalizes every commit against the
/// other bound.
#[component]
fn RangeField<T>(
    // Current bound value (displayed, and reverted to on invalid input).
    value: Signal<T>,
    // Stepper step; passed through to the native input.
    step: T,

    // Full-range numeric bounds, passed through to the native input and
    // used for clamping typed/stepped values.
    min: T,
    max: T,

    #[prop(optional)] disabled: bool,

    // Receives validated values; the parent keeps `min <= max`.
    commit: Callback<T>,
) -> impl IntoView
where
    T: InputNumberValue,
{
    // Hide the browser's native number-input spinners (we render our own
    // `-`/`+` steppers). Same as [`crate::InputNumber`].
    const NO_NATIVE_SPINNERS: &str = "[-moz-appearance:textfield] [appearance:textfield] \
        [&::-webkit-outer-spin-button]:m-0 [&::-webkit-outer-spin-button]:appearance-none \
        [&::-webkit-inner-spin-button]:m-0 [&::-webkit-inner-spin-button]:appearance-none";

    let node_ref = NodeRef::<html::Input>::new();

    let min_attr = min.format();
    let max_attr = max.format();
    let step_attr = step.format();

    // Verbatim [`crate::InputNumber`] field paint; the strip geometry comes
    // from the parent's `.inr-range-top` grid, so no rounding joins here.
    let input_class = tw_merge!("inr-input", NO_NATIVE_SPINNERS);

    let clamp = move |next: T| -> T {
        if next < min {
            min
        } else if next > max {
            max
        } else {
            next
        }
    };

    let number_from_dom = move || {
        node_ref
            .get()
            .map(|input| input.value_as_number())
            .filter(|value| value.is_finite())
    };

    let value_for_commit = value;
    let commit_for_commit = commit;
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
                    commit_for_commit.run(clamped);
                }
            }
            None => revert(),
        }
    };

    let value_for_nudge = value;
    let commit_for_nudge = commit;
    let nudge = move |direction: f64| {
        let current = number_from_dom().unwrap_or_else(|| value_for_nudge.get().to_f64());
        let raw = current + direction * step.to_f64();
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
        commit_for_nudge.run(clamp(next));
    };

    let nudge_down = nudge;
    let step_down = move |_| nudge_down(-1.0);
    let step_up = move |_| nudge(1.0);

    // Same steppers as [`crate::InputNumber`]: plain buttons with the
    // `.inr-step` paint and mark spans (no `Button` wrapper). A bare
    // fragment — the parent grid owns the strip geometry.
    view! {
        <input
            data-name="InputNumberField"
            type="number"
            class=input_class
            prop:value=move || value.get().format()
            on:change=move |event| commit_text(event_target_value(&event))
            disabled=disabled
            min=min_attr
            max=max_attr
            step=step_attr
            node_ref=node_ref
        />
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
}

/// A range input in the [`crate::InputNumber`] card style: one shared
/// `.inr-card` whose top strip holds both bound fields with their own `-` /
/// `+` steppers (`min-field/−/+`-divider-`max-field/−/+`) and whose bottom
/// edge carries a single value track highlighting the span between the two
/// handles — all bound to a single range value, mirroring
/// [`crate::InputNumber`]: it accepts any [`InputNumberValue`] (`u32`,
/// `f32`, …) instead of only `f64`.
///
/// The bound value accepts anything convertible into a range: `value`
/// reads through `Into<RangeInclusive<T>>` and commits through
/// `From<RangeInclusive<T>>` (a plain `RangeInclusive<T>` signal works
/// directly via the reflexive impls).
///
/// Both handles are real `<input type="range">` elements stacked over the
/// shared track (see `style.scss`): their own tracks stay transparent so no
/// input can paint over the other's thumb; the visible track is the
/// `.inr-track` div with its `.inr-track-fill` span, exactly like
/// [`crate::InputNumber`]. Dragging one handle past the other swaps their
/// roles instead of crossing. Typed or stepped field values are clamped to
/// the full bounds and order-normalized the same way.
///
/// The optional children render as the group label above the card; the
/// `min_label` / `max_label` captions render in a row above the card:
///
/// ```ignore
/// <InputRange value=my_range min=0.0 max=255.0 step=1.0 min_label="Weak" max_label="Strong"/>
/// ```
#[component]
#[allow(clippy::too_many_lines, reason = "Leptos component: view logic inline")]
pub fn InputRange<T, R, V>(
    // Controlled range value: anything readable and writable (`RwSignal`,
    // `MapRwSignal`, …) holding a range-like value.
    value: V,

    // Range bounds (required); passed through to the native inputs and used
    // for clamping. Plain `T` (no `into`) so numeric literals infer to the
    // value type (`min=0.0` with an `f32` range); `impl Into<T>` would pin
    // literals to `f64` instead.
    min: T,
    max: T,

    // Step; passed through to the native inputs and steppers.
    #[prop(optional)] step: Option<T>,

    // Small captions above the min/max fields (omit either to hide it).
    #[prop(into, optional)] min_label: String,
    #[prop(into, optional)] max_label: String,

    // Styling of the component wrapper.
    #[prop(into, optional)] class: String,
    #[prop(into, optional)] style: String,

    // Group label rendered above the fields.
    #[prop(optional)] children: Option<Children>,

    #[prop(optional)] disabled: bool,
) -> impl IntoView
where
    T: InputNumberValue,
    R: Into<RangeInclusive<T>> + From<RangeInclusive<T>> + Clone + Send + Sync + 'static,
    V: Get<Value = R> + Set<Value = R> + Clone + Send + Sync + 'static,
{
    // Same handle paint as every `Slider` (see `style.scss`); both tracks
    // stay transparent so no input can paint over the other's thumb — the
    // single visible track is the shared `.inr-track` div below.
    const INPUT_CLASS: &str =
        "w-full cursor-pointer disabled:cursor-not-allowed disabled:opacity-50 slider-no-track";

    let min_f = min.to_f64();
    let max_f = max.to_f64();
    let min_attr = min.format();
    let max_attr = max.format();
    let step_attr = step.map(T::format);
    // The fields always need a step; fall back to 1 (the native range
    // default) when none is set.
    let step_value = step.or_else(|| T::from_f64(1.0)).unwrap_or(min);

    let wrap_class = tw_merge!("flex min-w-0 flex-1 flex-col gap-1", class);

    let bounds_of = move |source: &V| -> (T, T) {
        source.get().into().into_inner()
    };

    let value_for_lo = value.clone();
    let lo_value = Signal::derive(move || value_for_lo.get().into().into_inner().0);
    let value_for_hi = value.clone();
    let hi_value = Signal::derive(move || value_for_hi.get().into().into_inner().1);

    // Order-normalizing commit for one bound: values stay within the full
    // bounds (the fields clamp first) and `min <= max` (a commit past the
    // other bound swaps their roles, like a drag).
    let value_for_lo_commit = value.clone();
    let commit_lo = Callback::new(move |v: T| {
        let (cur_lo, cur_hi) = bounds_of(&value_for_lo_commit);
        let (lo, hi) = apply_drag(true, v, cur_lo, cur_hi);
        if lo != cur_lo || hi != cur_hi {
            value_for_lo_commit.set(R::from(lo..=hi));
        }
    });
    let value_for_hi_commit = value.clone();
    let commit_hi = Callback::new(move |v: T| {
        let (cur_lo, cur_hi) = bounds_of(&value_for_hi_commit);
        let (lo, hi) = apply_drag(false, v, cur_lo, cur_hi);
        if lo != cur_lo || hi != cur_hi {
            value_for_hi_commit.set(R::from(lo..=hi));
        }
    });

    let value_for_lo_prop = value.clone();
    let lo_prop = move || bounds_of(&value_for_lo_prop).0.to_f64();
    let value_for_hi_prop = value.clone();
    let hi_prop = move || bounds_of(&value_for_hi_prop).1.to_f64();

    // The shared track highlights exactly the span between the handles
    // (sorted for display; interaction order is kept by `apply_drag`) as
    // an `.inr-track-fill` span positioned by `left`/`width`, like
    // [`crate::InputNumber`]. The dual inputs' own tracks stay transparent.
    let value_for_track = value.clone();
    let track_style = move || {
        let (lo, hi) = bounds_of(&value_for_track);
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        let lo_f = fill_frac(lo.to_f64(), min_f, max_f);
        let hi_f = fill_frac(hi.to_f64(), min_f, max_f);
        format!(
            "left: {:.4}%; width: {:.4}%;",
            lo_f * 100.0,
            (hi_f - lo_f).max(0.0) * 100.0,
        )
    };
    let track_class = if disabled {
        "inr-track inr-track-disabled"
    } else {
        "inr-track"
    };

    let value_for_min = value.clone();
    let on_min_input = move |event| {
        let Ok(raw) = event_target_value(&event).parse::<f64>() else {
            return;
        };
        let Some(v) = clamp_to_bounds::<T>(raw, min_f, max_f) else {
            return;
        };
        let (cur_lo, cur_hi) = bounds_of(&value_for_min);
        let (lo, hi) = apply_drag(true, v, cur_lo, cur_hi);
        if lo != cur_lo || hi != cur_hi {
            value_for_min.set(R::from(lo..=hi));
        }
    };
    let value_for_max = value;
    let on_max_input = move |event| {
        let Ok(raw) = event_target_value(&event).parse::<f64>() else {
            return;
        };
        let Some(v) = clamp_to_bounds::<T>(raw, min_f, max_f) else {
            return;
        };
        let (cur_lo, cur_hi) = bounds_of(&value_for_max);
        let (lo, hi) = apply_drag(false, v, cur_lo, cur_hi);
        if lo != cur_lo || hi != cur_hi {
            value_for_max.set(R::from(lo..=hi));
        }
    };

    let min_label = (!min_label.is_empty()).then_some(min_label);
    let max_label = (!max_label.is_empty()).then_some(max_label);
    let has_bound_labels = min_label.is_some() || max_label.is_some();

    view! {
        <div data-name="InputRange" class=wrap_class style=style>
            {children.map(|label| view! {
                <label style="color: var(--muted-foreground); font-size: 13px;">{label()}</label>
            })}
            {has_bound_labels
                .then(|| view! {
                    <div class="flex w-full min-w-0 gap-0">
                        <div class="flex min-w-0 flex-1 flex-col gap-1">
                            {min_label
                                .map(|text| view! {
                                    <label style="color: var(--muted-foreground); font-size: 13px;">{text}</label>
                                })}
                        </div>
                        <div class="flex min-w-0 flex-1 flex-col gap-1">
                            {max_label
                                .map(|text| view! {
                                    <label style="color: var(--muted-foreground); font-size: 13px;">{text}</label>
                                })}
                        </div>
                    </div>
                })}
            <div class="inr-card" role="group" aria-label="Range slider">
                <div class="inr-top inr-range-top">
                    <RangeField
                        value=lo_value
                        step=step_value
                        min=min
                        max=max
                        disabled=disabled
                        commit=commit_lo
                    />
                    <RangeField
                        value=hi_value
                        step=step_value
                        min=min
                        max=max
                        disabled=disabled
                        commit=commit_hi
                    />
                </div>
                <div class=track_class>
                    <div class="inr-track-fill" style=track_style></div>
                </div>
                <input
                    data-name="Slider"
                    type="range"
                    class=INPUT_CLASS
                    min=min_attr.clone()
                    max=max_attr.clone()
                    step=step_attr.clone()
                    disabled=disabled
                    prop:value=lo_prop
                    on:input=on_min_input
                />
                <input
                    data-name="Slider"
                    type="range"
                    class=INPUT_CLASS
                    min=min_attr
                    max=max_attr
                    step=step_attr
                    disabled=disabled
                    prop:value=hi_prop
                    on:input=on_max_input
                />
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_drag, clamp_to_bounds};

    #[test]
    fn dragging_min_within_bounds_moves_only_min() {
        assert_eq!(apply_drag(true, 20u32, 10, 90), (20, 90));
        assert_eq!(apply_drag(true, 10u32, 10, 90), (10, 90));
    }

    #[test]
    fn dragging_min_past_max_swaps_roles() {
        assert_eq!(apply_drag(true, 95u32, 10, 90), (90, 95));
    }

    #[test]
    fn dragging_max_within_bounds_moves_only_max() {
        assert_eq!(apply_drag(false, 80u32, 10, 90), (10, 80));
        assert_eq!(apply_drag(false, 90u32, 10, 90), (10, 90));
    }

    #[test]
    fn dragging_max_below_min_swaps_roles() {
        assert_eq!(apply_drag(false, 5u32, 10, 90), (5, 10));
    }

    #[test]
    fn swap_is_symmetric_for_floats() {
        assert_eq!(apply_drag(true, 200.0f32, 12.0, 150.0), (150.0, 200.0));
        assert_eq!(apply_drag(false, 5.0f32, 12.0, 150.0), (5.0, 12.0));
    }

    #[test]
    fn raw_readings_clamp_to_bounds() {
        assert_eq!(clamp_to_bounds::<u32>(60.0, 0.0, 100.0), Some(60));
        assert_eq!(clamp_to_bounds::<u32>(-5.0, 0.0, 100.0), Some(0));
        assert_eq!(clamp_to_bounds::<u32>(150.0, 0.0, 100.0), Some(100));
        assert_eq!(clamp_to_bounds::<u32>(f64::NAN, 0.0, 100.0), None);
        assert_eq!(clamp_to_bounds::<f32>(12.5, 0.0, 255.0), Some(12.5));
    }
}
