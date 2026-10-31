#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary tests inspect trees and script public domain requests"
)]
use crate::tests::fixtures::refusal_answer;
use crate::tests::fixtures::{Fixture, ReadPage, Watched};
use crate::tests::kit::Goldens;
pub(in crate::tests) use crate::tests::kit::card as render;
pub(in crate::tests) use crate::tests::kit::list;
use crate::tests::tree::cards::{CLIENT, contains, document, drive, limits, start, text, words};
use crate::{JigBindings, decode, tree::cards};
use skein_ui::Act;
use skein_ui::{Builder, Role, Tree};

use crate::{
    Event, Request,
    state::{action::Act as Action, fields::Purpose, objects::Object},
};
use jig_client::contract::*;
use skein_lib::{Duration, Id, List, Wall};

fn chip() -> Chip {
    Chip {
        task: 8,
        title: DocumentText::complete(text(b"Review")),
        executor: Executor::Party { party: 1 },
        phase: Phase::Held,
        details: Box::new({
            let mut phase =
                crate::tests::fixtures::chip_phase(OwnerChipPhase::Held { reason: HoldCategory::Budget, revision: 4 });
            phase.details.hold = Some(crate::tests::fixtures::hold());
            phase
        }),
    }
}
pub(in crate::tests) fn owner_authority() -> OwnerAuthorityDocument {
    OwnerAuthorityDocument {
        authority: OwnerAuthority {
            models: None,
            tools: 0,
            grants: List::with_capacity(0),
            delegation: OwnerDelegation { kinds: List::with_capacity(0), tasks: 3, depth: 2 },
            spend: 10,
            deadline: None,
            notes: 0,
            note_resources: List::with_capacity(0),
        },
        cut: false,
    }
}

fn offered(act: ActKind) -> List<Offered> {
    list([
        Offered { effect: None, act, admission: Admission::Allowed },
        Offered {
            effect: None,
            act: ActKind::PassUp,
            admission: Admission::Refused(Refusal::Authority {
                lacking: Lacking::Budget { needed: 40, left: 12 },
                proposable: None,
            }),
        },
    ])
}

pub(in crate::tests) fn question() -> Question {
    Question {
        task: chip(),
        from: None,
        message: 1,
        words: DocumentText::complete(text(b"whole question")),
        since: Wall::EPOCH,
        acts: offered(ActKind::Answer),
        ending: None,
    }
}

fn result() -> Result {
    Result {
        task: chip(),
        ending: Ending::Failed { reason: Some(text(b"whole failure")) },
        value: Some(ResultValue::Report(DocumentText::complete(text(b"whole report")))),
        at: Some(Wall::EPOCH),
        acts: offered(ActKind::Seen),
        details: None,
    }
}
fn loaded_with_watch(key: ObjectKey, value: ReadValue) -> (Fixture, Id<Object>, jig_client::Handle) {
    let (mut d, frame) = start();
    let out = drive(&mut d, Watched { watch: frame, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
    let inbox =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Inbox { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let out = drive(
        &mut d,
        Watched {
            watch: inbox,
            event: WatchEvent::Delivered(Delivery::OwnerInbox(Box::new(crate::tests::fixtures::inbox(list([
                crate::tests::fixtures::inbox_entry(key),
            ]))))),
        },
    );
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Read { handle: read, .. } } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(&mut d, ReadPage { read, result: ReadResult::Value(value) });
    let id = d.objects().find(key).unwrap();
    (d, id, inbox)
}
pub(in crate::tests) fn loaded(key: ObjectKey, value: ReadValue) -> (Fixture, Id<Object>) {
    let (d, id, _) = loaded_with_watch(key, value);
    (d, id)
}

fn press(tree: &Tree, label: &[u8], bindings: &mut JigBindings) -> Action {
    let token = tree
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(label))
        .unwrap()
        .binding
        .unwrap()
        .token;
    decode(bindings, token, Act::Press).unwrap()
}

fn result_value_payloads(words: &crate::words::Words, part: &Part) {
    let mut goldens = Goldens::new("tree/cards_objects/result_value_payloads");
    let (d, _) = start();
    for value in [
        ResultValue::Report(DocumentText::complete(text(b"full report"))),
        ResultValue::Words(DocumentText::complete(text(b"full words"))),
        ResultValue::Verdict {
            verdict: 2,
            words: DocumentText::complete(text(b"explanation")),
            fields: list([ResultField { name: b"field".as_slice().into(), value: b"full field".as_slice().into() }]),
            tasks: list([]),
            proposal: None,
        },
        ResultValue::Connector { summary: DocumentText::complete(text(b"full connector")), part: part.clone() },
        ResultValue::Choice {
            option: 3,
            by: crate::tests::tree::cards::party(),
            reason: DocumentText::complete(text(b"full reason")),
        },
        ResultValue::Approved {
            state: b"full state".as_slice().into(),
            words: Some(DocumentText::complete(text(b"approved words"))),
        },
        ResultValue::Failure(DocumentText::complete(text(b"full failure"))),
    ] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.client.text);
        cards::result::value_nodes(&mut b, &d.view(), &value, words, &mut bindings);
        b.close();
        goldens.check(&b.finish());
    }
}

