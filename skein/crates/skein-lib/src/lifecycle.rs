//! Component lifecycle vocabulary (programming-model.md, 5.2).

/// A component takes requests while `Live`; a close makes it `Closing`, an abort
/// `Aborting`, and it is `Closed` once it has told its owner `Closed`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lifecycle {
    Live,
    Closing,
    Aborting,
    Closed,
}
