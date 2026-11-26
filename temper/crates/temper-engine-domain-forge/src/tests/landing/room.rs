//! Landing writes join the shared line without keeping their decision (forge/room.md, 3–4).
use super::*;
use crate::room;
use jig_domain_core_vocabulary::waits::{AtBound, Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_forge::api::{self, Answer, Error, Op, Read, Write};
use skein_lib::List;
use temper_client_protocol_wire::waits::{CHANGE_OPENING, LANDING_ROOM, THING_ENTRY};

const TASK: u64 = 61;
const OTHER: u64 = 62;
const FRESH: u64 = 101;
const KEY: policy::Key = [7; 32];

fn configured(kind: &str) -> (Domain, LandingRow) {
    let mut d = landing_fixture();
    fixtures::connect(&mut d);
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
        flight.inputs = landing::inputs(fixtures::HEAD, KEY, None, false);
        row.follows = Some(Box::new([saved_dispatch(&row, &flight, 0, 2)]));
        d.landing
            .flights
            .insert(
                ReceiptKey { project: 5, repository: REPO, check: Check::Gate(b"gate".as_slice().into()), key: KEY },
                flight,
            )
            .unwrap();
    }
    (d, row)
}

fn start_empty(d: &mut Domain) {
    outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(d, Event::CatchUp);
}

fn full(kind: &str) -> Domain {
    let (mut d, row) = configured(kind);
    start_empty(&mut d);
    outputs(&mut d, Event::Landing { row });
    for number in 1..=u64::from(env().limits.entries) {
        outputs(&mut d, Event::Enqueue { entry: refusals::entry(number) });
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
                preparation: Some(policy::Preparation::Idle),
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        },
    );
}

fn finish(d: &mut Domain, base: api::Commit) -> Box<[Request]> {
    finish_with_run(d, base, None)
}

fn finish_with_run(d: &mut Domain, base: api::Commit, run: Option<&api::Run>) -> Box<[Request]> {
    for _ in 0_u32..32 {
        let (call, op) = fixtures::next_call(d);
        let answer = answer_for(op, base, run);
        let out = fixtures::answer(d, call, answer);
        for request in &out {
            if matches!(request, Request::LandingDecision { .. } | Request::ResourceRefused { .. }) {
                return out;
            }
        }
    }
    panic!("landing decides after its coherent gather")
}