fn impact() -> Impact {
    Impact {
        tasks: PhaseCounts { ended: 0, waiting: 1, running: 2, held: 3, closing: 4, done: 5, failed: 6, cancelled: 7 },
        live_runs: 8,
        proposals: 9,
    }
}

fn dialog(d: &Fixture, words: &crate::words::Words, bindings: &mut JigBindings) -> Tree {
    bindings.begin();
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    crate::tree::confirmation::confirmation(&mut b, &d.view(), words, bindings);
    b.close();
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= crate::confirmation_nodes(&CLIENT, &(CLIENT).client).unwrap().checked_add(1).unwrap());
    tree
}
pub(in crate::tests) fn task_doc(acts: &[ActKind]) -> Task {
    Task {
        dependency_facts: List::with_capacity(0),
        kept_account: None,
        run_wait: None,
        settings: None,
        llm: None,
        label: 0,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        last_activity: Wall::EPOCH,
        waits_on: WaitsOn::Nothing,
        chip: chip(),
        goal: Some(8),
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 4,
        requester: OwnerRequester::Person(1),
        lineage: list([]),
        spec: specification(text(b"whole task specification")),
        spec_revision: Revision { value: 6 },
        person_revision: None,
        taken_by: None,
        instructions: Some(DocumentText::complete(text(b"whole task instructions"))),
        wake: OwnerWakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Each,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: Subscriptions { entries: List::with_capacity(0) },
        result_contract: TaskContractDocument::Report { words: 32 },
        dependencies: list([]),
        cut: false,
        references: list([]),
        resources: list([]),
        budget: OwnerBudget {
            numbers: OwnerNumbers { budget: 20, spent: 0, spent_below: 0, reserved: 0 },
            left: 20,
            funder: Funder::Pool { project: 4, person: 1, period: 3 },
            deadline: None,
        },
        authority: owner_authority(),
        hold: Some(Box::new(crate::tests::fixtures::hold())),
        proposals: list([]),
        activity: 0,
        changed: Wall::EPOCH,
        acts: offered_acts(acts, ReleaseEffect::Judged),
        result: None,
        delegates: list([LiveDelegate { chip: chip(), takeover_budget: 13 }]),

        context: Some(Box::new(TaskContext {
            parts: list([]),
            closing: None,
            spend: list([]),
            runtime: Runtime::Held { was: None },
            impact: impact(),
            writer_slots: list([]),
        })),
    }
}
fn input(tree: &Tree, label: &[u8], value: &[u8], bindings: &mut JigBindings) -> Action {
    let node = tree.nodes().iter().find(|n| n.name.as_deref() == Some(label) && n.binding.is_some()).unwrap();
    decode(bindings, node.binding.unwrap().token, Act::Input { value: Box::from(value) }).unwrap()
}

