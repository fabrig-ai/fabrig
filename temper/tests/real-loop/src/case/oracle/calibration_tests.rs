//! Pure oracle calibration (temper/docs/design/testing/real-loop.md).
use super::*;
use serde_json::json;

#[derive(Clone, Copy)]
enum Mutation {
    Owner,
    Commit,
    Draft,
    FailedRead(Option<Exit>),
    MissingRead(bool),
}

struct Row {
    name: &'static str,
    mutation: Mutation,
    expected: Vec<(Promise, &'static str)>,
}

fn absent_design() -> Vec<(Promise, &'static str)> {
    vec![
        (DELEGATE, "DesignEnded: task number must be 3"),
        (DELEGATE, "DesignEnded: project must be 1"),
        (AUTHORITY, "Design.authority/spend exceeds its parent or is absent"),
        (AUTHORITY, "Design.authority/delegation/tasks exceeds its parent or is absent"),
        (AUTHORITY, "Design.authority/delegation/depth exceeds its parent or is absent"),
        (AUTHORITY, "Design.authority.tools must be a subset"),
        (AUTHORITY, "Design.authority.delegation.kinds is absent"),
        (AUTHORITY, "Design.authority.grants is absent"),
        (SPEND, "A.budget.spent_below must cover its child's spent + spent_below"),
    ]
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
    let position = reads.iter().position(|&(read, _)| read == Read::DesignEnded).expect("design read");
    let (documents, observed_at) = read_documents(&calls, &reads);
    assert_eq!(documents, observed.documents, "successful calls preserve every document");
    assert_eq!(observed_at.len(), reads.len(), "successful reads retain their wall input");
    match mutation {
        Mutation::FailedRead(exit) => calls[position].exit = exit,
        Mutation::MissingRead(true) => reads[position].1 = usize::MAX,
        Mutation::MissingRead(false) => {
            calls.truncate(position);
            // Keep the other reads intact while omitting the design outcome.
            for (index, (&read, document)) in observed.documents.iter().enumerate().skip(position + 1) {
                reads[index].1 = calls.len();
                assert_ne!(read, Read::DesignEnded);
                calls.push(called(document));
            }
            reads[position].1 = calls.len();
        }
        Mutation::Owner | Mutation::Commit | Mutation::Draft => {
            panic!("call mutation")
        }
    }
    (observed.documents, observed.observed_at) = read_documents(&calls, &reads);
    assert!(!observed.documents.contains_key(&Read::DesignEnded));
    assert!(!observed.observed_at.contains_key(&Read::DesignEnded));
}

fn rows() -> [Row; 7] {
    [
        Row {
            name: "the_default_model_reports_the_delegation_must",
            mutation: Mutation::Owner,
            expected: vec![(DELEGATE, "A.llm.main.model must be owner"), (DELEGATE, "A.llm.main.effort must be xhigh")],
        },
        Row {
            name: "a_brief_without_the_commit_reports_the_delegation_must",
            mutation: Mutation::Commit,
            expected: vec![(DELEGATE, "A.spec must name the draft and its first commit")],
        },
        Row {
            name: "a_changed_draft_reports_the_markdown_must",
            mutation: Mutation::Draft,
            expected: vec![(MARKDOWN, "kit's draft must be unchanged")],
        },
        Row {
            name: "a_failed_document_act_cannot_silently_pass",
            mutation: Mutation::FailedRead(Some(Exit::Code(1))),
            expected: absent_design(),
        },
        Row {
            name: "a_failed_document_act_cannot_silently_pass",
            mutation: Mutation::FailedRead(None),
            expected: absent_design(),
        },
        Row {
            name: "a_read_past_calls_cannot_silently_pass",
            mutation: Mutation::MissingRead(true),
            expected: absent_design(),
        },
        Row {
            name: "a_read_past_calls_cannot_silently_pass",
            mutation: Mutation::MissingRead(false),
            expected: absent_design(),
        },
    ]
}

#[test]
fn passing_observations_report_the_calibration_rows_named_misses() {
    let opening = tests::met();
    assert_met(&step_4_1(&opening));
    for Row { name, mutation, expected } in rows() {
        let mut observed = opening.clone();
        match mutation {
            Mutation::Owner => {
                observed.documents.get_mut(&Read::AEnded).expect("A")["task"]["llm"] =
                    json!({"main":{"model":"default","effort":"low"}});
            }
            Mutation::Commit => {
                observed.documents.get_mut(&Read::AEnded).expect("A")["task"]["spec"]["specification"]["words"]["bytes"] =
                    json!(format!("{}{}", crate::case::A_SPEC_PREFIX, crate::case::A_SPEC_SUFFIX));
            }
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
