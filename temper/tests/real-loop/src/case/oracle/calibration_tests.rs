//! Pure oracle calibration (temper/docs/design/domain/testing.md, 2.4).
use super::*;
use serde_json::json;
use skein_fake_llm_domain::api::Part;

#[derive(Clone, Copy)]
enum Mutation {
    Owner,
    Commit,
    Draft,
    FailedRead(Option<Exit>),
    MissingRead(bool),
    Deferred(&'static str),
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
        Mutation::Owner | Mutation::Commit | Mutation::Draft | Mutation::Deferred(_) => {
            panic!("call mutation")
        }
    }
    (observed.documents, observed.observed_at) = read_documents(&calls, &reads);
    assert!(!observed.documents.contains_key(&Read::DesignEnded));
    assert!(!observed.observed_at.contains_key(&Read::DesignEnded));
}

fn rows() -> [Row; 9] {
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
        Row {
            name: "missing_deferred_work_is_reported_after_real_cancellation",
            mutation: Mutation::Deferred("Task 4 (x3)"),
            expected: vec![(
                DIRECTION_CANCELLED,
                "A must write a project note naming x3 and its deferred plan, then recall it",
            )],
        },
        Row {
            name: "missing_deferred_work_is_reported_after_real_cancellation",
            mutation: Mutation::Deferred("closed pull"),
            expected: vec![(
                DIRECTION_CANCELLED,
                "A must write a project note naming x3 and its deferred plan, then recall it",
            )],
        },
    ]
}

#[test]
fn passing_observations_report_the_six_calibration_rows_named_misses() {
    let opening = tests::met();
    let direction = direction_tests::met();
    assert_met(&step_4_1(&opening));
    assert_met(&step_5_4(&direction));
    for Row { name, mutation, expected } in rows() {
        let deferred = matches!(mutation, Mutation::Deferred(_));
        let mut observed = if deferred { direction.clone() } else { opening.clone() };
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
            Mutation::Deferred(reference) => {
                let body = crate::case::DEFERRED_WORDS.replace(reference, "omitted");
                assert_ne!(body, crate::case::DEFERRED_WORDS, "{name}: reference is present before mutation");
                for told in &mut observed.told {
                    if told.agent != Agent::StoryOwner {
                        continue;
                    }
                    for message in &mut told.history {
                        for part in &mut message.parts {
                            match part {
                                Part::ToolCall { name, arguments, .. } if name.as_ref() == b"note" => {
                                    let mut args: Value = serde_json::from_slice(arguments).expect("note arguments");
                                    args["body"] = json!(body);
                                    *arguments = serde_json::to_vec(&args).expect("mutated note").into();
                                }
                                Part::ToolOutput { output, .. } => {
                                    *output = String::from_utf8_lossy(output)
                                        .replace(crate::case::DEFERRED_WORDS, &body)
                                        .into_bytes()
                                        .into();
                                }
                                Part::Text { .. } | Part::Opaque { .. } | Part::ToolCall { .. } => {}
                            }
                        }
                    }
                }
            }
        }
        let (step, misses) = if deferred { ("5.4", step_5_4(&observed)) } else { ("4.1", step_4_1(&observed)) };
        let reported: Vec<_> = misses.iter().map(|miss| (miss.step, miss.promise, miss.what.as_str())).collect();
        let expected: Vec<_> = expected.iter().map(|&(promise, what)| (step, promise, what)).collect();
        assert_eq!(reported, expected, "{name}");
    }
}
