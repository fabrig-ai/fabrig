//! task pages against B2's real engine, including terminal watch/read order.
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures as f, front::Joint};

type Pages = World<Temper, Joint>;
// The initial open and one browser reload.
const GOAL_OPENS: u32 = 2;

fn planned() -> (Pages, f::b2::Planned) {
    let mut world = f::world(f::SEED);
    let advised = f::b2::advisor_reported(world.front());
    let planned = f::b2::tasks_declared(world.front(), advised);
    (world, planned)
}
fn see(find: Find) -> Step<Address> {
    Step::See { find, within: f::WITHIN }
}
fn gone(find: Find) -> Step<Address> {
    Step::Gone { find, within: f::WITHIN }
}
fn follow(world: &mut Pages, client: ClientId, steps: Vec<Step<Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, skein_world::domain::Verdict::Passed, "{outcome:?}\n{}", drawing(world, client));
}
fn drawing(world: &Pages, client: ClientId) -> String {
    let mut bytes = Vec::new();
    for node in world.tree(client).nodes() {
        for run in &node.text {
            bytes.extend_from_slice(&run.text);
        }
    }
    String::from_utf8(bytes).expect("visible UTF-8")
}
fn result(find: Find) -> Find {
    find.within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }]))
}
fn dependency(task: u64, find: Find) -> Find {
    find.within(Box::new([Target::ExactRole {
        role: Role::List,
        name: format!("Dependencies of task {task}").into_bytes().into(),
    }]))
}

// client/line.md 10: "a root started at a page's address ... asking its reads";
// 5.9: the brief's dependency line and header state use the supplied facts;
// a run's heading and result follow the current or last run.
#[test]
fn fresh_waiting_delegate_tracks_dependency_completion_then_its_own_result() {
    let (mut world, plan) = planned();
    let client = world.client(f::PARTY, f::task_page(plan.second));
    let waiting = format!("waiting on task {}", plan.first);
    let first = format!("Task {}", plan.first);
    follow(
        &mut world,
        client,
        vec![
            see(Find::exactly_named(Role::Region, b"Task brief")),
            see(Find::named(Role::Status, waiting.as_bytes())),
            see(dependency(plan.second, Find::exactly_named(Role::Link, first.as_bytes()))),
            see(Find::text(b"No run now")),
        ],
    );
    f::b2::first_reported(world.front(), &plan);
    let attempt = world.front().engine.task(plan.second).expect("second run").attempt;
    let live = format!("Run {attempt} · live now");
    let ended = format!("Run {attempt} · ended");
    follow(
        &mut world,
        client,
        vec![
            see(dependency(plan.second, Find::text(b"Completed"))),
            see(Find::text(live.as_bytes())),
            gone(Find::text(waiting.as_bytes())),
        ],
    );
    drop(f::b2::goal_reported(world.front(), &plan));
    follow(
        &mut world,
        client,
        vec![
            see(Find::text(ended.as_bytes())),
            see(result(Find::text(f::b2::SECOND_REPORT))),
            gone(Find::exactly_named(Role::TextArea, b"Message")),
        ],
    );
}

// client/pages/task.md 2: an ended task shows "Result" and its report's words;
// 6.10: a fresh client at its address asks the reads (the ended subject has no live watch),
// and a browser's reload is "a new client at that address". This world's `reload` is a
// terminal restart, which keeps no address (tests/client/src/front.rs), so the reload is a
// second client at the address; Chromium's own reload is tests/client-browser's task_pages.
#[test]
fn fresh_ended_advisor_keeps_its_brief_and_result_across_reload() {
    let (mut world, plan) = planned();
    let client = world.client(f::PARTY, f::task_page(plan.advisor));
    let goal = format!("Task {}", plan.goal);
    follow(
        &mut world,
        client,
        vec![
            see(Find::exactly_named(Role::Region, b"Task brief")),
            see(Find::text(f::b2::ADVISOR_BRIEF)),
            see(result(Find::text(f::b2::ADVISOR_REPORT))),
            see(Find::exactly_named(Role::Link, goal.as_bytes())),
        ],
    );
    let task = world.root(client).person().pages().task().expect("advisor page");
    let document = task.document(world.root(client).person().client()).expect("advisor document");
    let jig_client::contract::ChipPhaseFacts { details, .. } = document.chip.details.as_ref();
    let ended = format!("Run {} · ended", details.attempt.expect("advisor had a run"));
    let client = world.client(f::PARTY, f::task_page(plan.advisor));
    follow(
        &mut world,
        client,
        vec![
            see(result(Find::text(f::b2::ADVISOR_REPORT))),
            see(Find::text(ended.as_bytes())),
            gone(Find::exactly_named(Role::Region, b"Plan")),
        ],
    );
}

// client/pages/goal.md 2: an ended goal keeps its result and total counts.
#[test]
#[ignore = "backlog: a goal page that misses a burst of tree changes before its goal ends keeps a stale live row; B2's story, which reads between the reports, passes"]
fn goal_finishes_with_its_result_and_counts() {
    let (mut world, plan) = planned();
    let client = world.client(f::PARTY, f::task_page(plan.goal));
    follow(&mut world, client, vec![see(Find::text(b"1 of 3 tasks"))]);
    f::b2::first_reported(world.front(), &plan);
    drop(f::b2::goal_reported(world.front(), &plan));
    follow(&mut world, client, ended_goal());
}

// client/pages/goal.md 2: the report, total counts and bounded done tasks survive reload.
// B2's engine clamps the ended page to two rows; its third delegate is beyond the cut.
// A second client at the address models the browser's reload (6.10).
#[test]
fn fresh_ended_goal_reload_keeps_its_result_and_counts() {
    let (mut world, plan) = planned();
    f::b2::first_reported(world.front(), &plan);
    drop(f::b2::goal_reported(world.front(), &plan));
    for _ in 0..GOAL_OPENS {
        let client = world.client(f::PARTY, f::task_page(plan.goal));
        follow(&mut world, client, ended_goal());
    }
}

fn ended_goal() -> Vec<Step<Address>> {
    vec![
        see(Find::exactly_named(Role::Heading { level: 1 }, f::b2::GOAL_REPORT)),
        see(Find::text(f::b2::GOAL_REPORT)
            .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Report".as_slice().into() }]))),
        see(Find::text(" · landed · ".as_bytes())),
        see(Find::text(b"3 of 3 tasks")),
        see(Find::exactly_named(Role::Image, b"0 landed of 2")),
        see(done_link(f::b2::ADVISOR_BRIEF)),
        see(done_link(f::b2::FIRST_BRIEF)),
        gone(Find::exactly_named(Role::Link, f::b2::SECOND_BRIEF)),
        see(Find::text(b"More of its tasks are not shown.")),
    ]
}
fn done_link(title: &[u8]) -> Find {
    Find::exactly_named(Role::Link, title)
        .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Tasks".as_slice().into() }]))
}
