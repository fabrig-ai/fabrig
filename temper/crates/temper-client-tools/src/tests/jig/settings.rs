#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust fixtures and golden comparisons; testing-strategy.md, step tests"
)]
use crate::{
    jig::{
        self,
        call::{Value, Values},
        rows, settings_file,
    },
    tests::fixtures,
};
use jig_client::contract as c;
use skein_lib::{Duration, List, Wall};

fn list<T, I: IntoIterator<Item = T>>(values: I) -> List<T> {
    let values: Vec<T> = values.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(values.len()).expect("fixture length"));
    for value in values {
        match out.push(value) {
            Ok(()) => {}
            Err(_) => unreachable!("fixture capacity"),
        }
    }
    out
}
fn words(text: &str) -> c::Text {
    c::Text::new(text.as_bytes().into(), 1_048_576).expect("fixture text")
}
fn authority() -> c::Authority {
    c::Authority {
        models: Some(c::AuthorityModels::Named(list([b"model".as_slice().into()]))),
        tools: 3,
        grants: list([c::Grant {
            connector: 1,
            kind: 2,
            pattern: c::Pattern {
                segments: list([b"a/b".as_slice().into(), b"\xff".as_slice().into()]),
                last: c::PatternEnd::Open(b"last".as_slice().into()),
            },
        }]),
        delegation: c::Delegation { kinds: list([c::AuthorityExecutor::Charter(3)]), tasks: 4, depth: 5 },
        spend: 600,
        deadline: Some(Wall::from_nanos(700)),
        notes: 1,
        note_resources: list([c::ResourceScope {
            connector: 1,
            pattern: c::Pattern { segments: list([]), last: c::PatternEnd::Exact(b"end".as_slice().into()) },
        }]),
    }
}
fn model() -> c::SettingModel {
    c::SettingModel {
        dialect: 1,
        endpoint: 2,
        program: Some(words("program")),
        name: words("model"),
        window: 4096,
        output: 1024,
        input_price: 1,
        cached_price: 2,
        cache_write_price: Some(3),
        output_price: 4,
        price_unit: 1000,
        efforts: list([words("high")]),
        effort: Some(words("high")),
    }
}
fn documents() -> List<c::CharterDocument> {
    list([
        c::CharterDocument {
            connector: 0,
            name: list(["forge", "code.example", "team", "rules", "branch", "main"].map(|s| s.as_bytes().into())),
            path: b"AGENTS.md".as_slice().into(),
            heading: Some(b"Rules".as_slice().into()),
        },
        c::CharterDocument {
            connector: 0,
            name: list(
                ["forge", "code.example", "team", "rules", "branch", "topic", "rules"].map(|s| s.as_bytes().into()),
            ),
            path: b"docs/rules.md".as_slice().into(),
            heading: None,
        },
    ])
}
fn charter_file(documents: &str) -> Vec<u8> {
    let (setting, c::SettingValue::Charter(mut value)) = values().remove(0) else { panic!("charter") };
    value.documents = list([]);
    let file = file_from_setting(&setting_json(setting, c::SettingValue::Charter(value)));
    String::from_utf8(file)
        .expect("file JSON")
        .replace("\"documents\":[]", &format!("\"documents\":{documents}"))
        .into_bytes()
}
const DOCUMENTS: &str = r#"[{"forge":"code.example","repository":"team/rules","branch":"main","path":"AGENTS.md","heading":"Rules"},{"forge":"code.example","repository":"team/rules","branch":"topic/rules","path":"docs/rules.md"}]"#;

#[test]
fn a_settings_file_charter_names_documents() {
    let file = charter_file(DOCUMENTS);
    let (root, call) = changed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") };
    let (setting, value) = values().remove(0);
    assert_eq!(
        change,
        &c::ChangeSettings {
            against: 5,
            items: list([c::SettingItem {
                setting,
                edit: c::SettingEdit::Set(c::SettingSet { value: value.clone() })
            }]),
            running: c::Running::Moves
        }
    );
    let (root, call) = proposed(&file, true);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent") else { panic!("jig ask") };
    let c::Ask::Propose { action: c::ProposeAction::Settings(proposal), .. } = ask.as_ref() else {
        panic!("settings proposal")
    };
    assert_eq!(proposal.items, change.items);

    let form = jig::form_setting_value(&value);
    let bounds = crate::limits::form();
    let wire_value = jig_client_protocol_link::translate::settings::wire_setting_value(&value, &bounds.settings)
        .expect("wire value");
    let wire = jig_client_protocol_wire::SettingDocument::new(
        &bounds.settings,
        jig_client_protocol_wire::SettingDocumentParts {
            setting: jig_client_protocol_wire::Setting::Charter(
                jig_client_protocol_wire::SettingNumber::new(
                    &bounds.settings,
                    jig_client_protocol_wire::SettingNumberParts { number: 3 },
                )
                .expect("number"),
            ),
            revision: 5,
            value: Some(wire_value),
            application: None,
            proposed: None,
        },
    )
    .expect("wire document");
    assert_eq!(settings_form::wire::restore_setting_document(&wire, &bounds).expect("restore").value, Some(form));
    let neutral = settings_form::view::ChangeSettings {
        against: 5,
        items: settings_form::file::decode(&file, &bounds).expect("neutral items"),
        running: settings_form::view::Running::Moves,
    };
    let wire = settings_form::wire::wire_change_settings(&neutral, &bounds.settings).expect("neutral wire");
    let preset = crate::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
    assert_eq!(
        jig_client_protocol_link::translate::settings::restore_change_settings(&wire, &link).expect("client ask"),
        *change
    );
}
#[test]
fn settings_show_charter_renders_its_documents() {
    let (setting, value) = values().remove(0);
    for json in [false, true] {
        let mut argv = vec!["settings", "show", "charter", "3"];
        if json {
            argv.push("--json");
        }
        let answer = read(
            &argv,
            c::ReadValue::Setting(Box::new(c::SettingDocument {
                setting,
                revision: 5,
                value: Some(value.clone()),
                application: None,
                proposed: None,
            })),
        );
        assert_eq!(answer.status, 0);
        golden(if json { "settings_value_0_json" } else { "settings_value_0" }, &answer.stdout);
    }
}
#[test]
fn a_document_that_is_no_forge_branch_renders_its_name() {
    for (connector, name) in [
        (7, vec!["forge", "code.example", "team", "rules", "branch", "main"]),
        (0, vec!["other", "resource"]),
        (0, vec!["forge", "code.example", "team", "rules", "branch"]),
    ] {
        let (setting, c::SettingValue::Charter(mut value)) = values().remove(0) else { panic!("charter") };
        value.documents = list([c::CharterDocument {
            connector,
            name: list(name.iter().map(|s| s.as_bytes().into())),
            path: b"rules.md".as_slice().into(),
            heading: None,
        }]);
        let json = setting_json(setting, c::SettingValue::Charter(value));
        let text = core::str::from_utf8(&json).expect("JSON");
        let names = name.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(",");
        assert!(
            text.contains(&format!(
                r#""documents":[{{"connector":{connector},"name":[{names}],"path":"rules.md","heading":null}}]"#
            )),
            "{text}"
        );
        let file = file_from_setting(&json);
        let error = settings_file::decode(&file, &crate::limits::tools()).expect_err("not a forge branch");
        let error = core::str::from_utf8(&error).expect("error text");
        assert!(error.contains("charter.documents[0].name") && error.contains("shape"), "{error}");
        let (mut root, call) = changed(&file, false);
        assert!(root.sends.is_empty());
        assert_eq!(root.answer(call).expect("local refusal").status, 1);
    }
}
#[test]
fn document_bounds_refuse_each_member_one_past_its_bound() {
    let file = charter_file(DOCUMENTS);
    for (name, length, path) in [
        ("charter_documents", 2, "charter.documents"),
        ("charter_document_name", 7, "charter.documents[1].branch"),
        ("charter_document_name_item", 12, "charter.documents[0].forge"),
        ("charter_document_path", 13, "charter.documents[1].path"),
        ("charter_document_heading_some", 5, "charter.documents[0].heading"),
    ] {
        for (bound, allowed) in [(length, true), (length - 1, false)] {
            let mut limits = crate::limits::tools();
            match name {
                "charter_documents" => limits.settings.charter_documents = bound,
                "charter_document_name" => limits.settings.charter_document_name = bound,
                "charter_document_name_item" => limits.settings.charter_document_name_item = bound,
                "charter_document_path" => limits.settings.charter_document_path = bound,
                "charter_document_heading_some" => limits.settings.charter_document_heading_some = bound,
                _ => panic!("fixture bound"),
            }
            let result = settings_file::decode(&file, &limits);
            assert_eq!(result.is_ok(), allowed, "{name}: {result:?}");
            if let Err(error) = result {
                let text = core::str::from_utf8(&error).expect("error text");
                assert!(text.contains("item 1") && text.contains(path), "{text}");
            }
        }
    }
}

