//! Tests staged for the words (temper/docs/design/client/view-crates.md, 4.3).
mod accounts;
mod cards;
mod connect_forge;
mod fixtures;
mod frame;
mod look;
mod paths;
pub(crate) mod times;
mod waits;
pub(crate) mod words;

#[test]
fn capacity_checks_follow_the_public_connector() {
    let mut words = fixtures::words();
    let connector = words.connector;
    let ordinary = words.ordinary_capacity();
    let classes = words.classes_capacity();
    let follows = words.follows_capacity();
    words.connector = u16::MAX;
    assert_eq!(words.ordinary_capacity(), None);
    assert_eq!(words.classes_capacity(), classes);
    assert_eq!(words.follows_capacity(), follows);
    assert_eq!(words.capacity(), None);
    assert!(crate::Limits::of(&fixtures::limits(), &words).is_none());
    words.connector = connector;
    assert_eq!(words.ordinary_capacity(), ordinary);
    assert_eq!(words.classes_capacity(), classes);
    assert_eq!(words.follows_capacity(), follows);
}

#[test]
fn validity_accepts_stock_limits_and_refuses_invalid_edges() {
    let mut domain = fixtures::limits();
    let words = fixtures::words();
    let limits = crate::Limits::of(&domain, &words).unwrap();
    assert!(crate::valid(&limits, &domain, &words));
    domain.client.domain.rows = u32::MAX; // Checked account/node row counts overflow.
    assert!(!crate::valid(&limits, &domain, &words));
    assert!(crate::Limits::of(&domain, &words).is_none());
}

#[test]
fn validity_accepts_large_ui_edges_and_refuses_overflowing_scene_capacity() {
    let domain = fixtures::limits();
    let words = fixtures::words();
    let mut limits = crate::Limits::of(&domain, &words).unwrap();
    limits.tree.nodes = 1 << 31_u32;
    limits.tree.drawing_bytes = 1 << 31_u32;
    assert!(crate::valid(&limits, &domain, &words));
    limits.tree.scene_marks = u32::MAX;
    limits.tree.styles_per_node = 2;
    assert!(!crate::valid(&limits, &domain, &words));
}
