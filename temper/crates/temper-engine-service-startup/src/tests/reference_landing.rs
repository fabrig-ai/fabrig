//! Reference landing settings (temper/docs/design/protocol/configuration/reference.md).
#![expect(clippy::disallowed_macros, reason = "the reference settings are compiled into their reader regression")]
use super::fixtures;
use crate::startup::{Problems, application, lines};
use jig_protocol_config::{self as config, read};
use settings_form::{
    file,
    view::{Setting, SettingEdit, SettingValue},
};
use temper_engine_domain::{RepositoryChecks, TierCheck};
use temper_engine_domain_forge::{Follows, GateLast, settings};
use temper_engine_domain_forge_tiers::Format;

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
fn the_reference_landing_member_parses() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    let input = fixtures::with(
        &fixtures::document(),
        b"application.landing",
        include_bytes!("../../../../docs/examples/reference/landing.json"),
    );
    let document = config::document(&input, &DOCUMENT).expect("startup document");
    let at = read::Object::new(&document, 0).expect("document object").member(&document, b"application").unwrap();
    let mut jig = fixtures::engine().jig;
    jig.limits.core.tasks.spec_bytes = 16_384;
    jig.limits.core.tasks.result_bytes = 4_096;
    let mut problems = Problems::empty();
    let members = application::parse(&document, at, &jig, &config::Directories::working(), &mut problems);
    assert_eq!(lines(&problems).as_ref(), b"", "reference landing accepted");
    let members = members.expect("application members");
    assert_eq!(members.landings.len(), 1);
    assert_eq!(
        members.landings.get(&1).expect("reference project landing").merge_job.as_deref(),
        Some(b".temper/merge".as_slice())
    );
    let landing = members.landings.get(&1).expect("reference project landing");
    assert_eq!(landing.close_job.as_deref(), Some(b".temper/close".as_slice()));
    assert_eq!(
        landing.repositories.as_ref(),
        &[RepositoryChecks {
            repository: skein_forge::api::Repository { forge: 1, repository: 1 },
            gates: Box::new([]),
            tiers: Box::new([
                TierCheck {
                    name: b"slow".as_slice().into(),
                    workflow: b"tier-slow.yml".as_slice().into(),
                    ignored: Box::new([]),
                    artifact: b"failing".as_slice().into(),
                    format: Format::Lines,
                    follows: Follows::Landing,
                    span_ms: 0,
                    description: b"the slow suites".as_slice().into(),
                },
                TierCheck {
                    name: b"goal".as_slice().into(),
                    workflow: b"tier-goal.yml".as_slice().into(),
                    ignored: Box::new([]),
                    artifact: b"failing".as_slice().into(),
                    format: Format::Lines,
                    follows: Follows::Goals,
                    span_ms: 1_200_000,
                    description: b"the gate without its slowest suites".as_slice().into(),
                },
            ]),
        }]
    );
}

#[test]
fn project_batch_defaults_and_bounds_name_its_member() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    for (member, cap, expected, reason) in [
        ("", 2_u32, Some(1), ""),
        (",\"batch\":2", 2, Some(2), ""),
        (",\"batch\":0", 2, None, ""),
        (",\"batch\":3", 2, None, "forge.landing_members"),
        (",\"batch\":3", 3, Some(3), ""),
        (",\"batch\":65536", 2, None, "forge.landing_members"),
    ] {
        let input = fixtures::with(
            &fixtures::document(),
            b"application.landing",
            format!("{{\"projects\":[{{\"project\":1{member}}}]}}").as_bytes(),
        );
        let input = fixtures::with(
            &input,
            b"application.limits.overrides",
            format!("{{\"forge.landing_members\":{cap},\"forge.resources_per_task\":4}}").as_bytes(),
        );
        let document = config::document(&input, &DOCUMENT).unwrap();
        let at = read::Object::new(&document, 0).unwrap().member(&document, b"application").unwrap();
        let jig = fixtures::engine().jig;
        let mut problems = Problems::empty();
        let members = application::parse(&document, at, &jig, &config::Directories::working(), &mut problems);
        let words = lines(&problems);
        if let Some(batch) = expected {
            assert_eq!(words.as_ref(), b"");
            assert_eq!(members.unwrap().landings.get(&1).unwrap().batch, batch);
        } else {
            assert!(skein_lib::bytes::find(&words, b"application.landing.projects[0].batch").is_some(), "{words:?}");
            assert!(skein_lib::bytes::find(&words, reason.as_bytes()).is_some(), "{words:?}");
        }
    }
}

