//! Readiness admission decisions without opening a poller (runtime/readiness.md, 4).

use super::{BTreeMap, Config, Fd, Op, OpenError, Readiness, Waiters, check_waiters};

const FD: Fd = Fd::new(123);

#[test]
fn zero_capacity_is_refused_before_opening_a_poller() {
    assert_eq!(Readiness::open(Config { operations: 0 }).expect_err("no syscall needed"), OpenError::NoOperations);
}

#[test]
fn the_other_side_and_other_descriptor_remain_available() {
    let waiters = BTreeMap::from([(FD, Waiters { read: Some(0), write: None })]);
    check_waiters(&waiters, &Op::PipeWrite { fd: FD, bytes: Box::from(*b"x"), from: 0 });
    check_waiters(&waiters, &Op::Accept { fd: Fd::new(456) });
    check_waiters(&waiters, &Op::Close { fd: Fd::new(456) });
}
