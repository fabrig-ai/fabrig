use crate::lines::{ending_with_binary, note_line};
use temper_service::{End, Note, WorkerJobEnding, worker_alone as worker};
use temper_worker_shell::lines::{ending_with_binary as worker_ending_words, note_line as worker_note_words};

const BINARY: &str = "temper-worker";
fn directories() -> temper_shell::directories::Directories {
    let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories").0
}
#[test]
fn worker_job_words_equal_the_joined_words() {
    let ending = WorkerJobEnding::Failed {
        failure: jig_domain_host::JobFailure::Deadline,
        detail: b"details\n".as_slice().into(),
    };
    let worker = worker::Note::WorkerJob {
        run: 7,
        attempt: 2,
        kind: 3,
        verb: b"merge".as_slice().into(),
        ending: ending.clone(),
        why: b"deadline".as_slice().into(),
    };
    let joined = Note::Worker(temper_service::worker_alone::Note::WorkerJob {
        run: 7,
        attempt: 2,
        kind: 3,
        verb: b"merge".as_slice().into(),
        ending,
        why: b"deadline".as_slice().into(),
    });
    assert_eq!(worker_note_words(BINARY, worker), note_line(BINARY, joined));
}

#[test]
fn worker_start_problem_words_and_exit_equal_the_joined_ending() {
    let mut problems = jig_protocol_config::Problems::empty();
    problems.push(jig_protocol_config::Path::root().member(b"git"), jig_protocol_config::What::Form);
    problems.push(jig_protocol_config::Path::root().member(b"limits"), jig_protocol_config::What::Bound);
    let worker = worker::End::Stopped(worker::Stop::Start(problems.clone()));
    let joined = End::Worker(worker::End::Stopped(worker::Stop::Start(problems)));
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker), false, &directories, b"");
    assert_eq!(actual, ending_with_binary(BINARY, Some(&joined), false, &directories, b"", &[]));
    assert_eq!(
        ending_with_binary("temper-box", Some(&joined), false, &directories, b"", &[]),
        ("temper-box serve: git: invalid form\ntemper-box serve: limits: past its bound\n".into(), 1)
    );
    assert_eq!(
        actual,
        ("temper-worker serve: git: invalid form\ntemper-worker serve: limits: past its bound\n".into(), 1)
    );
}
#[test]
fn worker_drained_words_and_exit_equal_the_joined_ending() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Drained), false, &directories, b"");
    assert_eq!(
        actual,
        ending_with_binary(BINARY, Some(&End::Worker(worker::End::Drained)), false, &directories, b"", &[])
    );
    assert_eq!(actual, (String::new(), 0));
}
#[test]
fn worker_output_failure_words_and_exit_equal_the_joined_ending() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Drained), true, &directories, b"");
    assert_eq!(
        actual,
        ending_with_binary(BINARY, Some(&End::Worker(worker::End::Drained)), true, &directories, b"", &[])
    );
    assert_eq!(actual, (String::new(), 1));
}