fn notice_scenarios(words: &crate::words::Words) {
    use crate::tree::notices::kept::Kind;
    let mut goldens = Goldens::new("tree/cards_objects/notice_scenarios");
    for kind in [
        Kind::Link(jig_client::Link::Offline),
        Kind::Link(jig_client::Link::Live),
        Kind::Answer {
            object: ObjectKey::Task(8),
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::Released { task: 8 })),
        },
        Kind::PageAnswer {
            answer: refusal_answer(Refusal::Authority {
                lacking: Lacking::Budget { needed: 40, left: 12 },
                proposable: None,
            }),
        },
        Kind::Unknown { object: ObjectKey::Escalation(8), act: ActKind::Release },
        Kind::PageUnknown { purpose: crate::state::page::IntentPurpose::CreateGoal },
        Kind::Refused(Limit::Objects),
    ] {
        let mut b = Builder::new(&limits());
        crate::tree::notices::facts(&mut b, &kind, words, CLIENT.client.text);
        let tree = b.finish();
        crate::tests::tree::cards_access::check(&tree);
        assert_eq!(tree.nodes()[0].role, Role::Status);
        goldens.check(&tree);
        assert!(tree.len() > 1);
    }
    let (mut d, _) = start();
    crate::tree::notices::kept::show(&mut d, Kind::Refused(Limit::Objects));
    let mut b = Builder::new(&limits());
    let mut bindings = JigBindings::new(limits().bindings, CLIENT.client.text);
    crate::tree::notices::notices(&mut b, &d.view(), words, &mut bindings);
    let tree = b.finish();
    crate::tests::tree::cards_access::check(&tree);
    goldens.check(&tree);
    let dismiss = press(&tree, b"Dismiss", &mut bindings);
    drive(&mut d, Event::Act { action: dismiss });
    assert!(d.notices().ids().is_empty());
}

fn note_loaded(note: Note) -> (Fixture, Id<Object>) {
    use crate::state::address::{Address, Place, ProjectSection};
    let number = note.number;
    let (mut d, frame) = start();
    drive(&mut d, Watched { watch: frame, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
    let out = drive(
        &mut d,
        Event::Went { place: Place::Jig(Address::Project { project: 4, section: ProjectSection::Notes }) },
    );
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Client {
                asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Notes { .. }) },
            } => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut d,
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Notes(Box::new(NotePage {
                cut: false,
                rows: list([note]),
                before: None,
                after: None,
            }))),
        },
    );
    let id = d.objects().find(ObjectKey::Note { project: 4, note: number }).unwrap();
    (d, id)
}

fn closing_connectors_use_their_owners_words(words: &crate::words::Words) {
    let mut goldens = Goldens::new("tree/cards_objects/closing_connectors_use");
    let mut b = Builder::new(&limits());
    let mut bindings = JigBindings::new(16, CLIENT.client.text);
    b.open(Role::Page);
    cards::common::closing(
        &mut b,
        &Closing { delegates: 2, connectors: Some(list([7, 11])), stage: Some(ClosingStage::Effects), ending: None },
        words,
        &mut bindings,
    );
    b.close();
    let slots = b.slots(0);
    assert_eq!(slots.len(), 2);
    for slot in &slots {
        let connector = match bindings.part(slot.reference.token).unwrap() {
            Part::Connector { connector } => *connector,
            _ => panic!("a name-only connector part"),
        };
        assert_eq!(slot.reference.owner, (crate::Owner::Application { number: connector }).opaque());
        let mut own = Builder::new(&limits());
        own.open(Role::Region);
        own.text(skein_ui::Run::Plain, if connector == 7 { b"amber" } else { b"blue" });
        own.close();
        b.splice(slot.id, own);
    }
    let tree = b.finish();
    goldens.check(&tree);
}

fn authority_refusal_uses_only_supplied_proposal_eligibility(words: &crate::words::Words) {
    let mut goldens = Goldens::new("tree/cards_objects/authority_refusal_uses");
    for proposable in [None, Some(true), Some(false)] {
        let mut b = Builder::new(&limits());
        b.open(Role::Page);
        words.refusal(&mut b, &Refusal::Authority { lacking: Lacking::Approver, proposable }, CLIENT.client.text);
        b.close();
        let tree = b.finish();
        goldens.check(&tree);
    }
}

