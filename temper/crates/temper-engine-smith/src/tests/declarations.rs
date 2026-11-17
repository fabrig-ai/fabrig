//! Declared forms and role shares (domain/agent/declarations.md, 2 to 4).
#![expect(clippy::disallowed_macros, reason = "schema projection diagnostics carry the nested path")]
#![expect(clippy::disallowed_methods, reason = "step tests read schema names as UTF-8 diagnostics")]

use super::{ASK_PROPOSE_SETTINGS, PLAN_CHANGE, SPECS, Spec};
use crate::json::{self, Value};
use skein_lib::{Duration, List};

fn declarations() -> Box<[Spec]> {
    let mut specs = List::uncapped();
    for spec in SPECS {
        assert!(specs.push(spec).is_ok());
    }
    for spec in [PLAN_CHANGE, ASK_PROPOSE_SETTINGS] {
        assert!(specs.push(spec).is_ok());
    }
    specs.into_boxed()
}

fn bytes(spec: &Spec) -> usize {
    spec.name.len().checked_add(spec.description.len()).unwrap().checked_add(spec.schema.len()).unwrap()
}

#[test]
fn every_declaration_is_within_its_bound() {
    for spec in declarations() {
        let name = core::str::from_utf8(spec.name).unwrap();
        assert!(bytes(&spec) <= 1024, "{name}: {} bytes, bound 1024", bytes(&spec));
    }
}

#[test]
fn stock_roles_non_forge_shares_are_within_their_bounds() {
    let goal = [
        "delegate",
        "goal",
        "change",
        "finding",
        "red",
        "follow",
        "message",
        "introduce",
        "decided",
        "amend",
        "cancel",
        "release",
        "subscribe",
        "unsubscribe",
        "subscribe_forge",
        "note",
        "recall",
        "read_task",
        "read_docs",
    ];
    let coder = ["message", "introduce", "read_task", "read_docs"];
    let families = jig_domain_core::ToolFamilies::JIG;
    let authority = jig_domain_tasks::Tools(!families.decide.0 & !families.propose.0 & !families.effect.0);
    let charter = jig_domain_tasks::Tools(!families.settings & !families.ask.0);
    let tools = super::tools_for(Duration::from_secs(1), charter, authority);
    for (names, bound) in [(goal.as_slice(), 11_648), (coder.as_slice(), 1_408)] {
        let mut count = 0_usize;
        let mut sum = 0_usize;
        for tool in &tools {
            if names.contains(&core::str::from_utf8(&tool.name).unwrap()) {
                count = count.checked_add(1).unwrap();
                for bytes in [tool.name.len(), tool.description.len(), tool.schema.len()] {
                    sum = sum.checked_add(bytes).unwrap();
                }
            }
        }
        assert_eq!(count, names.len());
        assert!(sum <= bound, "{} tools: {sum} bytes, bound {bound}", names.len());
    }
}

#[test]
fn every_declared_form_projects_its_whole_form() {
    for spec in declarations() {
        let name = core::str::from_utf8(spec.name).unwrap();
        let declared = json::parse(spec.schema).unwrap();
        let whole = json::parse(spec.whole()).unwrap();
        projection(&declared, &whole, name);
        if let Some(required) = whole.field(b"required").unwrap() {
            for member in required.array(b"required").unwrap() {
                let member = member.text(b"required").unwrap();
                assert!(
                    field(declared.required(b"properties").unwrap(), &member).is_some(),
                    "{name}: required root member {member:?}"
                );
            }
        }
    }
}

fn projection(declared: &Value, whole: &Value, path: &str) {
    for facet in [b"type".as_slice(), b"enum"] {
        if let Some(value) = declared.field(facet).unwrap() {
            assert_eq!(Some(value), whole.field(facet).unwrap(), "{path}: {facet:?}");
        }
    }
    let properties = match declared.field(b"properties").unwrap() {
        Some(Value::Object(properties)) => properties.as_ref(),
        None => &[],
        _ => panic!("schema properties"),
    };
    if let Some(Value::Object(whole_properties)) = whole.field(b"properties").unwrap() {
        for (name, _) in whole_properties {
            if !properties.iter().any(|(key, _)| key == name) {
                assert_ne!(
                    declared.field(b"additionalProperties").unwrap(),
                    Some(&Value::Boolean(false)),
                    "{path}: omitted member must stay open"
                );
            }
        }
    }
    for (name, member) in properties {
        let whole_member = field(whole.required(b"properties").unwrap(), name).unwrap();
        projection(member, whole_member, &format!("{path}.{}", core::str::from_utf8(name).unwrap()));
    }
    if let Some(items) = declared.field(b"items").unwrap() {
        projection(items, whole.required(b"items").unwrap(), &format!("{path}[]"));
    }
    for alternative in [b"oneOf".as_slice(), b"anyOf", b"allOf"] {
        if let Some(variants) = declared.field(alternative).unwrap() {
            let variants = variants.array(alternative).unwrap();
            let whole_variants = whole.required(alternative).unwrap().array(alternative).unwrap();
            assert_eq!(variants.len(), whole_variants.len(), "{path}: alternatives");
            for (variant, whole_variant) in variants.iter().zip(whole_variants) {
                projection(variant, whole_variant, path);
            }
        }
    }
}

