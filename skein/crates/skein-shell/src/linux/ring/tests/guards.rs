//! Submit guards with constructed records and no open ring (shell.md, 3).

use skein_io::kernel::{Fd, Op, Submit};
use skein_lib::Token;

use super::super::{Config, OpenError, Ring, Table, check_record};

const FD: Fd = Fd::new(123);
const TOKEN: Token = Token::new(1);

// Lowers ring::a_kernel_of_no_operations_does_not_open
#[test]
fn zero_capacity_is_refused_before_opening_a_ring() {
    assert_eq!(Ring::open(Config { operations: 0 }).expect_err("no syscall needed"), OpenError::NoOperations);
}

// Lowers ring::an_invalid_record_is_refused_at_submit
#[test]
#[should_panic(expected = "io submits only valid records")]
fn an_empty_receive_is_refused_by_the_submit_guard() {
    check_record(&Submit { op: TOKEN, kind: Op::Recv { fd: FD, buf: Box::from([]) } });
}

// Lowers ring::a_token_already_in_flight_is_refused_at_submit
#[test]
#[should_panic(expected = "io never reuses a token in flight")]
fn a_live_token_is_refused_by_the_submit_guard() {
    let mut table = Table::new(2);
    table.tokens.insert(TOKEN, 0);
    table.check_token(Token::new(2));
    let record = Submit { op: TOKEN, kind: Op::Close { fd: FD } };
    check_record(&record);
    table.check_token(record.op);
}

// Lowers ring::a_wait_forever_with_nothing_in_flight_is_refused
#[test]
#[should_panic(expected = "waiting forever with nothing in flight never returns")]
fn an_empty_forever_wait_is_refused_by_the_enter_guard() {
    Table::new(1).check_forever();
}

// Lowers ring::a_wait_forever_with_nothing_in_flight_is_refused
#[test]
fn a_forever_wait_accepts_live_or_ready_work() {
    let mut table = Table::new(1);
    table.tokens.insert(TOKEN, 0);
    table.check_forever();
    table.tokens.clear();
    table.ready.push_back(0);
    table.check_forever();
}

// Lowers ring::files::a_name_that_could_leave_its_directory_is_refused_at_submit
#[test]
#[should_panic(expected = "io submits only valid records")]
fn an_escaping_name_is_refused_by_the_submit_guard() {
    check_record(&Submit {
        op: TOKEN,
        kind: Op::Remove { dir: FD, name: Box::from(&b"../out"[..]), directory: false },
    });
}