#[test]
fn document_name_parts_and_fixed_byte_bounds_name_the_refused_member() {
    let limits = crate::limits::tools();
    for (member, maximum, base) in [
        ("forge", limits.settings.charter_document_name_item, "x"),
        ("repository", limits.settings.charter_document_name_item, "x/repo"),
        ("repository", limits.settings.charter_document_name_item, "owner/x"),
        ("branch", limits.settings.charter_document_name_item, "x"),
        ("path", 256, "x"),
        ("heading", 80, "x"),
    ] {
        for extra in [0, 1] {
            let count = usize::try_from(maximum + extra).expect("test length");
            let text = base.replace('x', &"x".repeat(count));
            let mut fields = [
                ("forge", "host".to_owned()),
                ("repository", "owner/repo".to_owned()),
                ("branch", "main".to_owned()),
                ("path", "rules.md".to_owned()),
                ("heading", "Rules".to_owned()),
            ];
            fields.iter_mut().find(|(name, _)| *name == member).expect("field").1 = text;
            let document =
                fields.iter().map(|(name, value)| format!(r#""{name}":"{value}""#)).collect::<Vec<_>>().join(",");
            let file = charter_file(&format!("[{{{document}}}]"));
            let result = settings_file::decode(&file, &limits);
            assert_eq!(result.is_ok(), extra == 0, "{member} bound {maximum}, extra {extra}: {result:?}");
            if let Err(error) = result {
                let error = core::str::from_utf8(&error).expect("error text");
                assert!(error.contains(&format!("charter.documents[0].{member}")), "{error}");
            }
        }
    }
}
#[test]
fn document_shape_refusals_name_the_member_and_a_heading_is_optional() {
    for (member, value) in [
        ("forge", "null"),
        ("forge", r#""""#),
        ("repository", r#""owner""#),
        ("repository", r#""owner/repo/other""#),
        ("repository", r#""/repo""#),
        ("repository", r#""owner/""#),
        ("branch", r#""topic//rules""#),
        ("path", "false"),
        ("path", r#""""#),
        ("heading", "7"),
        ("heading", r#""""#),
    ] {
        let document =
            r#"[{"forge":"host","repository":"owner/repo","branch":"main","path":"rules.md","heading":null}]"#
                .to_owned();
        let original = match member {
            "forge" => r#""forge":"host""#,
            "repository" => r#""repository":"owner/repo""#,
            "branch" => r#""branch":"main""#,
            "path" => r#""path":"rules.md""#,
            "heading" => r#""heading":null"#,
            _ => panic!("fixture field"),
        };
        let document = document.replace(original, &format!(r#""{member}":{value}"#));
        let file = charter_file(&document);
        let error = settings_file::decode(&file, &crate::limits::tools()).expect_err("invalid document");
        let error = core::str::from_utf8(&error).expect("error text");
        assert!(error.contains(&format!("charter.documents[0].{member}")), "{error}");
    }
    for heading in ["", r#", "heading":null"#] {
        let file = charter_file(&format!(
            r#"[{{"forge":"host","repository":"owner/repo","branch":"main","path":"rules.md"{heading}}}]"#
        ));
        let items = settings_file::decode(&file, &crate::limits::tools()).expect("optional heading");
        let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Charter(charter) }) =
            &items.get(0).expect("charter item").edit
        else {
            panic!("charter")
        };
        assert!(charter.documents.get(0).expect("document").heading.is_none());
    }
}

fn values() -> Vec<(c::Setting, c::SettingValue)> {
    vec![
        (
            c::Setting::Charter(c::SettingNumber { number: 3 }),
            c::SettingValue::Charter(Box::new(c::Charter {
                documents: documents(),
                number: 3,
                instructions: words("keep \"boundaries\"\nwhole"),
                waiting: Duration::from_secs(1),
                resume: true,
                turns: 8,
                time: Duration::from_secs(2),
                reserve_turns: 1,
                reserve_time: Duration::from_secs(1),
                model: model(),
                alternatives: list([model()]),
                inspect: true,
                modify: true,
                shell: false,
                agents: false,
                call_timeout: Duration::from_secs(3),
                hosts: c::SettingHosts::Both,
                spend: 900,
                wake: c::WakePolicy {
                    words: c::WakeRule::Immediate,
                    notices: c::WakeRule::Immediate,
                    news: c::WakeRule::Immediate,
                    results: c::ResultsWake::LastOrFailure,
                    questions: true,
                    answers: true,
                    timers: true,
                },
            })),
        ),
        (
            c::Setting::Template(c::SettingNumber { number: 4 }),
            c::SettingValue::Template(Box::new(c::Template {
                resources: List::with_capacity(0),
                number: 4,
                name: c::DocumentText::complete(words("work")),
                executor: c::TemplateExecutor::Charter(3),
                contract: c::TaskContractDocument::Report { words: 1024 },
                shallower: 1,
                budget: c::TemplateBudget::AsAllowed { cap: Some(1000) },
                deadline: c::TemplateDeadline::Span(Duration::from_secs(10)),
                tracked: Some(5),
                flags: c::TaskFlags {
                    conversation: false,
                    listed: true,
                    retention: c::Retention::Kept,
                    ended_by: c::EndedBy::Result,
                },
                label: 1,
                grace: Some(Duration::from_secs(1)),
                llm: Some(c::LlmChoice {
                    account: Some(9),
                    main: c::ModelEffort { model: words("model"), effort: Some(words("high")) },
                    subagents: list([c::ModelEffort { model: words("model"), effort: None }]),
                }),
            })),
        ),
        (
            c::Setting::Policy(c::SettingNumber { number: 5 }),
            c::SettingValue::Policy(Box::new(c::SettingPolicy {
                escalation_role: Some(1),
                ceiling: authority(),
                projections: list([authority().grants.get(0).expect("grant").clone()]),
                period_spend: 2000,
                roles: list([c::PolicyEditRole {
                    number: 1,
                    authority: authority(),
                    period_spend: 1000,
                    requests: 3,
                    decides: 1,
                }]),
                requirements: list([]),
                permissions: list([c::PolicyPermission { connector: 1, permission: 2, role: 1 }]),
                templates: list([c::TemplateOverride {
                    template: 4,
                    executor: None,
                    budget: Some(c::TemplateBudget::Amount(500)),
                    deadline: Some(c::TemplateDeadline::Given),
                    priority: None,
                }]),
            })),
        ),
        (
            c::Setting::Rules,
            c::SettingValue::Rules(Box::new(c::Rules {
                ceiling: authority(),
                period_spend: 3000,
                minimum_run_spend: 10,
                maximum_run_spend: 1000,
                implies: list([c::KindImplication { connector: 1, kind: 2, implies: 3 }]),
                requirements: list([]),
            })),
        ),
    ]
}
fn read(argv: &[&str], value: c::ReadValue) -> jig::Printed {
    let (mut root, call) = fixtures::read_call(argv);
    if let c::ReadValue::Setting(_) = &value {
        newest_for_setting(&mut root, call);
    }
    root.serve(call, c::ReadResult::Value(value));
    while let Some((_, c::Read::Setting(request))) = root.reads.get(&call).cloned() {
        let (_, value) =
            values().into_iter().find(|(setting, _)| *setting == request.setting).expect("fixture setting");
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                setting: request.setting,
                revision: 6,
                value: Some(value),
                application: None,
                proposed: None,
            }))),
        );
    }
    root.answer(call).expect("read completed")
}
fn newest_for_setting(root: &mut fixtures::Harness, call: u32) {
    if root.reads.get(&call).expect("setting read").1 == c::Read::Settings {
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
                newest: 5,
                settings: list([]),
                held: list([]),
                pending: list([]),
            }))),
        );
    }
}
fn setting_json(setting: c::Setting, value: c::SettingValue) -> Vec<u8> {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut call = fixtures::call(&["settings", "show", "rules"]);
    call.tool = rows::SETTING_AT;
    let number = root.submit_prepared(call, true, None);
    newest_for_setting(&mut root, number);
    root.serve(
        number,
        c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting,
            revision: 5,
            value: Some(value),
            application: None,
            proposed: None,
        }))),
    );
    root.answer(number).expect("setting answered").stdout.to_vec()
}
fn file_from_setting(bytes: &[u8]) -> Vec<u8> {
    // The document's selector and value are the FILE item's exact values.
    let text = core::str::from_utf8(bytes).expect("JSON");
    let text = text.replace(",\"revision\":5,\"value\":", ",\"set\":");
    format!("{{\"items\":[{text}]}}").into_bytes()
}
fn changed(bytes: &[u8], json: bool) -> (fixtures::Harness, u32) {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut call =
        fixtures::call(&["settings", "apply", "FILE", "--against", "5", "--running", "moves", "--key", "settings-5"]);
    *call.values.get_mut(0).expect("FILE") = Some(Values::One(Value::Text(bytes.into())));
    let number = root.submit_prepared(call, json, None);
    (root, number)
}
fn head(by: c::RevisionDecider) -> c::RevisionHead {
    c::RevisionHead {
        number: 5,
        against: 4,
        at: Wall::EPOCH,
        by,
        proposal: Some(c::ProposalKey { proposer: c::Proposer::Party(2), number: 7 }),
        moved: false,
    }
}
fn golden(name: &str, bytes: &[u8]) {
    let path = std::path::Path::new("src/tests/golden").join(name).with_extension("txt");
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some() {
        std::fs::write(&path, bytes).expect("write settings golden");
    }
    assert_eq!(bytes, std::fs::read(path).expect("settings golden"));
}
#[test]
fn settings_calls_parse_and_write_back_with_existing_numbers_unchanged() {
    for argv in [
        vec!["settings", "show"],
        vec!["settings", "show", "charter", "3", "--revision", "5"],
        vec!["settings", "show", "template", "4"],
        vec!["settings", "show", "policy", "5"],
        vec!["settings", "show", "rules"],
        vec!["revision", "show", "5"],
        vec!["revision", "list", "--before", "5"],
        vec!["settings", "apply", "-", "--against", "5", "--running", "moves", "--key", "settings-5"],
    ] {
        let call = fixtures::call(&argv);
        assert_eq!(fixtures::back(&call), fixtures::owned(&argv));
        assert_eq!(fixtures::printed(&call), jig::call::Parsed::Call { call, json: false });
    }
    assert_eq!((rows::EXPORT_AT, rows::IMPORT_AT, rows::SETTINGS_AT, rows::CHANGE_SETTINGS_AT), (41, 42, 43, 47));
    for argv in [
        vec!["settings", "show", "rules", "--project", "1"],
        vec!["settings", "show", "charter", "3", "--project", "1"],
        vec!["revision", "show", "4294967296"],
        vec!["settings", "show", "--watch"],
        vec!["settings", "apply", "FILE", "--against", "5"],
    ] {
        assert!(fixtures::parse(&argv).is_err(), "{argv:?}");
    }
}
#[test]
fn each_whole_setting_json_is_exactly_the_set_value_and_sends_under_its_name() {
    for (setting, value) in values() {
        let json = setting_json(setting, value.clone());
        let file = file_from_setting(&json);
        let items = settings_file::decode(&file, &crate::limits::tools()).expect("whole setting round trip");
        assert_eq!(items.as_slice(), &[c::SettingItem { setting, edit: c::SettingEdit::Set(c::SettingSet { value }) }]);
        let (root, call) = changed(&file, false);
        assert_eq!(root.sent_name(call), Some(fixtures::named("settings-5")));
        let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("one keyed request") else {
            panic!("jig ask")
        };
        assert_eq!(
            ask.as_ref(),
            &c::Ask::ChangeSettings(c::ChangeSettings { against: 5, items, running: c::Running::Moves })
        );
    }
}
#[test]
fn settings_decoder_accepts_removed_and_reset_but_refuses_unknown_members_and_duplicates() {
    let limits = crate::limits::tools();
    let bytes = br#"{"items":[{"setting":{"kind":"charter","number":3},"edit":"removed"},{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    assert_eq!(settings_file::decode(bytes, &limits).expect("two edits").len(), 2);
    for (bytes, member) in [
        (br#"{"items":[],"extra":0}"#.as_slice(), "extra"),
        (br#"{"items":[{"setting":{"kind":"rules","project":1},"edit":"reset"}]}"#, "project"),
        (br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset","other":1}]}"#, "other"),
        (
            br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"},{"setting":{"kind":"rules"},"edit":"removed"}]}"#,
            "duplicate setting",
        ),
        (br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset","set":{}}]}"#, "exactly one"),
        (br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset","edit":"removed"}]}"#, "duplicate member"),
    ] {
        let error = settings_file::decode(bytes, &limits).expect_err("invalid input");
        assert!(core::str::from_utf8(&error).expect("message").contains(member));
        let (mut root, call) = changed(bytes, false);
        assert!(root.sends.is_empty());
        assert_eq!(root.answer(call).expect("local refusal").status, 1);
    }
}
#[test]
fn settings_file_checks_file_and_item_count_bounds_before_sending() {
    let mut limits = crate::limits::tools();
    limits.ask_bytes = 10;
    assert!(
        core::str::from_utf8(&settings_file::decode(br#"{"items":[]}"#, &limits).expect_err("bytes bound"))
            .expect("message")
            .contains("file bytes: bound 10")
    );
    let file = format!("{{\"items\":[{}]}}", vec![r#"{"setting":{"kind":"rules"},"edit":"reset"}"#; 1025].join(","));
    assert!(
        core::str::from_utf8(&settings_file::decode(file.as_bytes(), &crate::limits::tools()).expect_err("item count"))
            .expect("message")
            .contains("items: bound 1024")
    );
}
#[test]
fn whole_settings_document_renderings_are_stable() {
    let inventory = c::ReadValue::Settings(Box::new(c::SettingsDocument {
        newest: 5,
        settings: list([
            c::SettingLine {
                setting: c::Setting::Charter(c::SettingNumber { number: 3 }),
                name: Some(c::DocumentText::complete(words("worker"))),
                follows: c::SettingFollows::Change,
                last: Some(5),
                application: None,
            },
            c::SettingLine {
                setting: c::Setting::Rules,
                name: None,
                follows: c::SettingFollows::Default,
                last: Some(0),
                application: None,
            },
        ]),
        held: list([c::HeldRevision {
            head: head(c::RevisionDecider::Party(c::RevisionParty { number: 2 })),
            holders: 3,
        }]),
        pending: list([c::SettingsProposal { proposer: c::Proposer::Party(2), number: 7 }]),
    }));
    let revision = c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
        head: head(c::RevisionDecider::Release),
        items: list([
            c::RevisionItem {
                setting: c::Setting::Charter(c::SettingNumber { number: 3 }),
                edit: c::RevisionEdit::TooLarge(c::Size { bytes: 1_000_000 }),
            },
            c::RevisionItem { setting: c::Setting::Rules, edit: c::RevisionEdit::Reset },
        ]),
    })));
    let revisions = c::ReadValue::Revisions(Box::new(c::RevisionsPage {
        rows: list([c::RevisionRow {
            head: head(c::RevisionDecider::FirstStart),
            items: list([c::RevisionItemHead { setting: c::Setting::Rules, edit: c::RevisionItemEdit::Set }]),
            more: 2,
        }]),
        next: Some(4),
        forgotten: false,
    }));
    let forgotten = c::ReadValue::Revision(Box::new(c::RevisionAnswer::Forgotten(c::RevisionForgotten { oldest: 2 })));
    for (argv, document, name) in [
        (vec!["settings", "show"], inventory, "settings_inventory"),
        (vec!["revision", "show", "5"], revision, "settings_revision"),
        (vec!["revision", "list"], revisions, "settings_revisions"),
        (vec!["revision", "show", "1"], forgotten, "settings_forgotten"),
    ] {
        let text = read(&argv, document.clone());
        assert_eq!(text.status, 0);
        golden(name, &text.stdout);
        let mut argv = argv;
        argv.push("--json");
        let json = read(&argv, document);
        assert_eq!(json.status, 0);
        golden(&format!("{name}_json"), &json.stdout);
    }
    for (index, (setting, value)) in values().into_iter().enumerate() {
        let json = setting_json(setting, value);
        golden(&format!("settings_value_{index}_json"), &json);
    }
}
#[test]
fn settings_change_outcomes_name_the_revision_and_refusal_facts() {
    let file = br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    let (mut root, call) = changed(file, false);
    root.done(call, c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 6 }));
    assert_eq!(root.answer(call).expect("changed").stdout.as_ref(), b"revision 6\n");
    for (refusal, expected) in [
        (c::Refusal::Stale(c::RefusalStale { newest: 6 }), "stale; newest revision 6"),
        (
            c::Refusal::Malformed {
                at: 1,
                why: c::Malformed::Settings(c::MalformedSettings {
                    setting: c::Setting::Rules,
                    fault: c::SettingFault::Owner,
                }),
            },
            "out of shape; item 1; rules: owner",
        ),
        (
            c::Refusal::Full(c::RefusalFull { revision: 2, tasks: list([41, 42]), more: 3 }),
            "full; oldest held revision 2; live tasks: 41 42; 3 more",
        ),
        (c::Refusal::Role { held: None }, "role"),
    ] {
        for json in [false, true] {
            let (mut root, call) = changed(file, json);
            root.answered(call, c::Answer::Refused { refusal: refusal.clone(), at: None });
            let answer = root.answer(call).expect("refused");
            assert_eq!(answer.status, 2);
            if !json && expected != "role" {
                assert!(core::str::from_utf8(&answer.stderr).expect("text").contains(expected));
            }
        }
    }
}

#[test]
fn settings_values_refuse_each_nonempty_byte_and_list_bound_on_both_sides() {
    // Exercise the same input at its bound and one below it. Runtime limits
    // come from jig's link, so the tools do not install a parallel preset.
    let (_, policy) = values().remove(2);
    let c::SettingValue::Policy(policy) = policy else {
        panic!("policy");
    };
    let wire =
        jig_client_protocol_link::translate::settings::wire_setting_policy(&policy, &crate::limits::tools().settings)
            .expect("wire policy");
    let grant_bytes = u32::try_from(wire.ceiling().grants().len()).expect("grant bytes");
    for (index, name, length) in [
        (0, "charter_instructions", 23),
        (0, "charter_alternatives", 1),
        (0, "setting_model_program_some", 7),
        (0, "setting_model_name", 5),
        (0, "setting_model_efforts", 1),
        (0, "setting_model_efforts_item", 4),
        (0, "setting_model_effort_some", 4),
        (1, "document_text_bytes", 4),
        (1, "model_effort_model", 5),
        (1, "model_effort_effort_some", 4),
        (2, "setting_policy_roles", 1),
        (2, "setting_policy_permissions", 1),
        (2, "setting_policy_templates", 1),
        (2, "authority_grants", grant_bytes),
        (2, "delegation_kinds", 1),
        (2, "pattern_segments", 2),
        (2, "pattern_segments_item", 3),
        (2, "pattern_end_open_value", 4),
        (2, "model_set_names_names", 1),
        (2, "model_set_names_names_item", 5),
        (3, "rules_implies", 1),
    ] {
        let (setting, value) = values().remove(index);
        let file = file_from_setting(&setting_json(setting, value));
        for (bound, allowed) in [(length, true), (length - 1, false)] {
            let mut limits = crate::limits::tools();
            let wire = &mut limits.settings;
            match name {
                "charter_instructions" => wire.charter_instructions = bound,
                "charter_alternatives" => wire.charter_alternatives = bound,
                "setting_model_program_some" => wire.setting_model_program_some = bound,
                "setting_model_name" => wire.setting_model_name = bound,
                "setting_model_efforts" => wire.setting_model_efforts = bound,
                "setting_model_efforts_item" => wire.setting_model_efforts_item = bound,
                "setting_model_effort_some" => wire.setting_model_effort_some = bound,
                "document_text_bytes" => wire.document_text_bytes = bound,
                "model_effort_model" => wire.model_effort_model = bound,
                "model_effort_effort_some" => wire.model_effort_effort_some = bound,
                "setting_policy_roles" => wire.setting_policy_roles = bound,
                "setting_policy_permissions" => wire.setting_policy_permissions = bound,
                "setting_policy_templates" => wire.setting_policy_templates = bound,
                "authority_grants" => wire.authority_grants = bound,
                "delegation_kinds" => wire.delegation_kinds = bound,
                "pattern_segments" => wire.pattern_segments = bound,
                "pattern_segments_item" => wire.pattern_segments_item = bound,
                "pattern_end_open_value" => wire.pattern_end_open_value = bound,
                "model_set_names_names" => wire.model_set_names_names = bound,
                "model_set_names_names_item" => wire.model_set_names_names_item = bound,
                "rules_implies" => wire.rules_implies = bound,
                _ => panic!("fixture bound name"),
            }
            let result = settings_file::decode(&file, &limits);
            assert_eq!(result.is_ok(), allowed, "{name} bound {bound}: {result:?}");
            if let Err(error) = result {
                let text = core::str::from_utf8(&error).expect("message");
                assert!(text.contains("item 1") && text.contains(name), "{text}");
            }
        }
    }
}

#[test]
fn settings_reads_request_the_exact_contract_and_do_not_watch_settings() {
    for (argv, expected) in [
        (vec!["settings", "show"], c::Read::Settings),
        (
            vec!["settings", "show", "charter", "3", "--revision", "5"],
            c::Read::Setting(c::ReadSetting {
                setting: c::Setting::Charter(c::SettingNumber { number: 3 }),
                revision: 5,
                proposal: None,
            }),
        ),
        (vec!["settings", "show", "rules"], c::Read::Settings),
        (vec!["revision", "show", "5"], c::Read::Revision(c::ReadRevision { number: 5 })),
        (vec!["revision", "list", "--before", "5"], c::Read::Revisions(c::ReadRevisions { before: Some(5) })),
    ] {
        let (root, call) = fixtures::read_call(&argv);
        assert_eq!(root.reads.get(&call).expect("one read").1, expected);
        assert_eq!(root.watches.len(), 1, "only shared me watch");
    }
}

#[test]
fn an_unqualified_setting_reads_the_inventoried_revision_before_its_value() {
    for (argv, setting) in [
        (vec!["settings", "show", "charter", "3"], c::Setting::Charter(c::SettingNumber { number: 3 })),
        (vec!["settings", "show", "template", "4"], c::Setting::Template(c::SettingNumber { number: 4 })),
        (vec!["settings", "show", "policy", "5"], c::Setting::Policy(c::SettingNumber { number: 5 })),
        (vec!["settings", "show", "rules"], c::Setting::Rules),
    ] {
        let (mut root, call) = fixtures::read_call(&argv);
        assert_eq!(root.reads.get(&call).expect("first read").1, c::Read::Settings);
        newest_for_setting(&mut root, call);
        assert!(root.answer(call).is_none());
        assert_eq!(
            root.reads.get(&call).expect("value read").1,
            c::Read::Setting(c::ReadSetting { setting, revision: 5, proposal: None })
        );
        root.serve(call, c::ReadResult::None(c::ReadNone::Unknown));
        assert_eq!(root.answer(call).expect("value refusal").status, 2);
        assert_eq!(root.watches.len(), 1, "only the shared me watch");
    }
}

#[test]
fn settings_reads_keep_their_neighbours_not_served_and_role_statuses() {
    for argv in [
        vec!["settings", "show"],
        vec!["settings", "show", "rules"],
        vec!["revision", "show", "5"],
        vec!["revision", "list"],
    ] {
        for (page, status) in [
            (jig_client::Readout::NotServed { family: None, served: None }, 1),
            (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), 2),
        ] {
            let (mut root, call) = fixtures::read_call(&argv);
            root.serve_readout(call, page);
            assert_eq!(root.answer(call).expect("terminal read").status, status);
        }
    }
}

#[test]
fn setting_text_and_revisions_include_whole_set_values_and_last_page_facts() {
    for (index, (setting, value)) in values().into_iter().enumerate() {
        let argv = match setting {
            c::Setting::Charter(_) => vec!["settings", "show", "charter", "3"],
            c::Setting::Template(_) => vec!["settings", "show", "template", "4"],
            c::Setting::Policy(_) => vec!["settings", "show", "policy", "5"],
            c::Setting::Application { .. } => continue,
            c::Setting::Rules => vec!["settings", "show", "rules"],
        };
        let answer = read(
            &argv,
            c::ReadValue::Setting(Box::new(c::SettingDocument {
                setting,
                revision: 5,
                value: Some(value.clone()),
                application: None,
                proposed: None,
            })),
        );
        assert_eq!(answer.status, 0);
        golden(&format!("settings_value_{index}"), &answer.stdout);
        for json in [false, true] {
            let mut argv = vec!["revision", "show", "5"];
            if json {
                argv.push("--json");
            }
            let answer = read(
                &argv,
                c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
                    head: head(c::RevisionDecider::Release),
                    items: list([c::RevisionItem {
                        setting,
                        edit: c::RevisionEdit::Set(c::SettingSet { value: value.clone() }),
                    }]),
                }))),
            );
            assert_eq!(answer.status, 0);
            golden(&format!("settings_revision_value_{index}_{}", if json { "json" } else { "text" }), &answer.stdout);
        }
    }
    let answer = read(
        &["revision", "list"],
        c::ReadValue::Revisions(Box::new(c::RevisionsPage { rows: list([]), next: None, forgotten: true })),
    );
    assert_eq!(answer.stdout.as_ref(), b"older revisions were forgotten\n");
    let answer = read(&["revision", "show", "99"], c::ReadValue::Revision(Box::new(c::RevisionAnswer::Unknown)));
    assert_eq!((answer.status, answer.stderr.as_ref()), (2, b"unknown revision\n".as_slice()));
}

