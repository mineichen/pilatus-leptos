use std::fmt::Display;

use leptos::html;
use leptos::prelude::*;
use tw_merge::tw_merge;

use crate::button::{Button, ButtonSize, ButtonVariant};

/// Numeric value accepted by [`InputNumber`].
///
/// Implemented for the primitive integer and float types, so the component
/// accepts multiple numeric values (`i32`, `u32`, `f64`, …).
pub trait InputNumberValue:
    Copy + PartialEq + PartialOrd + Display + Send + Sync + 'static
{
    /// Smallest representable value (fallback lower clamp).
    const MIN: Self;
    /// Largest representable value (fallback upper clamp).
    const MAX: Self;
    /// Convert from `f64` (used for stepper arithmetic).
    ///
    /// The caller guarantees `value.is_finite()`; out-of-range values
    /// saturate to `Self::MIN`/`Self::MAX` instead of failing.
    fn from_f64(value: f64) -> Option<Self>;
    /// Convert to `f64` (used for stepper arithmetic).
    fn to_f64(self) -> f64;
    /// Parse user-typed text.
    ///
    /// `dom_number` is the browser's `input.valueAsNumber()` with non-finite
    /// values already filtered out. Floats must use it because decimal
    /// separators differ around the world (`.` vs `,`) and Rust's `parse`
    /// only understands `.`. Ints parse `text` themselves and ignore
    /// `dom_number`. Returns `None` for non-numeric text so the caller can
    /// fall back to the last valid value.
    fn parse_input(text: &str, dom_number: Option<f64>) -> Option<Self>;
}

macro_rules! impl_input_number_value_for_int {
    ($($ty:ty),*) => {
        $(
            impl InputNumberValue for $ty {
                const MIN: Self = <$ty>::MIN;
                const MAX: Self = <$ty>::MAX;

                fn parse_input(text: &str, _dom_number: Option<f64>) -> Option<Self> {
                    let parsed: f64 = text.trim().parse().ok()?;
                    if !parsed.is_finite() {
                        return None;
                    }
                    Self::from_f64(parsed)
                }

                fn from_f64(value: f64) -> Option<Self> {
                    // Caller guarantees a finite value; saturate instead of failing.
                    let rounded = value.round();
                    if rounded <= <$ty>::MIN as f64 {
                        Some(<$ty>::MIN)
                    } else if rounded >= <$ty>::MAX as f64 {
                        Some(<$ty>::MAX)
                    } else {
                        #[allow(clippy::cast_possible_truncation)]
                        Some(rounded as Self)
                    }
                }

                fn to_f64(self) -> f64 {
                    self as f64
                }
            }
        )*
    };
}

impl_input_number_value_for_int!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);

impl InputNumberValue for f32 {
    const MIN: Self = f32::MIN;
    const MAX: Self = f32::MAX;

    fn parse_input(_text: &str, dom_number: Option<f64>) -> Option<Self> {
        dom_number.and_then(Self::from_f64)
    }

    fn from_f64(value: f64) -> Option<Self> {
        // Caller guarantees a finite value; saturate instead of failing
        // (`as f32` of a huge-but-finite `f64` would overflow to infinity).
        if value >= f32::MAX as f64 {
            Some(f32::MAX)
        } else if value <= f32::MIN as f64 {
            Some(f32::MIN)
        } else {
            Some(value as f32)
        }
    }

    fn to_f64(self) -> f64 {
        f64::from(self)
    }
}

impl InputNumberValue for f64 {
    const MIN: Self = f64::MIN;
    const MAX: Self = f64::MAX;

    fn parse_input(_text: &str, dom_number: Option<f64>) -> Option<Self> {
        dom_number.and_then(Self::from_f64)
    }

    fn from_f64(value: f64) -> Option<Self> {
        // Caller guarantees a finite value.
        Some(value)
    }

    fn to_f64(self) -> f64 {
        self
    }
}

