//! Real mmdebstrap captures paired with hand-reviewed expectations (hearth/docs/design/testing/tool-fakes.md, 5).
use crate::reference::{self, Decoded};
use hearth_recordings::Recording;
use skein_scratch::paths;
use std::fs;
#[derive(Debug)]
pub struct Case {
    pub case: Box<str>,
    pub recording: Recording,
    pub expected: Option<Decoded>,
}
#[must_use]
pub fn all() -> Vec<Case> {
    hearth_recordings::all("mmdebstrap")
        .into_iter()
        .map(|(case, recording)| {
            let path =
                paths::tree().join("tests/mmdebstrap/expect").join(&*recording.version).join(format!("{case}.expect"));
            let expected = match fs::read_to_string(&path) {
                Ok(text) => Some(reference::expectation(&text)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("{}: {error}", path.display()),
            };
            Case { case, recording, expected }
        })
        .collect()
}
