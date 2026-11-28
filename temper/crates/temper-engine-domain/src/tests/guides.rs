//! A durable answer's report is part of its attempt's terminal decision
//! (temper/docs/design/domain/agent/project-charters.md, 6).
use super::projection_settled::live_task;
use crate::{Domain, Event, Limits, Parts, Request, StoreRequest, WorkerInput, Write};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_hub_link as hub;
use jig_domain_link as link;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Queue, ReplyTo, Token};

// Start at the result decoder's boundary: the fleet has already fenced and
// handed out this answer, so the root holds it until the decoder returns.
const ATTEMPT: u64 = 7;
const CHANNEL: Token = Token::new(8);

fn claimed() -> (Domain, Env<Limits>, u64) {
    let (mut root, env, task) = live_task(None, tasks::Executor::Agent { charter: 1 });
    let mut tasks_out = Queue::uncapped();
    for event in [
        tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(task)), task },
        tasks::Event::Claim {
            sources: Some(tasks::ClaimSources {
                build: "test".into(),
                settings: 1,
                source: Box::new([]),
                guide_limit: 1,
            }),
            fresh: None,
            session_end: None,
            offered: tasks::Reasons::ALL,
            reasons: tasks::Reasons::FIRST,
            account: None,
            reply_to: ReplyTo::new(Token::new(task)),
            task,
            attempt: ATTEMPT,
            resumed: false,
            budget: 1,
            writes: Box::new([]),
        },
    ] {
        tasks::step(&mut root.engine.core_mut().tasks, &env.with(env.limits.tasks), event, &mut tasks_out);
    }
    let open = root.engine.core().tasks.task(task).unwrap().open_attempt.as_ref().expect("an open attempt");
    assert_eq!(open.sources.as_ref().unwrap().guides, None);
    root.engine
        .core_mut()
        .proofs
        .insert(
            task,
            core::RunProof {
                kept_part: 0,
                worker: Some(0),
                task,
                attempt: ATTEMPT,
                conversation: 1,
                endpoint: None,
                credentials: Box::new([]),
                accounts: Box::new([]),
                placement: Some(CHANNEL),
                account_failure: None,
                account_terminal: false,
                window: None,
                crossed: false,
                host: jig_domain_fleet::HostKind::Worker,
                offered: None,
                outstanding: Box::new([]),
                turn: None,
                terminal: None,
            },
        )
        .unwrap();
    let run = Token::new(task);
    let attempt = Token::new(ATTEMPT);
    let mut out = Queue::uncapped();
    for event in [
        fleet::Event::Adopt {
            reply_to: ReplyTo::new(Token::new(task)),
            run,
            attempt,
            worker: Some(0),
            kept: 0,
            kept_part: 0,
            kind: fleet::HostKind::Worker,
            worked: false,
            holds: Box::new([]),
            grants: Box::new([]),
        },
        fleet::Event::Hello {
            channel: CHANNEL,
            worker: 0,
            hello: link::Hello {
                fresh: false,
                reads_usage: true,
                jobs: Box::new([]),
                bounds: jig_test_profile::fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([link::Hosting { run, attempt, phase: link::Phase::Answered }]),
            },
        },
    ] {
        fleet::step(&mut root.engine.core_mut().fleet, &env.with(env.limits.fleet), event, &mut out);
    }
    assert!(root.engine.core().fleet.owns_attempt(CHANNEL, run, attempt), "worker adopted the hosted answer: {out:?}");
    (root, env, task)
}

fn guides() -> Box<[tasks::Guide]> {
    Box::new([tasks::Guide {
        repository: "acme/shop".into(),
        commit: "9f8e7d6c5b4a39000ff1e2d3c4b5a69788776655".into(),
        path: "AGENTS.md".into(),
        digest: [23; 32],
        bytes: 3201,
        whole: true,
    }])
}

fn result(root: &mut Domain, env: &Env<Limits>, task: u64) -> Option<tasks::AttemptRow> {
    let end = tasks::End::Parked;
    let payload = root.hub_link.retain(crate::Payload::Answer {
        task,
        attempt: ATTEMPT,
        cumulative: 0,
        read: None,
        refused: Box::new([]),
        end: hub::Ending::Link(host::Ending::Ended {
            outcome: Box::new([]),
            work: host::Work { left: Some(Token::new(1)), saved: None },
        }),
    });
    root.answer_parts.insert(payload, Parts::none()).unwrap();
    crate::try_step(root, env, Event::Worker(WorkerInput::ResultRead { payload, end })).unwrap();
    attempt_write(root, env)
}

fn attempt_write(root: &mut Domain, env: &Env<Limits>) -> Option<tasks::AttemptRow> {
    let mut out = Queue::uncapped();
    crate::release(root, env, &mut out);
    let mut recorded = None;
    for request in out.into_boxed() {
        if let Request::Store(StoreRequest::Commit { number, writes }) = request {
            for write in writes {
                if let Write::Save(crate::Record::Core(core::Record::Tasks(tasks::Stored::Attempt(row)))) = write {
                    assert!(recorded.replace(row).is_none(), "the decision writes one row for the attempt");
                }
            }
            crate::try_step(root, env, Event::Store(crate::StoreInput::Committed { number, fill: 0 })).unwrap();
        }
    }
    recorded
}

fn send_guides(root: &mut Domain, env: &Env<Limits>, task: u64, channel: Token) -> Option<tasks::AttemptRow> {
    crate::try_step(
        root,
        env,
        Event::Worker(WorkerInput::Guides {
            channel,
            run: Token::new(task),
            attempt: Token::new(ATTEMPT),
            guides: guides(),
        }),
    )
    .unwrap();
    attempt_write(root, env)
}

#[test]
fn a_guides_record_reaches_its_open_attempt_before_the_answer_closes_it() {
    let (mut root, env, task) = claimed();
    assert!(send_guides(&mut root, &env, task, Token::new(9)).is_none(), "a different channel cannot report");
    let row = send_guides(&mut root, &env, task, CHANNEL).expect("report kept on open attempt");
    assert_eq!((row.task, row.attempt), (task, ATTEMPT));
    assert!(row.ended.is_none());
    assert_eq!(row.sources.unwrap().guides, Some(guides()));
    let row = result(&mut root, &env, task).expect("the attempt closed");
    assert!(row.ended.is_some());
    assert_eq!(row.sources.unwrap().guides, Some(guides()));
}

#[test]
fn a_guides_replay_after_the_attempt_closed_is_dropped() {
    let (mut root, env, task) = claimed();
    let row = result(&mut root, &env, task).expect("the first answer closed the attempt");
    assert!(row.ended.is_some());
    assert_eq!(row.sources.unwrap().guides, None);
    // Simulate a replay admitted by the fleet before it sees the terminal
    // acknowledgement. The tasks guard must still discard this record.
    assert!(root.engine.core().fleet.owns_attempt(CHANNEL, Token::new(task), Token::new(ATTEMPT)));
    assert!(send_guides(&mut root, &env, task, CHANNEL).is_none(), "the late report cannot write an attempt row");
}
