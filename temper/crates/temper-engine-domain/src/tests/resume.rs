use super::resume_fixture::root;
use crate::{Domain, Event, Limits, Request, StoreInput};
use skein_lib::{Env, List, Queue, Time, Wall};

// More than the service's two empty release passes, at one injected instant.
const RESUMES: u32 = 4;
// Initial instant, then an independent change of each injected clock.
const CLOCK_PAIRS: u32 = 3;
const WATCHER: skein_lib::Token = skein_lib::Token::new(30);
const SIGN_IN: u64 = 10;

#[test]
fn released_continuations_are_taken_in_arrival_order_at_the_watermark() {
    let (mut domain, env) = root();
    for turn in 1..=env.limits.watermark {
        let mut decision = crate::decision::Decision::decide(&mut domain.journal, &env.limits.journal);
        decision.write(&env.limits.journal, super::turn(turn, 1)).unwrap();
        crate::decision::accept_routed(
            &mut domain.journal,
            &mut domain.core.counters,
            &env.limits.journal,
            decision,
            true,
        )
        .unwrap();
    }
    let mut last = 0;
    while let Some(crate::decision::JournalOutput::Commit { number, .. }) =
        crate::commit(&mut domain.journal, &env.limits.journal)
    {
        last = number;
    }
    assert!(crate::limits::backlog(&domain) >= u64::from(env.limits.watermark));

    let older = Event::Released(crate::Released(crate::Delivery::Fleet(jig_domain_fleet::Event::TurnKept {
        run: skein_lib::Token::new(1),
        attempt: skein_lib::Token::new(1),
        turn: 1,
    })));
    crate::try_step(&mut domain, &env, older).expect("admitted continuation is always taken");
    let younger = Event::Released(crate::Released(crate::Delivery::Fleet(jig_domain_fleet::Event::Acknowledge {
        run: skein_lib::Token::new(1),
        attempt: skein_lib::Token::new(1),
    })));
    crate::try_step(&mut domain, &env, younger).unwrap();
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number: last, fill: 0 })).unwrap();
    assert!(!domain.journal.stopped());
}

#[test]
fn every_public_mutating_entry_advances_the_root_revision() {
    let (mut domain, env) = root();
    let mut out = Queue::uncapped();
    let mut revisions = List::with_capacity(9);
    revisions.push(domain.revision()).unwrap();
    crate::step(&mut domain, &env, Event::Resume);
    revisions.push(domain.revision()).unwrap();
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    revisions.push(domain.revision()).unwrap();
    crate::fire(&mut domain, &env);
    revisions.push(domain.revision()).unwrap();
    crate::release(&mut domain, &env, &mut out);
    revisions.push(domain.revision()).unwrap();
    domain.connection_limits(1, 1);
    revisions.push(domain.revision()).unwrap();
    domain.drain_facts();
    revisions.push(domain.revision()).unwrap();
    assert!(domain.pop_fact().is_none());
    revisions.push(domain.revision()).unwrap();
    domain.reclaim();
    revisions.push(domain.revision()).unwrap();
    for pair in revisions.as_slice().windows(2) {
        assert!(pair[1] > pair[0], "even empty mutations invalidate a settled owner");
    }
    assert!(out.is_empty());
}

#[test]
fn empty_release_and_resume_probes_allocate_nothing() {
    use skein_world::domain::heap::Meter;

    let (mut domain, env) = root();
    let mut out = Queue::uncapped();
    let meter = Meter::new();
    meter.start();
    for _ in 0..RESUMES {
        crate::release(&mut domain, &env, &mut out);
        crate::try_step(&mut domain, &env, Event::Resume).unwrap();
        crate::release(&mut domain, &env, &mut out);
    }
    let measured = meter.end();
    assert!(out.is_empty());
    assert_eq!((measured.held(), measured.peak()), (0, 0));
}

fn resume(domain: &mut Domain, env: &Env<Limits>) {
    for _ in 0..RESUMES {
        crate::try_step(domain, env, Event::Resume).unwrap();
    }
}

#[test]
fn settled_resumes_skip_all_empty_decisions_across_clock_pairs() {
    let (mut domain, mut env) = root();
    let mut out = Queue::uncapped();
    let mut counts = List::with_capacity(CLOCK_PAIRS);
    for (now, wall) in [(0, 0), (1, 0), (1, 1)] {
        env.now = Time::from_nanos(now);
        env.wall = Wall::from_nanos(wall);
        resume(&mut domain, &env);
        crate::release(&mut domain, &env, &mut out);
        counts.push(domain.empty_resumes).unwrap();
    }
    assert_eq!(counts.as_slice(), [0, 0, 0]);
    assert!(out.is_empty());
    assert_eq!(domain.deployment().commits, 0);
}

