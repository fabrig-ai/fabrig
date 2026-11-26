//! The engine's pass-through vocabulary
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2).

use crate::{Event, Request};

/// An input from the root, using the core's vocabulary.
#[derive(Hash, Debug)]
pub enum In {
    Core(Event),
}

/// An output to the root, with the core's request mark intact.
#[derive(Hash, Debug)]
pub enum Out {
    Core(Request),
}

/// Which timers the root admits at its watermark.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Admit {
    All,
    Finishing,
}
