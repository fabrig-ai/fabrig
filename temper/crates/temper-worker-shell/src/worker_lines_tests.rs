use super::{ending_with_binary as worker_ending_words, note_line as worker_note_words};
use temper_worker_service::{Refusal, WorkerJobEnding, WorkerLoss, worker_alone as worker};

const BINARY: &str = "temper-worker";
fn directories() -> temper_shell::directories::Directories {
    let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories").0
}
#[test]
fn worker_job_words_keep_the_binary_prefix_and_reason() {
    let ending = WorkerJobEnding::Failed {
        failure: jig_domain_host::JobFailure::Deadline,
        detail: b"details\n".as_slice().into(),
    };
    let worker = worker::Note::WorkerJob {
        run: 7,
        attempt: 2,
        kind: 3,
        verb: b"merge".as_slice().into(),
        ending,
        why: b"deadline".as_slice().into(),
    };
    assert_eq!(
        worker_note_words(BINARY, worker),
        "temper-worker worker: run 7 attempt 2 job 3 merge failed Deadline: deadline\n"
    );
}
#[test]
fn worker_linked_words_name_the_worker_and_address() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let worker = worker::Note::Linked { worker: b"worker-a".as_slice().into(), address };
    assert_eq!(worker_note_words(BINARY, worker), "linked worker-a 127.0.0.1:7100\n");
}
#[test]
fn worker_lost_words_name_the_worker_address_and_reason() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let why = WorkerLoss::Silence;
    let worker = worker::Note::Lost { worker: b"worker-a".as_slice().into(), address, why };
    assert_eq!(worker_note_words(BINARY, worker), "lost worker-a 127.0.0.1:7100 silence\n");
}
#[test]
fn worker_refused_words_name_the_peer_refusal() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let reason = Refusal::Peer(256);
    let worker = worker::Note::Refused { worker: b"worker-a".as_slice().into(), address, reason };
    assert_eq!(worker_note_words(BINARY, worker), "refused worker-a 127.0.0.1:7100 unauthorized\n");
}
#[test]
fn worker_start_problem_words_and_exit_keep_their_behavior() {
    let mut problems = jig_protocol_config::Problems::empty();
    problems.push(jig_protocol_config::Path::root().member(b"git"), jig_protocol_config::What::Form);
    problems.push(jig_protocol_config::Path::root().member(b"limits"), jig_protocol_config::What::Bound);
    let worker = worker::End::Stopped(worker::Stop::Start(problems));
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker), false, &directories, b"");
    assert_eq!(
        actual,
        ("temper-worker serve: git: invalid form\ntemper-worker serve: limits: past its bound\n".into(), 1)
    );
}
#[test]
fn worker_drained_words_and_exit_keep_their_behavior() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Drained), false, &directories, b"");
    assert_eq!(actual, (String::new(), 0));
}
#[test]
fn worker_output_failure_words_and_exit_keep_their_behavior() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Drained), true, &directories, b"");
    assert_eq!(actual, (String::new(), 1));
}
#[test]
fn worker_aborted_words_and_exit_keep_their_behavior() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Aborted), false, &directories, b"");
    assert_eq!(actual, (String::new(), 1));
}

#[test]
fn worker_link_notes_print_each_address_and_refusal_exactly() {
    use temper_worker_service::{Refusal, worker_alone::Note};
    for (address, printed) in
        [(([127, 0, 0, 1], 7100).into(), "127.0.0.1:7100"), (([0, 0, 0, 0, 0, 0, 0, 1], 7100).into(), "[::1]:7100")]
    {
        assert_eq!(
            worker_note_words(
                "temper",
                Note::Linked { worker: b"worker-a".as_slice().into(), address: Box::new(address) }
            ),
            format!("linked worker-a {printed}\n")
        );
        for (reason, expected) in [
            (Refusal::Peer(256), "unauthorized"),
            (Refusal::Peer(257), "application"),
            (Refusal::Peer(258), "busy"),
            (Refusal::Peer(259), "replaced"),
            (Refusal::Peer(260), "not-admitted"),
            (Refusal::Peer(261), "stopping"),
            (Refusal::Terms, "terms"),
            (Refusal::Malformed, "malformed"),
            (Refusal::Peer(42), "code-42"),
        ] {
            assert_eq!(
                worker_note_words(
                    "temper",
                    Note::Refused { worker: b"worker-a".as_slice().into(), address: Box::new(address), reason }
                ),
                format!("refused worker-a {printed} {expected}\n")
            );
        }
    }
}

