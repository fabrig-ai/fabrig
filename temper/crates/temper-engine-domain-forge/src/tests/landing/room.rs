//! Landing writes join the shared line without keeping their decision (forge/room.md, 3–4).
use super::*;
use crate::room;
use skein_forge::api::{self, Answer, Error, Op, Read, Write};

const TASK: u64 = 61;
const OTHER: u64 = 62;
const FRESH: u64 = 101;
const KEY: policy::Key = [7; 32];

fn full(kind: &str) -> Domain {
    let mut d = landing_fixture();
    fixtures::connect(&mut d);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state =
        temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    let mut row = merge_landing_row(false);
    if kind == "open" {
        row.landing.phase = policy::Phase::Preparing;
        row.landing.round.pull = None;
    }
    if kind == "dispatch" || kind == "post" || kind == "rerun" {
        let after = d.after_landing(5).unwrap();
        assert!(d.project_landing(
            5,
            temper_engine_domain_forge_change::Way::Turn,
            Box::new([RepositoryGates {
                repository: REPO,
                gates: Box::new([policy::Gate {
                    name: b"gate".as_slice().into(),
                    workflow: b"gate.yml".as_slice().into(),
                    ignored: Box::new([]),
                    blocking: true,
                    base: false,
                    report: None,
                }]),
                tiers: Box::new([])
            }]),
            policy::Project::DEFAULT,
            after
        ));
        row.landing.phase = policy::Phase::Proving;
        row.landing.members[0].gates = Box::new([policy::GateProof {
            proof: if kind == "dispatch" {
                policy::Proof::Keyed { key: KEY }
            } else {
                policy::Proof::Dispatched { key: KEY, try_: 1 }
            },
            posted: policy::Posted::Nothing,
            faults: 0,
        }]);
        d.landing.main_keys.insert((REPO, b"gate".as_slice().into()), (fixtures::BASE, KEY)).unwrap();
    }
    if kind == "rerun" {
        row.landing.members[0].gates = Box::new([policy::GateProof {
            proof: policy::Proof::Dispatched { key: KEY, try_: 2 },
            posted: policy::Posted::Pending,
            faults: 0,
        }]);
        // The next failed job is sent through continue_retry, before policy stepping.
        let mut flight = runs::follow();
        flight.start = None;
        flight.purpose = purpose::EffectPurpose::Landing {
            task: TASK,
            member: 0,
            gate: 0,
            operation: 2,
            preparation: 0,
            attempt: 2,
        }
        .identity()
        .unwrap();
        flight.run = Some(api::Run {
            id: 7,
            number: 8,
            workflow: b"gate.yml".as_slice().into(),
            commit: fixtures::HEAD,
            event: api::Event::Dispatch,
            actor: 1,
            state: api::RunState::Completed,
            conclusion: Some(api::Conclusion::Failure),
            url: Box::new([]),
            created: env().now,
            jobs: Box::new([]),
        });
        flight.writes =
            Box::new([Write::Rerun { attempt: api::JobAttempt { head: fixtures::HEAD, run: 7, job: 9, attempt: 1 } }]);
        d.landing
            .flights
            .insert(
                ReceiptKey { project: 5, repository: REPO, check: Check::Gate(b"gate".as_slice().into()), key: KEY },
                flight,
            )
            .unwrap();
    }
    outputs(&mut d, Event::Landing { row });
    outputs(&mut d, Event::Client(skein_forge_client::Event::PauseOutbox));
    for number in 1..=u64::from(env().limits.entries) {
        outputs(&mut d, Event::Enqueue { entry: refusals::entry(number) });
        outputs(&mut d, Event::Committed { entry: number });
    }
    d
}

