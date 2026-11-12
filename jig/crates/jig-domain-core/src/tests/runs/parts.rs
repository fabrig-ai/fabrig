//! Run preparation chooses instructions and messages for its reasons (jig/docs/design/domain/engine/executors.md, 1).
use crate::routing::run::{
    parts::{offered_kinds, run_instructions},
    preparation,
};
use crate::tests::*;
use jig_domain_brief as brief;
use jig_domain_link::{Form, Scheme};
use tasks::Reasons;

fn part(on: Reasons, instructions: &[u8]) -> CharterPart {
    CharterPart { on, instructions: instructions.into() }
}

#[test]
fn run_parts_follow_reasons_in_order_and_own_instructions_replace_them() {
    let mut charter = config(1).defaults.charters[0].clone();
    charter.policy.resume = false;
    charter.policy.instructions = b"S".as_slice().into();
    let parts = Box::new([part(Reasons::FIRST, b"F"), part(Reasons::QUESTION, b"Q")]);
    let none: Box<[CharterPart]> = Box::new([]);
    for (name, parts, own, reasons, instructions, offered) in [
        ("none", none, None, Reasons::FIRST, b"S".as_slice(), Reasons::ALL),
        ("first", parts.clone(), None, Reasons::FIRST, b"S\n\nF", Reasons::EMPTY),
        (
            "first and question",
            parts.clone(),
            None,
            Reasons::FIRST.union(Reasons::QUESTION),
            b"S\n\nF\n\nQ",
            Reasons::QUESTION,
        ),
        ("question", parts.clone(), None, Reasons::QUESTION, b"S\n\nQ", Reasons::QUESTION),
        ("unmatched", parts.clone(), None, Reasons::RESULT, b"S", Reasons::EMPTY),
        (
            "both read once",
            Box::new([part(Reasons::QUESTION.union(Reasons::RESULT), b"QR")]),
            None,
            Reasons::QUESTION.union(Reasons::RESULT),
            b"S\n\nQR",
            Reasons::QUESTION.union(Reasons::RESULT),
        ),
        (
            "whole selected on",
            Box::new([part(Reasons::QUESTION.union(Reasons::RESULT), b"QR")]),
            None,
            Reasons::QUESTION,
            b"S\n\nQR",
            Reasons::QUESTION.union(Reasons::RESULT),
        ),
        ("own", parts.clone(), Some(b"own".as_slice()), Reasons::FIRST, b"own", Reasons::ALL),
        ("empty own", parts, Some(b"".as_slice()), Reasons::FIRST, b"", Reasons::ALL),
    ] {
        charter.parts = parts;
        assert_eq!(run_instructions(&charter, own, reasons, false).as_ref(), instructions, "{name}: instructions");
        assert_eq!(offered_kinds(&charter, own.is_some(), reasons, false), offered, "{name}: offered");
    }
}

