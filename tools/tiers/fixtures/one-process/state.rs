//! An order-dependent failure for tools/tiers/design.md, 11.6.
use std::sync::atomic::{AtomicBool, Ordering};

static SET: AtomicBool = AtomicBool::new(false);

#[test]
fn a_sets() {
    SET.store(true, Ordering::SeqCst);
}

#[test]
fn b_reads() {
    assert!(!SET.load(Ordering::SeqCst), "earlier state");
}