fn start(d: &mut Domain, task: u64, entry: u64) {
    outputs(
        d,
        Event::StepLanding {
            task,
            entry,
            heard: LandingHeard {
                job: None,
                outcomes: Box::new([]),
                dependencies: Box::new([]),
                held: Box::new([]),
                failure: Box::new([]),
                preparation: policy::Preparation::Idle,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        },
    );
}

fn finish(d: &mut Domain, base: api::Commit) -> Box<[Request]> {
    for _ in 0_u32..32 {
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Ok(Answer::Commit(if branch.as_ref() == b"main" { base } else { fixtures::HEAD }))
            }
            Op::Read(Read::Pull { .. }) => Ok(Answer::Pull(fixtures::open_pull())),
            Op::Read(Read::Reviews { .. }) => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            Op::Read(Read::PullFor { head, .. }) => {
                if head.as_ref() == b"temper/51" {
                    Ok(Answer::Pull(fixtures::open_pull()))
                } else {
                    Err(Error::Missing)
                }
            }
            Op::Read(Read::Commits { before, after, .. }) => Ok(Answer::Commits {
                commits: if before == fixtures::BASE && after == fixtures::HEAD {
                    Box::new([api::CommitMessage {
                        id: fixtures::HEAD,
                        parents: Box::new([fixtures::BASE, fixtures::HEAD]),
                        message: Box::new([]),
                    }])
                } else {
                    Box::new([])
                },
                more: false,
            }),
            Op::Read(_) | Op::Write(_) => panic!("unexpected landing read {op:?}"),
        };
        let out = fixtures::answer(d, call, answer);
        for request in &out {
            if matches!(request, Request::LandingDecision { .. } | Request::ResourceRefused { .. }) {
                return out;
            }
        }
    }
    panic!("landing decides after its coherent gather")
}

fn drive(d: &mut Domain, task: u64, entry: u64) -> Box<[Request]> {
    start(d, task, entry);
    finish(d, fixtures::BASE)
}

fn assert_wait(out: &[Request]) {
    let mut waited = false;
    for request in out {
        waited |= matches!(
            request,
            Request::LandingDecision {
                task: TASK,
                decision: policy::Decision::Wait { until: None },
                entry: None,
                evidence: None,
            }
        );
        assert!(
            !matches!(
                request,
                Request::ResourceRefused { .. }
                    | Request::Refused { .. }
                    | Request::LandingDecision { decision: policy::Decision::Hold(_), .. }
                    | Request::Save { record: Stored::Entry { .. }, .. }
            ),
            "{out:?}"
        );
    }
    assert!(waited, "{out:?}");
}

fn asks(out: &[Request]) -> Box<[u64]> {
    let mut asks = skein_lib::List::uncapped();
    for request in out {
        if let Request::WakeProcedure { task } = request {
            asks.push(*task).unwrap();
        }
    }
    asks.into_boxed()
}
fn entries(out: &[Request]) -> Box<[u64]> {
    let mut entries = skein_lib::List::uncapped();
    for request in out {
        if let Request::Save { record: Stored::Entry { entry, .. }, .. } = request {
            entries.push(entry.number).unwrap();
        }
    }
    entries.into_boxed()
}
fn free_one(d: &mut Domain) -> Box<[Request]> {
    // Withdrawal's outcome frees one of the committed entries.
    outputs(d, Event::Withdraw { entry: 1 })
}

#[test]
fn landing_writes_wait_and_keep_one_fresh_entry() {
    for kind in ["open", "merge", "rerun", "dispatch", "post"] {
        let mut d = full(kind);
        let mut prior = d.landing(TASK).unwrap().clone();
        assert_wait(&drive(&mut d, TASK, 91));
        prior.room = Some(1);
        assert_eq!(d.landing(TASK), Some(&prior), "{kind}: only its place is saved");
        assert!(d.landing.pending.is_empty(), "no decision or entry number survives");
        assert!(!d.entries.contains_key(&91));
        d.wait_landing(&env(), TASK, None);
        assert!(!d.landing.due.contains_key(&TASK), "no room poll");
        assert_eq!(asks(&free_one(&mut d)).as_ref(), &[TASK]);
        assert!(d.room_asked(TASK));
        assert_eq!(room::reserved(&d), 1);
        let out = drive(&mut d, TASK, FRESH);
        assert_eq!(entries(&out).as_ref(), &[FRESH], "{kind}: kept exactly once");
        let write = &d.entries.get(&FRESH).unwrap().entry.effect.write;
        assert!(
            match kind {
                "open" => matches!(write, Write::OpenPull { .. }),
                "merge" => matches!(write, Write::Merge { .. }),
                "rerun" => matches!(write, Write::Rerun { .. }),
                "dispatch" => matches!(write, Write::Dispatch { .. }),
                "post" => matches!(write, Write::Status { .. }),
                _ => unreachable!(),
            },
            "{write:?}"
        );
        assert_eq!(d.landing(TASK).unwrap().room, None);
        assert_eq!(room::reserved(&d), 0);
        assert_eq!(d.entries.len(), env().limits.entries);
    }
}