#[test]
fn preparation_offers_person_words_and_keeps_an_uncovered_question() {
    let (mut core, mut limits) = make_tree();
    let charter = core.current.test_charter_mut(1);
    charter.policy.resume = false;
    charter.policy.instructions = b"S".as_slice().into();
    charter.parts = Box::new([part(Reasons::FIRST, b"F")]);
    // A skipped question comes first, so the run-inbox cap is applied after filtering.
    limits.tasks.run_inbox = 1;
    for (number, from, kind) in [
        (1, tasks::Party::Task(2), tasks::MessageKind::Question),
        (2, tasks::Party::Person(1), tasks::MessageKind::Words),
    ] {
        send_task(
            &mut core,
            &limits,
            tasks::Event::Message {
                reply_to: ReplyTo::new(Token::new(number)),
                project: 1,
                operation: tasks::Message {
                    task: 1,
                    word: tasks::Word {
                        number,
                        from,
                        kind,
                        words: b"person".as_slice().into(),
                        at: Wall::EPOCH,
                        hits: 1,
                        eligible: true,
                    },
                },
            },
        );
    }
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.begin_transcript(1, 0, None);
    watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
    let attempt = core.workspace_flight(1).unwrap().1;
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: None,
        },
    );
    let mut prepared = None;
    for request in &out {
        if let Request::Now(now) = request
            && let Now::RunPrepared { run, messages, .. } = now.as_ref()
        {
            prepared = Some((run, messages));
        }
    }
    let (run, messages) = prepared.expect("prepared run");
    assert_eq!(run.policy.instructions.as_ref(), b"S\n\nF");
    let mut numbers = List::with_capacity(limits.tasks.run_inbox);
    for word in messages {
        numbers.push(word.number).unwrap();
    }
    assert_eq!(numbers.as_slice(), [2]);
    let proof = core.proofs.get(&1).unwrap();
    assert_eq!(proof.outstanding.as_ref(), [2]);
    assert_eq!(proof.offered, Some(2));
    let mut work = Queue::uncapped();
    let reasons = core.contexts.get(&1).unwrap().reasons;
    preparation::route_claim_prepared(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        &mut work,
        1,
        attempt,
        run.budget,
        Box::new([]),
    );
    let Some(Work::Event(Event::Tasks(tasks::Event::Claim { offered, reasons: claimed, .. }))) = work.pop() else {
        panic!("prepared claim");
    };
    assert_eq!(claimed, reasons);
    assert_eq!(offered, Reasons::EMPTY);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arrival {
    BeforeBrief,
    DuringResult,
    DuringNotes,
    BeforeComplete,
    BeforeWorkspace,
}

fn preparing_parts(parts: bool, questions: bool) -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    let mut root = new_row(&core, 1, tasks::Funder::Period { project: 1, period: 1 }, Some(4));
    root.wake.questions = questions;
    root.wake.words = tasks::WakeRule::Never;
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([root]));
    let child = new_row(&core, 2, tasks::Funder::Task(1), None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
    // Asking its requester keeps a live sender's phase unchanged in the prepared brief.
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(2)), task: 2 });
    send_task(
        &mut core,
        &limits,
        tasks::Event::Claim {
            session_end: None,
            reply_to: ReplyTo::new(Token::new(2)),
            task: 2,
            attempt: 1,
            resumed: false,
            budget: 1,
            account: None,
            writes: Box::new([]),
            reasons: Reasons::FIRST,
            offered: Reasons::ALL,
        },
    );
    let charter = core.current.test_charter_mut(1);
    charter.policy.resume = false;
    charter.policy.instructions = b"S".as_slice().into();
    if parts {
        charter.parts = Box::new([part(Reasons::FIRST, b"F"), part(Reasons::QUESTION, b"Q")]);
    }
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    runs::run_choice::account(&mut core, &limits, accounts::Event::Want);
    runs::run_choice::granted(&mut core, &limits, 1);
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.begin_transcript(1, 0, None);
    (core, limits)
}

#[test]
fn sent_words_follow_prepared_parts_and_always_complete_their_route() {
    for (name, parts, own, from, kind, appended) in [
        ("kept question", true, false, tasks::Party::Task(2), tasks::MessageKind::Question, false),
        ("person words", true, false, tasks::Party::Person(1), tasks::MessageKind::Words, true),
        ("no parts", false, false, tasks::Party::Task(2), tasks::MessageKind::Question, true),
        ("own instructions", true, true, tasks::Party::Task(2), tasks::MessageKind::Question, true),
    ] {
        let (mut core, limits) = preparing_parts(parts, true);
        if own {
            let mut row = core.tasks.task(1).unwrap().clone();
            row.instructions = Some(b"own".as_slice().into());
            fixtures::restore(&mut core, &limits, Box::new([row]));
        }
        let prepared = core.contexts.get(&1).unwrap().clone();
        let word = tasks::Word {
            number: 1,
            from,
            kind,
            words: b"arriving".as_slice().into(),
            at: Wall::EPOCH,
            hits: 1,
            eligible: true,
        };
        let request = Token::new(11);
        let key = CallKey { task: 2, attempt: 1, completion: 1, position: 0 };
        let flight = core.fly_call(request, RoutedCall::Message(key));
        let route = core.sent(flight, 1, &word, limits.tasks.inbox_messages, limits.tasks.tasks);
        let SentRoute::Call { request: completed, key: caller, message } = route else {
            panic!("{name}: sent word completes its call");
        };
        assert_eq!((completed, caller, message), (request, key, word.number), "{name}: terminal route");
        let mut expected = prepared;
        if appended {
            expected.inbox = Box::new([word]);
            expected.last_message = 1;
        }
        assert_eq!(core.contexts.get(&1), Some(&expected), "{name}: prepared snapshot");
        assert!(core.land_task(flight).is_none(), "{name}: route consumed once");
    }
}

