use super::*;

fn provider(number: u32, withdrawn: bool) -> c::SettingValue {
    c::SettingValue::Provider(Box::new(c::ProviderSetting {
        provider: c::Provider {
            number,
            name: b"local".as_slice().into(),
            endpoint: Some(b"local".as_slice().into()),
            direct: None,
            list: None,
            signed_in_plan: false,
            manage_usage: None,
            sign_in: None,
            keys: false,
            programs: Box::new([]),
        },
        endpoints: Box::new([c::Endpoint {
            number: 5,
            name: b"local".as_slice().into(),
            dialect: 3,
            provider: b"local".as_slice().into(),
            program: None,
            address: c::EndpointAddress::Socket("127.0.0.1:8080".parse().unwrap()),
            transport: c::Transport::Plaintext,
            headers: Box::new([]),
            connect: Duration::from_secs(10),
            handshake: Duration::from_secs(10),
        }]),
        withdrawn,
    }))
}
fn identity(number: u32) -> c::Setting {
    c::Setting::Provider(c::SettingNumber { number })
}
fn document(number: u32, withdrawn: bool) -> c::ReadResult {
    c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
        setting: identity(number),
        revision: 5,
        value: Some(provider(number, withdrawn)),
        overlay: None,
        application: None,
        proposed: None,
    })))
}
fn inventory() -> c::ReadResult {
    c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
        newest: 5,
        settings: list([
            c::SettingLine {
                setting: identity(1),
                shown: None,
                follows: c::SettingFollows::Default,
                last: None,
                application: None,
            },
            c::SettingLine {
                setting: identity(3),
                shown: None,
                follows: c::SettingFollows::Change,
                last: Some(5),
                application: None,
            },
            c::SettingLine {
                setting: c::Setting::Worker(c::SettingNumber { number: 2 }),
                shown: None,
                follows: c::SettingFollows::Change,
                last: Some(5),
                application: None,
            },
        ]),
        held: list([]),
        pending: list([]),
    })))
}
#[test]
fn ctl_applies_proposes_and_shows_a_provider_setting() {
    let value = provider(3, false);
    let file = file_from_setting(&setting_json(identity(3), value.clone()));
    let decoded = settings_file::decode(&file, &crate::limits::tools()).unwrap();
    assert_eq!(
        decoded.as_slice(),
        &[c::SettingItem { setting: identity(3), edit: c::SettingEdit::Set(c::SettingSet { value: value.clone() }) }]
    );
    let (root, call) = changed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).unwrap() else { panic!("change") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings") };
    assert_eq!(change.items, decoded);
    let (root, call) = proposed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).unwrap() else { panic!("proposal") };
    let c::Ask::Propose { action: c::ProposeAction::Settings(proposal), .. } = ask.as_ref() else { panic!("settings") };
    assert_eq!(proposal.items, decoded);
    let (mut root, call) = fixtures::read_call(&["settings", "show", "provider", "3"]);
    assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Settings);
    root.serve(call, inventory());
    assert_eq!(
        root.reads.get(&call).unwrap().1,
        c::Read::Setting(c::ReadSetting { setting: identity(3), revision: 5, proposal: None })
    );
    root.serve(call, document(3, false));
    let answer = root.answer(call).unwrap();
    let text = core::str::from_utf8(&answer.stdout).unwrap();
    assert!(text.starts_with("provider 3, revision 5\n"), "{text}");
    assert!(text.contains(r#""name":"local","endpoint":"local""#), "{text}");
    assert!(!text.contains("stock"), "{text}");
    let (mut root, call) = fixtures::read_call(&["settings", "show", "provider", "1"]);
    root.serve(call, inventory());
    root.serve(call, document(1, false));
    assert!(core::str::from_utf8(&root.answer(call).unwrap().stdout).unwrap().contains("the stock's default"));
}
#[test]
fn ctl_shows_all_providers_with_stock_and_withdrawn_marks() {
    for json in [false, true] {
        let args =
            if json { vec!["settings", "show", "provider", "--json"] } else { vec!["settings", "show", "provider"] };
        let (mut root, call) = fixtures::read_call(&args);
        assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Settings);
        root.serve(call, inventory());
        for (number, withdrawn) in [(1, false), (3, true)] {
            assert_eq!(
                root.reads.get(&call).unwrap().1,
                c::Read::Setting(c::ReadSetting { setting: identity(number), revision: 5, proposal: None })
            );
            root.serve(call, document(number, withdrawn));
        }
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        if json {
            let value: serde_json::Value = serde_json::from_slice(&answer.stdout).unwrap();
            assert_eq!(value["providers"].as_array().unwrap().len(), 2);
            assert_eq!(value["providers"][1]["value"]["withdrawn"], true);
            assert_eq!(value["stock"], serde_json::json!([1_u32]));
            let raw = &value["providers"][1]["value"];
            let file = format!(r#"{{"items":[{{"setting":{{"kind":"provider","number":3}},"set":{raw}}}]}}"#);
            let decoded = settings_file::decode(file.as_bytes(), &crate::limits::tools()).unwrap();
            assert_eq!(decoded.get(0).unwrap().edit, c::SettingEdit::Set(c::SettingSet { value: provider(3, true) }));
        } else {
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            assert!(text.contains("provider 1, revision 5\n") && text.contains("provider 3, revision 5\n"), "{text}");
            assert!(text.contains("the stock's default\n") && text.contains("withdrawn\n"), "{text}");
            assert!(!text.contains("worker 2"), "{text}");
        }
    }
}
#[test]
fn provider_accounts_refusal_explains_what_to_remove_first() {
    let refusal = c::Refusal::Malformed {
        at: 1,
        why: c::Malformed::Settings(c::MalformedSettings {
            setting: identity(3),
            fault: c::SettingFault::Accounts,
            detail: None,
        }),
    };
    let text = jig::render::settings_refusal(&refusal, 4096);
    let text = core::str::from_utf8(&text).unwrap();
    assert!(text.contains("provider 3"), "{text}");
    assert!(text.contains("provider still has accounts: remove its accounts first"), "{text}");
}
