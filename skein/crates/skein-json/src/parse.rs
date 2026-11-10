//! One whole borrowed input to a retained document (skein/docs/design/data/json.md, 5.3).
//! The tokenizer owns grammar state; the builder admits retained counts.
//! No stream state survives `parse`, and only the compact document is returned.

use crate::Document;
use crate::document::{self, Builder};
use crate::tokenizer::{self, Event, Feed, Request};

/// A grammar/input refusal or a retained-count refusal (skein/docs/design/data/json.md, 5.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ParseError {
    /// The tokenizer refused the input under its limits.
    Tokenizer(tokenizer::Error),
    /// The retained document exceeded its token or text limit.
    Document(document::Error),
}

/// Parses exactly one value and trailing whitespace (skein/docs/design/data/json.md, 5.3).
/// Input, nesting, strings and numbers use `limits`; retained records and
/// decoded text use `retained`. No partial document is returned on failure.
/// As for `Feed::new`, `limits.chunk` must be positive and
/// `limits.length` must be less than `u32::MAX`.
pub fn parse(input: &[u8], limits: &tokenizer::Limits, retained: &document::Limits) -> Result<Document, ParseError> {
    if input.len() > skein_lib::usize_of(limits.length) {
        return Err(ParseError::Tokenizer(tokenizer::Error::TooLong));
    }
    let mut feed = Feed::new(input, limits);
    let mut document = Builder::new(*retained);
    // Every token reads a byte, so the end comes within one answer more.
    for _answer in 0..=input.len() {
        match feed.answer(Request::Next) {
            Event::Token(token) => {
                if let Err(error) = document.push(&token) {
                    return Err(ParseError::Document(error));
                }
            }
            Event::Done => return Ok(document.into_document()),
            Event::Failed(error) => return Err(ParseError::Tokenizer(error)),
            Event::Skipped(_) | Event::Long(_) | Event::Closed => {
                return Err(ParseError::Tokenizer(tokenizer::Error::Unexpected));
            }
        }
    }
    Err(ParseError::Tokenizer(tokenizer::Error::Unexpected))
}