#[test]
fn change_settings_running_defaults_to_stays_and_json_names_the_new_revision() {
    let mut call = fixtures::call(&["settings", "apply", "FILE", "--against", "5", "--key", "settings-5"]);
    *call.values.get_mut(0).expect("FILE") =
        Some(Values::One(Value::Text(br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#.as_slice().into())));
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let number = root.submit_prepared(call, true, None);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&number).expect("sent") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("change settings") };
    assert_eq!(change.running, c::Running::Stays);
    root.done(number, c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 6 }));
    assert_eq!(root.answer(number).expect("durable answer").stdout.as_ref(), br#"{"revision":6}"#);
}

fn requirement() -> c::PolicyRequirement {
    c::PolicyRequirement {
        connector: 1,
        kind: 2,
        pattern: c::Pattern {
            segments: list([b"one".as_slice().into()]),
            last: c::PatternEnd::Exact(b"two".as_slice().into()),
        },
        judge: c::PolicyJudge { connector: 1, requirement: 2, parameters: 3 },
        guard: c::PolicyGuard::Observed { freshness: Duration::from_secs(1) },
        must_be_guarded: true,
    }
}
#[test]
fn remaining_nested_collections_and_bytes_obey_jigs_bounds() {
    macro_rules! boundary {
        ($setting:expr, $value:expr, $field:ident, $length:expr) => {{
            let file = file_from_setting(&setting_json($setting, $value));
            let mut limits = crate::limits::tools();
            limits.settings.$field = $length;
            settings_file::decode(&file, &limits).expect("at the bound");
            limits.settings.$field = $length - 1;
            let why = settings_file::decode(&file, &limits).expect_err("past the bound");
            let why = core::str::from_utf8(&why).expect("bound text");
            assert!(why.contains("item 1") && why.contains(stringify!($field)), "{why}");
        }};
    }
    let (setting, value) = values().remove(2);
    boundary!(setting, value.clone(), setting_policy_projections, 1);
    let c::SettingValue::Policy(policy) = &value else {
        panic!("policy");
    };
    let wire =
        jig_client_protocol_link::translate::settings::wire_setting_policy(policy, &crate::limits::tools().settings)
            .expect("wire policy");
    let resource_bytes = u32::try_from(wire.ceiling().note_resources().len()).expect("resource bytes");
    boundary!(setting, value.clone(), authority_note_resources, resource_bytes);
    boundary!(setting, value.clone(), pattern_end_exact_value, 3);
    let c::SettingValue::Policy(mut policy) = value else { panic!("policy") };
    policy.requirements = list([requirement()]);
    boundary!(setting, c::SettingValue::Policy(policy), setting_policy_requirements, 1);
    let (setting, value) = values().remove(3);
    let c::SettingValue::Rules(mut rules) = value else { panic!("rules") };
    rules.requirements = list([requirement()]);
    boundary!(setting, c::SettingValue::Rules(rules), rules_requirements, 1);
    let (setting, value) = values().remove(1);
    boundary!(setting, value.clone(), llm_choice_subagents, 1);
    let c::SettingValue::Template(mut template) = value else { panic!("template") };
    template.contract = c::TaskContractDocument::Approval {
        state: b"yes".as_slice().into(),
        part: c::Part {
            connector: 1,
            kind: c::PartKind::Resource,
            key: c::PartKey::Path(list([b"path".as_slice().into()])),
        },
    };
    boundary!(setting, c::SettingValue::Template(template.clone()), task_contract_document_approval_state, 3);
    boundary!(setting, c::SettingValue::Template(template.clone()), part_key_path_value, 1);
    boundary!(setting, c::SettingValue::Template(template.clone()), part_key_path_value_item, 4);
    template.contract = c::TaskContractDocument::Verdict {
        choices: list([c::VerdictChoice {
            code: 1,
            words: 2,
            followups: 3,
            fields: list([c::FieldRule { name: b"field".as_slice().into(), max: 5 }]),
        }]),
        cut: false,
    };
    let wire = jig_client_protocol_link::translate::settings::wire_setting_value(
        &c::SettingValue::Template(template.clone()),
        &crate::limits::tools().settings,
    )
    .expect("wire template");
    let jig_client_protocol_wire::SettingValue::Template(wire) = wire else {
        panic!("wire template");
    };
    let jig_client_protocol_wire::TaskContractDocument::Verdict(verdict) = wire.contract() else {
        panic!("wire verdict");
    };
    let choice_bytes = u32::try_from(verdict.choices().len()).expect("choice bytes");
    boundary!(
        setting,
        c::SettingValue::Template(template.clone()),
        task_contract_document_verdict_choices,
        choice_bytes
    );
    boundary!(setting, c::SettingValue::Template(template.clone()), verdict_choice_fields, 1);
    boundary!(setting, c::SettingValue::Template(template), field_rule_name, 5);
}

