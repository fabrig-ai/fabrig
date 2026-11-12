use crate::process::Process;
use skein_io::{self as io, kernel::{Complete, Done, Fd, Op, Submit}};
use skein_lib::Token;

#[test]
fn operations_are_the_staged_capacity_not_the_shell_queue() {
    let mut process = Process::new(io::Limits::TEST, 32, 4);
    assert_eq!(process.operations(), 32);
    assert_eq!(process.completions().capacity(), 4);
    assert_eq!(process.submissions().capacity(), 4);
}

#[test]
fn the_signal_source_waits_for_credit_then_closes_once_and_reclaims() {
    let mut process = Process::new(io::Limits::TEST, 32, 4);
    let fd = Fd::new(50);
    process.adopt_signals(fd).unwrap();
    assert_eq!(process.adopt_signals(Fd::new(51)), Err(Fd::new(51)));
    for index in 0..process.operations() {
        process.raw_submissions().push(Submit { op: Token::new(u64::from(index)), kind: Op::Close { fd: Fd::new(90) } });
    }
    process.close_signals();
    assert!(!process.signals_closing(), "no close credit yet");
    process.raw_submissions().clear();
    process.close_signals();
    assert!(process.signals_closing(), "the close is staged");
    let count = process.raw_submission_count();
    process.close_signals();
    assert_eq!(process.raw_submission_count(), count, "one close only");
    process.publish();
    let close = process.submissions().pop().expect("the signal descriptor closes without a pending read");
    assert_eq!(close.kind, Op::Close { fd });
    assert!(process.submissions().is_empty());
    process.completions().push(Complete { op: close.op, kind: close.kind, result: Ok(Done::Nothing) });
    let complete = process.take_completion().expect("a credited kernel completion");
    process.up(complete);
    assert_eq!(process.told().pop(), Some(io::Event::Closed { owner: Token::new(crate::tokens::SIGNALS) }));
    process.signals_closed();
    process.publish();
    assert!(process.is_empty(), "publication reclaims the completed source");
}
