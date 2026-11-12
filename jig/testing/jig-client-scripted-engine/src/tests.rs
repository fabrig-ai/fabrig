use super::*;
use crate::Bounds;
use crate::Decision as ApplicationDecision;
use fixtures::query;
use jig_client_domain::contract::Decision;
use jig_client_domain::contract::*;
use skein_lib::List;
mod frame_facts;
mod policy;
mod project;
mod settings_restore;

pub(crate) const REQUESTER: u64 = 1;
const DECIDER: u64 = 2;
const PROJECT: u32 = 1;
pub(crate) const REQUESTER_SIGN_IN: u64 = 11;
const REQUESTER_OTHER_SIGN_IN: u64 = 12;
const DECIDER_SIGN_IN: u64 = 22;

fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 4096).expect("fixture text fits")
}
fn list<T>(items: Vec<T>) -> List<T> {
    List::from_boxed(items.into_boxed_slice())
}
fn frame(number: u64) -> Me {
    Me {
        providers: skein_lib::List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        words_bytes: 8192,
        party: MeParty {
            number,
            kind: PartyKind::Person,
            login: DocumentText::complete(text(b"fixture-login")),
            name: DocumentText::complete(text(b"party")),
        },
        projects: list(vec![MeProject {
            left_out: 0,
            defaults: List::with_capacity(0),
            public: None,
            models: skein_lib::List::with_capacity(0),
            templates: skein_lib::List::with_capacity(0),
            creating: None,
            project: PROJECT,
            name: DocumentText::complete(text(b"project")),
            role: HeldRole::Policy { role: 1 },
            pool: Some(PoolLedger {
                source: Funder::Pool { project: PROJECT, person: number, period: 1 },
                parent: None,
                numbers: Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
                left: 100,
                closed: false,
                began: None,
                next: None,
            }),
        }]),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        results_seen: 0,
        cut: false,
        updates: Count { value: 0, limited: false },

        retention: Duration::from_secs(1),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    }
}
pub(crate) fn authority() -> AuthorityDocument {
    AuthorityDocument {
        authority: Authority {
            models: None,
            tools: 0,
            grants: List::with_capacity(0),
            delegation: Delegation { kinds: List::with_capacity(0), tasks: 10, depth: 10 },
            spend: 100,
            deadline: None,
            notes: 0,
            note_resources: List::with_capacity(0),
        },
        cut: false,
    }
}
fn script() -> Script {
    let mut truth = Truth::default();
    truth.frames.insert(REQUESTER, frame(REQUESTER));
    truth.frames.insert(DECIDER, frame(DECIDER));
    for frame in truth.frames.values_mut() {
        fixtures::creating(frame);
    }
    truth.sign_ins.insert(REQUESTER_SIGN_IN, REQUESTER);
    truth.sign_ins.insert(REQUESTER_OTHER_SIGN_IN, REQUESTER);
    truth.sign_ins.insert(DECIDER_SIGN_IN, DECIDER);
    Script { truth, ..Script::default() }
}
fn send(engine: &mut Engine, now: u64, sign_in: u64, key: u8, token: u64, ask: Ask) {
    engine
        .send(Time::from_nanos(now), Wall::from_nanos(now), sign_in, [key; 16], Token::new(token), ask)
        .expect("typed request fits");
}
fn final_answer(up: &Up) -> &Answer {
    match up {
        Up::Answer { envelope: Envelope::Final, answer: Some(answer), .. } => answer,
        Up::ApplicationAnswer { .. }
        | Up::ApplicationRead { .. }
        | Up::App { .. }
        | Up::Read { .. }
        | Up::Watch { .. }
        | Up::Ended { .. }
        | Up::Answer { .. } => {
            panic!("expected a final answer: {up:?}")
        }
    }
}
fn proposal() -> Proposal {
    Proposal {
        projects: List::with_capacity(0),
        key: ProposalKey { proposer: Proposer::Task(8), number: 1 },
        project: PROJECT,
        reason: DocumentText::complete(text(b"wider")),
        action: ProposalAction::Batch(list(vec![])),
        as_holder: false,
        waiting: ProposalWaiting { holder: HistoryHolder::Person(REQUESTER), since: Wall::EPOCH },
        funding: list(vec![]),
        acts: list(vec![Offered { effect: None, act: ActKind::Accept, admission: Admission::Allowed }]),
    }
}
fn proposal_engine() -> (Engine, Ask) {
    let mut setup = script();
    let p = proposal();
    setup.truth.objects.insert(ObjectKey::Proposal(p.key), ReadValue::Proposal(Box::new(p.clone())));
    let ask = Ask::Decide {
        project: PROJECT,
        on: DecideOn::Proposal { proposal: p.key, decision: Decision::Accepted, running: None },
    };
    (Engine::new(setup, 1).expect("valid script"), ask)
}
#[test]
fn answer_waits_for_its_durable_commit() {
    let (mut engine, ask) = proposal_engine();
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask);
    assert_eq!(engine.next_due(), Some(at(10_000_000)));
    assert!(engine.poll(at(9_999_999)).is_empty());
    assert!(engine.commits().is_empty());
    assert_eq!(engine.poll(at(10_000_000)).len(), 1);
    assert_eq!(engine.commits()[0].durable, Some(at(10_000_000)));
}
#[test]
fn retained_key_replays_across_sign_ins() {
    for count in [None, Some(0), Some(u32::MAX)] {
        let (mut engine, ask) = if let Some(count) = count {
            let ask = Ask::ReadLists { provider: 3 };
            let mut setup = script();
            setup.rules.push(Rule {
                party: Some(REQUESTER),
                ask: ask.clone(),
                answer: Answer::Done(Outcome::Account(AccountAnswer::Reading { count })),
                changes: vec![],
            });
            (Engine::new(setup, 301).expect("valid script"), ask)
        } else {
            proposal_engine()
        };
        send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask.clone());
        send(&mut engine, 1, REQUESTER_OTHER_SIGN_IN, 1, 2, ask.clone());
        let up = engine.poll(at(10_000_001));
        assert_eq!(up.len(), 2);
        if let Some(count) = count {
            assert_eq!(final_answer(&up[0]), &Answer::Done(Outcome::Account(AccountAnswer::Reading { count })));
        }
        assert_eq!(final_answer(&up[0]), final_answer(&up[1]));
        send(&mut engine, 20_000_000, REQUESTER_SIGN_IN, 1, 3, ask);
        assert_eq!(final_answer(&engine.poll(at(20_000_000))[0]), final_answer(&up[0]));
        assert_eq!(engine.commits().len(), 1);
    }
}