fn collective_recipients_name_every_supplied_role_in_order(words: &crate::words::Words) {
    let mut b = Builder::new(&limits());
    b.open(Role::Page);
    cards::common::recipient(&mut b, &Recipient::Roles { project: 4, roles: list([7, 2]) }, words);
    b.close();
    let tree = b.finish();
    let names: Vec<_> =
        tree.nodes().iter().flat_map(|node| node.text.as_slice().iter().map(|run| run.text.as_ref())).collect();
    assert_eq!(names, [b"Waiting at policy roles in project 4".as_slice(), b"Role 7", b"Role 2"]);
}

#[test]
fn approved_and_failed_results_show_only_their_supplied_owner_payloads() {
    let mut goldens = Goldens::new("tree/cards_objects/approved_and_failed");
    let (domain, _) = start();
    let words = words();
    for explanation in [None, Some(text(b"")), Some(text(b"exact explanation"))] {
        let value = ResultValue::Approved {
            state: Box::new([0, 255, 1]),
            words: explanation.clone().map(DocumentText::complete),
        };
        let mut builder = Builder::new(&limits());
        builder.open(Role::Page);
        let mut bindings = JigBindings::new(64, CLIENT.client.text);
        cards::result::value_nodes(&mut builder, &domain.view(), &value, words, &mut bindings);
        builder.close();
        let tree = builder.finish();
        goldens.check(&tree);
    }
    let mut builder = Builder::new(&limits());
    builder.open(Role::Page);
    let mut bindings = JigBindings::new(64, CLIENT.client.text);
    cards::result::value_nodes(
        &mut builder,
        &domain.view(),
        &ResultValue::Failure(DocumentText::complete(text(b"cannot complete"))),
        words,
        &mut bindings,
    );
    builder.close();
    let tree = builder.finish();
    goldens.check(&tree);
}

fn specification(words: Text) -> SpecificationDocument {
    SpecificationDocument::complete(Specification {
        asks: false,
        words,
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    })
}

#[test]
fn whole_specification_words_edit_preserves_parameters_and_input_identities() {
    let mut goldens = Goldens::new("tree/cards_objects/whole_specification_words");
    let mut row = task_doc(&[ActKind::Amend]);
    row.spec.specification.parameters = list([
        ParameterDocument::Number { name: u32::MAX, value: u64::MAX },
        ParameterDocument::Bytes { name: 8, value: DocumentText::complete(text(UTF8_TEXTS[0])) },
        ParameterDocument::Resource { name: 9, connector: u16::MAX, resource: u64::MAX },
    ]);
    row.spec.specification.inputs = list([u64::MAX, 0]);
    let expected = row.spec.whole().unwrap();
    let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
    drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
    let words = words();
    let mut bindings = JigBindings::new(256, CLIENT.client.text);
    let tree = dialog(&domain, words, &mut bindings);
    goldens.check(&tree);
    drive(&mut domain, Event::Act { action: input(&tree, b"New specification", b"replacement", &mut bindings) });
    let out = drive(&mut domain, Event::Act { action: Action::Confirm });
    let sent = out
        .iter()
        .find_map(|request| match request {
            Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } => {
                Some(&**ask)
            }
            _ => None,
        })
        .expect("whole amendment sent");
    let Ask::Amend { change, .. } = sent else { panic!("amendment") };
    let replacement = change.spec.as_ref().expect("whole specification");
    assert_eq!(replacement.words.bytes(), b"replacement");
    assert_eq!(replacement.parameters, expected.parameters);
    assert_eq!(replacement.inputs, expected.inputs);
}