#[test]
fn a_settings_change_keeps_the_exact_request_for_recovery_and_pending() {
    let file = br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    let (root, number) = changed(file, false);
    let (_, _, request) = root.sends.get(&number).expect("sent request");
    let records = list([skein_lib::RequestRecord {
        key: fixtures::named("settings-5"),
        scope: fixtures::PARTY,
        first_sent: None,
        request: jig_client::Held {
            key: jig_client::Key {
                asker: crate::root::RUNNER.asker(),
                number: (u64::from(fixtures::RESTORED_PREFIX) << 32u32) | 1,
            },
            request: request.clone(),
        },
    }]);
    let pending = jig::render::pending::pending(records.as_slice(), 0, false);
    assert_eq!(pending.stdout.as_ref(), b"change-settings, party 73, not sent, unanswered: it clears when the engine answers it, or past its retention; pending lists it\n");
    let mut restored = fixtures::started_with(records, fixtures::SEED, false);
    restored.deliver(c::Delivery::Me(Box::new(fixtures::me(fixtures::PARTY))));
    let (_, _, resent) = restored.sends.get(&0).expect("kept request resent");
    assert_eq!(resent, request);
    restored.done(0, c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 6 }));
    assert_eq!(restored.kept_lines.as_slice(), &[Box::<[u8]>::from(b"kept: change-settings, revision 6\n".as_slice())]);
}