fn answer_for(op: Op, base: api::Commit, run: Option<&api::Run>) -> Result<Answer, Error> {
    match op {
        Op::Read(Read::Branch { branch }) => {
            Ok(Answer::Commit(if branch.as_ref() == b"main" { base } else { fixtures::HEAD }))
        }
        Op::Read(Read::Pull { number, .. }) => {
            let mut pull = fixtures::open_pull();
            pull.number = number;
            Ok(Answer::Pull(pull))
        }
        Op::Read(Read::Reviews { .. }) => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
        Op::Read(Read::PullFor { head, .. }) => {
            if head.as_ref() == b"temper/51" {
                Ok(Answer::Pull(fixtures::open_pull()))
            } else {
                Err(Error::Missing)
            }
        }
        Op::Read(Read::Commits { before, after, .. }) => Ok(Answer::Commits {
            commits: if (before == fixtures::BASE || before == [9; 32]) && after == fixtures::HEAD {
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
        Op::Read(Read::Runs { .. }) => {
            Ok(Answer::Runs { runs: Box::new([run.expect("listed gate run").clone()]), more: false })
        }
        Op::Read(Read::Run { run: id }) => {
            let run = run.expect("read gate run");
            assert_eq!(id, run.id);
            Ok(Answer::Run(run.clone()))
        }
        Op::Read(_) | Op::Write(_) => panic!("unexpected landing read {op:?}"),
    }
}

fn drive(d: &mut Domain, task: u64, entry: u64) -> Box<[Request]> {
    start(d, task, entry);
    finish(d, fixtures::BASE)
}

fn assert_wait(out: &[Request], on: policy::Awaited) {
    let mut waited = false;
    for request in out {
        waited |= matches!(
            request,
            Request::LandingDecision {
                task: TASK,
                decision: policy::Decision::Wait { until: None, on: found },
                entry: None,
                evidence: None,
                ..
            } if *found == on
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
    let mut asks = List::uncapped();
    for request in out {
        if let Request::WakeProcedure { task } = request {
            asks.push(*task).unwrap();
        }
    }
    asks.into_boxed()
}
fn entries(out: &[Request]) -> Box<[u64]> {
    let mut entries = List::uncapped();
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

fn assert_room_report(out: &[Request], task: u64, step: u32, since: Wall, bound: ReportedBound) {
    let [record] = wait_report(out, task).expect("room report") else { panic!("one room wait: {out:?}") };
    let owner = refusals::entry(1).task;
    let mut entries = List::with_capacity(4);
    for number in 1..=4 {
        entries.push(Thing { what: THING_ENTRY, number, owner: Some(owner) }).unwrap();
    }
    assert_eq!(
        record,
        &Reported { kind: WaitKind::Room, object: WaitObject::Procedure { step, things: entries }, since, bound }
    );
}

#[test]
fn landing_writes_wait_and_keep_one_fresh_entry() {
    for (kind, on) in [
        ("open", policy::Awaited::Open),
        ("merge", policy::Awaited::Merge),
        ("rerun", policy::Awaited::Run { member: 0, gate: 0 }),
        ("dispatch", policy::Awaited::Run { member: 0, gate: 0 }),
        ("post", policy::Awaited::Post { member: 0, gate: 0 }),
    ] {
        let mut d = full(kind);
        let mut prior = d.landing(TASK).unwrap().clone();
        let waiting = drive(&mut d, TASK, 91);
        assert_wait(&waiting, on);
        let behind = ReportedBound::None(Unbounded::Behind(refusals::entry(1).task));
        assert_room_report(&waiting, TASK, LANDING_ROOM, env().wall, behind);
        assert_panel_wait(&d, &env(), TASK, LandingWaitFor::Room, Some(&wait_report(&waiting, TASK).unwrap()[0]));
        prior.room = Some(1);
        prior.waiting = Some(on);
        assert_eq!(d.landing(TASK), Some(&prior), "{kind}: only its place and wait are saved");
        assert!(d.landing.pending.is_empty(), "no decision or entry number survives");
        assert!(!d.entries.contains_key(&91));
        d.wait_landing(&env(), TASK, None);
        assert!(!d.landing.due.contains_key(&TASK), "no room poll");
        let freed = free_one(&mut d);
        assert_eq!(asks(&freed).as_ref(), &[TASK]);
        assert_eq!(wait_report(&freed, TASK), Some([].as_slice()), "{kind}: the ask clears its room wait");
        assert!(d.room_asked(TASK));
        assert_eq!(room::reserved(&d), 1);
        let out = drive(&mut d, TASK, FRESH);
        assert_eq!(entries(&out).as_ref(), &[FRESH], "{kind}: kept exactly once");
        assert_eq!(wait_report(&out, TASK), None, "{kind}: keeping the entry does not clear room twice");
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
        assert_eq!(d.landing(TASK).unwrap().waiting, None);
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

fn pending_write(step: u32, things: &[WaitThing]) -> Reported {
    let e = env();
    let at = Wall::from_nanos(e.wall.as_nanos().saturating_add(e.limits.change_policy.stall.as_nanos()));
    step_wait(step, e.wall, ReportedBound::At { at, then: AtBound::Hold }, things)
}

#[test]
fn a_change_waiting_first_takes_room_before_a_landing() {
    let mut d = full("open");
    change_waiter(&mut d, None);
    let joined = open_change(&mut d, 90);
    let behind = ReportedBound::None(Unbounded::Behind(refusals::entry(1).task));
    assert_room_report(&joined, OTHER, CHANGE_OPENING, env().wall, behind);
    assert_eq!(d.change(OTHER).unwrap().room, Some(1));
    let checked_again = open_change(&mut d, 93);
    assert_eq!(wait_report(&checked_again, OTHER), None, "an unchanged step is silent");
    assert_wait(&drive(&mut d, TASK, 91), policy::Awaited::Open);
    assert_eq!(d.landing(TASK).unwrap().room, Some(2));
    assert_eq!(asks(&free_one(&mut d)).as_ref(), &[OTHER]);
    assert_wait(&drive(&mut d, TASK, 92), policy::Awaited::Open);
    let out = open_change(&mut d, FRESH);
    assert_eq!(entries(&out).as_ref(), &[FRESH]);
    assert_eq!(
        wait_report(&out, OTHER),
        Some(
            [pending_write(CHANGE_OPENING, &[Thing { what: THING_ENTRY, number: FRESH, owner: Some(OTHER) }])]
                .as_slice()
        )
    );
    assert_eq!(d.landing(TASK).unwrap().room, Some(2));
    assert_eq!(d.change(OTHER).unwrap().room, None);
}

#[test]
fn landing_room_places_restart_in_order_without_a_saved_decision() {
    let mut before = full("open");
    assert_wait(&drive(&mut before, TASK, 91), policy::Awaited::Open);
    change_waiter(&mut before, None);
    open_change(&mut before, 92);
    let started = Wall::from_nanos(1);
    assert!(started < env().wall);
    let mut d = restoring_domain(started);
    fixtures::connect(&mut d);
    let mut d = landing_fixture_with(d);
    for task in [OTHER, fixtures::CHANGE] {
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(before.change(task).unwrap().clone())) });
    }
    outputs(&mut d, Event::Restore { record: Stored::Landing(before.landing(TASK).unwrap().clone()) });
    let mut ended = landing_row(63, policy::Phase::Cancelled);
    ended.room = Some(3);
    outputs(&mut d, Event::Restore { record: Stored::Landing(ended) });
    let closer_task = 64;
    let ended_closer = 65;
    for (task, done) in [(closer_task, false), (ended_closer, true)] {
        let row = closer::closer(task, 7, done);
        outputs(&mut d, Event::Restore { record: Stored::Closer(row) });
    }
    for (_, row) in &before.entries {
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry: row.entry.clone(), held: true } });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let reports = outputs(&mut d, Event::CatchUp);
    // No live task was restored for the entries' writer. Its end and both joins count from start.
    let at = advanced_wall(started, env().limits.waits.room);
    let bound = ReportedBound::At { at, then: AtBound::Hold };
    assert_room_report(&reports, TASK, LANDING_ROOM, started, bound);
    assert_room_report(&reports, OTHER, CHANGE_OPENING, started, bound);
    assert_eq!(
        wait_report(&reports, fixtures::CHANGE),
        Some(
            [pending_write(
                temper_client_protocol_wire::waits::CHANGE_READY,
                &[
                    Thing { what: THING_ENTRY, number: fixtures::OPEN, owner: Some(fixtures::CHANGE) },
                    wait_thing(temper_client_protocol_wire::waits::THING_PULL, 1),
                ]
            )]
            .as_slice()
        )
    );
    assert_eq!(wait_report(&reports, closer_task), Some([].as_slice()));
    assert_eq!(wait_report(&reports, 63), None);
    assert_eq!(wait_report(&reports, ended_closer), None);

    for _ in 0..32_u32 {
        if d.fresh_reads_in() {
            break;
        }
        let (call, op) = fixtures::next_call(&mut d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"temper/1/l61" => Err(Error::Missing),
            Op::Read(Read::Branch { branch }) => {
                Ok(Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { fixtures::HEAD }))
            }
            Op::Read(Read::Pull { .. }) => Ok(Answer::Pull(fixtures::open_pull())),
            Op::Read(Read::PullFor { .. }) => Err(Error::Missing),
            Op::Read(Read::Commits { .. }) => Ok(Answer::Commits { commits: Box::new([]), more: false }),
            Op::Read(Read::Reviews { .. }) => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            Op::Read(Read::Items { .. }) => {
                Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now })
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("restart fresh read: {other:?}"),
        };
        fixtures::answer(&mut d, call, answer);
    }
    assert!(d.fresh_reads_in(), "the room is judged after the fresh pass");
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
        assert_wait(&drive(&mut d, TASK, 91), policy::Awaited::Merge);
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
        assert_wait(&drive(&mut d, TASK, 91), policy::Awaited::Open);
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

#[test]
fn a_closing_round_keeps_the_close_wait_on_its_row() {
    let mut d = full("merge");
    let row = d.landing.rows.get_mut(&TASK).unwrap();
    row.landing.phase = policy::Phase::Proving;
    row.landing.round.ending = Some(policy::RoundEnd::Leave);
    d.landing.results.insert((TASK, 0, 0, 7), (policy::Status::Pending, false)).unwrap();
    assert_wait(&drive(&mut d, TASK, 91), policy::Awaited::Close);
    assert_eq!(d.landing(TASK).unwrap().waiting, Some(policy::Awaited::Close));
    let Document::Landing(view) =
        d.document(TASK, jig_domain_people::ApplicationScope::Project(5), 1, env().wall, &env().limits).unwrap()
    else {
        panic!("landing")
    };
    assert_eq!(view.waits.unwrap().on, LandingWaitFor::Close);
}

fn gate_key() -> ReceiptKey {
    ReceiptKey { project: 5, repository: REPO, check: Check::Gate(b"gate".as_slice().into()), key: KEY }
}

fn dispatched(out: &[Request], owner: u64) -> bool {
    for request in out {
        if let Request::LandingDecision { task, decision: policy::Decision::Dispatch { key: KEY, .. }, .. } = request
            && *task == owner
        {
            return true;
        }
    }
    false
}

fn write_outcome(out: &[Request]) -> Option<skein_forge_client::Outcome> {
    for request in out {
        if let Request::Outcome { task: TASK, entry: FRESH, outcome, .. } = request {
            return Some(*outcome);
        }
    }
    None
}

fn dispatch() -> (Domain, skein_forge_client::Entry) {
    let (mut d, row) = configured("dispatch");
    start_empty(&mut d);
    outputs(&mut d, Event::Landing { row });
    let out = drive(&mut d, TASK, FRESH);
    assert_eq!(entries(&out).as_ref(), &[FRESH]);
    assert!(dispatched(&out, TASK), "{out:?}");
    outputs(&mut d, Event::Committed { entry: FRESH });
    let (call, op) = fixtures::next_call(&mut d);
    assert!(matches!(op, Op::Read(Read::Items { .. })), "write's clock: {op:?}");
    fixtures::answer(&mut d, call, Ok(Answer::Items { items: Box::new([]), more: false, cut: false, now: env().now }));
    let (_, op) = fixtures::next_call(&mut d);
    assert!(matches!(op, Op::Write(Write::Dispatch { .. })));
    assert!(d.flights.contains_key(&(TASK, FRESH)));
    let kept = d.entries.get(&FRESH).unwrap().entry.clone();
    (d, kept)
}

fn outcome(d: &mut Domain, outcome: skein_forge_client::Outcome) -> Queue<Request> {
    let mut out = Queue::uncapped();
    domain::scripted_outcome(d, &env(), FRESH, TASK, outcome, &mut out);
    out
}

fn made(d: &mut Domain) -> Queue<Request> {
    outcome(d, skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Set, found: false })
}

#[test]
fn dispatch_failures_free_room_at_settlement_not_at_the_hold() {
    use skein_forge_client::{Made, Outcome};
    for final_outcome in
        [Outcome::Failed(Error::Forbidden), Outcome::Raced { made: Made::Set, why: Error::Stale }, Outcome::Held]
    {
        let (mut d, _) = dispatch();
        let free = room::free(&d, &LIMITS);
        outcome(&mut d, Outcome::Uncertain);
        assert_eq!(room::free(&d, &LIMITS), free);
        outcome(&mut d, final_outcome);
        if final_outcome == Outcome::Held {
            assert_eq!(room::free(&d, &LIMITS), free, "held is not settled");
            assert!(d.entries.get(&FRESH).unwrap().held);
            let settled =
                outputs(&mut d, Event::SettleEffects { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled });
            assert!(settled.contains(&Request::NotMade { task: TASK, entry: FRESH }));
            let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH + 1 });
            assert!(out.contains(&Request::EffectsSettled { task: TASK }));
        }
        assert!(!d.entries.contains_key(&FRESH));
        assert_eq!(d.landing(TASK).unwrap().entries.as_ref(), []);
        assert_eq!(room::free(&d, &LIMITS), free + 1);
    }
}