#[test]
fn cut_specification_is_shown_and_cannot_be_reconstructed_by_the_words_editor() {
    let mut goldens = Goldens::new("tree/cards_objects/cut_specification_is");
    let mut row = task_doc(&[ActKind::Amend]);
    row.spec.cut = true;
    let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    let words = words();
    let mut builder = Builder::new(&limits());
    builder.open(Role::Page);
    crate::tree::fragments::specification::document(
        &mut builder,
        &{
            let mut document = SpecificationDocument::complete(Specification {
                asks: false,
                words: text(b"partial"),
                parameters: list([]),
                inputs: list([]),
            });
            document.cut = true;
            document
        },
        words,
    );
    builder.close();
    goldens.check(&builder.finish());
    drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
    drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
    let mut bindings = JigBindings::new(256, CLIENT.client.text);
    let tree = dialog(&domain, words, &mut bindings);
    goldens.check(&tree);
    assert!(drive(&mut domain, Event::Act { action: Action::Confirm }).is_empty());
}

#[test]
fn retained_task_facts_keep_person_revision_taker_and_full_executor_identity_independent() {
    for (executor, person_revision, taken_by) in [
        (Executor::Role { project: 4, role: u32::MAX }, Some(Revision { value: u64::MAX }), Some(u64::MAX - 1)),
        (Executor::Procedure { connector: 2, kind: u32::MAX }, None, None),
    ] {
        let mut value = task_doc(&[ActKind::Amend]);
        value.goal = None;
        value.chip.executor = executor;
        value.person_revision = person_revision;
        value.taken_by = taken_by;
        let (domain, object, _) = loaded_with_watch(ObjectKey::Task(8), ReadValue::Task(Box::new(value)));
        let Some(crate::state::objects::Body::Task(retained)) = &domain.objects().get(object).unwrap().body else {
            panic!("the actual task document remains retained");
        };
        assert_eq!(retained.chip.executor, executor);
        assert_eq!(retained.person_revision, person_revision);
        assert_eq!(retained.taken_by, taken_by);
        assert_eq!(retained.spec_revision, Revision { value: 6 });
        assert_eq!(retained.hold.as_ref().unwrap().revision(), Revision { value: 4 });
    }
}

#[test]
fn specification_documents_keep_independent_text_and_list_loss_and_block_whole_edits() {
    let mut goldens = Goldens::new("tree/cards_objects/specification_documents_keep");
    for mask in 0_u8..8 {
        let mut row = task_doc(&[ActKind::Amend]);
        row.spec.specification.parameters = list([ParameterDocument::Bytes {
            name: 8,
            value: DocumentText { bytes: text(UTF8_TEXTS[0]), cut: mask & 2 != 0 },
        }]);
        row.spec.specification.words.cut = mask & 1 != 0;
        row.spec.cut = mask & 4 != 0;
        row.spec.specification.inputs = list([u64::MAX]);
        assert_eq!(row.spec.has_loss(), mask != 0);
        assert_eq!(row.spec.whole().is_some(), mask == 0);
        let words = words();
        let mut builder = Builder::new(&limits());
        crate::tree::fragments::specification::document(&mut builder, &row.spec, words);
        let tree = builder.finish();
        goldens.check(&tree);
        let (mut domain, id) = loaded(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act: ActKind::Amend } });
        drive(&mut domain, Event::Act { action: Action::Edit { field: Purpose::Reason, text: text(b"") } });
        let mut bindings = JigBindings::new(256, CLIENT.client.text);
        let tree = dialog(&domain, words, &mut bindings);
        if mask != 0 {
            goldens.check(&tree);
            assert!(drive(&mut domain, Event::Act { action: Action::Confirm }).is_empty());
        } else {
            drive(
                &mut domain,
                Event::Act { action: input(&tree, b"New specification", b"replacement", &mut bindings) },
            );
            let out = drive(&mut domain, Event::Act { action: Action::Confirm });
            assert!(out.iter().any(|request| match request {
                Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } =>
                    match &**ask {
                        Ask::Amend { change, .. } =>
                            change.spec.as_ref().is_some_and(|spec| spec.words.bytes() == b"replacement"
                                && spec.inputs == list([u64::MAX])
                                && spec.parameters == list([Parameter::Bytes { name: 8, value: text(UTF8_TEXTS[0]) }])),
                        _ => false,
                    },
                _ => false,
            }));
        }
    }
}