#[test]
fn settings_items_check_the_aggregate_wire_byte_bound_and_maximum_count() {
    let file = br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    let mut limits = crate::limits::tools();
    let items = settings_file::decode(file, &limits).expect("items");
    let wire = jig_client_protocol_link::translate::settings::wire_change_settings(
        &c::ChangeSettings { against: 5, items, running: c::Running::Stays },
        &limits.settings,
    )
    .expect("wire items");
    let size = u32::try_from(wire.items().len()).expect("wire bytes");
    limits.settings.change_settings_items = size;
    settings_file::decode(file, &limits).expect("at item bytes bound");
    limits.settings.change_settings_items = size - 1;
    let why = settings_file::decode(file, &limits).expect_err("past item bytes bound");
    assert!(core::str::from_utf8(&why).expect("message").contains("item 1: encoded items bytes: bound"));
    let items: Vec<String> = (0_u32..1024)
        .map(|number| format!(r#"{{"setting":{{"kind":"charter","number":{number}}},"edit":"reset"}}"#))
        .collect();
    let file = format!("{{\"items\":[{}]}}", items.join(","));
    assert_eq!(
        settings_file::decode(file.as_bytes(), &crate::limits::tools()).expect("at item count bound").len(),
        1024
    );
}

#[test]
fn non_charter_revision_hints_explicitly_read_the_current_value() {
    for (setting, name) in [
        (c::Setting::Template(c::SettingNumber { number: 4 }), "template 4"),
        (c::Setting::Policy(c::SettingNumber { number: 5 }), "policy 5"),
        (c::Setting::Rules, "rules"),
    ] {
        let answer = read(
            &["revision", "show", "5"],
            c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
                head: head(c::RevisionDecider::Release),
                items: list([c::RevisionItem {
                    setting,
                    edit: c::RevisionEdit::TooLarge(c::Size { bytes: 1_000_000 }),
                }]),
            }))),
        );
        let line = format!("  {name}: too large, 1000000 bytes; read current value with settings show {name}\n");
        assert!(answer.stdout.ends_with(line.as_bytes()), "{}", String::from_utf8_lossy(&answer.stdout));
    }
}

#[test]
fn settings_proposals_and_decision_choices_parse_round_trip_and_preserve_identities() {
    for argv in [
        vec!["settings", "propose", "-", "--against", "5", "--key", "proposal-5"],
        vec!["proposal", "show", "p73/7"],
        vec!["proposal", "decide", "p73/7", "accept", "--running", "moves"],
        vec!["proposal", "decide", "t41/3", "reject", "reason", "--running", "stays"],
    ] {
        let mut call = fixtures::call(&argv);
        assert_eq!(fixtures::back(&call), fixtures::owned(&argv));
        if call.tool == rows::PROPOSE_SETTINGS_AT {
            *call.values.get_mut(0).expect("loaded FILE") =
                Some(Values::One(Value::Text(br#"{"items":[]}"#.as_slice().into())));
        }
        assert_eq!(fixtures::from_json(&jig::json::write_call(crate::table::TOOLS, &call).expect("JSON")), call);
    }
    assert_eq!((rows::PROPOSE_SETTINGS_AT, rows::PROPOSAL_AT), (48, 49));
    for argv in [
        vec!["settings", "propose", "FILE", "--against", "5"],
        vec!["settings", "propose", "FILE", "--against", "5", "--running", "moves", "--key", "proposal-5"],
        vec!["proposal", "show", "p73/7", "--key", "read"],
        vec!["proposal", "decide", "p73/7", "accept", "--running", "other"],
        vec!["task", "decide", "41", "--hold", "2", "release", "--running", "moves"],
    ] {
        assert!(fixtures::parse(&argv).is_err(), "{argv:?}");
    }
}
fn proposed(bytes: &[u8], json: bool) -> (fixtures::Harness, u32) {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut call = fixtures::call(&["settings", "propose", "FILE", "--against", "5", "--key", "proposal-5"]);
    *call.values.get_mut(0).expect("FILE") = Some(Values::One(Value::Text(bytes.into())));
    let number = root.submit_prepared(call, json, None);
    (root, number)
}
#[test]
fn propose_settings_uses_the_same_whole_items_and_names_its_durable_proposal() {
    for (setting, value) in values() {
        let file = file_from_setting(&setting_json(setting, value));
        for json in [false, true] {
            let (mut root, number) = proposed(&file, json);
            assert_eq!(root.sent_name(number), Some(fixtures::named("proposal-5")));
            let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&number).expect("sent") else { panic!("jig ask") };
            let c::Ask::Propose { project, action: c::ProposeAction::Settings(s), .. } = ask.as_ref() else {
                panic!("settings proposal")
            };
            assert_eq!(*project, 0);
            assert_eq!(s.against, 5);
            assert_eq!(s.items, settings_file::decode(&file, &crate::limits::tools()).expect("same decoder"));
            root.done(number, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 7, to: 0 }));
            let answer = root.answer(number).expect("durable proposal");
            assert_eq!(answer.status, 0);
            golden(if json { "settings_proposed_json" } else { "settings_proposed" }, &answer.stdout);
        }
    }
    let (mut root, number) = proposed(br#"{"items":[],"unknown":1}"#, false);
    assert!(root.sends.is_empty());
    assert_eq!(root.answer(number).expect("local refusal").status, 1);
}
fn review_proposal(review: Option<c::SettingsReview>) -> c::ReadValue {
    let c::ReadValue::Proposal(mut p) = fixtures::proposal(c::Proposer::Party(73), &[c::ActKind::Accept]) else {
        panic!("owner proposal")
    };
    p.key.number = 7;
    p.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 5,
        items: list([c::RevisionItemHead { setting: c::Setting::Rules, edit: c::RevisionItemEdit::Reset }]),
        review,
        made: None,
    }));
    c::ReadValue::Proposal(p)
}
#[test]
fn settings_review_goldens_keep_every_edit_value_origin_and_head_fact() {
    for (index, (setting, value)) in values().into_iter().enumerate() {
        let set = c::ReviewValue::Value(c::SettingSet { value });
        let review = c::SettingsReview {
            key: None,
            newest: 6,
            cut: true,
            items: list([
                c::ReviewItem {
                    setting,
                    edit: c::RevisionItemEdit::Set,
                    now: set.clone(),
                    origin: c::SettingOrigin::Change(c::OriginRevision { revision: 4 }),
                    would_be: set.clone(),
                    application: None,
                },
                c::ReviewItem {
                    setting,
                    edit: c::RevisionItemEdit::Removed,
                    now: set.clone(),
                    origin: c::SettingOrigin::Default(c::OriginRevision { revision: 2 }),
                    would_be: c::ReviewValue::None,
                    application: None,
                },
                c::ReviewItem {
                    setting,
                    edit: c::RevisionItemEdit::Reset,
                    now: c::ReviewValue::None,
                    origin: c::SettingOrigin::NotOffered,
                    would_be: set.clone(),
                    application: None,
                },
                c::ReviewItem {
                    setting,
                    edit: c::RevisionItemEdit::Reset,
                    now: set.clone(),
                    origin: c::SettingOrigin::Default(c::OriginRevision { revision: 2 }),
                    would_be: set.clone(),
                    application: None,
                },
                c::ReviewItem {
                    setting,
                    edit: c::RevisionItemEdit::Set,
                    now: c::ReviewValue::TooLarge(c::Size { bytes: 4096 }),
                    origin: c::SettingOrigin::Change(c::OriginRevision { revision: 3 }),
                    would_be: set.clone(),
                    application: None,
                },
                c::ReviewItem {
                    setting,
                    edit: c::RevisionItemEdit::Set,
                    now: set,
                    origin: c::SettingOrigin::Default(c::OriginRevision { revision: 0 }),
                    would_be: c::ReviewValue::TooLarge(c::Size { bytes: 8192 }),
                    application: None,
                },
            ]),
        };
        for json in [false, true] {
            let mut argv = vec!["proposal", "show", "p73/7"];
            if json {
                argv.push("--json");
            }
            let answer = read(&argv, review_proposal(Some(review.clone())));
            assert_eq!(answer.status, 0);
            golden(&format!("settings_review_{index}_{}", if json { "json" } else { "text" }), &answer.stdout);
        }
    }
    for review in [None, Some(c::SettingsReview { key: None, newest: 5, items: list([]), cut: false })] {
        let answer = read(&["proposal", "show", "p73/7"], review_proposal(review));
        assert_eq!(answer.status, 0);
        assert!(!core::str::from_utf8(&answer.stdout).expect("text").contains("cut:"));
    }
}
#[test]
fn proposal_reads_use_one_exact_read_and_keep_not_served_and_role_statuses() {
    let (root, call) = fixtures::read_call(&["proposal", "show", "p73/7"]);
    assert_eq!(
        root.reads.get(&call).expect("exact read").1,
        c::Read::Proposal { proposal: c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 } }
    );
    assert_eq!(root.watches.len(), 1);
    for (page, status) in [
        (jig_client::Readout::NotServed { family: None, served: None }, 1),
        (jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)), 2),
    ] {
        let (mut root, call) = fixtures::read_call(&["proposal", "show", "p73/7"]);
        root.serve_readout(call, page);
        assert_eq!(root.answer(call).expect("terminal read").status, status);
    }
}
#[test]
fn decide_sends_only_the_given_running_choice_and_leaves_malformed_to_the_engine() {
    for proposer in ["t41/17", "p73/17"] {
        for (choice, running) in
            [(None, None), (Some("stays"), Some(c::Running::Stays)), (Some("moves"), Some(c::Running::Moves))]
        {
            for decision in ["accept", "reject"] {
                let mut argv = vec!["proposal", "decide", proposer, decision];
                if decision == "reject" {
                    argv.push("because");
                }
                if let Some(choice) = choice {
                    argv.extend(["--running", choice]);
                }
                let (mut root, call) = fixtures::send_act(&argv, fixtures::SEED, fixtures::EMPTY.into());
                let c::Ask::Decide { on: c::DecideOn::Proposal { running: sent, .. }, .. } =
                    fixtures::act_ask(&root, call)
                else {
                    panic!("proposal decision")
                };
                assert_eq!(*sent, running);
                root.answered(
                    call,
                    c::Answer::Refused {
                        refusal: c::Refusal::Malformed { at: 0, why: c::Malformed::Running },
                        at: None,
                    },
                );
                let answer = root.answer(call).expect("malformed running");
                assert_eq!(answer.status, 2);
                assert!(core::str::from_utf8(&answer.stderr).expect("text").contains("running"));
            }
        }
    }
}