fn restore_dispatch(row: LandingRow, old_entry: Option<skein_forge_client::Entry>) -> Domain {
    let (mut d, _) = configured("dispatch");
    outputs(&mut d, Event::Restore { record: Stored::Landing(row) });
    if let Some(entry) = old_entry {
        // The old writer kept it after Made, without a Flight row.
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: false } });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    d
}

fn gate_run(start: Time) -> api::Run {
    api::Run {
        id: 7,
        number: 8,
        workflow: b"gate.yml".as_slice().into(),
        commit: fixtures::HEAD,
        event: api::Event::Dispatch,
        actor: 7,
        state: api::RunState::Completed,
        conclusion: Some(api::Conclusion::Success),
        url: Box::new([]),
        created: start.saturating_add(Duration::from_millis(1)),
        jobs: Box::new([]),
    }
}

#[test]
fn a_made_dispatch_restarts_from_its_landing_and_receipts_the_run() {
    let (mut d, entry) = dispatch();
    let free = room::free(&d, &LIMITS);
    let mut out = made(&mut d);
    let mut erased = false;
    while let Some(request) = out.pop() {
        erased |= request == (Request::Erase { key: Key::Entry(FRESH) });
    }
    assert!(erased && !d.entries.contains_key(&FRESH));
    assert_eq!(room::free(&d, &LIMITS), free + 1);
    let key = gate_key();
    let before = d.landing.flights.get(&key).unwrap().clone();
    let start = entry.start.unwrap().at;
    let mut row = d.landing(TASK).unwrap().clone();
    // Pending status has already been posted when the next proof poll starts.
    row.landing.members[0].gates[0].posted = policy::Posted::Pending;
    let mut d = restore_dispatch(row, None);
    assert!(d.entries.is_empty());
    let after = d.landing.flights.get(&key).expect("the row rebuilds the run without an outbox entry");
    assert_eq!((&after.workflow, &after.branch, &after.inputs), (&before.workflow, &before.branch, &before.inputs));
    assert_eq!((after.start, after.since, after.wall), (Some(start), before.since, before.wall));
    let run = gate_run(start);
    for entry in [FRESH + 1, FRESH + 2] {
        start_step_for_run(&mut d, entry, &run);
        if d.receipt(&key).is_some() {
            break;
        }
    }
    let receipt = d.receipt(&key).expect("the rebuilt follower made its receipt");
    assert_eq!((receipt.run.id, receipt.passed), (run.id, true));
    assert!(!d.landing.flights.contains_key(&key));
}

