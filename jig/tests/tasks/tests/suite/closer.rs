//! Closer ending authority is retained and checked at making (jig/docs/design/domain/tasks.md, 5.10).
use jig_domain_tasks::{self as tasks, EndedBy, Executor, Party, Problem, Refusal};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn member() -> tasks::New {
    let mut member = task(1, &[]);
    member.flags.ended_by = EndedBy::Closer(tasks::Closer { connector: 1, code: 6 });
    member.authority.delegation.kinds = Box::new([tasks::AuthorityExecutor::Procedure(6)]);
    member
}
#[test]
fn a_task_made_with_a_closer_keeps_it_through_restart() {
    let mut world = World::new(861, LIMITS);
    let member = member();
    let flags = member.flags;
    assert_eq!(world.make(Party::Person(1), vec![member]), Reply::Made(vec![1]));
    assert_eq!(world.record(1).flags, flags);
    assert_eq!(world.record(1).closer, None);
    let record = world.record(1).clone();
    world.restart();
    assert_eq!(world.record(1), &record);
}
#[test]
fn a_closer_on_a_conversation_is_refused() {
    let mut member = member();
    member.flags.conversation = true;
    refused(member);
}
#[test]
fn a_closer_on_a_task_that_is_no_agents_is_refused() {
    for executor in [Executor::Person(tasks::PersonAddress::Person(1)), Executor::Procedure { connector: 1, code: 6 }] {
        let mut member = member();
        member.executor = executor;
        refused(member);
    }
}
fn refused(member: tasks::New) {
    let mut world = World::new(861, LIMITS);
    let before = world.records.clone();
    assert_eq!(world.make(Party::Person(1), vec![member]), Reply::Refused(Problem::new(Some(1), Refusal::Contract)));
    assert_eq!(world.records, before);
}

#[test]
fn a_closer_its_tasks_delegation_does_not_name_is_refused() {
    use jig_core_kit::{
        fixtures,
        world::{Setup, World as CoreWorld},
    };
    use jig_domain_people as people;
    let mut world = CoreWorld::new(Setup { seed: 861, ..Setup::default() });
    let people::Ask::Create { project, mut batch } = fixtures::chat(&world, 1, b"close".as_slice().into()) else {
        panic!("create fixture");
    };
    batch[0].flags.conversation = false;
    batch[0].flags.ended_by = people::EndedBy::Closer(people::Closer { connector: 1, code: 6 });
    batch[0].authority.delegation.kinds = Box::new([]);
    fixtures::ask(&mut world, 91, people::Ask::Create { project, batch });
    fixtures::refusal(
        &world,
        people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::TaskKind {
                executor: Some(people::TaskExecutor::Procedure { connector: 1, code: 6 }),
            },
        },
    );
}

#[test]
fn a_delegates_closer_must_be_named_by_its_own_delegation() {
    use jig_core_kit::{
        fixtures,
        world::{Setup, World as CoreWorld},
    };
    let mut world = CoreWorld::new(Setup { seed: 861, ..Setup::default() });
    let mut member = fixtures::member(&world, Executor::Agent { charter: 1 }, b"close");
    member.flags.ended_by = EndedBy::Closer(tasks::Closer { connector: 1, code: 6 });
    let key = world.call_key(1);
    let limits = world.env().limits.core.tasks;
    let findings = Box::new([jig_domain_authority::Finding::Executor { source: jig_domain_authority::Source::Task }]);
    assert_eq!(
        world.domain.core_mut().delegate_preflight(&limits, key, &[member.clone()], skein_lib::Wall::EPOCH),
        Err(jig_domain_core::CallPart::DelegationDenied {
            answer: jig_domain_authority::Answer::Refuse,
            findings: findings.clone(),
        }),
    );
    assert_eq!(
        world.domain.core_mut().procedure_batch_admit(&limits, key.task, &[member], skein_lib::Wall::EPOCH),
        Err(jig_domain_core::StepRefusal::Denied { answer: jig_domain_authority::Answer::Refuse, findings }),
    );
}
