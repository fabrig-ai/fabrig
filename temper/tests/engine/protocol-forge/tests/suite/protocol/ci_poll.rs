//! GitHub CI polling with the shipped paces, without delivery hints.
use super::*;
use skein_lib::Duration;
use temper_engine_service::limits::{Profile, profile};

/// Open a change whose first CI verdict arrives ninety seconds after production.
pub(super) fn pending_github_change(passes: u32) -> (Protocol, Wall, Wall, u64) {
    let shipped = profile(Profile::Small).forge;
    assert_eq!(shipped.change_policy.stall, Duration::from_secs(30 * 60));
    let mut limits = temper_engine_forge_world::LIMITS;
    limits.client = shipped.client;
    limits.change_policy.stall = shipped.change_policy.stall;
    limits.resources_per_task = shipped.resources_per_task;
    let mut checks = skein_forge_kit::joint::checks(passes);
    checks.latency_min = Duration::from_secs(90);
    checks.latency_max = Duration::from_secs(90);
    let mut world = Protocol::github_with_limits(71, checks, &limits);
    world.adopt();
    let run = branch_name(b"temper/1/r1");
    world.event(top::Event::Names { task: 1, resources: Box::new([run.clone()]) });
    world.event(top::Event::Hold { entry: 0, task: 1, resource: run, from: None });
    let mut row = change_row();
    row.change.since = world.wall();
    world.event(top::Event::Change { row });
    let branch = branch_name(b"temper/51");
    world.event(top::Event::Names { task: 51, resources: Box::new([branch.clone(), branch_name(b"main")]) });
    world.event(top::Event::Hold { entry: 0, task: 51, resource: branch, from: None });
    world.run_for(0);
    assert_eq!(
        change_step(&mut world, 2, change::Status::Unknown, Box::new([])),
        change::Decision::Delegate(change::Delegate::Produce)
    );
    let pushed = world.wall();
    world.produce(b"temper/51");
    let head = world.branch(b"temper/51");
    let verdict = Wall::from_nanos(pushed.as_nanos() + Duration::from_secs(90).as_nanos());
    assert_eq!(
        change_step(&mut world, 3, change::Status::Passed, Box::new([])),
        change::Decision::Effect(change::Effect::Open)
    );
    (world, pushed, verdict, head)
}

/// Register a discovered pull once, as the root does when saving its number.
pub(super) fn name_pull(world: &mut Protocol) -> bool {
    let Some(top::Stored::Change(row)) = world.stored().get(&top::Key::Change(51)) else { return false };
    let Some(number) = row.pull else { return false };
    world.event(top::Event::Rename {
        task: 51,
        resources: Box::new([
            branch_name(b"temper/51"),
            top::Name { forge: REPO.forge, repository: REPO.repository, what: top::What::Pull(number) },
        ]),
    });
    true
}

fn metadata(world: &Protocol) -> Option<skein_forge::api::Summary> {
    world.stored().iter().find_map(|(_, record)| match record {
        top::Stored::Client(skein_forge_client::Stored::Live(row))
            if matches!(row.watch.resource.what, skein_forge_client::What::Pull(_)) =>
        {
            row.cached.item.clone()
        }
        _ => None,
    })
}

struct Schedule {
    due: bool,
    until: Option<Wall>,
    repaired: bool,
    entry: u64,
}

fn hear(world: &mut Protocol, schedule: &mut Schedule) {
    for request in world.take_seen() {
        match request {
            top::Request::WakeProcedure { task: 51 } => schedule.due = true,
            top::Request::ChangeDecision { task: 51, decision, entry: committed, .. } => {
                if let Some(entry) = committed {
                    world.event(top::Event::Committed { entry });
                }
                match decision {
                    change::Decision::Wait { until: at } => schedule.until = at,
                    change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci)) => {
                        schedule.repaired = true;
                    }
                    change::Decision::Hold(why) => panic!("pending CI held: {why:?}"),
                    _ => schedule.due = true,
                }
            }
            _ => {}
        }
    }
}

fn advance(world: &mut Protocol, schedule: &mut Schedule) {
    if schedule.until.is_some_and(|at| world.wall() >= at) {
        schedule.due = true;
        schedule.until = None;
    }
    if schedule.due {
        schedule.due = false;
        step_change(world, schedule.entry, change::Status::Unknown, false);
        schedule.entry += 1;
        world.run_for(0);
    } else {
        world.run_for(1);
    }
}

#[test]
fn a_github_ninety_second_ci_failure_wakes_a_repair_at_the_same_head() {
    let (mut world, _, verdict, head) = pending_github_change(0);
    let deadline = Wall::from_nanos(
        verdict.as_nanos()
            + profile(Profile::Small).forge.client.poll_max.as_nanos()
            + Duration::from_secs(1).as_nanos(),
    );
    let mut schedule = Schedule { due: true, until: None, repaired: false, entry: 4 };
    let (mut named, mut before) = (false, None);
    while world.wall() <= deadline && !schedule.repaired {
        if !named {
            named = name_pull(&mut world);
        }
        if before.is_none() {
            before = metadata(&world);
        }
        hear(&mut world, &mut schedule);
        advance(&mut world, &mut schedule);
    }
    assert!(named && schedule.repaired, "failed CI starts a repair by verdict + poll_max + 1s");
    assert_eq!(world.branch(b"temper/51"), head);
    assert!(before.is_some(), "the keep read the pull's metadata");
    assert_eq!(metadata(&world), before, "CI does not move PR metadata");
    assert_eq!(world.delivery_hints(), 0);
    assert_eq!(world.tally().routes.get(&Route::Merge), None);
}