fn arrive(core: &mut Core, limits: &Limits, from: tasks::Party, kind: tasks::MessageKind) {
    watch_outputs(
        core,
        limits,
        Event::Tasks(tasks::Event::Message {
            reply_to: ReplyTo::new(Token::new(1)),
            project: 1,
            operation: tasks::Message {
                task: 1,
                word: tasks::Word {
                    number: 1,
                    from,
                    kind,
                    words: b"arriving".as_slice().into(),
                    at: Wall::EPOCH,
                    hits: 1,
                    eligible: true,
                },
            },
        }),
    );
}

fn notes_load(out: &[Request]) -> Token {
    for request in out {
        if let Request::Held(held) = request
            && let Held::NotesLoad { owner, .. } = held.as_ref()
        {
            return *owner;
        }
    }
    panic!("preparation loads notes");
}

fn finish_brief(core: &mut Core, limits: &Limits, mut owner: Token) {
    // The fixture reads its goal, project and deployment scopes.
    const SCOPES: u32 = 3;
    let mut out: Box<[Request]> = Box::new([]);
    for _ in 0..SCOPES {
        out = watch_outputs(
            core,
            limits,
            Event::Notes(notes::Event::Loaded {
                owner,
                rows: notes::Rows { records: List::with_capacity(limits.notes.load_rows) },
                more: false,
            }),
        );
        for request in &out {
            if let Request::Held(held) = request
                && let Held::NotesLoad { owner: next, .. } = held.as_ref()
            {
                owner = *next;
            }
        }
    }
    let mut planned = false;
    for request in &out {
        if let Request::Now(now) = request
            && let Now::BriefCorePlanned { task: 1, .. } = now.as_ref()
        {
            planned = true;
        }
    }
    assert!(planned, "preparation plans its brief");
    watch_outputs(
        core,
        limits,
        Event::Brief(brief::GatherEvent::Plan {
            brief: Token::new(1),
            budget: 128,
            deadline: Time::from_nanos(100),
            sections: Box::new([brief::Planned::Connector {
                name: b"context",
                connector: 1,
                kind: 1,
                token: Token::new(19),
                size: 0,
                limit: 128,
                priority: 0,
                required: true,
            }]),
        }),
    );
}