#[test]
fn retained_key_refuses_a_different_ask() {
    let (mut engine, ask) = proposal_engine();
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask);
    drop(engine.poll(at(10_000_000)));
    send(
        &mut engine,
        20_000_000,
        REQUESTER_SIGN_IN,
        1,
        2,
        Ask::Decide {
            project: PROJECT,
            on: DecideOn::Proposal { proposal: proposal().key, decision: Decision::PassedUp, running: None },
        },
    );
    assert_eq!(
        final_answer(&engine.poll(at(20_000_000))[0]),
        &Answer::Refused { refusal: Refusal::KeyConflict, at: None }
    );
    assert_eq!(engine.commits().len(), 1);
}
#[test]
fn second_party_receives_the_first_proposal_decision() {
    let (mut engine, ask) = proposal_engine();
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask.clone());
    send(&mut engine, 2, DECIDER_SIGN_IN, 2, 2, ask);
    let up = engine.poll(at(10_000_002));
    assert_eq!(
        final_answer(&up[1]),
        &Answer::Done(Outcome::Document(DocumentOutcome::ProposalDecided {
            proposal: proposal().key,
            by: REQUESTER,
            choice: ProposalChoice::Accepted,
            at: Wall::EPOCH,
            repeated: true,
        }))
    );
}
#[test]
fn expired_key_can_make_another_decision() {
    let mut setup = script();
    let ask = Ask::MakeService { project: PROJECT, name: text(b"service"), role: HeldRole::Owner };
    setup.rules.push(Rule {
        party: None,
        ask: ask.clone(),
        answer: Answer::Refused {
            refusal: Refusal::Authority { lacking: Lacking::Budget { needed: 100, left: 5 }, proposable: None },
            at: None,
        },
        changes: vec![],
    });
    let mut engine = Engine::new(setup, 3).expect("valid script");
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 1, ask.clone());
    assert_eq!(
        final_answer(&engine.poll(at(10_000_000))[0]),
        &Answer::Refused {
            refusal: Refusal::Authority { lacking: Lacking::Budget { needed: 100, left: 5 }, proposable: None },
            at: None,
        }
    );
    send(&mut engine, 2_000_000_000, REQUESTER_SIGN_IN, 1, 2, ask);
    drop(engine.poll(at(2_010_000_000)));
    assert_eq!(engine.commits().len(), 2);
}
fn counts() -> ImpactCounts {
    ImpactCounts { ended: 0, waiting: 0, running: 0, held: 0, closing: 0, done: 0, failed: 0, cancelled: 0 }
}
fn task() -> Task {
    Task {
        talk: None,
        session: None,
        dependency_facts: skein_lib::List::with_capacity(0),
        kept_account: None,
        run_wait: None,
        settings: Some(TaskSettings { task: 1, run: None, newest: 1 }),
        llm: None,
        label: 0,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        last_activity: Wall::EPOCH,
        made: Wall::EPOCH,
        waits_on: WaitsOn::Nothing,
        chip: Chip {
            task: 8,
            title: DocumentText::complete(text(b"chat")),
            executor: ChipExecutor::Agent { charter: 1 },
            phase: Phase::Running,
            details: Box::new(chip_details(ChipPhase::Active(RunStage::Running { attempt: 1 }))),
        },
        goal: None,
        priority: None,
        lineage_chips: list(vec![]),
        inputs: list(vec![]),
        project: PROJECT,
        requester: Requester::Person(REQUESTER),
        lineage: list(vec![]),
        spec: specification(text(b"spec")),
        spec_revision: Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: Some(DocumentText::complete(text(b"instructions"))),
        wake: WakePolicy {
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
        dependencies: list(vec![]),
        cut: false,
        references: list(vec![]),
        resources: list(vec![]),
        budget: Budget {
            numbers: Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            left: 100,
            funder: Funder::Period { project: PROJECT, period: 3 },
            deadline: None,
        },
        authority: authority(),
        hold: None,
        proposals: list(vec![]),
        activity: 0,
        changed: Wall::EPOCH,
        acts: list(vec![]),
        result: None,
        delegates: list(vec![]),

        context: Some(Box::new(jig_client_domain::contract::TaskContext {
            parts: list(vec![]),
            closing: None,
            spend: list(vec![]),
            runtime: Runtime::Running { attempt: 1, since: Wall::EPOCH, tool: None },
            impact: Impact { tasks: counts(), live_runs: 1, proposals: 0 },
            writer_slots: list(vec![]),
        })),
    }
}
fn live_script() -> Script {
    let mut setup = script();
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(task())));
    setup
        .truth
        .trees
        .insert(8, Tree { root: task().chip, rows: list(vec![]), folded: list(vec![]), held_below: 0, next: None });
    setup
}
fn at(nanos: u64) -> Time {
    Time::from_nanos(nanos)
}
fn scenario_watch_subjects_coalescing_and_lifecycle() {
    let mut setup = live_script();
    setup.truth.deployment = Some(deployment::empty());
    let mut engine = Engine::new(setup, 5).expect("live truth");
    let subjects = vec![
        Subject::Me,
        Subject::Task { task: 8 },
        Subject::Tree { task: 8, window: 1 },
        Subject::Chips { tasks: list(vec![8]) },
        Subject::Inbox { window: 1 },
        Subject::Tasks { query: query(PROJECT), window: 1 },
        Subject::Run { task: 8, turn: None, offset: 0 },
        Subject::Deployment,
    ];
    for (index, subject) in subjects.into_iter().enumerate() {
        engine.open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(index as u64), subject).expect("subject");
    }
    let up = engine.poll(Time::ZERO);
    assert_eq!(up.iter().filter(|event| matches!(event, Up::Watch { event: WatchEvent::Opened, .. })).count(), 8);
    assert_eq!(
        up.iter().filter(|event| matches!(event, Up::Watch { event: WatchEvent::Delivered(_), .. })).count(),
        7,
        "idle run has no fabricated turn"
    );
    let mut changed = task();
    changed.chip.title = DocumentText::complete(text(b"one"));
    engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Object {
                project: PROJECT,
                key: ObjectKey::Task(8),
                value: ReadValue::Task(Box::new(changed.clone())),
            },
        )
        .expect("one");
    changed.chip.title = DocumentText::complete(text(b"two"));
    engine
        .act(
            at(1),
            Wall::EPOCH,
            Change::Object {
                project: PROJECT,
                key: ObjectKey::Task(8),
                value: ReadValue::Task(Box::new(changed.clone())),
            },
        )
        .expect("two");
    assert!(engine.poll(at(10_000_001)).is_empty(), "in-flight coalesces");
    engine.taken(Token::new(1));
    let up = engine.poll(at(10_000_001));
    assert_eq!(
        up,
        vec![Up::Watch { token: Token::new(1), event: WatchEvent::Delivered(Delivery::Task(Box::new(changed))) }]
    );
    engine.taken(Token::new(1));
    assert!(engine.poll(at(10_000_001)).is_empty());
    let up = engine.poll(at(1_010_000_001));
    assert_eq!(up.iter().filter(|event| matches!(event, Up::Watch { event: WatchEvent::Alive, .. })).count(), 8);
    engine.close(at(1_010_000_001), Token::new(1));
    engine.close(at(1_010_000_001), Token::new(1));
    assert_eq!(engine.poll(at(1_010_000_001)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::Closed }]);
    let mut changed = task();
    changed.chip.phase = Phase::Done;
    *changed.chip.details = chip_details(ChipPhase::Ended);
    changed.context.as_mut().expect("supplied task context").runtime = Runtime::Ended;
    engine
        .act(
            at(1_010_000_001),
            Wall::EPOCH,
            Change::Object { project: PROJECT, key: ObjectKey::Task(8), value: ReadValue::Task(Box::new(changed)) },
        )
        .expect("ended");
    let up = engine.poll(at(1_020_000_001));
    for id in [2, 3, 6] {
        assert!(up.contains(&Up::Ended { token: Token::new(id), why: WatchEnd::SubjectEnded }));
    }
    let mut setup = live_script();
    setup.bounds.watches = 1;
    let mut engine = Engine::new(setup, 1).expect("limited engine");
    engine.open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(1), Subject::Me).expect("first");
    engine.open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(2), Subject::Me).expect("over limit");
    assert!(engine.poll(Time::ZERO).contains(&Up::Ended { token: Token::new(2), why: WatchEnd::Busy }));
    engine.act(Time::ZERO, Wall::EPOCH, Change::SignedOut { sign_in: REQUESTER_SIGN_IN }).expect("signed out");
    assert_eq!(engine.poll(at(10_000_000)), vec![Up::Ended { token: Token::new(1), why: WatchEnd::SignedOut }]);
}
fn scenario_run_offsets_cut_names_and_activity() {
    let mut setup = live_script();
    setup.bounds.text = 32;
    let mut engine = Engine::new(setup, 6).expect("run bound");
    let name = TurnName { attempt: 1, number: 0 };
    engine
        .open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(1), Subject::Run { task: 8, turn: None, offset: 0 })
        .expect("run watch");
    drop(engine.poll(Time::ZERO));
    engine.write(Time::ZERO, 8, name, Wall::EPOCH, &text(b"abcdefghijklmnopqrst"), None).expect("first chunk");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[0] else {
        panic!("run text");
    };
    let run = state.writing.as_ref().expect("provisional text");
    assert_eq!((run.from, run.through, run.cut), (0, 20, false));
    assert_eq!(run.text.bytes(), b"abcdefghijklmnopqrst");
    engine.write(at(1), 8, name, Wall::EPOCH, &text(b"uvwxyz0123456789ABCD"), None).expect("second chunk");
    assert!(engine.poll(at(1)).is_empty());
    engine.continue_run(Token::new(1), Some(provisional(name)), 20).expect("same watch cursor");
    let up = engine.poll(at(1));
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[0] else {
        panic!("run text");
    };
    let run = state.writing.as_ref().expect("provisional text");
    assert_eq!((run.from, run.through, run.cut), (20, 40, false), "a valid acknowledged suffix needs no replacement");
    assert_eq!(run.text.bytes(), b"uvwxyz0123456789ABCD");
    engine.continue_run(Token::new(1), Some(provisional(name)), 0).expect("lagging offset");
    let up = engine.poll(at(1));
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[0] else {
        panic!("cut text");
    };
    let run = state.writing.as_ref().expect("provisional text");
    assert_eq!((run.from, run.through), (8, 40));
    assert!(run.cut, "the actual lagging origin still proves retained-prefix loss");
    let second = TurnName { attempt: 1, number: 1 };
    engine.write(at(2), 8, second, Wall::EPOCH, &text(b"second"), None).expect("new turn");
    engine.continue_run(Token::new(1), Some(provisional(name)), 40).expect("old turn position");
    let up = engine.poll(at(2));
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[0] else {
        panic!("new turn text");
    };
    let run = state.writing.as_ref().expect("provisional text");
    assert_eq!((run.turn, run.from, run.through), (provisional(second), 0, 6));
    engine
        .act(
            at(2),
            Wall::EPOCH,
            Change::Source(Source {
                party: REQUESTER,
                read: Read::Activity { task: 8, before: None },
                rows: Rows::Activity(vec![turn_event(
                    1,
                    second,
                    TurnRead::Decoded(Turn {
                        messages: list(vec![]),
                        used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                        spent: 1,
                        read_through: None,
                        llm: None,
                        cut: false,
                    }),
                )]),
            }),
        )
        .expect("turn committed through activity");
    assert_eq!(
        engine.truth().objects.get(&ObjectKey::Task(8)).and_then(|value| if let ReadValue::Task(task) = value {
            Some(task.activity)
        } else {
            None
        }),
        Some(1)
    );
    engine.read(at(2), REQUESTER_SIGN_IN, Token::new(2), Read::Activity { task: 8, before: None }).expect("activity");
    assert!(engine.poll(at(10_000_001)).is_empty());
    let up = engine.poll(at(10_000_002));
    assert!(
        matches!(&up[0],Up::Read { terminal:ReadResult::Value(ReadValue::Activity(page)),.. } if page.rows.iter().next().expect("turn").committed_turn() == Some(second))
    );
    assert!(!engine.runs.contains_key(&8), "durable matching activity replaces the provisional turn");
}