#[test]
fn inputs_fire_and_reclaim_preserve_derived_quiet_readiness() {
    let (mut domain, env) = root();
    resume(&mut domain, &env);
    // An inert input, fire and reclaim must not invent actionable finishing.
    let mut counts = List::with_capacity(3);
    crate::try_step(&mut domain, &env, Event::Party(crate::PartyInput::Unwatch { watcher: skein_lib::Token::new(1) }))
        .unwrap();
    resume(&mut domain, &env);
    counts.push(domain.empty_resumes).unwrap();
    crate::fire(&mut domain, &env);
    resume(&mut domain, &env);
    counts.push(domain.empty_resumes).unwrap();
    domain.reclaim();
    resume(&mut domain, &env);
    counts.push(domain.empty_resumes).unwrap();
    assert_eq!(counts.as_slice(), [0, 0, 0]);
    assert_eq!(domain.deployment().commits, 0);
}

#[test]
fn an_automatic_header_commit_cannot_mark_the_pass_quiet() {
    let (mut domain, env) = root();
    crate::fresh(&mut domain.core.counters, crate::Family::Message).unwrap();
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    assert_eq!(domain.empty_resumes, 0);
    assert_eq!(domain.settled, None);
    assert_eq!(domain.deployment().commits, 1);
}

#[test]
fn a_commit_completion_preserves_quiet_without_an_empty_pass() {
    let (mut domain, env) = root();
    crate::fresh(&mut domain.core.counters, crate::Family::Message).unwrap();
    resume(&mut domain, &env);
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, .. })) = out.pop() else { panic!("header commit") };
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    resume(&mut domain, &env);
    assert_eq!(domain.empty_resumes, 0);
    assert_eq!(domain.deployment().commits, 1);
}

#[test]
fn released_callbacks_and_restart_preserve_derived_quiet_readiness() {
    let (mut domain, env) = root();
    resume(&mut domain, &env);
    let released = crate::Released(crate::Delivery::Fleet(jig_domain_fleet::Event::Acknowledge {
        run: skein_lib::Token::new(1),
        attempt: skein_lib::Token::new(1),
    }));
    crate::try_step(&mut domain, &env, Event::Released(released)).unwrap();
    resume(&mut domain, &env);
    let after_callback = domain.empty_resumes;
    crate::try_step(&mut domain, &env, Event::Restart { pending: false }).unwrap();
    resume(&mut domain, &env);
    assert_eq!((after_callback, domain.empty_resumes), (0, 0));
}

#[test]
fn finishing_frees_retained_text_without_any_watcher() {
    let (mut domain, env) = root();
    let run = skein_lib::Token::new(1);
    let mut out = Queue::with_capacity(1);
    jig_domain_views::step(
        &mut domain.core.views,
        &env.with(crate::VIEWS),
        jig_domain_views::Event::Text {
            run,
            turn: jig_domain_views::TurnName { attempt: 1, turn: 1, retry: 0 },
            offset: 0,
            text: b"provisional".as_slice().into(),
            call: None,
        },
        &mut out,
    );
    let retained = domain.core.views.retained_runs().len();
    resume(&mut domain, &env);
    assert_eq!((retained, domain.core.views.retained_runs().len(), domain.empty_resumes), (1, 0, 1));
    assert!(out.is_empty());
}

fn watched_root() -> (Domain, Env<Limits>) {
    watch_root(root())
}

fn watch_root((mut domain, env): (Domain, Env<Limits>)) -> (Domain, Env<Limits>) {
    use jig_domain_people as people;
    let mut out = Queue::uncapped();
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.core.people.start_restore(env.wall);
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"owner".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: SIGN_IN, person: 1, expires: Wall::from_nanos(100) },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(
            &mut domain.core.people,
            &env.with(env.limits.people),
            people::Event::Restore { record },
            &mut out,
        );
    }
    let people_env = env.with(env.limits.people);
    assert!(domain.core.people.start_link());
    domain.core.people.start_sweep();
    assert!(domain.core.people.start_complete(&people_env));
    crate::try_step(
        &mut domain,
        &env,
        Event::Party(crate::PartyInput::Watch {
            watcher: WATCHER,
            sign_in: SIGN_IN,
            subject: jig_domain_core::documents::Watched::Frame,
            window: jig_domain_core::documents::Window { rows: 1, after: None },
        }),
    )
    .unwrap();
    let mut released = Queue::uncapped();
    crate::release(&mut domain, &env, &mut released);
    crate::release(&mut domain, &env, &mut released);
    crate::try_step(&mut domain, &env, Event::Party(crate::PartyInput::Delivered { watcher: WATCHER })).unwrap();
    resume(&mut domain, &env);
    (domain, env)
}

