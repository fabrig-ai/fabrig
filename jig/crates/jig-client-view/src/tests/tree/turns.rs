#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary fixtures use public events and inspect semantic nodes"
)]
use crate::tests::fixtures::{AnsweredByEngine, Fixture, ReadPage};
use crate::tests::kit::Goldens;
use crate::tests::tree::{
    cards::{drive, text, words},
    pages::{build, deliver, named_binding},
    work_pages::{entered, list, part},
};
use crate::words::Words;
use crate::{
    Event, Request,
    state::{
        action::Act,
        address::{Address, TaskSection},
        fields::Purpose,
    },
};
use jig_client::contract::*;
use skein_lib::{Token, Wall};
use skein_ui::{Key, Role, Tree};
fn read(out: &[Request]) -> jig_client::Handle {
    out.iter()
        .find_map(|request| match request {
            Request::Client {
                asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::Activity { .. }) },
            } => Some(*read),
            _ => None,
        })
        .unwrap()
}
pub(in crate::tests) fn ready() -> (Fixture, jig_client::Handle, jig_client::Handle, jig_client::Handle) {
    ready_task(8)
}
fn ready_task(task: u64) -> (Fixture, jig_client::Handle, jig_client::Handle, jig_client::Handle) {
    let (mut d, out) = entered(Address::Task { task, section: TaskSection::Activity });
    let watch =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let mut doc = crate::tests::tree::cards_objects::task_doc(&[ActKind::Message]);
    doc.chip.task = task;
    doc.chip.executor = Executor::Agent { charter: 2 };
    doc.chip.phase = Phase::Running;
    *doc.chip.details = crate::tests::fixtures::chip_phase(OwnerChipPhase::Active(RunStage::Running { attempt: 2 }));
    doc.context.as_mut().expect("supplied task context").runtime =
        Runtime::Running { attempt: 2, since: Wall::EPOCH, tool: None };
    doc.hold = None;
    doc.activity = 2;
    let out = deliver(&mut d, watch, Delivery::Task(Box::new(doc)));
    let run = out
        .iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Watch { handle: watch, subject: jig_client::Watchable::Jig(Subject::Run { .. }) } } => Some(*watch),
            _ => None,
        })
        .unwrap();
    (d, watch, run, read(&out))
}
fn call() -> Call {
    Call {
        number: 1,
        tool: text(b"ReadFact"),
        input: text(b"whole private input"),
        outcome: Some(text(b"whole tool result")),
        failed: false,
        objects: list([ObjectKey::Task(8)]),
        parts: list([part()]),
    }
}
fn turn() -> Turn {
    Turn {
        messages: list([
            Message {
                speaker: Speaker::Assistant,
                blocks: list([Block::Text(text(b"# committed\n**reply**")), Block::Call(call())]),
            },
            Message {
                speaker: Speaker::Tool,
                blocks: list([
                    Block::Opaque { dialect: text(b"provider-fragment"), size: 91 },
                    Block::Result { call: 1, summary: text(b"result summary"), failed: true },
                ]),
            },
        ]),
        used: Usage { input: 2, output: 3, cached: 4, cache_written: 5 },
        spent: 7,
        read_through: Some(11),
        cut: true,
    }
}
fn live(d: &mut Fixture, run: jig_client::Handle, number: u32) {
    let value = text(if number == 1 { b"# provisional first" } else { b"later provisional" });
    let length = u32::try_from(value.bytes().len()).unwrap();
    deliver(
        d,
        run,
        crate::tests::fixtures::run(RunText {
            writing: None,
            attempt: 2,
            began: Some(Wall::EPOCH),
            turn: ProvisionalTurnName { retry: 0, attempt: 2, number },
            from: 0,
            through: length,
            text: value,
            cut: true,
            call: if number == 1 { Some(LiveCall::Decoded(call())) } else { None },
        }),
    );
}
fn keys(tree: &Tree) -> Vec<Key> {
    tree.nodes().iter().filter_map(|node| node.key).filter(|key| key.kind == 24).collect()
}
fn access(tree: &Tree) {
    assert!(
        tree.nodes().iter().filter(|n| n.binding.is_some()).all(|n| n.name.as_ref().is_some_and(|n| !n.is_empty()))
    );
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Heading { level: 1 }).count(), 1);
    assert!(tree.nodes().iter().any(|n| n.role == Role::Log && n.name.as_ref().is_some_and(|v| !v.is_empty())));
}

