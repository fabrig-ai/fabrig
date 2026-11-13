//! The row runner's failures and admission (skein/docs/design/testing/README.md, 6).

use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind, panic_any};

use super::{check, panics};

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn panics_returns_when_the_message_holds_what_is_expected() {
    panics("expected", || panic!("an expected message"));
    panics("expected", || panic_any(String::from("an expected message")));
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn panics_fails_when_the_call_returns() {
    assert_eq!(failure(|| panics("expected", || {})), "call must panic: ()");
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn panics_fails_on_another_message() {
    let message = failure(|| panics("expected", || panic!("another message")));
    assert!(message.contains("another message"));
    assert!(message.contains("must contain \"expected\""));
    assert_eq!(failure(|| panics("expected", || panic_any(7_u8))), "panic must have a message");
}

fn failure(call: impl FnOnce()) -> String {
    let payload = catch_unwind(AssertUnwindSafe(call)).expect_err("table must fail");
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .expect("table failure is a message")
        .to_string()
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn every_row_runs_after_an_earlier_row_fails() {
    let count = Cell::new(0);
    let message = failure(|| {
        check([("first", false), ("later", true)], |passes| {
            count.set(count.get() + 1);
            assert!(passes, "first failure");
        });
    });
    assert_eq!(count.get(), 2);
    assert!(message.contains("1 of 2 rows failed"));
    assert!(message.contains("first: first failure"));
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn a_failure_names_each_failing_row_with_its_message() {
    let message = failure(|| {
        check([("first", Some("first message")), ("passing", None), ("later", Some("later message"))], |message| {
            if let Some(message) = message {
                panic_any(message);
            }
        });
    });
    assert_eq!(message, "2 of 3 rows failed\nfirst: first message\nlater: later message");
    assert!(!message.contains("passing"));
    let message = failure(|| {
        check([("string", true), ("opaque", false)], |string| {
            if string {
                panic_any(String::from("owned message"));
            }
            panic_any(7_u8);
        });
    });
    assert_eq!(message, "2 of 2 rows failed\nstring: owned message\nopaque: <no message>");
}

#[test]
fn rows_that_all_pass_return() {
    let count = Cell::new(0);
    let result = catch_unwind(AssertUnwindSafe(|| {
        check([("first", 1), ("later", 2)], |value| count.set(count.get() + value));
    }));
    assert!(result.is_ok());
    assert_eq!(count.get(), 3);
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn a_table_of_fewer_than_two_rows_fails_before_any_row_runs() {
    for rows in [vec![], vec![("only", ())]] {
        let count = Cell::new(0);
        let message = failure(|| check(rows, |()| count.set(count.get() + 1)));
        assert!(message.contains("at least two rows"));
        assert_eq!(count.get(), 0);
    }
}

#[test]
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn two_rows_of_one_name_fail_before_any_row_runs() {
    let count = Cell::new(0);
    let message = failure(|| {
        check([("same", ()), ("other", ()), ("same", ())], |()| count.set(count.get() + 1));
    });
    assert!(message.contains("two rows have one name: same"));
    assert_eq!(count.get(), 0);
}
