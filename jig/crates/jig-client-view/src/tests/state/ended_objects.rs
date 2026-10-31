#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public fragment proofs inspect full supplied sources and tree bindings"
)]
use crate::tests::fixtures::{Fixture, ReadPage, Served, Watched};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards::{CLIENT, text, words};
use crate::{JigBindings, tree::fragments::ended as view};
use skein_ui::{Builder, Role, Tree};

use jig_client::contract::*;
use skein_lib::{List, Wall};
fn task() -> Task {
    let mut task = crate::tests::tree::cards_objects::task_doc(&[ActKind::Cancel]);
    task.context = None;
    task
}
fn result() -> Result {
    Result {
        task: task().chip,
        ending: Ending::Done,
        value: Some(ResultValue::Report(DocumentText { bytes: text(b"independent result"), cut: true })),
        at: None,
        details: None,
        acts: List::with_capacity(0),
    }
}
fn envelope(key: ObjectKey, value: EndedValue) -> Ended {
    Ended { key, value, at: None, by: None }
}
fn person() -> Ended {
    let task = task();
    let source = crate::tests::tree::fragments::history::state();
    let path = crate::tests::kit::list([
        Bytes::new(Box::from(b"a/b".as_slice()), 128).unwrap(),
        Bytes::new(Box::from([255, 0]), 128).unwrap(),
    ]);
    let holdings = crate::tests::kit::list([
        GoalHolding::Write { connector: u16::MAX, path: path.clone(), kind: u16::MAX },
        GoalHolding::Slot { connector: 0, path, kind: 0 },
    ]);
    let goal = PersonProposalGoal {
        number: u64::MAX,
        project: u32::MAX,
        executor: Executor::Agent { charter: u32::MAX },
        spec: Specification {
            asks: false,
            words: text(b"raw goal"),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        },
        contract: TaskContractDocument::Report { words: u32::MAX },
        authority: Authority {
            grants: List::with_capacity(0),
            delegation: Delegation { kinds: List::with_capacity(0), tasks: u32::MAX, depth: u32::MAX },
            note_resources: List::with_capacity(0),
            ..source.authority.authority
        },
        numbers: Numbers { budget: 1, spent: 7, spent_below: 11, reserved: u64::MAX },
        funder: Funder::Task(u64::MAX),
        dependencies: List::with_capacity(0),
        holdings,
        wake: task.wake,
        tracked: Some(u32::MAX),
    };
    envelope(
        ObjectKey::Proposal(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }),
        EndedValue::PersonProposal {
            proposal: Box::new(PersonProposal {
                number: u64::MAX,
                proposer: u64::MAX,
                project: u32::MAX,
                goal,
                state: PersonProposalState::Pending { since: Wall::EPOCH },
            }),
            decision: None,
        },
    )
}
fn question(answer: bool, ending: bool) -> Ended {
    let q = crate::tests::tree::cards_objects::question();
    let answer = if answer {
        Some(ActivityLine {
            task: q.task.task,
            position: u64::MAX,
            at: Wall::from_nanos(17),
            line: Activity::Words {
                message: 99,
                by: Requester::Deployment { project: u32::MAX },
                kind: MessageKind::Answer { question: q.message },
                words: text(UTF8_TEXTS[0]),
            },
        })
    } else {
        None
    };
    envelope(
        ObjectKey::Question { task: q.task.task, message: q.message },
        EndedValue::Question { question: q, answer, ending: if ending { Some(Box::new(result())) } else { None } },
    )
}

