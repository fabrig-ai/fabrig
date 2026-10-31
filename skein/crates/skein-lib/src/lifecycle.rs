//! A component's lifecycle (programming-model.md, 5.2): live, then closing
//! or aborting once its owner closes or aborts it, then closed.

/// Where a component stands in the lifecycle its owner drives
/// (programming-model.md, 5.2). It starts `Live`; the owner's close or
/// abort moves it on, and its `Closed` event, the last thing it says, ends
/// it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lifecycle {
    /// The owner has neither closed nor aborted it: it admits new work.
    Live,
    /// The owner has closed it: it refuses new work as closed, closes what
    /// it keeps, and each request it holds still ends with its own terminal.
    /// Another close changes nothing; an abort moves it on to `Aborting`.
    Closing,
    /// The owner has aborted it, live or closing: it refuses new work as
    /// closed, ends each request it holds as cancelled and aborts what it
    /// keeps. A close or another abort changes nothing.
    Aborting,
    /// Everything below it has settled and it has reported `Closed`, once:
    /// it has ended, and its owner sends it nothing more.
    Closed,
}
