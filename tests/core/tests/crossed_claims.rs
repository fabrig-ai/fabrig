//! engine.md, 15: a fresh mark stops a live resumed claim before retrying.
#[path = "support/fresh_windows.rs"]
mod fresh_windows;

use fresh_windows::{AFTER, SEED, TAIL};
use jig_core_world::{effects::World, fixtures};
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, Token};

// The world's fleet grace (world::limits); check its two sides one nanosecond apart.
const GRACE: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_nanos(1);
const BEFORE_GRACE: Duration = Duration::from_nanos(4_999_999_999);
const WAKE: u8 = 87;

fn live_resumed() -> (World, u64, u64) {
    let mut world = fresh_windows::parked();
    let read = world.say(WAKE);
    // A real committed turn in the resumed attempt, with no additional said
    // text: the closed window's rendered tail stays the named TAIL fixture.
    world.turn(1, 1, Some(read), &fresh_windows::turn(b""));
    // Hold time through restart and peer traffic so the grace's two sides
    // are exact; the ordinary scripted drain advances a millisecond per step.
    let (now, wall) = world.clocks();
    world.set_clock(now, wall);
    let assignment = world.assigned.as_ref().expect("live resumed assignment");
    let (task, attempt) = (assignment.task, assignment.attempt);
    (world, task, attempt)
}

fn hosting(world: &World) -> fleet::Hosted {
    let assignment = world.assigned.as_ref().expect("live resumed assignment");
    fleet::Hosted {
        run: Token::new(assignment.task),
        attempt: Token::new(assignment.attempt),
        phase: fleet::Phase::Active,
    }
}

fn lost_once_then_fresh(world: &mut World, task: u64, attempt: u64) {
    assert_eq!(
        fresh_windows::tries(world),
        tasks::Tries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 1, invalid: 0 }
    );
    assert_eq!(fixtures::live_row(world, task).refusals, 0);
    world.retry_due();
    fresh_windows::fresh(world);
    assert_eq!(fresh_windows::tail(world), Some(TAIL));
    assert!(fresh_windows::proof(world).attempt > attempt);
}

#[test]
fn a_crossed_claim_listed_at_hello_is_stopped_and_lost() {
    let (mut world, task, attempt) = live_resumed();
    world.restart_with_patch_sets(SEED, false, AFTER);
    let proof = fresh_windows::proof(&world);
    assert!(proof.crossed);
    assert_eq!(proof.window, Some(core::CurrentWindow { attempt: 2, turn: 1, closed: true }));
    let hosted = hosting(&world);
    fresh_windows::hello_hosting(&mut world, Box::new([hosted]));
    world.grant_account(1);
    assert_eq!(world.cancellations, [(task, attempt)]);
    assert_eq!(fresh_windows::tries(&world).lost, 0, "cancelling waits for the worker's answer");
    assert_eq!(world.assignments.last(), Some(&(task, attempt)), "the next run still waits");

    let last_turn = fixtures::live_row(&world, task).turn;
    world.turn(2, 2, None, &fresh_windows::turn(b"late crossed turn"));
    let late = root::Key::Core(core::Key::Core(core::CoreKey::Turn { task, attempt, turn: 2 }));
    assert!(!world.store.rows.contains_key(&late));
    assert_eq!(fixtures::live_row(&world, task).turn, last_turn);
    // The chat fixture takes no resources, so there is no writer slot to
    // exercise; keep that explicit instead of claiming writer coverage.
    assert!(fixtures::live_row(&world, task).holdings.is_empty());
    world.end(tasks::End::Failed(tasks::Class::Agent), 1);
    lost_once_then_fresh(&mut world, task, attempt);
}

#[test]
fn a_crossed_claim_no_worker_lists_is_lost_once_the_grace_passes() {
    let (mut world, task, attempt) = live_resumed();
    fresh_windows::restart(&mut world, AFTER);
    assert!(fresh_windows::proof(&world).crossed);
    assert_eq!(jig_core_world::world::limits().core.fleet.grace, GRACE);
    world.advance(BEFORE_GRACE);
    assert_eq!(fresh_windows::tries(&world).lost, 0);
    assert_eq!(world.assignments.last(), Some(&(task, attempt)));
    world.advance(TICK);
    lost_once_then_fresh(&mut world, task, attempt);
}
