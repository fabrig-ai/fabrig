//! Step-level fault reports, without a world or a ledger dependency.
use super::*;
use crate::{Called, Fault, Fell};

fn blank(at: Time, fault: Fault) -> Fell {
    Fell {
        at,
        until: None,
        fault,
        repository: None,
        user: None,
        call: None,
        reply_to: None,
        pull: None,
        commit: None,
        run: None,
        runner: None,
    }
}

fn call_record(at: Time, fault: Fault, token: u64, call: Called) -> Fell {
    Fell {
        repository: Some(copy_of(REPOSITORY)),
        user: Some(ENGINE),
        call: Some(call),
        reply_to: Some(Token::new(token)),
        ..blank(at, fault)
    }
}

fn send(h: &mut Harness, token: u64) {
    step(
        &mut h.domain,
        &h.env,
        Event::Call {
            reply_to: ReplyTo::new(Token::new(token)),
            user: ENGINE,
            repository: copy_of(REPOSITORY),
            op: read(Read::Locate),
        },
        &mut h.out,
    );
}

#[test]
fn busy_reports_the_second_call_not_the_one_in_flight() {
    let mut config = calm();
    config.limits.calls = 1;
    let mut h = Harness::new(config);
    h.env.now = Time::from_nanos(3_000_000_000);
    send(&mut h, 11);
    assert_eq!(h.domain.pop_fell(), None);
    send(&mut h, 22);
    let Request::Reply { to, result } = h.out.pop().unwrap() else { panic!("busy reply") };
    assert_eq!(to.into_token(), Token::new(22));
    assert_eq!(result, Err(Error::Unavailable));
    assert_eq!(h.domain.pop_fell(), Some(call_record(h.env.now, Fault::Busy, 22, Called::Read)));
    assert_eq!(h.domain.pop_fell(), None);
}

#[test]
fn unavailable_and_timeout_report_the_call_once() {
    for (fault, config, error) in [
        (Fault::Unavailable, Config { unavailable: 1000, ..calm() }, Error::Unavailable),
        (Fault::TimedOut, Config { timeouts: 1000, ..calm() }, Error::Timeout),
    ] {
        let mut h = Harness::new(config);
        h.env.now = Time::from_nanos(7_000_000_000);
        let at = h.env.now;
        assert_eq!(h.call(ENGINE, read(Read::Locate)), Err(error));
        assert_eq!(h.domain.pop_fell(), Some(call_record(at, fault, 1, Called::Read)));
        assert_eq!(h.domain.pop_fell(), None);
    }
}

#[test]
fn a_landing_reports_the_merge_and_its_landing_deadline() {
    let mut h = Harness::new(calm());
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    let Answer::Created(pull) = h.open(b"work").unwrap() else { panic!("pull") };
    h.settle();
    assert_eq!(h.domain.pop_fell(), None);
    h.env.limits.landing = 1000;
    assert_eq!(h.call(ENGINE, merge(pull, head)), Err(Error::Timeout));
    let fell = h.domain.pop_fell().unwrap();
    let until = h.env.now.saturating_add(Duration::from_secs(3));
    assert_eq!(
        fell,
        Fell {
            until: Some(until),
            pull: Some(pull),
            commit: Some(head),
            ..call_record(h.env.now, Fault::Landing, h.calls, Called::Merge)
        }
    );
    assert_eq!(h.branch(MAIN), Some(FIRST));
    h.until(until);
    assert_ne!(h.branch(MAIN), Some(FIRST));
    assert_eq!(h.domain.pop_fell(), None);
}

#[test]
fn late_reports_world_time_and_the_answer_deadline() {
    let config = Config { late: 1000, skew: Skew::Ahead(Duration::from_secs(200)), ..calm() };
    let mut h = Harness::new(config);
    h.env.now = Time::from_nanos(5_000_000_000);
    let at = h.env.now;
    h.ok(ENGINE, read(Read::Locate));
    assert_eq!(
        h.domain.pop_fell(),
        Some(Fell { until: Some(h.env.now), ..call_record(at, Fault::Late, 1, Called::Read) })
    );
    assert_eq!(h.env.now, at.saturating_add(Duration::from_secs(30)));
    assert_eq!(h.domain.pop_fell(), None);
}

