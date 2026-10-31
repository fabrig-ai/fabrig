//! A component's lifecycle (lib.md, 3.1; programming-model.md, 5.2): live,
//! then closing or aborting, then closed.

/// Where a component that keeps entities for its owner stands in its
/// lifecycle. A component holds one, refuses new work at its entrance
/// unless it is `Live`, and reports `Closed` once, last, when everything
/// below it has settled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lifecycle {
    /// Taking work.
    Live,
    /// Closing gracefully, after the owner's close: busy work ends first.
    Closing,
    /// Aborting, after the owner's abort: nothing waits.
    Aborting,
    /// Ended, its `Closed` reported: its owner sends it nothing more.
    Closed,
}

impl Lifecycle {
    /// The owner's close. A live component starts closing, and the answer is
    /// `true`; a second close, or a close after an abort, changes nothing.
    /// A close after `Closed` is the owner's bug, asserted.
    #[must_use]
    pub fn close(&mut self) -> bool {
        match self {
            Lifecycle::Live => {
                *self = Lifecycle::Closing;
                true
            }
            Lifecycle::Closing | Lifecycle::Aborting => false,
            Lifecycle::Closed => unreachable!("an owner sends nothing after Closed"),
        }
    }

    /// The owner's abort. A live or closing component aborts, and the answer
    /// is `true`, turning what still closes gracefully into aborts; a second
    /// abort changes nothing. An abort after `Closed` is the owner's bug,
    /// asserted.
    #[must_use]
    pub fn abort(&mut self) -> bool {
        match self {
            Lifecycle::Live | Lifecycle::Closing => {
                *self = Lifecycle::Aborting;
                true
            }
            Lifecycle::Aborting => false,
            Lifecycle::Closed => unreachable!("an owner sends nothing after Closed"),
        }
    }
}
