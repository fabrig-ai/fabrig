//! The sweep's tally (skein/docs/design/testing/README.md, 6).

use std::collections::BTreeMap;

/// How often each thing a sweep injects or reaches fell.
#[derive(Default, Debug)]
pub struct Seen(pub BTreeMap<String, u32>);

impl Seen {
    /// Record one occurrence of `what`.
    pub fn note(&mut self, what: String) {
        *self.0.entry(what).or_default() += 1;
    }

    /// Check that every wanted occurrence fell.
    pub fn assert_fell<S: AsRef<str>>(&self, wanted: &[S]) {
        for what in wanted {
            let what = what.as_ref();
            assert!(self.0.contains_key(what), "{what} fell: {:#?}", self.0);
        }
    }
}
