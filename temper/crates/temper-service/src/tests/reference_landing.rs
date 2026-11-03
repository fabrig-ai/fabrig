//! Reference landing settings (temper/docs/design/protocol/configuration.md, 13.9).
#![expect(clippy::disallowed_macros, reason = "the reference settings are compiled into their reader regression")]
use super::fixtures;
use crate::startup::{Problems, application, lines};
use jig_protocol_config::{self as config, read};
use settings_form::{
    file,
    view::{Setting, SettingEdit, SettingValue},
};
use temper_engine_domain_forge::{GateLast, settings};

#[test]
fn the_reference_landing_settings_decode() {
    // configuration.md, 13.9: main and goal branches.
    const RULES: usize = 2;
    let items = file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("reference settings file");
    let mut rules = 0;
    for item in &items {
        if let Setting::Application { kind, scope, number } = item.setting {
            assert_eq!((kind, scope), (1, 0));
            let SettingEdit::Set(set) = &item.edit else { panic!("set rule") };
            let SettingValue::Application(bytes) = &set.value else { panic!("application setting") };
            let value = temper_engine_smith::kinds::decode(
                kind,
                bytes,
                &temper_engine_smith::kinds::Bounds { segments: 8, segment_bytes: 64, gates: 4, approvals: 4 },
            )
            .expect("landing rule");
            let settings::Value::Rule(rule) = value.value else { panic!("rule") };
            let (last, checks) = match number {
                1 => (GateLast::Exact(b"main".as_slice().into()), true),
                2 => (GateLast::Open(b"temper".as_slice().into()), false),
                _ => panic!("reference landing rule number"),
            };
            assert_eq!(
                rule.pattern.segments.as_ref(),
                [b"forge".as_slice(), b"127.0.0.1:3000", b"acme", b"widget", b"branch"].map(Box::from)
            );
            assert_eq!(rule.pattern.last, last);
            assert!(rule.enforce);
            assert_eq!((rule.ci, rule.up_to_date), (checks, checks));
            assert!(rule.gates.is_empty());
            assert!(rule.approvals.is_empty());
            rules += 1;
        }
    }
    assert_eq!(rules, RULES);
}

#[test]
fn the_reference_landing_member_parses() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    let input = fixtures::with(
        &fixtures::document(),
        b"application.landing",
        include_bytes!("../../../../docs/examples/reference/landing.json"),
    );
    let document = config::document(&input, &DOCUMENT).expect("startup document");
    let at = read::Object::new(&document, 0).expect("document object").member(&document, b"application").unwrap();
    let jig = fixtures::engine().jig;
    let mut problems = Problems::empty();
    let members = application::parse(&document, at, &jig, &config::Directories::working(), &mut problems);
    assert_eq!(lines(&problems).as_ref(), b"", "reference landing accepted");
    let members = members.expect("application members");
    assert_eq!(members.landings.len(), 1);
    assert_eq!(
        members.landings.get(&1).expect("reference project landing").merge_job.as_deref(),
        Some(b".temper/merge".as_slice())
    );
}