fn build(value: &Ended, words: &crate::words::Words) -> (Tree, JigBindings) {
    let f = crate::tests::render_limits(&CLIENT, words).unwrap();
    let l = f;
    let domain = Fixture::new(&CLIENT);
    let mut b = Builder::new(&l);
    let mut bindings = JigBindings::new(f.bindings, CLIENT.client.text);
    view::render(&mut b, &domain.view(), value, words, &mut bindings);
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= f.nodes);
    assert!(tree.nodes().iter().map(crate::tests::text_bytes).sum::<u32>() <= f.text_bytes);
    assert!(
        tree.nodes()
            .iter()
            .all(|node| node.role != Role::Button && node.role != Role::TextField && node.role != Role::TextArea)
    );
    (tree, bindings)
}
#[test]
fn ended_task_renders_whole_task_and_independent_result_without_current_task_controls_or_context_defaults() {
    let mut goldens = Goldens::new("state/ended_objects/ended_task_renders");
    let row = task();
    assert!(row.context.is_none());
    let value = envelope(ObjectKey::Task(8), EndedValue::Task { document: Box::new(row), result: Box::new(result()) });
    let (tree, _) = build(&value, words());
    goldens.check(&tree);
    assert!(tree.nodes().iter().all(|node| node.role != Role::Button));
}
#[test]
fn ended_raw_person_goal_displays_all_four_numbers_and_distinct_unjoined_opaque_holdings() {
    let mut goldens = Goldens::new("state/ended_objects/ended_raw_person");
    let value = person();
    let (tree, _) = build(&value, words());
    goldens.check(&tree);
    let selected = words();
    for choice in [
        ProposalChoice::Accepted,
        ProposalChoice::Rejected,
        ProposalChoice::Passed,
        ProposalChoice::Withdrawn,
        ProposalChoice::Stale,
    ] {
        let mut decided = value.clone();
        let EndedValue::PersonProposal { decision, .. } = &mut decided.value else {
            panic!("supplied raw person proposal")
        };
        *decision = Some(ProposalDecision {
            project: u32::MAX,
            proposer: Proposer::Party(u64::MAX),
            proposal: u64::MAX,
            kind: MessageProposalKind::Batch,
            by: u64::MAX,
            at: Wall::EPOCH,
            choice,
            created: None,
        });
        let (tree, _) = build(&decided, selected);
        goldens.check(&tree);
        assert!(tree.nodes().iter().all(|node| node.role != Role::Button));
    }
}
#[test]
fn ended_retained_proposal_uses_bare_whole_fragment_and_keeps_pending_state_without_fake_history_row() {
    let mut goldens = Goldens::new("state/ended_objects/ended_retained_proposal");
    let p = crate::tests::tree::fragments::history::proposal(RetainedAction::Release { task: u64::MAX });
    let value = envelope(
        ObjectKey::Proposal(ProposalKey { proposer: Proposer::Task(p.proposer), number: p.number }),
        EndedValue::Proposal(p),
    );
    assert!(!value.is_terminal());
    let (tree, _) = build(&value, words());
    goldens.check(&tree);
    assert!(
        crate::tree::fragments::history::proposal_max_nodes(&CLIENT, &(CLIENT).client).unwrap()
            <= view::max_nodes(&CLIENT, &(CLIENT).client).unwrap()
    );
}
#[test]
fn ended_escalation_keeps_rejection_revision_sender_and_optional_retained_hold_independently() {
    let mut goldens = Goldens::new("state/ended_objects/ended_escalation_keeps");
    let value = envelope(
        ObjectKey::Escalation(8),
        EndedValue::Escalation {
            task: Box::new(task()),
            decision: EscalationDecisionRecord {
                project: 4,
                requester: u64::MAX,
                task: 8,
                revision: u64::MAX,
                by: 17,
                at: Wall::EPOCH,
                decision: EscalationDecision::Reject { reason: text(UTF8_TEXTS[0]) },
            },
            hold: None,
        },
    );
    let (tree, _) = build(&value, words());
    goldens.check(&tree);
}
#[test]
fn ended_question_keeps_answer_full_result_and_outer_absence_as_three_independent_sources() {
    let mut goldens = Goldens::new("state/ended_objects/ended_question_keeps");
    let mut value = question(true, true);
    value.at = Some(Wall::from_nanos(23));
    value.by = Some(Requester::Person(19));
    let (tree, _) = build(&value, words());
    goldens.check(&tree);
    let pending = question(false, false);
    assert!(!pending.is_terminal());
    let (tree, _) = build(&pending, words());
    goldens.check(&tree);
}
#[test]
fn ended_person_task_keeps_whole_address_contract_loss_and_independent_result_without_answer_form() {
    let mut goldens = Goldens::new("state/ended_objects/ended_person_task");
    let t = task();
    let document = PersonTask {
        task: t.chip,
        revision: Revision { value: u64::MAX },
        requester: Requester::Deployment { project: u32::MAX },
        spec: t.spec,
        address: PersonAddress::Role(u32::MAX),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract: TaskContractDocument::Report { words: 32 },
        results: List::with_capacity(0),
        cut: true,
        acts: t.acts,
    };
    let value = envelope(
        ObjectKey::PersonTask(8),
        EndedValue::PersonTask { document: Box::new(document), result: Box::new(result()) },
    );
    let (tree, _) = build(&value, words());
    goldens.check(&tree);
    assert!(tree.nodes().iter().all(|node| node.role != Role::TextField && node.role != Role::TextArea));
}
#[test]
fn ended_passive_card_retains_named_whole_departure_and_does_not_bind_current_question_actions() {
    let mut goldens = Goldens::new("state/ended_objects/ended_passive_card");
    let value = question(true, false);
    let (d, id) = crate::tests::tree::cards_objects::loaded(value.key, ReadValue::Ended(Box::new(value.clone())));
    let f = crate::tests::render_limits(&CLIENT, words()).unwrap();
    let mut l = f;
    l.nodes = l.nodes.checked_add(64).unwrap();
    l.text_bytes = l.text_bytes.checked_add(64_u32.checked_mul(l.text_per_node).unwrap()).unwrap();
    let mut b = Builder::new(&l);
    let mut bindings = JigBindings::new(l.bindings, CLIENT.client.text);
    assert!(crate::tree::cards::card(
        &mut b,
        &d.view(),
        id,
        &crate::tree::cards::Place::Elsewhere,
        words(),
        &mut bindings
    ));
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    goldens.check(&tree);
    assert!(tree.nodes().iter().all(|node| node.role != Role::Button));
    let crate::state::objects::Card::Leaving { how: crate::state::objects::Left::DocumentEnded { facts }, .. } =
        &d.objects().get(id).unwrap().card
    else {
        panic!("whole ended departure")
    };
    assert_eq!(facts.as_ref(), &value);
}
fn step(
    domain: &mut Fixture,
    l: &crate::tests::fixtures::Profile,
    event: impl crate::tests::fixtures::Input,
) -> Vec<crate::Request> {
    domain.ui.limits = l.view;
    domain.drive(skein_lib::Time::ZERO, event).into_vec()
}

