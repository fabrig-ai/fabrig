//! Rejection names classify one write's report (skein/docs/design/ui/web/README.md, section 9).
use crate::clipboard::{Outcome, rejected};

#[test]
fn only_not_allowed_is_refused() {
    for (name, expected) in [
        (Some("NotAllowedError"), Outcome::Refused),
        (Some("UnknownError"), Outcome::Failed),
        (Some("notallowederror"), Outcome::Failed),
        (Some(""), Outcome::Failed),
        (None, Outcome::Failed),
    ] {
        assert_eq!(rejected(name), expected, "rejection name {name:?}");
    }
}
