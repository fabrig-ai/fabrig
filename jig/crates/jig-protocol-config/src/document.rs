//! Bounded JSON documents and configuration problems (skein/docs/design/json.md, 5.3).
use crate::{Path, Problem, What};
use skein_json::{Document, document, tokenizer};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DocumentLimits {
    pub bytes: u32,
    pub tokens: u32,
    pub depth: u32,
}
pub fn document(input: &[u8], bounds: &DocumentLimits) -> Result<Document, Problem> {
    let problem = Problem { path: Path::root(), what: What::Json };
    let length = u32::try_from(input.len()).or(Err(problem.clone()))?;
    if length > bounds.bytes {
        return Err(problem);
    }
    let limits =
        tokenizer::Limits { depth: bounds.depth, string: length, number: length, chunk: length.max(1), length };
    let retained = document::Limits { tokens: bounds.tokens, text: bounds.bytes };
    skein_json::parse(input, &limits, &retained).or(Err(problem))
}