fn words_states(words: &Words) {
    let mut goldens = Goldens::new("tree/turns/words_states");
    let (mut d, _, _, read) = ready();
    drive(
        &mut d,
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                cut: false,
                rows: list([]),
                before: None,
                after: None,
            }))),
        },
    );
    drive(
        &mut d,
        Event::Act { action: Act::Edit { field: Purpose::Composer { task: 8 }, text: text(b"whole new words") } },
    );
    drive(&mut d, Event::Act { action: Act::Submit { field: Purpose::Composer { task: 8 } } });
    let out = drive(&mut d, Event::Act { action: Act::Confirm });
    let request = out
        .iter()
        .find_map(|r| match r {
            Request::Client { asked: jig_client::Event::Send { key: request, .. } } => Some(*request),
            _ => None,
        })
        .unwrap();
    goldens.check(&build(&d, 3, words).0);
    drive(
        &mut d,
        AnsweredByEngine { request, answer: Answer::Done(Outcome::Owner(OwnerOutcome::Said { task: 8, message: 12 })) },
    );
    goldens.check(&build(&d, 3, words).0);
}

#[test]
fn raw_live_call_rendering_omits_unsupplied_wall_and_preserves_escaped_literals_loss_and_folds() {
    let mut goldens = Goldens::new("tree/turns/raw_live_call_rendering");
    let words = words();
    let (mut d, _, run, _) = ready();
    let name = ProvisionalTurnName { retry: 0, attempt: u64::MAX, number: u32::MAX };
    let raw = CurrentCall {
        identity: current_identity(UTF8_TEXTS[0]),
        began: None,
        tool: text(UTF8_TEXTS[1]),
        input: text(UTF8_TEXTS[2]),
        cut: true,
    };
    let mut value = RunText {
        writing: None,
        attempt: name.attempt,
        began: None,
        turn: name,
        from: 0,
        through: 4,
        text: text(b"live"),
        cut: false,
        call: Some(LiveCall::Raw(raw.clone())),
    };
    deliver(&mut d, run, crate::tests::fixtures::run(value.clone()));
    assert_eq!(d.conversation().live.get(0).unwrap().began, None);
    assert_eq!(d.conversation().live.get(0).unwrap().call, value.call);
    let (folded, bindings) = build(&d, 3, words);
    access(&folded);
    goldens.check(&folded);
    let missing_nodes = folded.nodes().len();
    drop(bindings);
    value.began = Some(Wall::EPOCH);
    deliver(&mut d, run, crate::tests::fixtures::run(value.clone()));
    let supplied = build(&d, 3, words).0;
    assert_eq!(supplied.nodes().len(), missing_nodes + 2, "an actual epoch gets its own time and text nodes");
    assert_eq!(d.conversation().live.get(0).unwrap().began, Some(Wall::EPOCH));
    drive(&mut d, Event::Act { action: Act::FoldCurrentCall { identity: raw.identity.clone(), open: true } });
    let expanded = build(&d, 3, words).0;
    access(&expanded);
    goldens.check(&expanded);
    assert_eq!(d.conversation().parts(4).len(), 0);
    match value.call.as_mut().unwrap() {
        LiveCall::Raw(call) => call.cut = false,
        LiveCall::Decoded(_) => unreachable!(),
    }
    value.began = None;
    deliver(&mut d, run, crate::tests::fixtures::run(value));
    let uncut = build(&d, 3, words).0;
    assert_eq!(uncut.nodes().len() + 4, expanded.nodes().len(), "wall and raw-call loss are independent leaves");
    goldens.check(&uncut);
    let mut first_seen = RunText {
        writing: None,
        attempt: name.attempt,
        began: None,
        turn: name,
        from: 0,
        through: 4,
        text: text(b"live"),
        cut: false,
        call: Some(LiveCall::Raw(CurrentCall { began: Some(Wall::EPOCH), cut: false, ..raw })),
    };
    deliver(&mut d, run, crate::tests::fixtures::run(first_seen.clone()));
    let observed = build(&d, 3, words).0;
    assert_eq!(observed.nodes().len(), uncut.nodes().len() + 2);
    assert_eq!(d.conversation().live.get(0).unwrap().began, None);
    if let Some(LiveCall::Raw(call)) = &mut first_seen.call {
        call.began = None;
    }
    deliver(&mut d, run, crate::tests::fixtures::run(first_seen));
    assert_eq!(build(&d, 3, words).0.nodes().len(), uncut.nodes().len());
}

