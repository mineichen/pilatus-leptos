use super::InputNumberValue;
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

                fn format(self) -> String {
                    self.to_string()
                }
            }
        )*
    };
}

impl_input_number_value_for_int!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
