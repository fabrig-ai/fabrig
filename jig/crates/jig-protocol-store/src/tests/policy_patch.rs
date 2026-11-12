//! Retained policy grants meet the deployment (protocol/store/records.md, 1).
use super::fixtures;
use crate::{codec::people, patches};
use jig_domain_authority::Executor;
use jig_domain_people::{Executor as PolicyExecutor, Stored};
use skein_codec::patch::{Capacity, Patcher, Step};

fn narrowed(executors: &[Executor]) -> jig_domain_people::PolicyValue {
    let mut value = fixtures::people_policy();
    let retained = Box::new([
        PolicyExecutor::Charter(91),
        PolicyExecutor::Charter(7),
        PolicyExecutor::Procedure(12),
        PolicyExecutor::Role(4),
    ]);
    value.ceiling.delegation.kinds = retained.clone();
    for role in &mut value.roles {
        role.authority.delegation.kinds = retained.clone();
    }
    let limits = people::limits(&fixtures::CORE_LIMITS).expect("fixture bounds");
    let record = Stored::Policy { project: 3, name: "Tally".into(), value: value.clone() };
    let mut bytes = people::encode(&record, &limits, &[]).expect("policy row");
    // The written conversion changes meaning; versions 14 and 15 have the same fields.
    *bytes.get_mut(1).expect("version prefix") = 14;
    let mut patcher = Patcher::new(Capacity {
        versions: 3,
        patches: 2,
        text: skein_codec::schema::MAX_TEXT,
        bytes: 131_072,
        nodes: 16_384,
    });
    for text in patches::kept::jig_store_people::TEXTS {
        patcher.keep(text.as_bytes()).expect("kept schema");
    }
    let mut context = patches::Context { executors: executors.into() };
    for (version, patch) in [(14, 0), (15, 1)] {
        let root = patcher.schema(version).unwrap().find(b"Row").unwrap();
        let plan = patcher.plan(patches::PATCHES[patch].as_bytes()).unwrap().unwrap();
        patcher.start(plan, root, bytes).expect("retained row");
        loop {
            match patcher.next().expect("patch step") {
                Step::Written { site, at } => {
                    let (old, new) = patcher.parts();
                    patches::written(site, old, new, at, &mut context).expect("policy narrowing");
                }
                Step::Held { at, .. } => {
                    let (old, new) = patcher.parts();
                    new.carry(old, at).expect("unchanged held field");
                }
                Step::Done => break,
            }
        }
        bytes = skein_lib::bytes::copy_of(patcher.finish().expect("patched policy row"));
    }
    let Stored::Policy { project, name, value: result } = people::decode(&bytes, &limits, &[]).expect("current row")
    else {
        panic!("policy row")
    };
    assert_eq!(project, 3);
    assert_eq!(name.as_ref(), "Tally");
    value.ceiling.delegation.kinds = result.ceiling.delegation.kinds.clone();
    for (expected, actual) in value.roles.iter_mut().zip(result.roles.iter()) {
        expected.authority.delegation.kinds = actual.authority.delegation.kinds.clone();
    }
    assert_eq!(result, value, "all other policy fields survive");
    result
}

#[test]
fn retired_executors_drop_and_configured_grants_survive() {
    let value = narrowed(&[Executor::Charter(7), Executor::Procedure(12), Executor::Role(4), Executor::Charter(88)]);
    assert_eq!(
        value.ceiling.delegation.kinds.as_ref(),
        &[PolicyExecutor::Charter(7), PolicyExecutor::Procedure(12), PolicyExecutor::Role(4)]
    );
}

#[test]
fn each_roles_delegation_is_narrowed_like_the_ceiling() {
    let value = narrowed(&[Executor::Procedure(12)]);
    assert!(!value.roles.is_empty());
    for role in &value.roles {
        assert_eq!(role.authority.delegation.kinds.as_ref(), &[PolicyExecutor::Procedure(12)]);
    }
}

#[test]
fn no_deployment_grants_leaves_no_delegation_executors() {
    let value = narrowed(&[]);
    assert!(value.ceiling.delegation.kinds.is_empty());
    for role in &value.roles {
        assert!(role.authority.delegation.kinds.is_empty());
    }
}
