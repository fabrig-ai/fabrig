use jig_client_world::harness::Until;
use skein_fake_person::{Find, Step, Target};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::Address;
use temper_client_world::fixtures::{self, WITHIN, b2};

#[test]
fn roadmap_b2_through_the_task_pages() {
    let mut world = fixtures::world(fixtures::SEED);
    let advised = b2::advisor_reported(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(advised.goal));
    world.person(person, open_advisors_result_and_return(&advised));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    let planned = b2::tasks_declared(world.front(), advised);
    let attempt = world.front().engine.task(planned.first).expect("first delegate").attempt;
    world.person(person, follow_the_plan_and_each_delegates_lineage(&planned, attempt));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    b2::first_reported(world.front(), &planned);
    world.person(person, see_the_dependency_completed_and_second_running(&planned));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    let report = b2::goal_reported(world.front(), &planned);
    let budget = fixtures::task_budget(world.front(), planned.goal);
    assert!(budget.numbers.spent_below > 0, "the engine settled the delegates' spend");
    world.person(person, see_the_goals_report_and_settled_budget(&planned, &report, budget));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    // A fresh client proves the root's part of opening a delegate by its address.
    let fresh = world.client(fixtures::PARTY, fixtures::task_page(planned.second));
    world.person(
        fresh,
        vec![
            see(task_heading(planned.second, b2::SECOND_BRIEF)),
            see(Find::exactly_named(Role::Region, b"Task brief")),
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn open_advisors_result_and_return(advised: &b2::Advised) -> Vec<Step<Address>> {
    let link = in_region(task_link(advised.advisor), b"Latest activity");
    let mut steps = vec![
        see(link.clone()),
        see(fixtures::plan_item(advised.advisor, b"Completed")),
        see(in_region(Find::text(b"Its delegates: 0 live, 1 ended."), b"Plan")),
        Step::Press { find: link },
        see(task_heading(advised.advisor, b2::ADVISOR_BRIEF)),
        see(in_region(Find::exactly_named(Role::Status, b"Completed"), b"Result")),
        see(in_region(Find::text(b2::ADVISOR_REPORT), b"Result")),
    ];
    steps.extend(return_to_goal(advised.goal));
    steps
}

fn follow_the_plan_and_each_delegates_lineage(planned: &b2::Planned, attempt: u64) -> Vec<Step<Address>> {
    let mut steps = vec![
        see(fixtures::plan_item(planned.second, format!("waiting on task {}", planned.first).as_bytes())),
        see(fixtures::dependency_item(planned.second, task_link(planned.first))),
        see(fixtures::dependency_item(planned.second, Find::exactly_named(Role::Status, b"running"))),
        see(fixtures::plan_item(planned.advisor, b"Completed")),
        see(in_region(Find::text(b"Its delegates: 2 live, 1 ended."), b"Plan")),
        Step::Press { find: plan_link(planned.second) },
        see(task_heading(planned.second, b2::SECOND_BRIEF)),
        see(Find::exactly_named(Role::Status, format!("waiting on task {}", planned.first).as_bytes())),
        see(task_link(planned.first).within(Box::new([
            Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() },
            Target::ExactRole {
                role: Role::List,
                name: format!("Dependencies of task {}", planned.second).into_bytes().into(),
            },
        ]))),
    ];
    steps.extend(return_to_goal(planned.goal));
    steps.extend([
        see(plan_link(planned.first)),
        Step::Press { find: plan_link(planned.first) },
        see(task_heading(planned.first, b2::FIRST_BRIEF)),
        see(Find::exactly_named(Role::Heading { level: 2 }, format!("Run {attempt} · live now").as_bytes())),
    ]);
    steps.extend(return_to_goal(planned.goal));
    steps
}

fn see_the_dependency_completed_and_second_running(planned: &b2::Planned) -> Vec<Step<Address>> {
    vec![
        see(fixtures::dependency_item(planned.second, Find::exactly_named(Role::Status, b"Completed"))),
        see(fixtures::plan_item(planned.second, b"running")),
        see(fixtures::plan_item(planned.first, b"Completed")),
        see(in_region(Find::text(b"Its delegates: 1 live, 2 ended."), b"Plan")),
    ]
}

fn see_the_goals_report_and_settled_budget(
    planned: &b2::Planned,
    report: &[u8],
    budget: jig_domain_core::documents::BudgetLine,
) -> Vec<Step<Address>> {
    vec![
        see(in_region(Find::exactly_named(Role::Status, b"Completed"), b"Result")),
        see(in_region(Find::text(report), b"Result")),
        // The world's engine answers the ended read two rows a page: the third is past it.
        see(in_region(Find::text(b"Its delegates: 0 live, more than 2 ended."), b"Plan")),
        see(in_region(Find::text(b"More of its ended tasks are not shown."), b"Plan")),
        see(fixtures::plan_item(planned.advisor, b"Completed")),
        see(fixtures::plan_item(planned.first, b"Completed")),
        gone(Find::exactly_named(Role::TreeItem, format!("Task {}", planned.second).as_bytes())),
        Step::Press { find: in_region(Find::exactly_named(Role::Button, b"Live only"), b"Plan") },
        see(in_region(Find::text(b"No live tasks."), b"Plan")),
        gone(Find::exactly_named(Role::TreeItem, format!("Task {}", planned.first).as_bytes())),
        see(in_region(
            Find::text(
                format!("{} spent · {} left of {}", budget.numbers.spent, budget.left, budget.numbers.budget)
                    .as_bytes(),
            ),
            b"Budget and authority",
        )),
        see(in_region(Find::exactly_named(Role::Term, b"Delegates spent"), b"Budget and authority")),
        see(in_region(
            Find::exactly_named(Role::Description, budget.numbers.spent_below.to_string().as_bytes()),
            b"Budget and authority",
        )),
    ]
}

fn see(find: Find) -> Step<Address> {
    Step::See { find, within: WITHIN }
}
fn gone(find: Find) -> Step<Address> {
    Step::Gone { find, within: WITHIN }
}
fn in_region(find: Find, name: &[u8]) -> Find {
    find.within(Box::new([Target::ExactRole { role: Role::Region, name: name.into() }]))
}
fn task_link(task: u64) -> Find {
    Find::exactly_named(Role::Link, format!("Task {task}").as_bytes())
}
fn plan_link(task: u64) -> Find {
    task_link(task).within(Box::new([Target::ExactRole {
        role: Role::TreeItem,
        name: format!("Task {task}").into_bytes().into(),
    }]))
}
fn task_heading(task: u64, brief: &[u8]) -> Find {
    Find::exactly_named(
        Role::Heading { level: 1 },
        format!("T{task} · {}", std::str::from_utf8(brief).expect("fixture's brief")).as_bytes(),
    )
}
fn return_to_goal(goal: u64) -> Vec<Step<Address>> {
    vec![
        Step::Press { find: task_link(goal) },
        see(Find::exactly_named(Role::Heading { level: 1 }, fixtures::GOAL_BRIEF)),
    ]
}
