use core::cmp::Ordering;

/// A caller-driven lower-bound search of a sorted slice (runtime/lib.md, 5.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Bisection {
    low: usize,
    high: usize,
}

impl Bisection {
    /// The most probes any length needs.
    pub const PROBES: u32 = usize::BITS;

    #[must_use]
    pub const fn over(len: usize) -> Bisection {
        Bisection { low: 0, high: len }
    }

    /// The middle open index, or none once the search has ended.
    #[must_use]
    pub const fn probe(&self) -> Option<usize> {
        if self.low == self.high {
            return None;
        }
        let span = self.high.checked_sub(self.low).expect("low <= high");
        Some(self.low.checked_add(span / 2).expect("the middle is below high"))
    }

    /// Takes the probed item's ordering against the key sought.
    /// A call with no probe open is a bug, asserted.
    pub fn narrow(&mut self, order: Ordering) {
        let probe = self.probe().expect("narrow has a probe open");
        match order {
            Ordering::Less => self.low = probe.checked_add(1).expect("probe < high <= usize::MAX"),
            Ordering::Equal | Ordering::Greater => self.high = probe,
        }
    }

    /// Once no probe is open, the first index not below the key, or the length.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.low
    }
}
