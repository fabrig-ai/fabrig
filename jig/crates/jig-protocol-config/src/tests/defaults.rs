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
    document = fixtures::with(&document, br#""models":["small"]"#, br#""endpoint":"fake""#);
    let first = bytes::find(&document, b",\n\"models\":").unwrap();
    let after = bytes::find(&document, b"\n},\n\"hosts\":").unwrap();
    fixtures::with(&document, &document[first..after], b"")
}
fn defaults() -> jig_domain_core_vocabulary::settings::Defaults {
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
    assert_eq!(
        found.problems.get(0).unwrap().what,
        What::Because(b"policy authority or period spend exceeds deployment rules")
    );
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

#[test]
fn passed_models_are_the_catalogue() {
    use jig_domain_core_vocabulary::settings::ModelSource;
    let mut engine = passed();
    assert!(engine.llm.models.is_empty(), "passed catalogues have no document models");
    let mut given = defaults();
    let entry = &mut given.models[0];
    entry.number = 64;
    entry.model.name = Box::from(b"passed".as_slice());
    entry.model.output = 2048;
    entry.shown = Some(Box::from(b"Passed".as_slice()));
    entry.description = Some(Box::from(b"Application catalogue".as_slice()));
    entry.picker = false;
    entry.withdrawn = true;
    entry.source = ModelSource::Person;
    entry.estimated = true;
    given.charters[0].policy.model = entry.model.clone();
    let expected = given.models.clone();
    crate::pass(&mut engine, given).expect("passed catalogue supplies the model defaults");
    assert_eq!(engine.llm.models, expected);
    let config = crate::core(&engine, [3; 16], 11);
    assert_eq!(config.defaults.models, expected);
    assert_eq!(config.settings.models.len(), 64);
    for model in &config.settings.models[..63] {
        assert!(model.is_hole());
    }
    assert_eq!(config.settings.models[63], expected[0].model);
    let models = crate::clients::models(&engine.llm);
    assert_eq!(models.len(), 64);
    assert_eq!(models[63].name, expected[0].model.name);
    assert_eq!(models[63].provider, Some(expected[0].model.provider));
    assert_eq!(models[63].shown, expected[0].shown);
    assert_eq!(models[63].description, expected[0].description);
    assert!(!models[63].picker);
}

#[test]
fn a_documents_models_are_numbered_by_position() {
    use jig_domain_core_vocabulary::settings::ModelSource;
    let bytes = fixtures::with(
        fixtures::ENGINE_SMALL,
        br#""unit": 1000}
}]"#,
        br#""unit": 1000}, "shown":"Small", "picker":false, "description":"First choice"
}, {"name":"later","description":"Later choice","endpoint":"fake","window":8192,"output":1024,
"prices":{"input":1,"cached":1,"output":1,"unit":1000}}]"#,
    );
    let bytes = fixtures::with(&bytes, br#""models":["small"]"#, br#""models":["small","later"]"#);
    let engine = fixtures::engine(&bytes).unwrap();
    let config = crate::core(&engine, [3; 16], 11);
    assert_eq!(config.defaults.models, engine.llm.models);
    assert_eq!(config.settings.models.as_ref(), engine.llm.models().as_ref());
    let rows = &config.defaults.models;
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row.number, u32::try_from(index).unwrap() + 1);
        assert_eq!(row.source, ModelSource::Stock);
        assert!(!row.withdrawn);
        assert!(!row.estimated);
    }
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].shown.as_deref(), Some(b"Small".as_slice()));
    assert_eq!(rows[0].description.as_deref(), Some(b"First choice".as_slice()));
    assert!(!rows[0].picker);
    assert_eq!(rows[1].shown, None);
    assert_eq!(rows[1].description.as_deref(), Some(b"Later choice".as_slice()));
    assert!(rows[1].picker);
}

#[test]
fn a_charter_with_no_models_takes_the_rest() {
    for members in [b"".as_slice(), br#""models":["small"],"rest":true,"#] {
        let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#""model": "small", "alternatives": [],"#, members);
        let engine = fixtures::engine(&bytes).expect("charter's remaining models come from the catalogue");
        assert!(engine.charters[0].rest);
        assert!(crate::core(&engine, [3; 16], 11).defaults.charters[0].rest);
    }
    let bytes =
        fixtures::with(fixtures::ENGINE_SMALL, br#""model": "small", "alternatives": [],"#, br#""models":["small"],"#);
    assert!(!fixtures::engine(&bytes).unwrap().charters[0].rest);
}

#[test]
#[expect(clippy::disallowed_macros, reason = "tests are ordinary Rust (programming-model.md, 10.2)")]
fn a_resolved_charter_of_64_models_fits_run_bytes() {
    let engine = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
    let mut charter = engine.charters[0].clone();
    charter.policy.instructions = Box::new([b'i'; 8192]);
    let mut models = skein_lib::List::with_capacity(64);
    for number in 0_u32..64 {
        let mut model = engine.llm.models[0].model.clone();
        model.name = format!("model-{number:03}").into_bytes().into_boxed_slice();
        model.efforts =
            Box::new([Box::from(b"low".as_slice()), Box::from(b"medium".as_slice()), Box::from(b"high".as_slice())]);
        model.effort = Some(Box::from(b"high".as_slice()));
        models.push(model).unwrap();
    }
    let models = models.into_boxed();
    charter.policy.model = models[0].clone();
    charter.policy.alternatives = models[1..].into();
    let bytes = jig_domain_core_vocabulary::policy_owned_bytes(&charter.policy).unwrap();
    let standard = crate::profile(crate::Profile::Standard);
    assert_eq!(standard.core.settings.alternatives, 63);
    assert_eq!(crate::profile(crate::Profile::Small).core.settings.alternatives, 63);
    assert_eq!(jig_client_protocol_wire::v21::CEILINGS.charter_alternatives, 63);
    assert!(bytes <= u64::from(standard.core.run_bytes), "64-model resolved policy owns {bytes} bytes");
    assert!(
        bytes > u64::from(crate::profile(crate::Profile::Small).core.run_bytes),
        "small leaves models out when its byte bound is reached"
    );
}
