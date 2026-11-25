//! Connector reports in the core world (jig/docs/design/domain/engine/waits.md, 6 and 7).
use jig_core_world::{
    fixtures::{PARTY, delegate, member},
    reads,
    world::{Setup, World},
};
use jig_domain_core::{
    self as core, AtBound, Bound, Nearest, Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject,
};
use jig_domain_tasks as tasks;
use jig_test_connector::ProcedureAction;
use jig_test_domain as root;
use skein_lib::{Duration, List, Time, Wall};

const STALL: Duration = Duration::from_secs(60);
const ROOM: Duration = Duration::from_secs(20);
const FIRST_WALL: Wall = Wall::from_nanos(1_000_000_000);

fn fixture(seed: u64, action: ProcedureAction) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_core_world::faults::plan_fixture(seed);
    config.first.procedures[0].actions = Box::new([action]);
    config.first.procedures[0].stall = STALL;
    limits.core.waits.room = ROOM;
    (config, limits)
}

fn world(seed: u64, action: ProcedureAction) -> World {
    let (configuration, limits) = fixture(seed, action);
    World::new(Setup {
        seed,
        configuration: Some(configuration),
        limits,
        clock: Some((Time::ZERO, FIRST_WALL)),
        ..Setup::default()
    })
}

fn procedure(world: &mut World, completion: u32, deadline: Option<Wall>) -> u64 {
    let run = world.assignments[0];
    let mut child = member(world, tasks::Executor::Procedure { connector: 1, code: 1 }, b"report waits");
    child.authority.budget.deadline = deadline;
    delegate(world, run, completion, child)
}

fn at(since: Wall, after: Duration) -> Wall {
    Wall::from_nanos(since.as_nanos() + after.as_nanos())
}

fn report(world: &mut World, connector: u16, task: u64, waits: Box<[Reported]>) {
    world.send(root::Event::Core(core::Event::Waits { connector, task, waits: List::from_boxed(waits) }));
}

fn reported(kind: WaitKind, since: Wall, bound: ReportedBound, things: Box<[Thing]>) -> Reported {
    Reported { kind, since, bound, object: WaitObject::Procedure { step: 0, things: List::from_boxed(things) } }
}

fn step_wait(world: &World, task: u64) -> core::Wait {
    let waits = world.domain.core().waits(task);
    assert_eq!(waits.count, 1);
    let wait = waits.first.expect("the procedure's first wait");
    assert_eq!(wait.kind, WaitKind::Step);
    assert_eq!(waits.own.as_slice(), std::slice::from_ref(&wait));
    wait
}

#[test]
fn a_fake_procedures_clock_is_its_own_on_the_document_and_line() {
    let mut world = world(13601, ProcedureAction::ReportWait { state: 5 });
    let since = world.clocks().1;
    let due = at(since, STALL);
    let task = procedure(&mut world, 1, Some(at(since, ROOM)));
    let wait = step_wait(&world, task);
    assert_eq!(wait.since, since);
    assert_eq!(wait.bound, Bound::At { at: due, then: AtBound::Hold });
    world.reads = Some(reads::Parties::new(world.store.applied));
    reads::ask(&mut world, 1, core::reads::Read::Task { task });
    let core::reads::Answer::Document(document) = reads::latest(&world) else { panic!("task read") };
    let core::documents::Document::Task(document) = document.as_ref() else { panic!("task document") };
    assert_eq!(document.wait_count, 1);
    assert_eq!(document.waits.as_slice(), std::slice::from_ref(&wait));
    let core = world.domain.core();
    let row = core.tasks.task(task).expect("live procedure");
    let line = core::documents::task_line(core, &world.env().limits.core, PARTY, row, None).expect("visible line");
    assert_eq!(line.wait, Some(wait));
    assert_eq!(line.nearest, Some(Nearest { at: due, then: AtBound::Hold, kind: WaitKind::Step }));

    world.domain.script_wait_reports();
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::WakeProcedure { task })));
    assert!(world.domain.take_wait_report().is_none(), "the same clock is not reported again");
}

#[test]
fn room_counts_from_floored_since_and_turn_keeps_the_task_ahead() {
    let mut world = world(13602, ProcedureAction::Stall);
    let since = world.clocks().1;
    let earlier = at(since, Duration::from_secs(4));
    let first = procedure(&mut world, 1, None);
    let next = procedure(&mut world, 2, Some(earlier));
    for (task, due) in [(first, at(since, ROOM)), (next, earlier)] {
        let room = reported(WaitKind::Room, Wall::EPOCH, ReportedBound::Figure(AtBound::Hold), Box::new([]));
        let turn = Reported {
            kind: WaitKind::Turn,
            object: WaitObject::Task(world.assignments[0].0),
            since,
            bound: ReportedBound::None(Unbounded::Behind(world.assignments[0].0)),
        };
        report(&mut world, 1, task, Box::new([turn.clone(), room]));
        let waits = world.domain.core().waits(task);
        let [room, behind] = waits.own.as_slice() else { panic!("only the reported room and turn: {waits:?}") };
        assert_eq!(room.kind, WaitKind::Room);
        assert_eq!(room.since, since);
        assert_eq!(room.bound, Bound::At { at: due, then: AtBound::Hold });
        assert_eq!(waits.first.as_ref(), Some(room));
        assert_eq!(waits.nearest, Some(Nearest { at: due, then: AtBound::Hold, kind: WaitKind::Room }));
        assert_eq!(behind.kind, WaitKind::Turn);
        assert_eq!(behind.object, turn.object);
        assert_eq!(behind.bound, Bound::None(Unbounded::Behind(world.assignments[0].0)));
    }
}

