use super::InputNumberValue;

/// Render `value` with at most `decimals` decimal places, trimming trailing
/// zeros (and a dangling dot), so `80.0` shows as `"80"` and `0.1239` as
/// `"0.124"` with `decimals = 3`.
fn format_trimmed(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let s = format!("{value:.decimals$}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    match trimmed {
        "" | "-0" => "0".to_string(),
        s => s.to_string(),
    }
}

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

    fn format(self) -> String {
        format_trimmed(f64::from(self), 3)
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

    fn format(self) -> String {
        format_trimmed(self, 3)
    }
}

impl InputNumberValue for pilatus::Percentage {
    const MIN: Self = pilatus::Percentage::min();
    const MAX: Self = pilatus::Percentage::max();

    fn parse_input(_text: &str, dom_number: Option<f64>) -> Option<Self> {
        dom_number.and_then(Self::from_f64)
    }

    fn from_f64(value: f64) -> Option<Self> {
        // Display units are percentage points (0..=100); saturate instead of
        // failing so stepper arithmetic never drops a commit.
        if value <= 0.0 {
            Some(pilatus::Percentage::min())
        } else if value >= 100.0 {
            Some(pilatus::Percentage::max())
        } else {
            pilatus::Percentage::new(value / 100.0)
        }
    }

    fn to_f64(self) -> f64 {
        self.value() * 100.0
    }

    fn format(self) -> String {
        format_trimmed(self.value() * 100.0, 1)
    }
}