#[test]
fn a_withdrawn_settings_proposal_is_read_as_its_retained_state() {
    let key = c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 };
    let history = c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Proposal(key),
        value: c::EndedValue::Proposal(c::ProposalHistory {
            action: c::RetainedAction::Settings(c::RetainedActionSettings {
                against: 5,
                items: list([c::RevisionItemHead { setting: c::Setting::Rules, edit: c::RevisionItemEdit::Reset }]),
                made: None,
            }),
            number: key.number,
            proposer: 73,
            as_holder: false,
            state: c::ProposalHistoryState::Withdrawn,
        }),
        at: None,
        by: None,
    }));
    let text = read(&["proposal", "show", "p73/7"], history.clone());
    assert_eq!(text.status, 0);
    assert_eq!(text.stdout.as_ref(), b"proposal p73/7 \xc2\xb7 settings \xc2\xb7 withdrawn\nagainst revision 5\n");
    let json = read(&["proposal", "show", "p73/7", "--json"], history);
    assert_eq!(json.status, 0);
    assert_eq!(json.stdout.as_ref(), br#"{"key":{"proposer":{"party":73},"number":7},"action":{"settings":{"against":5,"items":[{"setting":{"kind":"rules"},"edit":"reset"}],"review":null,"made":null}},"state":{"withdrawn":{}}}"#);
}

#[test]
fn propose_settings_forwards_optional_reason_and_prints_the_original_returned_role() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut call =
        fixtures::call(&["settings", "propose", "FILE", "Please review", "--against", "5", "--key", "with-reason"]);
    *call.values.get_mut(0).expect("FILE") =
        Some(Values::One(Value::Text(file_from_setting(&setting_json(values()[0].0, values()[0].1.clone())).into())));
    let number = root.submit_prepared(call, true, None);
    let c::Ask::Propose { project, reason, .. } = fixtures::act_ask(&root, number) else { panic!("actual proposal") };
    assert_eq!(*project, 0);
    assert_eq!(reason.bytes(), b"Please review");
    root.done(number, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 7, to: 9 }));
    let printed = root.answer(number).expect("actual owner result");
    let json = core::str::from_utf8(&printed.stdout).expect("JSON");
    assert!(json.contains("\"to\":{\"role\":{\"project\":0,\"role\":9}}"));
    assert!(json.contains("\"proposed\":{\"proposal\":7,\"to\":9}"));
}

fn retained_proposal(key: c::ProposalKey, action: c::RetainedAction) -> c::ReadValue {
    let proposer = match key.proposer {
        c::Proposer::Task(n) | c::Proposer::Party(n) => n,
    };
    c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Proposal(key),
        value: c::EndedValue::Proposal(c::ProposalHistory {
            action,
            number: key.number,
            proposer,
            as_holder: false,
            state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(19) },
        }),
        at: None,
        by: None,
    }))
}
#[test]
fn retained_task_release_keeps_its_typed_identity_kind_and_decision_in_text_and_json() {
    let key = c::ProposalKey { proposer: c::Proposer::Task(73), number: 7 };
    let history = retained_proposal(key, c::RetainedAction::Release { task: 41 });
    let text = read(&["proposal", "show", "t73/7"], history.clone());
    assert_eq!(text.status, 0);
    assert_eq!(text.stdout.as_ref(), b"proposal t73/7 \xc2\xb7 release \xc2\xb7 accepted\n");
    let json = read(&["proposal", "show", "t73/7", "--json"], history);
    assert_eq!(json.status, 0);
    assert_eq!(
        json.stdout.as_ref(),
        br#"{"key":{"proposer":{"task":73},"number":7},"action":"release","state":{"accepted":{"by":{"person":19}}}}"#
    );
}
#[test]
fn retained_settings_preserves_task_or_party_keys_and_its_revision_details() {
    for (proposer, name) in [(c::Proposer::Task(73), "t73/7"), (c::Proposer::Party(73), "p73/7")] {
        let key = c::ProposalKey { proposer, number: 7 };
        let history = retained_proposal(
            key,
            c::RetainedAction::Settings(c::RetainedActionSettings { against: 5, items: list([]), made: Some(6) }),
        );
        for json in [false, true] {
            let argv = if json { vec!["proposal", "show", name, "--json"] } else { vec!["proposal", "show", name] };
            let answer = read(&argv, history.clone());
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).expect("text");
            if json {
                let expected = match proposer {
                    c::Proposer::Task(_) => r#""task":73"#,
                    c::Proposer::Party(_) => r#""party":73"#,
                };
                assert!(text.contains(expected));
                assert!(text.contains(r#""against":5"#));
                assert!(text.contains(r#""made":6"#));
            } else {
                assert!(text.contains(name));
                assert!(text.contains("against revision 5, made revision 6"));
            }
        }
    }
}
#[test]
fn retained_proposals_refuse_mismatched_inner_outer_and_requested_identity() {
    let key = c::ProposalKey { proposer: c::Proposer::Task(73), number: 7 };
    let good = retained_proposal(key, c::RetainedAction::Release { task: 41 });
    let c::ReadValue::Ended(original) = good else { panic!("ended") };
    for (name, key, proposer, number) in [
        ("t73/7", c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 }, 73, 7),
        ("t73/7", key, 74, 7),
        ("t73/7", key, 73, 8),
        ("t74/7", key, 73, 7),
    ] {
        let mut broken = original.clone();
        broken.key = c::ObjectKey::Proposal(key);
        let c::EndedValue::Proposal(h) = &mut broken.value else { panic!("history") };
        h.proposer = proposer;
        h.number = number;
        for json in [false, true] {
            let argv = if json { vec!["proposal", "show", name, "--json"] } else { vec!["proposal", "show", name] };
            assert_eq!(read(&argv, c::ReadValue::Ended(broken.clone())).status, 1);
        }
    }
}
fn retained_party_batch() -> c::ReadValue {
    let (_, _, task) = fixtures::task_read();
    c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 }),
        value: c::EndedValue::PersonProposal {
            proposal: Box::new(c::PersonProposal {
                number: 7,
                proposer: 73,
                project: 0,
                goal: c::PersonProposalGoal {
                    number: 41,
                    project: 0,
                    executor: c::Executor::Agent { charter: 3 },
                    spec: c::Specification {
                        asks: false,
                        words: words("goal"),
                        parameters: list([]),
                        inputs: list([]),
                    },
                    contract: c::TaskContractDocument::Report { words: 1024 },
                    authority: authority(),
                    numbers: c::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                    funder: c::Funder::Task(41),
                    dependencies: list([]),
                    holdings: list([]),
                    wake: task.wake,
                    tracked: None,
                },
                state: c::PersonProposalState::Pending { since: Wall::from_nanos(17) },
            }),
            decision: None,
        },
        at: None,
        by: None,
    }))
}
#[test]
fn retained_party_single_goal_batch_keeps_its_actual_kind_key_and_pending_state() {
    let history = retained_party_batch();
    let text = read(&["proposal", "show", "p73/7"], history.clone());
    assert_eq!(text.status, 0);
    assert_eq!(text.stdout.as_ref(), b"proposal p73/7 \xc2\xb7 batch \xc2\xb7 pending\n");
    let json = read(&["proposal", "show", "p73/7", "--json"], history);
    assert_eq!(json.status, 0);
    assert_eq!(
        json.stdout.as_ref(),
        br#"{"key":{"proposer":{"party":73},"number":7},"action":"batch","state":{"pending":{"since":17}}}"#
    );
}