#[test]
#[expect(clippy::type_complexity, reason = "each fixture pairs its declaration, omitted member path and whole call")]
fn shortened_forms_still_decode_omitted_members_under_every_contract() {
    let proposal = SPECS.iter().find(|spec| spec.name == b"propose_settings").unwrap();
    let delegate = SPECS.iter().find(|spec| spec.name == b"delegate").unwrap();
    let amend = SPECS.iter().find(|spec| spec.name == b"amend").unwrap();
    let settings = SPECS.iter().find(|spec| spec.name == b"settings").unwrap();
    let change = SPECS.iter().find(|spec| spec.name == b"change").unwrap();
    let cases: &[(&Spec, &[&[u8]], &[u8])] = &[
        (delegate, &[b"batch", b"[]", b"authority", b"budget", b"deadline"], br#"{"batch":[{"executor":{"kind":"agent","charter":1},"spec":{"words":"x"},"contract":{"kind":"report","words":128},"authority":{"tools":1,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":40,"deadline":20000000000},"notes":0}}]}"#),
        (amend, &[b"amendment", b"authority", b"budget", b"deadline"], br#"{"target":3,"revision":1,"amendment":{"reason":"Friday","authority":{"tools":5,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":1000,"deadline":1793404800000000000},"notes":0}}}"#),
        (change, &[b"questions"], br#"{"words":"x","questions":"up"}"#),
        (&PLAN_CHANGE, &[b"changes", b"[]", b"branch"], br#"{"changes":[{"words":"x","branch":"mine","producer":2,"questions":"up"}]}"#),
        (settings, &[b"setting", b"kind"], br#"{"setting":{"kind":"charter","number":2}}"#),
        (proposal, &[b"note"], br#"{"against":2,"reason":"whole","note":{"name":8,"revision":3}}"#),
        (&ASK_PROPOSE_SETTINGS, &[b"items", b"[]", b"setting", b"kind"], br#"{"against":1,"reason":"restore","items":[{"setting":{"kind":"charter","number":1},"edit":"reset"}],"more":true}"#),
    ];
    for &(spec, path, input) in cases {
        assert!(!has_member(&json::parse(spec.schema).unwrap(), path));
        assert!(has_member(&json::parse(spec.whole()).unwrap(), path));
        let input = smith_domain_run::HostInput::attested(input.into()).unwrap();
        let name = smith_domain_run::CallName { activation: 7, completion: 2, position: 1 };
        let crate::Decoded::Call(current) = crate::call(name, spec.name, &input).unwrap() else {
            panic!("whole call decoded")
        };
        for version in 1..=crate::wire::VERSION {
            let crate::Decoded::Call(kept) =
                crate::call_for_contract(version, name, spec.name, &input, &[], &[], &[]).unwrap()
            else {
                panic!("kept call decoded")
            };
            assert_eq!(kept, current, "{:?}: version {version} preserves whole input", spec.name);
        }
    }
}

fn has_member(schema: &Value, path: &[&[u8]]) -> bool {
    let Some((first, rest)) = path.split_first() else { return true };
    let next = if *first == b"[]" {
        schema.field(b"items").unwrap()
    } else {
        schema.field(b"properties").unwrap().and_then(|properties| field(properties, first))
    };
    next.is_some_and(|next| has_member(next, rest))
}

fn field<'a>(value: &'a Value, key: &[u8]) -> Option<&'a Value> {
    let Value::Object(fields) = value else { panic!("schema object") };
    fields.iter().find(|(name, _)| name.as_ref() == key).map(|(_, value)| value)
}
