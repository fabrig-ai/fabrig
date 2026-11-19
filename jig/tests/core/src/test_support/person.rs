//! Person-task policy shared by memory and disk stories (jig/docs/design/domain/testing.md, 9).
use jig_domain_authority as authority;
use jig_test_domain as root;
use skein_lib::Queue;

#[must_use]
pub fn person_world_fixture(seed: u64) -> (root::Config, root::Limits) {
    let (mut configuration, mut limits) = crate::faults::plan_fixture(seed);
    limits.core.tasks.person_stall = skein_lib::Duration::from_secs(1);
    let kinds: Box<[authority::Executor]> = Box::new([authority::Executor::Charter(1), authority::Executor::Role(0)]);

    let mut rules = configuration.core.defaults.rules.clone();
    rules.ceiling.delegation.kinds.clone_from(&kinds);
    let mut policy = configuration.core.seed_projects[0].policy.policy.clone();
    policy.ceiling.delegation.kinds.clone_from(&kinds);
    policy.roles[0].authority.delegation.kinds = kinds;
    let mut domain = authority::Domain::new(rules, limits.core.authority).expect("bounded person-task policy");
    let mut out = Queue::uncapped();
    authority::step(&mut domain, authority::Event::Policy { project: 1, policy }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    configuration.core.seed_projects[0].policy.policy = domain.policy(1).expect("checked seed policy").clone();
    configuration.core.defaults.rules = (jig_domain_authority::Domain::new(domain.rules().clone(), *domain.limits())
        .expect("checked rules"))
    .rules()
    .clone();
    (configuration, limits)
}