#[test]
fn maximum_raw_live_calls_fit_whole_page_and_escaped_phrase_capacities() {
    use crate::tests::tree::cards::CLIENT;
    let mut goldens = Goldens::new("tree/turns/maximum_raw_live");
    let words = words();
    let (mut d, _, run, _) = ready();
    let tool = full_raw_byte(UTF8_TEXTS[3][0]);
    let input = full_raw_byte(UTF8_TEXTS[4][0]);
    for number in 1..=1 {
        let name = ProvisionalTurnName { retry: 0, attempt: u64::MAX, number };
        let call = CurrentCall {
            identity: CallIdentity {
                sequence: number,
                id: full_raw_byte(UTF8_TEXTS[5][0]),
                ..current_identity(UTF8_TEXTS[0])
            },
            began: Some(Wall::EPOCH),
            tool: tool.clone(),
            input: input.clone(),
            cut: true,
        };
        deliver(
            &mut d,
            run,
            crate::tests::fixtures::run(RunText {
                writing: None,
                attempt: name.attempt,
                began: Some(Wall::from_nanos(u64::MAX)),
                turn: name,
                from: 0,
                through: CLIENT.client.text,
                text: full_raw_byte(b'x'),
                cut: true,
                call: Some(LiveCall::Raw(call.clone())),
            }),
        );
        drive(&mut d, Event::Act { action: Act::FoldCurrentCall { identity: call.identity.clone(), open: true } });
    }
    assert_eq!(d.conversation().live.len(), 1);
    assert_eq!(d.conversation().current_opened.len(), 1);
    let limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
    let mut b = skein_ui::Builder::new(&limits);
    let mut bindings = crate::JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        &d.view(),
        skein_ui::SlotRef { owner: (crate::Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    access(&tree);
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= limits.nodes);
    let mut escaped = Vec::new();
    for _ in 0..CLIENT.client.text {
        escaped.push(UTF8_TEXTS[4][0]);
    }
    goldens.check(&tree);
    match d.conversation().live.get(0).unwrap().call.as_ref().unwrap() {
        LiveCall::Raw(call) => {
            assert_eq!(call.identity.owner, Token::new(u64::MAX));
            assert_eq!(call.identity.id, full_raw_byte(UTF8_TEXTS[5][0]));
        }
        LiveCall::Decoded(_) => panic!("expected the supplied raw current call"),
    }
    assert!(limits.text_per_node >= CLIENT.client.text * 4);
    let (mut reloaded, _, run, _) = ready();
    assert!(reloaded.conversation().live.is_empty());
    deliver(
        &mut reloaded,
        run,
        crate::tests::fixtures::run(RunText {
            writing: None,
            attempt: u64::MAX,
            began: None,
            turn: ProvisionalTurnName { retry: 0, attempt: u64::MAX, number: 1 },
            from: 0,
            through: 0,
            text: text(b""),
            cut: false,
            call: Some(LiveCall::Raw(CurrentCall {
                identity: current_identity(UTF8_TEXTS[0]),
                began: None,
                tool,
                input,
                cut: false,
            })),
        }),
    );
    assert_eq!(reloaded.conversation().live.get(0).unwrap().began, None);
}

#[test]
fn raw_live_call_controls_use_selected_normal_language_and_checked_capacity() {
    use crate::tests::tree::cards::CLIENT;
    use crate::words::live_call_words::LiveCallPhrase;
    use crate::words::phrases::english;
    use skein_ui::phrases::{Form, PhraseLimits, Phrasebook, PluralClass, Segment};
    let mut goldens = Goldens::new("tree/turns/raw_live_call_controls");

    let first = english::language();
    let mut selected = english::language();
    for phrase in &mut selected.phrases {
        let label: Option<&[u8]> = if phrase.id == LiveCallPhrase::OpenCurrentCallInput.id() {
            Some(b"Selected language opens actual current input")
        } else if phrase.id == LiveCallPhrase::FoldCurrentCallInput.id() {
            Some(b"Selected language folds actual current input")
        } else {
            None
        };
        if let Some(label) = label {
            phrase.forms =
                Box::new([Form { class: PluralClass::Other, segments: Box::new([Segment::Text(label.into())]) }]);
        }
    }
    let book = Phrasebook::load(
        crate::words::phrases::jig::shapes(CLIENT.client.text),
        Box::new([first, selected]),
        &PhraseLimits {
            languages: 2,
            phrases: crate::words::owner_words::phrase_count(),
            holes: 4,
            segments: 8000,
            text_bytes: 100_000,
            hole_bytes: CLIENT.client.text,
        },
    )
    .unwrap();
    let words = Words::new(book, 1, 0);
    let (mut d, _, run, _) = ready();
    deliver(
        &mut d,
        run,
        crate::tests::fixtures::run(RunText {
            writing: None,
            attempt: 2,
            began: None,
            turn: ProvisionalTurnName { retry: 0, attempt: 2, number: 1 },
            from: 0,
            through: 0,
            text: text(b""),
            cut: false,
            call: Some(LiveCall::Raw(CurrentCall {
                identity: current_identity(UTF8_TEXTS[0]),
                began: None,
                tool: text(b"literal/tool"),
                input: text(b"input"),
                cut: false,
            })),
        }),
    );
    let (folded, mut bindings) = build(&d, 3, &words);
    goldens.check(&folded);
    let act = crate::decode(
        &mut bindings,
        named_binding(&folded, b"Selected language opens actual current input"),
        skein_ui::Act::Press,
    )
    .unwrap();
    drive(&mut d, Event::Act { action: act });
    let opened = build(&d, 3, &words).0;
    goldens.check(&opened);
    let limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, &words).unwrap();
    assert!(limits.text_per_node >= words.book().capacity(LiveCallPhrase::OpenCurrentCallInput.id()).unwrap());
}