#[test]
fn arrivals_adding_reasons_invalidate_preparation_and_other_words_follow_parts() {
    for arrival in [
        Arrival::BeforeBrief,
        Arrival::DuringResult,
        Arrival::DuringNotes,
        Arrival::BeforeComplete,
        Arrival::BeforeWorkspace,
    ] {
        for (name, parts, questions, from, kind, kept) in [
            ("new question", true, true, tasks::Party::Task(2), tasks::MessageKind::Question, false),
            ("non-waking question", true, false, tasks::Party::Task(2), tasks::MessageKind::Question, false),
            ("non-waking words", true, false, tasks::Party::Task(2), tasks::MessageKind::Words, true),
            ("person words", true, true, tasks::Party::Person(1), tasks::MessageKind::Words, false),
            ("no parts", false, true, tasks::Party::Task(2), tasks::MessageKind::Question, false),
        ] {
            let (mut core, limits) = preparing_parts(parts, questions);
            let prepared = core.contexts.get(&1).unwrap().clone();
            let mut result = None;
            if arrival == Arrival::DuringResult {
                let mut out = crate::routing::run::results::begin(&mut core, &mut Queue::uncapped(), 1, Box::new([12]));
                let mut requests = List::with_capacity(out.len());
                while let Some(request) = out.pop() {
                    requests.push(request).unwrap();
                }
                result = Some(runs::result_reads::requested(requests.as_slice(), 12));
            }
            let mut owner = None;
            if arrival != Arrival::BeforeBrief && arrival != Arrival::DuringResult {
                let out = watch_outputs(
                    &mut core,
                    &limits,
                    Event::TranscriptLoaded { task: 1, load: TranscriptLoad::Page { rows: Box::new([]), done: true } },
                );
                owner = Some(notes_load(&out));
            }
            if arrival == Arrival::BeforeComplete || arrival == Arrival::BeforeWorkspace {
                finish_brief(&mut core, &limits, owner.unwrap());
            }
            if arrival == Arrival::BeforeWorkspace {
                watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
            }
            arrive(&mut core, &limits, from, kind);
            assert_eq!(core.contexts.contains_key(&1), kept, "{arrival:?}: {name}: context current");
            if !kept {
                let tasks::Phase::Active(tasks::Active::BackingOff { .. }) = core.tasks.task(1).unwrap().phase else {
                    panic!("{arrival:?}: {name}: stale preparation backs off");
                };
                assert!(!core.claiming.contains_key(&1));
                continue;
            }
            assert_eq!(core.contexts.get(&1), Some(&prepared), "{arrival:?}: prepared facts retained");
            if arrival == Arrival::BeforeBrief || arrival == Arrival::DuringResult {
                let event = match result {
                    Some(request) => Event::EndedTaskLoaded {
                        request,
                        row: Some(Box::new(fixtures::ended_report(12, tasks::Party::Person(1)))),
                        busy: false,
                    },
                    None => Event::TranscriptLoaded {
                        task: 1,
                        load: TranscriptLoad::Page { rows: Box::new([]), done: true },
                    },
                };
                let out = watch_outputs(&mut core, &limits, event);
                owner = Some(notes_load(&out));
            }
            if arrival != Arrival::BeforeComplete && arrival != Arrival::BeforeWorkspace {
                finish_brief(&mut core, &limits, owner.unwrap());
            }
            let out = watch_outputs(
                &mut core,
                &limits,
                Event::Brief(brief::GatherEvent::Ready { brief: Token::new(1), section: Token::new(19), size: 1 }),
            );
            let mut complete = false;
            for request in &out {
                if let Request::Now(now) = request
                    && let Now::CompleteBrief { .. } = now.as_ref()
                {
                    complete = true;
                }
            }
            assert!(complete, "{arrival:?}: completes brief");
            if arrival != Arrival::BeforeWorkspace {
                watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
            }
            let attempt = core.workspace_flight(1).unwrap().1;
            let out = watch_outputs(
                &mut core,
                &limits,
                Event::WorkspacePrepared {
                    task: 1,
                    attempt,
                    holds: Box::new([]),
                    writes: Some(Box::new([])),
                    credentials: Box::new([]),
                    workspace: None,
                },
            );
            let mut budget = None;
            for request in &out {
                if let Request::Now(now) = request
                    && let Now::RunPrepared { run, messages, .. } = now.as_ref()
                {
                    assert_eq!(run.policy.instructions.as_ref(), b"S\n\nF", "{arrival:?}: prepared instructions");
                    assert!(messages.is_empty(), "{arrival:?}: kept word absent at start");
                    budget = Some(run.budget);
                }
            }
            let Some(budget) = budget else { panic!("{arrival:?}: prepared run reaches claim") };
            watch_outputs(&mut core, &limits, Event::ClaimPrepared { task: 1, attempt, budget, writes: Box::new([]) });
            let row = core.tasks.task(1).unwrap();
            assert_eq!(row.attempt, attempt, "{arrival:?}: claim made");
            assert_eq!(row.reasons, prepared.reasons, "{arrival:?}: prepared reasons claimed");
            assert_eq!(row.offered, Reasons::EMPTY);
            assert_eq!(row.refusals, 0);
            assert_eq!(row.inbox.len(), 1, "{arrival:?}: kept word keeps its place");
        }
    }
}

