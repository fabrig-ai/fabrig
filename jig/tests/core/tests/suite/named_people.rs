//! Outside person names must link at the next start (jig/docs/design/domain/engine.md, 5.1 and 6).
use jig_core_world::fixtures::{ask, live_row};
use jig_core_world::world::{Setup, World};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

const SEED: u64 = 29_501;
const PROJECT: u32 = 1;
const PERSON: u64 = 1;
const PERSON_TASK: u64 = 2;
const ROLE: u32 = 0;

fn configuration() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = jig_test_profile::effect_fixture(SEED, false);
    // The fixture offers a charter, a procedure and the owner's person role.
    limits.core.tasks.executor_kinds = 3;
    limits.core.authority.executors = 3;
    limits.core.documents.executors = 3;
    let kinds: Box<[authority::Executor]> = Box::new([
        authority::Executor::Charter(1),
        authority::Executor::Procedure(1),
        authority::Executor::Role(ROLE),
    ]);
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    (config, limits)
}

fn world() -> World {
    let (config, limits) = configuration();
    let mut world = World::new(Setup { seed: SEED, configuration: Some(config), limits, ..Setup::default() });
    let mut member = jig_test_profile::fixtures::people_member(b"check the answer", 0);
    member.executor = people::TaskExecutor::Person(people::PersonAddress::Role(ROLE));
    ask(&mut world, 51, people::Ask::Create { project: PROJECT, batch: Box::new([member]) });
    world
}

fn amendment(person: u64) -> people::Amendment {
    people::Amendment {
        addressed: Some(people::AddressChange { revision: 0, address: people::PersonAddress::Person(person) }),
        tracked: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: b"name the person".as_slice().into(),
    }
}

fn amend(world: &mut World, person: u64) {
    let revision = live_row(world, PERSON_TASK).revisions.spec;
    ask(
        world,
        52,
        people::Ask::Amend {
            project: PROJECT,
            task: PERSON_TASK,
            revision,
            amendment: amendment(person),
            release: None,
        },
    );
}

#[test]
fn a_person_named_from_outside_is_held_or_refused_and_the_start_links() {
    type Input = fn(&mut World, u64);
    let routes: [(&str, Input); 1] = [("party amendment", amend)];
    for (route, input) in routes {
        for (named, held) in [(PERSON, true), (PERSON_TASK, false)] {
            let mut world = world();
            let before = live_row(&world, PERSON_TASK).clone();
            input(&mut world, named);
            let answer = world.people_answers.last().cloned();
            let after = live_row(&world, PERSON_TASK).clone();
            world.restart_with(configuration().0);
            assert_eq!(world.domain.start_failure(), None, "{route}, named {named}");
            assert!(world.domain.ready(), "{route}, named {named}: {:?}", world.domain.start_phase());
            assert_eq!(world.domain.start_phase(), None, "{route} completed Open, named {named}");
            if held {
                assert_eq!(after.executor, tasks::Executor::Person(tasks::PersonAddress::Person(named)), "{route}");
                assert!(matches!(answer, Some(people::Reply::Outcome(people::Outcome::Amended { .. }))), "{answer:?}");
            } else {
                assert_eq!(answer, Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown))), "{route}");
                assert_eq!(after, before, "{route} changes nothing");
            }
            assert!(!world.store.rows.values().any(|row| matches!(row,
                root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))
                    if row.executor == tasks::Executor::Person(tasks::PersonAddress::Person(PERSON_TASK))
            )), "{route} saves no absent person");
        }
    }
}
