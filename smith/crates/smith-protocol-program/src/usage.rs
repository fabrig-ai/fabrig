//! The bounded local usage result (protocol/programs.md, 5.11).
//! Reuses the program JSON reader and retains no state; the host reads the
//! result's spend and words only after the command has settled.

use skein_json::Document;

use crate::{USAGE_BYTES, mcp::json};

/// Read one bounded JSON object of type result, leaving its fields for the host.
#[must_use]
pub fn usage_result(bytes: &[u8]) -> Option<Document> {
    if u32::try_from(bytes.len()).ok()? > USAGE_BYTES {
        return None;
    }
    let document = json::collect(bytes)?;
    if json::text(&document, json::field(&document, 0, b"type")?)? != b"result" {
        return None;
    }
    Some(document)
}