fn scenario_all_read_shapes_and_reverse_activity() {
    let mut setup = live_script();
    setup.bounds.rows = 1;
    setup.truth.sources.push(Source {
        party: REQUESTER,
        read: Read::Activity { task: 8, before: None },
        rows: Rows::Activity(
            (0..3)
                .map(|position| DecodedActivityLine {
                    source: Box::new(ActivityLine {
                        task: 8,
                        position,
                        at: Wall::EPOCH,
                        line: Activity::RunBegan { attempt: 1, why: RunWhy::Ready, resumed: true },
                    }),
                    decoded: None,
                })
                .collect(),
        ),
    });
    let mut engine = Engine::new(setup, 7).expect("all reads");
    let mut cursor = None;
    for position in (0..3).rev() {
        engine
            .read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(10 + position), Read::Activity { task: 8, before: cursor })
            .expect("reverse page");
        let up = engine.poll(Time::ZERO);
        let Up::Read { terminal: ReadResult::Value(ReadValue::Activity(page)), .. } = &up[0] else {
            panic!("activity");
        };
        assert_eq!(page.rows.iter().next().expect("row").source.position, position);
        assert_eq!(page.before.is_some(), position != 0);
        cursor = page.before.clone();
    }
    let reads = vec![
        Read::Me,
        Read::Task { task: 8 },
        Read::Proposal { proposal: ProposalKey { proposer: Proposer::Task(8), number: 1 } },
        Read::Escalation { task: 8 },
        Read::Question { task: 8, message: 1 },
        Read::PersonTask { task: 8 },
        Read::Result { task: 8 },
        Read::Ended { task: 8, after: None },
        Read::History { task: 8, subtree: true, after: None },
        Read::Inbox {
            group: jig_client_domain::contract::InboxReadGroup::Updates,
            kind: Some(InboxGroup::Replies),
            goal: None,
            project: None,
            after: None,
        },
        Read::Inbox {
            group: jig_client_domain::contract::InboxReadGroup::Needs,
            project: Some(PROJECT),
            kind: Some(InboxGroup::Questions),
            goal: None,
            after: None,
        },
        Read::Spend { project: PROJECT, after: None },
        Read::Project { project: PROJECT },
        Read::Notes { scope: NoteScope::Project(PROJECT), description: Some(text(b"match")), after: None },
        Read::Deployment,
    ];
    for (index, read) in reads.into_iter().enumerate() {
        let token = Token::new(100 + index as u64);
        engine.read(Time::ZERO, REQUESTER_SIGN_IN, token, read).expect("contract read shape");
        let up = engine.poll(Time::ZERO);
        assert_eq!(up.len(), 1);
        assert!(matches!(up[0],Up::Read { token:id,.. } if id == token));
        assert!(engine.poll(Time::ZERO).is_empty());
    }
    scenario_reply_identity();
}
fn scenario_reply_identity() {
    let mut setup = live_script();
    let line = {
        let mut value = task();
        value.chip.title = DocumentText::complete(text(b"whole reply"));
        value.label = 1;
        value.flags = TaskFlags {
            ended_by: EndedBy::Result,
            conversation: true,
            listed: false,
            retention: Retention::ForgottenOnceRead,
        };
        crate::queries::line(
            &value,
            core::iter::empty(),
            Some(ConversationRead { seen: 0, unread: Count { value: 1, limited: false } }),
        )
    };
    let entry = inbox_entry(ObjectKey::Task(9), PROJECT, InboxGroup::Replies, 1, Wall::EPOCH, Some(Box::new(line)));
    setup.truth.sources.push(Source {
        party: REQUESTER,
        read: Read::Inbox {
            group: jig_client_domain::contract::InboxReadGroup::Updates,
            project: None,
            kind: None,
            goal: None,
            after: None,
        },
        rows: Rows::Inbox(vec![entry]),
    });
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Identity), "reply task identity is whole and authoritative");
}
#[test]
fn reads_watches_and_run_scenarios() {
    scenario_all_read_shapes_and_reverse_activity();
    scenario_watch_subjects_coalescing_and_lifecycle();
    scenario_run_offsets_cut_names_and_activity();
}

