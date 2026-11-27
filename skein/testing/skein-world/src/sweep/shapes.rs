//! Per-case counts of opaque shape text (skein/docs/design/testing/sweeps.md, 4.6).

use alloc::collections::BTreeMap;
use alloc::{format, string::String, vec::Vec};
use core::fmt;

/// The most distinct shapes a sweep keeps.
pub const SHAPES_KEPT: usize = 4096;

/// The shapes a sweep reached, counted by case, in its world's vocabulary.
#[derive(Debug)]
pub struct Shapes {
    sweep: &'static str,
    vocabulary: &'static str,
    cases: u64,
    kept: BTreeMap<String, Reach>,
    beyond: u64,
}

#[derive(Debug)]
struct Reach {
    cases: u64,
    last: u64,
}

impl Shapes {
    /// Keeps at most `SHAPES_KEPT` distinct shapes, in first-reached order of admission.
    #[must_use]
    pub fn new(sweep: &'static str, vocabulary: &'static str) -> Shapes {
        Shapes { sweep, vocabulary, cases: 0, kept: BTreeMap::new(), beyond: 0 }
    }

    /// Folds one case, counting each distinct shape in it once.
    ///
    /// # Errors
    /// Refuses the whole case if any shape is outside the required text form.
    pub fn ran(&mut self, case: &[&str]) -> Result<(), String> {
        for shape in case {
            if let Err(why) = in_form(shape) {
                return Err(format!("sweep {}: shape not in form: {why}", self.sweep));
            }
        }
        self.cases = self.cases.checked_add(1).expect("a sweep's case count fits u64");
        let mut beyond = false;
        for &shape in case {
            if let Some(reach) = self.kept.get_mut(shape) {
                if reach.last != self.cases {
                    reach.cases = reach.cases.checked_add(1).expect("a shape's case count fits u64");
                    reach.last = self.cases;
                }
            } else if self.kept.len() < SHAPES_KEPT {
                self.kept.insert(shape.into(), Reach { cases: 1, last: self.cases });
            } else {
                beyond = true;
            }
        }
        if beyond {
            self.beyond = self.beyond.checked_add(1).expect("a sweep's beyond count fits u64");
        }
        Ok(())
    }

    /// The cases folded, including those that reached nothing.
    #[must_use]
    pub fn cases(&self) -> u64 {
        self.cases
    }

    /// The cases that reached a kept shape, or zero if it was not kept.
    #[must_use]
    pub fn reached(&self, shape: &str) -> u64 {
        self.kept.get(shape).map_or(0, |reach| reach.cases)
    }

    /// Writes the sweep's summary and kept shapes to stderr.
    pub fn write(&self) {
        self.write_to(super::cases::stderr);
    }

    pub(crate) fn write_to(&self, mut report: impl FnMut(fmt::Arguments<'_>)) {
        let (sweep, vocabulary) = (self.sweep, self.vocabulary);
        report(format_args!(
            "fuzzy shapes: {sweep} {vocabulary} cases {} distinct {} beyond {}",
            self.cases,
            self.kept.len(),
            self.beyond
        ));
        // The map gives the tie order; the stable sort moves only differing counts.
        let mut ordered: Vec<_> = self.kept.iter().collect();
        ordered.sort_by_key(|(_, reach)| core::cmp::Reverse(reach.cases));
        for (shape, reach) in ordered {
            report(format_args!("fuzzy shape: {sweep} {vocabulary} {} {shape}", reach.cases));
        }
    }
}

const MAX_SHAPE_BYTES: usize = 1024;

fn in_form(shape: &str) -> Result<(), &'static str> {
    if shape.is_empty() {
        return Err("empty text");
    }
    if shape.len() > MAX_SHAPE_BYTES {
        return Err("more than 1024 bytes");
    }
    for byte in shape.bytes() {
        match byte {
            b'\n' => return Err("line feed"),
            b' '..=b'~' => {}
            _ => return Err("non-printable ASCII byte"),
        }
    }
    Ok(())
}