fn start_step_for_run(d: &mut Domain, entry: u64, run: &api::Run) {
    start(d, TASK, entry);
    finish_with_run(d, fixtures::BASE, Some(run));
}

#[test]
fn cancelling_a_dispatch_lets_the_next_landing_dispatch_its_key() {
    let (mut d, _) = dispatch();
    made(&mut d);
    let key = gate_key();
    assert!(d.landing.flights.contains_key(&key));
    let fault = d.landing.flights.get(&key).unwrap().clone();
    d.landing.faults.insert((TASK, 0, 0), fault).unwrap();
    outputs(&mut d, Event::Restore { record: Stored::Landing(landing_row(63, policy::Phase::Waiting)) });
    let cancelled = outputs(&mut d, Event::SettleEffects { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled });
    assert!(!d.landing.flights.contains_key(&key));
    assert!(d.landing.faults.is_empty());
    assert!(asks(&cancelled).contains(&63));
    assert!(d.receipt(&key).is_none());
    let (_, mut row) = configured("dispatch");
    row.landing.task = OTHER;
    outputs(&mut d, Event::Landing { row });
    let out = drive(&mut d, OTHER, FRESH + 1);
    assert!(dispatched(&out, OTHER), "{out:?}");
    assert_eq!(d.landing.flights.get(&key).unwrap().task, OTHER);
}