#[test]
fn worker_loss_lines_name_each_cause_and_refusal_exactly() {
    use temper_worker_service::{WorkerClosed as Closed, WorkerLoss as Loss, worker_alone::Note};
    for (why, expected) in [
        (Loss::Closed(Closed::Stream), "lost worker-a [::1]:7100 stream\n"),
        (Loss::Closed(Closed::Truncated), "lost worker-a [::1]:7100 truncated\n"),
        (Loss::Closed(Closed::Framing), "lost worker-a [::1]:7100 framing\n"),
        (Loss::Closed(Closed::Owner), "lost worker-a [::1]:7100 owner\n"),
        (Loss::Silence, "lost worker-a [::1]:7100 silence\n"),
        (Loss::Stall, "lost worker-a [::1]:7100 stall\n"),
        (Loss::Handshake, "lost worker-a [::1]:7100 handshake\n"),
        (Loss::Full, "lost worker-a [::1]:7100 full\n"),
        (Loss::Busy, "lost worker-a [::1]:7100 busy\n"),
        (Loss::Fault, "lost worker-a [::1]:7100 fault\n"),
        (Loss::Unreachable, "lost worker-a [::1]:7100 unreachable\n"),
        (Loss::Closed(Closed::RefusedHere(2)), "lost worker-a [::1]:7100 refused terms\n"),
        (Loss::Closed(Closed::RefusedHere(3)), "lost worker-a [::1]:7100 refused malformed\n"),
        (Loss::Closed(Closed::RefusedHere(256)), "lost worker-a [::1]:7100 refused unauthorized\n"),
        (Loss::Closed(Closed::RefusedHere(257)), "lost worker-a [::1]:7100 refused application\n"),
        (Loss::Closed(Closed::RefusedHere(258)), "lost worker-a [::1]:7100 refused busy\n"),
        (Loss::Closed(Closed::RefusedHere(259)), "lost worker-a [::1]:7100 refused replaced\n"),
        (Loss::Closed(Closed::RefusedHere(260)), "lost worker-a [::1]:7100 refused not-admitted\n"),
        (Loss::Closed(Closed::RefusedHere(261)), "lost worker-a [::1]:7100 refused stopping\n"),
        (Loss::Closed(Closed::RefusedHere(42)), "lost worker-a [::1]:7100 refused code-42\n"),
        (Loss::Closed(Closed::RefusedPeer(2)), "lost worker-a [::1]:7100 refused-by-peer terms\n"),
        (Loss::Closed(Closed::RefusedPeer(3)), "lost worker-a [::1]:7100 refused-by-peer malformed\n"),
        (Loss::Closed(Closed::RefusedPeer(256)), "lost worker-a [::1]:7100 refused-by-peer unauthorized\n"),
        (Loss::Closed(Closed::RefusedPeer(257)), "lost worker-a [::1]:7100 refused-by-peer application\n"),
        (Loss::Closed(Closed::RefusedPeer(258)), "lost worker-a [::1]:7100 refused-by-peer busy\n"),
        (Loss::Closed(Closed::RefusedPeer(259)), "lost worker-a [::1]:7100 refused-by-peer replaced\n"),
        (Loss::Closed(Closed::RefusedPeer(260)), "lost worker-a [::1]:7100 refused-by-peer not-admitted\n"),
        (Loss::Closed(Closed::RefusedPeer(261)), "lost worker-a [::1]:7100 refused-by-peer stopping\n"),
        (Loss::Closed(Closed::RefusedPeer(42)), "lost worker-a [::1]:7100 refused-by-peer code-42\n"),
    ] {
        assert_eq!(
            worker_note_words(
                "temper",
                Note::Lost {
                    worker: b"worker-a".as_slice().into(),
                    address: Box::new(([0, 0, 0, 0, 0, 0, 0, 1], 7100).into()),
                    why,
                }
            ),
            expected,
            "{why:?}"
        );
    }
}
