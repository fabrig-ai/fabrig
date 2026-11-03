//! Reference landing settings (temper/docs/design/protocol/configuration.md, 13.9).
#![expect(clippy::disallowed_macros, reason = "the reference settings are compiled into their reader regression")]
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
