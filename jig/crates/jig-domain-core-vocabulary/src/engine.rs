//! The engine's pass-through vocabulary
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2).

use crate::{Event, Request};

/// An input from the root, using the core's vocabulary.
#[derive(Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "the root moves a complete input by value")]
pub enum In {
    Core(Event),
    Store(store::In),
}

/// An output to the root, with the core's request mark intact.
#[derive(Hash, Debug)]
pub enum Out {
    Core(Request),
    Held(Peer),
}

/// Which timers the root admits at its watermark.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Admit {
    All,
    Finishing,
}

/// The peer addressed by a held engine output.
#[derive(Hash, Debug)]
pub enum Peer {
    Store(store::Out),
}

/// The engine's in-process store port.
pub mod store;