fn full_raw_byte(byte: u8) -> Text {
    let mut bytes = Vec::new();
    bytes.resize(usize::try_from(crate::tests::tree::cards::CLIENT.client.text).unwrap(), byte);
    Text::new(bytes.into_boxed_slice(), crate::tests::tree::cards::CLIENT.client.text).unwrap()
}

fn current_identity(id: &[u8]) -> CallIdentity {
    CallIdentity { owner: Token::new(u64::MAX), sequence: u32::MAX, retry: u32::MAX, id: text(id) }
}

#[test]
fn whole_run_call_only_is_visible_once_without_a_turn_and_uses_normal_document_ack() {
    let mut goldens = Goldens::new("tree/turns/whole_run_call");
    let words = words();
    let (mut d, _, run, _) = ready();
    let raw = CurrentCall {
        identity: current_identity(UTF8_TEXTS[0]),
        began: None,
        tool: text(b"call-only-tool"),
        input: text(b"call-only-input"),
        cut: true,
    };
    let mut state = RunState { task: 8, attempt: Some(2), committed_turn: 0, writing: None, call: Some(raw.clone()) };
    let _out = deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    for down in &d.down {
        if let jig_client::Down::Continue { .. } = down {
            panic!("a current-call-only state needs no text continuation");
        }
    }
    assert!(d.conversation().live.is_empty());
    assert_eq!(d.first_turn(), None);
    assert_eq!(
        match d.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        0
    );
    assert_eq!(d.conversation().current_run, Some(&state));
    let folded = build(&d, 3, words).0;
    access(&folded);
    assert!(keys(&folded).is_empty(), "call-only has no invented provisional turn key");
    goldens.check(&folded);

    drive(&mut d, Event::Act { action: Act::FoldCurrentCall { identity: raw.identity.clone(), open: true } });
    let opened = build(&d, 3, words).0;
    goldens.check(&opened);
    let without_wall = opened.nodes().len();
    state.call.as_mut().unwrap().began = Some(Wall::EPOCH);
    deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    assert_eq!(build(&d, 3, words).0.nodes().len(), without_wall + 2);
    state.writing = Some(RunText {
        writing: None,
        attempt: 2,
        began: None,
        turn: ProvisionalTurnName { retry: 0, attempt: 2, number: 1 },
        from: 0,
        through: 4,
        text: text(b"live"),
        cut: false,
        call: Some(LiveCall::Raw(raw)),
    });
    deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    let writing = build(&d, 3, words).0;
    assert_eq!(keys(&writing).len(), 1);
    goldens.check(&writing);
    state.writing = None;
    state.committed_turn = 1;
    deliver(&mut d, run, Delivery::RunState(Box::new(state.clone())));
    assert_eq!(d.conversation().current_run, Some(&state));
    goldens.check(&build(&d, 3, words).0);
    state.call = None;
    deliver(&mut d, run, Delivery::RunState(Box::new(state)));
    let cleared = build(&d, 3, words).0;
    goldens.check(&cleared);
}