fn presence_finish(domain: &mut Fixture, l: &crate::tests::fixtures::Profile, task_read: jig_client::Handle, mask: u8) {
    let mut goldens = Goldens::case("state/ended_objects/presence", mask);
    let mut doc = task();
    doc.inputs = List::with_capacity(l.client.rows);
    for _ in 0..l.client.rows {
        doc.inputs.push(TaskLink::Stored { number: 9, status: None }).unwrap();
    }
    let supplied =
        envelope(ObjectKey::Task(8), EndedValue::Task { document: Box::new(doc), result: Box::new(result()) });
    step(
        domain,
        l,
        ReadPage { read: task_read, result: ReadResult::Value(ReadValue::Ended(Box::new(supplied.clone()))) },
    );
    let id = domain.objects().find(supplied.key).unwrap();
    let words = words();
    let f = crate::tests::render_limits(l, words).unwrap();
    let mut cap = f;
    cap.nodes = cap.nodes.checked_add(64).unwrap();
    cap.text_bytes = cap.text_bytes.checked_add(64_u32.checked_mul(cap.text_per_node).unwrap()).unwrap();
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, l.client.text);
    assert!(crate::tree::cards::card(
        &mut b,
        &domain.view(),
        id,
        &crate::tree::cards::Place::Elsewhere,
        words,
        &mut bindings
    ));
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= f.nodes.checked_add(64).unwrap());
    goldens.check(&tree);
    assert_eq!(
        domain.objects().len(),
        1_u32
            .checked_add(u32::from(mask & 1 != 0))
            .unwrap()
            .checked_add(u32::from(mask & 2 != 0))
            .unwrap()
            .checked_add(u32::from(mask & 4 != 0))
            .unwrap()
    );
    assert!(tree.nodes().iter().all(|node| node.role != Role::Button));
}

#[test]
fn ended_selected_language_uses_checked_native_captions_and_preserves_opaque_goal_values() {
    use skein_ui::phrases::{PhraseLimits, Phrasebook, Segment};
    let mut goldens = Goldens::new("state/ended_objects/ended_selected_language");
    let english = crate::words::phrases::english::language();
    let mut selected = english.clone();
    for phrase in &mut selected.phrases {
        if phrase.id == crate::words::ended_words::EndedPhrase::Ended.id() {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    if let Segment::Text(bytes) = segment {
                        *bytes = Box::from(b"Selected ended source".as_slice());
                    }
                }
            }
        }
    }
    let words = crate::words::Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(CLIENT.client.text),
            Box::new([english, selected]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::document_words::phrase_count(),
                holes: 4,
                segments: 9000,
                text_bytes: 200_000,
                hole_bytes: CLIENT.client.text,
            },
        )
        .unwrap(),
        1,
        0,
    );
    let value = person();
    let (tree, _) = build(&value, &words);
    goldens.check(&tree);
    let f = crate::tests::render_limits(&CLIENT, &words).unwrap();
    assert!(tree.len() <= f.nodes);
    assert!(tree.nodes().iter().map(crate::tests::text_bytes).sum::<u32>() <= f.text_bytes);
    for phrase in crate::words::ended_words::ALL {
        assert!(words.book().capacity(phrase.id()).is_some());
    }
}

