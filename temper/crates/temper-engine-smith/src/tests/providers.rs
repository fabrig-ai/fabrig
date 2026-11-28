//! The client JSON port agrees with jig's engine form (llm-providers.md, 6).
use jig_domain_core::settings as s;
use jig_protocol_config::{DocumentLimits, Path, Problems, llm};
use settings_form::{Limits, file, render, view as c, wire};
use skein_json::writer::{Encoder, Limits as JsonLimits};

const LOCAL: &str = r#"{"number":3,"name":"local","endpoint":"local","signed_in_plan":false,"sources":{},"endpoints":[{"name":"local","number":5,"dialect":3,"address":{"socket":"127.0.0.1:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}]}"#;
const FULL: &str = r#"{"number":4,"name":"local","endpoint":"program","direct":"direct","signed_in_plan":true,"manage_usage":"https://provider.example/manage","keep":["local-*"],"drop":["*-old"],"sources":{"keys":true,"programs":["local_cli"],"sign_in":{"issuer":"https://provider.example","client":"public-client","authorization":"https://provider.example/authorize","token":"https://provider.example/token","keys":"https://provider.example/keys","redirect_path":"callback","scopes":["openid","profile"],"must_grant":["openid"],"name_hint":"email","revocation":"https://provider.example/revoke","resource":"https://provider.example/resource","redirect_port":9123,"transport":"tls","server_name":"provider.example","trust":"/config/trust.pem"}},"endpoints":[{"name":"program","number":5,"dialect":3,"program":"local_cli","address":{"host":"provider.example","port":443},"transport":"tls","server_name":"provider.example","trust":"/config/trust.pem","headers":[{"name":"X-Client","value":"temper"}],"connect_ms":1234,"handshake_ms":5678},{"name":"direct","number":6,"dialect":3,"address":{"socket":"[::1]:8080"},"transport":"plaintext","headers":[],"connect_ms":10000,"handshake_ms":10000}],"withdrawn":true}"#;
fn port(bytes: &[u8]) -> c::SettingValue {
    let json: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let number = json["number"].as_u64().unwrap();
    let bytes = format!(r#"{{"items":[{{"setting":{{"kind":"provider","number":{number}}},"set":{json}}}]}}"#);
    let items = file::decode(bytes.as_bytes(), &Limits::standard()).unwrap();
    let c::SettingEdit::Set(set) = &items.get(0).unwrap().edit else { panic!("set") };
    set.value.clone()
}
fn core(value: c::SettingValue) -> s::ProviderSetting {
    let c::SettingValue::Provider(provider) = &value else { panic!("provider") };
    let change = c::ChangeSettings {
        against: 1,
        items: skein_lib::List::from_boxed(Box::new([c::SettingItem {
            setting: c::Setting::Provider(c::SettingNumber { number: provider.provider.number }),
            edit: c::SettingEdit::Set(c::SettingSet { value }),
        }])),
        running: c::Running::Stays,
    };
    let change = wire::wire_change_settings(&change, &Limits::standard().settings).unwrap();
    let items =
        jig_protocol_engine_clients::translate::settings::items(change.items, &[], &Limits::standard().settings)
            .unwrap();
    let s::Edit::Set(s::Value::Provider(value)) = &items.first().unwrap().edit else { panic!("provider") };
    *value.clone()
}
fn jig(bytes: &[u8]) -> s::ProviderSetting {
    let document =
        jig_protocol_config::document(bytes, &DocumentLimits { bytes: 16384, tokens: 4096, depth: 32 }).unwrap();
    let mut problems = Problems::empty();
    let value = llm::provider_setting(&document, 0, &Path::root(), &mut problems);
    assert!(problems.is_empty(), "{problems:?}");
    value.unwrap()
}
fn printed(value: &c::SettingValue) -> Box<[u8]> {
    let limits = JsonLimits { depth: 32, length: 16384 };
    let mut measured = Encoder::measure(&limits);
    render::encode_value(value, &mut measured);
    let mut out = Encoder::write(measured.measured().unwrap(), &limits);
    render::encode_value(value, &mut out);
    out.finish()
}
fn jig_printed(value: &s::ProviderSetting) -> Box<[u8]> {
    let limits = JsonLimits { depth: 32, length: 16384 };
    let mut measured = Encoder::measure(&limits);
    llm::render_provider(value, &mut measured);
    let mut out = Encoder::write(measured.measured().unwrap(), &limits);
    llm::render_provider(value, &mut out);
    out.finish()
}
#[test]
fn provider_json_agrees_with_jig_in_both_directions() {
    let jig_example = LOCAL.replace("\"number\":3", "\"number\":4");
    for bytes in [LOCAL.as_bytes(), jig_example.as_bytes(), FULL.as_bytes()] {
        let client = port(bytes);
        let value = core(client.clone());
        assert_eq!(jig(bytes), value);
        let port_json = printed(&client);
        assert_eq!(jig(&port_json), value);
        let jig_json = jig_printed(&value);
        assert_eq!(port(&jig_json), client);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&port_json).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&jig_json).unwrap()
        );
    }
}
#[test]
fn ask_temper_reads_and_proposes_the_whole_local_provider() {
    let read = super::decided(b"settings", br#"{"setting":{"kind":"provider","number":3}}"#).unwrap();
    assert_eq!(
        read.tool,
        temper_engine_domain::Tool::Settings {
            read: jig_domain_core::SettingsRead::Setting { setting: s::Setting::Provider(3), revision: 0 },
            from: 0
        }
    );
    let bytes = format!(
        r#"{{"against":1,"reason":"Add local","items":[{{"setting":{{"kind":"provider","number":3}},"set":{LOCAL}}}]}}"#
    );
    let call = super::decided(b"propose_settings", bytes.as_bytes()).unwrap();
    let temper_engine_domain::Tool::ProposeSettings { items, .. } = call.tool else { panic!("proposal") };
    assert_eq!(
        items.as_ref(),
        &[s::Item {
            setting: s::Setting::Provider(3),
            edit: s::Edit::Set(s::Value::Provider(Box::new(jig(LOCAL.as_bytes()))))
        }]
    );
    let value = jig(LOCAL.as_bytes());
    let answer = jig_domain_core::SettingsAnswer::Setting(jig_domain_core::reads::SettingDocument {
        project: None,
        setting: s::Setting::Provider(3),
        revision: 1,
        value: Some(s::Value::Provider(Box::new(value))),
        overlay: None,
        application: None,
        proposed: None,
    });
    let text = crate::settings::settings_text(&answer, None, &[]).unwrap();
    assert!(skein_lib::bytes::find(&text, LOCAL.as_bytes()).is_some());
}
