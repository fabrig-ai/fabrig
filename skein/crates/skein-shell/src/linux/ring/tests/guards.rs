//! Ring admission and wait guards without an open ring (runtime/shell.md, 3).

use skein_lib::Token;

use super::super::{Config, OpenError, Ring, Table};

const TOKEN: Token = Token::new(1);

#[test]
fn zero_capacity_is_refused_before_opening_a_ring() {
    assert_eq!(Ring::open(Config { operations: 0 }).expect_err("no syscall needed"), OpenError::NoOperations);
}

#[test]
fn a_forever_wait_accepts_live_or_ready_work() {
    let mut table = Table::new(1);
    table.tokens.insert(TOKEN, 0);
    table.check_forever();
    table.tokens.clear();
    table.ready.push_back(0);
    table.check_forever();
}