#[test]
fn move_to_newest_parses_the_revision_and_requires_a_number_and_key() {
    let argv = ["revision", "advance", "7", "--key", "move-7"];
    let call = fixtures::call(&argv);
    assert_eq!(call.tool, rows::MOVE_TO_NEWEST_AT);
    assert_eq!(fixtures::back(&call), fixtures::owned(&argv));
    for argv in [
        vec!["revision", "advance", "--key", "move-7"],
        vec!["revision", "advance", "seven", "--key", "move-7"],
        vec!["revision", "advance", "4294967296", "--key", "move-7"],
        vec!["revision", "advance", "7"],
    ] {
        assert!(fixtures::parse(&argv).is_err(), "{argv:?}");
    }
}
#[test]
fn move_to_newest_sends_the_revision_and_renders_both_counts() {
    for (revision, newest, moved, stayed, text) in [
        (1, 1, 0, 0, "moved 0 tasks to revision 1\n"),
        (7, 9, 3, 1, "moved 3 tasks to revision 9; 1 could not, and finishes on 7\n"),
        (7, 9, 1, 2, "moved 1 task to revision 9; 2 could not, and finish on 7\n"),
    ] {
        for json in [false, true] {
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = fixtures::call(&["revision", "advance", &revision.to_string(), "--key", "move-7"]);
            let call = root.submit_prepared(call, json, None);
            let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("one keyed request") else {
                panic!("jig ask")
            };
            assert_eq!(ask.as_ref(), &c::Ask::MoveToNewest(c::MoveToNewest { revision }));
            assert_eq!(root.sent_name(call), Some(fixtures::named("move-7")));
            root.done(call, c::Outcome::MovedToNewest(c::OutcomeMovedToNewest { newest, moved, stayed }));
            let printed = root.answer(call).expect("move answer");
            assert_eq!(printed.status, 0);
            let expected = if json {
                format!("{{\"newest\":{newest},\"moved\":{moved},\"stayed\":{stayed}}}")
            } else {
                text.into()
            };
            assert_eq!(printed.stdout.as_ref(), expected.as_bytes());
        }
    }
}

#[test]
fn settings_refusals_propose_the_called_tool_without_a_retry_call() {
    let file = br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    for (propose, tool, operation) in
        [(false, "settings apply", "change-settings"), (true, "settings propose", "propose_settings")]
    {
        for json in [false, true] {
            let (mut root, call) = if propose { proposed(file, json) } else { changed(file, json) };
            root.answered(
                call,
                c::Answer::Refused {
                    refusal: c::Refusal::Authority { lacking: c::Lacking::Approver, proposable: Some(true) },
                    at: None,
                },
            );
            let printed = root.answer(call).expect("authority refusal");
            assert_eq!(printed.status, 2);
            if json {
                let text = core::str::from_utf8(&printed.stderr).expect("JSON");
                assert!(text.contains(&format!(r#""operation":"{operation}""#)));
                assert!(text.contains(&format!(r#""propose_instead":"{tool}""#)));
            } else {
                assert_eq!(
                    printed.stderr.as_ref(),
                    format!("{tool}: Authority missing\npropose instead: {tool}\n").as_bytes()
                );
            }
        }
    }
}

#[test]
fn proposal_review_reads_both_large_values_whole_and_keeps_refusals_in_place() {
    for json in [false, true] {
        for refused in [false, true] {
            let (setting, value) = values().remove(0);
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&["proposal", "show", "p73/7"]), json, None);
            let large = c::ReviewValue::TooLarge(c::Size { bytes: 8192 });
            root.serve(
                call,
                c::ReadResult::Value(review_proposal(Some(c::SettingsReview {
                    key: None,
                    newest: 6,
                    cut: false,
                    items: list([c::ReviewItem {
                        setting,
                        edit: c::RevisionItemEdit::Set,
                        now: large.clone(),
                        would_be: large,
                        origin: c::SettingOrigin::Default(c::OriginRevision { revision: 1 }),
                        application: None,
                    }]),
                }))),
            );
            assert_eq!(
                root.reads.get(&call).expect("now read").1,
                c::Read::Setting(c::ReadSetting { setting, revision: 6, proposal: None })
            );
            assert!(root.answer(call).is_none());
            root.serve(
                call,
                c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                    setting,
                    revision: 6,
                    value: Some(value.clone()),
                    application: None,
                    proposed: None,
                }))),
            );
            assert_eq!(
                root.reads.get(&call).expect("proposed read").1,
                c::Read::Setting(c::ReadSetting {
                    setting,
                    revision: 0,
                    proposal: Some(c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 }),
                })
            );
            if refused {
                root.serve(call, c::ReadResult::None(c::ReadNone::Refused));
            } else {
                root.serve(
                    call,
                    c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                        setting,
                        revision: 6,
                        value: Some(value),
                        application: None,
                        proposed: None,
                    }))),
                );
            }
            let answer = root.answer(call).expect("whole review");
            assert_eq!(answer.status, 0);
            let output = core::str::from_utf8(&answer.stdout).expect("text or JSON");
            assert!(output.contains("boundaries"));
            assert!(!output.contains("too large"));
            assert!(!output.contains("too_large"));
            if refused {
                assert!(output.contains("refused"));
            } else {
                assert_eq!(output.matches("boundaries").count(), 2);
            }
        }
    }
}

#[test]
fn settings_acceptance_defaults_to_stays_and_preserves_other_decisions() {
    for json in [false, true] {
        for settings in [false, true] {
            for (decision, extra, expected) in [
                ("accept", &[][..], if settings { Some(c::Running::Stays) } else { None }),
                ("accept", &["--running", "stays"][..], Some(c::Running::Stays)),
                ("accept", &["--running", "moves"][..], Some(c::Running::Moves)),
                ("reject", &[][..], None),
                ("pass-up", &[][..], None),
            ] {
                let mut root = fixtures::signed_in(fixtures::PARTY);
                let mut argv = vec!["proposal", "decide", "p73/7", decision, "--key", "decision"];
                argv.extend(extra);
                let call = root.submit_prepared(fixtures::call(&argv), json, None);
                let mut document = review_proposal(None);
                let c::ReadValue::Proposal(proposal) = &mut document else { unreachable!() };
                proposal.acts = fixtures::offered(&[c::ActKind::Accept, c::ActKind::Reject, c::ActKind::PassUp]).acts;
                if !settings {
                    proposal.action = c::ProposalAction::Batch(list([]));
                }
                root.serve(call, c::ReadResult::Value(document));
                let c::Ask::Decide { on: c::DecideOn::Proposal { running, .. }, .. } = fixtures::act_ask(&root, call)
                else {
                    panic!("proposal decision")
                };
                assert_eq!(*running, expected, "settings {settings}, {argv:?}, json {json}");
                root.done(call, c::Outcome::Document(c::DocumentOutcome::Amended { task: 9 }));
                assert_eq!(root.answer(call).expect("decision answered").status, 0);
            }
        }
    }
}

#[test]
fn repeated_settings_decisions_send_the_given_key_and_print_the_engine_answer() {
    let proposal = c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 };
    for json in [false, true] {
        for accepted in [false, true] {
            for (key, accept) in [("first", accepted), ("new-accept", true), ("new-reject", false)] {
                let mut root = fixtures::signed_in(fixtures::PARTY);
                let mut argv = vec!["proposal", "decide", "p73/7", if accept { "accept" } else { "reject" }];
                if !accept {
                    argv.push("because");
                }
                argv.extend(["--key", key]);
                let call = root.submit_prepared(fixtures::call(&argv), json, None);
                let mut history = retained_proposal(
                    proposal,
                    c::RetainedAction::Settings(c::RetainedActionSettings {
                        against: 5,
                        items: list([]),
                        made: if accepted { Some(6) } else { None },
                    }),
                );
                if !accepted {
                    let c::ReadValue::Ended(history) = &mut history else { unreachable!() };
                    let c::EndedValue::Proposal(proposal) = &mut history.value else { unreachable!() };
                    proposal.state = c::ProposalHistoryState::Rejected {
                        by: c::Requester::Person(19),
                        reason: c::DocumentText::complete(words("first reason")),
                    };
                }
                root.serve(call, c::ReadResult::Value(history));
                assert_eq!(root.sent_name(call), Some(fixtures::named(key)));
                let c::Ask::Decide { project, on: c::DecideOn::Proposal { proposal: sent, running, .. } } =
                    fixtures::act_ask(&root, call)
                else {
                    panic!("proposal decision")
                };
                assert_eq!((*project, *sent), (0, proposal));
                assert_eq!(*running, if accept { Some(c::Running::Stays) } else { None });
                root.done(
                    call,
                    c::Outcome::Document(c::DocumentOutcome::ProposalDecided {
                        proposal,
                        by: 19,
                        at: fixtures::DECIDED_AT,
                        choice: if accepted { c::ProposalChoice::Accepted } else { c::ProposalChoice::Rejected },
                        repeated: key != "first",
                    }),
                );
                let answer = root.answer(call).expect("first decision answer");
                assert_eq!(answer.status, 0);
                let output = core::str::from_utf8(&answer.stdout).expect("text or JSON");
                if json {
                    assert!(output.contains("\"operation\":\"decide\""), "{output}");
                    assert!(
                        output.contains(if key == "first" { "\"repeated\":false" } else { "\"repeated\":true" }),
                        "{output}"
                    );
                } else if key == "first" {
                    assert_eq!(output, "done: decide\n");
                } else {
                    assert!(output.starts_with("decided first: by party 19,"));
                    assert!(output.contains(if accepted { "accepted" } else { "rejected" }));
                }
            }
        }
    }
}