#[test]
#[expect(clippy::match_like_matches_macro, reason = "scenario matching stays explicit; matches is disallowed")]
fn note_documents_keep_each_text_loss_and_reference_loss_without_reconstructing_corrections() {
    let mut goldens = Goldens::new("tree/cards_objects/note_documents_keep");
    for mask in 0_u8..8 {
        for act in [ActKind::EditNote, ActKind::DeleteNote] {
            let note = Note {
                number: 77,
                revision: Revision { value: 123 },
                scope: NoteScope::Task { project: 1, task: 900 },
                description: DocumentText { bytes: text(b"note title"), cut: mask & 1 != 0 },
                body: DocumentText { bytes: text(b"note body"), cut: mask & 2 != 0 },
                writer: NoteAuthor::Party { party: 1 },
                tasks: list([91]),
                changed: Some(Wall::EPOCH),
                acts: offered(act),
                cut: mask & 4 != 0,
            };
            let (mut domain, id) = note_loaded(note);
            let words = words();
            let mut bindings = JigBindings::new(256, CLIENT.client.text);
            let mut builder = Builder::new(&limits());
            cards::card(&mut builder, &domain.view(), id, &cards::Place::Elsewhere, words, &mut bindings);
            let tree = builder.finish();
            goldens.check(&tree);
            drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act } });
            let tree = dialog(&domain, words, &mut bindings);
            let out = drive(&mut domain, Event::Act { action: Action::Confirm });
            if act == ActKind::EditNote && mask != 0 {
                assert!(tree.nodes().iter().all(|node| node.value.is_none()));
                assert!(out.iter().all(|request| match request {
                    Request::Client { asked: jig_client::Event::Send { .. } } => false,
                    _ => true,
                }));
            } else {
                assert!(out.iter().any(|request| match request {
                    Request::Client { asked: jig_client::Event::Send { .. } } => true,
                    _ => false,
                }));
            }
        }
    }
}

pub(crate) mod amendment_presence;

#[test]
#[expect(clippy::match_like_matches_macro, reason = "scenario matching stays explicit; matches is disallowed")]
fn a_cut_note_shows_its_reference_loss_and_blocks_correction_but_keeps_exact_deletion() {
    let mut goldens = Goldens::new("tree/cards_objects/a_cut_note");
    let words = words();
    let mut bindings = JigBindings::new(512, CLIENT.client.text);
    for act in [ActKind::EditNote, ActKind::DeleteNote] {
        let note = Note {
            number: 77,
            revision: Revision { value: 123 },
            scope: NoteScope::Task { project: 1, task: 900 },
            description: DocumentText::complete(text(b"note title")),
            body: DocumentText::complete(text(b"note body")),
            writer: NoteAuthor::Party { party: 1 },
            tasks: list([91, u64::MAX]),
            changed: Some(Wall::EPOCH),
            acts: offered(act),
            cut: true,
        };
        let (mut domain, id) = note_loaded(note);
        drive(&mut domain, Event::Act { action: Action::Intend { object: id.token(), act } });
        let tree = dialog(&domain, words, &mut bindings);
        let outgoing = drive(&mut domain, Event::Act { action: Action::Confirm });
        if act == ActKind::EditNote {
            goldens.check(&tree);
            assert!(tree.nodes().iter().all(|node| node.value.is_none()));
            assert!(domain.confirmation().unwrap().missing.is_some());
            assert!(outgoing.iter().all(|request| match request {
                Request::Client { asked: jig_client::Event::Send { .. } } => false,
                _ => true,
            }));
        } else {
            assert!(outgoing.iter().any(|request| match request {
                Request::Client { asked: jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. } } =>
                    match &**ask {
                        Ask::DeleteNote {
                            note: 77,
                            scope: NoteScope::Task { project: 1, task: 900 },
                            revision: Revision { value: 123 },
                            ..
                        } => true,
                        _ => false,
                    },
                _ => false,
            }));
        }
    }
}
pub(crate) mod instruction_facts;

