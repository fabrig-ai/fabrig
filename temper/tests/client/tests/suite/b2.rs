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
    world.person(person, see_the_goals_report_and_settled_budget(&report, budget));
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
    let link = goal_link(b2::ADVISOR_BRIEF);
    let mut steps = vec![
        see(Find::exactly_named(Role::Disclosure, b"1 landed")),
        see(Find::exactly_named(Role::Image, b"1 landed of 1")),
        see(landed(b"1 landed", b2::ADVISOR_BRIEF)),
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
        see(goal_line(b2::SECOND_BRIEF, Find::text(b"waits on Implement the export"))),
        see(Find::exactly_named(Role::Disclosure, b"1 landed")),
        see(Find::text(b"1 of 3 tasks")),
        Step::Press { find: goal_link(b2::SECOND_BRIEF) },
        see(task_heading(planned.second, b2::SECOND_BRIEF)),
        see(Find::exactly_named(Role::Status, format!("waiting on task {}", planned.first).as_bytes())),
        see(dependency(planned.second, Find::exactly_named(Role::Status, b"running"))),
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
        see(goal_link(b2::FIRST_BRIEF)),
        Step::Press { find: goal_link(b2::FIRST_BRIEF) },
        see(task_heading(planned.first, b2::FIRST_BRIEF)),
        see(Find::exactly_named(Role::Heading { level: 2 }, format!("Run {attempt} · live now").as_bytes())),
    ]);
    steps.extend(return_to_goal(planned.goal));
    steps
}

fn see_the_dependency_completed_and_second_running(planned: &b2::Planned) -> Vec<Step<Address>> {
    let mut steps = vec![
        see(goal_line(b2::SECOND_BRIEF, Find::text(b"running"))),
        see(Find::exactly_named(Role::Disclosure, b"2 landed")),
        see(Find::text(b"2 of 3 tasks")),
        see(landed(b"2 landed", b2::ADVISOR_BRIEF)),
        see(landed(b"2 landed", b2::FIRST_BRIEF)),
        Step::Press { find: goal_link(b2::SECOND_BRIEF) },
        see(dependency(planned.second, Find::exactly_named(Role::Status, b"Completed"))),
        see(Find::exactly_named(Role::Status, b"running")),
    ];
    steps.extend(return_to_goal(planned.goal));
    steps
}

fn see_the_goals_report_and_settled_budget(
    report: &[u8],
    budget: jig_domain_core::documents::BudgetLine,
) -> Vec<Step<Address>> {
    vec![
        see(Find::exactly_named(Role::Heading { level: 1 }, report)),
        see(in_region(Find::text(report), b"Report")),
        see(Find::text(b"3 of 3 tasks")),
        // The world's engine answers the ended read two rows a page: the third is past it.
        see(Find::exactly_named(Role::Image, b"2 landed of 2")),
        see(in_region(Find::text(b"More of its tasks are not shown."), b"Tasks")),
        see(Find::exactly_named(Role::Disclosure, b"2 landed")),
        see(landed(b"2 landed", b2::ADVISOR_BRIEF)),
        see(landed(b"2 landed", b2::FIRST_BRIEF)),
        gone(goal_link(b2::SECOND_BRIEF)),
        see(Find::text(format!("{} of {} spent", budget.numbers.spent, budget.numbers.budget).as_bytes())),
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
fn goal_link(title: &[u8]) -> Find {
    in_region(Find::exactly_named(Role::Link, title), b"Tasks")
}
fn dependency(task: u64, find: Find) -> Find {
    find.within(Box::new([Target::ExactRole {
        role: Role::List,
        name: format!("Dependencies of task {task}").into_bytes().into(),
    }]))
}
fn task_heading(task: u64, brief: &[u8]) -> Find {
    Find::exactly_named(
        Role::Heading { level: 1 },
        format!("T{task} · {}", std::str::from_utf8(brief).expect("fixture's brief")).as_bytes(),
    )
}
fn return_to_goal(goal: u64) -> Vec<Step<Address>> {
    vec![Step::Press { find: task_link(goal) }, see(Find::exactly_named(Role::Heading { level: 2 }, b"Tasks"))]
}

fn goal_line(title: &[u8], find: Find) -> Find {
    find.within(Box::new([
        Target::ExactRole { role: Role::Region, name: b"Tasks".as_slice().into() },
        Target::Role { role: Role::Paragraph, name: title.into() },
    ]))
}
fn landed(fold: &[u8], title: &[u8]) -> Find {
    // The joint's scripted reports fall within the epoch's first minute.
    Find::text(b"00:00").within(Box::new([
        Target::ExactRole { role: Role::Disclosure, name: fold.into() },
        Target::Role { role: Role::Paragraph, name: title.into() },
    ]))
}