mod application;
mod faults;
#[test]
fn restart_faults_and_application_hooks_keep_one_durable_truth() {
    faults::run();
    application::run();
}

#[test]
fn amendment_release_checks_the_named_hold_separately_from_the_spec() {
    for stale_hold in [false, true] {
        let mut setup = script();
        let mut row = task();
        row.hold = Some(Box::new(hold(HoldCategory::Stopped, 7)));
        row.acts = list(vec![Offered { effect: None, act: ActKind::Amend, admission: Admission::Allowed }]);
        setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
        let mut engine = Engine::new(setup, 1).expect("valid script");
        send(
            &mut engine,
            0,
            REQUESTER_SIGN_IN,
            1,
            1,
            crate::tests::releasing(
                Ask::Amend {
                    reason: text(b""),
                    project: PROJECT,
                    task: 8,
                    revision: Some(Revision { value: if stale_hold { 1 } else { 2 } }),

                    change: Box::new(Amendment {
                        llm: None,
                        addressed: None,
                        spec: Some(specification(text(b"new spec")).whole().expect("complete fixture specification")),
                        wake: None,
                        dependencies: None,
                        authority: None,
                        instructions: None,
                        tracked: None,
                    }),
                },
                Revision { value: if stale_hold { 6 } else { 7 } },
            ),
        );
        assert_eq!(
            final_answer(&engine.poll(Time::from_nanos(10_000_000))[0]),
            &Answer::Refused {
                refusal: Refusal::Moved {
                    now: Revision { value: if stale_hold { 7 } else { 1 } },
                    what: if stale_hold {
                        Moved::Hold { reason: Some(CurrentHold::Reason(HoldCategory::Stopped)) }
                    } else {
                        Moved::Spec
                    },
                },
                at: None
            }
        );
    }
}

