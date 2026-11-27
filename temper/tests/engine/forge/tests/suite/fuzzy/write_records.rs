//! Writes and lagging reads through the connector (temper/docs/design/domain/forge/writes.md, 3–6).
use std::collections::BTreeMap;

use skein_fake_forge_domain::{self as forge, Lag, Observation, api as raw};
use skein_forge::api::{self, Commit};
use skein_lib::{Duration, Time, Wall};
use skein_world::{
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, LANDING},
    restarts::{BRANCH as GOAL_BRANCH, GOAL, goal_name},
    translate,
};
use temper_faults::forge::{HEAD_BEHIND, TIMED_OUT};

const MAIN: &[u8] = b"main";
const OTHER: &[u8] = b"other";
const PRESSURE: u8 = 34;
const TICKS: u64 = 180;
const ACTOR: u64 = 2;
const DESIGN: u64 = 1;
const LOST: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outside {
    Push,
    Reset { hinted: bool },
    Close,
    Retarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Merge,
    Turn,
    Cancel,
    Outside(Outside),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Start,
    Open,
    Advance,
    Probe,
    Finish,
    Closing,
    Done,
}

#[derive(PartialEq, Eq)]
enum Merge {
    None,
    Made,
    Disputed,
}

struct Lane {
    task: u64,
    branch: &'static [u8],
    file: &'static [u8],
    role: Role,
    stage: Stage,
    waiting: bool,
    old: Option<u64>,
    tip: Option<Commit>,
    bound: Option<Wall>,
    outside: Option<top::DriftChange>,
    merge: Merge,
}

impl Lane {
    fn new(task: u64, branch: &'static [u8], file: &'static [u8], role: Role) -> Self {
        Self {
            task,
            branch,
            file,
            role,
            stage: Stage::Start,
            waiting: false,
            old: None,
            tip: None,
            bound: None,
            outside: None,
            merge: Merge::None,
        }
    }

    fn base(&self) -> &'static [u8] {
        match self.role {
            Role::Merge | Role::Turn => GOAL_BRANCH,
            Role::Cancel | Role::Outside(_) => MAIN,
        }
    }
}

fn name(branch: &[u8]) -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(branch.split(|&byte| byte == b'/').map(Box::from).collect()),
    }
}

fn attempt_done(world: &mut World, task: u64, attempt: u64, pushed: Option<(top::Name, Commit, bool)>) {
    world.event(top::Event::Finished { task, attempt, lost: pushed.is_none() });
    if let Some(pushed) = pushed {
        world.event(top::Event::Answered { task, attempt, pushed: Box::new([pushed]) });
    }
}

fn push(world: &mut World, name: &top::Name, attempt: (u64, u64), branch: &[u8], file: &[u8], data: &[u8]) -> Commit {
    let git = world.lease_attempt_push(name, attempt, branch, file, data);
    sent_push(world, git)
}

fn sent_push(world: &mut World, git: raw::Git) -> Commit {
    let (raw::Git::Push { commit, .. } | raw::Git::PushNew { commit, .. }) = &git else {
        unreachable!("a leased push")
    };
    let head = translate::commit(*commit);
    let expected = if matches!(git, raw::Git::PushNew { .. }) {
        raw::Answer::PushNew { existing: None }
    } else {
        raw::Answer::Pushed(raw::Pushed::Pushed)
    };
    assert_eq!(world.external_now(raw::Op::Git(git), 1), expected);
    head
}

fn producer(world: &mut World, lane: &Lane, attempt: u64) -> Commit {
    let branch = name(lane.branch);
    let task = lane.task + 100;
    world.event(top::Event::Delegated { task: lane.task, child: task, kind: change::Delegate::Produce });
    world.event(top::Event::Claim {
        task,
        attempt,
        writes: Box::new([branch.clone()]),
        designs: Box::new([]),
        holders: Box::new([lane.task]),
    });
    let data = attempt.to_be_bytes();
    let head = if attempt == 1 {
        let git = world.lease_new_attempt_push(&branch, (task, attempt), lane.branch, lane.base(), lane.file, &data);
        sent_push(world, git)
    } else {
        push(world, &branch, (task, attempt), lane.branch, lane.file, &data)
    };
    attempt_done(world, task, attempt, Some((branch, head, true)));
    head
}

