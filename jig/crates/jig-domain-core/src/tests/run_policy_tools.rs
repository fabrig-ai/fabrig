//! A charter's host-family mask composes with task authority (domain/engine.md, 7.1).

use crate::RunPolicy;

#[test]
fn every_family_is_the_identity() {
    for tools in [0, 1, 512 | 1024, u64::MAX] {
        assert_eq!(RunPolicy::EVERY_FAMILY.0 & tools, tools);
    }
}
