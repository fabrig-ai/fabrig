//! Bounded whole JSON tokenization.
use skein_json::{Document, tokenizer};
pub(crate) fn document_depth(bytes: &[u8], depth: u32) -> Result<Document, ()> {
    let length = u32::try_from(bytes.len()).or(Err(()))?;
    if length == u32::MAX {
        return Err(());
    }
    let limits = tokenizer::Limits { depth, string: length, number: 20, chunk: length.max(1), length };
    skein_json::parse(bytes, &limits, &skein_json::document::Limits { tokens: length, text: length }).or(Err(()))
}
