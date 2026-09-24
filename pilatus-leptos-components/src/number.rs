use std::fmt::Display;

mod float;
mod input_number;
mod input_range;
mod integer;

pub use input_number::InputNumber;
pub use input_range::InputRange;

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
    /// Format for display in the `<input>` field and its `min`/`max`/`step`
    /// attributes (all in display units, i.e. what the user sees and types).
    ///
    /// Integers render exactly; floats render with at most 3 decimal places
    /// (trailing zeros trimmed); `Percentage` renders as percentage points
    /// (`value * 100`) with at most 1 decimal place.
    fn format(self) -> String;
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
}
