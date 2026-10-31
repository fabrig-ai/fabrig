//! Ops' pooled provisioning procedures retain slots through a quota shrink.
use super::{Violation, World, check, core, ended, infra, production, root, state, state_mut, tasks};
const NAMES: [&str; 3] = ["first", "second", "waiter"];
fn environment(index: usize) -> infra::Environment {
    infra::Environment::new(b"staging".as_slice().into(), NAMES[index].as_bytes().into())
}
pub(super) fn quota(world: &mut World, slots: u32) {
    world.systems.production.set_pool_quota("staging", slots);
    let event =
        world.systems.infrastructure(infra::SystemRequest::Pool { pool: infra::Pool(b"staging".as_slice().into()) });
    world.send(root::Event::Infrastructure(infra::Event::System(event)));
}
pub(super) fn fill(world: &mut World) -> Result<(), Violation> {
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Kinds {
        connector: 2,
        kinds: Box::new([
            tasks::Kind { connector: 2, kind: 1, hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits } },
            tasks::Kind { connector: 2, kind: 2, hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits } },
            tasks::Kind { connector: 2, kind: 3, hold: tasks::HoldKind::Pooled { taken: tasks::Taken::Waits } },
        ]),
    })));
    for resource in std::iter::once(infra::Resource::Pool(infra::Pool(b"staging".as_slice().into())))
        .chain((0..3).map(|index| infra::Resource::Environment(environment(index))))
    {
        world.send(root::Event::Infrastructure(infra::Event::Names {
            task: 0,
            project: 1,
            resources: Box::new([resource]),
        }));
    }
    quota(world, 2);
    world.drain()?;
    for (index, delay) in [0, 1, 0].into_iter().enumerate() {
        world.systems.production.queue_fault(production::Fault::SlowProvisioning { seconds: delay });
        start(world, index);
        world.drain()?;
    }
    let children = world
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record
                && row.spec.words.as_ref() == b"pool holder"
            {
                Some(row.number)
            } else {
                None
            }
        })
        .collect();
    state_mut(world).children = children;
    check(state(world).children.len() == 3, "three provisioning tasks were not made")?;
    check(
        holders(world) == 2,
        &format!(
            "pool holders: {:?}; systems {:?}",
            state(world)
                .children
                .iter()
                .map(|&task| {
                    let row = super::row(world, task);
                    (&row.phase, row.holds_taken, &row.holdings)
                })
                .collect::<Vec<_>>(),
            world.systems.production.observed()
        ),
    )
}
fn holders(world: &World) -> usize {
    state(world)
        .children
        .iter()
        .filter(|&&task| {
            let row = super::row(world, task);
            row.holds_taken && !matches!(row.phase, tasks::Phase::Ended(_))
        })
        .count()
}
pub(super) fn finish(world: &mut World, index: u32) -> Result<(), Violation> {
    let index = usize::try_from(index).expect("three holders");
    let task = state(world).children[index];
    world.advance(if index == 1 { 1_000_000_000 } else { 5_000_000_000 });
    for _ in 0..256 {
        world.iterate()?;
        if ended(world, task) {
            break;
        }
    }
    let environment =
        world.systems.production.read_environment(&production::EnvironmentName::new("staging", NAMES[index]));
    check(
        ended(world, task),
        &format!(
            "provisioning holder {index} not complete: {:?}; rows {:?}; environment {:?}",
            super::row(world, task).phase,
            world
                .store
                .rows
                .values()
                .filter(|record| matches!(record, root::Record::Infrastructure(infra::Record::Procedure(_))))
                .collect::<Vec<_>>(),
            environment
        ),
    )?;
    // The independent participant releases the completed environment before
    // the next system write reaches the quota. The jig slot was kept until end.
    world.systems.production.other_hand(production::Fault::HandDelete {
        environment: production::EnvironmentName::new("staging", NAMES[index]),
    });
    world.drain()?;
    if index == 0 {
        check(
            holders(world) == 1 && !super::row(world, state(world).children[2]).holds_taken,
            "shrinking the pool admitted its waiter while the second holder remained",
        )?;
    }
    Ok(())
}
pub(super) fn check_ending(world: &World) -> Result<(), Violation> {
    check(
        state(world).children.len() == 3 && state(world).children.iter().all(|&task| ended(world, task)),
        "pool did not eventually serve and finish all three provisioning tasks",
    )?;
    check(
        world
            .systems
            .production
            .observed()
            .iter()
            .filter(|effect| matches!(effect, production::ObservedEffect::Create { applied: true, .. }))
            .count()
            == 3,
        "pool waiter never reached the independent production",
    )
}
fn start(world: &mut World, index: usize) {
    let authority = super::super::config::environment_authority(150, false);
    let mut standing = super::super::config::task_authority(150);
    standing.grants = authority
        .grants
        .into_iter()
        .map(|grant| tasks::Grant {
            connector: grant.connector,
            kind: grant.kind,
            pattern: tasks::Pattern {
                segments: grant.pattern.segments,
                last: match grant.pattern.last {
                    jig_domain_authority::Last::Exact(last) => tasks::Last::Exact(last),
                    jig_domain_authority::Last::Open(last) => tasks::Last::Open(last),
                },
            },
        })
        .collect();
    standing.delegation.kinds = Box::new([tasks::AuthorityExecutor::Procedure(infra::PROVISION_CODE)]);
    let mut child = standing.clone();
    child.budget.spend = 40;
    child.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::StartRecurring {
        project: 1,
        authority: standing,
        template: tasks::RecurringTemplate {
            key: u32::try_from(index + 1).expect("three templates"),
            overlap: tasks::RecurringOverlap::Skip,
            batch: Box::new([tasks::New {
                deadline: tasks::Deadline::Named,
                label: 0,
                flags: tasks::Flags::PLAIN,
                number: 1,
                project: 1,
                executor: tasks::Executor::Procedure { connector: 2, code: infra::PROVISION_CODE },
                llm: None,
                spec: tasks::Spec {
                    asks: false,
                    words: b"pool holder".as_slice().into(),
                    parameters: Box::new([
                        tasks::Parameter::Bytes { name: 1, value: b"staging".as_slice().into() },
                        tasks::Parameter::Bytes { name: 2, value: NAMES[index].as_bytes().into() },
                        tasks::Parameter::Number { name: 3, value: 10_000 },
                        tasks::Parameter::Number { name: 4, value: 40 },
                        tasks::Parameter::Number { name: 5, value: 10_000 },
                    ]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority: child,
                numbers: tasks::Numbers { budget: 40, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 0 },
                dependencies: Box::new([]),
                holdings: Box::new([
                    tasks::Holding::Write {
                        resource: tasks::Name {
                            connector: 2,
                            path: infra::Resource::Environment(environment(index)).segments(),
                        },
                        kind: 2,
                    },
                    tasks::Holding::Slot {
                        pool: tasks::Name {
                            connector: 2,
                            path: infra::Resource::Pool(infra::Pool(b"staging".as_slice().into())).segments(),
                        },
                        kind: 3,
                    },
                ]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        },
    }));
}
