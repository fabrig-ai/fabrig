//! Seed references copy the same three parts as `MakeProject` (people.md, 5.2).
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
    assert_eq!(problems.problems.as_slice(), &[Problem { path: seed_path().member(b"policy"), what: What::Bound }]);
}

#[test]
fn a_seed_policy_number_must_be_a_u32() {
    for number in [b"-1".as_slice(), b"1.5", b"4294967296"] {
        let project = fixtures::with(br#""projects": [{"number":1,"policy":N}]"#, b"N", number);
        let problems = fixtures::engine(&document(&project)).unwrap_err();
        assert_eq!(problems.problems.as_slice(), &[Problem { path: seed_path().member(b"policy"), what: What::Bound }]);
    }
}