#[test]
fn maximum_independent_current_call_fits_page_with_all_existing_provisional_rows() {
    use crate::tests::tree::cards::CLIENT;
    let mut goldens = Goldens::new("tree/turns/maximum_independent_current");
    let words = words();
    let (mut d, _, run, _) = ready();
    for number in 1..=CLIENT.client.rows {
        live(&mut d, run, number);
    }
    let call = CurrentCall {
        identity: CallIdentity { id: full_raw_byte(UTF8_TEXTS[5][0]), ..current_identity(b"id") },
        began: Some(Wall::from_nanos(u64::MAX)),
        tool: full_raw_byte(UTF8_TEXTS[3][0]),
        input: full_raw_byte(UTF8_TEXTS[4][0]),
        cut: true,
    };
    let writing = d.client.text(run).cloned();
    deliver(
        &mut d,
        run,
        Delivery::RunState(Box::new(RunState {
            task: 8,
            attempt: Some(2),
            committed_turn: 0,
            writing,
            call: Some(call.clone()),
        })),
    );
    drive(&mut d, Event::Act { action: Act::FoldCurrentCall { identity: call.identity, open: true } });
    let limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, words).unwrap();
    let mut b = skein_ui::Builder::new(&limits);
    let mut bindings = crate::JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    assert!(crate::jig(
        &mut b,
        &d.view(),
        skein_ui::SlotRef { owner: (crate::Owner::Application { number: 1 }).opaque(), token: Token::new(1) },
        words,
        &mut bindings
    ));
    b.close();
    let tree = b.finish();
    access(&tree);
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= limits.nodes);
    let mut escaped = Vec::new();
    for _ in 0..CLIENT.client.text {
        escaped.push(UTF8_TEXTS[4][0]);
    }
    goldens.check(&tree);
    assert_eq!(d.conversation().live.len(), 2);
}