const UTF8_TEXTS: [&[u8]; 2] = [b"~\0", &[b'~'; 129]];

fn presence_domain(l: &crate::tests::fixtures::Profile) -> Fixture {
    use crate::{Event, Request};
    let mut domain = Fixture::new(l);
    let first = step(
        &mut domain,
        l,
        Event::Start {
            seed: 8,
            place: crate::state::address::Place::Jig(crate::state::address::Address::Inbox { project: None }),
            records: List::with_capacity(0),
        },
    );
    let watch = first
        .iter()
        .find_map(|request| {
            if let Request::Client {
                asked: jig_client::Event::Watch { handle: watch, subject: jig_client::Watchable::Jig(Subject::Me) },
            } = request
            {
                Some(*watch)
            } else {
                None
            }
        })
        .unwrap();
    step(&mut domain, l, Served { served: true });
    step(
        &mut domain,
        l,
        Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(crate::tests::tree::cards::document()))) },
    );
    domain
}

fn presence(mask: u8) {
    use crate::{Event, Request};
    let l = crate::tests::fixtures::Profile {
        view: crate::Config { objects: 4, ..(CLIENT).view },
        client: jig_client::DomainLimits { reads: 8, ..(CLIENT).client },
    };
    let mut domain = presence_domain(&l);
    let proposal_key = ProposalKey { proposer: Proposer::Task(9), number: 7 };
    let mut keys = crate::tests::kit::list_in(4, [ObjectKey::Task(8)]);
    if mask & 1 != 0 {
        keys.push(ObjectKey::Result(9)).unwrap();
    }
    if mask & 2 != 0 {
        keys.push(ObjectKey::PersonTask(8)).unwrap();
    }
    if mask & 4 != 0 {
        keys.push(ObjectKey::Proposal(proposal_key)).unwrap();
    }
    let reads = step(&mut domain, &l, Event::Named { sections: List::with_capacity(0), objects: keys });
    let mut task_read = None;
    for request in reads {
        let Request::Client { asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(what) } } =
            request
        else {
            continue;
        };
        let value = match what {
            Read::Task { task: 8 } => {
                task_read = Some(read);
                continue;
            }
            Read::Result { task: 9 } => {
                let mut value = result();
                value.task.task = 9;
                value.value = Some(ResultValue::Verdict {
                    verdict: 1,
                    words: DocumentText::complete(text(b"whole kept verdict")),
                    fields: List::with_capacity(0),
                    tasks: List::with_capacity(0),
                    proposal: Some(proposal_key),
                });
                ReadValue::Result(Box::new(value))
            }
            Read::PersonTask { task: 8 } => {
                let t = task();
                ReadValue::PersonTask(Box::new(PersonTask {
                    task: t.chip,
                    revision: t.spec_revision,
                    requester: t.requester,
                    spec: t.spec,
                    address: PersonAddress::Party(1),
                    taken_by: None,
                    since: None,
                    stall: None,
                    deadline: None,
                    contract: t.result_contract,
                    results: List::with_capacity(0),
                    cut: false,
                    acts: List::with_capacity(0),
                }))
            }
            Read::Proposal { proposal } => {
                ReadValue::Proposal(Box::new(crate::tests::fixtures::proposal(proposal, text(b"whole kept proposal"))))
            }
            _ => continue,
        };
        step(&mut domain, &l, ReadPage { read, result: ReadResult::Value(value) });
    }
    presence_finish(&mut domain, &l, task_read.unwrap(), mask);
}

#[test]
fn ended_task_absent_result_person_and_proposal_keeps_loading_within_whole_bound() {
    presence(0);
}

#[test]
fn ended_task_result_presence_keeps_repeated_whole_inputs_within_bound() {
    presence(1);
}

#[test]
fn ended_task_person_presence_keeps_whole_supplied_person_within_bound() {
    presence(2);
}

#[test]
fn ended_task_result_and_person_presence_keeps_both_whole_sources_within_bound() {
    presence(3);
}

#[test]
fn ended_task_proposal_presence_keeps_supplied_kept_projection_within_bound() {
    presence(4);
}

#[test]
fn ended_task_result_and_proposal_presence_keeps_original_verdict_key_projection_within_bound() {
    presence(5);
}

#[test]
fn ended_task_person_and_proposal_presence_keeps_independent_sources_within_bound() {
    presence(6);
}

#[test]
fn ended_task_all_three_supplied_presences_keep_repetition_and_whole_sources_within_bound() {
    presence(7);
}
