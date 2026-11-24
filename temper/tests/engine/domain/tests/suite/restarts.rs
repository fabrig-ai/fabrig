//! Named engine cuts (testing/restarts.md, 4 and 10).
use super::*;
use skein_world::faults::{FaultLedger, Kind, Struck};
use skein_world::streams::{Stream, Streams};
use temper_engine_domain_world::forge_routes::restarts::Injector;
use temper_faults::restarts::*;

fn interval(ledger: &FaultLedger, kind: Kind, from: Time, until: Time) {
    let entry = ledger.entries().iter().find(|entry| entry.kind == kind).expect("fault fell");
    assert_eq!(entry.from, from);
    assert_eq!(entry.until, Some(until));
    assert_eq!(entry.struck, Struck::default());
}

#[test]
fn crash_with_an_unanswered_commit_and_forge_call_keeps_or_loses_the_commit() {
    for (keep, make_call) in [(false, false), (false, true), (true, false), (true, true)] {
        let (mut world, _) = pending_issue_effect();
        // The effect's commit is durable: releasing it sends CreateIssue.
        world.inject_restarts(Injector::at(0, ENGINE_CRASH, 10_000));
        let number = world.store.apply();
        world.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        for _ in 0..20 {
            if world
                .pending
                .values()
                .any(|op| matches!(op, skein_forge::api::Op::Write(skein_forge::api::Write::CreateIssue { .. })))
            {
                break;
            }
            world.tick();
        }
        assert!(
            world
                .pending
                .values()
                .any(|op| matches!(op, skein_forge::api::Op::Write(skein_forge::api::Write::CreateIssue { .. }))),
            "issue call sent, not answered"
        );
        let sign_in = world.store.header().sign_ins + 1;
        let key = Key::Core(jig_domain_core::Key::People(people::Key::SignIn(sign_in)));
        assert!(!world.store.rows.contains_key(&key));
        world.send(engine::Event::Party(engine::PartyInput::SignedIn {
            until: None,
            held: None,
            reply_to: ReplyTo::new(Token::new(199)),
            identity: people::Identity {
                key: temper_engine_domain_forge::user_key(1, 7_u64),
                login: b"owner".as_slice().into(),
                name: b"Owner".as_slice().into(),
            },
        }));
        assert!(!world.store.pending.is_empty(), "new sign-in submitted, not answered");
        // Find a seed for the named persistence outcome; the cut's production
        // draw still decides the outcome, not a fixture edit to durable rows.
        assert_eq!(world.store.pending.len(), 1);
        assert_eq!(world.pending.len(), 1);
        let seed = (0..100)
            .find(|seed| {
                let mut random = Streams::new(*seed).rng(Stream::Faults, PART);
                (random.below(2) != 0) == keep && (random.below(2) != 0) == make_call
            })
            .expect("each persistence and forge outcome");
        let from = world.env.now;
        let mut injector = Injector::at(seed, ENGINE_CRASH, 0);
        world.restarts = None;
        injector.crash(&mut world);
        assert_eq!(injector.ledger.entries()[0].until, None, "no opening yet");
        assert!(world.pending.is_empty());
        assert!(world.immediate.is_empty());
        assert!(world.held_write_answers.is_empty());
        assert_eq!(world.store.rows.contains_key(&key), keep, "whole sign-in transaction");
        assert_eq!(issue_count(&mut world), usize::from(make_call), "pending forge call made or lost");
        assert!(
            world.events.iter().all(|event| !matches!(
                event,
                engine::Event::Store(engine::StoreInput::Committed { .. }) | engine::Event::Forge(_)
            )),
            "no old terminal crosses the crash"
        );
        world.open_after_fault();
        injector.opened(&world);
        assert_eq!(world.store.rows.contains_key(&key), keep, "new root reads the kept rows");
        interval(&injector.ledger, ENGINE_CRASH, from, world.env.now);
        assert_eq!(injector.ledger.entries().len(), 1);
        assert!(world.root.ready());
        world.advance_ticks(10);
        assert!(world.root.ready(), "old terminals did not reach replacement");
    }
}

#[test]
fn settled_stop_after_opening_restarts_from_its_durable_word() {
    let mut world = World::new().with_restart_wait(Duration::from_secs(900), false);
    world.until(Until::Ready);
    let from = world.env.now;
    let commits = world.store.header().commits;
    let mut injector = Injector::at(0, SETTLED_STOP, 0);
    injector.last(&mut world);
    assert!(world.root.ready());
    assert!(world.restarting.contains(&(Duration::from_secs(900), commits + 1)));
    assert_eq!(injector.ledger.entries().len(), 1);
    interval(&injector.ledger, SETTLED_STOP, from, world.env.now);
}

