//! The component lifecycle (skein/docs/foundation/programming-model.md, 5.2).

/// A component's phases from accepting work through its one terminal event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lifecycle {
    /// Active, accepting new work at its entrance.
    Live,
    /// Refusing new work and draining what it owns through the ordinary lifecycle.
    Closing,
    /// Cancelling each request it holds and aborting what it keeps.
    Aborting,
    /// Everything below has settled and its one terminal event has been reported.
    Closed,
}