#[test]
fn rate_reports_limited_and_crowded_with_call_identity() {
    let mut h = Harness::new(Config { rate_limit: 1, ..calm() });
    h.ok(ENGINE, read(Read::Locate));
    let at = h.env.now;
    let _denied = h.call(ENGINE, read(Read::Locate)).expect_err("the user exhausted its window");
    assert_eq!(h.domain.pop_fell(), Some(call_record(at, Fault::Limited, 2, Called::Read)));
    for user in 2..=u64::from(LIMITS.users) {
        let _answer = h.call(user, read(Read::Locate));
    }
    let at = h.env.now;
    let user = 99;
    let _denied = h.call(user, read(Read::Locate)).expect_err("all retained windows are running");
    assert_eq!(
        h.domain.pop_fell(),
        Some(Fell { user: Some(user), ..call_record(at, Fault::Crowded, h.calls, Called::Read) })
    );
    assert_eq!(h.domain.pop_fell(), None);
}

#[test]
fn protocol_rate_reports_without_inventing_call_context() {
    let mut h = Harness::new(calm());
    for user in 1..=u64::from(LIMITS.users) {
        let _rate = crate::rate(&mut h.domain, &h.env, crate::RateKey::User(user), 1, Duration::from_secs(10)).unwrap();
    }
    let _denied = crate::rate(&mut h.domain, &h.env, crate::RateKey::User(1), 1, Duration::from_secs(10))
        .expect_err("exhausted quota");
    assert_eq!(h.domain.pop_fell(), Some(Fell { user: Some(1), ..blank(h.env.now, Fault::Limited) }));
    let _denied = crate::rate(&mut h.domain, &h.env, crate::RateKey::Installation(99), 1, Duration::from_secs(10))
        .expect_err("full rate table");
    assert_eq!(h.domain.pop_fell(), Some(blank(h.env.now, Fault::Crowded)));
    assert_eq!(h.domain.pop_fell(), None);
}

#[test]
fn hook_faults_report_the_hook_not_its_call() {
    let mut dropped = calm();
    dropped.limits.hooks = 0;
    for (fault, config) in [
        (Fault::HookLost, Config { hooks_lost: 1000, ..calm() }),
        (Fault::HookLate, Config { hooks_late: 1000, ..calm() }),
        (Fault::HookDropped, dropped),
    ] {
        let mut configured = setup();
        configured.checks.contexts = Box::new([]);
        let mut h = Harness::with(config, configured);
        let at = h.env.now;
        let head = h.commit(FIRST, &[(b"src", b"one")]);
        h.push(ENGINE, b"work", head).unwrap();
        assert_eq!(
            h.domain.pop_fell(),
            Some(Fell {
                until: if fault == Fault::HookLate { Some(at.saturating_add(Duration::from_secs(30))) } else { None },
                repository: Some(copy_of(REPOSITORY)),
                user: Some(ENGINE),
                commit: Some(head),
                ..blank(at, fault)
            })
        );
        h.settle();
        assert_eq!(h.domain.pop_fell(), None);
    }
}

#[test]
fn check_silence_and_rerun_report_the_commit_and_run() {
    for fault in [Fault::Silent, Fault::Rerun] {
        let mut configured = setup();
        configured.checks.silent = if fault == Fault::Silent { 1000 } else { 0 };
        configured.checks.reruns = if fault == Fault::Rerun { 1000 } else { 0 };
        let mut h = Harness::with(calm(), configured);
        let at = h.env.now;
        let head = h.commit(FIRST, &[(b"src", b"one")]);
        h.push(ENGINE, b"work", head).unwrap();
        h.settle();
        let run = h.domain.check_run(REPOSITORY, head, b"ci").unwrap().run;
        assert_eq!(
            h.domain.pop_fell(),
            Some(Fell {
                repository: Some(copy_of(REPOSITORY)),
                commit: Some(head),
                run: Some(run),
                ..blank(if fault == Fault::Silent { at } else { at.saturating_add(Duration::from_secs(5)) }, fault)
            })
        );
        assert_eq!(h.domain.pop_fell(), None);
    }
}

