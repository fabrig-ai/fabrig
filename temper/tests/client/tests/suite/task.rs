use jig_client_kit::{
    harness::{App, Until},
    observe,
};
use skein_fake_person::{Face, Find, Found, Step, Target, TreeFace};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_world::{app::Temper, fixtures, front::Joint};

fn running_delegate_page() -> (jig_client_kit::harness::World<Temper, Joint>, jig_client_kit::harness::ClientId, u64) {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { first, .. } = fixtures::goal_with_dependent_delegates(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(first));
    (world, person, first)
}

#[test]
fn a_message_from_a_delegates_page_reaches_its_run() {
    let (mut world, person, task) = running_delegate_page();
    let words = b"Look closely at the retry path";
    world.person(
        person,
        vec![
            Step::See {
                find: Find::text("Steer this task · words reach its live run at the next turn.".as_bytes()),
                within: fixtures::WITHIN,
            },
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Message"), words: words.as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Send") },
            Step::See { find: conversation(Find::text(words)), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    let row = world.front().engine.task(task).expect("messaged delegate");
    assert!(row.inbox.iter().any(|message| {
        message.from == jig_domain_tasks::Party::Person(fixtures::PARTY) && message.words.as_ref() == words
    }));
    let next_turn = row.turn + 1;
    let number = fixtures::read_messages(world.front(), task);
    let row = world.front().engine.task(task).expect("delegate after reading");
    assert_eq!(row.turn, next_turn);
    assert_eq!(row.last_message, number);
    assert!(row.inbox.iter().all(|message| message.number > number), "the committed turn took the message");
    world.person(
        person,
        vec![
            Step::See { find: conversation(Find::text(b"I will follow that direction")), within: fixtures::WITHIN },
            Step::See { find: conversation(Find::text(words)), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(b"Being sent"), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(b"Sent"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn conversation(find: Find) -> Find {
    find.within(Box::new([Target::ExactRole { role: Role::Log, name: b"Conversation".as_slice().into() }]))
}

#[test]
fn a_delegate_paused_from_its_page_is_held_then_resumed() {
    use jig_domain_tasks as tasks;
    let (mut world, person, task) = running_delegate_page();
    let attempt = world.front().engine.task(task).expect("running delegate").attempt;
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"Pause"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Pause") },
            Step::See { find: Find::text(b"Paused"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Button, b"Resume"), within: fixtures::WITHIN },
            Step::Gone { find: Find::named(Role::Button, b"Release task"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(world.root(person).person().pages().confirmation().is_none());
    assert!(matches!(world.front().engine.task(task).expect("paused delegate").phase, tasks::Phase::Held { .. }));
    world.person(
        person,
        vec![
            Step::Press { find: Find::named(Role::Button, b"Resume") },
            Step::Gone { find: Find::named(Role::Button, b"Resume"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(world.root(person).person().pages().confirmation().is_none());
    let row = world.front().engine.task(task).expect("resumed delegate");
    assert!(matches!(row.phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    assert!(row.attempt > attempt, "resume assigned a new run");
    assert_eq!(row.tries, tasks::Tries::NONE);
}

#[test]
fn a_delegate_closed_from_its_page_ends() {
    use jig_domain_tasks as tasks;
    let (mut world, person, task) = running_delegate_page();
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"Close"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Close") },
            Step::See { find: Find::named(Role::Dialog, b"Close this task?"), within: fixtures::WITHIN },
            Step::See {
                find: Find::text("You can still read it, but it can’t be reopened.".as_bytes()),
                within: fixtures::WITHIN,
            },
            Step::See { find: Find::named(Role::Button, b"Keep it"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(matches!(world.front().engine.task(task).expect("live delegate").phase, tasks::Phase::Active(_)));
    world.person(
        person,
        vec![
            Step::Press {
                find: Find::named(Role::Button, b"Close task")
                    .within(Box::new([Target::Role { role: Role::Dialog, name: Box::new([]) }])),
            },
            Step::Gone { find: Find::named(Role::Dialog, b"Close this task?"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Cancelled"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
    let Some(temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        world.front().engine.store.rows.get(&key)
    else {
        panic!("closed delegate has an ended record");
    };
    assert!(matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
}

#[test]
fn a_goal_cancelled_from_its_page_ends_its_delegates() {
    use jig_domain_tasks as tasks;
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_dependent_delegates(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![Step::See { find: Find::named(Role::Heading { level: 2 }, b"Tasks"), within: fixtures::WITHIN }],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    // Close asks for confirmation before cancelling the goal and its delegates.
    world.event(
        person,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        }
        .into(),
    );
    assert!(world.root(person).person().pages().confirmation().is_some(), "Close awaits confirmation");
    world.event(
        person,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Confirm),
        }
        .into(),
    );
    world.person(person, vec![Step::See { find: Find::text(b"Closed before it landed."), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    for task in [first, second] {
        let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)));
        let Some(temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            world.front().engine.store.rows.get(&key)
        else {
            panic!("delegate {task} has an ended record");
        };
        assert!(matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })), "delegate {task} cancelled");
    }
}

#[test]
fn a_waiting_delegates_message_is_refused_with_its_reason() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { second, .. } = fixtures::goal_with_dependent_delegates(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(second));
    world.person(
        person,
        vec![
            Step::See { find: Find::text(b"Message"), within: fixtures::WITHIN },
            Step::See { find: Find::text(b"Messages reach it once it runs."), within: fixtures::WITHIN },
            Step::Gone { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN },
            Step::Gone { find: Find::named(Role::Button, b"Send"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(
        world
            .front()
            .engine
            .task(second)
            .expect("waiting delegate")
            .inbox
            .iter()
            .all(|message| { message.from != jig_domain_tasks::Party::Person(fixtures::PARTY) }),
        "the waiting delegate received no person message"
    );
}

#[test]
fn an_agents_older_turns_are_read_at_the_log_head() {
    let (mut world, person, task) = running_delegate_page();
    fixtures::extend_delegate_conversation(world.front(), task);
    world.person(
        person,
        vec![
            Step::See { find: conversation(Find::text(b"The latest findings")), within: fixtures::WITHIN },
            Step::Gone { find: conversation(Find::text(b"Findings in progress")), within: fixtures::WITHIN },
            Step::Gone { find: Find::named(Role::Button, b"Older"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let Found::Node(log) = TreeFace::new(world.tree(person)).find(&Find::named(Role::Log, b"Conversation")) else {
        panic!("conversation log");
    };
    world.act(person, log, skein_ui::Act::Near { edge: skein_ui::LogEdge::Head });
    world.person(
        person,
        vec![Step::See { find: conversation(Find::text(b"Findings in progress")), within: fixtures::WITHIN }],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_goals_plan_says_which_task_a_delegate_waits_for() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_dependent_delegates(world.front());
    let driver = &world.front().engine;
    let delegates = driver.task(goal).expect("goal").delegates.len();
    let spent = driver.task(first).expect("first delegate").numbers.spent;
    let second_spent = driver.task(second).expect("dependent delegate").numbers.spent;
    assert!(spent > 0, "the row has its own committed spend");
    let budget = fixtures::task_budget(world.front(), goal);
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::text(format!("0 of {delegates} tasks").as_bytes()), within: fixtures::WITHIN },
            Step::See {
                find: goal_task_line(b"Use the findings", Find::text(b"waits on Investigate the change")),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::text(format!("{} of {} spent", budget.numbers.spent, budget.numbers.budget).as_bytes()),
                within: fixtures::WITHIN,
            },
            Step::Gone { find: goal_task_link(fixtures::GOAL_BRIEF), within: fixtures::WITHIN },
            Step::Press { find: goal_task_link(b"Investigate the change") },
            Step::See { find: Find::text(format!("{spent} spent").as_bytes()), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, format!("Task {goal}").as_bytes()) },
            Step::See { find: goal_task_link(b"Use the findings"), within: fixtures::WITHIN },
            Step::Press { find: goal_task_link(b"Use the findings") },
            Step::See { find: Find::text(format!("{second_spent} spent").as_bytes()), within: fixtures::WITHIN },
            Step::See {
                find: own_dependency(second, Find::named(Role::Link, format!("Task {first}").as_bytes())),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_finished_dependency_says_completed_in_its_dependants_line() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_a_finished_dependency(world.front());
    let budget = fixtures::task_budget(world.front(), goal);
    let spent_below = budget.numbers.spent_below;
    assert!(spent_below > 0, "the completed delegate's expense settled to its goal");
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: goal_task_link(b"Use the findings"), within: fixtures::WITHIN },
            Step::Press { find: goal_task_link(b"Use the findings") },
            Step::See {
                find: own_dependency(second, Find::named(Role::Link, format!("Task {first}").as_bytes())),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: own_dependency(second, Find::named(Role::Status, b"Completed")),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_delegates_page_shows_its_brief_run_and_report() {
    let mut world = fixtures::world(fixtures::SEED);
    let fixtures::Dependents { goal, first, second } = fixtures::goal_with_dependent_delegates(world.front());
    let attempt = world.front().engine.last_assignment().attempt;
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(person, open_delegate_and_return_to_goal(goal, first, attempt));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    fixtures::report_first_delegate(world.front(), first);
    world.person(person, see_reported_delegate_and_dependency(first, second, attempt));
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn open_delegate_and_return_to_goal(goal: u64, first: u64, attempt: u64) -> Vec<Step<temper_client_domain::Address>> {
    vec![
        Step::See { find: goal_task_link(b"Investigate the change"), within: fixtures::WITHIN },
        Step::Press { find: goal_task_link(b"Investigate the change") },
        Step::See {
            find: Find::named(Role::Heading { level: 1 }, format!("T{first} · Investigate the change").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(b"Investigate the change")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(format!("Asked by task {goal}").as_bytes())
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::named(Role::Heading { level: 2 }, format!("Run {attempt} · live now").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(b"Findings in progress")
                .within(Box::new([Target::ExactRole { role: Role::Log, name: b"Conversation".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::Gone { find: Find::named(Role::Region, b"Task details"), within: fixtures::WITHIN },
        Step::Gone { find: Find::named(Role::Region, b"Result"), within: fixtures::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Link, format!("Task {goal}").as_bytes()) },
        Step::See { find: Find::named(Role::Heading { level: 2 }, b"Tasks"), within: fixtures::WITHIN },
    ]
}

fn see_reported_delegate_and_dependency(
    first: u64,
    second: u64,
    attempt: u64,
) -> Vec<Step<temper_client_domain::Address>> {
    vec![
        Step::Go { address: fixtures::task_page(first) },
        Step::See {
            find: Find::named(Role::Heading { level: 2 }, format!("Run {attempt} · ended").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::named(Role::Status, b"Completed")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::text(b"The change is feasible")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Result".as_slice().into() }])),
            within: fixtures::WITHIN,
        },
        Step::Gone {
            find: Find::named(Role::Heading { level: 2 }, format!("Run {attempt} · live now").as_bytes()),
            within: fixtures::WITHIN,
        },
        Step::Go { address: fixtures::task_page(second) },
        Step::See {
            find: Find::exactly_named(Role::Link, format!("Task {first}").as_bytes()).within(Box::new([
                Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() },
                Target::ExactRole {
                    role: Role::List,
                    name: format!("Dependencies of task {second}").into_bytes().into(),
                },
            ])),
            within: fixtures::WITHIN,
        },
        Step::See {
            find: Find::named(Role::Status, b"Completed").within(Box::new([
                Target::ExactRole { role: Role::Region, name: b"Task brief".as_slice().into() },
                Target::ExactRole {
                    role: Role::List,
                    name: format!("Dependencies of task {second}").into_bytes().into(),
                },
            ])),
            within: fixtures::WITHIN,
        },
    ]
}

#[test]
fn a_finished_goals_page_shows_its_report() {
    use jig_domain_tasks as tasks;
    let mut world = fixtures::world(fixtures::SEED);
    let goal = fixtures::finished_goal(world.front());
    let key = temper_engine_domain::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(goal)));
    let Some(temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        world.front().engine.store.rows.get(&key)
    else {
        panic!("ended goal retained");
    };
    let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words })) = &row.phase else {
        panic!("goal finished with a report");
    };
    let words = words.clone();
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::text(" · landed · ".as_bytes()), within: fixtures::WITHIN },
            Step::See { find: Find::exactly_named(Role::Heading { level: 1 }, &words), within: fixtures::WITHIN },
            Step::See {
                find: Find::text(&words)
                    .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Report".as_slice().into() }])),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

// A chat's goal and its report, reached through Home and Goals
// (temper/docs/design/domain/testing.md, 2.6 and 6.3;
// temper/docs/design/client/README.md, 5.3, 5.8 and 5.13;
// temper/docs/design/domain/agent.md, 8.3).
#[test]
fn a_chats_goal_shows_its_report_from_its_goals_row() {
    use smith_domain_session::llm::{Block, Message, Role as Speaker};
    use temper_client_domain::{Address, GoalsFilter};

    let mut world = fixtures::world(fixtures::SEED);
    *world.front() = report_front();
    fixtures::adopt_change_repository(world.front());
    let chat = report_chat(world.front());
    world.front().next_client_limits(report_limits());
    let person = world.client_with(fixtures::PARTY, fixtures::chat_page(chat.task), report_limits(), report_root);
    let mut messages = report_host_call(
        world.front(),
        &chat,
        1,
        b"goal",
        br#"{"words":"Write a report for this local demonstration."}"#,
    );
    messages.push(Message {
        role: Speaker::Assistant,
        content: Box::new([Block::Text {
            text: b"I started a goal to write its report.".as_slice().into(),
            replay: None,
        }]),
    });
    report_turn(world.front(), &chat, 1, messages);
    goal_route_steps(
        &mut world,
        person,
        vec![Step::See {
            find: conversation(Find::text(b"I started a goal to write its report.")),
            within: fixtures::WITHIN,
        }],
    );
    assert!(world.root(person).person().forge().ways(1).expect("project ways").goals);

    let goal = world
        .front()
        .assignments
        .iter()
        .find(|run| run.task != chat.task)
        .expect("the chat's goal was assigned")
        .clone();
    report_goal(world.front(), &goal);

    goal_route_steps(
        &mut world,
        person,
        vec![
            Step::Go { address: Address::Home { project: 1 } },
            Step::See { find: Find::exactly_named(Role::Link, b"Work"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Link, b"Work") },
            Step::See { find: Find::named(Role::Link, "Goals · ".as_bytes()), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Link, "Goals · ".as_bytes()) },
        ],
    );
    assert_eq!(world.root(person).person().pages().address(), &Address::Goals { project: 1, filter: GoalsFilter::All });
    goal_route_steps(
        &mut world,
        person,
        vec![
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"Nothing in flight yet."),
                within: fixtures::WITHIN,
            },
            Step::See {
                find: Find::named(Role::Link, b"Write a report for this local demonstration."),
                within: fixtures::WITHIN,
            },
        ],
    );
    let row = world
        .tree(person)
        .nodes()
        .iter()
        .find(|node| {
            node.role == Role::Link
                && node.name.as_deref().is_some_and(|name| {
                    name.starts_with(b"Write a report for this local demonstration.")
                        && name.windows(b"landed".len()).any(|part| part == b"landed")
                })
        })
        .expect("the goal row names its brief and landed state")
        .id;
    world.act(person, row, skein_ui::Act::Press);
    goal_route_steps(
        &mut world,
        person,
        vec![
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"The local goal wrote its report."),
                within: fixtures::WITHIN,
            },
            Step::See { find: report_text(b"I wrote this report on the local fake box."), within: fixtures::WITHIN },
            Step::See { find: report_text(b"Nothing remains in this demonstration."), within: fixtures::WITHIN },
            Step::See {
                find: report_text(b"This scripted model does not change a repository."),
                within: fixtures::WITHIN,
            },
            Step::See { find: Find::exactly_named(Role::Heading { level: 2 }, b"Landing"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.root(person).person().pages().address(), &fixtures::task_page(goal.task));
}

fn report_text(words: &[u8]) -> Find {
    Find::text(words).within(Box::new([Target::ExactRole { role: Role::Region, name: b"Report".as_slice().into() }]))
}

fn report_goal(front: &mut Joint, goal: &temper_engine_domain::Assignment) {
    use smith_domain_session::llm::{Block, Decoded, Message, Role, ToolCall};
    let input = format!(
        r#"{{"scope":{{"kind":"goal","project":1,"goal":{}}},"description":"report: The local goal wrote its report.","body":"I wrote this report on the local fake box.\n\nLeft: Nothing remains in this demonstration.\nRisk: This scripted model does not change a repository."}}"#,
        goal.task,
    );
    let messages = report_host_call(front, goal, 1, b"note", input.as_bytes());
    report_turn(front, goal, 1, messages);
    // Finish is local to the agent; its transcript and terminal cross the worker link.
    report_turn(
        front,
        goal,
        2,
        vec![Message {
            role: Role::Assistant,
            content: Box::new([Block::ToolCall(ToolCall {
                id: b"finish-2".as_slice().into(),
                name: b"finish".as_slice().into(),
                input: br#"{"form":"report","text":"The local goal wrote its report.","fields":{}}"#.as_slice().into(),
                call: Decoded::Historical,
                replay: None,
            })]),
        }],
    );
    front.engine.answer(
        skein_lib::Token::new(7),
        goal.task,
        goal.attempt,
        0,
        jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Report {
                words: b"The local goal wrote its report.".as_slice().into(),
            },
            cancel_delegates: false,
        },
        temper_engine_domain::Parts::none(),
    );
    front.engine.settle();
}

fn goal_route_steps(
    world: &mut jig_client_kit::harness::World<Temper, Joint>,
    person: jig_client_kit::harness::ClientId,
    steps: Vec<Step<temper_client_domain::Address>>,
) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn report_chat(front: &mut Joint) -> temper_engine_domain::Assignment {
    let jig_domain_people::Ask::Create { project, mut batch } =
        front.engine.chat_ask(1, b"Start a local goal and write its report.")
    else {
        panic!("chat creation");
    };
    // The chat reserves 100 for its run and can grant 100 to its goal.
    batch[0].budget = jig_domain_people::Budget::Named;
    batch[0].authority.spend = 300;
    front.engine.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(901)),
        sign_in: front.engine.session(),
        key: [57; 16],
        ask: jig_domain_people::Ask::Create { project, batch },
    }));
    front.engine.settle();
    assert!(
        front.engine.delivered.iter().any(|delivery| matches!(
            delivery,
            temper_engine_domain::Delivery::WebReply {
                reply: jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Created { .. }),
                ..
            }
        )),
        "chat creation: {:?}",
        front.engine.delivered
    );
    front.engine.last_assignment()
}

fn report_front() -> Joint {
    use jig_domain_authority::{Executor, Scopes};
    use temper_engine_domain_forge::settings::{ProjectWays, SizedValue, Stock, Value};
    use temper_engine_domain_world::{store::Store, walking};

    // The chat-draft profile's two slots and report room, with project ways declared.
    let mut limits = temper_client_world::front::engine_limits();
    limits.fleet.slots = 2;
    limits.tasks.tasks = 3;
    limits.tasks.project_tasks = 3;
    limits.tasks.depth = 2;
    limits.tasks.tree_tasks = 3;
    limits.people.inbox_tasks = 6;
    limits.authority.executors = 2;
    limits.tasks.executor_kinds = 2;
    limits.documents.executors = 2;
    limits.tasks.result_bytes = 4096;
    limits.forge.client.op_bytes = limits.forge.client.op_bytes.max(limits.tasks.result_bytes);
    limits.journal.result_bytes = limits.tasks.result_bytes;
    limits.journal.transcript_bytes = 32768;
    limits.tasks.spec_bytes = 1024;
    limits.people.words = 1024;
    limits.tasks.message_bytes = limits.people.words;
    limits.tasks.inbox_bytes = limits.tasks.result_bytes + limits.tasks.message_bytes;
    limits.watermark = temper_engine_domain::default_watermark(&limits.journal);
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        config.core.connectors = Box::new([fixtures::FORGE]);
        for permission in &mut config.core.seed_projects[0].policy.permissions {
            permission.connector = fixtures::FORGE;
        }
        let kinds: Box<[Executor]> = Box::new([Executor::Charter(1), Executor::Role(0)]);
        let ceiling = &mut config.core.defaults.rules.ceiling;
        ceiling.delegation.kinds.clone_from(&kinds);
        ceiling.delegation.depth = 3;
        ceiling.delegation.tasks = 3;
        ceiling.notes = Scopes(1);
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.kinds.clone_from(&kinds);
        policy.roles[0].authority.delegation.kinds = kinds;
        policy.ceiling.delegation.depth = 3;
        policy.roles[0].authority.delegation.depth = 3;
        policy.ceiling.delegation.tasks = 3;
        policy.roles[0].authority.delegation.tasks = 3;
        policy.ceiling.notes = Scopes(1);
        policy.roles[0].authority.notes = Scopes(1);
        for template in &mut config.core.defaults.templates {
            if template.tracked.is_some() {
                template.deadline = jig_domain_people::TemplateDeadline::None;
            }
        }
        let value = Value::Ways(ProjectWays { drafts: false, goals: true, changes: false, way: 0 });
        let mut stock = config.kinds_stock.into_vec();
        stock.push(Stock {
            kind: 7,
            scope: 1,
            number: 1,
            value: SizedValue { size: temper_engine_smith::kinds::size(&value), value },
        });
        config.kinds_stock = stock.into_boxed_slice();
        config
    });
    Joint::with_engine(fixtures::SEED, &report_limits(), Store::with_limits(&limits), configs, &limits, 2)
}

fn report_limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    // Closing chat and Home watches overlap the session's and Goals' watches.
    limits.client.domain.watches = 24;
    limits.client.domain.handles = 24;
    limits
}

fn report_root(seed: u64) -> (temper_client_world::app::Root, temper_client_world::app::View) {
    let limits = report_limits();
    let words = fixtures::phrasebooks(&limits);
    let bounds = fixtures::view_bounds(&limits, &words);
    (
        temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        temper_client_world::app::View::Person(Box::new(temper_client_view::View::new(
            &bounds,
            &limits,
            words,
            fixtures::FORGE,
        ))),
    )
}

fn report_host_call(
    front: &mut Joint,
    run: &temper_engine_domain::Assignment,
    sequence: u32,
    tool: &[u8],
    input: &[u8],
) -> Vec<smith_domain_session::llm::Message> {
    use smith_domain_session::llm::{Block, Decoded, Message, Returned, Role, ToolCall};
    let (text, error) = temper_engine_domain_world::host_tool::host_tool(&mut front.engine, run, sequence, tool, input);
    assert!(!error, "{}: {}", String::from_utf8_lossy(tool), String::from_utf8_lossy(&text));
    let id = format!("{}-{sequence}", String::from_utf8_lossy(tool)).into_bytes().into_boxed_slice();
    vec![
        Message {
            role: Role::Assistant,
            content: Box::new([Block::ToolCall(ToolCall {
                id: id.clone(),
                name: tool.into(),
                input: input.into(),
                call: Decoded::Historical,
                replay: None,
            })]),
        },
        Message {
            role: Role::User,
            content: Box::new([Block::ToolResult { id, result: Returned::Text { text, error: false, replay: None } }]),
        },
    ]
}

fn report_turn(
    front: &mut Joint,
    run: &temper_engine_domain::Assignment,
    sequence: u32,
    messages: Vec<smith_domain_session::llm::Message>,
) {
    let mut turn = temper_worker_domain_world::protocol::session_turn(sequence, 0, 0, sequence == 1);
    turn.messages = messages.into_boxed_slice();
    let body = smith_protocol_channel::encode_turn(
        &turn,
        &smith_transcript::CEILINGS,
        &temper_worker_domain_world::protocol::endpoints(),
    )
    .expect("scripted report turn encodes");
    front.engine.send(temper_engine_domain_world::protocol::turn(
        skein_lib::Token::new(7),
        run.task,
        run.attempt,
        sequence,
        0,
        None,
        &body,
    ));
    front.engine.settle();
}

#[test]
fn a_goals_page_shows_its_plan_with_the_held_task_marked() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, _) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Tasks"), within: fixtures::WITHIN },
            Step::See { find: goal_task_line(b"Review the change", Find::text(b"held")), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
#[test]
fn a_task_with_the_chat_label_opens_as_its_chat() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::task_page(task));
    world.person(person, vec![Step::See { find: Find::named(Role::TextArea, b"Message"), within: fixtures::WITHIN }]);
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
#[test]
fn a_changes_page_says_the_forge_knows_no_change() {
    let mut world = fixtures::world(fixtures::SEED);
    let goal = fixtures::goal_with_forge_panel(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![Step::See { find: Find::text(b"The forge knows no change for this task."), within: fixtures::WITHIN }],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
#[test]
fn a_held_task_released_from_a_goals_plan_runs_again() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, held) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: goal_task_link(b"Review the change"), within: fixtures::WITHIN },
            Step::Press { find: goal_task_link(b"Review the change") },
            Step::See { find: Find::named(Role::Button, b"Release task"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Release task") },
            Step::See { find: Find::named(Role::Dialog, b"Release this task?"), within: fixtures::WITHIN },
            Step::See {
                find: Find::text(b"The next run resumes its conversation; tries count again from zero"),
                within: fixtures::WITHIN,
            },
            Step::Press {
                find: Find::named(Role::Button, b"Release")
                    .within(Box::new([skein_fake_person::Target::Role { role: Role::Dialog, name: Box::new([]) }])),
            },
            Step::See { find: Find::exactly_named(Role::Status, b"running"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let delegate = world.front().engine.task(held).expect("released delegate");
    assert_eq!(delegate.tries, jig_domain_tasks::Tries::NONE);
    assert!(matches!(delegate.phase, jig_domain_tasks::Phase::Active(jig_domain_tasks::Active::Running { .. })));
}

#[test]
fn a_chat_reply_is_identified_by_its_task_in_the_native_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::replied_chat(world.front(), b"Hello");
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    // An inbox entry is named by its kind and its source task (client/README.md, 3.5).
    let entry = |kind: &str| format!("{kind} from T{task}").into_bytes().into_boxed_slice();
    let reply = Find::named(Role::Heading { level: 3 }, b"replied")
        .within(Box::new([skein_fake_person::Target::Role { role: Role::Item, name: entry("Reply") }]));
    world.person(person, vec![Step::See { find: reply, within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    fixtures::finish_replied_chat(world.front(), task);
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Status, b"Done"), within: fixtures::WITHIN },
            Step::See { find: Find::named(Role::Heading { level: 3 }, b"replied"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    let tree = world.tree(person);
    for (object, heading, name) in [
        (jig_client::contract::ObjectKey::Result(task), format!("T{task} Hello is done").as_bytes(), entry("Updates")),
        (jig_client::contract::ObjectKey::Task(task), format!("T{task} Hello replied").as_bytes(), entry("Reply")),
    ] {
        let item = tree
            .nodes()
            .iter()
            .find(|node| node.role == Role::Item && Temper::card(tree, node.id) == Some(observe::key(object)))
            .expect("independent same-task inbox card");
        let mut face = TreeFace::subtree(tree, item.id).expect("card subtree");
        assert_eq!(face.find(&Find::named(Role::Item, &name)), Found::Node(item.id));
        assert!(matches!(face.find(&Find::exactly_named(Role::Heading { level: 3 }, heading)), Found::Node(_)));
    }
    let root = world.root(person).person();
    let inbox = root.pages().inbox().expect("inbox");
    let Some(jig_client::Latest::Jig(jig_client::contract::Delivery::Inbox(owner))) =
        root.client().document(inbox.entries).and_then(|document| document.latest)
    else {
        panic!("owner inbox delivered")
    };
    assert_eq!(owner.results_seen, 0, "Result read position is independent of conversation Seen");
    let reply = owner
        .updates
        .iter()
        .find(|row| row.task_number() == task && matches!(row.object, jig_client::contract::InboxObject::Reply { .. }))
        .expect("same-task unread Reply");
    assert_eq!(reply.unread, Some(true));
    let line = reply.reply.as_ref().expect("Reply task context");
    assert!(line.ended_at.is_some());
    let conversation = line.conversation.expect("conversation read position");
    assert_eq!(conversation.seen, 0);
    assert!(conversation.unread.value > 0);
}

#[test]
fn a_client_closed_on_a_tasks_page_opens_at_the_inbox_again() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, _) = fixtures::goal_with_held_delegate(world.front());
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See { find: goal_task_link(b"Review the change"), within: fixtures::WITHIN },
            Step::Press { find: goal_task_link(b"Review the change") },
            Step::See { find: Find::named(Role::Region, b"Task brief"), within: fixtures::WITHIN },
            Step::Reload,
            Step::See { find: Find::named(Role::Heading { level: 1 }, b"Inbox"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn a_goals_specification_amended_from_its_page_reaches_the_engine() {
    let mut world = fixtures::world(fixtures::SEED);
    let (goal, _) = fixtures::goal_with_held_delegate(world.front());
    let revision = world.front().engine.task(goal).expect("goal").revisions.spec;
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    let words = b"Rotate tokens and preserve existing sessions";
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Button, b"Amend"), within: fixtures::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Amend") },
            Step::Type { find: Find::named(Role::TextArea, b"Specification"), words: words.as_slice().into() },
            Step::Press { find: Find::named(Role::Button, b"Send amendment") },
            Step::See {
                find: Find::named(Role::Status, b"Applied").within(Box::new([Target::Role {
                    role: Role::Region,
                    name: format!("Amend task {goal}").into_bytes().into_boxed_slice(),
                }])),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let task = world.front().engine.task(goal).expect("amended goal");
    assert_eq!(task.spec.words.as_ref(), words);
    assert!(task.revisions.spec > revision);
    assert!(
        world
            .root(person)
            .person()
            .pages()
            .task()
            .expect("goal page")
            .amending
            .as_ref()
            .expect("amendment outcome")
            .done
    );
}

#[test]
fn a_busy_ended_task_read_shows_progress_then_its_document() {
    // The shipped first backoff is one second, followed by the usual one-second response allowance.
    const AFTER_RETRY: skein_lib::Duration = skein_lib::Duration::from_secs(2);
    let mut world = fixtures::world(fixtures::SEED);
    let goal = fixtures::finished_goal(world.front());
    world.front().task_read_ends.push_back(jig_client::contract::ReadNone::Busy);
    let person = world.client(fixtures::PARTY, fixtures::task_page(goal));
    world.person(
        person,
        vec![
            Step::See {
                find: Find::named(Role::Status, format!("Reading task {goal}…").as_bytes()),
                within: fixtures::WITHIN,
            },
            Step::See { find: Find::text(b"The repository is ready"), within: AFTER_RETRY },
            Step::Gone {
                find: Find::named(Role::Status, format!("Reading task {goal}…").as_bytes()),
                within: fixtures::WITHIN,
            },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

#[test]
fn an_unknown_task_address_explains_its_missing_document() {
    const UNKNOWN_TASK: u64 = 999;
    let mut world = fixtures::world(fixtures::SEED);
    let person = world.client(fixtures::PARTY, fixtures::task_page(UNKNOWN_TASK));
    world.person(
        person,
        vec![Step::See { find: Find::named(Role::Status, b"This task could not be found."), within: fixtures::WITHIN }],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn goal_task_link(title: &[u8]) -> Find {
    Find::exactly_named(Role::Link, title)
        .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Tasks".as_slice().into() }]))
}

fn goal_task_line(title: &[u8], find: Find) -> Find {
    find.within(Box::new([
        Target::ExactRole { role: Role::Region, name: b"Tasks".as_slice().into() },
        Target::Role { role: Role::Paragraph, name: title.into() },
    ]))
}
fn own_dependency(task: u64, find: Find) -> Find {
    find.within(Box::new([Target::ExactRole {
        role: Role::List,
        name: format!("Dependencies of task {task}").into_bytes().into(),
    }]))
}
