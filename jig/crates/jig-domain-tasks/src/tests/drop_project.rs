//! Bounded root enumeration and atomic project-drop refusal.
use super::revisions::{Harness, task, to};
use crate::{Control, Event, Party, Request};
use skein_lib::List;

#[test]
fn root_enumeration_reports_full_without_losing_its_prefix() {
    let mut test = Harness::new();
    test.make([task(2, &[]), task(1, &[])].into());
    let mut empty = List::with_capacity(0);
    assert_eq!(test.domain.live_roots(1, &mut empty), None);
    assert_eq!(test.domain.live_roots(2, &mut empty), Some(()));
    let mut one = List::with_capacity(1);
    assert_eq!(test.domain.live_roots(1, &mut one), None);
    assert_eq!(one.as_slice(), &[1]);
    let mut two = List::with_capacity(2);
    assert_eq!(test.domain.live_roots(1, &mut two), Some(()));
    assert_eq!(two.as_slice(), &[1, 2]);
}

#[test]
fn a_closing_task_refuses_the_drop_before_any_row_changes() {
    let mut test = Harness::new();
    test.make([task(1, &[])].into());
    test.send(Event::Control {
        reply_to: to(),
        by: Party::Person(9),
        task: 1,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    let before = test.stored.clone();
    assert!(test.domain.live_in(1));
    assert_eq!(test.send(Event::DropProject { project: 1 }), [Request::DropRefused { project: 1 }]);
    assert_eq!(test.stored, before);
}