fn specification(words: jig_client_domain::contract::Text) -> jig_client_domain::contract::SpecificationDocument {
    jig_client_domain::contract::SpecificationDocument::complete(jig_client_domain::contract::Specification {
        asks: false,
        words,
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    })
}

fn cursor_inbox_entry(number: u64, kind: InboxGroup, at: u64, _goal: Option<u64>) -> InboxEntry {
    inbox_entry(
        match kind {
            InboxGroup::Questions => ObjectKey::Question { task: number, message: 1 },
            InboxGroup::Results => ObjectKey::Result(number),
            InboxGroup::PersonTasks => ObjectKey::PersonTask(number),
            InboxGroup::Proposals | InboxGroup::Escalations | InboxGroup::Replies => unreachable!("fixture kinds"),
        },
        PROJECT,
        kind,
        number,
        Wall::from_nanos(at),
        None,
    )
}

#[test]
fn inbox_group_cursors_keep_exact_filters_and_whole_query_order() {
    let mut setup = live_script();
    setup.bounds.rows = 1;
    for (group, rows) in [
        (
            InboxReadGroup::Needs,
            vec![
                cursor_inbox_entry(5, InboxGroup::Questions, 0, Some(8)),
                cursor_inbox_entry(6, InboxGroup::PersonTasks, 1, Some(9)),
                cursor_inbox_entry(7, InboxGroup::Questions, 2, Some(9)),
                cursor_inbox_entry(8, InboxGroup::Questions, 3, Some(9)),
            ],
        ),
        (
            InboxReadGroup::Updates,
            vec![
                cursor_inbox_entry(11, InboxGroup::Results, 5, Some(9)),
                cursor_inbox_entry(10, InboxGroup::Results, 4, Some(9)),
            ],
        ),
    ] {
        setup.truth.sources.push(Source {
            party: REQUESTER,
            read: Read::Inbox { group, project: None, kind: None, goal: None, after: None },
            rows: Rows::Inbox(rows),
        });
    }
    inbox_selections(&mut setup.truth);
    let mut engine = Engine::new(setup, 92).expect("whole owner collections fit");
    let needs = Read::Inbox {
        group: InboxReadGroup::Needs,
        project: Some(PROJECT),
        kind: Some(InboxGroup::Questions),
        goal: Some(9),
        after: None,
    };
    let updates = Read::Inbox {
        group: InboxReadGroup::Updates,
        project: Some(PROJECT),
        kind: Some(InboxGroup::Results),
        goal: Some(9),
        after: None,
    };
    engine.read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(1), needs.clone()).expect("typed read fits");
    let up = engine.poll(Time::ZERO);
    let Up::Read { terminal: ReadResult::Value(ReadValue::Inbox(page)), .. } = &up[0] else {
        panic!("needs page");
    };
    assert_eq!(
        page.rows.get(0).expect("needs page row").card_key(),
        ObjectKey::Question { task: 7, message: 1 },
        "filter before bounded page slicing"
    );
    let needs_cursor = page.after.clone().expect("needs page continuation");
    engine.read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(2), updates.clone()).expect("typed read fits");
    let up = engine.poll(Time::ZERO);
    let Up::Read { terminal: ReadResult::Value(ReadValue::Inbox(page)), .. } = &up[0] else {
        panic!("updates page");
    };
    assert_eq!(page.rows.get(0).expect("updates page row").card_key(), ObjectKey::Result(11));
    let updates_cursor = page.after.clone().expect("updates page continuation");
    for (token, mut query, cursor, wanted) in [
        (3, needs.clone(), needs_cursor.clone(), ObjectKey::Question { task: 8, message: 1 }),
        (4, updates.clone(), updates_cursor, ObjectKey::Result(10)),
    ] {
        if let Read::Inbox { after, .. } = &mut query {
            *after = Some(cursor);
        }
        engine.read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(token), query).expect("typed read fits");
        let up = engine.poll(Time::ZERO);
        let Up::Read { terminal: ReadResult::Value(ReadValue::Inbox(page)), .. } = &up[0] else {
            panic!("independent continuation");
        };
        assert_eq!(page.rows.get(0).expect("continuation page row").card_key(), wanted);
        assert!(page.after.is_none());
    }
    let mut wrong = updates;
    if let Read::Inbox { after, .. } = &mut wrong {
        *after = Some(needs_cursor);
    }
    engine.read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(5), wrong).expect("typed read fits");
    assert!(matches!(engine.poll(Time::ZERO)[0], Up::Read { terminal: ReadResult::None(ReadNone::Gone), .. }));
    engine.open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(6), Subject::Inbox { window: 1 }).expect("typed watch fits");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::Inbox(value)), .. } = &up[1] else {
        panic!("shared live watch");
    };
    assert_eq!(
        value.needs.get(0).expect("live needs row").card_key(),
        ObjectKey::Question { task: 5, message: 1 },
        "live watch remains unfiltered"
    );
    assert!(value.next.is_some());
}

