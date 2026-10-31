//! Owner-driven lifecycle transitions (programming-model.md, 5.2).

use crate::Lifecycle;

#[test]
fn a_close_can_be_escalated_to_an_abort() {
    let mut lifecycle = Lifecycle::Live;
    assert!(lifecycle.close());
    assert_eq!(lifecycle, Lifecycle::Closing);
    assert!(lifecycle.abort());
    assert_eq!(lifecycle, Lifecycle::Aborting);
}

#[test]
fn an_abort_ignores_later_close_and_abort_requests() {
    let mut lifecycle = Lifecycle::Live;
    assert!(lifecycle.abort());
    assert!(!lifecycle.close());
    assert!(!lifecycle.abort());
    assert_eq!(lifecycle, Lifecycle::Aborting);
}

#[test]
fn a_second_close_keeps_the_component_closing() {
    let mut lifecycle = Lifecycle::Live;
    assert!(lifecycle.close());
    assert!(!lifecycle.close());
    assert_eq!(lifecycle, Lifecycle::Closing);
}

#[test]
fn neither_close_nor_abort_changes_a_closed_component() {
    let mut lifecycle = Lifecycle::Closed;
    assert!(!lifecycle.close());
    assert!(!lifecycle.abort());
    assert_eq!(lifecycle, Lifecycle::Closed);
}