#[test]
fn check_silence_after_a_declared_wait_is_reported_once() {
    let mut configured = setup();
    configured.checks.silent = 1000;
    configured.checks.cue = Some(Cue::Declared { path: copy_of(b"ci") });
    let mut h = Harness::with(calm(), configured);
    let head = h.commit(FIRST, &[(b"ci", b"wait 2s pass")]);
    h.push(ENGINE, b"work", head).unwrap();
    assert_eq!(h.domain.pop_fell(), None);
    h.settle();
    let fell = h.domain.pop_fell().unwrap();
    assert_eq!(
        (fell.at, fell.fault, fell.commit, fell.run),
        (Time::from_nanos(2_000_000_000), Fault::Silent, Some(head), Some(1))
    );
    assert_eq!(h.domain.pop_fell(), None);
}

fn workflow() -> crate::api::Workflow {
    crate::api::Workflow {
        file: copy_of(b"gate.yml"),
        jobs: names(&[b"ci"]),
        requirements: Box::new([]),
        latency_min: Duration::from_secs(5),
        latency_max: Duration::from_secs(5),
        silent: 0,
        passes: 1000,
        cue: None,
        fails: Box::new([]),
        flaky: Box::new([]),
        select: None,
        report: None,
    }
}

fn dispatch() -> Op {
    write(Write::Dispatch { workflow: copy_of(b"gate.yml"), branch: copy_of(MAIN), inputs: Box::new([]) })
}

#[test]
fn workflow_silence_setup_and_loss_report_the_run_and_commit() {
    for fault in [Fault::Silent, Fault::SetUp, Fault::Lost] {
        let config = Config {
            setup_faults: if fault == Fault::SetUp { 1000 } else { 0 },
            lost: if fault == Fault::Lost { 1000 } else { 0 },
            ..calm()
        };
        let mut configured = setup();
        let mut workflow = workflow();
        workflow.silent = if fault == Fault::Silent { 1000 } else { 0 };
        configured.workflows = Box::new([workflow]);
        let mut h = Harness::with(config, configured);
        let at = h.env.now;
        h.ok(ENGINE, dispatch());
        h.settle();
        let Answer::Runs(runs) = h
            .inspect(&Read::Runs {
                workflow: Some(copy_of(b"gate.yml")),
                commit: None,
                number: None,
                page: 1,
                limit: 0,
            })
            .unwrap()
        else {
            panic!("workflow runs")
        };
        assert_eq!(
            h.domain.pop_fell(),
            Some(Fell {
                repository: Some(copy_of(REPOSITORY)),
                commit: Some(FIRST),
                run: Some(runs.first().unwrap().id),
                ..blank(if fault == Fault::Silent { at } else { at.saturating_add(Duration::from_secs(5)) }, fault)
            })
        );
        assert_eq!(h.domain.pop_fell(), None);
    }
}

#[test]
fn records_are_bounded_and_draining_makes_room() {
    let mut config = Config { unavailable: 1000, ..calm() };
    config.limits.observations = 2;
    let mut h = Harness::new(config);
    for _ in 0_u32..3 {
        assert_eq!(h.call(ENGINE, read(Read::Locate)), Err(Error::Unavailable));
    }
    assert_eq!(h.domain.fell_lost(), 1);
    assert_eq!(h.domain.pop_fell().unwrap().reply_to, Some(Token::new(1)));
    assert_eq!(h.domain.pop_fell().unwrap().reply_to, Some(Token::new(2)));
    assert_eq!(h.domain.pop_fell(), None);
    assert_eq!(h.call(ENGINE, read(Read::Locate)), Err(Error::Unavailable));
    assert_eq!(h.domain.pop_fell().unwrap().reply_to, Some(Token::new(4)));
    assert_eq!(h.domain.pop_fell(), None);
    assert_eq!(h.domain.fell_lost(), 1);
}

