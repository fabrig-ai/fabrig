//! Reference landing settings (temper/docs/design/protocol/configuration/reference.md).
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
    // protocol/configuration/reference.md: main and goal branches.
    const RULES: usize = 2;
    let items = file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("reference settings file");
    let mut rules = 0;
    for item in &items {
        if let Setting::Application { kind, scope, number } = item.setting {
            if (kind, scope, number) == (11, 0, 4) {
                continue;
            }
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
fn the_reference_landing_settings_are_project_kinds() {
    let items = file::decode(
        include_bytes!("../../../../docs/examples/reference/landing.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("landing settings file");
    assert_eq!(items.len(), 2);
    for (item, kind) in items.iter().zip([4, 5]) {
        assert_eq!(item.setting, Setting::Application { kind, scope: 1, number: 1 });
        let SettingEdit::Set(set) = &item.edit else { panic!("set") };
        let SettingValue::Application(bytes) = &set.value else { panic!("application value") };
        let value = temper_engine_smith::kinds::decode(
            kind,
            bytes,
            &temper_engine_smith::kinds::Bounds { segments: 8, segment_bytes: 64, gates: 4, approvals: 4 },
        )
        .expect("landing record");
        if let settings::Value::Landing(v) = value.value {
            assert_eq!(v.merge_job.as_deref(), Some(b".temper/merge".as_slice()));
            assert_eq!(v.close_job.as_deref(), Some(b".temper/close".as_slice()));
        } else if let settings::Value::Gates(v) = value.value {
            assert_eq!(v.repositories.len(), 1);
            assert_eq!(v.repositories[0].tiers.len(), 2);
            assert_eq!(v.repositories[0].tiers[1].follows, temper_engine_domain_forge::Follows::Goals);
        } else {
            panic!("kind 4 or 5");
        }
    }
}
#[test]
fn application_landing_is_refused_whole_with_its_moved_words() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    for value in [b"{}".as_slice(), b"null", br#"{"projects":[]}"#] {
        let parts = [
            b"{\"landing\":".as_slice(),
            value,
            b",",
            fixtures::APPLICATION.strip_prefix(b"{").expect("application object"),
        ];
        let mut size = 0_usize;
        for part in parts {
            size = size.checked_add(part.len()).expect("fixture input size");
        }
        let mut app = skein_lib::Writer::new(size);
        for part in parts {
            app.put(part).expect("fixture application");
        }
        let input = fixtures::with(&fixtures::document(), b"application", &app.finish());
        let document = config::document(&input, &DOCUMENT).expect("startup document");
        let at =
            read::Object::new(&document, 0).expect("object").member(&document, b"application").expect("application");
        let mut problems = Problems::empty();
        assert!(
            application::parse(&document, at, &fixtures::engine().jig, &config::Directories::working(), &mut problems)
                .is_none()
        );
        let words = lines(&problems);
        assert!(skein_lib::bytes::find(&words, b"application.landing: moved: a project's landing is settings, project-landing and project-gates; set them with temper ctl settings apply").is_some(), "{words:?}");
    }
}
