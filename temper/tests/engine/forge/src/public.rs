//! Scripted public core and forbidden sources (temper/docs/design/client/forge/README.md, 9.5).
use crate::{REPO, World, changes, landings};
use skein_fake_forge_domain::api as raw;
use skein_lib::{Duration, Reader};
use temper_client_protocol_wire::public_v1 as w;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_landing as policy;
pub const SEED: u64 = 409;
pub const GOAL: u64 = 0x5EC0_0000_0000_0100;
pub const CHANGE: u64 = 0x5EC0_0000_0000_0101;
pub const SECOND: u64 = 0x5EC0_0000_0000_0102;
pub const LANDING: u64 = 0x5EC0_0000_0000_0110;
pub const BRANCH: &[u8] = b"temper/secret-public-branch";
pub const OTHER_BRANCH: &[u8] = b"temper/secret-second-branch";
pub const GATE: &[u8] = b"secret-public-gate";
const MARKERS: [&[u8]; 7] = [
    BRANCH,
    OTHER_BRANCH,
    GATE,
    b"secret-public-title",
    b"secret-public-body",
    b"secret-public-question",
    b"secret-public-reason",
];
#[must_use]
pub fn handed() -> [top::PublicTask; 3] {
    [
        top::PublicTask { task: GOAL, goal: Some(GOAL) },
        top::PublicTask { task: CHANGE, goal: Some(GOAL) },
        top::PublicTask { task: SECOND, goal: Some(GOAL) },
    ]
}
#[must_use]
pub fn world(passes: u32) -> World {
    let mut world = World::public_turn(
        SEED,
        raw::Workflow {
            fails: Box::new([]),
            flaky: Box::new([]),
            report: None,
            select: None,
            file: Box::from(b"gate.yml".as_slice()),
            jobs: Box::new([Box::from(b"test".as_slice())]),
            latency_min: Duration::from_secs(2),
            latency_max: Duration::from_secs(2),
            silent: 0,
            passes,
            cue: None,
        },
    );
    world.adopt();
    let gates = Box::new([policy::Gate {
        name: Box::from(GATE),
        workflow: Box::from(b"gate.yml".as_slice()),
        ignored: Box::new([]),
        blocking: true,
        base: false,
    }]);
    assert!(world.project_landing(
        landings::PROJECT,
        c::Way::Turn,
        Box::new([
            top::RepositoryGates { repository: REPO, tiers: Box::new([]), gates: gates.clone() },
            top::RepositoryGates {
                repository: skein_forge::api::Repository { forge: REPO.forge, repository: 3 },
                tiers: Box::new([]),
                gates
            }
        ])
    ));
    world
}
pub fn mark(row: &mut top::ChangeRow) {
    row.title = Box::from(MARKERS[3]);
    row.body = Box::from(MARKERS[4]);
    row.producer_failure = Some(Box::from(MARKERS[6]));
    row.questions.open = Box::new([c::Open {
        asker: SECOND,
        number: 1,
        line: Box::from(MARKERS[5]),
        words: Box::from(MARKERS[5]),
        stands: c::Stands::Taken,
    }]);
}
pub fn ready(world: &mut World, task: u64, branch: &[u8]) {
    if task == SECOND {
        world.public_second(task, branch);
        return;
    }
    world.produce(branch);
    let pull = world.open_pull(branch, b"main");
    let head = crate::translate::commit(world.branch(branch));
    let mut row = changes::change_row();
    row.task = task;
    row.change.task = task;
    row.branch = Box::from(branch);
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = c::State::Gating { head, asked: Box::new([]) };
    mark(&mut row);
    if !world.stored().contains_key(&top::Key::Change(task)) {
        world.event(top::Event::Change { row });
    }
    changes::change_step_task(world, task, 1, c::Status::Passed, Box::new([]), false, false);
}
pub fn queued(world: &mut World) {
    world.produce(BRANCH);
    let pull = world.open_pull(BRANCH, b"main");
    let head = crate::translate::commit(world.branch(BRANCH));
    let mut row = changes::change_row();
    row.task = CHANGE;
    row.change.task = CHANGE;
    row.branch = Box::from(BRANCH);
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = c::State::Queued { head, ready_since: world.wall() };
    mark(&mut row);
    world.event(top::Event::Change { row });
}

pub fn install_landing(world: &mut World, two: bool) {
    let mut row = landings::landing_row(world.change(CHANGE));
    row.landing.task = LANDING;
    if two {
        let mut second = row.landing.members.first().expect("member").clone();
        second.repository.repository = 3;
        second.change = SECOND;
        second.pull = world.change(SECOND).pull.expect("pull");
        row.landing.members = Box::new([row.landing.members.first().expect("member").clone(), second]);
        row.asks = Box::new([None, None]);
    }
    world.event(top::Event::Landing { row });
}
#[must_use]
pub fn answer(world: &World, kind: u16, task: Option<u64>, public: &[top::PublicTask]) -> top::PublicMade {
    let made = world.public(landings::PROJECT, kind, task, public).expect("public answer");
    clean(&made.record);
    made
}
pub fn clean(bytes: &[u8]) {
    for marker in MARKERS {
        assert!(!bytes.windows(marker.len()).any(|part| part == marker), "seed {SEED}: {marker:?}");
    }
    for task in [GOAL, CHANGE, SECOND, LANDING] {
        assert!(!bytes.windows(8).any(|part| part == task.to_be_bytes()), "seed {SEED}: big-endian task");
        assert!(!bytes.windows(8).any(|part| part == task.to_le_bytes()), "seed {SEED}: internal task");
        let decimal = task.to_string();
        assert!(!bytes.windows(decimal.len()).any(|part| part == decimal.as_bytes()), "seed {SEED}: decimal task");
    }
}
#[must_use]
pub fn change(world: &World) -> w::PublicChange {
    let made = answer(world, w::PUBLIC_CHANGE, Some(CHANGE), &handed());
    w::PublicChange::decode(&w::CEILINGS, &mut Reader::new(&made.record)).expect("public change")
}
#[must_use]
pub fn step(world: &mut World, position: u64, two: bool) -> policy::Decision {
    let entry = 100_u64.checked_add(position.checked_mul(3).expect("bounded steps")).expect("bounded entry");
    let choice = landings::step_landing(world, LANDING, entry);
    for (task, offset) in [(CHANGE, 1), (SECOND, 2)] {
        if task == SECOND && !two {
            continue;
        }
        changes::change_step_task(
            world,
            task,
            entry.checked_add(offset).expect("entry"),
            c::Status::Passed,
            Box::new([]),
            false,
            false,
        );
    }
    choice
}

/// A stock queue's change, landed through the fake forge and observed by its procedure.
#[must_use]
pub fn queue_merge() -> World {
    let mut world = World::new(SEED);
    world.adopt();
    world.slow_calls(Duration::ZERO);
    world.produce(BRANCH);
    world.status(BRANCH, raw::Check::Passed);
    let pull = world.open_pull(BRANCH, b"main");
    let head = crate::translate::commit(world.branch(BRANCH));
    let mut row = changes::change_row();
    row.task = CHANGE;
    row.change.task = CHANGE;
    row.branch = Box::from(BRANCH);
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = c::State::Queued { head, ready_since: world.wall() };
    mark(&mut row);
    world.event(top::Event::Change { row });
    // The change procedure takes the queue's turn, merges, and observes the commit.
    for entry in 1..=8 {
        changes::change_step_task(&mut world, CHANGE, entry, c::Status::Passed, Box::new([]), false, false);
        if matches!(world.change(CHANGE).change.state, c::State::Landed { .. }) {
            break;
        }
    }
    world
}
