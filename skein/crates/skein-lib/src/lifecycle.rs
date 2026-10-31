//! Owner-driven component shutdown (programming-model.md, 5.2).

/// A component's lifecycle as its owner drives it (programming-model.md, 5.2).
/// The owner sets `Closed` once everything below has settled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lifecycle {
    Live,
    Closing,
    Aborting,
    Closed,
}

impl Lifecycle {
    /// The owner's close: `Live` becomes `Closing`. True when it changed;
    /// a close while closing, aborting or closed changes nothing.
    pub fn close(&mut self) -> bool {
        match self {
            Lifecycle::Live => {
                *self = Lifecycle::Closing;
                true
            }
            Lifecycle::Closing | Lifecycle::Aborting | Lifecycle::Closed => false,
        }
    }

    /// The owner's abort: `Live` or `Closing` becomes `Aborting`. True when it changed.
    pub fn abort(&mut self) -> bool {
        match self {
            Lifecycle::Live | Lifecycle::Closing => {
                *self = Lifecycle::Aborting;
                true
            }
            Lifecycle::Aborting | Lifecycle::Closed => false,
        }
    }
}