#[test]
fn activation_observations_and_prior_closing_are_admitted_whole_with_their_allocated_bounds() {
    let mut setup = script();
    let mut row = task();
    let mut active = ActiveRun {
        stage: RunStage::Running { attempt: u64::MAX },
        phase_since: Wall::EPOCH,
        attempt: Some(u64::MAX),
        placement: Some(Wall::EPOCH),
        retry: Some(RetryNotice::Refused { count: u32::MAX, reason: None }),
        observation: Some(RunObservation {
            channel: Token::new(u64::MAX),
            work: RunWork::Tool { call: current_identity(b"call"), tool: text(b"tool") },
            since: Wall::EPOCH,
            reported_at: Wall::EPOCH,
            started: None,
        }),
    };
    *row.chip.details = chip_details(ChipPhase::Active(active.stage));
    row.context.as_mut().expect("supplied task context").runtime = Runtime::Active(active.clone());
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
    assert!(Engine::new(setup.clone(), 1).is_ok());
    active.observation.as_mut().expect("fixture run observation").work = RunWork::Tool {
        call: CallIdentity {
            id: Text::new(vec![b'x'; setup.bounds.text as usize + 1].into_boxed_slice(), setup.bounds.text + 1)
                .expect("producer call identity fits"),
            ..current_identity(b"call")
        },
        tool: text(b"tool"),
    };
    row.context.as_mut().expect("supplied task context").runtime = Runtime::Active(active);
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row.clone())));
    assert_eq!(Engine::new(setup.clone(), 1).err(), Some(Invalid::Payload));
    row.context.as_mut().expect("supplied task context").runtime = Runtime::Held {
        was: Some(HeldRuntime::Closing(Closing {
            delegates: 0,
            connectors: Some(list(vec![
                1;
                usize::try_from(setup.bounds.rows + 1).expect("fixture row count fits usize")
            ])),
            stage: Some(ClosingStage::Run { attempt: u64::MAX }),
            ending: None,
        })),
    };
    setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
    assert_eq!(Engine::new(setup, 1).err(), Some(Invalid::Payload));
}

#[test]
fn full_width_task_and_party_makers_remain_distinct_in_scripted_reads_decisions_and_keyed_replays() {
    let mut setup = script();
    let keys = [
        ProposalKey { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
        ProposalKey { proposer: Proposer::Party(u64::MAX), number: u64::MAX },
    ];
    for (index, key) in keys.iter().enumerate() {
        let mut row = proposal();
        row.key = *key;
        row.action = ProposalAction::Settings(Box::new(ProposalActionSettings {
            against: 1,
            items: list(vec![]),
            review: None,
            made: None,
        }));
        row.reason = DocumentText::complete(text(if index == 0 { b"task maker" } else { b"party maker" }));
        setup.truth.objects.insert(ObjectKey::Proposal(*key), ReadValue::Proposal(Box::new(row)));
    }
    let mut engine = Engine::new(setup, 197).expect("valid script");
    for (index, key) in keys.iter().enumerate() {
        engine
            .read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(index as u64 + 1), Read::Proposal { proposal: *key })
            .expect("typed read fits");
        let rows = engine.poll(Time::ZERO);
        let Up::Read { terminal: ReadResult::Value(ReadValue::Proposal(row)), .. } = &rows[0] else {
            panic!("exact typed read")
        };
        assert_eq!(row.key, *key);
        assert_eq!(row.reason.bytes(), if index == 0 { b"task maker".as_slice() } else { b"party maker".as_slice() });
    }
    let asks = keys.map(|proposal| Ask::Decide {
        project: PROJECT,
        on: DecideOn::Proposal { proposal, decision: Decision::Accepted, running: None },
    });
    send(&mut engine, 0, REQUESTER_SIGN_IN, 1, 10, asks[0].clone());
    send(&mut engine, 1, REQUESTER_SIGN_IN, 2, 11, asks[1].clone());
    let answers = engine.poll(Time::from_nanos(10_000_001));
    assert_eq!(answers.len(), 2);
    assert_eq!(engine.commits().len(), 2);
    assert_eq!(engine.commits()[0].ask, Some(asks[0].clone()));
    assert_eq!(engine.commits()[1].ask, Some(asks[1].clone()));
    send(&mut engine, 20_000_000, REQUESTER_SIGN_IN, 1, 12, asks[1].clone());
    assert_eq!(
        final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]),
        &Answer::Refused { refusal: Refusal::KeyConflict, at: None }
    );
    send(&mut engine, 20_000_000, REQUESTER_SIGN_IN, 1, 13, asks[0].clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&answers[0]));
    send(&mut engine, 20_000_000, REQUESTER_SIGN_IN, 2, 14, asks[1].clone());
    assert_eq!(final_answer(&engine.poll(Time::from_nanos(20_000_000))[0]), final_answer(&answers[1]));
    assert_eq!(engine.commits().len(), 2);
}

fn current_identity(id: &[u8]) -> jig_client_domain::contract::CallIdentity {
    jig_client_domain::contract::CallIdentity {
        owner: skein_lib::Token::new(u64::MAX),
        sequence: u32::MAX,
        retry: u32::MAX,
        id: text(id),
    }
}

fn provisional(name: TurnName) -> ProvisionalTurnName {
    ProvisionalTurnName { attempt: name.attempt, number: name.number, retry: 0 }
}

