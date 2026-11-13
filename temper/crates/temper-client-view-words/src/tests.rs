//! Tests staged for the words (temper/docs/design/client/view-crates.md, 4.3).
mod accounts;
mod cards;
mod connect_forge;
mod fixtures;
mod frame;
mod look;
mod paths;
pub(crate) mod times;
pub(crate) mod words;

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
