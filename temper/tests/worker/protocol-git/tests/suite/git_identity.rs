//! Missing identity values stop at git admission: protocol/credentials.md, 4,
//! and protocol/git.md, 2. The existing world supplies the value directly;
//! selection from GitHub entries belongs to the worker routing's harness.
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git_world::{
    fixtures,
    referee::{Expected, Verdict},
    world,
};

fn refuses_without_git(mut scenario: world::Scenario) {
    scenario.identity = false;
    scenario.script.invocations.clear();
    scenario.expected = Expected::fault(git::Fault::Broken);
    let ran = world::run(&scenario, fixtures::SEED);
    assert_eq!(ran.verdict, Verdict::Passed);
    assert_eq!(ran.faults(), [(fixtures::OWNER, Some(git::Fault::Broken))]);
    assert_eq!(ran.spawns, 0, "no git invocation may begin without the identity value");
}

#[test]
fn remote_operations_without_an_identity_never_run_git() {
    for (_, scenario) in fixtures::operations().into_iter().filter(|(_, scenario)| scenario.op.is_remote()) {
        refuses_without_git(scenario);
    }
}

#[test]
fn commits_without_an_author_identity_never_run_git() {
    for merging in [false, true] {
        refuses_without_git(fixtures::commit(merging));
    }
}