#[test]
fn scripted_provisional_retry_reopens_exact_positions_and_replaces_on_empty_start() {
    let mut setup = live_script();
    setup.bounds.text = 32;
    let mut engine = Engine::new(setup, 28).expect("valid script");
    let old = ProvisionalTurnName { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX - 1 };
    engine
        .write_provisional(Time::ZERO, 8, old, None, &text(b"abcdefghijklmnopqrst"), None)
        .expect("provisional text fits");
    engine
        .write_provisional(Time::ZERO, 8, old, None, &text(b"uvwxyz0123456789ABCD"), None)
        .expect("provisional text fits");
    let token = Token::new(1);
    engine
        .open(Time::ZERO, REQUESTER_SIGN_IN, token, Subject::Run { task: 8, turn: Some(old), offset: 20 })
        .expect("typed watch fits");
    let up = engine.poll(Time::ZERO);
    let suffix = RunText {
        writing: None,
        attempt: old.attempt,
        began: None,
        turn: old,
        from: 20,
        through: 40,
        text: text(b"uvwxyz0123456789ABCD"),
        cut: false,
        call: None,
    };
    assert!(up.contains(&Up::Watch { token, event: WatchEvent::Delivered(run_delivery(suffix)) }));
    engine.continue_run(token, Some(old), 99).expect("run continuation fits");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[0] else {
        panic!("bounded retained restart");
    };
    let value = state.writing.as_ref().expect("provisional text");
    assert_eq!((value.turn, value.from, value.through, value.cut), (old, 8, 40, true));
    assert_eq!(value.text.bytes(), b"ijklmnopqrstuvwxyz0123456789ABCD");
    let captured = value.clone();
    let current = ProvisionalTurnName { retry: u32::MAX, ..old };
    engine.write_provisional(Time::ZERO, 8, current, None, &text(b""), None).expect("provisional text fits");
    engine.continue_run(token, Some(old), 40).expect("run continuation fits");
    let up = engine.poll(Time::ZERO);
    let empty = RunText {
        writing: None,
        attempt: current.attempt,
        began: None,
        turn: current,
        from: 0,
        through: 0,
        text: text(b""),
        cut: false,
        call: None,
    };
    assert!(up.contains(&Up::Watch { token, event: WatchEvent::Delivered(run_delivery(empty)) }));
    assert_eq!(captured.turn, old);
    assert_eq!(
        captured.text.bytes(),
        b"ijklmnopqrstuvwxyz0123456789ABCD",
        "captured delivery is immutable across replacement"
    );
    engine
        .write_provisional(Time::ZERO, 8, old, Some(Wall::EPOCH), &text(b"stale"), None)
        .expect("provisional text fits");
    engine.continue_run(token, Some(current), 0).expect("run continuation fits");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[0] else {
        panic!("current retry");
    };
    let value = state.writing.as_ref().expect("provisional text");
    assert_eq!((value.turn, value.from, value.through, value.cut), (current, 0, 0, false));
    assert_eq!(value.began, None);
    assert_eq!(value.text.bytes(), b"");
    engine
        .write_provisional(Time::ZERO, 8, current, Some(Wall::EPOCH), &text(b"retry"), None)
        .expect("provisional text fits");
    engine.close(Time::ZERO, token);
    drop(engine.poll(Time::ZERO));
    let reopened = Token::new(2);
    engine
        .open(Time::ZERO, REQUESTER_SIGN_IN, reopened, Subject::Run { task: 8, turn: Some(old), offset: 99 })
        .expect("typed watch fits");
    let up = engine.poll(Time::ZERO);
    let Up::Watch { event: WatchEvent::Delivered(Delivery::RunState(state)), .. } = &up[1] else {
        panic!("reopened current retry");
    };
    let value = state.writing.as_ref().expect("provisional text");
    assert_eq!((value.turn, value.from, value.through, value.cut), (current, 0, 5, false));
    assert_eq!(value.text.bytes(), b"retry");
    assert_eq!(value.began, Some(Wall::EPOCH));
}

mod task_links;

mod task_contract;

mod history_pages;

mod amendment_reason;
mod question_sender;
mod task_ending;

mod task_hold;

mod escalation;
mod offered;
mod person_task;

mod deployment;
mod ended_objects;

use jig_client_domain::contract::DecideOn;

mod compound;
use compound::releasing;

mod fixtures;
mod person_task_fixture;
mod propose;

#[test]
fn span_acceptance_uses_the_supplied_decision_wall() {
    for (seconds, expected) in [
        (15, Answer::Done(Outcome::Document(DocumentOutcome::Amended { task: 8 }))),
        (
            16,
            Answer::Refused {
                refusal: Refusal::Authority {
                    lacking: Lacking::Deadline {
                        asked: Some(Wall::from_nanos(21_000_000_000)),
                        latest: Some(Wall::from_nanos(20_000_000_000)),
                    },
                    proposable: Some(false),
                },
                at: None,
            },
        ),
    ] {
        let (setup, ask) = fixtures::span_proposal();
        let mut engine = Engine::new(setup, 1).expect("span fixture");
        send(&mut engine, seconds * 1_000_000_000, REQUESTER_SIGN_IN, 1, 1, ask);
        let answers = engine.poll(at(seconds * 1_000_000_000 + 10_000_000));
        assert_eq!(final_answer(&answers[0]), &expected, "{seconds}");
    }
}

mod projects;

fn inbox_entry(
    object: ObjectKey,
    project: u32,
    group: InboxGroup,
    position: u64,
    at: Wall,
    reply: Option<Box<TaskLine>>,
) -> InboxEntry {
    let (task, object) = match object {
        ObjectKey::Question { task, message } => (task, InboxObject::Question { message }),
        ObjectKey::Proposal(key) => (
            match key.proposer {
                Proposer::Task(n) | Proposer::Party(n) => n,
            },
            InboxObject::Proposal { proposer: key.proposer, number: key.number },
        ),
        ObjectKey::Escalation(task) => (task, InboxObject::Escalation { revision: 1 }),
        ObjectKey::PersonTask(task) => (task, InboxObject::PersonTask),
        ObjectKey::Task(task) => (task, InboxObject::Reply { message: position }),
        ObjectKey::Result(task) => (task, InboxObject::Result { position }),
        ObjectKey::Note { .. } => unreachable!("notes have no inbox row"),
    };
    let row = InboxEntry {
        projects: skein_lib::List::with_capacity(0),
        reply,
        task: TaskLink::Stored { number: task, status: None },
        project: Some(project),
        object,
        activity: None,
        at: Some(at),
        unread: None,
        cursor: InboxCursor::Number { number: position },
    };
    assert_eq!(row.group(), group);
    row
}

