//! One tokenizer event in, one decoder step out.
use super::document::{Document, Row};
use skein_json::tokenizer;
pub(crate) mod walk;
pub use crate::decode::Malformed;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Demand(tokenizer::Request),
    Row(Row, tokenizer::Request),
    Done(Document),
    Malformed(Malformed),
}
