use super::{Answer, Ask, Gathered, Host, Program, Ran, Step, Stream};

const HOLDER: u32 = 42;

fn head() -> Ask {
    Ask::Run(Program::new(b"git", &[b"rev-parse", b"HEAD"]))
}

fn merge_head() -> Ask {
    Ask::Exists(".git/MERGE_HEAD".into())
}

fn head_step() -> Step {
    Step { ask: head(), answer: Answer::Ran(Ran::new(Some(0), b"abc\n")) }
}

fn merge_head_step() -> Step {
    Step { ask: merge_head(), answer: Answer::Is(false) }
}

#[test]
fn canned_steps_answer_in_order_without_gathering_kept_bytes() {
    let mut host = Host::canned(vec![head_step(), merge_head_step()]);
    let head_answer = host.ask(head());
    let merge_answer = host.ask(merge_head());
    assert_eq!(
        (head_answer, merge_answer, host.finish()),
        (Answer::Ran(Ran::new(Some(0), b"abc\n")), Answer::Is(false), Gathered { stdout: vec![], stderr: vec![] }),
    );
}

#[test]
#[should_panic(expected = "host step 1: expected Exists(\".git/MERGE_HEAD\"), got Alive(42)")]
fn different_request_names_the_step_and_both_requests() {
    let mut host = Host::canned(vec![head_step(), merge_head_step()]);
    let _ = host.ask(head());
    let _ = host.ask(Ask::Alive(HOLDER));
}

#[test]
#[should_panic(expected = "host step 1: no request expected, got Alive(42)")]
fn extra_request_names_the_step_and_unexpected_request() {
    let mut host = Host::canned(vec![head_step()]);
    let _ = host.ask(head());
    let _ = host.ask(Ask::Alive(HOLDER));
}

#[test]
#[should_panic(expected = "host step 1 not taken: Exists(\".git/MERGE_HEAD\")")]
fn finishing_early_names_the_first_step_not_taken() {
    let mut host = Host::canned(vec![head_step(), merge_head_step()]);
    let _ = host.ask(head());
    let _ = host.finish();
}

fn printing(stdout: Stream, stderr: Stream) -> Ask {
    Ask::Run(Program {
        stdout,
        stderr,
        ..Program::new(b"cargo", &[b"run", b"--locked", b"-p", b"temper-client-walk", b"--", b"pages"])
    })
}

#[test]
fn gathers_own_writes_and_streamed_children_in_order() {
    let mut host = Host::canned(vec![
        Step { ask: printing(Stream::Out, Stream::Err), answer: Answer::Ran(Ran::new(Some(0), b"b\n")) },
        Step { ask: printing(Stream::Err, Stream::Err), answer: Answer::Ran(Ran::new(Some(0), b"d\n")) },
    ]);
    host.out(b"a\n");
    let _ = host.ask(printing(Stream::Out, Stream::Err));
    host.err(b"c\n");
    let _ = host.ask(printing(Stream::Err, Stream::Err));
    assert_eq!(host.finish(), Gathered { stdout: b"a\nb\n".to_vec(), stderr: b"c\nd\n".to_vec() });
}
