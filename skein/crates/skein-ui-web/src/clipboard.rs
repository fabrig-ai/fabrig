//! The clipboard written, one effect's part (skein/docs/design/ui-web.md, section 11).
#[cfg(target_arch = "wasm32")]
pub mod web;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    Refused,
    Failed,
}

/// One write's one report, carrying its token (skein/docs/design/ui.md, section 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    pub token: u32,
    pub outcome: Outcome,
}

/// What a rejected promise's error name says.
#[must_use]
pub fn rejected(name: Option<&str>) -> Outcome {
    if name == Some("NotAllowedError") { Outcome::Refused } else { Outcome::Failed }
}