#[test]
fn a_projection_feed_then_committed_resume_delivers_the_watch() {
    use jig_domain_tasks as tasks;
    let (mut domain, env) = watched_root();
    let goal = crate::fresh(&mut domain.core.counters, crate::Family::Task).unwrap();
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut authority = jig_domain_core::task_authority(&domain.core.authority.policy(1).unwrap().ceiling);
    authority.budget.spend = 100;
    for event in [
        tasks::Event::OpenPeriod {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(u64::MAX - 2)),
            project: 1,
            period: 1,
            budget: 100,
        },
        tasks::Event::Make {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(u64::MAX - 3)),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([tasks::New {
                number: goal,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Agent { charter: 1 },
                llm: None,
                spec: tasks::Spec {
                    job: None,
                    asks: false,
                    words: b"goal".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                deadline: tasks::Deadline::Named,
                authority,
                numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: Some(0),
            }]),
        },
    ] {
        let requests = jig_domain_core::step(
            &mut domain.core,
            &crate::limits::environment_core(&env),
            jig_domain_core::Event::Tasks(event),
        );
        crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
    }
    crate::route::route_into(&mut domain, &env, &mut decision);
    let mut immediate = Queue::uncapped();
    crate::domain::close(&mut domain, &env, decision, &mut immediate);
    assert!(immediate.is_empty());
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, writes })) = out.pop() else {
        panic!("projection and period commit")
    };
    let mut projection_written = false;
    for write in &writes {
        projection_written |= write.key()
            == crate::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Projection(
                jig_domain_core::ProjectionKey::Goal(1),
            )));
    }
    let before_committed = out.len();
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Party(crate::PartyRequest::Deliver(crate::Delivery::Document { watcher, .. }))) = out.pop()
    else {
        panic!("deferred watch document: {out:?}")
    };
    assert!(projection_written);
    assert_eq!((before_committed, watcher, domain.empty_resumes), (0, WATCHER, 0));
    assert_eq!(domain.deployment().commits, 1);
    assert!(out.is_empty());
}

#[test]
fn advancing_wall_time_ends_a_settled_watch() {
    let (mut domain, mut env) = watched_root();
    env.wall = Wall::from_nanos(100);
    crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(
        out.pop(),
        Some(Request::Party(crate::PartyRequest::Deliver(crate::Delivery::WatchEnded {
            watcher: WATCHER,
            why: jig_domain_views::End::SignedOut,
        })))
    );
    assert!(out.is_empty());
    assert_eq!(domain.core.people.sign_in_person(SIGN_IN), Some(1));
    assert_eq!(domain.deployment().commits, 0);
    assert_eq!(domain.next_deadline(&env), Some(env.now));
}

#[test]
fn stop_from_settled_still_releases_the_drain_terminal() {
    let (mut domain, env) = root();
    resume(&mut domain, &env);
    crate::try_step(&mut domain, &env, Event::Stop).unwrap();
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(out.pop(), Some(Request::Drained));
    assert!(out.is_empty());
    assert_eq!(domain.settled, None);
}

#[test]
#[expect(clippy::too_many_lines, reason = "the exhaustive output rejection includes public forge requests")]
fn stopping_continues_projection_deferred_finishing_without_new_commits() {
    use jig_domain_core as core;
    use jig_domain_tasks as tasks;
    let (mut domain, env) = watched_root();
    let goal = crate::fresh(&mut domain.core.counters, crate::Family::Task).unwrap();
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    let core_env = crate::limits::environment_core(&env);
    let mut authority = core::task_authority(&domain.core.authority.policy(1).unwrap().ceiling);
    authority.budget.spend = 100;
    for event in [
        tasks::Event::Restored,
        tasks::Event::OpenPeriod {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(u64::MAX - 2)),
            project: 1,
            period: 1,
            budget: 100,
        },
        tasks::Event::Make {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(u64::MAX - 3)),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([tasks::New {
                number: goal,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Agent { charter: 1 },
                llm: None,
                spec: tasks::Spec {
                    job: None,
                    asks: false,
                    words: b"queued goal before Stop".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                deadline: tasks::Deadline::Named,
                authority,
                numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: Some(0),
            }]),
        },
    ] {
        let requests = core::step(&mut domain.core, &core_env, core::Event::Tasks(event));
        crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
    }
    assert!(domain.core.tasks.task(goal).is_some(), "real goal before Stop");
    let requests = core::step(&mut domain.core, &core_env, core::Event::Stopping);
    crate::route::route_core_requests(&mut domain, &env, &mut decision, requests);
    crate::route::route_into(&mut domain, &env, &mut decision);
    let mut out = Queue::uncapped();
    crate::domain::close(&mut domain, &env, decision, &mut out);
    assert!(domain.core.finish_owed(), "projection defers watch finishing");
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, .. })) = out.pop() else {
        panic!("goal and projection commit")
    };
    assert!(out.is_empty(), "no watch or drain before the previous commit");
    let commits = domain.deployment().commits;
    // Even finishing before Committed must keep both terminals behind it.
    resume(&mut domain, &env);
    crate::release(&mut domain, &env, &mut out);
    assert!(out.is_empty(), "finishing remains behind the last commit");
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    let mut endings = 0_u32;
    let mut drains = 0_u32;
    for _ in 0_u32..12 {
        crate::try_step(&mut domain, &env, Event::Resume).unwrap();
        crate::release(&mut domain, &env, &mut out);
        crate::release(&mut domain, &env, &mut out);
        while let Some(request) = out.pop() {
            match request {
                Request::Party(crate::PartyRequest::Deliver(crate::Delivery::WatchEnded { watcher, why })) => {
                    assert_eq!((watcher, why), (WATCHER, jig_domain_views::End::Stopping));
                    endings += 1;
                }
                Request::Drained => drains += 1,
                Request::Party(crate::PartyRequest::Deliver(crate::Delivery::WebReply { .. })) => {}
                other @ (Request::Setup { .. }
                | Request::ForgeKept(_)
                | Request::ForgeGone { .. }
                | Request::BoxGroup { .. }
                | Request::PlaygroundActs { .. }
                | Request::ResetAct { .. }
                | Request::BoxRemove { .. }
                | Request::Connect { .. }
                | Request::Replace { .. }
                | Request::Manifest { .. }
                | Request::AdoptAccess { .. }
                | Request::Reachable { .. }
                | Request::Install { .. }
                | Request::Accept { .. }
                | Request::EraseCarried { .. }
                | Request::MakingCopied { .. }
                | Request::SetUp { .. }
                | Request::Bound { .. }
                | Request::Store(_)
                | Request::Worker(_)
                | Request::Party(_)
                | Request::Forge { .. }
                | Request::Account(_)
                | Request::ToChild(_)
                | Request::Busy(_)
                | Request::Unsettled(_)
                | Request::Stop) => panic!("unexpected stopped continuation: {other:?}"),
            }
        }
    }
    assert_eq!((endings, drains), (1, 1));
    assert_eq!(domain.deployment().commits, commits, "zero new commits");
    assert!(!domain.core.finish_owed());
    resume(&mut domain, &env);
    crate::release(&mut domain, &env, &mut out);
    assert!(out.is_empty(), "repeated stopped Resume stays quiet");
}

