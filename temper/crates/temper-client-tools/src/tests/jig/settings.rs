#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust fixtures and golden comparisons"
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
        provider: 1,
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
            connector: 1,
            name: list(["forge", "code.example", "team", "rules", "branch", "main"].map(|s| s.as_bytes().into())),
            path: b"AGENTS.md".as_slice().into(),
            heading: Some(b"Rules".as_slice().into()),
        },
        c::CharterDocument {
            connector: 1,
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
const PARTS: &str = r#"[{"on":["first"],"instructions":"instructions for the first run only"},{"on":["question_owed","question"],"instructions":"instructions for questions"}]"#;
fn parts() -> List<c::CharterPart> {
    list([
        c::CharterPart { on: 1, instructions: words("instructions for the first run only") },
        c::CharterPart { on: (1 << 1) | (1 << 4), instructions: words("instructions for questions") },
    ])
}
fn with_parts(file: &[u8], parts: &str, resume: bool) -> Vec<u8> {
    String::from_utf8(file.to_vec())
        .expect("file JSON")
        .replace(r#""documents":["#, &format!(r#""parts":{parts},"documents":["#))
        .replace(r#""resume":true"#, if resume { r#""resume":true"# } else { r#""resume":false"# })
        .into_bytes()
}
const DOCUMENTS: &str = r#"[{"forge":"code.example","repository":"team/rules","branch":"main","path":"AGENTS.md","heading":"Rules"},{"forge":"code.example","repository":"team/rules","branch":"topic/rules","path":"docs/rules.md"}]"#;

#[test]
fn settings_reason_names_keep_jigs_bit_order_and_always_offered_names() {
    assert_eq!(settings_form::view::REASON_NAMES.len(), jig_client_domain::REASON_NAMES.len());
    for (form, client) in settings_form::view::REASON_NAMES.iter().zip(jig_client_domain::REASON_NAMES) {
        assert_eq!(*form, client.as_bytes());
    }
    assert_eq!(settings_form::view::ALWAYS_OFFERED_NAMES, jig_domain_tasks::ALWAYS_OFFERED_NAMES);
}
#[test]
fn a_settings_file_charter_names_documents() {
    let file = with_parts(&charter_file(DOCUMENTS), PARTS, false);
    settings_file::decode(&file, &crate::limits::tools()).expect("parts and documents decode");
    let (root, call) = changed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") };
    let (setting, c::SettingValue::Charter(mut charter)) = values().remove(0) else { panic!("charter") };
    charter.parts = parts();
    charter.resume = false;
    let value = c::SettingValue::Charter(charter);
    let json = setting_json(setting, value.clone());
    assert!(core::str::from_utf8(&json).expect("JSON").contains(r#""on":["question","question_owed"]"#));
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
    let mut cases = vec![(setting, value, "settings_value_0", "")];
    let (setting, c::SettingValue::Charter(mut charter)) = values().remove(0) else { panic!("charter") };
    charter.parts = parts();
    charter.resume = false;
    cases.push((
        setting,
        c::SettingValue::Charter(charter),
        "settings_charter_parts",
        "parts[0]: on first; 35 bytes\nparts[1]: on question question_owed; 26 bytes\n",
    ));
    for (effort, name, suffix) in [
        (Some("high"), "settings_template_later", "later: claude-opus-5-5 high\n"),
        (None, "settings_template_later_no_effort", "later: claude-opus-5-5\n"),
    ] {
        let (setting, c::SettingValue::Template(mut template)) = values().remove(1) else { panic!("template") };
        template.llm.as_mut().expect("choice").later = Some(c::LlmLater {
            main: c::ModelEffort { model: words("claude-opus-5-5"), effort: effort.map(words) },
            subagents: list([c::ModelEffort { model: words("later-agent"), effort: None }]),
        });
        cases.push((setting, c::SettingValue::Template(template), name, suffix));
    }
    for (setting, value, name, suffix) in cases {
        for json in [false, true] {
            let mut argv = match setting {
                c::Setting::Charter(_) => vec!["settings", "show", "charter", "3"],
                c::Setting::Template(_) => vec!["settings", "show", "template", "4"],
                c::Setting::Policy(_)
                | c::Setting::Rules
                | c::Setting::Model(_)
                | c::Setting::Application { .. }
                | c::Setting::ProjectPolicy { .. } => panic!("charter or template"),
            };
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
            if !json {
                let text = core::str::from_utf8(&answer.stdout).expect("show text");
                assert!(text.ends_with(suffix), "{name}: expected suffix {suffix:?}, got {text:?}");
            }
            golden(&if json { format!("{name}_json") } else { name.to_owned() }, &answer.stdout);
        }
    }
}
#[test]
fn a_document_that_is_no_forge_branch_renders_its_name() {
    for (connector, name) in [
        (7, vec!["forge", "code.example", "team", "rules", "branch", "main"]),
        (0, vec!["forge", "code.example", "team", "rules", "branch", "main"]),
        (1, vec!["other", "resource"]),
        (1, vec!["forge", "code.example", "team", "rules", "branch"]),
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
    let file = with_parts(&charter_file(DOCUMENTS), PARTS, false);
    for (name, length, path) in [
        ("charter_parts", 2, "charter.parts"),
        ("charter_instructions", 35, "charter.parts[0].instructions"),
        ("charter_part_instructions", 35, "charter.parts[0].instructions"),
        ("charter_documents", 2, "charter.documents"),
        ("charter_document_name", 7, "charter.documents[1].branch"),
        ("charter_document_name_item", 12, "charter.documents[0].forge"),
        ("charter_document_path", 13, "charter.documents[1].path"),
        ("charter_document_heading_some", 5, "charter.documents[0].heading"),
    ] {
        for (bound, allowed) in [(length, true), (length - 1, false)] {
            let mut limits = crate::limits::tools();
            match name {
                "charter_parts" => limits.settings.charter_parts = bound,
                "charter_instructions" => limits.settings.charter_instructions = bound,
                "charter_part_instructions" => limits.settings.charter_part_instructions = bound,
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
                if name == "charter_parts" {
                    assert!(text.contains("tasks.charter_parts"), "{text}");
                }
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
    for (parts, resume, path, why) in [
        (r#"[{"on":["unknown"],"instructions":"x"}]"#, false, "charter.parts[0].on[0]", r#""unknown" is no reason"#),
        (
            r#"[{"on":["answer"],"instructions":"x"}]"#,
            false,
            "charter.parts[0].on[0]",
            r#""answer" goes to every run and chooses no part"#,
        ),
        (
            r#"[{"on":["question","question"],"instructions":"x"}]"#,
            false,
            "charter.parts[0].on[1]",
            r#""question" twice"#,
        ),
        (r#"[{"on":[],"instructions":"x"}]"#, false, "charter.parts[0].on", "names no kind"),
        (PARTS, true, "charter.parts", "a charter that resumes takes no parts"),
        ("null", false, "charter.parts", "expected array"),
        (r#"[{"instructions":"x"}]"#, false, "charter.parts[0].on", "missing member"),
        (r#"[{"on":["first"]}]"#, false, "charter.parts[0].instructions", "missing member"),
    ] {
        let file = with_parts(&charter_file(DOCUMENTS), parts, resume);
        let error = settings_file::decode(&file, &crate::limits::tools()).expect_err("invalid part");
        let error = core::str::from_utf8(&error).expect("error text");
        assert_eq!(error, format!("settings file item 1: {path}: {why}"));
        let (mut root, call) = changed(&file, false);
        assert!(root.sends.is_empty());
        assert_eq!(root.answer(call).expect("local refusal").status, 1);
    }
    for (later, path, why) in [
        ("null", "llm.later", "expected object"),
        ("{}", "llm.later.main", "missing member"),
        (r#"{"main":{"effort":null}}"#, "llm.later.main.model", "missing member"),
        (r#"{"main":{"model":7}}"#, "llm.later.main.model", "expected string"),
        (r#"{"main":{"model":"m","effort":7}}"#, "llm.later.main.effort", "expected string"),
        (r#"{"main":{"model":"m","effort":null},"subagents":null}"#, "llm.later.subagents", "expected array"),
        (r#"{"main":{"model":"m","effort":null},"subagents":[{}]}"#, "llm.later.subagents[0].model", "missing member"),
        (r#"{"main":{"model":"m","effort":null},"account":1}"#, "llm.later.account", "unknown member"),
    ] {
        let (setting, value) = values().remove(1);
        let file = String::from_utf8(file_from_setting(&setting_json(setting, value)))
            .expect("JSON")
            .replace(r#""llm":{"#, &format!(r#""llm":{{"later":{later},"#));
        let error = settings_file::decode(file.as_bytes(), &crate::limits::tools()).expect_err("invalid later");
        assert_eq!(core::str::from_utf8(&error).expect("error text"), format!("settings file item 1: {path}: {why}"));
        let (mut root, call) = changed(file.as_bytes(), false);
        assert!(root.sends.is_empty());
        assert_eq!(root.answer(call).expect("local refusal").status, 1);
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

#[expect(clippy::too_many_lines, reason = "the shared fixture lists every field of the four setting kinds")]
fn values() -> Vec<(c::Setting, c::SettingValue)> {
    vec![
        (
            c::Setting::Charter(c::SettingNumber { number: 3 }),
            c::SettingValue::Charter(Box::new(c::Charter {
                parts: List::with_capacity(0),
                rest: false,
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
                tools: 1536,
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
                    later: None,
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
    serve_review(&mut root, call, value);
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
    while let Some((_, c::Read::Revision(_))) = root.reads.get(&call) {
        root.serve(call, c::ReadResult::Value(c::ReadValue::Revision(Box::new(c::RevisionAnswer::Unknown))));
    }
    while root.family_reads.contains_key(&call) {
        let temper_client_protocol_wire::v1::Read::RevisionSetting(query) = forge_query(&root, call) else {
            panic!("revision value")
        };
        root.serve_readout(call, forge_setting_page(Some(forge_value(query.kind()))));
    }
    root.answer(call).expect("read completed")
}
fn serve_review(root: &mut fixtures::Harness, call: u32, value: c::ReadValue) {
    root.serve(call, c::ReadResult::Value(value));
    if let Some((_, c::Read::Settings)) = root.reads.get(&call) {
        newest_for_setting(root, call);
    }
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
    let text = match text.split_once(",\"application\":") {
        Some((value, _)) => format!("{value}}}"),
        None => text.to_owned(),
    };
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
    if temper_test_env::rewrite() {
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
    let mut cases = values();
    let (setting, c::SettingValue::Charter(mut charter)) = values().remove(0) else { panic!("charter") };
    charter.parts = parts();
    charter.resume = false;
    cases.push((setting, c::SettingValue::Charter(charter)));
    let (setting, c::SettingValue::Template(mut template)) = values().remove(1) else { panic!("template") };
    template.llm.as_mut().expect("choice").later = Some(c::LlmLater {
        main: c::ModelEffort { model: words("later-model"), effort: Some(words("medium")) },
        subagents: list([c::ModelEffort { model: words("later-agent"), effort: None }]),
    });
    cases.push((setting, c::SettingValue::Template(template)));
    for (setting, value) in cases {
        let bounds = crate::limits::form();
        let wire = jig_client_protocol_wire::SettingDocument::new(
            &bounds.settings,
            jig_client_protocol_wire::SettingDocumentParts {
                setting: jig_client_protocol_link::translate::settings::wire_setting(&setting, &bounds.settings)
                    .expect("wire selector"),
                revision: 5,
                value: Some(
                    jig_client_protocol_link::translate::settings::wire_setting_value(&value, &bounds.settings)
                        .expect("wire value"),
                ),
                application: None,
                proposed: None,
            },
        )
        .expect("wire document");
        assert_eq!(
            settings_form::wire::restore_setting_document(&wire, &bounds).expect("restore value").value,
            Some(jig::form_setting_value(&value))
        );
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
                replaced: c::RevisionReplaced::NotOffered,
                setting: c::Setting::Charter(c::SettingNumber { number: 3 }),
                edit: c::RevisionEdit::TooLarge(c::Size { bytes: 1_000_000 }),
            },
            c::RevisionItem {
                replaced: c::RevisionReplaced::NotOffered,
                setting: c::Setting::Rules,
                edit: c::RevisionEdit::Reset,
            },
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
            c::Refusal::Malformed { at: 1, why: super::malformed_setting(c::Setting::Rules, c::SettingFault::Owner) },
            "out of shape; item 1; rules: owner",
        ),
        (
            c::Refusal::Full(c::RefusalFull { revision: 2, tasks: list([41, 42]), more: 3 }),
            "full; oldest held revision 2; live tasks: 41 42; 3 more",
        ),
        (
            c::Refusal::Malformed {
                at: 1,
                why: c::Malformed::Settings(c::MalformedSettings {
                    setting: c::Setting::Charter(c::SettingNumber { number: 5 }),
                    fault: c::SettingFault::Shape,
                    detail: Some(c::SettingDetail {
                        member: c::SettingMember::StoredPolicy,
                        document: 0,
                        segment: 0,
                        rule: c::SettingRule::Maximum,
                        supplied: 70112,
                        bound: 65536,
                    }),
                }),
            },
            "out of shape; item 1; charter 5: shape; stored policy: supplied 70112; maximum root.journal.run_bytes 65536",
        ),
        (c::Refusal::Role { held: None }, "role"),
    ] {
        for json in [false, true] {
            let (mut root, call) = changed(file, json);
            root.answered(call, c::Answer::Refused { refusal: refusal.clone(), at: None });
            let answer = root.answer(call).expect("refused");
            assert_eq!(answer.status, 2);
            if json
                && let c::Refusal::Malformed { why: c::Malformed::Settings(value), .. } = &refusal
                && value.detail.is_some()
            {
                let text = core::str::from_utf8(&answer.stderr).expect("JSON refusal");
                assert!(text.contains(r#""detail":{"member":"stored_policy","rule":"maximum","document":0,"segment":0,"supplied":70112,"bound":65536}"#), "{text}");
            }
            if !json && expected != "role" {
                let text = core::str::from_utf8(&answer.stderr).expect("text");
                assert!(text.contains(expected), "expected {expected:?}, got {text:?}");
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
        (2, "delegation_kinds", 16),
        (2, "pattern_segments", 2),
        (2, "pattern_segments_item", 3),
        (2, "pattern_end_open_value", 4),
        (2, "model_set_names_names", 1),
        (2, "model_set_names_names_item", 5),
        (3, "rules_implies", 1),
    ] {
        let (setting, mut value) = values().remove(index);
        if name == "delegation_kinds" {
            let c::SettingValue::Policy(policy) = &mut value else { panic!("policy") };
            policy.ceiling.delegation.kinds = list((1..=16).map(c::AuthorityExecutor::Charter));
        }
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
            c::Setting::Application { .. } | c::Setting::ProjectPolicy { .. } => continue,
            c::Setting::Model(_) => vec!["settings", "show", "model", "9"],
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
                        replaced: c::RevisionReplaced::NotOffered,
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
                    replaced: c::RevisionReplaced::NotOffered,
                    setting,
                    edit: c::RevisionEdit::TooLarge(c::Size { bytes: 1_000_000 }),
                }]),
            }))),
        );
        let line = format!(
            "  {name}: too large, 1000000 bytes; read current value with settings show {name}\n    was: not offered\n"
        );
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
fn deployment_settings_proposal_names_its_owners() {
    let file = charter_file(DOCUMENTS);
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut call = fixtures::call(&["settings", "propose", "FILE", "Please review", "--against", "5", "--key", "k1"]);
    *call.values.get_mut(0).expect("FILE") = Some(Values::One(Value::Text(file.clone().into())));
    let number = root.submit_prepared(call, false, None);
    let c::Ask::Propose { project, reason, action: c::ProposeAction::Settings(settings) } =
        fixtures::act_ask(&root, number)
    else {
        panic!("settings proposal")
    };
    assert_eq!(*project, 0);
    assert_eq!(reason.bytes(), b"Please review");
    assert_eq!(settings.against, 5);
    assert_eq!(settings.items, settings_file::decode(&file, &crate::limits::tools()).expect("settings items"));
    assert_eq!(root.sent_name(number), Some(fixtures::named("k1")));
    root.done(number, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 7, to: 0 }));
    let answer = root.answer(number).expect("proposal outcome");
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), b"proposed: p73/7, to the deployment's owners\n");
}

#[test]
fn scoped_settings_proposal_names_every_required_owner() {
    for (selectors, recipient) in [
        (
            vec![
                r#"{"kind":"template","number":1}"#,
                r#"{"kind":"policy","number":1}"#,
                r#"{"kind":"rules"}"#,
                r#"{"kind":"landing-rule","number":1}"#,
                r#"{"kind":"forge-charter","number":1}"#,
            ],
            "the deployment's owners",
        ),
        (vec![r#"{"kind":"landing-rule","project":2,"number":1}"#], "project 2's owners"),
        (vec![r#"{"kind":"project-policy","project":2}"#], "project 2's owners"),
        (
            vec![
                r#"{"kind":"landing-rule","project":2,"number":1}"#,
                r#"{"kind":"landing-rule","project":2,"number":2}"#,
            ],
            "project 2's owners",
        ),
        (
            vec![
                r#"{"kind":"landing-rule","project":2,"number":1}"#,
                r#"{"kind":"landing-rule","project":1,"number":1}"#,
            ],
            "project 1's owners and project 2's owners; one person must own every affected scope",
        ),
        (
            vec![r#"{"kind":"charter","number":1}"#, r#"{"kind":"landing-rule","project":1,"number":1}"#],
            "the deployment's owners and project 1's owners; one person must own every affected scope",
        ),
    ] {
        assert_settings_recipient(&selectors, recipient);
    }
}

fn assert_settings_recipient(selectors: &[&str], recipient: &str) {
    let items: Vec<_> = selectors
        .iter()
        .map(|selector| {
            if selector.contains("project-policy") {
                let policy = setting_json(c::Setting::Policy(c::SettingNumber { number: 5 }), values()[2].1.clone());
                let policy = String::from_utf8(file_from_setting(&policy)).expect("policy JSON");
                policy
                    .strip_prefix("{\"items\":[")
                    .expect("items")
                    .strip_suffix("]}")
                    .expect("items end")
                    .replace(r#"{"kind":"policy","number":5}"#, selector)
            } else {
                format!(r#"{{"setting":{selector},"edit":"reset"}}"#)
            }
        })
        .collect();
    let file = format!(r#"{{"items":[{}]}}"#, items.join(","));
    let (mut root, number) = proposed(file.as_bytes(), false);
    assert!(root.sends.contains_key(&number), "{file}");
    root.done(number, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 7, to: 0 }));
    let answer = root.answer(number).expect("proposal outcome");
    assert_eq!(answer.status, 0);
    assert_eq!(answer.stdout.as_ref(), format!("proposed: p73/7, to {recipient}\n").as_bytes(), "{file}");
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
            reason: c::DocumentText::complete(
                c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
            ),
            action: c::RetainedAction::Settings(c::RetainedActionSettings {
                projects: List::with_capacity(0),
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
            reason: c::DocumentText::complete(
                c::Text::new(b"review".as_slice().into(), u32::MAX).expect("fixture reason"),
            ),
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
            c::RetainedAction::Settings(c::RetainedActionSettings {
                projects: List::with_capacity(0),
                against: 5,
                items: list([]),
                made: Some(6),
            }),
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

fn large_charter_review(setting: c::Setting) -> c::ReadValue {
    let large = c::ReviewValue::TooLarge(c::Size { bytes: 8192 });
    review_proposal(Some(c::SettingsReview {
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
    }))
}
#[test]
fn proposal_review_reads_both_large_values_whole_and_keeps_refusals_in_place() {
    for json in [false, true] {
        for refused in [false, true] {
            let (setting, value) = values().remove(0);
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&["proposal", "show", "p73/7"]), json, None);
            serve_review(&mut root, call, large_charter_review(setting));
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
fn restore_review_compares_resolved_rules_values() {
    for equal in [false, true] {
        for paged in [[true, true], [false, true], [true, false], [false, false]] {
            for refused in [[false, false], [true, false], [false, true], [true, true]] {
                if paged.iter().zip(refused).any(|(paged, refused)| refused && !paged) {
                    continue;
                }
                let now = rules_spend(8_300_000);
                let proposed = rules_spend(if equal { 8_300_000 } else { 8_117_760 });
                assert_eq!(measured_review_value(&now), measured_review_value(&proposed));
                let [current, restored] =
                    [(now.clone(), paged[0]), (proposed.clone(), paged[1])].map(|(value, paged)| {
                        if paged {
                            measured_review_value(&value)
                        } else {
                            c::ReviewValue::Value(c::SettingSet { value })
                        }
                    });
                let (mut root, call) = fixtures::read_call(&["proposal", "show", "p73/7"]);
                serve_review(&mut root, call, restored_rules_review(current, restored));
                for (index, value) in [now, proposed].into_iter().enumerate() {
                    if !paged[index] {
                        continue;
                    }
                    assert_eq!(
                        root.reads.get(&call).expect("rules read").1,
                        c::Read::Setting(c::ReadSetting {
                            setting: c::Setting::Rules,
                            revision: if index == 0 { 4 } else { 0 },
                            proposal: (index == 1)
                                .then_some(c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 }),
                        })
                    );
                    if refused[index] {
                        root.serve(call, c::ReadResult::None(c::ReadNone::Refused));
                    } else {
                        serve_policy_value(&mut root, call, c::Setting::Rules, &value);
                    }
                }
                let answer = root.answer(call).expect("restore review");
                assert_eq!(answer.status, 0);
                let output = core::str::from_utf8(&answer.stdout).expect("text");
                let kept = equal && !refused.contains(&true);
                let label = if kept { "kept" } else { "changed" };
                assert!(output.contains(&format!("rules · {label} · now ")), "{paged:?} {refused:?}: {output}");
                assert_eq!(output.contains("rules · kept"), kept);
                if refused.contains(&true) {
                    assert_eq!(output.matches("refused").count(), refused.iter().filter(|v| **v).count());
                } else {
                    assert!(output.contains("8300000"));
                    assert!(output.contains(if equal { "8300000" } else { "8117760" }));
                }
            }
        }
    }
}
fn rules_spend(spend: u64) -> c::SettingValue {
    let (_, value) = values().remove(3);
    let c::SettingValue::Rules(mut rules) = value else { panic!("rules") };
    rules.period_spend = spend;
    c::SettingValue::Rules(rules)
}
fn restored_rules_review(now: c::ReviewValue, would_be: c::ReviewValue) -> c::ReadValue {
    // Revision 4 accepted a rules set; restoring revision 1 resets it to the default.
    let mut proposal = review_proposal(Some(c::SettingsReview {
        key: None,
        newest: 4,
        cut: false,
        items: list([c::ReviewItem {
            setting: c::Setting::Rules,
            edit: c::RevisionItemEdit::Reset,
            now,
            would_be,
            origin: c::SettingOrigin::Change(c::OriginRevision { revision: 4 }),
            application: None,
        }]),
    }));
    let c::ReadValue::Proposal(p) = &mut proposal else { panic!("proposal") };
    let c::ProposalAction::Settings(s) = &mut p.action else { panic!("settings") };
    s.against = 4;
    proposal
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
                        projects: List::with_capacity(0),
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

#[test]
fn unknown_settings_decisions_explain_that_the_proposal_is_not_waiting() {
    for (proposer, name) in [(c::Proposer::Task(73), "t73/7"), (c::Proposer::Party(73), "p73/7")] {
        for (extra, expected) in [
            (&[][..], c::Running::Stays),
            (&["--running", "stays"][..], c::Running::Stays),
            (&["--running", "moves"][..], c::Running::Moves),
        ] {
            for json in [false, true] {
                let mut root = fixtures::signed_in(fixtures::PARTY);
                let mut argv = vec!["proposal", "decide", name, "accept", "--key", "late"];
                argv.extend(extra);
                let call = root.submit_prepared(fixtures::call(&argv), json, None);
                let mut history = retained_proposal(
                    c::ProposalKey { proposer, number: 7 },
                    c::RetainedAction::Settings(c::RetainedActionSettings {
                        projects: List::with_capacity(0),
                        against: 5,
                        items: list([]),
                        made: None,
                    }),
                );
                let c::ReadValue::Ended(history_row) = &mut history else { unreachable!() };
                let c::EndedValue::Proposal(proposal) = &mut history_row.value else { unreachable!() };
                proposal.state = c::ProposalHistoryState::Withdrawn;
                root.serve(call, c::ReadResult::Value(history));
                let c::Ask::Decide { on: c::DecideOn::Proposal { running, .. }, .. } = fixtures::act_ask(&root, call)
                else {
                    panic!("settings decision")
                };
                assert_eq!(*running, Some(expected.clone()));
                root.answered(call, c::Answer::Refused { refusal: c::Refusal::Unknown, at: None });
                let answer = root.answer(call).expect("engine refusal");
                assert_eq!(answer.status, 2);
                assert!(answer.stdout.is_empty());
                if json {
                    let output = core::str::from_utf8(&answer.stderr).expect("JSON refusal");
                    assert!(output.contains("\"operation\":\"decide\""), "{output}");
                    assert!(output.contains("\"unknown\""), "{output}");
                    assert!(!output.contains("isn't waiting"), "{output}");
                } else {
                    assert_eq!(
                        answer.stderr.as_ref(),
                        b"proposal decide: this proposal isn't waiting; it may have been decided or replaced\n"
                    );
                }
            }
        }
    }
}

fn forge_value(kind: u32) -> Box<[u8]> {
    let json = if kind == 8 {
        br#"{"items":[{"setting":{"kind":"project-classes","project":7},"set":{"items":[{"kind":"wording","class":"decide"},{"kind":"implementation","words":"keep the format","class":"ask"},{"set":2,"class":"ask-away"}],"sets":[{"number":2,"name":"protocol","patterns":["src/protocol.*","docs/protocol/**"]}]}}]}"#.as_slice()
    } else if kind == 7 {
        br#"{"items":[{"setting":{"kind":"project-ways","project":7},"set":{"drafts":true,"goals":false,"changes":false,"way":0}}]}"#.as_slice()
    } else if kind == 6 {
        br#"{"items":[{"setting":{"kind":"project-questions","project":7},"set":"foreman"}]}"#.as_slice()
    } else if kind == 9 {
        br#"{"items":[{"setting":{"kind":"charter-words","number":1},"set":"reviews every change"}]}"#.as_slice()
    } else if kind == 10 {
        br#"{"items":[{"setting":{"kind":"project-words","project":7},"set":{"headline":"Our way","lede":"We review every change."}}]}"#.as_slice()
    } else if kind == 12 {
        br#"{"items":[{"setting":{"kind":"project-areas","project":7},"set":{"areas":[{"name":"tests","words":"C tests","owner":{"task":42}}]}}]}"#.as_slice()
    } else if kind == 13 {
        br#"{"items":[{"setting":{"kind":"project-triager","project":7},"set":{"owner":{"charter":14}}}]}"#.as_slice()
    } else if kind == 3 {
        br#"{"items":[{"setting":{"kind":"forge-charter","number":1},"set":6}]}"#.as_slice()
    } else if kind == 11 {
        br#"{"items":[{"setting":{"kind":"template-goal","number":1},"set":{"design":true,"close_out":true}}]}"#
            .as_slice()
    } else {
        br#"{"items":[{"setting":{"kind":"landing-rule","number":1},"set":{"pattern":{"segments":["ff"],"last":{"open":"61"}},"enforce":true,"ci":false,"up_to_date":true,"gates":[{"number":2,"blocking":false,"freshness":"clean"}],"approvals":[{"role":0,"people":1,"freshness":"exact"}]}}]}"#.as_slice()
    };
    forge_file_value(json)
}
fn forge_file_value(json: &[u8]) -> Box<[u8]> {
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
    for (argv, kind, scope, label, stock) in [
        (vec!["settings", "show", "landing-rule", "1"], 1, 0, "landing-rule 1", false),
        (vec!["settings", "show", "landing-rule", "1", "--project", "7"], 2, 7, "landing-rule 1 --project 7", false),
        (
            vec!["settings", "show", "landing-rule", "1", "--project", "shop"],
            2,
            fixtures::PROJECT,
            "landing-rule 1 --project",
            false,
        ),
        (vec!["settings", "show", "forge-charter", "1"], 3, 0, "forge-charter 1", false),
        (vec!["settings", "show", "charter-words", "1"], 9, 0, "charter-words 1", false),
        (vec!["settings", "show", "charter-words", "1"], 9, 0, "charter-words 1", true),
        (vec!["settings", "show", "project-areas", "7"], 12, 7, "project-areas 7", false),
        (vec!["settings", "show", "project-triager", "7"], 13, 7, "project-triager 7", false),
        (vec!["settings", "show", "project-words", "7"], 10, 7, "project-words 7", false),
        (vec!["settings", "show", "project-words", "shop"], 10, fixtures::PROJECT, "project-words", false),
        (vec!["settings", "show", "template-goal", "1"], 11, 0, "template-goal 1", false),
        (vec!["settings", "show", "project-questions", "7"], 6, 7, "project-questions 7", false),
        (vec!["settings", "show", "project-ways", "7"], 7, 7, "project-ways 7", false),
        (vec!["settings", "show", "project-classes", "7"], 8, 7, "project-classes 7", false),
        (vec!["settings", "show", "project-classes", "shop"], 8, fixtures::PROJECT, "project-classes", false),
        (vec!["settings", "show", "project-classes", "7", "--revision", "5"], 8, 7, "project-classes 7", false),
    ] {
        for json in [false, true] {
            let setting = c::Setting::Application { kind, scope, number: 1 };
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&argv), json, None);
            let historical = argv.contains(&"--revision");
            if historical {
                assert!(!root.reads.contains_key(&call), "historical reads go straight to the forge");
            } else {
                assert_eq!(root.reads.get(&call).unwrap().1, c::Read::Settings);
                root.serve(
                    call,
                    forge_inventory(
                        setting,
                        if stock {
                            c::ApplicationFollows::Default
                        } else {
                            c::ApplicationFollows::Changed { revision: 5 }
                        },
                    ),
                );
            }
            assert!(root.answer(call).is_none());
            if historical {
                let w::Read::RevisionSetting(query) = forge_query(&root, call) else { panic!("revision query") };
                assert_eq!((query.kind(), query.scope(), query.number(), query.revision()), (kind, scope, 1, 5));
            } else {
                let w::Read::Setting(query) = forge_query(&root, call) else { panic!("setting query") };
                assert_eq!((query.kind(), query.scope(), query.number()), (kind, scope, 1));
            }
            root.serve_readout(call, forge_setting_page(Some(forge_value(kind))));
            let answer = root.answer(call).expect("joined setting");
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            if json {
                let expected = match kind {
                    3 => "\"value\":6",
                    6 => "\"value\":\"foreman\"",
                    7 => "\"drafts\":true",
                    8 => "\"class\":\"ask-away\"",
                    9 => "\"value\":\"reviews every change\"",
                    10 => "\"value\":{\"headline\":\"Our way\",\"lede\":\"We review every change.\"}",
                    11 => "\"design\":true,\"close_out\":true",
                    _ => "\"ci\":false",
                };
                assert!(text.contains(expected), "{text}");
                let decoded = settings_file::decode(&file_from_setting(&answer.stdout), &fixtures::limits())
                    .expect("show JSON applies");
                assert_eq!(
                    decoded.as_slice(),
                    &[c::SettingItem {
                        setting,
                        edit: c::SettingEdit::Set(c::SettingSet {
                            value: c::SettingValue::Application(forge_value(kind))
                        })
                    }]
                );
                if historical {
                    assert!(!text.contains("\"application\":"), "{text}");
                } else if stock {
                    assert!(text.contains("\"application\":{\"follows\":\"default\""), "{text}");
                } else {
                    assert!(text.contains("\"application\":{\"follows\":\"change\",\"revision\":5"), "{text}");
                }
                assert!(text.contains(label.split_whitespace().next().unwrap()), "{text}");
            } else {
                let canonical = setting_json(setting, c::SettingValue::Application(forge_value(kind)));
                let canonical = core::str::from_utf8(&canonical).unwrap();
                let (_, value) = canonical.split_once("\"value\":").unwrap();
                let value = value.trim_end().strip_suffix('}').unwrap();
                if kind == 8 {
                    for line in [
                        "wording: agents decide, then tell you",
                        "implementation: asks you — keep the format",
                        "path set 2 (protocol): asks you, even when away",
                        "  path set 2 (protocol):\n    src/protocol.*\n    docs/protocol/**",
                        "Always waits: money, irreversible, rules, other.",
                    ] {
                        assert!(text.contains(line), "{text}");
                    }
                } else {
                    assert!(text.contains(value), "inline value differs from the shared settings form: {text}");
                }
                assert!(text.starts_with(label), "{text}");
                assert!(
                    text.contains(if historical {
                        "revision 5"
                    } else if stock {
                        "the stock"
                    } else {
                        "changed at revision 5"
                    }),
                    "{text}"
                );
                assert_eq!(text.contains("the stock"), stock, "{text}");
            }
            assert_eq!(root.watches.len(), 1);
            if historical {
                continue;
            }
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let mut historical = argv.clone();
            historical.extend_from_slice(&["--revision", "5"]);
            let call = root.submit_prepared(fixtures::call(&historical), json, None);
            let w::Read::RevisionSetting(query) = forge_query(&root, call) else { panic!("historical setting query") };
            assert_eq!((query.kind(), query.scope(), query.number(), query.revision()), (kind, scope, 1, 5));
            root.serve_readout(call, forge_setting_page(Some(forge_value(kind))));
            let answer = root.answer(call).expect("historical setting");
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            assert!(text.contains(if json { "\"revision\":5" } else { ", revision 5" }), "{text}");
            let expected = match kind {
                3 => "6",
                6 => "\"foreman\"",
                7 => "\"drafts\":true",
                8 => {
                    if json {
                        "\"class\":\"ask-away\""
                    } else {
                        "path set 2 (protocol): asks you, even when away"
                    }
                }
                9 => "reviews every change",
                10 => "We review every change.",
                11 => "\"design\":true,\"close_out\":true",
                _ => "\"ci\":false",
            };
            assert!(text.contains(expected), "{text}");
        }
    }
}
#[test]
fn forge_setting_commands_print_removed_and_unset_values_and_preserve_refusals() {
    for json in [false, true] {
        for follows in [c::ApplicationFollows::Default, c::ApplicationFollows::Removed { revision: 5 }] {
            for (kind, scope, number, argv) in [
                (3, 0, 2, vec!["settings", "show", "forge-charter", "2"]),
                (11, 0, 2, vec!["settings", "show", "template-goal", "2"]),
                (7, 2, 1, vec!["settings", "show", "project-ways", "2"]),
                (8, 2, 1, vec!["settings", "show", "project-classes", "2"]),
                (9, 0, 1, vec!["settings", "show", "charter-words", "1"]),
                (10, 7, 1, vec!["settings", "show", "project-words", "7"]),
            ] {
                let mut root = fixtures::signed_in(fixtures::PARTY);
                let call = root.submit_prepared(fixtures::call(&argv), json, None);
                root.serve(call, forge_inventory(c::Setting::Application { kind, scope, number }, follows.clone()));
                root.serve_readout(call, forge_setting_page(None));
                let answer = root.answer(call).unwrap();
                assert_eq!(answer.status, 0);
                let text = core::str::from_utf8(&answer.stdout).unwrap();
                if kind == 3 || kind == 9 || kind == 10 || kind == 11 {
                    assert!(
                        text.contains(if json {
                            "\"value\":null"
                        } else if kind == 11 {
                            "template-goal 2: none kept; its coordinator plans and makes no close-out.\n"
                        } else {
                            "\nnone\n"
                        }),
                        "{text}"
                    );
                } else if json {
                    let decoded = settings_file::decode(&file_from_setting(&answer.stdout), &fixtures::limits())
                        .expect("stock JSON applies");
                    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(bytes) }) =
                        &decoded.as_slice()[0].edit
                    else {
                        panic!("stock value")
                    };
                    if kind == 8 {
                        let mut reader = skein_lib::Reader::new(bytes);
                        let record = temper_client_protocol_wire::v1::ProjectClasses::decode(
                            &temper_client_protocol_wire::v1::CEILINGS,
                            &mut reader,
                        )
                        .unwrap();
                        assert_eq!(
                            temper_protocol_reaches::classes_from(&record),
                            temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview)
                        );
                    }
                } else {
                    assert!(text.contains("the stock"), "{text}");
                    if kind == 8 {
                        assert!(text.contains("tests: agents decide, then tell you"), "{text}");
                        assert!(text.contains("dependency: asks you"), "{text}");
                    }
                }
            }
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
        (9, 0, 2, c::ApplicationFollows::Default),
        (10, 7, 1, c::ApplicationFollows::Changed { revision: 5 }),
        (11, 0, 4, c::ApplicationFollows::Default),
    ];
    let mut lines = List::with_capacity(u32::try_from(settings.len()).unwrap());
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
        "charter-words 2: default",
        "project-words 7: changed in revision 5",
        "template-goal 4: default",
    ] {
        assert!(text.contains(line), "{text}");
    }
}
fn forge_review() -> c::ReadValue {
    forge_state_review(c::Setting::Application { kind: 1, scope: 0, number: 1 }, Some(c::ApplicationFollows::Default))
}
fn forge_state_review(setting: c::Setting, follows: Option<c::ApplicationFollows>) -> c::ReadValue {
    let mut proposal = review_proposal(Some(c::SettingsReview {
        key: Some(u64::MAX),
        newest: 5,
        items: list([c::ReviewItem {
            setting,
            edit: c::RevisionItemEdit::Set,
            now: c::ReviewValue::None,
            would_be: c::ReviewValue::None,
            origin: c::SettingOrigin::NotOffered,
            application: Some(c::ApplicationReview {
                edit: c::SizedEdit::Set { size: 4 },
                now: follows.map(|follows| c::ApplicationState { follows, size: 4, charters: list([]) }),
            }),
        }]),
        cut: true,
    }));
    let c::ReadValue::Proposal(p) = &mut proposal else { panic!("proposal") };
    let c::ProposalAction::Settings(settings) = &mut p.action else { panic!("settings proposal") };
    settings.items = list([c::RevisionItemHead { setting, edit: c::RevisionItemEdit::Set }]);
    proposal
}
fn forge_set_items(kind: u32) -> jig_client::Readout {
    use temper_client_protocol_wire::v1 as w;
    let item = w::ProposalItem::new(
        &w::CEILINGS,
        w::ProposalItemParts {
            kind,
            scope: 0,
            number: 1,
            edit: w::SettingEdit::Set(
                w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: forge_value(kind) }).unwrap(),
            ),
        },
    )
    .unwrap();
    forge_items_page(list([item]))
}
fn forge_items_page(items: List<temper_client_protocol_wire::v1::ProposalItem>) -> jig_client::Readout {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let record =
        w::ProposalItemsAnswer::new(&w::CEILINGS, w::ProposalItemsAnswerParts { items, owned_approval: false })
            .unwrap();
    jig_client::Readout::Family { family: wire::FORGE, record: Some(wire::proposal_items_answer(&record)) }
}
fn forge_proposed_review(kind: u32, follows: Option<c::ApplicationFollows>, json: bool) -> jig::Printed {
    let file = if kind == 3 {
        br#"{"items":[{"setting":{"kind":"forge-charter","number":1},"set":6}]}"#.to_vec()
    } else {
        file_from_setting(&setting_json(
            c::Setting::Application { kind, scope: 0, number: 1 },
            c::SettingValue::Application(forge_value(kind)),
        ))
    };
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut request =
        fixtures::call(&["settings", "propose", "FILE", "Please review", "--against", "5", "--key", "forge-review"]);
    *request.values.get_mut(0).unwrap() = Some(Values::One(Value::Text(file.into())));
    let proposed = root.submit_prepared(request, false, None);
    let c::Ask::Propose { reason, action: c::ProposeAction::Settings(settings), .. } =
        fixtures::act_ask(&root, proposed)
    else {
        panic!("settings proposal")
    };
    assert_eq!(reason.bytes(), b"Please review");
    assert_eq!(settings.items.as_slice()[0].setting, c::Setting::Application { kind, scope: 0, number: 1 });
    root.done(proposed, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 7, to: 0 }));
    let answer = root.answer(proposed).unwrap();
    assert_eq!(answer.status, 0);
    let printed = core::str::from_utf8(&answer.stdout).unwrap();
    let key = printed.strip_prefix("proposed: ").unwrap().split(',').next().unwrap();
    let call = root.submit_prepared(fixtures::call(&["proposal", "show", key]), json, None);
    serve_review(
        &mut root,
        call,
        forge_state_review(c::Setting::Application { kind, scope: 0, number: 1 }, follows.clone()),
    );
    root.serve_readout(call, forge_set_items(kind));
    let value = if follows == Some(c::ApplicationFollows::Removed { revision: 5 }) {
        None
    } else if kind == 3 {
        Some(forge_file_value(br#"{"items":[{"setting":{"kind":"forge-charter","number":1},"set":2}]}"#))
    } else {
        Some(forge_value(kind))
    };
    root.serve_readout(call, forge_setting_page(value));
    root.answer(call).expect("proposal review")
}
#[test]
fn forge_proposal_show_uses_application_state_and_common_line_form() {
    for (follows, origin, json_origin, now) in [
        (Some(c::ApplicationFollows::Default), "default", r#""default""#, "2"),
        (Some(c::ApplicationFollows::Changed { revision: 5 }), "revision 5", r#"{"change":{"revision":5}}"#, "2"),
        (
            Some(c::ApplicationFollows::Removed { revision: 5 }),
            "removed at revision 5",
            r#"{"removed":{"revision":5}}"#,
            "none",
        ),
        (None, "not offered", r#""not_offered""#, "2"),
    ] {
        for json in [false, true] {
            let answer = forge_proposed_review(3, follows.clone(), json);
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            if json {
                let (_, items) = text.split_once("\"application_items\":").unwrap();
                let expected = format!("\"origin\":{json_origin},");
                assert!(items.contains(&expected), "{text}");
                assert_eq!(text.matches(&expected).count(), 2, "both review origins: {text}");
            } else {
                let expected = format!("forge-charter 1 · set · now {now} ({origin}) · would be 6");
                assert!(text.lines().any(|line| line == expected), "{text}");
            }
        }
    }
    for follows in [c::ApplicationFollows::Default, c::ApplicationFollows::Changed { revision: 5 }] {
        let answer = forge_proposed_review(1, Some(follows.clone()), false);
        assert_eq!(answer.status, 0);
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        let line =
            text.lines().find(|line| line.starts_with("landing-rule 1 · set · now ")).expect("landing rule line");
        assert!(
            line.contains(if follows == c::ApplicationFollows::Default {
                " (default) · would be "
            } else {
                " (revision 5) · would be "
            }),
            "{text}"
        );
    }
}
#[test]
fn forge_proposal_show_reads_the_reviews_key_and_prints_every_temper_edit() {
    use temper_client_protocol_wire::v1 as w;
    for json in [false, true] {
        let mut root = fixtures::signed_in(fixtures::PARTY);
        let call = root.submit_prepared(fixtures::call(&["proposal", "show", "p73/7"]), json, None);
        serve_review(&mut root, call, forge_review());
        let w::Read::ProposalItems(query) = forge_query(&root, call) else { panic!("proposal items") };
        assert_eq!(query.key(), u64::MAX, "the review's key, not proposer 73's number 7");
        let items = list([
            w::ProposalItem::new(
                &w::CEILINGS,
                w::ProposalItemParts {
                    kind: 1,
                    scope: 0,
                    number: 1,
                    edit: w::SettingEdit::Set(
                        w::SettingValue::new(&w::CEILINGS, w::SettingValueParts { value: forge_value(1) }).unwrap(),
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
        ]);
        root.serve_readout(call, forge_items_page(items));
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
        vec!["settings", "show", "charter-words", "0"],
        vec!["settings", "show", "charter-words", "1", "--project", "7"],
        vec!["settings", "show", "forge-charter", "0"],
        vec!["settings", "show", "forge-charter", "3"],
        vec!["settings", "show", "forge-charter", "1", "--project", "1"],
        vec!["settings", "show", "project-classes"],
        vec!["settings", "show", "project-classes", "7", "1"],
        vec!["settings", "show", "template-goal", "0"],
        vec!["settings", "show", "template-goal", "4", "--project", "1"],
    ] {
        assert!(fixtures::parse(&argv).is_err(), "{argv:?}");
    }
}

fn project_policy() -> c::SettingValue {
    let (_, value) = values().remove(2);
    value
}
fn project_policy_file() -> Vec<u8> {
    file_from_setting(&setting_json(c::Setting::ProjectPolicy { project: 1 }, project_policy()))
}
#[test]
fn a_settings_file_with_a_project_policy_sends_its_item() {
    let file = project_policy_file();
    let (root, call) = changed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent change") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") };
    assert_eq!(root.sends.len(), 1);
    assert_eq!(
        change.items,
        list([c::SettingItem {
            setting: c::Setting::ProjectPolicy { project: 1 },
            edit: c::SettingEdit::Set(c::SettingSet { value: project_policy() }),
        }])
    );
    let (root, call) = proposed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent proposal") else { panic!("jig ask") };
    let c::Ask::Propose { action: c::ProposeAction::Settings(proposal), .. } = ask.as_ref() else {
        panic!("settings proposal")
    };
    assert_eq!(root.sends.len(), 1);
    assert_eq!(proposal.items, change.items);
}
fn project_policy_json() -> String {
    setting_value_json(&project_policy())
}
fn setting_value_json(value: &c::SettingValue) -> String {
    let value = jig::form_setting_value(value);
    let bounds = skein_json::writer::Limits { depth: 32, length: 8192 };
    let mut measure = skein_json::writer::Encoder::measure(&bounds);
    settings_form::render::encode_value(&value, &mut measure);
    let mut out = skein_json::writer::Encoder::write(measure.measured().expect("policy JSON fits"), &bounds);
    settings_form::render::encode_value(&value, &mut out);
    String::from_utf8(out.finish().into_vec()).expect("policy JSON")
}
#[test]
fn a_project_policy_item_refused_by_the_form_sends_nothing() {
    let value = project_policy_json();
    for (items, expected) in [
        (
            r#"{"setting":{"kind":"project-policy","project":1},"edit":"removed"}"#.to_owned(),
            "settings file item 1: edit: a project's policy is set alone",
        ),
        (
            r#"{"setting":{"kind":"project-policy","project":1},"edit":"reset"}"#.to_owned(),
            "settings file item 1: edit: a project's policy is set alone",
        ),
        (
            format!(
                r#"{{"setting":{{"kind":"project-policy","project":1}},"set":{value}}},{{"setting":{{"kind":"project-policy","project":2}},"set":{value}}}"#
            ),
            "settings file item 2: setting: one project's policy per file",
        ),
        (
            format!(r#"{{"setting":{{"kind":"project-policy","project":0}},"set":{value}}}"#),
            "settings file item 1: project: from 1",
        ),
        (
            format!(r#"{{"setting":{{"kind":"project-policy"}},"set":{value}}}"#),
            "settings file item 1: project: missing member",
        ),
        (
            format!(r#"{{"setting":{{"kind":"project-policy","project":1,"number":1}},"set":{value}}}"#),
            "settings file item 1: number: project-policy has no number",
        ),
    ] {
        let file = format!(r#"{{"items":[{items}]}}"#);
        let (root, call) = changed(file.as_bytes(), false);
        project_policy_form_refusal(root, call, expected);
        let (root, call) = proposed(file.as_bytes(), false);
        project_policy_form_refusal(root, call, expected);
    }
}
fn project_policy_form_refusal(mut root: fixtures::Harness, call: u32, expected: &str) {
    let answer = root.answer(call).expect("form refusal");
    assert_eq!(answer.status, 1);
    assert!(root.sends.is_empty());
    assert_eq!(answer.stderr.as_ref(), format!("{expected}\n").as_bytes());
}
fn project_policy_review(now: c::ReviewValue, would_be: c::ReviewValue) -> c::ReadValue {
    let setting = c::Setting::ProjectPolicy { project: 1 };
    let mut proposal = review_proposal(Some(c::SettingsReview {
        key: None,
        newest: 5,
        items: list([c::ReviewItem {
            setting,
            edit: c::RevisionItemEdit::Set,
            now,
            origin: c::SettingOrigin::NotOffered,
            would_be,
            application: None,
        }]),
        cut: false,
    }));
    let c::ReadValue::Proposal(p) = &mut proposal else { panic!("proposal") };
    let c::ProposalAction::Settings(s) = &mut p.action else { panic!("settings proposal") };
    s.items = list([c::RevisionItemHead { setting, edit: c::RevisionItemEdit::Set }]);
    proposal
}
#[test]
fn proposal_show_prints_a_project_policy_now_and_as_it_would_be() {
    let now = project_policy();
    let c::SettingValue::Policy(mut policy) = now.clone() else { panic!("policy") };
    policy.period_spend = 3000;
    let would_be = c::SettingValue::Policy(policy);
    let review = project_policy_review(
        c::ReviewValue::Value(c::SettingSet { value: now }),
        c::ReviewValue::Value(c::SettingSet { value: would_be }),
    );
    for (argv, name) in [
        (vec!["proposal", "show", "p73/7"], "settings_review_project_policy_text"),
        (vec!["proposal", "show", "p73/7", "--json"], "settings_review_project_policy_json"),
    ] {
        let answer = read(&argv, review.clone());
        assert_eq!(answer.status, 0);
        golden(name, &answer.stdout);
    }
}
fn policy_review_values(spend: u64) -> Vec<(c::Setting, c::SettingValue, c::SettingValue)> {
    let c::SettingValue::Policy(mut now) = project_policy() else { panic!("policy") };
    now.period_spend = spend;
    let mut proposed = now.clone();
    proposed.period_spend = 8_200_000;
    let (_, rules) = values().remove(3);
    let c::SettingValue::Rules(mut proposed_rules) = rules.clone() else { panic!("rules") };
    proposed_rules.period_spend = 8_300_000;
    vec![
        (c::Setting::ProjectPolicy { project: 1 }, c::SettingValue::Policy(now), c::SettingValue::Policy(proposed)),
        (c::Setting::Rules, rules, c::SettingValue::Rules(proposed_rules)),
    ]
}
fn measured_review_value(value: &c::SettingValue) -> c::ReviewValue {
    let wire =
        jig_client_protocol_link::translate::settings::wire_setting_value(value, &crate::limits::tools().settings)
            .expect("wire setting");
    c::ReviewValue::TooLarge(c::Size { bytes: u64::from(wire.measure()) })
}
fn policy_review_proposal(values: &[(c::Setting, c::SettingValue, c::SettingValue)]) -> c::ReadValue {
    let mut proposal = review_proposal(Some(c::SettingsReview {
        key: None,
        newest: 5,
        cut: false,
        items: list(values.iter().map(|(setting, now, proposed)| c::ReviewItem {
            setting: *setting,
            edit: c::RevisionItemEdit::Set,
            now: measured_review_value(now),
            would_be: measured_review_value(proposed),
            origin: c::SettingOrigin::NotOffered,
            application: None,
        })),
    }));
    let c::ReadValue::Proposal(p) = &mut proposal else { panic!("proposal") };
    p.reason = c::DocumentText::complete(words("Please review"));
    let c::ProposalAction::Settings(s) = &mut p.action else { panic!("settings") };
    s.items = list(
        values.iter().map(|(setting, _, _)| c::RevisionItemHead { setting: *setting, edit: c::RevisionItemEdit::Set }),
    );
    proposal
}
fn propose_policy_file(root: &mut fixtures::Harness, values: &[(c::Setting, c::SettingValue, c::SettingValue)]) {
    let (_, _, proposed) = &values[0];
    let setting = c::Setting::Policy(c::SettingNumber { number: 1 });
    let default = project_policy();
    let call = root.submit_prepared(fixtures::call(&["settings", "show", "policy", "1"]), true, None);
    newest_for_setting(root, call);
    assert_eq!(
        root.reads.get(&call).expect("policy 1 read").1,
        c::Read::Setting(c::ReadSetting { setting, revision: 5, proposal: None })
    );
    serve_policy_value(root, call, setting, &default);
    let shown = root.answer(call).expect("policy 1");
    let current_file = String::from_utf8(file_from_setting(&shown.stdout))
        .unwrap()
        .replace(r#""kind":"policy","number":1"#, r#""kind":"project-policy","project":1"#);
    let current = setting_value_json(&default);
    let policy = setting_value_json(proposed);
    let file = current_file.replace(&current, &policy);
    let rules = setting_value_json(&values[1].2);
    let prefix = file.strip_suffix("]}").expect("settings file end");
    let file = format!("{prefix},{{\"setting\":{{\"kind\":\"rules\"}},\"set\":{rules}}}]}}");
    let mut request =
        fixtures::call(&["settings", "propose", "FILE", "Please review", "--against", "5", "--key", "k1"]);
    *request.values.get_mut(0).unwrap() = Some(Values::One(Value::Text(file.into_bytes().into())));
    let call = root.submit_prepared(request, false, None);
    let c::Ask::Propose { reason, action: c::ProposeAction::Settings(settings), .. } = fixtures::act_ask(root, call)
    else {
        panic!("settings proposal")
    };
    assert_eq!(reason.bytes(), b"Please review");
    assert_eq!(settings.against, 5);
    assert_eq!(
        settings.items,
        list(values.iter().map(|(setting, _, value)| c::SettingItem {
            setting: *setting,
            edit: c::SettingEdit::Set(c::SettingSet { value: value.clone() }),
        }))
    );
    root.done(call, c::Outcome::Document(c::DocumentOutcome::Proposed { proposal: 7, to: 0 }));
    assert_eq!(root.answer(call).unwrap().status, 0);
}
fn serve_policy_value(root: &mut fixtures::Harness, call: u32, setting: c::Setting, value: &c::SettingValue) {
    root.serve(
        call,
        c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting,
            revision: 5,
            value: Some(value.clone()),
            application: None,
            proposed: None,
        }))),
    );
}
#[test]
fn proposal_show_resolves_current_project_policy_whole() {
    for spend in [2000, 2500] {
        let values = policy_review_values(spend);
        for json in [false, true] {
            let mut root = fixtures::signed_in(fixtures::PARTY);
            propose_policy_file(&mut root, &values);
            let call = root.submit_prepared(fixtures::call(&["proposal", "show", "p73/7"]), json, None);
            serve_review(&mut root, call, policy_review_proposal(&values));
            for (setting, now, proposed) in &values {
                for (is_proposed, value) in [(false, now), (true, proposed)] {
                    assert_eq!(
                        root.reads.get(&call).expect("whole setting fallback").1,
                        c::Read::Setting(c::ReadSetting {
                            setting: *setting,
                            revision: if is_proposed { 0 } else { 5 },
                            proposal: is_proposed
                                .then_some(c::ProposalKey { proposer: c::Proposer::Party(73), number: 7 }),
                        }),
                        "current and proposed values both need fallback reads"
                    );
                    let c::ReviewValue::TooLarge(size) = measured_review_value(value) else { panic!("size") };
                    assert!(size.bytes > 0);
                    assert_ne!(
                        size.bytes,
                        u64::try_from(setting_value_json(value).len()).unwrap(),
                        "wire size differs from printed JSON"
                    );
                    serve_policy_value(&mut root, call, *setting, value);
                }
            }
            let answer = root.answer(call).expect("whole policy review");
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            for (_, now, proposed) in &values {
                assert!(text.contains(&setting_value_json(now)), "current value whole: {text}");
                assert!(text.contains(&setting_value_json(proposed)), "proposed value whole: {text}");
            }
            assert!(text.contains("Please review"), "reason retained: {text}");
            assert!(!text.contains("too_large") && !text.contains("too large"), "{text}");
            if json {
                assert!(text.contains("\"against\":5") && text.contains("\"newest\":5"), "{text}");
                assert!(text.contains("\"origin\":\"not_offered\""), "{text}");
            } else {
                assert!(text.contains("against revision 5, newest 5"), "{text}");
                assert!(text.contains("project-policy 1 · set · now {"), "{text}");
            }
        }
    }
}
#[test]
fn an_engine_refusal_naming_a_project_policy_prints_its_setting() {
    for (refusal, expected) in [
        (
            c::Refusal::Malformed {
                at: 0,
                why: super::malformed_setting(c::Setting::ProjectPolicy { project: 1 }, c::SettingFault::Rules),
            },
            "settings apply: out of shape; item 0; project-policy 1: rules\n",
        ),
        (c::Refusal::Role { held: None }, "settings apply: The held role does not permit this action\n"),
    ] {
        let (mut root, call) = changed(&project_policy_file(), false);
        root.answered(call, c::Answer::Refused { refusal, at: None });
        let answer = root.answer(call).expect("engine refusal");
        assert_eq!(answer.status, 2);
        assert_eq!(answer.stderr.as_ref(), expected.as_bytes());
    }
}

#[test]
fn a_large_project_policy_revision_prints_no_setting_read_hint() {
    // The retained value is represented by its byte size past the read's bound.
    const POLICY_BYTES: u64 = 1_000_000;
    let answer = read(
        &["revision", "show", "5"],
        c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
            head: head(c::RevisionDecider::Release),
            items: list([c::RevisionItem {
                replaced: c::RevisionReplaced::NotOffered,
                setting: c::Setting::ProjectPolicy { project: 1 },
                edit: c::RevisionEdit::TooLarge(c::Size { bytes: POLICY_BYTES }),
            }]),
        }))),
    );
    assert_eq!(answer.status, 0);
    assert!(answer.stdout.ends_with(b"  project-policy 1: too large, 1000000 bytes\n    was: not offered\n"));
    assert!(!core::str::from_utf8(&answer.stdout).expect("revision text").contains("settings show"));
}

#[test]
fn revision_show_prints_what_each_item_replaced() {
    let mut items = Vec::new();
    for (index, (setting, value)) in values().into_iter().enumerate() {
        let replaced = match index {
            0 => c::RevisionReplaced::Value(c::SettingSet { value: value.clone() }),
            1 => c::RevisionReplaced::Default,
            2 => c::RevisionReplaced::Removed,
            3 => c::RevisionReplaced::TooLarge(c::Size { bytes: 1_000_000 }),
            _ => unreachable!("four fixture kinds"),
        };
        items.push(c::RevisionItem {
            setting,
            edit: if index == 3 { c::RevisionEdit::Reset } else { c::RevisionEdit::Set(c::SettingSet { value }) },
            replaced,
        });
    }
    items.push(c::RevisionItem {
        setting: c::Setting::Application { kind: 2, scope: 3, number: 2 },
        edit: c::RevisionEdit::Application(c::SizedEdit::Set { size: 24 }),
        replaced: c::RevisionReplaced::Application { size: 12 },
    });
    let (_, policy) = values().remove(2);
    items.push(c::RevisionItem {
        setting: c::Setting::ProjectPolicy { project: 3 },
        edit: c::RevisionEdit::TooLarge(c::Size { bytes: 2_000_000 }),
        replaced: c::RevisionReplaced::Value(c::SettingSet { value: policy }),
    });
    items.push(c::RevisionItem {
        setting: c::Setting::Charter(c::SettingNumber { number: 9 }),
        edit: c::RevisionEdit::Removed,
        replaced: c::RevisionReplaced::NotOffered,
    });
    let document = c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
        head: head(c::RevisionDecider::Party(c::RevisionParty { number: 1 })),
        items: list(items),
    })));
    for json in [false, true] {
        let argv = if json { vec!["revision", "show", "5", "--json"] } else { vec!["revision", "show", "5"] };
        let answer = read(&argv, document.clone());
        assert_eq!(answer.status, 0);
        golden(if json { "settings_revision_replaced_json" } else { "settings_revision_replaced" }, &answer.stdout);
    }
}

#[test]
fn a_settings_file_template_sends_its_resources_to_jig() {
    let file = br#"{"items":[{"setting":{"kind":"template","number":4},"set":{"number":4,"name":{"bytes":"draft","cut":false},"executor":{"charter":1},"contract":{"report":{"words":1024}},"shallower":1,"budget":"given","deadline":"none","tracked":1,"flags":{"conversation":false,"listed":true,"retention":"kept","ended_by":"result"},"label":1,"grace":null,"llm":null,"resources":[{"connector":1,"symbol":1}]}}]}"#;
    let (root, call) = changed(file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") };
    let c::SettingEdit::Set(set) = &change.items.get(0).unwrap().edit else { panic!("set") };
    let c::SettingValue::Template(value) = &set.value else { panic!("template") };
    assert_eq!(value.resources.as_slice(), &[c::Symbol { connector: 1, symbol: 1 }]);

    let bounds = crate::limits::form();
    let neutral = settings_form::view::ChangeSettings {
        against: 5,
        items: settings_form::file::decode(file, &bounds).expect("neutral items"),
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
fn settings_show_template_resources_apply_again_unchanged() {
    let (setting, c::SettingValue::Template(mut template)) = values().remove(1) else { panic!("template") };
    template.resources = list([c::Symbol { connector: 1, symbol: 1 }, c::Symbol { connector: 1, symbol: 2 }]);
    let value = c::SettingValue::Template(template);
    let json = setting_json(setting, value.clone());
    let text = core::str::from_utf8(&json).unwrap();
    assert!(text.contains(r#""resources":[{"connector":1,"symbol":1},{"connector":1,"symbol":2}]"#), "{text}");
    let file = file_from_setting(&json);
    let (root, call) = changed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("sent") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") };
    assert_eq!(
        change.items.as_slice(),
        &[c::SettingItem { setting, edit: c::SettingEdit::Set(c::SettingSet { value: value.clone() }) }]
    );

    let bounds = crate::limits::form();
    let wire = jig_client_protocol_link::translate::settings::wire_setting_value(&value, &bounds.settings)
        .expect("wire template");
    let wire = jig_client_protocol_wire::SettingDocument::new(
        &bounds.settings,
        jig_client_protocol_wire::SettingDocumentParts {
            setting: jig_client_protocol_wire::Setting::Template(
                jig_client_protocol_wire::SettingNumber::new(
                    &bounds.settings,
                    jig_client_protocol_wire::SettingNumberParts { number: 4 },
                )
                .unwrap(),
            ),
            revision: 5,
            value: Some(wire),
            application: None,
            proposed: None,
        },
    )
    .unwrap();
    assert_eq!(
        settings_form::wire::restore_setting_document(&wire, &bounds).unwrap().value,
        Some(jig::form_setting_value(&value))
    );
}
#[test]
fn settings_show_template_resources_are_readable_when_present() {
    for resources in [list([]), list([c::Symbol { connector: 1, symbol: 1 }, c::Symbol { connector: 2, symbol: 3 }])] {
        let (setting, c::SettingValue::Template(mut template)) = values().remove(1) else { panic!("template") };
        template.resources = resources.clone();
        let printed = read(
            &["settings", "show", "template", "4"],
            c::ReadValue::Setting(Box::new(c::SettingDocument {
                setting,
                revision: 5,
                value: Some(c::SettingValue::Template(template)),
                application: None,
                proposed: None,
            })),
        );
        assert_eq!(printed.status, 0);
        let text = core::str::from_utf8(&printed.stdout).unwrap();
        if resources.is_empty() {
            assert!(!text.contains("connector "), "{text}");
        } else {
            assert!(text.contains("connector 1 symbol 1\nconnector 2 symbol 3\n"), "{text}");
        }
    }
}

#[test]
fn reference_goal_template_resources_survive_apply_and_show() {
    let file = include_bytes!("../../../../../docs/examples/reference/settings.json");
    let items = settings_file::decode(file, &crate::limits::tools()).expect("reference settings");
    let (root, call) = changed(file, false);
    let c::Ask::ChangeSettings(change) = fixtures::act_ask(&root, call) else { panic!("settings apply") };
    assert_eq!(change.items, items);
    let setting = c::Setting::Template(c::SettingNumber { number: 4 });
    let item = change.items.iter().find(|item| item.setting == setting).expect("goal template");
    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Template(template) }) = &item.edit else {
        panic!("template value")
    };
    assert_eq!(template.resources.as_slice(), &[c::Symbol { connector: 1, symbol: 1 }]);
    assert_eq!(template.tracked, Some(1));
    goal_template_wire_round_trip(file, change, template);
    for json in [false, true] {
        let mut root = fixtures::signed_in(fixtures::PARTY);
        let call = root.submit_prepared(fixtures::call(&["settings", "show", "template", "4"]), json, None);
        newest_for_setting(&mut root, call);
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
                setting,
                revision: 5,
                value: Some(c::SettingValue::Template(template.clone())),
                application: None,
                proposed: None,
            }))),
        );
        let answer = root.answer(call).expect("settings show");
        assert_eq!(answer.status, 0);
        let text = core::str::from_utf8(&answer.stdout).expect("text or JSON");
        assert!(text.contains(r#""resources":[{"connector":1,"symbol":1}]"#), "{text}");
        if json {
            let decoded = settings_file::decode(&file_from_setting(&answer.stdout), &crate::limits::tools())
                .expect("show JSON applies");
            assert_eq!(decoded.as_slice(), core::slice::from_ref(item));
        }
    }
}

fn goal_template_wire_round_trip(file: &[u8], change: &c::ChangeSettings, template: &c::Template) {
    use jig_client_protocol_wire as w;
    let bounds = crate::limits::form();
    let neutral = settings_form::view::ChangeSettings {
        against: 5,
        items: settings_form::file::decode(file, &bounds).expect("neutral items"),
        running: settings_form::view::Running::Moves,
    };
    let wire = settings_form::wire::wire_change_settings(&neutral, &bounds.settings).expect("neutral wire");
    let preset = crate::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
    assert_eq!(
        jig_client_protocol_link::translate::settings::restore_change_settings(&wire, &link).expect("client request"),
        *change
    );
    let value = c::SettingValue::Template(Box::new(template.clone()));
    let wire_value = jig_client_protocol_link::translate::settings::wire_setting_value(&value, &bounds.settings)
        .expect("client wire");
    let wire = w::SettingDocument::new(
        &bounds.settings,
        w::SettingDocumentParts {
            setting: w::Setting::Template(
                w::SettingNumber::new(&bounds.settings, w::SettingNumberParts { number: 4 }).expect("template number"),
            ),
            revision: 5,
            value: Some(wire_value),
            application: None,
            proposed: None,
        },
    )
    .expect("wire document");
    assert_eq!(
        settings_form::wire::restore_setting_document(&wire, &bounds).expect("neutral restore").value,
        Some(jig::form_setting_value(&value))
    );
    let mut smaller = bounds.settings;
    smaller.template_resources = 0;
    settings_form::wire::wire_change_settings(&neutral, &smaller).expect_err("wire resource bound");
}

fn template_resource_file(resources: Option<&str>) -> Vec<u8> {
    let (setting, value) = values().remove(1);
    let file = String::from_utf8(file_from_setting(&setting_json(setting, value))).expect("template JSON");
    match resources {
        Some(resources) => file.replace(r#""resources":[]"#, &format!(r#""resources":{resources}"#)).into_bytes(),
        None => file.replace(r#""resources":[],"#, "").into_bytes(),
    }
}

#[test]
fn template_resources_are_optional_and_bounded_connector_symbols() {
    for resources in [None, Some("[]"), Some(r#"[{"connector":65535,"symbol":65535}]"#)] {
        let file = template_resource_file(resources);
        let items = settings_file::decode(&file, &crate::limits::tools()).expect("optional symbols");
        let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Template(template) }) =
            &items.as_slice()[0].edit
        else {
            panic!("template")
        };
        let expected = if resources.is_some_and(|value| value.contains("65535")) {
            vec![c::Symbol { connector: u16::MAX, symbol: u16::MAX }]
        } else {
            vec![]
        };
        assert_eq!(template.resources.as_slice(), expected);
    }
    let resources = r#"[{"connector":0,"symbol":1},{"connector":7,"symbol":2}]"#;
    let file = template_resource_file(Some(resources));
    let mut limits = crate::limits::tools();
    limits.settings.template_resources = 2;
    settings_file::decode(&file, &limits).expect("resource count at bound");
    limits.settings.template_resources = 1;
    let why = settings_file::decode(&file, &limits).expect_err("resource count");
    assert!(core::str::from_utf8(&why).unwrap().contains("template.resources: bound 1"));
}

#[test]
fn malformed_template_resources_name_the_member_before_sending() {
    for (resources, path) in [
        ("null", "template.resources"),
        ("{}", "template.resources"),
        ("[1]", "template.resources"),
        (r#"[{"symbol":1}]"#, "template.resources.connector"),
        (r#"[{"connector":0}]"#, "template.resources.symbol"),
        (r#"[{"connector":65536,"symbol":1}]"#, "template.resources.connector"),
        (r#"[{"connector":0,"symbol":65536}]"#, "template.resources.symbol"),
        (r#"[{"connector":-1,"symbol":1}]"#, "template.resources.connector"),
        (r#"[{"connector":0,"symbol":null}]"#, "template.resources.symbol"),
        (r#"[{"connector":0,"symbol":1,"extra":0}]"#, "template.resources.extra"),
        (r#"[{"connector":0,"symbol":1,"symbol":2}]"#, "template.resources.symbol"),
    ] {
        let file = template_resource_file(Some(resources));
        let why = settings_file::decode(&file, &crate::limits::tools()).expect_err("malformed symbols");
        let why = core::str::from_utf8(&why).expect("refusal text");
        assert!(why.contains("item 1") && why.contains(path), "{resources}: {why}");
        let (mut root, call) = changed(&file, false);
        assert!(root.sends.is_empty());
        assert_eq!(root.answer(call).expect("local refusal").status, 1);
    }
}

#[test]
fn revision_show_reads_temper_set_values_at_the_recorded_revision() {
    for json in [false, true] {
        let (mut root, call) =
            fixtures::read_call(if json { &["revision", "show", "5", "--json"] } else { &["revision", "show", "5"] });
        let items = [(2, 1), (3, 0), (9, 0), (10, 7)].map(|(kind, scope)| c::RevisionItem {
            setting: c::Setting::Application { kind, scope, number: 1 },
            edit: c::RevisionEdit::Application(c::SizedEdit::Set {
                size: u32::try_from(forge_value(kind).len()).unwrap(),
            }),
            replaced: if kind == 3 { c::RevisionReplaced::Default } else { c::RevisionReplaced::NotOffered },
        });
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
                head: head(c::RevisionDecider::Release),
                items: list(items),
            })))),
        );
        for (kind, scope) in [(2, 1), (3, 0), (9, 0), (10, 7)] {
            assert!(root.answer(call).is_none(), "revision must read historical forge values before printing");
            let temper_client_protocol_wire::v1::Read::RevisionSetting(query) = forge_query(&root, call) else {
                panic!("revision-qualified read")
            };
            assert_eq!((query.kind(), query.scope(), query.number(), query.revision()), (kind, scope, 1, 5));
            root.serve_readout(call, forge_setting_page(Some(forge_value(kind))));
        }
        let answer = root.answer(call).expect("historical values joined");
        assert_eq!(answer.status, 0);
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        for needle in [
            "\"ci\":false",
            "charter-words",
            "project-words",
            "reviews every change",
            "Our way",
            "We review every change.",
        ] {
            assert!(text.contains(needle), "{text}");
        }
        assert!(text.contains(if json { "\"set\":6" } else { "\n6\n" }), "{text}");
        assert!(text.contains(if json { "\"replaced\":\"default\"" } else { "was: default" }), "{text}");
    }
}

#[test]
fn revision_show_large_temper_values_name_a_working_historical_read() {
    for json in [false, true] {
        let (mut root, call) =
            fixtures::read_call(if json { &["revision", "show", "5", "--json"] } else { &["revision", "show", "5"] });
        root.limits.tools.result_bytes = 512;
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
                head: head(c::RevisionDecider::Release),
                items: list([c::RevisionItem {
                    setting: c::Setting::Application { kind: 2, scope: 1, number: 1 },
                    edit: c::RevisionEdit::Application(c::SizedEdit::Set { size: 100 }),
                    replaced: c::RevisionReplaced::Default,
                }]),
            })))),
        );
        root.serve_readout(call, forge_setting_page(Some(forge_value(2))));
        let answer = root.answer(call).expect("bounded historical read");
        assert_eq!(answer.status, 0, "{answer:?}");
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        let command = "settings show landing-rule 1 --project 1 --revision 5";
        assert!(text.contains(command), "{text}");
        assert!(text.contains(if json { "\"too_large\":" } else { "too large," }), "{text}");
        let mut argv: Vec<_> = command.split(' ').collect();
        if json {
            argv.push("--json");
        }
        let (mut root, call) = fixtures::read_call(&argv);
        let temper_client_protocol_wire::v1::Read::RevisionSetting(query) = forge_query(&root, call) else {
            panic!("qualified historical read")
        };
        assert_eq!((query.kind(), query.scope(), query.number(), query.revision()), (2, 1, 1, 5));
        root.serve_readout(call, forge_setting_page(Some(forge_value(2))));
        let answer = root.answer(call).expect("whole historical value");
        assert_eq!(answer.status, 0);
        assert!(core::str::from_utf8(&answer.stdout).unwrap().contains("\"ci\":false"));
    }
}

#[test]
fn proposal_show_prints_the_whole_settings_reason_in_text_and_json() {
    for bytes in [900, 649] {
        let mut reason = String::from("temper enforces: review every change. your agents follow: keep reports short. ");
        let effect = " Running work keeps its current instructions until an owner moves it.";
        reason.push_str(&"x".repeat(bytes - reason.len() - effect.len()));
        reason.push_str(effect);
        for json in [false, true] {
            let mut proposal = review_proposal(None);
            let c::ReadValue::Proposal(value) = &mut proposal else { panic!("proposal") };
            value.reason = c::DocumentText::complete(words(&reason));
            let mut root = fixtures::signed_in(fixtures::PARTY);
            let call = root.submit_prepared(fixtures::call(&["proposal", "show", "p73/7"]), json, None);
            serve_review(&mut root, call, proposal);
            let answer = root.answer(call).expect("proposal shown");
            assert_eq!(answer.status, 0);
            let printed = core::str::from_utf8(&answer.stdout).expect("UTF-8 proposal");
            assert!(printed.contains(&reason), "whole reason: {printed}");
            if json {
                assert!(printed.contains("\"cut\":false"), "{printed}");
            }
        }
    }
}

#[test]
fn proposal_effect_pluralizes_zero_one_and_multiple_holders() {
    for holders in [0, 1, 2] {
        let mut root = fixtures::signed_in(fixtures::PARTY);
        let call = root.submit(&["proposal", "show", "p73/7"]);
        root.serve(
            call,
            c::ReadResult::Value(review_proposal(Some(c::SettingsReview {
                key: None,
                newest: 5,
                cut: false,
                items: list([]),
            }))),
        );
        root.serve(
            call,
            c::ReadResult::Value(c::ReadValue::Settings(Box::new(c::SettingsDocument {
                newest: 5,
                settings: list([]),
                pending: list([]),
                held: list([c::HeldRevision { head: head(c::RevisionDecider::Release), holders }]),
            }))),
        );
        let printed = root.answer(call).expect("proposal effect");
        let suffix = if holders == 1 { "" } else { "s" };
        let line = format!("revision 5: {holders} live task{suffix}\n");
        assert!(String::from_utf8_lossy(&printed.stdout).contains(&line), "{printed:?}");
    }
}

fn replaced_forge_revision(number: u32, edit: Option<c::RevisionEdit>) -> c::ReadResult {
    let mut header = head(c::RevisionDecider::Release);
    header.number = number;
    let items = match edit {
        Some(edit) => list([c::RevisionItem {
            setting: c::Setting::Application { kind: 2, scope: 1, number: 1 },
            edit,
            replaced: c::RevisionReplaced::Application { size: u32::try_from(forge_value(2).len()).unwrap() },
        }]),
        None => list([]),
    };
    c::ReadResult::Value(c::ReadValue::Revision(Box::new(c::RevisionAnswer::Revision(c::RevisionDocument {
        head: header,
        items,
    }))))
}

#[test]
fn revision_show_hydrates_replaced_values_for_every_edit() {
    for edit in
        [c::RevisionEdit::Application(c::SizedEdit::Set { size: 33 }), c::RevisionEdit::Removed, c::RevisionEdit::Reset]
    {
        for json in [false, true] {
            let (mut root, call) = fixtures::read_call(if json {
                &["revision", "show", "5", "--json"]
            } else {
                &["revision", "show", "5"]
            });
            let set = match edit {
                c::RevisionEdit::Application(c::SizedEdit::Set { .. }) => true,
                c::RevisionEdit::Application(c::SizedEdit::Remove | c::SizedEdit::Reset)
                | c::RevisionEdit::Set(_)
                | c::RevisionEdit::TooLarge(_)
                | c::RevisionEdit::Removed
                | c::RevisionEdit::Reset => false,
            };
            root.serve(call, replaced_forge_revision(5, Some(edit.clone())));
            assert_eq!(
                root.reads.get(&call).expect("read preceding history").1,
                c::Read::Revision(c::ReadRevision { number: 4 })
            );
            root.serve(call, replaced_forge_revision(4, None));
            assert_eq!(
                root.reads.get(&call).expect("skip unrelated revision").1,
                c::Read::Revision(c::ReadRevision { number: 3 })
            );
            root.serve(
                call,
                replaced_forge_revision(3, Some(c::RevisionEdit::Application(c::SizedEdit::Set { size: 33 }))),
            );
            for revision in if set { vec![5, 3] } else { vec![3] } {
                let temper_client_protocol_wire::v1::Read::RevisionSetting(query) = forge_query(&root, call) else {
                    panic!("historical read")
                };
                assert_eq!(query.revision(), revision);
                root.serve_readout(call, forge_setting_page(Some(forge_value(2))));
            }
            let answer = root.answer(call).expect("hydrated replaced value");
            assert_eq!(answer.status, 0);
            let text = core::str::from_utf8(&answer.stdout).unwrap();
            let replaced = text.split_once(if json { "\"replaced\":" } else { "was: " }).expect("previous state").1;
            assert!(replaced.contains("\"ci\":false"), "{text}");
        }
    }
}

#[test]
fn revision_show_preserves_unavailable_and_oversized_replaced_values() {
    let preset = crate::limits::tools();
    let forgotten =
        c::ReadResult::Value(c::ReadValue::Revision(Box::new(c::RevisionAnswer::Forgotten(c::RevisionForgotten {
            oldest: 5,
        }))));
    let set = replaced_forge_revision(4, Some(c::RevisionEdit::Application(c::SizedEdit::Set { size: 33 })));
    for (limits, history, value, expected) in [
        (preset, forgotten, None, " bytes"),
        (preset, set.clone(), Some(None), " bytes"),
        (jig::Limits { result_bytes: 512, ..preset }, set, Some(Some(forge_value(2))), "was: too large,"),
    ] {
        let (mut root, call) = fixtures::read_call(&["revision", "show", "5"]);
        root.limits.tools = limits;
        root.serve(call, replaced_forge_revision(5, Some(c::RevisionEdit::Reset)));
        assert_eq!(
            root.reads.get(&call).expect("preceding history").1,
            c::Read::Revision(c::ReadRevision { number: 4 })
        );
        root.serve(call, history);
        if let Some(value) = value {
            root.serve_readout(call, forge_setting_page(value));
        }
        let answer = root.answer(call).expect("metadata fallback");
        assert_eq!(answer.status, 0, "{answer:?}");
        let text = core::str::from_utf8(&answer.stdout).unwrap();
        assert!(text.contains(expected), "{text}");
        assert!(!text.contains("\"ci\":"), "must not read today's value: {text}");
    }
}

#[derive(Clone, Copy, Debug)]
enum EffortPlace {
    Main,
    Subagent,
    Charter,
    Alternative,
}

fn effort_file(place: EffortPlace, name: &str, effort: Option<&str>) -> Vec<u8> {
    let choice = fixtures::model_effort(name.as_bytes(), effort.map(str::as_bytes));
    let valid = fixtures::model_effort(fixtures::MODEL_NAME, Some(fixtures::EFFORT_NAME));
    let (setting, value) = match place {
        EffortPlace::Main | EffortPlace::Subagent => {
            let (setting, c::SettingValue::Template(mut template)) = values().remove(1) else {
                panic!("template fixture")
            };
            template.llm = Some(match place {
                EffortPlace::Main => c::LlmChoice { later: None, account: None, main: choice, subagents: list([]) },
                EffortPlace::Subagent => {
                    c::LlmChoice { later: None, account: None, main: valid, subagents: list([choice]) }
                }
                EffortPlace::Charter | EffortPlace::Alternative => unreachable!("template position"),
            });
            (setting, c::SettingValue::Template(template))
        }
        EffortPlace::Charter | EffortPlace::Alternative => {
            let (setting, c::SettingValue::Charter(mut charter)) = values().remove(0) else {
                panic!("charter fixture")
            };
            let mut selected = model();
            selected.name = choice.model;
            selected.effort = choice.effort;
            let mut valid_model = model();
            valid_model.name = valid.model;
            valid_model.effort = valid.effort;
            match place {
                EffortPlace::Charter => {
                    charter.model = selected;
                    charter.alternatives = list([]);
                }
                EffortPlace::Alternative => {
                    charter.model = valid_model.clone();
                    charter.alternatives = list([valid_model, selected]);
                }
                EffortPlace::Main | EffortPlace::Subagent => unreachable!("charter position"),
            }
            (setting, c::SettingValue::Charter(charter))
        }
    };
    file_from_setting(&setting_json(setting, value))
}

fn effort_me() -> c::Me {
    let mut me = fixtures::choice_me(fixtures::CHAT_BUDGET);
    let mut elsewhere = me.projects.get(0).expect("project").clone();
    elsewhere.project = 99;
    let chosen = elsewhere.models.get_mut(0).expect("chosen model");
    chosen.efforts = list([words("high"), words("medium")]);
    me.projects.get_mut(0).expect("first project").models = list([]);
    me.projects = list([me.projects.get(0).expect("first project").clone(), elsewhere]);
    me
}

fn effort_call(file: &[u8], propose: bool, me: c::Me, json: bool) -> (fixtures::Harness, u32) {
    let mut root = fixtures::choice_root(me);
    let argv = if propose {
        vec!["settings", "propose", "FILE", "Choose effort", "--against", "5", "--key", "effort-file"]
    } else {
        vec!["settings", "apply", "FILE", "--against", "5", "--key", "effort-file"]
    };
    let mut call = fixtures::call(&argv);
    *call.values.get_mut(0).expect("file") = Some(Values::One(Value::Text(file.into())));
    let number = root.submit_prepared(call, json, None);
    (root, number)
}

#[test]
fn settings_efforts_are_refused_at_every_choice_before_apply_or_propose_sends() {
    for place in [EffortPlace::Main, EffortPlace::Subagent, EffortPlace::Charter, EffortPlace::Alternative] {
        for propose in [false, true] {
            for (name, offered) in [("chosen-model", "high, medium"), ("plain-model", "none")] {
                let file = effort_file(place, name, Some("xhigh"));
                // Removed and reset items have no effort; the later item is numbered from one.
                let file = String::from_utf8(file).expect("file UTF-8").replace(
                    "\"items\":[",
                    r#""items":[{"setting":{"kind":"charter","number":7},"edit":"removed"},{"setting":{"kind":"rules"},"edit":"reset"},"#,
                );
                let (mut root, call) = effort_call(file.as_bytes(), propose, effort_me(), false);
                assert!(root.sends.is_empty(), "{place:?}, propose {propose}");
                assert!(root.saved.is_empty(), "a local refusal keeps no request");
                let answer = root.answer(call).expect("local effort refusal");
                assert_eq!(answer.status, 1);
                assert_eq!(
                    answer.stderr.as_ref(),
                    format!("settings file item 3: model {name} offers no effort xhigh; it offers {offered}\n")
                        .as_bytes(),
                    "{place:?}, propose {propose}"
                );
            }
        }
    }
}

#[test]
fn settings_offered_absent_and_unshown_efforts_send_unchanged() {
    for place in [EffortPlace::Main, EffortPlace::Subagent, EffortPlace::Charter, EffortPlace::Alternative] {
        for propose in [false, true] {
            for (name, effort) in [
                ("chosen-model", Some("high")),
                ("chosen-model", Some("medium")),
                ("chosen-model", None),
                ("plain-model", None),
                ("unshown-model", Some("xhigh")),
            ] {
                let file = effort_file(place, name, effort);
                let items = settings_file::decode(&file, &crate::limits::tools()).expect("items");
                let (mut root, call) = effort_call(&file, propose, effort_me(), false);
                let ask = fixtures::act_ask(&root, call);
                let expected = if propose {
                    c::Ask::Propose {
                        project: 0,
                        reason: words("Choose effort"),
                        action: c::ProposeAction::Settings(c::ProposeSettings { against: 5, items }),
                    }
                } else {
                    c::Ask::ChangeSettings(c::ChangeSettings { against: 5, items, running: c::Running::Stays })
                };
                assert_eq!(ask, &expected, "{place:?}, propose {propose}");
                if name == "unshown-model" {
                    root.answered(
                        call,
                        c::Answer::Refused {
                            refusal: c::Refusal::Llm { model: words(name), why: c::LlmWhy::Effort },
                            at: None,
                        },
                    );
                    let answer = root.answer(call).expect("engine refusal");
                    let verb = if propose { "propose" } else { "apply" };
                    assert_eq!(answer.status, 2);
                    assert_eq!(
                        answer.stderr.as_ref(),
                        format!("settings {verb}: This model cannot be chosen: {name}\n").as_bytes()
                    );
                }
            }
        }
    }
}

#[test]
fn settings_effort_names_use_the_terminal_name_cut() {
    for (name, cut) in [
        ("safe\u{1b}unsafe".to_owned(), "safe...".to_owned()),
        ("safe\u{7f}unsafe".to_owned(), "safe...".to_owned()),
        ("safe\u{80}unsafe".to_owned(), "safe...".to_owned()),
        (format!("{}é", "x".repeat(255)), format!("{}...", "x".repeat(255))),
    ] {
        for propose in [false, true] {
            for json in [false, true] {
                let mut me = effort_me();
                let model = me.projects.get_mut(1).expect("other project").models.get_mut(0).expect("model");
                model.name = words(&name);
                model.efforts = list([words(&name)]);
                let file = effort_file(EffortPlace::Main, &name, Some("xhigh"));
                let (mut root, call) = effort_call(&file, propose, me, json);
                assert!(root.sends.is_empty());
                let answer = root.answer(call).expect("cut name refusal");
                let text = format!("settings file item 1: model {cut} offers no effort xhigh; it offers {cut}");
                let expected = if json { format!(r#"{{"error":"{text}"}}"#) } else { format!("{text}\n") };
                assert_eq!(answer.status, 1);
                assert_eq!(answer.stderr.as_ref(), expected.as_bytes());
                // Supplied efforts have a 32-byte file bound; model names reach the echo bound.
                if name.len() > 32 {
                    continue;
                }
                let file = effort_file(EffortPlace::Main, "chosen-model", Some(&name));
                let (mut root, call) = effort_call(&file, propose, effort_me(), json);
                assert!(root.sends.is_empty());
                let answer = root.answer(call).expect("cut effort refusal");
                let text =
                    format!("settings file item 1: model chosen-model offers no effort {cut}; it offers high, medium");
                let expected = if json { format!(r#"{{"error":"{text}"}}"#) } else { format!("{text}\n") };
                assert_eq!(answer.stderr.as_ref(), expected.as_bytes());
            }
        }
    }
}

#[test]
fn ctl_reads_lists_and_writes_a_model_as_a_settings_file() {
    let setting = c::Setting::Model(c::SettingNumber { number: 4 });
    let value = c::SettingValue::Model(Box::new(c::CatalogueEntry {
        number: 4,
        model: model(),
        shown: Some(words("GPT-6.1 Terra")),
        description: None,
        picker: false,
        withdrawn: false,
        source: c::ModelSource::Stock,
        estimated: false,
    }));
    let (mut root, call) = fixtures::read_call(&["settings", "show", "model", "4", "--revision", "5"]);
    assert_eq!(
        root.reads.get(&call).unwrap().1,
        c::Read::Setting(c::ReadSetting { setting, revision: 5, proposal: None })
    );
    root.serve(
        call,
        c::ReadResult::Value(c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting,
            revision: 5,
            value: Some(value.clone()),
            application: None,
            proposed: None,
        }))),
    );
    let answer = root.answer(call).unwrap();
    let text = core::str::from_utf8(&answer.stdout).unwrap();
    assert!(text.starts_with("model 4, revision 5\n"), "{text}");
    let file = file_from_setting(&setting_json(setting, value.clone()));
    let decoded = settings_file::decode(&file, &crate::limits::tools()).unwrap();
    assert_eq!(
        decoded.get(0).unwrap(),
        &c::SettingItem { setting, edit: c::SettingEdit::Set(c::SettingSet { value }) }
    );
    let (root, call) = changed(&file, false);
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).unwrap() else { panic!("jig change") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") };
    assert_eq!(change.items, decoded);
    let inventory = c::ReadValue::Settings(Box::new(c::SettingsDocument {
        newest: 5,
        settings: list([c::SettingLine {
            setting,
            name: Some(c::DocumentText::complete(words("GPT-6.1 Terra"))),
            follows: c::SettingFollows::Change,
            last: Some(5),
            application: None,
        }]),
        held: list([]),
        pending: list([]),
    }));
    let bytes = jig::render::settings_text(&inventory, None, &crate::limits::tools());
    let text = core::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("model 4") && text.contains("GPT-6.1 Terra"), "{text}");
}
