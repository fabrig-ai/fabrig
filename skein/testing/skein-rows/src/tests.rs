//! The row runner's order and report (skein/docs/design/testing/README.md, 6).

use std::cell::RefCell;

use super::{check, check_with_report};

#[test]
fn each_row_is_named_before_its_body_runs() {
    let output = RefCell::new(String::new());
    check_with_report(
        [("a", "skein-rows: row a\n"), ("b", "skein-rows: row a\nskein-rows: row b\n")],
        |expected| assert_eq!(*output.borrow(), expected),
        |line| {
            let mut output = output.borrow_mut();
            output.push_str(line);
            output.push('\n');
        },
    );
}

#[test]
fn rows_that_all_pass_return() {
    let values = RefCell::new(Vec::new());
    check([("first", 1), ("later", 2)], |value| values.borrow_mut().push(value));
    assert_eq!(*values.borrow(), [1, 2]);
}
