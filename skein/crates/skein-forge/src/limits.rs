//! Bounds both dialects share (skein/docs/design/forge/README.md, 4.4).
use skein_json::{tokenizer, writer};
/// Bounds on tokens, documents, retained fields and targets.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub json: tokenizer::Limits,
    pub writer: writer::Limits,
    pub string_bytes: u32,
    pub body_bytes: u32,
    pub inner: u32,
    pub target_bytes: u32,
}
/// A limit that cannot hold a dialect's documents.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Problem {
    Zero,
    String,
    Depth,
}
/// Checks that both sides can honour the limits at a dialect's JSON depth.
pub fn check(limits: &Limits, depth: u32) -> Result<(), Problem> {
    if limits.string_bytes == 0
        || limits.body_bytes == 0
        || limits.inner == 0
        || limits.target_bytes == 0
        || limits.json.chunk == 0
        || limits.json.number == 0
        || limits.json.length == 0
        || limits.writer.length == 0
        || limits.json.string == 0
    {
        return Err(Problem::Zero);
    }
    if limits.json.string < limits.string_bytes || limits.json.string < limits.body_bytes {
        return Err(Problem::String);
    }
    if limits.json.depth < depth || limits.writer.depth < depth {
        return Err(Problem::Depth);
    }
    Ok(())
}
