//! M5 QA-B: FILE refusals and an open client during an account move.
use crate::support;

#[test]
fn refused_files_leave_the_account_with_its_holder() {
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let scratch = support::Scratch::new("qa_b_account_file");
    support::account_file_qa::files(scratch.path());
    scratch.remove();
    eprintln!("QA-B FILE probes: {:?}", clock.now().now.saturating_since(started));
}

#[test]
fn an_open_terminal_sees_the_move_and_the_tasks_moving_wait() {
    let scratch = support::Scratch::new("qa_b_account_terminal");
    support::account_file_qa::terminal(scratch.path());
    scratch.remove();
}