fn settle_legacy_dispatch(d: &mut Domain, start: Time) -> Box<[Request]> {
    for _ in 0..40_u32 {
        let (call, op) = fixtures::next_call(d);
        let out = fixtures::answer(d, call, answer_for(op, fixtures::BASE, Some(&gate_run(start))));
        if write_outcome(&out).is_some() {
            return out;
        }
    }
    panic!("client must settle the restored dispatch");
}

#[test]
fn old_retained_dispatches_settle_for_live_and_cancelled_landings() {
    let (d, entry) = dispatch();
    let start = entry.start.unwrap().at;
    for cancelled in [false, true] {
        let mut row = d.landing(TASK).unwrap().clone();
        row.follows = None;
        if cancelled {
            row.landing.phase = policy::Phase::Cancelled;
        }
        let mut d = restore_dispatch(row, Some(entry.clone()));
        let free = room::free(&d, &LIMITS);
        let key = gate_key();
        assert_eq!(d.landing.flights.contains_key(&key), !cancelled);
        // Fill the rest of the landing table to test admission at the bound.
        for task in 200..200 + u64::from(LIMITS.landings - 1) {
            outputs(&mut d, Event::Restore { record: Stored::Landing(landing_row(task, policy::Phase::Waiting)) });
        }
        if cancelled {
            assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Err(LandingRefusal::Landings));
        }
        let settled = settle_legacy_dispatch(&mut d, start);
        assert!(matches!(write_outcome(&settled), Some(skein_forge_client::Outcome::Made { .. })));
        assert!(!d.entries.contains_key(&FRESH));
        assert_eq!(room::free(&d, &LIMITS), free + 1);
        let row = d.landing(TASK).unwrap();
        assert!(row.entries.is_empty());
        if cancelled {
            assert!(!d.landing.flights.contains_key(&key));
            assert_eq!(row.follows(), []);
            assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Ok(()));
            let (_, mut next) = configured("dispatch");
            next.landing.task = OTHER;
            let out = outputs(&mut d, Event::Landing { row: next });
            assert!(out.contains(&Request::Erase { key: Key::Landing(TASK) }));
        } else {
            assert_eq!(row.follows()[0].start, Some(start));
            assert!(d.landing.flights.contains_key(&key));
            let after = restore_dispatch(row.clone(), None);
            assert_eq!(after.landing.flights.get(&key).unwrap().start, Some(start));
        }
    }
}

