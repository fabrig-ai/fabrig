//! Seed references copy the same three parts as `MakeProject` (domain/people/acts.md, 2).
use super::{defaults, fixtures, policies};
use crate::{Defaults, Path, Problem, Segment, What};

fn document(project: &[u8]) -> Box<[u8]> {
    fixtures::with(&policies::override_document(2), br#""projects": []"#, project)
}
fn without_defaults(document: &[u8]) -> Box<[u8]> {
    // policies::override_document puts policies last, after projects. The older
    // defaults fixture has none, so remove this member before using its helper.
    let start = skein_lib::bytes::find(document, br#", "policies": ["#).unwrap();
    let end = document.len().checked_sub(1).unwrap();
    let document = fixtures::with(document, &document[start..end], b"");
    defaults::without_defaults(&document)
}
fn seed_path() -> Path {
    Path::root().member(b"projects").child(Segment::Index(0))
}
fn given() -> jig_domain_core::settings::Defaults {
    crate::core(&fixtures::engine(&policies::override_document(2)).unwrap(), [3; 16], 11).defaults
}

#[test]
fn a_seed_naming_a_policy_copies_it() {
    let document = document(br#""projects": [{"number":7,"name":"Seed","policy":1}]"#);
    for mode in [Defaults::Document, Defaults::Passed] {
        let mut engine = match mode {
            Defaults::Document => defaults::parse(&document, mode).unwrap(),
            Defaults::Passed => defaults::parse(&without_defaults(&document), mode).unwrap(),
        };
        if mode == Defaults::Passed {
            let mut defaults = given();
            // The current defaults, not the document's former policy, supply the copy.
            defaults.policies.get_mut(&1).unwrap().templates[0].priority = Some(7);
            crate::pass(&mut engine, defaults).unwrap();
        }
        let core = crate::core(&engine, [3; 16], 11);
        let seed = &core.seed_projects[0];
        let expected = core.defaults.policies.get(&1).unwrap();
        assert_eq!(seed.number, 7);
        assert_eq!(seed.name.as_ref(), "Seed");
        assert_eq!(&seed.policy, expected);
        assert!(!seed.policy.permissions.is_empty());
        assert!(!seed.policy.templates.is_empty());
        // A later defaults change cannot change the project's owned copy.
        engine.policies[0].policy.period_spend = 99;
        assert_eq!(engine.projects[0].policy.period_spend, 100);
    }
}

#[test]
fn a_seed_naming_an_unknown_policy_is_refused() {
    let document = document(br#""projects": [{"number":1,"policy":9}]"#);
    for mode in [Defaults::Document, Defaults::Passed] {
        let problems = match mode {
            Defaults::Document => defaults::parse(&document, mode).unwrap_err(),
            Defaults::Passed => {
                let mut engine = defaults::parse(&without_defaults(&document), mode).unwrap();
                crate::pass(&mut engine, given()).unwrap_err()
            }
        };
        assert_eq!(
            problems.problems.as_slice(),
            &[Problem { path: seed_path().member(b"policy"), what: What::Refers }]
        );
        assert_eq!(problems.past, 0);
    }
}

#[test]
fn a_seed_naming_a_policy_refuses_the_members_it_gives() {
    // Permission-role mappings are the only copied member beside policy itself.
    // Template overrides and the authority's roles live inside the inline policy.
    for (member, project) in [
        (b"roles".as_slice(), br#""projects": [{"number":1,"policy":1,"roles":[]}]"#.as_slice()),
        (b"templates".as_slice(), br#""projects": [{"number":1,"policy":1,"templates":[]}]"#.as_slice()),
    ] {
        for mode in [Defaults::Document, Defaults::Passed] {
            let document = document(project);
            let document = if mode == Defaults::Passed { without_defaults(&document) } else { document };
            let problems = defaults::parse(&document, mode).unwrap_err();
            assert_eq!(
                problems.problems.as_slice(),
                &[Problem { path: seed_path().member(member), what: What::Unknown }]
            );
            assert_eq!(problems.past, 0);
        }
    }
}

#[test]
fn a_copied_seed_is_checked_against_the_passed_rules() {
    let document = document(br#""projects": [{"number":1,"policy":1}]"#);
    let mut engine = defaults::parse(&without_defaults(&document), Defaults::Passed).unwrap();
    let mut defaults = given();
    defaults.rules.period_spend = 99;
    let problems = crate::pass(&mut engine, defaults).unwrap_err();
    assert_eq!(
        problems.problems.as_slice(),
        &[Problem {
            path: seed_path().member(b"policy"),
            what: What::Because(b"policy authority or period spend exceeds deployment rules")
        }]
    );
}

#[test]
fn a_seed_policy_number_must_be_a_u32() {
    for number in [b"-1".as_slice(), b"1.5", b"4294967296"] {
        let project = fixtures::with(br#""projects": [{"number":1,"policy":N}]"#, b"N", number);
        let problems = fixtures::engine(&document(&project)).unwrap_err();
        assert_eq!(problems.problems.as_slice(), &[Problem { path: seed_path().member(b"policy"), what: What::Bound }]);
    }
}

// jig/docs/design/protocol/configuration.md, 3: the copied policy is checked whole.
#[test]
fn a_copied_seed_names_its_executor_count_and_limit() {
    let document = document(br#""projects": [{"number":1,"policy":1}]"#);
    for limit in [19, 20] {
        let mut engine = defaults::parse(&without_defaults(&document), Defaults::Passed).unwrap();
        engine.limits.core.authority.executors = limit;
        let mut defaults = given();
        let kinds = Box::new([jig_domain_authority::Executor::Charter(1); 20]);
        defaults.rules.ceiling.delegation.kinds = kinds.clone();
        defaults.policies.get_mut(&1).unwrap().policy.ceiling.delegation.kinds = kinds;
        let result = crate::pass(&mut engine, defaults);
        if limit == 20 {
            result.expect("twenty executors fit exactly");
        } else {
            let problems = result.unwrap_err();
            assert_eq!(
                problems.problems.as_slice(),
                &[Problem {
                    path: seed_path().member(b"policy").member(b"ceiling").member(b"delegation").member(b"kinds"),
                    what: What::PolicyExecutors { executors: 20, limit: 19 },
                }]
            );
            assert_eq!(problems.problems.get(0).unwrap().codec_message().unwrap().as_ref(), b"projects[0].policy.ceiling.delegation.kinds: policy has 20 executor kinds; authority.executors limit is 19.");
        }
    }
}

#[test]
fn a_copied_seed_names_its_snapshot_bytes_and_limit() {
    let document = document(br#""projects": [{"number":1,"policy":1}]"#);
    let initial = given();
    let policy = initial.policies.get(&1).unwrap();
    let snapshot = jig_domain_core::policy_snapshot(&policy.policy, &policy.permissions, &policy.templates).unwrap();
    let bytes = jig_domain_people::policy_bytes(&snapshot).unwrap();
    for limit in [bytes - 1, bytes] {
        let mut engine = defaults::parse(&without_defaults(&document), Defaults::Passed).unwrap();
        engine.limits.core.policy_bytes = limit;
        let result = crate::pass(&mut engine, given());
        if limit == bytes {
            result.expect("whole policy fits exactly");
        } else {
            let problems = result.unwrap_err();
            assert_eq!(
                problems.problems.as_slice(),
                &[Problem { path: seed_path().member(b"policy"), what: What::PolicyBytes { bytes, limit } }]
            );
            let message = problems.problems.get(0).unwrap().codec_message().unwrap();
            assert!(skein_lib::bytes::find(&message, skein_lib::Decimal::of(bytes).as_bytes()).is_some());
            assert!(skein_lib::bytes::find(&message, skein_lib::Decimal::of(limit).as_bytes()).is_some());
            assert!(skein_lib::bytes::find(&message, b"core.policy_bytes").is_some());
        }
    }
}
