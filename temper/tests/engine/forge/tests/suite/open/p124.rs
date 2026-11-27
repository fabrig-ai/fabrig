//! Room behind live and ended writers (engine-checks.md, 5.1).
use skein_lib::{Duration, Wall};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{LIMITS, World, changes};

const WRITER: u64 = 51;
const WAITER: u64 = 52;
const ENTRY: u64 = 100;

fn after(at: Wall, duration: Duration) -> Wall {
    Wall::from_nanos(at.as_nanos() + duration.as_nanos())
}

fn waiting_for_entry() -> World {
    let mut world = World::with_entries(124, 1);
    world.adopt();
    for task in [WRITER, WAITER] {
        let branch = format!("temper/{task}");
        world.produce(branch.as_bytes());
        let mut row = changes::change_row();
        row.task = task;
        row.change.task = task;
        row.branch = branch.into_bytes().into();
        row.change.state = change::State::Opening { requested: false };
        row.change.since = world.wall();
        world.event(top::Event::Change { row });
    }
    world.lose_writes();
    open(&mut world, WRITER, ENTRY);
    assert!(
        world.seen().iter().any(|request| matches!(
            request,
            top::Request::Outcome { task: WRITER, entry: ENTRY, outcome: skein_forge_client::Outcome::Uncertain, .. }
        )),
        "the writer's opening was lost"
    );
    open(&mut world, WAITER, ENTRY + 1);
    assert_eq!(world.change(WAITER).room, Some(1));
    assert_uncertain(&world);
    world
}

fn open(world: &mut World, task: u64, entry: u64) -> change::Decision {
    changes::change_step_task(world, task, entry, change::Status::Passed, Box::new([]), false, false)
}

fn assert_uncertain(world: &World) {
    assert!(
        matches!(world.stored().get(&top::Key::Entry(ENTRY)),
            Some(top::Stored::Entry { entry, held: false }) if entry.task == WRITER && entry.attempt.is_some()
        ),
        "the only write still awaits reconciliation"
    );
}

#[test]
fn a_change_waiting_for_room_has_a_bound() {
    let mut world = waiting_for_entry();
    let since = world.wall();
    world.unavailable(since, after(since, LIMITS.waits.room.saturating_add(Duration::from_secs(300))));
    // The release first closes the writer's effects. The uncertain write
    // keeps settling on its own; Release cannot follow EffectsSettled yet.
    world.event(top::Event::SettleEffects { task: WRITER, root: WRITER, ending: top::ReleaseEnding::Cancelled });
    let ended = world.wall();
    assert!(world.stored().contains_key(&top::Key::Release(WRITER)));
    assert!(
        !world.seen().iter().any(|request| matches!(
            request,
            top::Request::EffectsSettled { task: WRITER } | top::Request::Released { task: WRITER }
        )),
        "cancellation closes the writer while its attempted write remains uncertain"
    );
    let due = after(ended, LIMITS.waits.room);
    world.run_to(Wall::from_nanos(due.as_nanos() - 1));
    assert!(!matches!(world.change(WAITER).change.state, change::State::Held { .. }));
    world.run_to(after(due, Duration::from_secs(1)));
    assert!(matches!(
        world.change(WAITER).change.state,
        change::State::Held { hold: change::Hold { cause: change::Cause::Stalled, .. }, .. }
    ));
    assert_eq!(world.change(WAITER).room, Some(1), "held in the same place");
    let words = &world.change(WAITER).effect_account.as_ref().expect("room hold account").words;
    assert_eq!(words.as_ref(), format!("no progress at opening for {} minutes: waiting for room in the forge's outbox, held by entry {ENTRY} of ended task {WRITER}", LIMITS.waits.room.as_nanos() / Duration::from_secs(60).as_nanos()).as_bytes());
    assert_uncertain(&world);
}

#[test]
fn a_live_writer_stalls_before_its_room_waiter() {
    let mut world = waiting_for_entry();
    let stall = after(world.change(WRITER).change.since, LIMITS.change_policy.stall);
    world.unavailable(world.wall(), stall);
    world.run_to(stall);
    assert_eq!(open(&mut world, WRITER, ENTRY + 2), change::Decision::Hold(change::Cause::Stalled));
    assert_uncertain(&world);
    assert!(!matches!(world.change(WAITER).change.state, change::State::Held { .. }));
    let inspection = world.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.waiter == "task 52").expect("room waiter");
    assert_eq!(wait.kind, "change: room");
    assert_eq!(wait.bound, None);
    assert_eq!(wait.wakes, [skein_world::inspect::Wake::Waiter("task 51".into())]);
}