fn change_waiter(d: &mut Domain, place: Option<u64>) {
    let mut row = fixtures::opening_row();
    row.task = OTHER;
    row.change.task = OTHER;
    row.branch = b"temper/62".as_slice().into();
    row.room = place;
    row.pending = None;
    row.effect = temper_engine_domain_forge_change::EffectResult::None;
    row.change.state = temper_engine_domain_forge_change::State::Opening { requested: false };
    outputs(d, Event::Change { row });
}
fn open_change(d: &mut Domain, entry: u64) -> Box<[Request]> {
    let Event::StepChange { merge_job, heard, gates, queue_repair_active, pending, kept, .. } = fixtures::change_step()
    else {
        unreachable!()
    };
    outputs(d, Event::StepChange { task: OTHER, entry, merge_job, heard, gates, queue_repair_active, pending, kept });
    fixtures::missing_pull_reads(d);
    fixtures::finish_missing(d)
}

#[test]
fn a_change_waiting_first_takes_room_before_a_landing() {
    let mut d = full("open");
    change_waiter(&mut d, None);
    open_change(&mut d, 90);
    assert_eq!(d.change(OTHER).unwrap().room, Some(1));
    assert_wait(&drive(&mut d, TASK, 91));
    assert_eq!(d.landing(TASK).unwrap().room, Some(2));
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[OTHER]);
    assert_wait(&drive(&mut d, TASK, 92));
    let out = open_change(&mut d, FRESH);
    assert_eq!(entries(&out).as_ref(), &[FRESH]);
    assert_eq!(d.landing(TASK).unwrap().room, Some(2));
    assert_eq!(d.change(OTHER).unwrap().room, None);
}

#[test]
fn landing_room_places_restart_in_order_without_a_saved_decision() {
    let mut before = full("open");
    assert_wait(&drive(&mut before, TASK, 91));
    change_waiter(&mut before, None);
    open_change(&mut before, 92);
    let mut d = landing_fixture_with(fixtures::connected_domain());
    for task in [OTHER, fixtures::CHANGE] {
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(before.change(task).unwrap().clone())) });
    }
    outputs(&mut d, Event::Restore { record: Stored::Landing(before.landing(TASK).unwrap().clone()) });
    let mut ended = landing_row(63, policy::Phase::Cancelled);
    ended.room = Some(3);
    outputs(&mut d, Event::Restore { record: Stored::Landing(ended) });
    for (_, row) in &before.entries {
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry: row.entry.clone(), held: true } });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    assert_eq!(room::reserved(&d), 0);
    assert_eq!(d.landing(TASK).unwrap().room, Some(1));
    assert_eq!(d.change(OTHER).unwrap().room, Some(2));
    assert_eq!(d.landing(63).unwrap().room, None);
    assert!(!d.landing.due.contains_key(&TASK));
    let freed = outputs(&mut d, Event::Resolve { entry: 1, again: false });
    assert!(asks(&freed).contains(&TASK));
    assert!(!asks(&freed).contains(&OTHER));
    let out = drive(&mut d, TASK, FRESH);
    let mut choice = None;
    for request in &out {
        if let Request::LandingDecision { task, decision, entry, .. } = request {
            choice = Some((*task, *decision, *entry));
        }
    }
    assert_eq!(entries(&out).as_ref(), &[FRESH], "{choice:?}, {:?}", d.landing(TASK).unwrap().landing.phase);
    assert_eq!(d.landing(TASK).unwrap().room, None);
    assert_eq!(d.change(OTHER).unwrap().room, Some(2));
}

