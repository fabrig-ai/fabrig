//! Procedure waits from covering landing rules (temper/docs/design/domain/forge.md, 8.7).
use super::*;

fn template(repository: &Repository, last: GateLast, requires: Requires) -> GateTemplate {
    GateTemplate {
        pattern: GatePattern {
            segments: Box::new([
                Box::from(b"forge".as_slice()),
                repository.host.clone(),
                repository.owner.clone(),
                repository.name.clone(),
                Box::from(b"branch".as_slice()),
            ]),
            last,
        },
        gates: Box::new([]),
        requires,
    }
}

#[test]
fn requires_both_where_no_rule_covers_the_branch() {
    let mut d = domain();
    assert!(d.gate_policy(GatePolicy::empty(LIMITS.judge_projects), &LIMITS));
    assert_eq!(d.requires(&adopted(), b"main"), Requires { ci: true, up_to_date: true });
}

#[test]
fn requires_neither_under_a_rule_asking_neither() {
    let mut d = domain();
    let repository = adopted();
    let mut policy = GatePolicy::empty(LIMITS.judge_projects);
    policy.deployment = Box::new([template(
        &repository,
        GateLast::Open(Box::from(b"temper".as_slice())),
        Requires { ci: false, up_to_date: false },
    )]);
    assert!(d.gate_policy(policy, &LIMITS));
    assert_eq!(d.requires(&repository, b"temper/7/g7"), Requires { ci: false, up_to_date: false });
    assert_eq!(d.requires(&repository, b"main"), Requires { ci: true, up_to_date: true });
}

#[test]
fn requires_the_union_of_the_covering_rules() {
    let mut d = domain();
    let repository = adopted();
    let ci =
        template(&repository, GateLast::Exact(Box::from(b"main".as_slice())), Requires { ci: true, up_to_date: false });
    let up_to_date =
        template(&repository, GateLast::Exact(Box::from(b"main".as_slice())), Requires { ci: false, up_to_date: true });
    let mut policy = GatePolicy::empty(LIMITS.judge_projects);
    policy.deployment = Box::new([ci.clone()]);
    policy.projects.insert(repository.project + 1, Box::new([up_to_date.clone()])).expect("other project");
    assert!(d.gate_policy(policy, &LIMITS));
    assert_eq!(d.requires(&repository, b"main"), Requires { ci: true, up_to_date: false });
    assert_eq!(d.requires(&repository, b"main/child"), Requires { ci: true, up_to_date: true });

    let mut policy = GatePolicy::empty(LIMITS.judge_projects);
    policy.deployment = Box::new([ci]);
    policy.projects.insert(repository.project, Box::new([up_to_date.clone()])).expect("repository's project");
    policy.projects.insert(repository.project + 1, Box::new([up_to_date])).expect("other project");
    assert!(d.gate_policy(policy, &LIMITS));
    assert_eq!(d.requires(&repository, b"main"), Requires { ci: true, up_to_date: true });
}

#[test]
fn requires_a_branch_with_no_name_keeps_the_waits() {
    let mut d = domain();
    let repository = adopted();
    let mut policy = GatePolicy::empty(LIMITS.judge_projects);
    policy.deployment =
        Box::new([template(&repository, GateLast::Open(Box::new([])), Requires { ci: false, up_to_date: false })]);
    assert!(d.gate_policy(policy, &LIMITS));
    assert_eq!(d.requires(&repository, b"a/b"), Requires { ci: false, up_to_date: false });
    for branch in [b"a//b".as_slice(), b"", b"/a", b"a/"] {
        assert_eq!(d.requires(&repository, branch), Requires { ci: true, up_to_date: true });
    }
}