#[test]
fn people_deadlines_use_both_injected_clocks_and_keep_the_original_alarm() {
    let (domain, mut env) = watched_root();
    for (now, wall, expected) in [(1, 50, 51), (2, 0, 100), (3, 100, 3), (100, 0, 100)] {
        env.now = Time::from_nanos(now);
        env.wall = Wall::from_nanos(wall);
        assert_eq!(domain.next_deadline(&env), Some(Time::from_nanos(expected)));
    }
}

#[test]
fn ordinary_root_fire_expires_sign_ins_once_by_either_clock() {
    for (now, wall) in [(1, 100), (100, 0)] {
        let (mut domain, mut env) = watched_root();
        env.now = Time::from_nanos(now);
        env.wall = Wall::from_nanos(wall);
        assert_eq!(domain.core.people.sign_in_person(SIGN_IN), Some(1));
        assert_eq!(domain.next_deadline(&env), Some(env.now));
        crate::fire(&mut domain, &env);
        assert_eq!(domain.core.people.sign_in_person(SIGN_IN), None);
        assert_eq!(domain.core.people.next_wall_expiry(), None);
        assert_eq!(domain.deployment().commits, 1);
        crate::fire(&mut domain, &env);
        resume(&mut domain, &env);
        assert_eq!(domain.deployment().commits, 1);
    }
}

#[test]
fn refused_people_timer_stays_due_without_child_mutation_until_room_returns() {
    let (mut domain, mut env) = watched_root();
    // Header decisions bring the backlog exactly to the watermark.
    for _ in 0..env.limits.watermark {
        crate::fresh(&mut domain.core.counters, crate::Family::Message).unwrap();
        crate::try_step(&mut domain, &env, Event::Resume).unwrap();
    }
    assert!(!crate::domain::admits(&domain, &env.limits));
    env.now = Time::from_nanos(1);
    env.wall = Wall::from_nanos(100);
    let before = domain.deployment();
    let empty = domain.empty_resumes;
    crate::fire(&mut domain, &env);
    resume(&mut domain, &env);
    assert_eq!(domain.deployment(), before);
    assert_eq!(domain.empty_resumes, empty);
    assert_eq!(domain.core.people.sign_in_person(SIGN_IN), Some(1));
    assert_eq!(domain.next_deadline(&env), Some(env.now));
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, .. })) = out.pop() else {
        panic!("first admitted commit")
    };
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    assert!(crate::domain::admits(&domain, &env.limits));
    crate::fire(&mut domain, &env);
    assert_eq!(domain.core.people.sign_in_person(SIGN_IN), None);
    assert_eq!(domain.core.people.next_wall_expiry(), None);
    assert_eq!(domain.deployment().commits, before.commits + 1);
}

#[test]
fn authenticated_read_bytes_are_configured_and_validated() {
    let (_, env) = root();
    let mut limits = env.limits;
    limits.loads.read_bytes = 65_536;
    assert!(crate::valid(&limits));
    limits.loads.read_bytes += 65_536;
    assert_eq!(crate::limits::core_limits(&limits).load_bytes, 131_072);
    assert!(crate::valid(&limits));
    limits.loads.read_bytes = 0;
    assert!(!crate::valid(&limits));
}