#[test]
fn a_stop_before_the_mark_or_after_the_opening_opens_on_the_next_start() {
    for kind in [STOP_BEFORE_MARK, STOP_AFTER_OPENING] {
        let mut world = crate::forge_routes::follow_changes::proof::fixture();
        world.until(Until::Ready);
        world.drain_cut();
        world.restart_same();
        let commits = world.store.applied;
        let mut injector = Injector::at(0, kind, 0);
        let (phase, stopped_commits, catching_up) = injector.stop_in_class(&mut world);
        match kind {
            STOP_BEFORE_MARK => assert!(matches!(phase, Some(jig_domain_core::start::Phase::LoadCore))),
            STOP_AFTER_OPENING => {
                assert!(phase.is_none());
                assert!(catching_up, "the stop falls before the forge finishes catching up");
            }
            _ => unreachable!(),
        }
        let from = injector.ledger.entries().first().expect("class fault").from;
        if kind == STOP_BEFORE_MARK {
            assert_eq!(stopped_commits, commits, "no wires or later phase committed before the stop");
        }
        assert!(world.root.ready());
        assert_eq!(world.root.start_stop(), None);
        assert_eq!(injector.ledger.entries().len(), 1);
        assert!(from < world.env.now, "next start follows the stop");
        interval(&injector.ledger, kind, from, world.env.now);
        assert!(world.store.applied > commits, "next start committed its wires");
    }
}

#[test]
fn an_unlisted_committed_claim_is_replaced_only_after_the_restart_hold() {
    let mut world = World::configured(true, false).with_restart_wait(Duration::from_secs(900), false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.held_assignments = Some(Vec::new());
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(200)),
        sign_in: world.signed_in.expect("owner session"),
        key: [200; 16],
        ask: world.chat(1, b"a claim before its worker receives it"),
    }));
    for _ in 0..200 {
        if world.held_assignments.as_ref().is_some_and(|held| !held.is_empty()) {
            break;
        }
        world.tick();
    }
    let queued = world.held_assignments.as_ref().and_then(|held| held.first()).expect("queued assignment").clone();
    assert!(world.assigned.is_empty(), "the worker has not received this assignment");
    world.stop_engine();
    let wait = Duration::from_millis(world.store.header().restart_wait_ms.expect("durable stop word"));
    let hold_ends = world.env.now.saturating_add(wait);
    world.crash_engine(&mut Streams::new(0).rng(Stream::Faults, PART));
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.open_after_fault();
    for _ in 0..200 {
        if world.events.is_empty()
            && world.store.idle()
            && world.pending.is_empty()
            && !world.root.forge_work_ready()
            && world.root.backlog() == 0
        {
            break;
        }
        world.tick();
    }
    assert!(world.assigned.is_empty(), "the missing claim is still held");
    let next = world.root.next_deadline(&world.env).expect("recovery alarm");
    assert!(next > world.env.now);
    world.advance_to_recovery_deadline();
    assert_eq!(world.env.now, next, "the adapter visits the existing alarm");
    world.env.now = Time::from_nanos(hold_ends.as_nanos().saturating_sub(1));
    engine::fire(&mut world.root, &world.env);
    engine::release(&mut world.root, &world.env, &mut world.out);
    world.collect();
    assert!(world.assigned.is_empty(), "no replacement before the hold ends");
    // The adapter visits existing alarms until the missing claim can be retried.
    for _ in 0..(wait.as_nanos() / world.tick_interval.as_nanos()) {
        world.advance_to_recovery_deadline();
        world.tick();
        if world.assigned.iter().any(|run| run.task == queued.task && run.attempt > queued.attempt) {
            break;
        }
    }
    assert!(world.env.now >= hold_ends);
    assert!(world.assigned.iter().any(|run| run.task == queued.task && run.attempt > queued.attempt));
}

#[test]
fn second_signal_aborts_before_the_stop_commits() {
    let mut world = World::new().with_restart_wait(Duration::from_secs(900), false);
    world.until(Until::Ready);
    let from = world.env.now;
    let commits = world.store.applied;
    let mut injector = Injector::at(0, SECOND_SIGNAL, 0);
    injector.last(&mut world);
    assert!(world.root.ready());
    assert!(world.store.pending.is_empty(), "no aborted root's commit is still in flight");
    assert!(world.store.applied > commits, "the replacement committed its start");
    assert_eq!(injector.ledger.entries().len(), 2);
    interval(&injector.ledger, SETTLED_STOP, from, world.env.now);
    let second = injector.ledger.entries().iter().find(|entry| entry.kind == SECOND_SIGNAL).expect("abort");
    interval(&injector.ledger, SECOND_SIGNAL, second.from, second.from);
    assert!(world.env.now > second.from, "the opening belongs to the replacement, not the stopping root");
}