fn run_delivery(value: RunText) -> Delivery {
    Delivery::RunState(Box::new(RunState {
        task: 8,
        attempt: Some(value.attempt),
        committed_turn: 0,
        writing: Some(value),
        call: None,
    }))
}
pub(crate) fn chip_details(phase: ChipPhase) -> ChipPhaseFacts {
    let ended_at = if matches!(phase, ChipPhase::Ended) { Some(Wall::EPOCH) } else { None };
    ChipPhaseFacts {
        phase,
        details: Box::new(ChipDetails {
            dependencies: List::with_capacity(0),
            project: PROJECT,
            executor: Executor::Agent { charter: 1 },
            phase_since: Wall::EPOCH,
            run_since: None,
            attempt: Some(1),
            retry: None,
            wake: WakePolicy {
                words: WakeRule::Immediate,
                notices: WakeRule::Never,
                news: WakeRule::Never,
                results: ResultsWake::Each,
                questions: false,
                answers: false,
                timers: false,
            },
            person_wait: None,
            hold: None,
            budget: Budget {
                numbers: Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
                left: 100,
                funder: Funder::Period { project: PROJECT, period: 1 },
                deadline: None,
            },
            activity: 0,
            resources: List::with_capacity(0),
            resources_cut: false,
            resources_observed: false,
            procedure: None,
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at,
            observation: None,
        }),
    }
}
fn hold(reason: HoldCategory, revision: u64) -> ChipHold {
    ChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason,
        revision,
        before: PriorPhase::Active(RunStage::Running { attempt: 1 }),
        escalation: None,
        tries: FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    }
}
fn turn_event(position: u64, name: TurnName, turn: TurnRead) -> DecodedActivityLine {
    DecodedActivityLine {
        source: Box::new(ActivityLine {
            task: 8,
            position,
            at: Wall::EPOCH,
            line: Activity::Turn { reply: position, attempt: name.attempt, turn: name.number, read: None },
        }),
        decoded: Some(ArchivedTurn { task: 8, name, cumulative: 1, read: None, at: Wall::EPOCH, turn }),
    }
}

#[test]
fn accounts_watch_delivers_the_signed_in_partys_facts() {
    for history in account_list_rows() {
        let mut setup = script();
        let expected = Accounts {
            program_hosts: list(vec![]),
            accounts: list(vec![AccountLine {
                list: history,
                facts: skein_lib::List::with_capacity(0),
                number: 7,
                place: 0,
                uses: None,
                plan: false,
                provider: Some(3),
                provider_name: None,
                manage_usage: None,
                display: DocumentText::complete(text(b"Account")),
                holder: AccountHolder::Party(REQUESTER),
                sources: list(vec![]),
                cut: false,
            }]),
            signing: list(vec![]),
            last_end: None,
            usable: true,
            settled: true,
            cut: false,
            readiness: list(vec![]),
            readiness_cut: false,
        };
        setup.truth.accounts.insert(REQUESTER, expected.clone());
        let mut engine = Engine::new(setup, 76).expect("valid script");
        engine.open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(1), Subject::Accounts).expect("typed watch fits");
        assert_eq!(
            engine.poll(Time::ZERO),
            vec![
                Up::Watch { token: Token::new(1), event: WatchEvent::Opened },
                Up::Watch {
                    token: Token::new(1),
                    event: WatchEvent::Delivered(Delivery::Accounts(Box::new(expected)))
                }
            ]
        );
    }
}

fn inbox_selections(truth: &mut Truth) {
    truth.inbox_selection.insert(
        REQUESTER,
        [(6, 9), (8, 9), (7, 9), (5, 8), (10, 9), (11, 9)]
            .into_iter()
            .map(|(number, goal)| InboxSelection { cursor: InboxCursor::Number { number }, goals: list(vec![goal]) })
            .collect(),
    );
}

#[test]
fn oversized_request_is_refused_before_any_decision() {
    let mut engine = Engine::new(script(), 1).expect("valid script");
    let words = Text::new(vec![b'x'; 4097].into_boxed_slice(), 4097).expect("producer text");
    assert_eq!(
        engine.send(
            Time::ZERO,
            Wall::EPOCH,
            REQUESTER_SIGN_IN,
            [1; 16],
            Token::new(1),
            fixtures::create(PROJECT, words)
        ),
        Err(Invalid::Payload)
    );
    assert!(engine.commits().is_empty());
    assert!(engine.pending_commits().next().is_none());
}
#[test]
fn task_query_parent_uses_requester_and_ended_time_uses_terminal_phase() {
    let mut value = task();
    let mut query = query(PROJECT);
    query.parent = Some(42);
    value.requester = Requester::Task(42);
    value.lineage = list(vec![99]);
    assert!(crate::queries::selects(&query, &value));
    value.requester = Requester::Person(42);
    value.lineage = list(vec![42]);
    assert!(!crate::queries::selects(&query, &value));
    for phase in [
        Phase::Waiting,
        Phase::Idle,
        Phase::Due,
        Phase::Preparing,
        Phase::Claimed,
        Phase::Running,
        Phase::BackingOff,
        Phase::Active,
        Phase::Closing,
        Phase::Held,
    ] {
        value.chip.phase = phase;
        assert_eq!(crate::queries::line(&value, [&value].into_iter(), None).ended_at, None, "{phase:?}");
    }
    for phase in [Phase::Ended, Phase::Done, Phase::Failed, Phase::Cancelled] {
        value.chip.phase = phase;
        assert_eq!(crate::queries::line(&value, [&value].into_iter(), None).ended_at, Some(value.changed), "{phase:?}");
    }
}

fn account_list_rows() -> [Option<AccountList>; 8] {
    let empty = AccountList { read: None, tried: None, failure: None, left_out: 0 };
    let full = |failure| {
        Some(AccountList {
            read: Some(Wall::from_nanos(13)),
            tried: Some(Wall::from_nanos(17)),
            failure: Some(failure),
            left_out: u32::MAX,
        })
    };
    [
        None,
        Some(empty),
        full(AccountListFailure::Refused { status: 429 }),
        full(AccountListFailure::Unreachable),
        full(AccountListFailure::Unreadable),
        full(AccountListFailure::Deadline),
        full(AccountListFailure::Agent),
        full(AccountListFailure::Room),
    ]
}
