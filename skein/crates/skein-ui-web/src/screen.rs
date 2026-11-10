//! Full screen, one effect's part (skein/docs/design/ui/web/README.md, section 9).
#[cfg(target_arch = "wasm32")]
pub mod web;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// The page went into full screen, its Escape held where the browser lets it.
    Entered,
    /// The page left full screen, by the browser or by `leave`; once for each `Entered`.
    Left,
    /// A request the browser refused, or a page with no full screen.
    Refused,
}

/// The report a change of the document's full-screen element makes, from `was` to `now`.
#[must_use]
pub fn changed(was: bool, now: bool) -> Option<Report> {
    match (was, now) {
        (false, true) => Some(Report::Entered),
        (true, false) => Some(Report::Left),
        (false, false) | (true, true) => None,
    }
}