fn start(world: &mut World, lane: &mut Lane) {
    let branch = name(lane.branch);
    world.event(top::Event::Hold { entry: 0, task: lane.task, resource: branch.clone(), from: None });
    world.event(top::Event::Names { task: lane.task, resources: Box::new([branch, name(lane.base())]) });
    let mut row = changes::change_row();
    row.task = lane.task;
    row.branch = lane.branch.into();
    row.base = lane.base().into();
    row.change.task = lane.task;
    row.change.way = if lane.role == Role::Turn { change::Way::Turn } else { change::Way::Queue };
    row.change.state = change::State::Producing { requested: true };
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
    let head = producer(world, lane, 1);
    lane.old = Some(world.branch(lane.branch));
    lane.tip = Some(head);
    lane.stage = Stage::Open;
}

fn step_change(world: &mut World, lane: &mut Lane, entry: u64) {
    lane.waiting = true;
    world.event(top::Event::StepChange {
        task: lane.task,
        entry,
        heard: change::Heard {
            review_request: None,
            closer_waiting: false,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Passed,
            delegate_running: false,
            flake: false,
            no_test: false,
            delegate_live: false,
            effect: change::EffectResult::None,
            released: false,
            cancelled: false,
        },
        gates: Box::new([]),
        merge_job: false,
        queue_repair_active: false,
        pending: Box::new([]),
        // Outside changes stay open, with a real caller, instead of merging into main.
        kept: matches!(lane.role, Role::Outside(_)),
    });
}

fn disturb(world: &mut World, lane: &mut Lane, outside: Outside) {
    let pull = world.change(lane.task).pull.expect("the change's numbered pull");
    let expected = lane.tip.expect("temper's last confirmed push");
    lane.outside = Some(match outside {
        Outside::Push => {
            world.push_now_as(lane.branch, b"foreign", b"not the worker", ACTOR);
            top::DriftChange::Branch { expected, observed: Some(translate::commit(world.branch(lane.branch))) }
        }
        Outside::Reset { hinted } => {
            let old = lane.old.expect("the previous own push");
            assert_ne!(world.branch(lane.branch), old);
            world.reset_now_as(lane.branch, old, ACTOR);
            let found = translate::commit(old);
            if hinted {
                world.event(top::Event::Hint {
                    hint: api::Hint {
                        repository: REPO,
                        change: api::Change::BranchMoved { branch: lane.branch.into(), head: found, actor: ACTOR },
                        key: None,
                    },
                });
            }
            top::DriftChange::Branch { expected, observed: Some(found) }
        }
        Outside::Close => {
            assert_eq!(
                world.external_now(raw::Op::Write(raw::Write::Close { number: pull }), ACTOR),
                raw::Answer::Done
            );
            top::DriftChange::PullClosed { number: pull }
        }
        Outside::Retarget => {
            world.retarget_now_as(pull, OTHER, ACTOR);
            top::DriftChange::Retargeted { number: pull, before: lane.base().into(), after: OTHER.into() }
        }
    });
}

fn cleanup(world: &mut World, task: u64, entry: u64) {
    let ending = top::ReleaseEnding::Cancelled;
    world.event(top::Event::SettleEffects { task, root: task, ending });
    world.event(top::Event::Release { task, root: task, ending, reason: None, entry });
    world.event(top::Event::Committed { entry });
}

fn design_claim(world: &mut World, attempt: u64) {
    world.event(top::Event::Claim {
        task: GOAL,
        attempt,
        writes: Box::new([]),
        designs: Box::new([goal_name()]),
        holders: Box::new([]),
    });
}

fn design_push(world: &mut World, n: u8) {
    push(world, &goal_name(), (GOAL, DESIGN), GOAL_BRANCH, b"design.md", &[n]);
}