fn forge_value(kind: u32) -> Box<[u8]> {
    let json = if kind == 3 {
        br#"{"items":[{"setting":{"kind":"forge-charter","number":1},"set":6}]}"#.as_slice()
    } else {
        br#"{"items":[{"setting":{"kind":"landing-rule","number":1},"set":{"pattern":{"segments":["ff"],"last":{"open":"61"}},"enforce":true,"ci":false,"up_to_date":true,"gates":[{"number":2,"blocking":false,"freshness":"clean"}],"approvals":[{"role":0,"people":1,"freshness":"exact"}]}}]}"#.as_slice()
    };
    let items = settings_file::decode(json, &fixtures::limits()).unwrap();
    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(bytes) }) = &items.as_slice()[0].edit
    else {
        panic!("forge value")
    };
    bytes.clone()
}
fn forge_setting_page(value: Option<Box<[u8]>>) -> jig_client::Readout {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    jig_client::Readout::Family {
        family: wire::FORGE,
        record: Some(wire::setting_answer(
            &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value }).unwrap(),
        )),
    }
}
fn forge_query(root: &fixtures::Harness, call: u32) -> temper_client_protocol_wire::v1::Read {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let (_, jig_client::Readable::Family { scope, record, .. }) = root.family_reads.get(&call).expect("forge read")
    else {
        panic!("family read")
    };
    assert_eq!(*scope, c::Scope::Deployment);
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().into_parts().read
}
fn forge_inventory(setting: c::Setting, follows: c::ApplicationFollows) -> c::ReadResult {
    c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
        newest: 5,
        settings: list([c::SettingLine {
            setting,
            name: None,
            follows: if follows == c::ApplicationFollows::Default {
                c::SettingFollows::Default
            } else {
                c::SettingFollows::Change
            },
            last: if follows == c::ApplicationFollows::Default { None } else { Some(5) },
            application: Some(c::ApplicationState { follows, size: 4, charters: list([]) }),
        }]),
        held: list([]),
        pending: list([]),
    })))
}
#[test]
fn forge_setting_commands_join_inventory_state_and_the_forges_value_in_text_and_json() {
    use temper_client_protocol_wire::v1 as w;
    for (argv, kind, scope, label) in [
        (vec!["settings", "show", "landing-rule", "1"], 1, 0, "landing-rule 1"),
        (vec!["settings", "show", "landing-rule", "1", "--project", "7"], 2, 7, "landing-rule 1 --project 7"),
        (
            vec!["settings", "show", "landing-rule", "1", "--project", "shop"],
            2,
            fixtures::PROJECT,
            "landing-rule 1 --project",
        ),
        (vec!["settings", "show", "forge-charter", "1"], 3, 0, "forge-charter 1"),
    ] {
        for json in [false, true] {
            let setting = c::Setting::Application { kind, scope, number: 1 };
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&argv), json, None);
            assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Settings);
            root.serve(call, forge_inventory(setting, c::ApplicationFollows::Changed { revision: 5 }));
            assert!(root.answer(call).is_none());
            let w::Read::Setting(query) = forge_query(&root, call) else { panic!("setting query") };
            assert_eq!((query.kind(), query.scope(), query.number()), (kind, scope, 1));
            root.serve_readout(call, forge_setting_page(Some(forge_value(kind))));
            let answer = root.answer(call).expect("joined setting");
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            if json {
                assert!(text.contains(if kind == 3 { "\"value\":6" } else { "\"ci\":false" }), "{text}");
                assert!(text.contains("\"application\":{\"follows\":\"change\",\"revision\":5"), "{text}");
                assert!(text.contains(if kind == 3 { "forge-charter" } else { "landing-rule" }), "{text}");
            } else {
                let canonical = setting_json(setting, c::SettingValue::Application(forge_value(kind)));
                let canonical = core::str::from_utf8(&canonical).unwrap();
                let (_, value) = canonical.split_once("\"value\":").unwrap();
                let value = value.trim_end().strip_suffix('}').unwrap();
                assert!(text.contains(value), "inline value differs from the shared settings form: {text}");
                assert!(text.starts_with(label), "{text}");
                assert!(text.contains("changed at revision 5"), "{text}");
                assert!(text.contains(if kind == 3 { "\n6\n" } else { "\"ci\":false" }), "{text}");
            }
            assert_eq!(root.watches.len(), 1);
        }
    }
}
#[test]
fn forge_setting_commands_print_removed_and_unset_values_and_preserve_refusals() {
    for json in [false, true] {
        for follows in [c::ApplicationFollows::Default, c::ApplicationFollows::Removed { revision: 5 }] {
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&["settings", "show", "forge-charter", "2"]), json, None);
            root.serve(call, forge_inventory(c::Setting::Application { kind: 3, scope: 0, number: 2 }, follows));
            root.serve_readout(call, forge_setting_page(None));
            let answer = root.answer(call).unwrap();
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            assert!(text.contains(if json { "\"value\":null" } else { "\nnone\n" }), "{text}");
        }
        for page in [
            jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
            jig_client::Readout::NotServed { family: Some(1), served: None },
            jig_client::Readout::Family { family: 1, record: Some(Box::new([0])) },
        ] {
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&["settings", "show", "landing-rule", "1"]), json, None);
            root.serve(
                call,
                forge_inventory(
                    c::Setting::Application { kind: 1, scope: 0, number: 1 },
                    c::ApplicationFollows::Default,
                ),
            );
            root.serve_readout(call, page);
            assert_ne!(root.answer(call).unwrap().status, 0);
        }
    }
}
#[test]
fn forge_settings_inventory_names_the_kinds_with_defaults_changes_and_removals() {
    let settings = [
        (1, 0, 1, c::ApplicationFollows::Default),
        (2, 7, 2, c::ApplicationFollows::Changed { revision: 5 }),
        (3, 0, 1, c::ApplicationFollows::Removed { revision: 5 }),
    ];
    let mut lines = List::with_capacity(3);
    for (kind, scope, number, follows) in settings {
        let c::ReadResult::Value(c::ReadValue::Settings(inventory)) =
            forge_inventory(c::Setting::Application { kind, scope, number }, follows)
        else {
            unreachable!()
        };
        lines.push(inventory.settings.as_slice()[0].clone()).unwrap();
    }
    let document = c::ReadValue::Settings(Box::new(c::SettingsDocument {
        newest: 5,
        settings: lines,
        held: list([]),
        pending: list([]),
    }));
    let answer = read(&["settings", "show"], document);
    assert_eq!(answer.status, 0);
    let text = core::str::from_utf8(&answer.stdout).unwrap();
    for line in [
        "landing-rule 1: default",
        "landing-rule 2 --project 7: changed in revision 5",
        "forge-charter 1: changed in revision 5, removed",
    ] {
        assert!(text.contains(line), "{text}");
    }
}
#[test]
fn forge_proposal_show_reads_the_reviews_key_and_prints_every_temper_edit() {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    for json in [false, true] {
        let mut root = fixtures::signed_in(fixtures::PARTY);
        let call = root.submit_prepared(fixtures::call(&["proposal", "show", "p73/7"]), json, None);
        root.serve(
            call,
            c::ReadResult::Value(review_proposal(Some(c::SettingsReview {
                key: Some(u64::MAX),
                newest: 5,
                items: list([c::ReviewItem {
                    setting: c::Setting::Application { kind: 1, scope: 0, number: 1 },
                    edit: c::RevisionItemEdit::Set,
                    now: c::ReviewValue::None,
                    would_be: c::ReviewValue::None,
                    origin: c::SettingOrigin::Default(c::OriginRevision { revision: 1 }),
                    application: None,
                }]),
                cut: true,
            }))),
        );
        let w::Read::ProposalItems(query) = forge_query(&root, call) else { panic!("proposal items") };
        assert_eq!(query.key(), u64::MAX, "the review's key, not proposer 73's number 7");
        let record = w::ProposalItemsAnswer::new(
            &w::CEILINGS,
            w::ProposalItemsAnswerParts {
                items: list([
                    w::ProposalItem::new(
                        &w::CEILINGS,
                        w::ProposalItemParts {
                            kind: 1,
                            scope: 0,
                            number: 1,
                            edit: w::SettingEdit::Set(
                                w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: forge_value(1) })
                                    .unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                    w::ProposalItem::new(
                        &w::CEILINGS,
                        w::ProposalItemParts { kind: 2, scope: 7, number: 2, edit: w::SettingEdit::Removed },
                    )
                    .unwrap(),
                    w::ProposalItem::new(
                        &w::CEILINGS,
                        w::ProposalItemParts { kind: 3, scope: 0, number: 1, edit: w::SettingEdit::Reset },
                    )
                    .unwrap(),
                ]),
            },
        )
        .unwrap();
        root.serve_readout(
            call,
            jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::proposal_items_answer(&record)) },
        );
        for (kind, scope, number) in [(1, 0, 1), (2, 7, 2), (3, 0, 1)] {
            let w::Read::Setting(query) = forge_query(&root, call) else { panic!("current setting") };
            assert_eq!((query.kind(), query.scope(), query.number()), (kind, scope, number));
            root.serve_readout(call, forge_setting_page(Some(forge_value(kind))));
        }
        let answer = root.answer(call).unwrap();
        assert_eq!(answer.status, 0);
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        for needle in ["landing-rule", "forge-charter", "\"ci\":false", "removed", "reset"] {
            assert!(text.contains(needle), "{text}");
        }
        assert!(!text.contains("too large"), "{text}");
        if json {
            assert!(text.contains("\"application_items\":["), "{text}");
        }
    }
}

#[test]
fn forge_setting_commands_refuse_selectors_outside_their_forms() {
    for argv in [
        vec!["settings", "show", "landing-rule", "0"],
        vec!["settings", "show", "landing-rule", "1", "--revision", "2"],
        vec!["settings", "show", "forge-charter", "0"],
        vec!["settings", "show", "forge-charter", "3"],
        vec!["settings", "show", "forge-charter", "1", "--project", "1"],
    ] {
        assert!(fixtures::parse(&argv).is_err(), "{argv:?}");
    }
}
