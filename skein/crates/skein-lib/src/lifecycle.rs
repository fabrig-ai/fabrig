//! A component's lifecycle (skein/docs/foundation/programming-model.md, 5.2;
//! skein/docs/design/lib.md, 3).

/// A component's lifecycle (programming-model.md, 5.2): live, then closing or
/// aborting, then closed. What a request after `Closed` means is the owner's.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lifecycle {
    Live,
    Closing,
    Aborting,
    Closed,
}

impl Lifecycle {
    /// A close: `Live` becomes `Closing`; anything else is unchanged. Whether it changed.
    pub const fn close(&mut self) -> bool {
        match self {
            Lifecycle::Live => {
                *self = Lifecycle::Closing;
                true
            }
            Lifecycle::Closing | Lifecycle::Aborting | Lifecycle::Closed => false,
        }
    }

    /// An abort: `Live` or `Closing` becomes `Aborting`; anything else is unchanged. Whether it changed.
    pub const fn abort(&mut self) -> bool {
        match self {
            Lifecycle::Live | Lifecycle::Closing => {
                *self = Lifecycle::Aborting;
                true
            }
            Lifecycle::Aborting | Lifecycle::Closed => false,
        }
    }

    /// Whether it is closing or aborting: waiting to settle.
    #[must_use]
    pub const fn is_settling(self) -> bool {
        match self {
            Lifecycle::Closing | Lifecycle::Aborting => true,
            Lifecycle::Live | Lifecycle::Closed => false,
        }
    }

    /// Settled: `Closing` or `Aborting` becomes `Closed` (asserted).
    pub fn closed(&mut self) {
        assert!(self.is_settling(), "only a settling lifecycle can become closed");
        *self = Lifecycle::Closed;
    }
}