#[test]
fn ending_a_landing_withdraws_its_unsent_write() {
    for ending in [ReleaseEnding::Done, ReleaseEnding::Cancelled] {
        let (mut d, row) = configured("open");
        start_empty(&mut d);
        outputs(&mut d, Event::Landing { row });
        drive(&mut d, TASK, FRESH);
        assert!(d.entries.get(&FRESH).unwrap().entry.attempt.is_none());
        if ending == ReleaseEnding::Done {
            d.landing.rows.get_mut(&TASK).unwrap().landing.phase = policy::Phase::Landed;
        }
        let out = outputs(&mut d, Event::SettleEffects { task: TASK, root: TASK, ending });
        assert_eq!(write_outcome(&out), Some(skein_forge_client::Outcome::Withdrawn));
        assert!(out.contains(&Request::EffectsSettled { task: TASK }));
        assert!(!d.entries.contains_key(&FRESH));
        assert!(d.landing(TASK).unwrap().entries.is_empty());
    }
}

#[test]
fn a_room_wait_holds_the_landing_at_the_ended_writers_bound() {
    use outbox_room::later;
    let mut d = full("open");
    let joined = env();
    let waiting = drive(&mut d, TASK, 91);
    let writer = refusals::entry(1).task;
    assert_room_report(&waiting, TASK, LANDING_ROOM, joined.wall, ReportedBound::None(Unbounded::Behind(writer)));
    let ended = later(&joined, Duration::from_secs(1));
    let mut out = Queue::uncapped();
    step(&mut d, &ended, Event::ChangeEnding { task: writer, ending: Ending::Cancelled }, &mut out);
    let waiting = collected(out);
    let [record] = wait_report(&waiting, TASK).expect("finite room report") else { panic!("one room wait") };
    let at = advanced_wall(ended.wall, ended.limits.waits.room);
    assert_eq!(record.since, joined.wall);
    assert_eq!(record.bound, ReportedBound::At { at, then: AtBound::Hold });
    assert_eq!(d.inspection_room_wait(TASK), Some((at, true)));
    let before = later(&ended, Duration::from_nanos(ended.limits.waits.room.as_nanos() - 1));
    assert!(!asks(&fire_at(&mut d, &before, before.wall)).contains(&TASK));
    let due = later(&ended, ended.limits.waits.room);
    assert!(asks(&fire_at(&mut d, &due, due.wall)).contains(&TASK));

    let mut out = Queue::uncapped();
    step(&mut d, &due, Event::StepLanding { task: TASK, entry: 92, heard: preparing_heard(None) }, &mut out);
    let held = collected(out);
    let mut read_hold = false;
    for request in &held {
        read_hold |= matches!(
            request,
            Request::LandingDecision {
                task: TASK,
                decision: policy::Decision::Hold(policy::Hold::Read),
                entry: None,
                ..
            }
        );
    }
    assert!(read_hold);
    assert_eq!(wait_report(&held, TASK), Some([].as_slice()));
    let row = d.landing(TASK).unwrap();
    assert_eq!(row.room, Some(1));
    assert!(matches!(row.landing.phase, policy::Phase::Held { why: policy::Hold::Read, .. }));
    let words = row.read_hold.as_deref().expect("room hold words");
    let expected = format!("read: room in the forge's outbox: held by entry 1 of ended task {writer}");
    assert!(words == expected.as_bytes() || words.starts_with(format!("{expected}; ").as_bytes()), "{words:?}");
    let mut freed = Queue::uncapped();
    step(&mut d, &due, Event::Withdraw { entry: 1 }, &mut freed);
    assert!(!asks(&collected(freed)).contains(&TASK), "a held landing is passed over");
    let mut released = Queue::uncapped();
    let heard = LandingHeard { released: true, ..preparing_heard(None) };
    step(&mut d, &due, Event::StepLanding { task: TASK, entry: 93, heard }, &mut released);
    assert_eq!(d.landing(TASK).unwrap().room, Some(1), "release retains the landing's place");
    assert!(asks(&collected(released)).contains(&TASK), "its place may be asked again");
}
