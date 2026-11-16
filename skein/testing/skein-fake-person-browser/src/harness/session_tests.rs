//! Completion sorting keeps io's queue across polls, and a spawn's directory goes
//! beneath the session's root, without a kernel.
use super::{Pump, ROOT_CLOSE, Report, take_root_close};
use skein_io::kernel::{Complete, Done, Error, Op};
use skein_lib::{Queue, Token};
use std::path::Path;

// Three ordinary completions and the root close fill the queue.
const CAPACITY: u32 = 4;

fn completion(op: Token, result: Result<Done, Error>) -> Complete {
    Complete { op, kind: Op::Cancel { target: Token::new(0) }, result }
}

fn ordinary(op: u64) -> Complete {
    completion(Token::new(op), Ok(Done::Nothing))
}

#[test]
fn a_full_queue_keeps_every_ordinary_completion_in_order() {
    let mut complete = Queue::with_capacity(CAPACITY);
    for entry in [ordinary(1), completion(ROOT_CLOSE, Err(Error::Other(5))), ordinary(2), ordinary(3)] {
        complete.push(entry);
    }
    let mut reports = Vec::new();

    take_root_close(&mut complete, &mut reports);

    assert!(complete.iter().eq([ordinary(1), ordinary(2), ordinary(3)].iter()), "ordinary completions stay in order");
    assert_eq!(reports.len(), 1);
    let Report::Trouble(text) = &reports[0] else { panic!("root close failure is Trouble: {reports:?}") };
    assert_eq!(text, "closing session root: Other(5)");
}

#[test]
fn a_successful_root_close_makes_no_report() {
    let mut complete = Queue::with_capacity(CAPACITY);
    complete.push(completion(ROOT_CLOSE, Ok(Done::Nothing)));
    let mut reports = Vec::new();

    take_root_close(&mut complete, &mut reports);

    assert!(complete.is_empty());
    assert!(reports.is_empty(), "a successful close is silent: {reports:?}");
}

#[test]
fn another_sort_keeps_leftovers_before_newly_delivered_completions() {
    let mut complete = Queue::with_capacity(CAPACITY);
    complete.push(ordinary(1));
    complete.push(ordinary(2));
    let mut reports = Vec::new();
    take_root_close(&mut complete, &mut reports);

    // io's stage leaves these entries; the next delivery fills only the room left.
    complete.push(completion(ROOT_CLOSE, Ok(Done::Nothing)));
    complete.push(ordinary(3));
    take_root_close(&mut complete, &mut reports);

    assert!(complete.iter().eq([ordinary(1), ordinary(2), ordinary(3)].iter()), "leftovers precede new completions");
    assert!(reports.is_empty());
}

#[test]
fn a_spawn_directory_is_given_beneath_the_root() {
    // The root is `/`, and a spawn opens its directory beneath it, refusing an absolute path.
    for (path, beneath) in [
        ("/home/person/scratch", "home/person/scratch"),
        ("//home/person", "home/person"),
        ("/", ""),
        ("profile", "profile"),
    ] {
        assert_eq!(&*Pump::beneath(Path::new(path)), beneath.as_bytes(), "{path}");
    }
}
