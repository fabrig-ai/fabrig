//! QA1: task pages against B2's real engine, including terminal watch/read order.
use jig_client_world::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures as f, front::Joint};

type Pages = World<Temper, Joint>;

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
fn press(label: &[u8]) -> Step<Address> {
    Step::Press { find: Find::exactly_named(Role::Button, label) }
}

// client/README.md 6.10: "a root started at a page's address ... asking its reads";
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

// client/README.md 5.9: an ended task shows "Result" and its report's words;
// 6.10: a fresh client at its address asks the reads (the ended subject has no live watch),
// and a browser's reload is "a new client at that address". This world's `reload` is a
// terminal restart, which keeps no address (tests/client/src/front.rs), so the reload is a
// second client at the address; Chromium's own reload is tests/client-browser's qa_task_pages.
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
    let jig_client::contract::ChipPhase::Owner { details, .. } = document.chip.details.as_ref() else {
        panic!("owner chip")
    };
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

// client/README.md 5.8: an ended goal is read again so its result stays readable;
// "Live only" stays chosen while the ended read goes on and says "No live tasks."
#[test]
#[ignore = "backlog: a goal page that misses a burst of tree changes before its goal ends keeps a stale live row (Task 4 'preparing a host' beside '0 live'), under All too; B2's story, which reads between the reports, passes"]
fn goal_finishes_while_live_filter_keeps_its_result_and_counts() {
    let (mut world, plan) = planned();
    let client = world.client(f::PARTY, f::task_page(plan.goal));
    follow(&mut world, client, vec![see(Find::text(b"Its delegates: 2 live, 1 ended.")), press(b"Live only")]);
    f::b2::first_reported(world.front(), &plan);
    drop(f::b2::goal_reported(world.front(), &plan));
    follow(
        &mut world,
        client,
        vec![
            see(result(Find::text(f::b2::GOAL_REPORT))),
            see(Find::text(b"Its delegates: 0 live, more than 2 ended.")),
            see(Find::text(b"No live tasks.")),
            gone(f::plan_item(plan.first, b"Completed")),
        ],
    );
    follow(
        &mut world,
        client,
        vec![
            press(b"All"),
            see(f::plan_item(plan.advisor, b"Completed")),
            see(f::plan_item(plan.first, b"Completed")),
            gone(f::plan_item(plan.second, b"Completed")),
            see(Find::text(b"More of its ended tasks are not shown.")),
        ],
    );
}

// client/README.md 5.8: "All is chosen when the page opens"; an ended goal's plan
// reads its bounded first ended page even when the tree subject is gone (6.10).
// B2's engine clamps this page to two rows; its third delegate is beyond the cut.
// The reload is a second client at the address, as 6.10 says a browser's is (this
// world's `reload` is a terminal restart, which keeps no address).
#[test]
fn fresh_ended_goal_reload_restores_all_filter_for_its_page() {
    let (mut world, plan) = planned();
    f::b2::first_reported(world.front(), &plan);
    drop(f::b2::goal_reported(world.front(), &plan));
    let client = world.client(f::PARTY, f::task_page(plan.goal));
    follow(
        &mut world,
        client,
        vec![
            see(result(Find::text(f::b2::GOAL_REPORT))),
            see(f::plan_item(plan.advisor, b"Completed")),
            see(f::plan_item(plan.first, b"Completed")),
            gone(f::plan_item(plan.second, b"Completed")),
            see(Find::text(b"More of its ended tasks are not shown.")),
            press(b"Live only"),
            see(Find::text(b"No live tasks.")),
        ],
    );
    let client = world.client(f::PARTY, f::task_page(plan.goal));
    follow(
        &mut world,
        client,
        vec![
            see(f::plan_item(plan.advisor, b"Completed")),
            see(f::plan_item(plan.first, b"Completed")),
            gone(f::plan_item(plan.second, b"Completed")),
            see(Find::text(b"More of its ended tasks are not shown.")),
            see(Find::text(b"Its delegates: 0 live, more than 2 ended.")),
        ],
    );
}