#[test]
fn startup_requires_room_for_the_cancellation_reason() {
    let mut limits = super::resume_fixture::limits();
    limits.forge.client.op_bytes = limits.tasks.result_bytes;
    assert!(crate::valid(&limits), "the exact reason bound fits");
    limits.forge.client.op_bytes -= 1;
    assert!(!crate::valid(&limits), "a reason past the forge byte bound is refused");
}

fn cold() -> (Domain, Env<Limits>) {
    let limits = super::resume_fixture::limits();
    let domain = Domain::new(super::resume_fixture::config(7), &limits).unwrap();
    (domain, Env { now: Time::ZERO, wall: Wall::EPOCH, limits })
}

const START_PHASES: [jig_domain_core::start::Phase; 11] = {
    use jig_domain_core::start::Phase;
    [
        Phase::LoadCore,
        Phase::LoadInbox,
        Phase::RestoreConnector { connector: 0 },
        Phase::Link,
        Phase::Sweep,
        Phase::Switch,
        Phase::Wires,
        Phase::AdoptRuns,
        Phase::ReadAfresh { connector: 0 },
        Phase::SettleOutbox { connector: 0 },
        Phase::Open,
    ]
};

#[test]
fn an_empty_store_opens_through_every_start_phase_in_order() {
    let (mut domain, env) = cold();
    let mut store = super::resume_fixture::StartStore::new();
    crate::step(&mut domain, &env, Event::Restart { pending: false });
    store.run(&mut domain, &env, false);
    let phases = start_phases(&domain);
    assert_eq!(phases.as_slice(), START_PHASES);
    assert!(domain.ready());
    assert_eq!((store.switches, store.wires), (0, 1));
}

fn repository() -> temper_engine_domain_forge::Repository {
    use temper_engine_domain_forge as forge;
    forge::Repository {
        contexts: Box::new([]),
        workflows: Box::default(),
        project: 1,
        home: false,
        provider: skein_forge::api::Repository { forge: 1, repository: 1 },
        host: b"forge.invalid".as_slice().into(),
        owner: b"owner".as_slice().into(),
        name: b"repo".as_slice().into(),
        prefix: Box::new([]),
        role: forge::Role::Context,
        ci: false,
        checks: Box::new([]),
        kinds: forge::Kinds {
            read: false,
            push: false,
            open: false,
            land: false,
            review: false,
            status: false,
            comment: false,
            issue: false,
            branch: false,
        },
        protection: forge::Protection::Unknown,
        settings: skein_forge::api::Settings {
            default_branch: b"main".as_slice().into(),
            merge: true,
            squash: false,
            rebase: false,
        },
    }
}

fn restored_start_store() -> (super::resume_fixture::StartStore, Env<Limits>) {
    let (mut first, env) = cold();
    let mut store = super::resume_fixture::StartStore::new();
    crate::step(&mut first, &env, Event::Restart { pending: false });
    store.run(&mut first, &env, false);
    (store, env)
}

fn stored_deployment(store: &mut super::resume_fixture::StartStore) -> &mut jig_domain_core::Deployment {
    let header = crate::Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment));
    let crate::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(deployment))) =
        store.rows.get_mut(&header).unwrap()
    else {
        panic!("deployment row");
    };
    deployment
}

#[test]
fn a_project_live_task_and_forge_repository_open_together() {
    let (mut store, env) = restored_start_store();
    let (source, _, goal) = super::projection_settled::live_goal(None);
    let task = source.tasks().task(goal).unwrap().clone();
    let funding = *source.tasks().funding(task.funder).unwrap();
    let repo = repository();
    for row in [
        crate::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(task)))),
        crate::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ledger(funding))),
        crate::Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::Repository(repo.clone())) },
    ] {
        store.rows.insert(row.key(), row);
    }
    stored_deployment(&mut store).tasks = goal;
    let (mut domain, _) = cold();
    crate::step(&mut domain, &env, Event::Restart { pending: true });
    store.run(&mut domain, &env, false);
    assert!(domain.ready());
    assert!(domain.tasks().task(goal).is_some());
    assert_eq!(domain.forge.repository(repo.provider), Some(&repo));
    assert_eq!(domain.core.projects.as_slice(), [1]);
    assert_eq!((store.switches, store.wires), (1, 2));
}