#[test]
fn turn_without_prepare_is_refused_with_needed_and_configured_limits() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    for (by, spec, result, refused) in [
        ("queue", 4096, 512, false),
        ("turn", 4096, 512, true),
        ("turn", 16383, 4096, true),
        ("turn", 16384, 4095, true),
        ("turn", 16384, 4096, false),
    ] {
        let input = fixtures::with(
            &fixtures::document(),
            b"application.landing",
            format!("{{\"projects\":[{{\"project\":1,\"by\":\"{by}\"}}]}}").as_bytes(),
        );
        let document = config::document(&input, &DOCUMENT).unwrap();
        let at = read::Object::new(&document, 0).unwrap().member(&document, b"application").unwrap();
        let mut jig = fixtures::engine().jig;
        jig.limits.core.tasks.spec_bytes = spec;
        jig.limits.core.tasks.result_bytes = result;
        let mut problems = Problems::empty();
        let _members = application::parse(&document, at, &jig, &config::Directories::working(), &mut problems);
        let words = lines(&problems);
        assert_eq!(!words.is_empty(), refused, "{by}: {spec}/{result}: {words:?}");
        for (limit, has, needs) in [("tasks.spec_bytes", spec, 16384), ("tasks.result_bytes", result, 4096)] {
            if by == "turn" && has < needs {
                let has = skein_lib::Decimal::of(u64::from(has));
                let needs = skein_lib::Decimal::of(u64::from(needs));
                for expected in [limit.as_bytes(), has.as_bytes(), needs.as_bytes(), b"prepare job".as_slice()] {
                    assert!(skein_lib::bytes::find(&words, expected).is_some(), "missing {expected:?}: {words:?}");
                }
            }
        }
    }
}

#[test]
#[expect(clippy::disallowed_types, reason = "JSON fixture alternatives include an empty optional member")]
fn gate_reports_are_optional_and_bound_artifacts_and_fuzzy_prefixes() {
    const DOCUMENT: config::DocumentLimits = config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 };
    for (report, refused) in [
        (String::new(), None),
        (r#", "report":{"artifact":"results","format":"junit"}"#.into(), None),
        (r#", "report":{"artifact":"results","format":"lines","fuzzy":"f"}"#.into(), None),
        (
            format!(r#", "report":{{"artifact":"{}","format":"lines","fuzzy":"{}"}}"#, "a".repeat(32), "f".repeat(64)),
            None,
        ),
        (r#", "report":{"artifact":"","format":"junit"}"#.into(), Some("artifact")),
        (format!(r#", "report":{{"artifact":"{}","format":"junit"}}"#, "a".repeat(33)), Some("artifact")),
        (r#", "report":{"artifact":"a/b","format":"junit"}"#.into(), Some("artifact")),
        (r#", "report":{"artifact":"results","format":"other"}"#.into(), Some("format")),
        (r#", "report":{"artifact":"results","format":"junit","fuzzy":""}"#.into(), Some("fuzzy")),
        (
            format!(r#", "report":{{"artifact":"results","format":"junit","fuzzy":"{}"}}"#, "f".repeat(65)),
            Some("fuzzy"),
        ),
    ] {
        let input = fixtures::with(&fixtures::document(), b"application.landing", format!(
            r#"{{"projects":[{{"project":1,"gates":[{{"forge":1,"repository":1,"gates":[{{"name":"gate","workflow":"gate.yml","blocking":true{report}}}]}}]}}]}}"#
        ).as_bytes());
        let document = config::document(&input, &DOCUMENT).unwrap();
        let at = read::Object::new(&document, 0).unwrap().member(&document, b"application").unwrap();
        let mut problems = Problems::empty();
        let members =
            application::parse(&document, at, &fixtures::engine().jig, &config::Directories::working(), &mut problems);
        let words = lines(&problems);
        if let Some(member) = refused {
            let path = format!("application.landing.projects[0].gates[0].gates[0].report.{member}");
            assert!(skein_lib::bytes::find(&words, path.as_bytes()).is_some(), "{report}: {words:?}");
        } else {
            assert!(words.is_empty(), "{report}: {words:?}");
            let members = members.unwrap();
            let gate = &members.landings.get(&1).unwrap().repositories[0].gates[0];
            if report.is_empty() {
                assert_eq!(gate.report, None);
            } else {
                let parsed = gate.report.as_ref().unwrap();
                assert_eq!(parsed.format, if report.contains("junit") { Format::Junit } else { Format::Lines });
                assert_eq!(parsed.fuzzy.is_some(), report.contains("fuzzy"));
                assert!(!parsed.artifact.is_empty());
            }
        }
    }
}
