//! Secrets are checked in root formatting, stored bytes and actual agent environments.
use crate::fixtures;
use std::fmt;

/// The issued value's name, never the value itself.
#[must_use]
pub fn in_bytes(bytes: &[u8]) -> Option<&'static str> {
    // The issued fixture tokens and their Debug representations are ASCII.
    // Lossy decoding preserves every ASCII run, even in binary store frames.
    let text = String::from_utf8_lossy(bytes);
    for (name, token) in [
        ("LLM access token", fixtures::LLM_TOKEN),
        ("git identity token", fixtures::GIT_TOKEN),
        ("forge API token", fixtures::API_TOKEN),
    ] {
        let rendered = format!("{token:?}");
        if text.contains(std::str::from_utf8(token).expect("ASCII fixture token")) || text.contains(&rendered) {
            return Some(name);
        }
    }
    None
}
pub(crate) struct Roots<'a>(pub &'a temper_service::Service);
impl fmt::Debug for Roots<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_roots(formatter)
    }
}
pub(crate) fn check(bytes: &[u8], place: &str) -> Result<(), String> {
    if let Some(name) = in_bytes(bytes) {
        return Err(format!("{name} in {place}"));
    }
    Ok(())
}