#[test]
fn result_documents_keep_each_words_loss_fact_and_the_exact_change_kind_and_owner_part() {
    let mut goldens = Goldens::new("tree/cards_objects/result_documents_keep");
    let (domain, _) = start();
    let words = words();
    let part = Part::Result { connector: 2, task: 8 };
    let raw = OwnerPart {
        connector: u16::MAX,
        kind: OwnerPartKind::Resource,
        key: OwnerPartKey::Path(list([b"a/b".as_slice().into(), b"c\xff".as_slice().into()])),
    };
    for cut in [false, true] {
        let explanation = DocumentText { bytes: text(b"supplied words"), cut };
        let values = [
            ResultValue::Report(explanation.clone()),
            ResultValue::Words(explanation.clone()),
            ResultValue::Failure(explanation.clone()),
            ResultValue::Verdict {
                verdict: u32::MAX,
                words: explanation.clone(),
                fields: list([]),
                tasks: list([]),
                proposal: None,
            },
            ResultValue::Connector { summary: explanation.clone(), part: part.clone() },
            ResultValue::Choice {
                option: u32::MAX,
                by: crate::tests::tree::cards::party(),
                reason: explanation.clone(),
            },
            ResultValue::Approved { state: Box::new([0, 255, 1]), words: Some(explanation.clone()) },
            ResultValue::Change { kind: u16::MAX, part: raw.clone(), words: explanation },
        ];
        for value in values {
            let mut builder = Builder::new(&crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap());
            builder.open(Role::Page);
            let mut bindings = JigBindings::new(64, CLIENT.client.text);
            cards::result::value_nodes(&mut builder, &domain.view(), &value, words, &mut bindings);
            builder.close();
            let tree = builder.finish();
            goldens.check(&tree);
            if let ResultValue::Change { .. } = value {
                goldens.check(&tree);
                let slot = tree.nodes().iter().find_map(|node| node.slot).unwrap();
                assert_eq!(bindings.owner_part(slot.token), Some(&raw));
                assert_eq!(bindings.part(slot.token), None);
            }
        }
    }
}

#[test]
fn result_cards_distinguish_an_absent_completion_wall_from_epoch_and_render_supplied_metadata() {
    let mut goldens = Goldens::new("tree/cards_objects/result_cards_distinguish");
    let (domain, _) = start();
    let words = words();
    for at in [None, Some(Wall::EPOCH)] {
        let mut value = result();
        value.at = at;
        value.details = Some(Box::new(ResultDetails {
            contract: TaskContractDocument::Report { words: u32::MAX },
            requester: OwnerRequester::Deployment { project: u32::MAX },
            executor: OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX },
            chosen_by: None,
            delegates: list([
                OwnerTaskLink::Retained { chip: chip(), at: None },
                OwnerTaskLink::Stored { number: u64::MAX, status: None },
            ]),
            followups: Some(list([])),
            proposal: None,
            cut: false,
        }));
        let envelope = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
        let mut builder = Builder::new(&envelope);
        builder.open(Role::Page);
        let mut bindings = JigBindings::new(envelope.bindings, CLIENT.client.text);
        cards::result::facts(&mut builder, &domain.view(), &value, words, &mut bindings);
        builder.close();
        let tree = builder.finish();
        goldens.check(&tree);
    }
}

#[test]
fn exact_owner_verdict_renders_payload_and_stored_followups_without_inventing_chip_rows() {
    let mut goldens = Goldens::new("tree/cards_objects/exact_owner_verdict");
    let (domain, _) = start();
    let words = words();
    for cut in [false, true] {
        let mut value = result();
        value.value = Some(ResultValue::OwnerVerdict {
            verdict: u32::MAX,
            words: DocumentText { bytes: text(b"actual verdict words"), cut },
            fields: list([ResultField { name: Box::from(*b"opaque field"), value: Box::from(*b"supplied value") }]),
        });
        value.details = Some(Box::new(ResultDetails {
            contract: TaskContractDocument::Verdict { choices: list([]), cut: true },
            requester: OwnerRequester::Person(1),
            executor: OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
            chosen_by: None,
            delegates: list([]),
            followups: Some(list([OwnerTaskLink::Stored { number: u64::MAX, status: None }])),
            proposal: Some(ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX }),
            cut: true,
        }));
        let before = value.clone();
        let envelope = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
        let mut builder = Builder::new(&envelope);
        builder.open(Role::Page);
        let mut bindings = JigBindings::new(envelope.bindings, CLIENT.client.text);
        cards::result::facts(&mut builder, &domain.view(), &value, words, &mut bindings);
        builder.close();
        let tree = builder.finish();
        goldens.check(&tree);
        assert_eq!(value, before);
    }
}