#[test]
fn overlay_run_preparation_selects_default_then_overlay_parts() {
    let (mut core, mut limits) = make_tree();
    limits.run_bytes = 2048;
    let charter = core.current.test_charter_mut(1);
    charter.policy.resume = false;
    charter.policy.instructions = b"S".as_slice().into();
    charter.parts = Box::new([part(Reasons::FIRST, b"F")]);
    let overlay = crate::settings::Overlay {
        instructions: b"O".as_slice().into(),
        parts: Box::new([part(Reasons::FIRST, b"OF"), part(Reasons::RESULT, b"skip")]),
        ..crate::settings::Overlay::empty()
    };
    let items = [crate::settings::Item {
        setting: crate::settings::Setting::Charter(1),
        edit: crate::settings::Edit::Set(crate::settings::Value::Overlay(Box::new(overlay))),
    }];
    crate::settings::check_change(&core.current, &items, &core.settings, &core.connectors, &limits).unwrap();
    drop(crate::settings::apply(&mut core.current, &items, 1));
    charters::rebuild(&mut core.current, &limits);
    // A skipped question comes first, so the run-inbox cap is applied after filtering.
    limits.tasks.run_inbox = 1;
    for (number, from, kind) in [
        (1, tasks::Party::Task(2), tasks::MessageKind::Question),
        (2, tasks::Party::Person(1), tasks::MessageKind::Words),
    ] {
        send_task(
            &mut core,
            &limits,
            tasks::Event::Message {
                reply_to: ReplyTo::new(Token::new(number)),
                project: 1,
                operation: tasks::Message {
                    task: 1,
                    word: tasks::Word {
                        number,
                        from,
                        kind,
                        words: b"person".as_slice().into(),
                        at: Wall::EPOCH,
                        hits: 1,
                        eligible: true,
                    },
                },
            },
        );
    }
    runs::run_choice::host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 1 });
    core.contexts.insert(1, core.tasks.run_context(1).unwrap()).unwrap();
    core.begin_transcript(1, 0, None);
    watch_outputs(&mut core, &limits, Event::BriefAssembled { task: 1, failure: None });
    let attempt = core.workspace_flight(1).unwrap().1;
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::WorkspacePrepared {
            task: 1,
            attempt,
            holds: Box::new([]),
            writes: Some(Box::new([])),
            credentials: Box::new([]),
            workspace: None,
        },
    );
    let mut prepared = None;
    for request in &out {
        if let Request::Now(now) = request
            && let Now::RunPrepared { run, messages, .. } = now.as_ref()
        {
            prepared = Some((run, messages));
        }
    }
    let (run, messages) = prepared.expect("prepared run");
    assert_eq!(run.policy.instructions.as_ref(), b"S\n\nO\n\nF\n\nOF");
    let mut numbers = List::with_capacity(limits.tasks.run_inbox);
    for word in messages {
        numbers.push(word.number).unwrap();
    }
    assert_eq!(numbers.as_slice(), [2]);
    let proof = core.proofs.get(&1).unwrap();
    assert_eq!(proof.outstanding.as_ref(), [2]);
    assert_eq!(proof.offered, Some(2));
    let mut work = Queue::uncapped();
    let reasons = core.contexts.get(&1).unwrap().reasons;
    preparation::route_claim_prepared(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        &mut work,
        1,
        attempt,
        run.budget,
        Box::new([]),
    );
    let Some(Work::Event(Event::Tasks(tasks::Event::Claim { offered, reasons: claimed, .. }))) = work.pop() else {
        panic!("prepared claim");
    };
    assert_eq!(claimed, reasons);
    assert_eq!(offered, Reasons::EMPTY);
}
