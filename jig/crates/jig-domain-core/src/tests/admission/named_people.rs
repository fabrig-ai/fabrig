//! Raw application batches answer unknown names before changing a child
//! (jig/docs/design/domain/engine.md, 5.1).
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

#[test]
fn raw_batches_refuse_unknown_people_before_any_child_changes() {
    type Name = fn(&mut tasks::New, &mut tasks::Party, u64);
    let routes: [(&str, Name); 3] = [("executor", executor), ("funder", funder), ("requester", requester)];
    let mut failures = Vec::new();
    for (route, name) in routes {
        for named in [PERSON, PERSON_TASK] {
            let result = std::panic::catch_unwind(|| {
            let (mut core, limits) = tree_core();
            make_task(&mut core, &limits, PERSON);
            let opens = core.open_person_pool(1, PERSON);
            for event in [opens.0, opens.1].into_iter().flatten() { send_task(&mut core, &limits, event); }
            let person = person_task(&core, PERSON_TASK);
            send_task(&mut core, &limits, tasks::Event::Make {
                reply_to: ReplyTo::new(Token::new(1)),
                creator: tasks::Party::Deployment { project: 1 },
                batch: Box::new([person]),
            });
            let before = [core.tasks.task(PERSON).cloned(), core.tasks.task(PERSON_TASK).cloned()];
            let funding = core.tasks.funding(PERIOD).cloned();
            let mut row = person_task(&core, CREATED);
            let mut creator = tasks::Party::Deployment { project: 1 };
            name(&mut row, &mut creator, named);
            let mut out = step(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
                Event::Tasks(tasks::Event::Make {
                    reply_to: ReplyTo::new(Token::new(9053)), creator, batch: Box::new([row]),
                }));
            let mut refused = None;
            let mut writes = Vec::new();
            while let Some(request) = out.pop() {
                match request {
                    Request::Now(now) => if let Now::Link(LinkNow::RefusedPayload { problem, .. }) = *now {
                        refused = Some(problem);
                    },
                    Request::Write(write) => writes.push(write),
                    Request::Held(_) | Request::Ask { .. } | Request::ProjectMade { .. } => {},
                }
            }
            if named == PERSON {
                assert_eq!(refused, None, "{route}");
                assert!(core.tasks.task(CREATED).is_some(), "{route} admitted");
            } else {
                assert_eq!(refused, Some(tasks::Problem::new(Some(CREATED), tasks::Refusal::Unknown)), "{route}");
                assert!(writes.is_empty(), "{route}: {writes:?}");
                assert!(core.tasks.task(CREATED).is_none(), "{route}");
                assert_eq!([core.tasks.task(PERSON).cloned(), core.tasks.task(PERSON_TASK).cloned()], before, "{route}");
                assert_eq!(core.tasks.funding(PERIOD), funding.as_ref(), "{route}");
            }
            });
            if result.is_err() { failures.push((route, named)); }
        }
    }
    assert!(failures.is_empty(), "failed rows: {failures:?}");
}