#[test]
fn a_restored_forge_link_follows_the_deployments_sole_owner() {
    use jig_domain_people as people;
    use temper_engine_domain_forge as forge;
    const P: u64 = 1;
    const Q: u64 = 2;
    const FORGE: u16 = 1;
    const LINK: Option<forge::Link> = Some(forge::Link { user: 7, person: P });
    const P_OWNER: people::Holding = people::Holding { person: P, role: people::Role::Owner };
    const Q_OWNER: people::Holding = people::Holding { person: Q, role: people::Role::Owner };
    const Q_MEMBER: people::Holding = people::Holding { person: Q, role: people::Role::Member };
    let (mut restored, env) = restored_start_store();
    stored_deployment(&mut restored).people = Q;
    for row in [
        crate::Record::Core(jig_domain_core::Record::People(people::Stored::Person {
            number: Q,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Q.to_be_bytes().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        })),
        crate::Record::Forge {
            row: Box::new(forge::Stored::Forge(forge::ForgeRecord {
                number: FORGE,
                kind: forge::Kind::Forgejo,
                mode: forge::Mode::Owned,
                state: forge::ForgeState::Kept,
                place: forge::Place {
                    api: forge::Address { host: b"forge.invalid".as_slice().into(), port: 80, tls: None },
                    git: Box::new([]),
                    web: None,
                },
                git_program: None,
                read_only: false,
                bot: forge::Bot { login: Box::new([]), name: Box::new([]), email: Box::new([]) },
                asked: false,
                first: None,
                person: LINK,
            })),
        },
    ] {
        restored.rows.insert(row.key(), row);
    }
    let mut failures = List::uncapped();
    for (name, holdings, under_box, expected) in [
        ("P owner", Box::from([P_OWNER].as_slice()), true, LINK),
        ("P and Q owners", Box::from([P_OWNER, Q_OWNER].as_slice()), true, None),
        ("Q owner", Box::from([Q_OWNER].as_slice()), true, None),
        ("P owner and Q member", Box::from([P_OWNER, Q_MEMBER].as_slice()), true, LINK),
        ("P owner without box", Box::from([P_OWNER].as_slice()), false, None),
    ] {
        let mut store = super::resume_fixture::StartStore::new();
        store.rows = restored.rows.clone();
        let row = crate::Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
            project: people::DEPLOYMENT,
            holdings,
        }));
        store.rows.insert(row.key(), row);
        let mut config = super::resume_fixture::config(7);
        config.under_box = under_box;
        config.forges.first_mut().unwrap().mode = forge::Mode::Owned;
        let mut domain = Domain::new(config, &env.limits).unwrap();
        crate::step(&mut domain, &env, Event::Restart { pending: false });
        store.run(&mut domain, &env, false);
        let link = domain.forge.link(FORGE);
        if link != expected {
            failures.push((name, link, expected)).unwrap();
        }
    }
    assert!(failures.is_empty(), "restored links: {:?}", failures.as_slice());
}

#[test]
fn a_crash_after_wires_commit_drops_its_delivery_and_cold_loads_the_same_state() {
    let (mut domain, env) = cold();
    let mut store = super::resume_fixture::StartStore::new();
    crate::step(&mut domain, &env, Event::Restart { pending: false });
    store.run(&mut domain, &env, true);
    assert_eq!(domain.start_phase(), Some(jig_domain_core::start::Phase::Wires));
    assert!(!domain.ready());
    let projects = domain.core.projects.clone();
    let deployment = domain.deployment();
    drop(domain);
    let (mut domain, _) = cold();
    crate::step(&mut domain, &env, Event::Restart { pending: false });
    store.run(&mut domain, &env, false);
    assert!(domain.ready());
    assert_eq!(domain.core.projects, projects);
    assert_eq!(domain.deployment().projects, deployment.projects);
    assert_eq!(domain.deployment().people, deployment.people);
    assert_eq!(domain.deployment().tasks, deployment.tasks);
    assert_eq!(store.wires, 2);
}

/// The phase fixture supplies completed transport work; the root serves Switch and Wires.
fn at_switch(pending: bool) -> (Domain, Env<Limits>) {
    at_phase(pending, jig_domain_core::start::Phase::Switch)
}

pub(super) fn at_phase(pending: bool, target: jig_domain_core::start::Phase) -> (Domain, Env<Limits>) {
    at_phase_with_limits(pending, target, &super::resume_fixture::limits())
}

