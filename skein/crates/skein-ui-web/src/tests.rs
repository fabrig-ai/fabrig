mod bridge;
mod clipboard;
mod fixtures;
mod log;
mod mapping;
mod page;
mod screen;
mod svg;
mod worker;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::inbox::Inbox;

#[test]
fn a_report_raised_in_a_turn_waits_for_that_turn() {
    let inbox: Rc<Inbox<u8>> = Inbox::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let calls = Rc::new(Cell::new(0));
    let (output, count) = (Rc::clone(&seen), Rc::clone(&calls));
    inbox.start(Box::new(move |inbox| {
        count.set(count.get() + 1);
        while let Some(report) = inbox.next() {
            if report == 1 {
                inbox.raise(2);
            }
            output.borrow_mut().push(report);
        }
    }));
    inbox.raise(1);
    assert_eq!((seen.borrow().clone(), calls.get()), (vec![1, 2], 1));
}

#[test]
fn a_report_raised_after_the_drain_calls_another_turn() {
    let inbox: Rc<Inbox<u8>> = Inbox::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let calls = Rc::new(Cell::new(0));
    let (output, count) = (Rc::clone(&seen), Rc::clone(&calls));
    inbox.start(Box::new(move |inbox| {
        count.set(count.get() + 1);
        while let Some(report) = inbox.next() {
            output.borrow_mut().push(report);
        }
        if count.get() == 1 {
            inbox.raise(9);
        }
    }));
    inbox.raise(1);
    assert_eq!((seen.borrow().clone(), calls.get()), (vec![1, 9], 2));
}

#[test]
fn a_draining_turn_is_called_once_per_raise() {
    let inbox: Rc<Inbox<u8>> = Inbox::new();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let calls = Rc::new(Cell::new(0));
    let (output, count) = (Rc::clone(&seen), Rc::clone(&calls));
    inbox.start(Box::new(move |inbox| {
        count.set(count.get() + 1);
        while let Some(report) = inbox.next() {
            output.borrow_mut().push(report);
        }
    }));
    inbox.raise(1);
    assert_eq!((seen.borrow().clone(), calls.get()), (vec![1], 1));
    inbox.raise(2);
    assert_eq!((seen.borrow().clone(), calls.get()), (vec![1, 2], 2));
}

#[test]
fn a_turn_that_takes_no_report_returns_with_reports_queued() {
    let inbox: Rc<Inbox<u8>> = Inbox::new();
    let calls = Rc::new(Cell::new(0));
    let count = Rc::clone(&calls);
    inbox.start(Box::new(move |_| {
        count.set(count.get() + 1);
        assert_eq!(count.get(), 1, "a turn that takes no report is not called again");
    }));
    inbox.raise(1);
    assert_eq!(calls.get(), 1);
    assert_eq!(inbox.next(), Some(1));
    assert_eq!(inbox.next(), None);
}

#[test]
fn a_report_outside_a_turn_calls_it_at_once() {
    let (inbox, seen) = fixtures::collecting();
    inbox.raise(7);
    assert_eq!(*seen.borrow(), vec![7]);
}

#[test]
fn reports_keep_their_order() {
    let inbox = Inbox::new();
    inbox.raise(3);
    inbox.raise(1);
    inbox.raise(2);
    let seen = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&seen);
    inbox.start(Box::new(move |inbox| {
        while let Some(report) = inbox.next() {
            output.borrow_mut().push(report);
        }
    }));
    assert_eq!(*seen.borrow(), vec![3, 1, 2]);
}

mod fetch;
mod hidden;
mod platform;
mod store;
mod transport;

mod history;
mod restart;
