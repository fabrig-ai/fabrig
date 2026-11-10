//! The JSON tokenizer's machine worlds (testing-strategy.md, 2.4; data/json.md,
//! 6), and the writer against it.
//!
//! What the test binaries share:
//!
//! - [`world`]: one tokenizer from a seed, between a stream below that
//!   delivers exactly what was demanded, from the peer's bytes cut at
//!   random, and ends early or fails, and a user above that demands
//!   slowly, stops, and closes in every state. It checks the machine's
//!   contracts as it goes.
//! - [`reference`]: a simple parser of a whole document, held in memory,
//!   that the tokenizer is checked against.
//! - [`generate`]: documents from a seed, valid and mutated, and the
//!   token sequences the writer writes.
//! - [`transcript`]: the transcripts kept in `transcripts/`, each with
//!   what it must decode to.
//! - [`write`]: tokens through the writer's two passes.
//!
//! The focused tests are `tests/*.rs`; the sweeps over many seeds are
//! `tests/fuzzy_*.rs` (testing-strategy.md, 8).

pub mod transcript;
pub mod world;

pub use skein_json_kit::*;