#[test]
fn no_chances_mean_no_fault_records() {
    let mut configured = setup();
    configured.workflows = Box::new([workflow()]);
    let mut h = Harness::with(calm(), configured);
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    let Answer::Created(pull) = h.open(b"work").unwrap() else { panic!("pull") };
    h.settle();
    h.ok(ENGINE, merge(pull, head));
    h.ok(ENGINE, dispatch());
    h.settle();
    assert_eq!(h.domain.pop_fell(), None);
    assert_eq!(h.domain.fell_lost(), 0);
}

#[test]
fn images_keep_neither_records_nor_losses_nor_their_bytes() {
    let mut config = Config { unavailable: 1000, ..calm() };
    config.limits.observations = 1;
    let mut h = Harness::new(config);
    for _ in 0_u32..2 {
        assert_eq!(h.call(ENGINE, read(Read::Locate)), Err(Error::Unavailable));
    }
    assert_eq!(h.domain.fell_lost(), 1);
    let image = h.domain.image().unwrap();
    let mut restored = Domain::from_image(&h.env.limits, &image).unwrap();
    assert_eq!(restored.pop_fell(), None);
    assert_eq!(restored.fell_lost(), 0);
    assert_eq!(restored.image().unwrap(), image, "faults and losses add no image bytes");
    assert!(h.domain.pop_fell().is_some());
    assert_eq!(h.domain.image().unwrap(), image, "draining never changes an image's bytes");
}

#[test]
fn words_match_the_forge_vocabulary() {
    for (fault, family, name) in [
        (Fault::Busy, "busy", "no_room"),
        (Fault::Unavailable, "busy", "unavailable"),
        (Fault::TimedOut, "busy", "timed_out"),
        (Fault::Landing, "busy", "landing"),
        (Fault::Late, "busy", "late"),
        (Fault::Limited, "busy", "limited"),
        (Fault::Crowded, "busy", "crowded"),
        (Fault::HookLate, "hooks", "late"),
        (Fault::HookLost, "hooks", "lost"),
        (Fault::HookDropped, "hooks", "dropped"),
        (Fault::Silent, "ci", "silent"),
        (Fault::Rerun, "ci", "rerun"),
        (Fault::SetUp, "ci", "set_up"),
        (Fault::Lost, "ci", "lost"),
        (Fault::RunnerWait, "ci", "runner_wait"),
        (Fault::HeadBehind, "lagging", "head_behind"),
        (Fault::BaseBehind, "lagging", "base_behind"),
        (Fault::MergeLate, "lagging", "merge_late"),
        (Fault::CloseRefused, "refusing", "close_refused"),
    ] {
        assert_eq!((fault.family(), fault.name()), (family, name), "{fault:?}");
    }
}

fn replay(seed: u64) -> Box<[Fell]> {
    let mut h = Harness::seeded(Config { unavailable: 350, timeouts: 350, late: 350, ..calm() }, setup(), seed);
    let mut records = List::with_capacity(48);
    for _ in 0_u32..24 {
        let _answer = h.call(ENGINE, read(Read::Locate));
        while let Some(fell) = h.domain.pop_fell() {
            records.push(fell).expect("at most two faults per call");
        }
    }
    records.into_boxed()
}

#[test]
fn one_seed_replays_the_records() {
    let first = replay(41);
    assert!(!first.is_empty());
    assert_eq!(replay(41), first);
}

#[test]
fn ordinary_write_and_git_keep_their_call_kinds() {
    let mut h = Harness::new(Config { unavailable: 1000, ..calm() });
    for (op, kind, commit, pull) in [
        (put(b"Home", b"body"), Called::Write, None, None),
        (fetch(Want::Commit(FIRST)), Called::Git, Some(FIRST), None),
        (read(Read::Pull { number: 2 }), Called::Read, None, Some(2)),
    ] {
        let at = h.env.now;
        assert_eq!(h.call(ENGINE, op), Err(Error::Unavailable));
        assert_eq!(
            h.domain.pop_fell(),
            Some(Fell { pull, commit, ..call_record(at, Fault::Unavailable, h.calls, kind) })
        );
        assert_eq!(h.domain.pop_fell(), None);
    }
}