#[test]
fn displayed_retry_replaces_empty_then_resumes_with_exact_actual_watch_position() {
    let mut goldens = Goldens::new("tree/turns/displayed_retry_replaces");
    let words = words();
    let (mut d, _, run, _) = ready();
    live(&mut d, run, 1);
    drive(
        &mut d,
        Event::Act {
            action: Act::FoldCall { name: jig_client::thread::CallName { attempt: 2, turn: 1, call: 1 }, open: true },
        },
    );
    goldens.check(&build(&d, 3, words).0);
    let name = ProvisionalTurnName { attempt: 2, number: 1, retry: u32::MAX };
    let mut value = RunText {
        writing: None,
        attempt: 2,
        began: None,
        turn: name,
        from: 0,
        through: 0,
        text: text(b""),
        cut: false,
        call: None,
    };
    let _out = deliver(&mut d, run, crate::tests::fixtures::run(value.clone()));
    d.reopen(run);
    assert!(d.down.as_slice().contains(&jig_client::Down::Continue {
        watch: d.watch_token(run).unwrap(),
        subject: Subject::Run { task: 8, turn: Some(name), offset: 0 }
    }));
    assert_eq!(d.conversation().live.len(), 1);
    assert_eq!(d.conversation().live.get(0).unwrap().name, name);
    assert_eq!(d.conversation().live.get(0).unwrap().text.bytes(), b"");
    let empty = build(&d, 3, words).0;
    access(&empty);
    goldens.check(&empty);
    value.text = text(b"new retry bytes");
    value.through = 15;
    let _out = deliver(&mut d, run, crate::tests::fixtures::run(value.clone()));
    d.reopen(run);
    assert!(d.down.as_slice().contains(&jig_client::Down::Continue {
        watch: d.watch_token(run).unwrap(),
        subject: Subject::Run { task: 8, turn: Some(name), offset: 15 }
    }));
    goldens.check(&build(&d, 3, words).0);
    deliver(&mut d, run, crate::tests::fixtures::run(value));
    assert_eq!(d.conversation().live.get(0).unwrap().text.bytes(), b"new retry bytes");
    assert_eq!(
        match d.client.text(run) {
            Some(text) => text.through,
            None => 0,
        },
        15
    );
}

#[test]
fn whole_owner_activity_and_fetched_archive_keep_numeric_senders_references_and_separate_spend_facts_in_the_public_log()
{
    let mut goldens = Goldens::new("tree/turns/whole_owner_activity");
    let (mut domain, _, _, read) = ready();
    let words = words();
    let source = |position, line, decoded| ActivityLine {
        source: Box::new(OwnerActivityLine { task: 8, position, at: Wall::EPOCH, line }),
        decoded,
    };
    drive(
        &mut domain,
        ReadPage {
            read,
            result: ReadResult::Value(ReadValue::Activity(Box::new(ActivityPage {
                cut: false,
                rows: list([
                    source(
                        2,
                        OwnerActivity::Turn { reply: u64::MAX, attempt: 2, turn: 1, read: Some(0) },
                        Some(ArchivedTurn {
                            task: 8,
                            name: TurnName { attempt: 2, number: 1 },
                            cumulative: 19,
                            read: None,
                            at: Wall::from_nanos(42),
                            turn: TurnRead::Decoded(turn()),
                        }),
                    ),
                    source(
                        1,
                        OwnerActivity::Words {
                            message: 11,
                            by: OwnerRequester::Task(7),
                            kind: OwnerMessageKind::Proposal {
                                proposer: u64::MAX,
                                proposal: 0,
                                kind: OwnerMessageProposalKind::Amend,
                            },
                            words: text(b"raw owner words"),
                        },
                        None,
                    ),
                ]),
                before: None,
                after: None,
            }))),
        },
    );
    let (tree, _) = build(&domain, 3, words);
    goldens.check(&tree);
    let limits = crate::sizes::limits::of_words(
        &crate::tests::tree::cards::CLIENT,
        &(crate::tests::tree::cards::CLIENT).client,
        words,
    )
    .unwrap();
    assert!(tree.len() <= limits.nodes);
}

const UTF8_TEXTS: [&[u8]; 6] = [b"id/\x7e", b"raw/tool\x7e", b"a/b\x00\x21", b"~", b"!", b"?"];

#[test]
fn conversation_keeps_words_states() {
    words_states(words());
}