pub(crate) mod note_facts;

pub(crate) mod task_links;

pub(crate) mod task_wake;

pub(crate) mod task_contract;

pub(crate) mod task_authority;

pub(crate) mod task_budget;

pub(crate) mod amendment_reason;
pub(crate) mod task_ending;

pub(crate) mod task_resources;

pub(crate) mod task_hold;

pub(crate) mod task_context;

pub(crate) mod owner_escalation;

#[test]
fn connector_effect_slots_keep_raw_numbers_separate_from_jig_proposal_makers() {
    let words = words();
    let connector = u16::MAX;
    for proposal in [0, u64::MAX] {
        let native = Part::OwnerEffect { connector, proposal };
        let mut bindings = JigBindings::new(8, CLIENT.client.text);
        let mut builder = Builder::new(&limits());
        builder.open(Role::Page);
        cards::common::part(&mut builder, &native, words, &mut bindings);
        builder.close();
        let tree = builder.finish();
        let slot = tree.nodes().iter().find_map(|node| node.slot).unwrap();
        assert_eq!(slot.owner, (crate::Owner::Application { number: connector }).opaque());
        assert_eq!(bindings.part(slot.token), Some(&native));
        for proposer in [Proposer::Task(proposal), Proposer::Party(proposal)] {
            let maker_effect = Part::Effect { connector, proposal: ProposalKey { proposer, number: proposal } };
            let mut builder = Builder::new(&limits());
            builder.open(Role::Page);
            cards::common::part(&mut builder, &maker_effect, words, &mut bindings);
            builder.close();
            let tree = builder.finish();
            let old = tree.nodes().iter().find_map(|node| node.slot).unwrap();
            assert_eq!(old.owner, slot.owner);
            assert_ne!(old.token, slot.token);
            assert_eq!(bindings.part(old.token), Some(&maker_effect));
            assert_eq!(bindings.part(slot.token), Some(&native));
        }
        bindings.begin();
        assert_eq!(bindings.part(slot.token), None);
    }
}

#[test]
fn moved_effect_facts_say_the_supplied_connector_proposal_without_inventing_a_maker() {
    let mut goldens = Goldens::new("tree/cards_objects/moved_effect_facts");
    let words = words();
    for proposal in [0, u64::MAX] {
        let mut builder = Builder::new(&limits());
        builder.open(Role::Page);
        words.refusal(
            &mut builder,
            &Refusal::Moved {
                now: Revision { value: u64::MAX },
                what: Moved::Hold {
                    reason: Some(CurrentHold::Document(Box::new(HoldReason::EffectFailed {
                        part: Part::OwnerEffect { connector: u16::MAX, proposal },
                    }))),
                },
            },
            CLIENT.client.text,
        );
        builder.close();
        let tree = builder.finish();

        goldens.check(&tree);
        assert!(tree.nodes().iter().all(|node| node.slot.is_none() && node.binding.is_none()));
    }
}

const UTF8_TEXTS: [&[u8]; 1] = [b"\0~\x01"];

pub(crate) mod effect_fixtures;
use effect_fixtures::offered_acts;

#[test]
fn jig_object_cards_hold_actions_confirmations_and_notices() {
    let words = words();
    result_value_payloads(words, &crate::tests::tree::work_pages::part());
    notice_scenarios(words);
    closing_connectors_use_their_owners_words(words);
    authority_refusal_uses_only_supplied_proposal_eligibility(words);
    collective_recipients_name_every_supplied_role_in_order(words);
}
