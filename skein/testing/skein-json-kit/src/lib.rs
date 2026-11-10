//! What a second crate takes of the JSON world: the reference reader, the generator and their outcome types (skein/docs/design/testing/README.md, 6).

pub mod generate;
pub mod reference;

use skein_json::Token;
use skein_json::tokenizer::Error;
use skein_json::writer::{self, Encoder, Refusal};

/// `tokens`, a document, through the writer's two passes (data/json.md, 4):
/// measured, then written into exactly that length; or why the measuring
/// pass refused it.
///
/// # Errors
///
/// The refusal of the measuring pass.
pub fn write(tokens: &[Token], limits: &writer::Limits) -> Result<Box<[u8]>, Refusal> {
    let mut measure = Encoder::measure(limits);
    for token in tokens {
        measure.token(token);
    }
    let len = measure.measured()?;
    let mut write = Encoder::write(len, limits);
    for token in tokens {
        write.token(token);
    }
    let document = write.finish();
    assert_eq!(document.len(), usize::try_from(len).expect("fits a usize"), "written at the length measured");
    Ok(document)
}

/// How a document ended, as the side above saw it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    Done,
    Failed(Error),
}

/// A token sequence and how it ended: what a document decodes to.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Decoded {
    pub tokens: Vec<Token>,
    pub outcome: Outcome,
}

/// Runtime caps for the fixed selective filters used by the worlds.
pub const COLLECTOR_CAPS: &[u32] = &[4, 8, 0, 2];