fn landing_step(world: &mut World, entry: u64) {
    world.event(top::Event::StepLanding {
        task: LANDING,
        entry,
        heard: top::LandingHeard {
            job: None,
            outcomes: Box::new([]),
            dependencies: Box::new([]),
            held: Box::new([]),
            ended: Box::new([]),
            failure: Box::new([]),
            preparation: Some(landing::Preparation::Idle),
            released: false,
            cancelled: false,
            kept: Box::new([]),
        },
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Act {
    Change(usize),
    Design,
    Landing,
    Restart,
    Read,
}

#[derive(Debug, PartialEq, Eq)]
struct Point {
    at: Time,
    act: Act,
    observations: Vec<Observation>,
    calls: Vec<api::Op>,
    records: Vec<top::Request>,
    decisions: Vec<top::Request>,
}

/// The referee does not use the connector's judgement to classify the
/// stimulus. Each lane knows which write it gave the forge, if any.
struct Referee {
    rows: BTreeMap<top::Name, top::Writes>,
    trace: Vec<Point>,
    withheld: Option<Commit>,
    design_ended: bool,
    peak: usize,
    evicted: bool,
    behind: bool,
}

impl Referee {
    fn new() -> Self {
        Self {
            rows: BTreeMap::new(),
            trace: Vec::new(),
            withheld: None,
            design_ended: false,
            peak: 0,
            evicted: false,
            behind: false,
        }
    }

    fn records(&mut self, point: &Point) -> Result<(), String> {
        for request in &point.records {
            match request {
                top::Request::Save { record: top::Stored::Writes(row), .. } => {
                    if row.entries.len() > 32 {
                        return Err(format!("overfull write record: {row:?}"));
                    }
                    self.peak = self.peak.max(row.entries.len());
                    if let Some(previous) = self.rows.insert(row.name.clone(), row.clone()) {
                        for old in &previous.entries {
                            if row.entries.iter().any(|new| new.order == old.order && new.sender == old.sender) {
                                continue;
                            }
                            if old.state.unsettled() {
                                // This lease was admitted but never sent. Its attempt ended and
                                // fresh readback on the advanced branch can refuse it.
                                let withheld = self.design_ended
                                    && matches!(old.operation, top::writes::Operation::Push { after, .. }
                                        if Some(after) == self.withheld);
                                let refused = point.decisions.iter().any(|request| matches!(request,
                                    top::Request::Outcome { task, entry, outcome: skein_forge_client::Outcome::Failed(_), .. }
                                        if old.sender == (top::Sender::Entry { task: *task, entry: *entry })));
                                if !withheld && !refused {
                                    return Err(format!("unsettled write disappeared: {old:?} from {:?}", row.name));
                                }
                            } else {
                                self.evicted = true;
                            }
                        }
                    }
                }
                top::Request::Erase { key: top::Key::Writes(name) } => {
                    if self.rows.remove(name).is_some_and(|row| row.entries.iter().any(|entry| entry.state.unsettled()))
                    {
                        return Err(format!("erased record with unsettled writes: {name:?}"));
                    }
                }
                _ => return Err(format!("unexpected record observation: {request:?}")),
            }
        }
        Ok(())
    }

    fn change(
        lane: &mut Lane,
        decision: &change::Decision,
        evidence: Option<&top::ChangeEvidence>,
        at: Wall,
    ) -> Result<(), String> {
        let behind = evidence.and_then(|evidence| evidence.behind);
        if let change::Decision::Hold(why) = decision {
            // The client can report a made merge as Raced when its verifying
            // pull still reads open. This hold is not drift; the turn and
            // keep must still explain the merge from the forge.
            if *why == change::Hold::Failed && lane.merge == Merge::Disputed {
                lane.stage = Stage::Done;
                return Ok(());
            }
            let wants = match (lane.role, lane.outside.is_some()) {
                (Role::Outside(Outside::Push | Outside::Reset { hinted: true }), true) => change::Hold::BranchMoved,
                (Role::Outside(Outside::Reset { hinted: false }), true) => change::Hold::Stalled,
                (Role::Outside(Outside::Close), true) => change::Hold::PullClosed,
                (Role::Outside(Outside::Retarget), true) => change::Hold::Retargeted,
                _ => return Err(format!("unexplained hold of {}: {why:?}; {evidence:?}", lane.task)),
            };
            if *why != wants {
                return Err(format!("outside write {:?}: wanted {wants:?}, got {why:?}; {evidence:?}", lane.role));
            }
            if *why == change::Hold::Stalled {
                let Some(bound) = lane.bound else { return Err("reset held without waiting for its bound".into()) };
                let Some(top::DriftChange::Branch { expected, observed }) = lane.outside.as_ref() else {
                    return Err("can't tell without a known outside reset".into());
                };
                let want =
                    top::ReadBehind { read: top::BehindRead::Branch, expected: Some(*expected), found: *observed };
                if behind != Some(want) || at < bound || at.as_nanos() > bound.as_nanos().saturating_add(1_000_000_000)
                {
                    return Err(format!(
                        "reset did not hold as can't tell at its bound {bound:?}: {at:?} {evidence:?}"
                    ));
                }
            }
            lane.stage = Stage::Done;
            lane.bound = None;
        } else if !matches!(decision, change::Decision::Wait { .. }) {
            lane.bound = None;
        }
        Ok(())
    }

    fn quiet(&mut self, world: &World, lanes: &mut [Lane]) -> Result<(), String> {
        world.check()?;
        if let Some(drift) = world.hold(&goal_name()).and_then(|hold| hold.drift.as_ref()) {
            return Err(format!("temper's goal branch held as drift: {drift:?}"));
        }
        for lane in lanes {
            if let Some(until) = world.change_behind_bound(lane.task) {
                self.behind = true;
                let bound = *lane.bound.get_or_insert(until);
                if until > bound || world.wall() >= bound {
                    return Err(format!(
                        "change {} still behind past its caller's bound {bound:?} (now {:?}, current bound {until:?})",
                        lane.task,
                        world.wall()
                    ));
                }
            } else if lane.outside.is_none() {
                lane.bound = None;
            }
            if let Some(drift) = world.hold(&name(lane.branch)).and_then(|hold| hold.drift.as_ref())
                && (lane.outside.as_ref() != Some(&drift.change)
                    || lane.role == Role::Outside(Outside::Reset { hinted: false }))
            {
                return Err(format!(
                    "wrong drift on change {}: {drift:?}; outside write: {:?}",
                    lane.task, lane.outside
                ));
            }
        }
        Ok(())
    }
}

fn next_entry(entry: &mut u64) -> u64 {
    *entry += 10;
    *entry
}

fn observed_point(world: &mut World, act: Act) -> Point {
    Point {
        at: world.now(),
        act,
        observations: world.observations(),
        calls: world.take_calls(),
        records: world.take_write_records(),
        decisions: world.take_seen().into_vec(),
    }
}

fn ready_to_disturb(world: &World, lane: &Lane) -> bool {
    lane.bound.is_none()
        && world.change(lane.task).change.last_head == lane.tip
        && world.change(lane.task).pending.is_none()
        && matches!(world.stored().get(&top::Key::Writes(name(lane.branch))),
            Some(top::Stored::Writes(row)) if row.head == lane.tip)
}

fn change_act(world: &mut World, lane: &mut Lane, entry: u64, inject: bool) {
    match lane.stage {
        Stage::Start => start(world, lane),
        Stage::Open | Stage::Finish => step_change(world, lane, entry),
        Stage::Advance => {
            lane.tip = Some(producer(world, lane, 2));
            lane.stage = Stage::Probe;
        }
        Stage::Probe => {
            if let Role::Outside(outside) = lane.role
                && lane.outside.is_none()
                && inject
                && ready_to_disturb(world, lane)
            {
                disturb(world, lane, outside);
            }
            step_change(world, lane, entry);
        }
        Stage::Closing | Stage::Done => unreachable!("the scheduler uses eligible lanes"),
    }
}

fn change_request(world: &mut World, lane: &mut Lane, next: &mut u64, request: &top::Request) {
    match request {
        top::Request::ChangeDecision { decision, entry, .. } => {
            lane.waiting = false;
            if let Some(entry) = *entry {
                world.event(top::Event::Committed { entry });
            }
            if lane.stage == Stage::Open && world.change(lane.task).pull.is_some() {
                lane.stage = match lane.role {
                    Role::Outside(Outside::Reset { .. }) => Stage::Advance,
                    Role::Outside(_) => Stage::Probe,
                    Role::Merge | Role::Turn | Role::Cancel => Stage::Finish,
                };
            }
            if lane.stage == Stage::Finish {
                match (lane.role, decision) {
                    (Role::Turn, change::Decision::Ready) => {
                        let row = landings::landing_row(world.change(lane.task));
                        landings::register(world, row);
                        lane.stage = Stage::Done;
                    }
                    (Role::Merge, change::Decision::Finish { .. }) | (Role::Cancel, _) => {
                        cleanup(world, lane.task, next_entry(next));
                        lane.stage = Stage::Closing;
                    }
                    _ => {}
                }
            }
        }
        top::Request::ContinueRelease { .. } => {
            let entry = next_entry(next);
            world.event(top::Event::ContinueRelease { task: lane.task, entry });
            world.event(top::Event::Committed { entry });
        }
        top::Request::Released { .. } => lane.stage = Stage::Done,
        _ => {}
    }
}

fn landing_request(world: &mut World, request: &top::Request) -> Result<bool, String> {
    let top::Request::LandingDecision { decision, entry, .. } = request else { unreachable!("a landing decision") };
    if let Some(entry) = *entry {
        world.event(top::Event::Committed { entry });
    }
    match decision {
        landing::Decision::Prepare { .. } => {
            world.prepare_landing(LANDING, false);
        }
        landing::Decision::Hold(why) => return Err(format!("own landing held: {why:?}")),
        _ => {}
    }
    Ok(*decision == landing::Decision::Finish)
}

fn requests(world: &mut World, lanes: &mut [Lane], driver: &mut Driver, point: &Point) -> Result<(), String> {
    // Evidence independent of the connector: the fake really moved the
    // goal's branch, even if the write answer was disputed.
    if let Some(pull) = (lanes[0].stage != Stage::Start).then(|| world.change(lanes[0].task).pull).flatten()
        && point.observations.iter().any(|event| {
            matches!(event,
            Observation::Merged { number, base, by: 1, .. } if *number == pull && base.as_ref() == GOAL_BRANCH)
        })
    {
        lanes[0].merge = Merge::Made;
    }
    for request in &point.decisions {
        let task = match request {
            top::Request::ChangeDecision { task, decision, evidence, .. } => {
                let lane = lanes.iter_mut().find(|lane| lane.task == *task).expect("scripted change");
                Referee::change(lane, decision, evidence.as_ref(), world.wall())?;
                *task
            }
            top::Request::Outcome {
                task,
                outcome: skein_forge_client::Outcome::Raced { made: skein_forge_client::Made::Merged(_), .. },
                ..
            } if lanes[0].task == *task && lanes[0].merge == Merge::Made => {
                lanes[0].merge = Merge::Disputed;
                continue;
            }
            top::Request::ContinueRelease { task } | top::Request::Released { task } => *task,
            top::Request::ReleaseFailed { .. } | top::Request::MakeFailed { .. } => {
                return Err(format!("temper's own write failed: {request:?}"));
            }
            top::Request::LandingDecision { .. } => {
                driver.landing_waiting = false;
                driver.landed |= landing_request(world, request)?;
                continue;
            }
            top::Request::ContinueGoalBranch { task: GOAL } => {
                world.event(top::Event::ContinueGoalBranch { task: GOAL, entry: next_entry(&mut driver.entry) });
                continue;
            }
            _ => continue,
        };
        let lane = lanes.iter_mut().find(|lane| lane.task == task).expect("scripted change or cleanup");
        change_request(world, lane, &mut driver.entry, request);
    }
    Ok(())
}

#[derive(Default)]
struct Driver {
    entry: u64,
    design: u8,
    restarts: u8,
    landing_waiting: bool,
    landed: bool,
    turned: bool,
}

impl Driver {
    fn actions(&self, lanes: &[Lane], restart: bool) -> Vec<Act> {
        let mut actions = vec![Act::Read];
        for (i, lane) in lanes.iter().enumerate() {
            if lane.waiting || matches!(lane.stage, Stage::Closing | Stage::Done) {
                continue;
            }
            match (lane.role, lane.stage) {
                (Role::Merge, Stage::Finish) if self.design == 0 => {}
                (Role::Turn, _) if !self.turned => {}
                _ => actions.push(Act::Change(i)),
            }
        }
        if (self.design == 0 && lanes[0].stage == Stage::Finish)
            || (self.design < PRESSURE && self.landed && lanes[0].stage == Stage::Done)
        {
            actions.push(Act::Design);
        }
        if !self.landing_waiting && !self.landed && lanes[1].stage == Stage::Done && lanes[0].stage == Stage::Done {
            actions.push(Act::Landing);
        }
        if restart && self.restarts < 3 {
            actions.push(Act::Restart);
        }
        actions
    }
}

fn setup(seed: u64) -> World {
    let mut config = temper_engine_forge_world::fake_config();
    config.limits.commits = 64;
    config.limits.statuses = 16;
    let checks = raw::Checks {
        contexts: Box::new([b"build".as_slice().into()]),
        cues: Box::new([]),
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        silent: 0,
        passes: 1000,
        reruns: 0,
        cue: None,
    };
    let mut world = World::batch_turn_with_checks(&Streams::new(seed), None, config, Some(checks));
    world.adopt();
    assert!(world.project_landing(landings::PROJECT, change::Way::Queue, Box::new([])));
    world.landing_requirements(GOAL_BRANCH, false, true);
    assert_eq!(
        world.external_now(raw::Op::Git(raw::Git::Create { branch: OTHER.into(), commit: world.branch(MAIN) }), ACTOR),
        raw::Answer::Branch(raw::Created::Created)
    );
    world
}

#[expect(clippy::too_many_lines, reason = "one run ties each drawn action to its durable and forge observations")]
fn run(seed: u64, outside: Outside) -> Result<Vec<Point>, String> {
    let streams = Streams::new(seed);
    let mut scheduling = streams.rng(Stream::Scheduling, "write records");
    let mut parties = streams.rng(Stream::Parties, "write records");
    let mut faults = streams.rng(Stream::Faults, "write records");
    let mut world = setup(seed);
    let mut lanes = [
        Lane::new(51, b"temper/51", b"merge", Role::Merge),
        Lane::new(52, b"temper/52", b"turn", Role::Turn),
        Lane::new(53, b"temper/53", b"cancel", Role::Cancel),
        Lane::new(54, b"temper/54", b"outside", Role::Outside(outside)),
    ];
    let mut driver = Driver { entry: 1000, ..Driver::default() };
    let mut referee = Referee::new();
    world.take_write_records();
    world.event(top::Event::Hold { task: GOAL, resource: goal_name(), entry: 1, from: None });
    world.run_for(1);
    if !world.has_branch(GOAL_BRANCH) {
        return Err("goal's own create did not reach the forge".into());
    }
    world.event(top::Event::Names { task: GOAL, resources: Box::new([goal_name()]) });
    design_claim(&mut world, DESIGN);
    let withheld = world.lease_attempt_push(&goal_name(), (GOAL, DESIGN), GOAL_BRANCH, b"unused", b"not sent");
    if let raw::Git::Push { commit, .. } = withheld {
        referee.withheld = Some(translate::commit(commit));
    }
    forge::lag(world.forge_mut(), Lag { chance: 1000, min: Duration::from_secs(2), max: Duration::from_secs(4) });
    world.timeout_next_write();
    for tick in 0..TICKS {
        let actions = driver.actions(&lanes, tick > 3 && faults.chance(150));
        let last = u64::try_from(actions.len() - 1).expect("action count");
        let chosen = usize::try_from(scheduling.between(0, last)).expect("an action index");
        let act = actions[chosen];
        let entry = next_entry(&mut driver.entry);
        match act {
            Act::Change(index) => change_act(&mut world, &mut lanes[index], entry, parties.chance(650)),
            Act::Design => {
                driver.design += 1;
                design_push(&mut world, driver.design);
            }
            Act::Landing => {
                driver.landing_waiting = true;
                landing_step(&mut world, entry);
            }
            Act::Restart => {
                driver.restarts += 1;
                world.restart();
                for lane in &mut lanes {
                    lane.waiting = false;
                }
                driver.landing_waiting = false;
            }
            Act::Read => {}
        }
        world.run_for(1);
        let point = observed_point(&mut world, act);
        referee.records(&point).map_err(|why| format!("step {tick} {act:?}: {why}"))?;
        requests(&mut world, &mut lanes, &mut driver, &point).map_err(|why| format!("step {tick} {act:?}: {why}"))?;
        if !driver.turned && lanes[0].stage == Stage::Done {
            assert!(world.project_landing(landings::PROJECT, change::Way::Turn, Box::new([])));
            driver.turned = true;
        }
        referee.quiet(&world, &mut lanes).map_err(|why| format!("step {tick} {act:?}: {why}"))?;
        referee.trace.push(point);
        if lanes.iter().all(|lane| lane.stage == Stage::Done) && driver.landed && driver.design == PRESSURE {
            break;
        }
    }
    if lanes.iter().any(|lane| lane.stage != Stage::Done) || !driver.landed || driver.design != PRESSURE {
        return Err(format!(
            "work did not finish within {TICKS} steps: {:?}; landed {}; design {}",
            lanes.each_ref().map(|lane| lane.stage),
            driver.landed,
            driver.design
        ));
    }
    referee.design_ended = true;
    let head = translate::commit(world.branch(GOAL_BRANCH));
    attempt_done(&mut world, GOAL, DESIGN, Some((goal_name(), head, true)));
    world.run_for(2);
    design_claim(&mut world, LOST);
    world.lease_attempt_push(&goal_name(), (GOAL, LOST), GOAL_BRANCH, b"unused", b"lease still live");
    world.script_branch_error(GOAL_BRANCH, Some(api::Error::Unavailable));
    attempt_done(&mut world, GOAL, LOST, None);
    world.run_for(1);
    let point = observed_point(&mut world, Act::Read);
    referee.records(&point)?;
    referee.quiet(&world, &mut lanes)?;
    referee.trace.push(point);
    let mut lost_readback = false;
    for row in referee.rows.values() {
        for entry in &row.entries {
            if entry.state.unsettled() {
                if !world.write_readback_owed(&row.name, entry.order) {
                    return Err(format!("no sender readback owed at settle: {entry:?} on {:?}", row.name));
                }
                lost_readback |= row.name == goal_name()
                    && entry.sender == (top::Sender::Attempt { task: GOAL, attempt: LOST })
                    && entry.state == top::WriteState::Uncertain;
            }
        }
    }
    if !lost_readback || referee.peak != 32 || !referee.evicted || driver.restarts == 0 {
        return Err(format!(
            "missing pressure coverage: lost readback {lost_readback}, peak {}, evicted {}, restarts {}",
            referee.peak, referee.evicted, driver.restarts
        ));
    }
    if matches!(outside, Outside::Reset { hinted: false }) && !referee.behind {
        return Err("the reset never exercised the read-behind wait".into());
    }
    exercised(&world, &lanes, &referee)?;
    let mut sweep = Sweep::new("lagging_reads_distinguish_temper_writes_from_drift");
    sweep.configure(HEAD_BEHIND);
    sweep.configure(TIMED_OUT);
    sweep.ran(world.ledger());
    sweep.check_fell()?;
    Ok(referee.trace)
}

// Coverage is checked against the fake's mutations, not only calls or
// successful decisions. For example, a refused update isn't an update.
fn exercised(world: &World, lanes: &[Lane], referee: &Referee) -> Result<(), String> {
    let observed = || referee.trace.iter().flat_map(|point| &point.observations);
    let cancel = &lanes[2];
    let opened = observed().find_map(|event| match event {
        Observation::Opened { number, branches: Some(branches), by: 1, .. }
            if branches.head.as_ref() == cancel.branch =>
        {
            Some(*number)
        }
        _ => None,
    });
    let closed =
        observed().any(|event| matches!(event, Observation::Closed { number, by: 1, .. } if Some(*number) == opened));
    let deleted = observed()
        .any(|event| matches!(event, Observation::Deleted { branch, by: 1, .. } if branch.as_ref() == cancel.branch));
    let updated = referee.trace.iter().flat_map(|point| &point.calls).any(|op| matches!(op, api::Op::Write(api::Write::Update { .. })))
        && observed().any(|event| matches!(event, Observation::Moved { branch, from: Some(_), by: 1, .. } if branch.as_ref() == lanes[0].branch));
    let round = &world.landing(LANDING).landing;
    let turned = round.phase == landing::Phase::Landed
        && observed().any(|event| matches!(event, Observation::Merged { base, number, by: 1, .. } if base.as_ref() == GOAL_BRANCH && Some(*number) == round.round.pull));
    let goal_tip = referee.rows.get(&goal_name()).and_then(|row| row.head);
    if !closed
        || !deleted
        || !updated
        || !turned
        || lanes[0].merge == Merge::None
        || goal_tip != Some(translate::commit(world.branch(GOAL_BRANCH)))
    {
        return Err(format!(
            "did not make all writes: close {closed}, delete {deleted}, update {updated}, turn {turned}, queue merge {}, goal head {goal_tip:?}",
            lanes[0].merge != Merge::None
        ));
    }
    assert!(lanes[3].outside.is_some(), "the outside write reached the fake");
    Ok(())
}

#[test]
fn lagging_reads_distinguish_temper_writes_from_drift() {
    use Outside::{Close, Push, Reset, Retarget};
    for (seed, outside) in [
        (31, Push),
        (34, Close),
        (37, Reset { hinted: true }),
        (48, Reset { hinted: false }),
        (50, Retarget),
        (62, Reset { hinted: true }),
    ] {
        eprintln!("write records seed {seed}: {outside:?}");
        let trace = run(seed, outside);
        assert_eq!(trace, run(seed, outside), "seed {seed} replays the same calls, writes, and decisions");
        let trace = trace.unwrap_or_else(|finding| panic!("seed {seed}: {finding}"));
        eprintln!("seed {seed}: {} quiet points", trace.len());
    }
}

#[test]
fn a_disputed_merge_and_later_turn_do_not_make_a_design_push_drift() {
    let seed = 32;
    let outside = Outside::Reset { hinted: true };
    eprintln!("write records seed {seed}: {outside:?}");
    let run_once = run(seed, outside);
    assert_eq!(run_once, run(seed, outside));
    run_once.expect("a late merge verification must not make a subsequent own push drift");
}

#[test]
fn restarting_a_caller_behind_does_not_renew_its_bound() {
    let seed = 33;
    let outside = Outside::Reset { hinted: false };
    eprintln!("write records seed {seed}: {outside:?}");
    let run_once = run(seed, outside);
    assert_eq!(run_once, run(seed, outside));
    run_once.expect("a silent reset must still finish within the original bound");
}

#[test]
fn a_disputed_queue_merge_keeps_its_pull_readback_owed() {
    let seed = 35;
    let outside = Outside::Retarget;
    eprintln!("write records seed {seed}: {outside:?}");
    let run_once = run(seed, outside);
    assert_eq!(run_once, run(seed, outside));
    run_once.expect("both records of a queue merge must settle or retain a readback");
}
