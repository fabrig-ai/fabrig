//! Applications pass settings while seed project diagnostics keep document paths.
use super::fixtures;
use crate::{Defaults, Engine, Problem, What};
use skein_lib::bytes;

pub(super) fn parse(bytes: &[u8], mode: Defaults) -> Result<Engine, crate::Problems> {
    let document = crate::document(bytes, &crate::DocumentLimits { bytes: 8192, tokens: 1536, depth: 16 }).unwrap();
    crate::engine(&document, 0, &crate::Directories::working(), mode)
}
pub(super) fn without_defaults(bytes: &[u8]) -> Box<[u8]> {
    let mut document = Box::from(bytes);
    for (first, next) in [
        (b"\"charters\":".as_slice(), b"\"clients\":".as_slice()),
        (b"\"rules\":".as_slice(), b"\"projects\":".as_slice()),
    ] {
        let start = bytes::find(&document, first).unwrap();
        let end = bytes::find(&document, next).unwrap();
        document = fixtures::with(&document, &document[start..end], b"");
    }
    document
}
fn defaults() -> jig_domain_core::settings::Defaults {
    crate::core(&fixtures::engine(fixtures::ENGINE_SMALL).unwrap(), [3; 16], 11).defaults
}
fn passed() -> Engine {
    parse(&without_defaults(fixtures::ENGINE_SMALL), Defaults::Passed).unwrap()
}

#[test]
fn passed_defaults_refuse_each_member_naming_its_path() {
    for member in [b"charters".as_slice(), b"templates", b"policies", b"rules"] {
        let mut addition = skein_lib::Writer::new(member.len().checked_add(9).unwrap());
        addition.put(b"{\"").unwrap();
        addition.put(member).unwrap();
        addition.put(b"\":null,").unwrap();
        let addition = addition.finish();
        let document = fixtures::with(&without_defaults(fixtures::ENGINE_SMALL), b"{", &addition);
        let problems = parse(&document, Defaults::Passed).unwrap_err();
        assert_eq!(problems.problems.as_slice(), &[Problem { path: fixtures::path(&[member]), what: What::Stored }]);
        assert_eq!(problems.past, 0);
    }
}
#[test]
fn passed_defaults_need_none_of_the_four() {
    let mut engine = passed();
    assert!(engine.charters.is_empty());
    assert!(engine.templates.is_empty());
    assert!(engine.policies.is_empty());
    crate::pass(&mut engine, defaults()).unwrap();
    let expected = defaults();
    let actual = crate::core(&engine, [3; 16], 11).defaults;
    assert_eq!(actual.charters, expected.charters);
    assert_eq!(actual.templates, expected.templates);
    assert_eq!(actual.rules, expected.rules);
    assert_eq!(actual.policies.len(), expected.policies.len());
    for (number, policy) in &actual.policies {
        assert_eq!(Some(policy), expected.policies.get(number));
    }
}
#[test]
fn a_seed_override_naming_a_template_the_defaults_lack_is_refused_at_pass() {
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""templates": []"#,
        br#""templates": [{"template":2,"executor":null,"budget":null,"deadline":null,"priority":null}]"#,
    );
    let mut engine = parse(&without_defaults(&document), Defaults::Passed).unwrap();
    let mut given = defaults();
    given.templates = Box::new([given.templates[0].clone()]);
    let found = crate::pass(&mut engine, given).unwrap_err();
    let document = fixtures::with(&document, br#""template":2"#, br#""template":3"#);
    let expected = parse(&document, Defaults::Document).unwrap_err();
    assert_eq!(found.problems, expected.problems);
    assert_eq!(found.problems.get(0).unwrap().what, What::Refers);
}
#[test]
fn a_seed_policy_above_the_passed_rules_is_refused_at_pass() {
    let mut given = defaults();
    given.rules.period_spend = 99;
    let found = crate::pass(&mut passed(), given).unwrap_err();
    let document = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""period_spend": 100, "minimum_run_spend"#,
        br#""period_spend": 99, "minimum_run_spend"#,
    );
    let expected = parse(&document, Defaults::Document).unwrap_err();
    assert_eq!(found.problems, expected.problems);
    assert_eq!(found.problems.get(0).unwrap().what, What::Bound);
}
#[test]
fn files_name_no_charter_for_passed_defaults() {
    let mut engine = passed();
    crate::pass(&mut engine, defaults()).unwrap();
    let charter_path = fixtures::path(&[b"charters"]);
    for file in crate::files(&engine) {
        assert_ne!(file.at.segments.get(0), charter_path.segments.get(0));
    }
}

#[test]
fn pass_leaves_the_defaults_own_check_to_the_core() {
    let mut engine = passed();
    engine.limits.core.authority.grants = 0;
    let mut given = defaults();
    given.rules.ceiling.grants = Box::new([jig_domain_authority::Grant {
        connector: 1,
        kind: 1,
        pattern: jig_domain_authority::Pattern {
            segments: Box::default(),
            last: jig_domain_authority::Last::Open(Box::default()),
        },
    }]);
    assert!(!jig_domain_authority::valid_rules(&given.rules, &engine.limits.core.authority));
    crate::pass(&mut engine, given).expect("the seed fits the rules; the defaults' shape is the core's check");
}