/// Numeric input mirroring [`crate::Input`] (same props and styling), with
/// joined `-` / `+` stepper buttons on the sides whenever `step` is set.
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

    // Ref for direct DOM access
    #[prop(optional)] node_ref: NodeRef<html::Input>,
) -> impl IntoView
where
    T: InputNumberValue,
    V: Get<Value = T> + Set<Value = T> + Clone + Send + Sync + 'static,
{
    let min_attr = min.map(|v| v.to_string());
    let max_attr = max.map(|v| v.to_string());
    let step_attr = step.map(|v| v.to_string());
    let has_steppers = step.is_some();

    // Hide the browser's native number-input spinners (we render our own
    // `-`/`+` steppers). `textfield` covers Firefox, the `::-webkit-*`
    // selectors cover Chromium/Safari.
    const NO_NATIVE_SPINNERS: &str = "[-moz-appearance:textfield] [appearance:textfield] \
        [&::-webkit-outer-spin-button]:m-0 [&::-webkit-outer-spin-button]:appearance-none \
        [&::-webkit-inner-spin-button]:m-0 [&::-webkit-inner-spin-button]:appearance-none";

    // Same RustUI input styling as `crate::Input`; with steppers the inner
    // corners stay square so the joined row reads as one control.
    let input_class = if has_steppers {
        tw_merge!(
            "text-foreground file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 flex-1 rounded-none border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
            "focus-visible:border-ring focus-visible:ring-ring/50",
            "focus-visible:ring-2",
            "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
            "read-only:bg-muted",
            "-ml-px focus:z-10",
            NO_NATIVE_SPINNERS,
            class
        )
    } else {
        tw_merge!(
            "text-foreground file:text-foreground placeholder:text-muted-foreground selection:bg-primary selection:text-primary-foreground dark:bg-input/30 border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs transition-[color,box-shadow] outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
            "focus-visible:border-ring focus-visible:ring-ring/50",
            "focus-visible:ring-2",
            "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
            "read-only:bg-muted",
            NO_NATIVE_SPINNERS,
            class
        )
    };

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
                let _ = input.set_value(&value_for_commit.get().to_string());
            }
        };
        match T::parse_input(&text, number_from_dom()) {
            Some(parsed) => {
                let clamped = clamp(parsed);
                if clamped != value_for_commit.get() {
                    value_for_commit.set(clamped);
                } else {
                    // Normalize display (e.g. "1.50" -> "1.5").
                    revert();
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

    // RustUI pattern (`button_action.rs`): no `disabled` prop on `Button`,
    // disable via a `pointer-events-none` wrapper instead.
    let stepper_wrap = move || {
        if disabled {
            "shrink-0 pointer-events-none opacity-50"
        } else {
            "shrink-0"
        }
    };

    view! {
        <div class="flex w-full min-w-0 items-stretch gap-0" data-name="InputNumber">
            {has_steppers
                .then(|| {
                    view! {
                        <span class=stepper_wrap>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Icon
                                class="rounded-r-none"
                                on:click=step_down
                            >
                                "−"
                            </Button>
                        </span>
                    }
                })}
            <input
                data-name="InputNumberField"
                type="number"
                class=input_class
                prop:value=move || value_for_prop.get().to_string()
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
                        <span class=stepper_wrap>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Icon
                                class="rounded-l-none -ml-px"
                                on:click=step_up
                            >
                                "+"
                            </Button>
                        </span>
                    }
                })}
        </div>
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::InputNumberValue;

    #[test]
    fn integers_round_half_away_from_zero() {
        assert_eq!(i8::parse_input("3", Some(3.0)), Some(3));
        assert_eq!(i8::from_f64(2.5), Some(3));
        assert_eq!(i8::from_f64(-2.5), Some(-3));
        assert_eq!(u32::from_f64(2.4), Some(2));
        assert_eq!(i32::from_f64(-2.6), Some(-3));
    }

    #[test]
    fn integers_reject_non_finite_input() {
        assert_eq!(i8::parse_input("nan", None), None);
        assert_eq!(i8::parse_input("inf", None), None);
        assert_eq!(u8::parse_input("abc", None), None);
        assert_eq!(i32::parse_input("", None), None);
        assert_eq!(f64::parse_input("0.5", None), None);
    }

    #[test]
    fn integers_saturate_out_of_range_values() {
        assert_eq!(i8::from_f64(128.0), Some(127));
        assert_eq!(i8::from_f64(-129.0), Some(-128));
        assert_eq!(u8::from_f64(256.0), Some(255));
        assert_eq!(u8::from_f64(-1.0), Some(0));
        assert_eq!(u64::from_f64(-1.0), Some(0));
        assert_eq!(i8::parse_input("999", None), Some(127));
        assert_eq!(
            i64::from_f64(9_007_199_254_740_992.0),
            Some(9_007_199_254_740_992)
        );
    }

    #[test]
    fn floats_accept_any_finite_value() {
        assert_eq!(f64::from_f64(0.1), Some(0.1));
        assert_eq!(f32::from_f64(0.1), Some(0.1f32));
        assert_eq!(f32::from_f64(1e300), Some(f32::MAX));
        assert_eq!(f32::from_f64(-1e300), Some(f32::MIN));
    }

    #[test]
    fn values_round_trip_through_f64() {
        assert_eq!(8i8.to_f64(), 8.0);
        assert_eq!(42u32.to_f64(), 42.0);
        assert_eq!(1.5f64.to_f64(), 1.5);
    }

    #[test]
    fn text_parsing_trims_and_rejects_garbage() {
        assert_eq!(i32::parse_input("  -7 ", None), Some(-7));
        assert_eq!(i32::parse_input("abc", None), None);
        assert_eq!(i32::parse_input("", None), None);
        assert_eq!(i32::parse_input("2.5", Some(2.5)), Some(3));
        assert_eq!(f64::parse_input("0,5", Some(0.5)), Some(0.5));
        assert_eq!(f64::parse_input("0.5", None), None);
    }
}
