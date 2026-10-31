//! One whole borrowed input to a retained document (skein/docs/design/json.md, 5.3).
//! The tokenizer owns grammar state; the builder admits retained counts.
//! No stream state survives `parse`, and only the compact document is returned.

use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Env, Queue, Time, Wall};

use crate::Document;
use crate::document::{self, Builder};
use crate::tokenizer::{self, Event, Request, Tokenizer, down, up};

/// A grammar/input refusal or a retained-count refusal (skein/docs/design/json.md, 5.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ParseError {
    /// The tokenizer refused the input under its limits.
    Tokenizer(tokenizer::Error),
    /// The retained document exceeded its token or text limit.
    Document(document::Error),
}

/// Parses exactly one value and trailing whitespace (skein/docs/design/json.md, 5.3).
/// Input, nesting, strings and numbers use `limits`; retained records and
/// decoded text use `retained`. No partial document is returned on failure.
/// As for `Tokenizer::new`, `limits.chunk` must be positive and
/// `limits.length` must be less than `u32::MAX`.
pub fn parse(input: &[u8], limits: &tokenizer::Limits, retained: &document::Limits) -> Result<Document, ParseError> {
    if input.len() > usize::try_from(limits.length).expect("u32 fits usize") {
        return Err(ParseError::Tokenizer(tokenizer::Error::TooLong));
    }
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut machine = Tokenizer::new(limits);
    let mut document = Builder::new(*retained);
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    down(&mut machine, &env, Request::Next, &mut above, &mut below);
    let mut at = 0_usize;
    let ticks = input.len().saturating_mul(4).saturating_add(16);
    for _tick in 0..ticks {
        match above.pop() {
            Some(Event::Token(token)) => {
                if let Err(error) = document.push(&token) {
                    return Err(ParseError::Document(error));
                }
                down(&mut machine, &env, Request::Next, &mut above, &mut below);
                continue;
            }
            Some(Event::Done) => return Ok(document.into_document()),
            Some(Event::Failed(error)) => return Err(ParseError::Tokenizer(error)),
            Some(Event::Skipped(_) | Event::Long(_) | Event::Closed) => {
                return Err(ParseError::Tokenizer(tokenizer::Error::Unexpected));
            }
            None => {}
        }
        let demand = below.pop().ok_or(ParseError::Tokenizer(tokenizer::Error::Unexpected))?;
        let read = match demand {
            Down::Demand { read, .. } => read,
            Down::Send(_) | Down::Finish => return Err(ParseError::Tokenizer(tokenizer::Error::Unexpected)),
        };
        let remaining = input.get(at..).ok_or(ParseError::Tokenizer(tokenizer::Error::Unexpected))?;
        let count = match read {
            Read::Fill(count) => usize::try_from(count).expect("u32 fits usize"),
            Read::Scan { until, max } => {
                let max = usize::try_from(max).expect("u32 fits usize");
                let scanned = remaining.get(..remaining.len().min(max)).expect("bounded slice");
                match skein_lib::bytes::find(scanned, until.as_bytes()) {
                    Some(position) => position.saturating_add(until.as_bytes().len()),
                    None => max,
                }
            }
            Read::Nothing | Read::Line { .. } | Read::Arrived { .. } => {
                return Err(ParseError::Tokenizer(tokenizer::Error::Unexpected));
            }
        };
        let delivery = if count <= remaining.len() {
            let bytes = skein_lib::bytes::copy_of(remaining.get(..count).expect("delivery within input"));
            at = at.checked_add(count).ok_or(ParseError::Tokenizer(tokenizer::Error::TooLong))?;
            Up::Bytes(bytes)
        } else {
            Up::End
        };
        up(&mut machine, &env, delivery, &mut above, &mut below);
    }
    Err(ParseError::Tokenizer(tokenizer::Error::Unexpected))
}
