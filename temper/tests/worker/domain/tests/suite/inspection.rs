//! The root's inspection reads the hub's hosted slots (testing/worker.md, 2).

use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_world::inspect::{Moment, Release, check_tables};
use temper_worker_domain::{Domain, Event, host, step};

#[test]
fn root_inspection_names_the_hosted_attempt_and_slot_limit() {
    let limits = temper_worker_domain_world::limits();
    let mut domain = Domain::new(&limits, Box::new([]));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    step(&mut domain, &env, Event::Ready, &mut out);
    step(
        &mut domain,
        &env,
        Event::Link(host::LinkDown::Assign {
            assignment: host::Assignment {
                assignment: host::RunAssignment {
                    identity: [7; 16],
                    live: host::LiveAsk::NONE,
                    run: Token::new(41),
                    attempt: Token::new(91),
                    workspace: None,
                    save: false,
                    charter: temper_worker_domain_world::protocol::charter(),
                    grants: Box::new([]),
                },
                turns: Box::new([]),
                parts: Box::new([]),
                tell_parts: false,
                answered: Box::new([]),
                messages: Box::new([]),
            },
        }),
        &mut out,
    );
    assert_eq!(domain.host().hosted(), 1, "the assignment holds a slot before the agent answers");
    let mut found = temper_worker_inspect::inspection();
    temper_worker_inspect::inspect_root(&domain, &env.limits, &mut found);
    let [table] = found.tables() else { panic!("one run slots table") };
    assert_eq!(table.name, "run slots");
    assert_eq!(table.capacity, u64::from(env.limits.host.slots));
    let [entry] = table.entries.as_slice() else { panic!("one hosted attempt") };
    assert_eq!(entry.key, "run 41 attempt 91");
    assert_eq!(entry.rule, "the run's end, its answer acknowledged");
    assert_eq!(entry.release, Release::Pending);
    assert_eq!(check_tables(&[&found], Moment { seed: 1, at: env.now, iteration: 1 }), Ok(()));
}