#[test]
fn an_empty_wait_report_returns_the_task_to_news() {
    let mut world = world(13603, ProcedureAction::ReportWait { state: 5 });
    let task = procedure(&mut world, 1, None);
    let _wait = step_wait(&world, task);
    report(&mut world, 1, task, Box::new([]));
    let waits = world.domain.core().waits(task);
    assert_eq!(waits.count, 1);
    assert_eq!(waits.first.expect("news fallback").kind, WaitKind::News);
}

#[test]
fn an_invalid_wait_report_is_refused_whole_and_the_sender_is_told_why() {
    let mut world = world(13604, ProcedureAction::ReportWait { state: 5 });
    let agent = world.assignments[0].0;
    let procedure = procedure(&mut world, 1, None);
    let since = world.clocks().1;
    let full = (0..4)
        .map(|step| Reported {
            kind: WaitKind::Step,
            object: WaitObject::Procedure {
                step,
                things: List::from_boxed(Box::new([1, 2, 3, 4].map(|number| Thing { what: 1, number, owner: None }))),
            },
            since,
            bound: ReportedBound::At { at: at(since, STALL), then: AtBound::Hold },
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    report(&mut world, 1, procedure, full);
    let before = world.domain.core().waits(procedure);
    assert_eq!(before.count, 4);
    for wait in &before.own {
        let WaitObject::Procedure { things, .. } = &wait.object else { panic!("procedure wait") };
        assert_eq!(things.len(), 4);
    }
    let agent_before = world.domain.core().waits(agent);
    let room = reported(WaitKind::Room, world.clocks().1, ReportedBound::Figure(AtBound::Hold), Box::new([]));
    let oversized = reported(
        WaitKind::SignOff,
        world.clocks().1,
        ReportedBound::Figure(AtBound::Overdue),
        Box::new([Thing { what: 1, number: procedure, owner: None }; 5]),
    );
    for (connector, task, waits, reason) in [
        (1, agent, vec![room.clone()], "not a procedure"),
        (2, procedure, vec![room.clone()], "another connector owns the task"),
        (1, procedure, vec![room.clone(); 5], "more than four waits"),
        (1, procedure, vec![room.clone(), oversized], "more than four things"),
    ] {
        report(&mut world, connector, task, waits.into_boxed_slice());
        assert_eq!(world.domain.wait_refusal(connector), Some((task, reason)));
        assert_eq!(world.domain.core().waits(procedure), before);
        assert_eq!(world.domain.core().waits(agent), agent_before);
        assert_eq!(world.domain.core().wait_floor(WaitKind::Room), None);
    }
    crate::read_support::request(
        &mut world,
        1,
        jig_domain_people::Ask::Cancel {
            project: 1,
            task: procedure,
            reason: b"no longer needed".as_slice().into(),
            grace: None,
        },
    );
    assert!(world.domain.core().tasks.task(procedure).is_none());
    report(&mut world, 1, procedure, Box::new([room]));
    assert_eq!(world.domain.wait_refusal(1), Some((procedure, "not a live task")));
    assert_eq!(world.domain.core().waits(procedure).count, 0);
    assert_eq!(world.domain.core().wait_floor(WaitKind::Room), None);
}

#[test]
fn a_restart_waits_for_one_fresh_report_per_live_procedure_at_catch_up() {
    let seed = 13605;
    let action = ProcedureAction::ReportWait { state: 5 };
    let mut world = world(seed, action.clone());
    let first = procedure(&mut world, 1, None);
    let first_wait = step_wait(&world, first);
    world.advance(Duration::from_secs(1));
    let next = procedure(&mut world, 2, None);
    let next_wait = step_wait(&world, next);
    assert!(first_wait.since < next_wait.since);
    world.advance(Duration::from_secs(1));
    world.restart_with_startup(fixture(seed, action).0, root::Domain::script_wait_reports);
    for (task, previous) in [(first, first_wait), (next, next_wait)] {
        assert_eq!(world.domain.core().waits(task).first.expect("cold news").kind, WaitKind::News);
        let event = world.domain.take_wait_report().expect("one catch-up report for each live procedure");
        let core::Event::Waits { connector: 1, task: named, ref waits } = event else { panic!("connector report") };
        assert_eq!(named, task);
        assert_eq!(waits.get(0).expect("reconstructed clock").since, previous.since);
        world.send(root::Event::Core(event));
        assert_eq!(step_wait(&world, task), previous);
    }
    assert!(world.domain.take_wait_report().is_none());
}

#[test]
fn the_first_accepted_step_report_sets_the_kind_floor() {
    let mut world = world(13606, ProcedureAction::Stall);
    let task = procedure(&mut world, 1, None);
    let floor = world.clocks().1;
    let due = at(floor, STALL);
    assert_eq!(world.domain.core().wait_floor(WaitKind::Step), None);
    let clock = ReportedBound::At { at: due, then: AtBound::Hold };
    let first_since = Wall::from_nanos(100);
    report(&mut world, 1, task, Box::new([reported(WaitKind::Step, first_since, clock, Box::new([]))]));
    assert_eq!(world.domain.core().wait_floor(WaitKind::Step), Some(floor));
    world.advance(Duration::from_secs(1));
    let things = Box::new([Thing { what: 1, number: task, owner: None }]);
    report(&mut world, 1, task, Box::new([reported(WaitKind::Step, Wall::EPOCH, clock, things)]));
    let wait = step_wait(&world, task);
    assert_eq!(wait.since, floor);
    assert_eq!(wait.bound, Bound::At { at: due, then: AtBound::Hold });
}