pub(super) fn at_phase_with_limits(
    pending: bool,
    target: jig_domain_core::start::Phase,
    limits: &Limits,
) -> (Domain, Env<Limits>) {
    use jig_domain_core::start::{Action, Phase, SweepInput};
    let mut domain = Domain::new(super::resume_fixture::config(7), limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    domain.start_wall = Some(env.wall);
    domain.start_pending = pending;
    domain.core_header_loaded = true;
    domain.forge.restore_begin(env.wall);
    let core_env = crate::limits::environment_core(&env);
    let mut action = domain.core.start_begin(pending, env.wall);
    for _ in 0_u32..20 {
        let mut output = Queue::uncapped();
        action = match action {
            Action::Step(phase) if phase == target => {
                domain.core.counters = crate::Counters::new(domain.deployment());
                return (domain, env);
            }
            Action::Step(Phase::Sweep) => domain.core.start_sweep(&core_env, SweepInput::Finished, &mut output),
            Action::Step(phase) => domain.core.start_done(&core_env, phase, &mut output),
            Action::Commit(continuation) => domain.core.start_resume(&core_env, continuation, &mut output),
            Action::Idle | Action::Refused(_) => panic!("fixture stopped before Switch"),
        };
    }
    panic!("fixture did not reach Switch");
}

fn serve_switch(domain: &mut Domain, env: &Env<Limits>) {
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::restart::start_action(
        domain,
        env,
        &mut decision,
        jig_domain_core::start::Action::Step(jig_domain_core::start::Phase::Switch),
    );
    crate::route::route_into(domain, env, &mut decision);
    let mut immediate = Queue::uncapped();
    crate::domain::close(domain, env, decision, &mut immediate);
    assert!(immediate.is_empty());
}

fn wires_commit(domain: &mut Domain, env: &Env<Limits>) -> u64 {
    let mut out = Queue::uncapped();
    crate::release(domain, env, &mut out);
    let Some(Request::Store(crate::StoreRequest::Commit { number, writes })) = out.pop() else {
        panic!("wires commit");
    };
    assert!(writes.contains(&crate::Write::Wires));
    assert!(out.is_empty());
    crate::release(domain, env, &mut out);
    assert!(out.is_empty(), "continuation waits for its own commit");
    assert!(!domain.ready());
    number
}

#[test]
fn a_start_without_pending_still_takes_switch_and_commits_wires_with_its_continuation() {
    use jig_domain_core::start::Phase;
    let (mut domain, env) = at_switch(false);
    serve_switch(&mut domain, &env);
    let phases = start_phases(&domain);
    assert_eq!(phases.as_slice(), [Phase::Switch, Phase::Wires]);
    let number = wires_commit(&mut domain, &env);
    assert_eq!(domain.start_phase(), Some(Phase::Wires));
    crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 23 }));
    assert_eq!(domain.startup_fill, Some(23));
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::ToChild(crate::Released(crate::Delivery::Start(_)))) = out.pop() else {
        panic!("held start continuation");
    };
    assert!(out.is_empty());
}

#[test]
fn a_pending_start_waits_for_switched_before_committing_wires() {
    use jig_domain_core::start::Phase;
    let (mut domain, env) = at_switch(true);
    serve_switch(&mut domain, &env);
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    assert_eq!(out.pop(), Some(Request::Store(crate::StoreRequest::Switch)));
    assert!(out.is_empty());
    assert_eq!(domain.start_phase(), Some(Phase::Switch));
    crate::step(&mut domain, &env, Event::Store(StoreInput::Switched));
    let _number = wires_commit(&mut domain, &env);
    assert_eq!(domain.start_phase(), Some(Phase::Wires));
}

#[test]
#[should_panic(expected = "stale or out-of-order start continuation")]
fn a_stale_start_delivery_asserts() {
    let (mut domain, env) = at_switch(false);
    serve_switch(&mut domain, &env);
    let number = wires_commit(&mut domain, &env);
    crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 }));
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::ToChild(crate::Released(crate::Delivery::Start(continuation)))) = out.pop() else {
        panic!("start delivery");
    };
    let mut output = Queue::uncapped();
    let _action = domain.core.start_resume(&crate::limits::environment_core(&env), continuation.clone(), &mut output);
    crate::step(&mut domain, &env, Event::Released(crate::Released(crate::Delivery::Start(continuation))));
}

#[test]
fn firing_before_start_and_before_wires_does_not_consume_or_open_the_start() {
    let (mut domain, env) = cold();
    crate::fire(&mut domain, &env);
    crate::step(&mut domain, &env, Event::Restart { pending: false });
    crate::fire(&mut domain, &env);
    assert_eq!(domain.start_phase(), Some(jig_domain_core::start::Phase::LoadCore));
    assert!(!domain.ready());
    assert_eq!(domain.next_deadline(&env), None);
}

fn take_start(domain: &mut Domain, env: &Env<Limits>) -> crate::Released {
    let mut out = Queue::uncapped();
    for _ in 0_u32..3 {
        crate::release(domain, env, &mut out);
        if let Some(request) = out.pop() {
            let Request::ToChild(released) = request else {
                panic!("start delivery: {request:?}");
            };
            assert!(out.is_empty());
            return released;
        }
    }
    panic!("durable start delivery");
}

