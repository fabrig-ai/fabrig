use super::{defaults, fixtures};
use crate::{Defaults, Path, Problem, Problems, What, llm};
use skein_json::writer::{Encoder, Limits};
const EXAMPLE: &[u8] = br#"{"number":4,"name":"local","endpoint":"local","signed_in_plan":false,"sources":{},"endpoints":[{"name":"local","number":5,"dialect":3,"address":{"socket":"127.0.0.1:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}"#;
fn read(bytes: &[u8]) -> llm::ProviderSetting {
    let document = crate::document(bytes, &crate::DocumentLimits { bytes: 8192, tokens: 512, depth: 16 }).unwrap();
    let mut problems = Problems::empty();
    let value = llm::provider_setting(&document, 0, &Path::root(), &mut problems).unwrap();
    assert!(problems.is_empty(), "{problems:?}");
    value
}
fn render(value: &llm::ProviderSetting, list: bool) -> Box<[u8]> {
    let limits = Limits { depth: 16, length: 8192 };
    let mut measured = Encoder::measure(&limits);
    if list {
        llm::render_providers(core::slice::from_ref(value), &mut measured);
    } else {
        llm::render_provider(value, &mut measured);
    }
    let mut writer = Encoder::write(measured.measured().unwrap(), &limits);
    if list {
        llm::render_providers(core::slice::from_ref(value), &mut writer);
    } else {
        llm::render_provider(value, &mut writer);
    }
    writer.finish()
}
#[test]
#[expect(clippy::disallowed_methods, reason = "ordinary bounded JSON fixture assembly and diagnostic assertions")]
fn provider_example_is_the_json_golden_and_lists_round_trip() {
    let value = read(EXAMPLE);
    assert_eq!(value.provider.number, 4);
    assert_eq!(value.provider.name.as_ref(), b"local");
    assert_eq!(value.endpoints.first().unwrap().number, 5);
    assert_eq!(render(&value, false).as_ref(), EXAMPLE);
    assert_eq!(read(&render(&value, false)), value);
    let bytes = render(&value, true);
    let document = crate::document(&bytes, &crate::DocumentLimits { bytes: 8192, tokens: 512, depth: 16 }).unwrap();
    let mut problems = Problems::empty();
    assert_eq!(llm::providers_setting(&document, 0, &Path::root(), &mut problems).unwrap().as_ref(), &[value]);
    assert!(problems.is_empty());
    for header in [b"Authorization".as_slice(), b"authorization", b"COOKIE"] {
        let header = [br#""headers":[{"name":""#.as_slice(), header, br#"","value":"not-a-credential"}]"#].concat();
        let bytes = fixtures::with(EXAMPLE, br#""headers":[]"#, &header);
        let document = crate::document(&bytes, &crate::DocumentLimits { bytes: 8192, tokens: 512, depth: 16 }).unwrap();
        let mut problems = Problems::empty();
        let _value = llm::provider_setting(&document, 0, &Path::root(), &mut problems);
        assert!(!problems.is_empty());
    }
    let secret = fixtures::with(EXAMPLE, br#""sources":{}"#, br#""sources":{"sign_in":{"secret":"not-a-credential"}}"#);
    let document = crate::document(&secret, &crate::DocumentLimits { bytes: 8192, tokens: 512, depth: 16 }).unwrap();
    let mut problems = Problems::empty();
    assert!(llm::provider_setting(&document, 0, &Path::root(), &mut problems).is_none());
    assert!(problems.problems.iter().any(|problem| problem.what == What::Confidential));
}
#[test]
fn document_providers_are_defaults_but_passed_providers_refuse_llm() {
    let document = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
    let core = crate::core(&document, [3; 16], 11);
    assert_eq!(core.defaults.providers.len(), 1);
    assert_eq!(core.defaults.providers.first().unwrap().endpoints, document.llm.endpoints);
    assert!(core.settings.llm_providers.get(1).is_some());
    let bytes = defaults::without_defaults(fixtures::ENGINE_SMALL);
    let problems = defaults::parse(&bytes, Defaults::PassedProviders).unwrap_err();
    assert_eq!(problems.problems.get(0), Some(&Problem { path: Path::root().member(b"llm"), what: What::Stored }));
    let start = skein_lib::bytes::find(&bytes, b"\"llm\":").unwrap();
    let end = skein_lib::bytes::find(&bytes, b"\"hosts\":").unwrap();
    let bytes = fixtures::with(&bytes, &bytes[start..end], b"");
    let mut engine = defaults::parse(&bytes, Defaults::PassedProviders).unwrap();
    let mut fixed = read(EXAMPLE);
    fixed.provider.number = 3;
    let defaults = crate::core(&document, [3; 16], 11).defaults;
    crate::pass_providers(&mut engine, defaults, &llm::provider_values(&document.llm), Box::new([fixed.clone()]))
        .unwrap();
    let core = crate::core(&engine, [3; 16], 11);
    assert_eq!(core.defaults.providers.len(), 1);
    assert_eq!(core.fixed_providers.as_ref(), &[fixed]);
    assert!(core.settings.llm_providers.get(3).is_some());
    assert_eq!(core.settings.endpoints.len(), 2);
    let mut old_mode = defaults::parse(&defaults::without_defaults(fixtures::ENGINE_SMALL), Defaults::Passed).unwrap();
    crate::pass(&mut old_mode, crate::core(&document, [3; 16], 11).defaults).unwrap();
    assert_eq!(old_mode.llm.endpoints, document.llm.endpoints);
}

#[expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "ordinary bounded JSON list fixtures")]
fn list_case(kind: u8, count: usize) -> Box<[u8]> {
    let items = if kind == 0 {
        vec![r#"{"name":"X-Client","value":"jig"}"#.to_string(); count]
    } else if kind == 1 {
        (0..count).map(|number| format!("\"p{number}\"")).collect()
    } else {
        vec!["\"chat\"".to_string(); count]
    };
    let list = format!("[{}]", items.join(","));
    if kind == 0 {
        return fixtures::with(EXAMPLE, br#""headers":[]"#, format!("\"headers\":{list}").as_bytes());
    }
    if kind == 1 {
        return fixtures::with(
            EXAMPLE,
            br#""sources":{}"#,
            format!("\"manage_usage\":\"https://usage.example/\",\"sources\":{{\"programs\":{list}}}").as_bytes(),
        );
    }
    let scopes = if kind == 2 { list.as_str() } else { "[\"chat\"]" };
    let grants = if kind == 3 { list.as_str() } else { "[\"chat\"]" };
    let registration = format!(
        r#""sources":{{"sign_in":{{"issuer":"https://issuer.example","client":"public-client","scopes":{scopes},"authorization":"https://issuer.example/authorize","token":"https://issuer.example/token","keys":"https://issuer.example/keys","transport":"plaintext","must_grant":{grants},"redirect_path":"callback"}}}}"#
    );
    fixtures::with(EXAMPLE, br#""sources":{}"#, registration.as_bytes())
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "ordinary bounded list mutation proves both vocabulary and parser admission"
)]
fn provider_list_bounds_refuse_the_first_excess_item() {
    use jig_domain_core::settings::{Failure, Fault, Setting, Value, check};
    for (kind, bound, path) in [
        (0, 16, Path::root().member(b"endpoints").child(crate::Segment::Index(0)).member(b"headers")),
        (1, 64, Path::root().member(b"sources").member(b"programs")),
        (2, 64, Path::root().member(b"sources").member(b"sign_in").member(b"scopes")),
        (3, 64, Path::root().member(b"sources").member(b"sign_in").member(b"must_grant")),
    ] {
        let mut value = read(&list_case(kind, bound));
        match kind {
            0 => {
                let endpoint = value.endpoints.first_mut().unwrap();
                let mut values = endpoint.headers.to_vec();
                values.push(values[0].clone());
                endpoint.headers = values.into_boxed_slice();
            }
            1 => {
                let mut values = value.provider.programs.to_vec();
                values.push(b"past-the-bound".as_slice().into());
                value.provider.programs = values.into_boxed_slice();
            }
            2 => {
                let registration = value.provider.sign_in.as_mut().unwrap();
                let mut values = registration.scopes.to_vec();
                values.push(b"chat".as_slice().into());
                registration.scopes = values.into_boxed_slice();
            }
            3 => {
                let registration = value.provider.sign_in.as_mut().unwrap();
                let mut values = registration.must_grant.to_vec();
                values.push(b"chat".as_slice().into());
                registration.must_grant = values.into_boxed_slice();
            }
            _ => unreachable!("four list kinds"),
        }
        let engine = fixtures::engine(fixtures::ENGINE_SMALL).unwrap();
        assert_eq!(
            check::shape(Setting::Provider(4), &Value::Provider(Box::new(value)), &engine.limits.core),
            Err(Failure { setting: Setting::Provider(4), fault: Fault::Shape, detail: None })
        );
        let bytes = list_case(kind, bound + 1);
        let document =
            crate::document(&bytes, &crate::DocumentLimits { bytes: 16384, tokens: 4096, depth: 16 }).unwrap();
        let mut problems = Problems::empty();
        let _value = llm::provider_setting(&document, 0, &Path::root(), &mut problems);
        assert_eq!(
            problems.problems.get(0),
            Some(&Problem { path: path.child(crate::Segment::Index(u32::try_from(bound).unwrap())), what: What::Form })
        );
    }
}