#[test]
fn a_waiting_landing_decides_again_and_returns_room_when_no_write_is_needed() {
    for base in [Some([9; 32]), None] {
        let mut d = full("merge");
        assert_wait(&drive(&mut d, TASK, 91));
        change_waiter(&mut d, Some(2));
        assert_eq!(asks(&free_one(&mut d)).as_ref(), &[TASK]);
        start(&mut d, TASK, FRESH);
        let out = match base {
            Some(base) => finish(&mut d, base),
            None => fixtures::answer_next(&mut d, Err(Error::Missing)),
        };
        assert!(entries(&out).is_empty(), "new facts need no merge");
        assert_eq!(d.landing(TASK).unwrap().room, None);
        assert_eq!(asks(&out).as_ref(), &[OTHER]);
        assert!(!d.entries.contains_key(&91));
        assert!(!d.entries.contains_key(&FRESH));
    }
}

#[test]
fn a_full_outbox_does_not_hide_a_duplicate_landing_entry() {
    for kind in ["open", "rerun"] {
        let mut d = full(kind);
        let prior = d.landing(TASK).unwrap().clone();
        let out = drive(&mut d, TASK, 1);
        let mut refused = false;
        for request in &out {
            refused |= matches!(
                request,
                Request::ResourceRefused { task: TASK, cause: RefusalCause::Backend(Error::Exists), .. }
            );
        }
        assert!(refused, "{out:?}");
        assert_eq!(d.landing(TASK), Some(&prior));
        assert_eq!(room::reserved(&d), 0);
    }
}

#[test]
fn an_outbox_wait_does_not_expire_a_landings_gate() {
    let mut d = full("post");
    let row = d.landing.rows.get_mut(&TASK).unwrap();
    row.room = Some(1);
    row.landing.members[0].gates[0].posted = policy::Posted::Pending;
    d.landing.projects.get_mut(&5).unwrap().5.gate_timeout = Duration::from_secs(1);
    let mut flight = runs::follow();
    flight.start = None;
    flight.since = Some(Time::from_nanos(0));
    flight.run = Some(api::Run {
        id: 7,
        number: 8,
        workflow: b"gate.yml".as_slice().into(),
        commit: fixtures::HEAD,
        event: api::Event::Dispatch,
        actor: 1,
        state: api::RunState::Running,
        conclusion: None,
        url: Box::new([]),
        created: env().now,
        jobs: Box::new([]),
    });
    d.landing
        .flights
        .insert(
            ReceiptKey { project: 5, repository: REPO, check: Check::Gate(b"gate".as_slice().into()), key: KEY },
            flight,
        )
        .unwrap();
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[TASK]);
    let out = drive(&mut d, TASK, FRESH);
    let mut waited = false;
    for request in &out {
        if let Request::LandingDecision { decision, entry, .. } = request {
            assert!(matches!(decision, policy::Decision::Wait { .. }), "{decision:?}");
            assert_eq!(*entry, None);
            waited = true;
        }
    }
    assert!(waited);
    assert_eq!(d.landing(TASK).unwrap().room, None);
    assert_eq!(room::reserved(&d), 0);
}

#[test]
fn cancelling_a_waiting_or_asked_landing_returns_its_room() {
    for asked in [false, true] {
        let mut d = full("open");
        assert_wait(&drive(&mut d, TASK, 91));
        change_waiter(&mut d, Some(2));
        if asked {
            assert_eq!(asks(&free_one(&mut d)).as_ref(), &[TASK]);
        }
        let out = outputs(&mut d, Event::SettleEffects { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled });
        assert_eq!(d.landing(TASK).unwrap().room, None);
        assert!(!d.room_asked(TASK));
        let out = if asked { out } else { free_one(&mut d) };
        assert!(asks(&out).contains(&OTHER));
        assert_eq!(room::reserved(&d), 1);
    }
}
