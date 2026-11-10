//! Whole CSS pixels for drawing sizes and points (skein/docs/design/ui/web/README.md, 6).

pub(crate) fn pixels(value: f64) -> u16 {
    if value.is_nan() || value < 1.0 {
        return 0;
    }
    value.floor().clamp(0.0, f64::from(u16::MAX)).to_string().parse().expect("whole CSS pixels fit u16")
}

#[cfg(test)]
#[path = "../tests/measure.rs"]
mod tests;
