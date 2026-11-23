//! Pure oracle calibration (temper/docs/design/testing/real-loop.md).
use super::*;
use serde_json::json;

#[derive(Clone, Copy)]
enum Mutation {
    Draft,
    FailedRead(Option<Exit>),
    MissingRead(bool),
}

struct Row {
    name: &'static str,
    mutation: Mutation,
    expected: Vec<(Promise, &'static str)>,
}

fn called(document: &Value) -> calls::Called {
    calls::Called {
        locked: None,
        released: None,
        busy: 0,
        exit: Some(Exit::Code(0)),
        stdout: serde_json::to_vec(document).expect("document JSON"),
        stderr: Vec::new(),
        spawned: skein_lib::Time::ZERO,
        ended: skein_lib::Time::ZERO,
        observed_at: Some(Wall::from_nanos(7)),
        tries: 0,
        early: 0,
        answer: std::cell::RefCell::new(None),
        timeout: std::cell::RefCell::new(None),
    }
}

fn mutate_read(observed: &mut Observed, mutation: Mutation) {
    let mut reads = Vec::new();
    let mut calls = Vec::new();
    for (&read, document) in &observed.documents {
        reads.push((read, calls.len()));
        calls.push(called(document));
    }
    let position = reads.iter().position(|&(read, _)| read == Read::BenchmarkEnded).expect("benchmark read");
    let (documents, observed_at) = read_documents(&calls, &reads);
    assert_eq!(documents, observed.documents, "successful calls preserve every document");
    assert_eq!(observed_at.len(), reads.len(), "successful reads retain their wall input");
    match mutation {
        Mutation::FailedRead(exit) => calls[position].exit = exit,
        Mutation::MissingRead(true) => reads[position].1 = usize::MAX,
        Mutation::MissingRead(false) => {
            calls.truncate(position);
            // Keep the other reads intact while omitting the benchmark outcome.
            for (index, (&read, document)) in observed.documents.iter().enumerate().skip(position + 1) {
                reads[index].1 = calls.len();
                assert_ne!(read, Read::BenchmarkEnded);
                calls.push(called(document));
            }
            reads[position].1 = calls.len();
        }
        Mutation::Draft => {
            panic!("call mutation")
        }
    }
    (observed.documents, observed.observed_at) = read_documents(&calls, &reads);
    assert!(!observed.documents.contains_key(&Read::BenchmarkEnded));
    assert!(!observed.observed_at.contains_key(&Read::BenchmarkEnded));
}

fn rows() -> [Row; 1] {
    [Row {
        name: "a_changed_draft_reports_the_markdown_must",
        mutation: Mutation::Draft,
        expected: vec![(MARKDOWN, "kit's draft must be unchanged")],
    }]
}

#[test]
fn passing_observations_report_the_remaining_calibration_rows_named_misses() {
    let opening = tests::met();
    assert_met(&step_4_1(&opening));
    for Row { name, mutation, expected } in rows() {
        let mut observed = opening.clone();
        match mutation {
            Mutation::Draft => {
                observed.kit.main.insert(crate::case::DRAFT.as_bytes().to_vec(), b"# Changed draft\n".to_vec());
            }
            Mutation::FailedRead(_) | Mutation::MissingRead(_) => mutate_read(&mut observed, mutation),
        }
        let (step, misses) = ("4.1", step_4_1(&observed));
        let reported: Vec<_> = misses.iter().map(|miss| (miss.step, miss.promise, miss.what.as_str())).collect();
        let expected: Vec<_> = expected.iter().map(|&(promise, what)| (step, promise, what)).collect();
        assert_eq!(reported, expected, "{name}");
    }
}

// The opener no longer reads task identity or policy. Failed-read calibration
// belongs to the observation collector, independently of any checkpoint.
#[test]
fn failed_or_missing_document_acts_leave_no_observation() {
    for mutation in [
        Mutation::FailedRead(Some(Exit::Code(1))),
        Mutation::FailedRead(None),
        Mutation::MissingRead(true),
        Mutation::MissingRead(false),
    ] {
        let mut observed = tests::met();
        for (read, value) in
            [(Read::DirectorEnded, "director"), (Read::BenchmarkEnded, "benchmark"), (Read::BEnded, "owner")]
        {
            observed.documents.insert(read, json!({"task":value}));
        }
        mutate_read(&mut observed, mutation);
    }
}
