//! Raw application batches answer unknown names before changing a child
//! (jig/docs/design/domain/engine/README.md, 5.1).
use crate::tests::*;

const PERSON: u64 = 1;
const PERSON_TASK: u64 = 2;
const CREATED: u64 = 3;
const PERIOD: tasks::Funder = tasks::Funder::Period { project: 1, period: 1 };

fn person_task(core: &Core, number: u64) -> tasks::New {
    let mut row = new_row(core, number, PERIOD, None);
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    row.authority.tools = tasks::Tools(0);
    row.authority.models = tasks::Models::NONE;
    row.authority.budget.spend = 0;
    row.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    row.numbers = tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    row
}

fn executor(row: &mut tasks::New, creator: &mut tasks::Party, named: u64) {
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Person(named));
    *creator = tasks::Party::Deployment { project: 1 };
}
fn funder(row: &mut tasks::New, creator: &mut tasks::Party, named: u64) {
    row.funder = tasks::Funder::Pool { project: 1, person: named, period: 1 };
    *creator = tasks::Party::Deployment { project: 1 };
}
fn requester(_row: &mut tasks::New, creator: &mut tasks::Party, named: u64) {
    *creator = tasks::Party::Person(named);
}

fn batch_input(core: &Core, named: u64, name: fn(&mut tasks::New, &mut tasks::Party, u64)) -> Event {
    let mut row = person_task(core, CREATED);
    let mut creator = tasks::Party::Deployment { project: 1 };
    name(&mut row, &mut creator, named);
    Event::Tasks(tasks::Event::Make { reply_to: ReplyTo::new(Token::new(9053)), creator, batch: Box::new([row]) })
}
fn executor_input(core: &Core, named: u64) -> Event {
    batch_input(core, named, executor)
}
fn funder_input(core: &Core, named: u64) -> Event {
    batch_input(core, named, funder)
}
fn requester_input(core: &Core, named: u64) -> Event {
    batch_input(core, named, requester)
}
fn amendment_input(core: &Core, named: u64) -> Event {
    let mut amendment = fixtures::unchanged(b"name a person");
    amendment.addressed = Some(tasks::AddressChange {
        revision: core.tasks.task(PERSON_TASK).expect("person task").revisions.person,
        address: tasks::PersonAddress::Person(named),
    });
    Event::Tasks(tasks::Event::Amend {
        reply_to: ReplyTo::new(Token::new(9053)),
        by: tasks::Party::Person(PERSON),
        funder: None,
        places: Box::new([]),
        operation: tasks::Amend { task: PERSON_TASK, message: 1, stop_run: false, amendment: Box::new(amendment) },
    })
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::print_stderr,
    reason = "the table reports each route, person and case before running it (domain/testing.md, 7)"
)]
fn raw_writers_refuse_unknown_people_before_any_child_changes() {
    type Input = fn(&Core, u64) -> Event;
    let routes: [(&str, Input, u64); 4] = [
        ("executor", executor_input, CREATED),
        ("funder", funder_input, CREATED),
        ("requester", requester_input, CREATED),
        ("amendment", amendment_input, PERSON_TASK),
    ];
    // Four writers, with a held person and an absent person each.
    for (route, input, target) in routes {
        for named in [PERSON, PERSON_TASK] {
            let case = if named == PERSON { "held" } else { "absent" };
            std::eprintln!("route {route}, named person {named}, case {case}");
            let (mut core, limits) = tree_core();
            make_task(&mut core, &limits, PERSON);
            let opens = core.open_person_pool(1, PERSON);
            for event in [opens.0, opens.1].into_iter().flatten() {
                send_task(&mut core, &limits, event);
            }
            let person = person_task(&core, PERSON_TASK);
            send_task(
                &mut core,
                &limits,
                tasks::Event::Make {
                    reply_to: ReplyTo::new(Token::new(1)),
                    creator: tasks::Party::Deployment { project: 1 },
                    batch: Box::new([person]),
                },
            );
            let before = [core.tasks.task(PERSON).cloned(), core.tasks.task(PERSON_TASK).cloned()];
            let funding = core.tasks.funding(PERIOD).copied();
            let event = input(&core, named);
            let mut out = step(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, event);
            let mut refused = None;
            let mut writes = Queue::uncapped();
            while let Some(request) = out.pop() {
                match request {
                    Request::Now(now) => {
                        if let Now::Link(LinkNow::RefusedPayload { problem, .. }) = *now {
                            refused = Some(problem);
                        }
                    }
                    Request::Write(write) => writes.push(write),
                    Request::Held(_) | Request::Ask { .. } | Request::ProjectMade { .. } => {}
                }
            }
            if named == PERSON {
                assert_eq!(refused, None, "{route}");
                assert!(core.tasks.task(target).is_some(), "{route} admitted");
            } else {
                assert_eq!(refused, Some(tasks::Problem::new(Some(target), tasks::Refusal::Unknown)), "{route}");
                assert!(writes.is_empty(), "{route}: {writes:?}");
                assert!(core.tasks.task(CREATED).is_none(), "{route}");
                assert_eq!(
                    [core.tasks.task(PERSON).cloned(), core.tasks.task(PERSON_TASK).cloned()],
                    before,
                    "{route}"
                );
                assert_eq!(core.tasks.funding(PERIOD), funding.as_ref(), "{route}");
            }
        }
    }
}