#[test]
fn the_sweep_keeps_the_rest_of_its_page_until_each_erase_continuation_is_released() {
    use jig_domain_core::start::Phase;
    use jig_domain_people as people;
    let (mut domain, env) = at_phase(false, Phase::Sweep);
    domain.startup_range = Some(crate::Range::Core(crate::CoreRange::Sweep));
    let rows = Box::new([
        crate::Record::Core(jig_domain_core::Record::People(people::Stored::SignIn {
            number: 1,
            person: 999,
            expires: Wall::EPOCH,
        })),
        crate::Record::Core(jig_domain_core::Record::People(people::Stored::SignIn {
            number: 2,
            person: 999,
            expires: Wall::EPOCH,
        })),
    ]);
    let decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut immediate = Queue::uncapped();
    crate::restart::startup_page(&mut domain, &env, rows, None, decision, &mut immediate);
    assert!(immediate.is_empty());
    assert_eq!(domain.sweep_rows.len(), 1);
    for sign_in in [1, 2] {
        let mut out = Queue::uncapped();
        crate::release(&mut domain, &env, &mut out);
        let Some(Request::Store(crate::StoreRequest::Commit { number, writes })) = out.pop() else {
            panic!("sweep erase commit");
        };
        let erased = crate::Write::Erase(crate::Key::Core(jig_domain_core::Key::People(people::Key::SignIn(sign_in))));
        assert!(writes.contains(&erased));
        assert!(!writes.contains(&crate::Write::Wires));
        assert!(out.is_empty());
        crate::release(&mut domain, &env, &mut out);
        assert!(out.is_empty());
        crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 }));
        let delivery = take_start(&mut domain, &env);
        crate::step(&mut domain, &env, Event::Released(delivery));
        assert_eq!(domain.sweep_rows.len(), 0);
    }
    assert_eq!(domain.start_phase(), Some(Phase::Wires));
    assert!(!domain.ready());
    let _number = wires_commit(&mut domain, &env);
}

fn start_phases(domain: &Domain) -> List<jig_domain_core::start::Phase> {
    let mut phases = List::with_capacity(domain.start_phases.len());
    for phase in &domain.start_phases {
        phases.push(*phase).unwrap();
    }
    phases
}

fn accepted_statement() -> crate::WorkerStatement {
    crate::WorkerStatement {
        wires: Box::new([crate::OpenedWire {
            name: "example-wire".into(),
            lowest: 3,
            layouts: Box::new([[4; 32], [5; 32]]),
        }]),
        output: 4096,
        terms: Box::new([crate::OpenedTerm { kind: 12, largest: 34 }]),
    }
}

#[test]
fn accepted_openings_commit_separately_after_the_starts_wires_are_durable() {
    for early in [true, false] {
        let (mut domain, mut env) = at_switch(false);
        let mut restored = Queue::uncapped();
        temper_engine_domain_forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            temper_engine_domain_forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
            &mut restored,
        );
        let heard = Wall::from_nanos(123_456_789);
        env.wall = heard;
        if early {
            crate::try_step(
                &mut domain,
                &env,
                Event::Worker(crate::WorkerInput::Opened { worker: "w1".into(), statement: accepted_statement() }),
            )
            .unwrap();
            crate::step(&mut domain, &env, Event::Resume);
            let mut out = Queue::uncapped();
            crate::release(&mut domain, &env, &mut out);
            assert!(out.is_empty(), "opening before wires waits");
        }
        serve_switch(&mut domain, &env);
        let wires = wires_commit(&mut domain, &env);
        crate::step(&mut domain, &env, Event::Resume);
        let mut out = Queue::uncapped();
        crate::release(&mut domain, &env, &mut out);
        assert!(out.is_empty(), "opening waits for the wires' durability, not just their issue");
        crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number: wires, fill: 0 }));
        crate::release(&mut domain, &env, &mut out);
        crate::release(&mut domain, &env, &mut out);
        let Some(Request::ToChild(released)) = out.pop() else { panic!("wires' durable continuation") };
        assert!(out.is_empty());
        crate::step(&mut domain, &env, Event::Released(released));
        if !early {
            crate::try_step(
                &mut domain,
                &env,
                Event::Worker(crate::WorkerInput::Opened { worker: "w1".into(), statement: accepted_statement() }),
            )
            .unwrap();
        }
        // A second accepted opening, even for the same worker, must not coalesce with the first.
        crate::try_step(
            &mut domain,
            &env,
            Event::Worker(crate::WorkerInput::Opened { worker: "w1".into(), statement: accepted_statement() }),
        )
        .unwrap();
        env.wall = Wall::from_nanos(987_654_321);
        let expected = crate::Write::Opening(crate::WorkerOpening {
            worker: "w1".into(),
            statement: accepted_statement(),
            at: heard,
        });
        let mut commits = List::with_capacity(2);
        for _ in 0_u32..16 {
            crate::step(&mut domain, &env, Event::Resume);
            crate::release(&mut domain, &env, &mut out);
            while let Some(request) = out.pop() {
                if let Request::Store(crate::StoreRequest::Commit { number, writes }) = request {
                    if writes.contains(&expected) {
                        assert!(number > wires);
                        assert!(!writes.contains(&crate::Write::Wires));
                        assert_eq!(writes.len(), 2, "only the opening and deployment header");
                        commits.push(number).unwrap();
                    }
                    crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 }));
                } else if let Request::ToChild(released) = request {
                    crate::step(&mut domain, &env, Event::Released(released));
                } else {
                    panic!("unexpected opening output {request:?}");
                }
            }
            if commits.len() == 2 {
                break;
            }
        }
        assert_eq!(
            commits.len(),
            2,
            "each accepted opening commits its own statement and heard wall time; early={early}"
        );
        assert!(commits.get(0).unwrap() < commits.get(1).unwrap());
    }
}
