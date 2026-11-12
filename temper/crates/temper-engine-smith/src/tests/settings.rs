//! Settings tools use the shared ctl vocabulary and byte offsets.
use super::{decided, decoded};
use crate::{Problem, settings_answer, settings_text};
use jig_client::contract as c;
use jig_domain_core::{SettingsAnswer, SettingsRead, documents as d, settings as s};
use skein_lib::List;
use temper_engine_domain as engine;

#[test]
fn a_settings_proposal_answer_prints_its_ctl_name() {
    for (task, proposal, name) in [(1, 2, "t1/2"), (1, 8, "t1/8"), (7, 13, "t7/13"), (0, 13, "proposal 13")] {
        let answer = engine::CallAnswer::Proposed {
            proposal,
            holder: jig_domain_tasks::ProposalHolder::Policy {
                project: 0,
                kind: jig_domain_tasks::ProposalKind::Settings,
            },
        };
        let (body, error) = if task == 0 {
            crate::answer(&answer)
        } else {
            crate::answer_for_task(&answer, b"propose_settings", crate::DOC_ANSWER, task, crate::ForgeReadBounds::STOCK)
        }
        .into_parts();
        assert!(!error);
        assert_eq!(
            body.as_ref(),
            format!("proposed as {name} for the deployment's owners to decide: it waits as a card in this chat and in Needs you, and nothing changes until an owner accepts it. Tell the person so, without {name}.").as_bytes(),
            "task {task}'s proposal keeps its proposer and shared counter"
        );
    }
}

#[test]
fn every_settings_read_form_and_offset_decodes() {
    for (input, read, from) in [
        (br"{}".as_slice(), SettingsRead::Inventory, 0),
        (br#"{"from":7}"#, SettingsRead::Inventory, 7),
        (
            br#"{"setting":{"kind":"charter","number":2}}"#,
            SettingsRead::Setting { setting: s::Setting::Charter(2), revision: 0 },
            0,
        ),
        (
            br#"{"setting":{"kind":"charter","number":2},"revision":3,"from":8}"#,
            SettingsRead::Setting { setting: s::Setting::Charter(2), revision: 3 },
            8,
        ),
        (
            br#"{"setting":{"kind":"template","number":1}}"#,
            SettingsRead::Setting { setting: s::Setting::Template(1), revision: 0 },
            0,
        ),
        (
            br#"{"setting":{"kind":"policy","number":1}}"#,
            SettingsRead::Setting { setting: s::Setting::Policy(1), revision: 0 },
            0,
        ),
        (br#"{"setting":{"kind":"rules"}}"#, SettingsRead::Setting { setting: s::Setting::Rules, revision: 0 }, 0),
        (
            br#"{"setting":{"kind":"landing-rule","number":1}}"#,
            SettingsRead::Setting { setting: s::Setting::Application { kind: 1, scope: 0, number: 1 }, revision: 0 },
            0,
        ),
        (
            br#"{"setting":{"kind":"landing-rule","number":2,"project":3}}"#,
            SettingsRead::Setting { setting: s::Setting::Application { kind: 2, scope: 3, number: 2 }, revision: 0 },
            0,
        ),
        (
            br#"{"setting":{"kind":"forge-charter","number":1}}"#,
            SettingsRead::Setting { setting: s::Setting::Application { kind: 3, scope: 0, number: 1 }, revision: 0 },
            0,
        ),
        (
            br#"{"setting":{"kind":"project-policy","project":3}}"#,
            SettingsRead::Setting { setting: s::Setting::ProjectPolicy { project: 3 }, revision: 0 },
            0,
        ),
        (
            br#"{"setting":{"kind":"template-goal","number":4}}"#,
            SettingsRead::Setting { setting: s::Setting::Application { kind: 11, scope: 0, number: 4 }, revision: 0 },
            0,
        ),
        (br#"{"revision":4,"from":9}"#, SettingsRead::Revision { number: 4 }, 9),
        (br#"{"history":{},"from":10}"#, SettingsRead::Revisions { before: None }, 10),
        (br#"{"history":{"before":4},"from":11}"#, SettingsRead::Revisions { before: Some(4) }, 11),
    ] {
        assert_eq!(decided(b"settings", input).expect("valid read").tool, engine::Tool::Settings { read, from });
    }
}
#[test]
fn settings_bad_fields_are_refused_by_name() {
    for spec in &crate::calls::SPECS {
        if [b"settings".as_slice(), b"propose_settings"].contains(&spec.name) {
            assert!(skein_lib::bytes::find(spec.schema, br#""description":"Charter/template/policy/model/landing-rule/forge-charter/charter-words number; template-goal: template's; else none.""#).is_some());
        }
    }
    for (input, why) in [
        (br#"{"from":-1}"#.as_slice(), Problem::Range(b"from")),
        (br#"{"revision":0}"#, Problem::Range(b"revision")),
        (br#"{"setting":4}"#, Problem::Type(b"setting")),
        (br#"{"setting":{"kind":"charter"}}"#, Problem::Needs { kind: b"charter", member: b"number" }),
        (br#"{"setting":{"kind":"template"}}"#, Problem::Needs { kind: b"template", member: b"number" }),
        (br#"{"setting":{"kind":"policy"}}"#, Problem::Needs { kind: b"policy", member: b"number" }),
        (br#"{"setting":{"kind":"model"}}"#, Problem::Needs { kind: b"model", member: b"number" }),
        (br#"{"setting":{"kind":"landing-rule"}}"#, Problem::Needs { kind: b"landing-rule", member: b"number" }),
        (br#"{"setting":{"kind":"forge-charter"}}"#, Problem::Needs { kind: b"forge-charter", member: b"number" }),
        (br#"{"setting":{"kind":"template-goal"}}"#, Problem::Needs { kind: b"template-goal", member: b"number" }),
        (br#"{"setting":{"kind":"template-goal","number":4,"project":1}}"#, Problem::Range(b"project")),
        (br#"{"setting":{"kind":"rules","number":1}}"#, Problem::Range(b"number")),
        (br#"{"setting":{"kind":"charter","number":1,"project":1}}"#, Problem::Range(b"project")),
        (br#"{"setting":{"kind":"forge-charter","number":1,"project":1}}"#, Problem::Range(b"project")),
        (br#"{"setting":{"kind":"landing-rule","number":1,"project":0}}"#, Problem::Range(b"project")),
        (br#"{"setting":{"kind":"project-policy"}}"#, Problem::Needs { kind: b"project-policy", member: b"project" }),
        (br#"{"setting":{"kind":"project-ways"}}"#, Problem::Needs { kind: b"project-ways", member: b"project" }),
        (
            br#"{"setting":{"kind":"project-questions"}}"#,
            Problem::Needs { kind: b"project-questions", member: b"project" },
        ),
        (br#"{"setting":{"kind":"unknown"}}"#, Problem::Range(b"kind")),
        (br#"{"setting":{"kind":"unknown","number":1}}"#, Problem::Range(b"kind")),
        (br#"{"setting":{"kind":"project-policy","project":0}}"#, Problem::Range(b"project")),
        (br#"{"setting":{"kind":"project-policy","project":1,"number":1}}"#, Problem::Range(b"number")),
        (br#"{"history":[],"from":0}"#, Problem::Type(b"history")),
        (br#"{"history":{"before":0}}"#, Problem::Range(b"before")),
        (br#"{"history":{},"revision":1}"#, Problem::Range(b"history")),
        (br#"{"setting":{"kind":"charter-words"}}"#, Problem::Needs { kind: b"charter-words", member: b"number" }),
        (br#"{"setting":{"kind":"charter-words","number":4,"project":7}}"#, Problem::Range(b"project")),
    ] {
        assert_eq!(decoded(b"settings", input).expect_err("invalid read"), why);
    }
    for (input, field) in [
        (br#"{"items":[],"reason":"why"}"#.as_slice(), b"against".as_slice()),
        (br#"{"against":1,"items":[]}"#, b"reason"),
    ] {
        assert_eq!(decoded(b"propose_settings", input).expect_err("missing proposal field"), Problem::Missing(field));
    }
    let crate::Decoded::Answered(answer) = decoded(
        b"propose_settings",
        br#"{"against":1,"reason":"why","items":[{"setting":{"kind":"rules"},"edit":"wrong"}]}"#,
    )
    .expect("bounded decode error") else {
        panic!("bad edit")
    };
    let (body, error) = answer.into_parts();
    assert!(error);
    let mut named = false;
    for word in body.windows(4) {
        named |= word == b"edit";
    }
    assert!(named);
}
#[test]
fn settings_proposal_items_use_the_file_decoder() {
    let body = decided(b"propose_settings", br#"{"against":1,"reason":"restore the default","items":[{"setting":{"kind":"charter","number":1},"edit":"reset"}]}"#).expect("proposal");
    assert_eq!(
        body.tool,
        engine::Tool::ProposeSettings {
            more: false,
            kinds: Box::default(),
            against: 1,
            reason: b"restore the default".as_slice().into(),
            items: Box::new([s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Reset }])
        }
    );
}
fn inventory(count: u32) -> SettingsAnswer {
    let mut settings = List::with_capacity(count);
    for number in 1..=count {
        settings
            .push(d::SettingLine {
                setting: s::Setting::Charter(number),
                name: None,
                follows: d::Follows::Change,
                last: Some(5),
                application: None,
            })
            .expect("inventory count");
    }
    SettingsAnswer::Inventory(d::Settings {
        newest: 5,
        settings,
        held: List::with_capacity(0),
        pending: List::with_capacity(0),
    })
}
#[test]
fn settings_inventory_pages_continue_at_the_exact_offset_and_end() {
    let answer = inventory(80);
    let whole = settings_text(&answer, None, &[]).expect("inventory text");
    let no_models = b"models you can offer: none; no account can run a model yet\n";
    let mut expected = skein_lib::Writer::new(whole.len().checked_add(no_models.len()).expect("inventory size"));
    expected.put(&whole).expect("base inventory");
    expected.put(no_models).expect("empty offer");
    let whole = expected.finish();
    let mut from = 0;
    let mut combined = skein_lib::Writer::new(whole.len());
    let mut pages = 0_u32;
    loop {
        let (body, error) = settings_answer(&answer, from, None, &[], None).into_parts();
        assert!(!error);
        assert!(body.len() <= usize::try_from(crate::DOC_ANSWER).expect("host bound"));
        let mut marker = None;
        for (index, bytes) in body.windows(6).enumerate() {
            if bytes == b"next: " {
                marker = Some(index);
            }
        }
        if let Some(marker) = marker {
            let page = body.get(..marker).expect("page before marker");
            assert!(page.ends_with(b"\n"));
            let digits = body
                .get(marker.checked_add(6).expect("marker length")..body.len().checked_sub(1).expect("line end"))
                .expect("next offset");
            let next = u32::try_from(crate::decimal(digits, b"next").expect("offset")).expect("u32 offset");
            assert!(next > from);
            let count = usize::try_from(next.checked_sub(from).expect("offset advances")).expect("page bytes");
            combined.put(page.get(..count).expect("source bytes before separator")).expect("reconstructed page");
            from = next;
            pages += 1;
        } else {
            assert!(body.ends_with(b"end\n"));
            combined
                .put(body.get(..body.len().checked_sub(4).expect("last marker")).expect("page"))
                .expect("last page");
            break;
        }
    }
    assert!(pages >= 1, "the long inventory requires a continuation");
    assert_eq!(combined.finish(), whole);
    assert_eq!(settings_answer(&answer, u32::MAX, None, &[], None).into_parts().0.as_ref(), b"end\n");
}
#[test]
fn engine_and_ctl_render_the_same_settings_inventory() {
    let mut rows = List::with_capacity(2);
    for number in 1..=2 {
        rows.push(c::SettingLine {
            setting: c::Setting::Charter(c::SettingNumber { number }),
            name: None,
            follows: c::SettingFollows::Change,
            last: Some(5),
            application: None,
        })
        .expect("two rows");
    }
    let value = c::ReadValue::Settings(Box::new(c::SettingsDocument {
        newest: 5,
        settings: rows,
        held: List::with_capacity(0),
        pending: List::with_capacity(0),
    }));
    assert_eq!(
        settings_text(&inventory(2), None, &[]).expect("engine text"),
        temper_client_tools::jig::render::settings_text(&value, None, &temper_client_tools::limits::tools())
    );
}
#[test]
fn settings_inbox_outcomes_keep_the_revision_and_rejection_reason() {
    use jig_domain_tasks as tasks;
    for (outcome, expected) in [
        (tasks::SettingsOutcome::Accepted { revision: 7 }, "accepted: revision 7 made"),
        (tasks::SettingsOutcome::Rejected, "rejected: keep it"),
        (tasks::SettingsOutcome::Stale { newest: 8 }, "withdrawn as stale: the newest is 8"),
    ] {
        let word = tasks::Word {
            number: 1,
            from: tasks::Party::Deployment { project: 0 },
            kind: tasks::MessageKind::SettingsDecision { proposal: 1, outcome },
            words: b"keep it".as_slice().into(),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        };
        assert_eq!(engine::inbox_words(1, &word).as_ref(), expected.as_bytes());
    }
}

#[test]
fn settings_pages_price_escapes_cut_long_lines_and_keep_utf8_offsets() {
    let quoted = "\"".repeat(800);
    let first = settings_form::page::page_bounded(quoted.as_bytes(), 0, 1558).expect("quoted text");
    assert_eq!(first.as_ref(), format!("{}\nnext: 768\n", "\"".repeat(768)).as_bytes());
    let last = settings_form::page::page(quoted.as_bytes(), 768).expect("last quoted page");
    assert_eq!(last.as_ref(), format!("{}\nend\n", "\"".repeat(32)).as_bytes());
    let unicode = format!("{}λ{}", "x".repeat(1535), "z".repeat(100));
    let page = settings_form::page::page_bounded(unicode.as_bytes(), 0, 1558).expect("UTF-8 boundary cut");
    assert!(page.ends_with(b"next: 1535\n"));
    assert_eq!(
        settings_form::page::page(unicode.as_bytes(), 1535).expect("last character").as_ref(),
        format!("λ{}\nend\n", "z".repeat(100)).as_bytes()
    );
    assert_eq!(settings_form::page::page(unicode.as_bytes(), 1536), Err(settings_form::page::Error::Offset));
    let lines = format!("short\n{}", "q".repeat(1800));
    assert_eq!(
        settings_form::page::page(lines.as_bytes(), 0).expect("whole lines").as_ref(),
        format!("{lines}\nend\n").as_bytes()
    );
}

#[test]
fn a_large_settings_decoder_error_is_bounded_host_feedback() {
    let key = "x".repeat(65_536 - 128);
    let input =
        format!("{{\"against\":1,\"reason\":\"why\",\"items\":[{{\"setting\":{{\"kind\":\"rules\"}},\"{key}\":0}}]}}");
    let crate::Decoded::Answered(answer) =
        decoded(b"propose_settings", input.as_bytes()).expect("bounded malformed input")
    else {
        panic!("unknown item field")
    };
    let (body, error) = answer.into_parts();
    assert!(error);
    assert!(body.len() <= smith_domain_run::HostAnswer::CAPACITY);
    assert!(body.starts_with(b"settings file item 1:"));
}

#[test]
fn settings_neutral_form_preserves_ctls_bounds() {
    let neutral = settings_form::Limits::standard();
    let ctl = temper_client_tools::limits::tools();
    assert_eq!(neutral.ask_bytes, ctl.ask_bytes);
    assert_eq!(neutral.result_bytes, ctl.result_bytes);
    assert_eq!(neutral.text, ctl.words, "the settings form keeps the client's text");
    assert_eq!(neutral.settings.change_settings_items, ctl.settings.change_settings_items, "change_settings_items");
    assert_eq!(neutral.settings.charter_alternatives, ctl.settings.charter_alternatives, "charter_alternatives");
    assert_eq!(neutral.settings.charter_instructions, ctl.settings.charter_instructions, "charter_instructions");
    assert_eq!(neutral.settings.field_rule_name, ctl.settings.field_rule_name, "field_rule_name");
    assert_eq!(neutral.settings.llm_choice_subagents, ctl.settings.llm_choice_subagents, "llm_choice_subagents");
    assert_eq!(
        neutral.settings.model_effort_effort_some, ctl.settings.model_effort_effort_some,
        "model_effort_effort_some"
    );
    assert_eq!(neutral.settings.model_effort_model, ctl.settings.model_effort_model, "model_effort_model");
    assert_eq!(neutral.settings.model_set_names_names, ctl.settings.model_set_names_names, "model_set_names_names");
    assert_eq!(
        neutral.settings.model_set_names_names_item, ctl.settings.model_set_names_names_item,
        "model_set_names_names_item"
    );
    assert_eq!(neutral.settings.authority_grants, ctl.settings.authority_grants, "authority_grants");
    assert_eq!(
        neutral.settings.authority_note_resources, ctl.settings.authority_note_resources,
        "authority_note_resources"
    );
    // Eighteen executor places fit the whole reference delegation.
    assert_eq!(neutral.settings.delegation_kinds, 18);
    assert_eq!(neutral.settings.delegation_kinds, ctl.settings.delegation_kinds, "delegation_kinds");
    assert_eq!(neutral.settings.part_key_path_value, ctl.settings.part_key_path_value, "part_key_path_value");
    assert_eq!(
        neutral.settings.part_key_path_value_item, ctl.settings.part_key_path_value_item,
        "part_key_path_value_item"
    );
    assert_eq!(
        neutral.settings.pattern_end_exact_value, ctl.settings.pattern_end_exact_value,
        "pattern_end_exact_value"
    );
    assert_eq!(neutral.settings.pattern_end_open_value, ctl.settings.pattern_end_open_value, "pattern_end_open_value");
    assert_eq!(neutral.settings.pattern_segments, ctl.settings.pattern_segments, "pattern_segments");
    assert_eq!(neutral.settings.pattern_segments_item, ctl.settings.pattern_segments_item, "pattern_segments_item");
    assert_eq!(neutral.settings.rules_implies, ctl.settings.rules_implies, "rules_implies");
    assert_eq!(neutral.settings.rules_requirements, ctl.settings.rules_requirements, "rules_requirements");
    assert_eq!(
        neutral.settings.setting_model_effort_some, ctl.settings.setting_model_effort_some,
        "setting_model_effort_some"
    );
    assert_eq!(neutral.settings.setting_model_efforts, ctl.settings.setting_model_efforts, "setting_model_efforts");
    assert_eq!(
        neutral.settings.setting_model_efforts_item, ctl.settings.setting_model_efforts_item,
        "setting_model_efforts_item"
    );
    assert_eq!(neutral.settings.setting_model_name, ctl.settings.setting_model_name, "setting_model_name");
    assert_eq!(
        neutral.settings.setting_model_program_some, ctl.settings.setting_model_program_some,
        "setting_model_program_some"
    );
    assert_eq!(
        neutral.settings.setting_policy_permissions, ctl.settings.setting_policy_permissions,
        "setting_policy_permissions"
    );
    assert_eq!(
        neutral.settings.setting_policy_projections, ctl.settings.setting_policy_projections,
        "setting_policy_projections"
    );
    assert_eq!(
        neutral.settings.setting_policy_requirements, ctl.settings.setting_policy_requirements,
        "setting_policy_requirements"
    );
    assert_eq!(neutral.settings.setting_policy_roles, ctl.settings.setting_policy_roles, "setting_policy_roles");
    assert_eq!(
        neutral.settings.setting_policy_templates, ctl.settings.setting_policy_templates,
        "setting_policy_templates"
    );
    assert_eq!(
        neutral.settings.task_contract_document_approval_state, ctl.settings.task_contract_document_approval_state,
        "task_contract_document_approval_state"
    );
    assert_eq!(
        neutral.settings.task_contract_document_verdict_choices, ctl.settings.task_contract_document_verdict_choices,
        "task_contract_document_verdict_choices"
    );
    assert_eq!(neutral.settings.verdict_choice_fields, ctl.settings.verdict_choice_fields, "verdict_choice_fields");
}
#[test]
fn settings_neutral_reads_preserve_ctls_bounds() {
    let neutral = settings_form::Limits::standard();
    let ctl = temper_client_tools::limits::tools();
    assert_eq!(neutral.settings.owned_record, ctl.settings.owned_record, "owned_record");
    assert_eq!(
        neutral.settings.revision_document_items, ctl.settings.revision_document_items,
        "revision_document_items"
    );
    assert_eq!(neutral.settings.revision_row_items, ctl.settings.revision_row_items, "revision_row_items");
    assert_eq!(neutral.settings.revisions_page_rows, ctl.settings.revisions_page_rows, "revisions_page_rows");
    assert_eq!(neutral.settings.settings_document_held, ctl.settings.settings_document_held, "settings_document_held");
    assert_eq!(
        neutral.settings.settings_document_pending, ctl.settings.settings_document_pending,
        "settings_document_pending"
    );
    assert_eq!(
        neutral.settings.settings_document_settings, ctl.settings.settings_document_settings,
        "settings_document_settings"
    );
}
#[test]
fn settings_refusals_keep_ctls_words_without_a_client_dependency() {
    use jig_domain_people::{Lack, LimitKind, Malformed, Refusal as R};
    for (core, client) in [
        (R::Stale { newest: 3 }, c::Refusal::Stale(c::RefusalStale { newest: 3 })),
        (
            R::NotWellFormed { at: 0, why: Malformed::SettingsUnchanged },
            c::Refusal::Malformed { at: 0, why: c::Malformed::SettingsUnchanged },
        ),
        (
            R::Authority { lacking: Lack::Approver, proposable: None },
            c::Refusal::Authority { lacking: c::Lacking::Approver, proposable: Some(false) },
        ),
        (
            R::Limit { which: LimitKind::SettingsProposals },
            c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::SettingsProposals } },
        ),
    ] {
        assert_eq!(
            crate::settings::refusal(&core),
            temper_client_tools::jig::render::settings_refusal(&client, temper_client_tools::limits::TEXT)
        );
    }
    for (core, client) in [
        (jig_domain_people::SettingFault::Shape, c::SettingFault::Shape),
        (jig_domain_people::SettingFault::Count, c::SettingFault::Count),
        (jig_domain_people::SettingFault::Reference, c::SettingFault::Reference),
        (jig_domain_people::SettingFault::Rules, c::SettingFault::Rules),
        (jig_domain_people::SettingFault::Budget, c::SettingFault::Budget),
        (jig_domain_people::SettingFault::Grace, c::SettingFault::Grace),
        (jig_domain_people::SettingFault::Owner, c::SettingFault::Owner),
    ] {
        for (setting, client_setting) in [
            (s::Setting::Charter(1), c::Setting::Charter(c::SettingNumber { number: 1 })),
            (s::Setting::ProjectPolicy { project: 1 }, c::Setting::ProjectPolicy { project: 1 }),
        ] {
            let core = R::NotWellFormed { at: 2, why: Malformed::Settings { setting, fault: core, detail: None } };
            let client = c::Refusal::Malformed {
                at: 2,
                why: c::Malformed::Settings(c::MalformedSettings {
                    setting: client_setting,
                    fault: client,
                    detail: None,
                }),
            };
            assert_eq!(
                crate::settings::refusal(&core),
                temper_client_tools::jig::render::settings_refusal(&client, temper_client_tools::limits::TEXT)
            );
        }
    }
}

fn landing_value() -> temper_engine_domain_forge::settings::Value {
    use temper_engine_domain_forge::{GateLast, GatePattern, settings};
    settings::Value::Rule(settings::LandingRule {
        pattern: GatePattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: true,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    })
}
fn application_item(bytes: Box<[u8]>) -> s::Item {
    s::Item {
        setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
        edit: s::Edit::Set(s::Value::Application(bytes)),
    }
}
#[test]
fn settings_proposals_carry_typed_landing_values() {
    use temper_engine_domain_forge::{GateLast, GatePattern, settings};
    for (body, kind, scope, value) in [
        (
            br#"{"against":1,"reason":"landing","items":[{"setting":{"kind":"landing-rule","number":1},"set":{"pattern":{"segments":["666f726765"],"last":{"open":""}},"enforce":true,"ci":true,"up_to_date":false,"gates":[],"approvals":[]}}]}"#.as_slice(),
            1,
            0,
            landing_value(),
        ),
        (
            br#"{"against":1,"reason":"landing","items":[{"setting":{"kind":"landing-rule","project":7,"number":1},"set":{"pattern":{"segments":["6272616e6368"],"last":{"exact":"6d61696e"}},"enforce":true,"ci":true,"up_to_date":false,"gates":[],"approvals":[]}}]}"#,
            2,
            7,
            settings::Value::Rule(settings::LandingRule {
                pattern: GatePattern {
                    segments: Box::new([Box::from(b"branch".as_slice())]),
                    last: GateLast::Exact(Box::from(b"main".as_slice())),
                },
                enforce: true,
                ci: true,
                up_to_date: false,
                gates: Box::new([]),
                approvals: Box::new([]),
            }),
        ),
        (
            br#"{"against":1,"reason":"words","items":[{"setting":{"kind":"charter-words","number":1},"set":"Reviews changes"}]}"#,
            9, 0, settings::Value::CharterWords(b"Reviews changes".as_slice().into()),
        ),
        (
            br#"{"against":1,"reason":"areas","items":[{"setting":{"kind":"project-areas","project":7},"set":{"areas":[{"name":"tests","words":"C tests","owner":{"role":0}}]}}]}"#,
            12, 7, settings::Value::ProjectAreas(settings::ProjectAreas { areas: Box::new([settings::Area { name: b"tests".as_slice().into(), words: b"C tests".as_slice().into(), owner: settings::AreaOwner::Role(0) }]) }),
        ),
        (
            br#"{"against":1,"reason":"triager","items":[{"setting":{"kind":"project-triager","project":7},"set":{"owner":{"charter":14}}}]}"#,
            13, 7, settings::Value::ProjectTriager(settings::ProjectTriager { owner: settings::AreaOwner::Charter(14) }),
        ),
        (
            br#"{"against":1,"reason":"words","items":[{"setting":{"kind":"project-words","project":7},"set":{"headline":"Our way","lede":"We review changes."}}]}"#,
            10, 7, settings::Value::ProjectWords(settings::ProjectWords { headline: b"Our way".as_slice().into(), lede: b"We review changes.".as_slice().into() }),
        ),
    ] {
        let bytes = crate::kinds::encode(&value);
        let size = u32::try_from(bytes.len()).expect("fixture encoding");
        let call = decided(b"propose_settings", body).expect("landing proposal");
        let engine::Tool::ProposeSettings { kinds, .. } = call.tool else { panic!("settings proposal") };
        assert_eq!(
            kinds.as_ref(),
            &[engine::KindValue {
                kind,
                scope,
                number: 1,
                value: settings::SizedValue { value, size }
            }]
        );
    }
}
#[test]
fn settings_proposals_refuse_cut_or_trailing_value_bytes() {
    let bytes = crate::kinds::encode(&landing_value());
    let cut = &bytes[..bytes.len() - 1];
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).expect("one trailing byte"));
    trailing.put(&bytes).expect("encoding room");
    trailing.put(&[0]).expect("trailing byte room");
    let trailing = trailing.finish();
    for malformed in [cut, trailing.as_ref()] {
        assert_eq!(crate::settings::kind_values(&[application_item(malformed.into())]), Err(Problem::Range(b"items")));
    }
}

#[test]
fn a_runs_proposal_carries_a_project_policy_item() {
    let policy = r#"{"escalation_role":null,"ceiling":{"models":"all","tools":1,"grants":[],"delegation":{"kinds":[],"tasks":1,"depth":1},"spend":100,"deadline":null,"notes":0,"note_resources":[]},"projections":[],"period_spend":200,"roles":[],"requirements":[],"permissions":[],"templates":[]}"#;
    let body = format!(
        r#"{{"against":1,"reason":"project policy","items":[{{"setting":{{"kind":"project-policy","project":1}},"set":{policy}}}]}}"#
    );
    let call = decided(b"propose_settings", body.as_bytes()).expect("project policy proposal");
    let engine::Tool::ProposeSettings { items, .. } = call.tool else { panic!("settings proposal") };
    assert_eq!(items.len(), 1);
    let item = items.first().expect("project policy item");
    assert_eq!(item.setting, s::Setting::ProjectPolicy { project: 1 });
    let s::Edit::Set(s::Value::Policy(value)) = &item.edit else { panic!("whole policy value") };
    assert_eq!(value.policy.period_spend, 200);
    assert_eq!(value.policy.ceiling.budget.spend, 100);
}

fn settings_ceiling(grant_count: u32, kind_count: u32) -> jig_domain_authority::Authority {
    use jig_domain_authority as authority;
    let mut ceiling = authority::Authority {
        tools: authority::Tools(1),
        models: authority::Models::ALL,
        grants: Box::default(),
        delegation: authority::Delegation { kinds: Box::default(), tasks: 18, depth: 4 },
        budget: authority::Budget { spend: 40, deadline: None },
        notes: authority::Scopes(0),
        note_resources: Box::default(),
    };
    let mut grants = List::with_capacity(grant_count);
    for kind in [1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 16, 17, 18, 19]
        .into_iter()
        .take(usize::try_from(grant_count).expect("fixture grant count"))
    {
        grants
            .push(authority::Grant {
                connector: temper_client_protocol_wire::FORGE,
                kind,
                pattern: authority::Pattern {
                    segments: Box::new([Box::from(b"forge".as_slice())]),
                    last: authority::Last::Open(Box::default()),
                },
            })
            .expect("fixture grants");
    }
    ceiling.grants = grants.into_boxed();
    let mut kinds = List::with_capacity(kind_count);
    for kind in [
        authority::Executor::Charter(1),
        authority::Executor::Charter(2),
        authority::Executor::Charter(4),
        authority::Executor::Charter(6),
        authority::Executor::Procedure(2),
        authority::Executor::Procedure(3),
        authority::Executor::Procedure(4),
        authority::Executor::Procedure(6),
        authority::Executor::Job(4),
        authority::Executor::Charter(7),
        authority::Executor::Charter(8),
        authority::Executor::Charter(9),
        authority::Executor::Charter(10),
        authority::Executor::Charter(11),
        authority::Executor::Charter(12),
        authority::Executor::Charter(13),
        authority::Executor::Charter(14),
        authority::Executor::Charter(15),
    ]
    .into_iter()
    .take(usize::try_from(kind_count).expect("fixture kind count"))
    {
        kinds.push(kind).expect("fixture kinds");
    }
    ceiling.delegation.kinds = kinds.into_boxed();
    ceiling
}

#[test]
fn the_settings_tool_restores_the_stocks_policy_and_rules() {
    use jig_domain_authority as authority;
    use settings_form::view as form;
    let limits = settings_form::Limits::standard();
    // The closer procedure and close job add two executor places in each case.
    for (grant_count, kind_count) in [(14, 11), (16, 18)] {
        let ceiling = settings_ceiling(grant_count, kind_count);
        let policy = jig_domain_core::ConfiguredPolicy {
            policy: authority::Policy {
                escalation_role: Some(0),
                ceiling: ceiling.clone(),
                projections: ceiling.grants.clone(),
                period_spend: 40,
                roles: Box::new([authority::Role {
                    number: 0,
                    authority: ceiling.clone(),
                    period_spend: 40,
                    requests: authority::Requests::ALL,
                    decides: authority::Proposals::ALL,
                }]),
                requirements: Box::default(),
            },
            permissions: Box::new([jig_domain_people::PermissionRole {
                connector: temper_client_protocol_wire::FORGE,
                permission: 1,
                role: 0,
            }]),
            templates: Box::default(),
        };
        let rules = authority::Rules {
            ceiling,
            period_spend: 40,
            minimum_run_spend: 1,
            maximum_run_spend: 40,
            implies: authority::Implies::new(Box::default(), 0).expect("empty stock implications"),
            requirements: Box::default(),
        };
        for (setting, value) in [
            (s::Setting::Policy(1), s::Value::Policy(Box::new(policy))),
            (s::Setting::Rules, s::Value::Rules(Box::new(rules))),
        ] {
            let document = jig_domain_core::reads::SettingDocument {
                setting,
                revision: 1,
                value: Some(value),
                application: None,
                proposed: None,
            };
            let wire =
                jig_protocol_engine_clients::translate::settings::setting_document(&document, &[], &limits.settings)
                    .expect("settings document fits wire bounds");
            let restored = settings_form::wire::restore_setting_document(&wire, &limits)
                .expect("stock and eighteen-entry settings restore without document limit");
            assert_eq!(restored.revision, 1);
            let ceiling = match restored.value.expect("setting value") {
                form::SettingValue::Policy(policy) => {
                    assert_eq!(restored.setting, form::Setting::Policy(form::SettingNumber { number: 1 }));
                    assert_eq!(policy.projections.len(), grant_count);
                    let role = policy.roles.get(0).expect("stock role");
                    assert_eq!(role.authority, policy.ceiling);
                    policy.ceiling
                }
                form::SettingValue::Rules(rules) => {
                    assert_eq!(restored.setting, form::Setting::Rules);
                    rules.ceiling
                }
                form::SettingValue::Model(_)
                | form::SettingValue::Application(_)
                | form::SettingValue::Charter(_)
                | form::SettingValue::Template(_) => {
                    panic!("policy or rules");
                }
            };
            assert_eq!(ceiling.grants.len(), grant_count);
            assert_eq!(ceiling.delegation.kinds.len(), kind_count);
            settings_text(&SettingsAnswer::Setting(document), None, &[]).expect("settings tool renders the document");
        }
    }
}

#[test]
fn a_runs_proposal_carries_the_forges_charter_number() {
    use temper_engine_domain_forge::settings::{TemplateGoal, Value};
    for (input, kind, number, value) in [
        (br#"{"against":1,"reason":"the change charter","items":[{"setting":{"kind":"forge-charter","number":1},"set":2}]}"#.as_slice(), 3, 1, Value::Charter(2)),
        (br#"{"against":1,"reason":"the goal's work","items":[{"setting":{"kind":"template-goal","number":4},"set":{"design":true,"close_out":false}}]}"#, 11, 4, Value::Goal(TemplateGoal { design: true, close_out: false })),
    ] {
        let call = decided(b"propose_settings", input).expect("typed proposal");
        let engine::Tool::ProposeSettings { items, kinds, .. } = call.tool else { panic!("settings proposal") };
        assert_eq!(items[0].setting, s::Setting::Application { kind, scope: 0, number });
        assert_eq!(kinds.len(), 1);
        assert_eq!((kinds[0].kind, kinds[0].scope, kinds[0].number), (kind, 0, number));
        assert_eq!(kinds[0].value.value, value);
    }
    let crate::Decoded::Answered(answer) = decoded(b"propose_settings", br#"{"against":1,"reason":"bad scope","items":[{"setting":{"kind":"template-goal","number":4,"project":1},"set":{"design":true,"close_out":false}}]}"#).expect("form refusal") else { panic!("project refused") };
    let (body, error) = answer.into_parts();
    assert!(error);
    assert_eq!(body.as_ref(), b"settings file item 1: template-goal.project: unknown member");
}

#[test]
fn settings_proposals_carry_typed_project_questions() {
    for (input, kind, value) in [
        (
            br#"{"against":1,"reason":"questions","items":[{"setting":{"kind":"project-questions","project":7},"set":"foreman"}]}"#.as_slice(),
            6,
            temper_engine_domain_forge::settings::Value::Questions(temper_engine_domain_forge_change::Route::Foreman),
        ),
        (
            br#"{"against":1,"reason":"classes","items":[{"setting":{"kind":"project-classes","project":7},"set":{"items":[{"kind":"implementation","class":"decide"}],"sets":[]}}]}"#,
            8,
            temper_engine_domain_forge::settings::Value::Classes(temper_protocol_reaches::Classes {
                items: Box::new([temper_protocol_reaches::Item {
                    subject: temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Implementation),
                    words: Box::default(),
                    class: temper_protocol_reaches::Class::Decide,
                }]),
                sets: Box::default(),
            }),
        ),
    ] {
        let call = decided(b"propose_settings", input).expect("project settings proposal");
        let engine::Tool::ProposeSettings { kinds, .. } = call.tool else { panic!("settings proposal") };
        let size = u32::try_from(crate::kinds::encode(&value).len()).expect("project setting encoding");
        assert_eq!(
            kinds.as_ref(),
            &[engine::KindValue {
                kind,
                scope: 7,
                number: 1,
                value: temper_engine_domain_forge::settings::SizedValue { value, size }
            }]
        );
    }
}
#[test]
fn project_settings_reads_require_project_and_refuse_number() {
    for (kind, number) in [
        ("project-ways", 7),
        ("project-questions", 6),
        ("project-classes", 8),
        ("project-words", 10),
        ("project-areas", 12),
        ("project-triager", 13),
    ] {
        let bytes = format!(r#"{{"setting":{{"kind":"{kind}","project":7}},"revision":2}}"#);
        let call = decided(b"settings", bytes.as_bytes()).expect("project read");
        assert_eq!(
            call.tool,
            engine::Tool::Settings {
                read: SettingsRead::Setting {
                    setting: s::Setting::Application { kind: number, scope: 7, number: 1 },
                    revision: 2
                },
                from: 0
            }
        );
        for (suffix, expected) in [
            ("", Problem::Needs { kind: kind.as_bytes(), member: b"project" }),
            (",\"project\":7,\"number\":1", Problem::Range(b"number")),
            (",\"project\":0", Problem::Range(b"project")),
        ] {
            let bytes = format!(r#"{{"setting":{{"kind":"{kind}"{suffix}}}}}"#);
            assert_eq!(decided(b"settings", bytes.as_bytes()).unwrap_err(), expected);
        }
    }
}

#[test]
fn settings_pages_reserve_escaped_framing_within_the_allowance() {
    use settings_form::page::{Error, page_bounded};
    for text in ["\"".repeat(100), "\\".repeat(100), "\n".repeat(100), "\0".repeat(100), "λ".repeat(100)] {
        for maximum in [32, 64, 128, 512, 1024] {
            let body = page_bounded(text.as_bytes(), 0, maximum).expect("a page fits");
            let limits = skein_json::writer::Limits { depth: 1, length: maximum + 2 };
            let mut encoded = skein_json::writer::Encoder::measure(&limits);
            encoded.string(&body);
            encoded.measured().expect("escaped page and JSON quotes fit the allowance");
        }
    }
    for maximum in 0..8 {
        assert_eq!(page_bounded(b"a", 0, maximum), Err(Error::TooLarge));
    }
    assert_eq!(page_bounded(b"a", 0, 8).expect("exact allowance").as_ref(), b"a\nend\n");
    assert_eq!(page_bounded("λ".as_bytes(), 1, 128), Err(Error::Offset));
    assert_eq!(page_bounded(b"", 0, 5).expect("empty page").as_ref(), b"end\n");
    for maximum in [0, 1, 16] {
        let answer = crate::settings_answer_bounded(&inventory(1), 0, None, &[], None, maximum);
        assert!(answer.error());
        assert!(answer.text().len() <= usize::try_from(maximum).unwrap());
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test reconstructs rendered UTF-8 pages")]
#[expect(
    clippy::too_many_lines,
    reason = "the fixtures and reconstruction exercise one bounded inventory pagination path"
)]
fn inventory_people_forms_share_repository_pagination() {
    let repositories = [engine::RepositoryIdentity {
        host: b"forge.example:3847".as_slice().into(),
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
    }];
    let people = [
        engine::ProjectPerson {
            person: 1,
            role: 0,
            login: b"owner".as_slice().into(),
            forges: Box::new([engine::PersonForge {
                host: b"forge.example:3847".as_slice().into(),
                user: None,
                writer: false,
            }]),
            you: true,
        },
        engine::ProjectPerson {
            person: 4,
            role: 1,
            login: b"ada".as_slice().into(),
            forges: Box::new([engine::PersonForge {
                host: b"forge.example:3847".as_slice().into(),
                user: Some(3),
                writer: false,
            }]),
            you: false,
        },
        engine::ProjectPerson {
            person: 5,
            role: 2,
            login: Box::new([]),
            forges: Box::new([engine::PersonForge {
                host: b"forge.example:3847".as_slice().into(),
                user: Some(7),
                writer: false,
            }]),
            you: false,
        },
        engine::ProjectPerson {
            person: 6,
            role: 2,
            login: Box::new([]),
            forges: Box::new([engine::PersonForge {
                host: b"forge.example:3847".as_slice().into(),
                user: Some(2),
                writer: true,
            }]),
            you: false,
        },
        engine::ProjectPerson {
            person: 7,
            role: 2,
            login: b"grace".as_slice().into(),
            forges: Box::new([
                engine::PersonForge { host: b"forge.example:3847".as_slice().into(), user: Some(8), writer: false },
                engine::PersonForge { host: b"other.example".as_slice().into(), user: None, writer: false },
            ]),
            you: false,
        },
    ];
    let forge_people =
        [engine::ForgePerson { host: b"127.0.0.1:3000".as_slice().into(), login: b"ada".as_slice().into() }];
    let offer = [engine::OfferedModel {
        name: b"claude-opus-5-5".as_slice().into(),
        provider: b"anthropic".as_slice().into(),
        efforts: Box::new([b"low".as_slice().into(), b"high".as_slice().into()]),
        effort: Some(b"high".as_slice().into()),
    }];
    let answer = inventory(0);
    let (empty, _) =
        crate::settings_repository_answer(&answer, 0, None, &[], None, &[], &[], &[], &[], 8192).into_parts();
    let empty = core::str::from_utf8(&empty).expect("empty inventory UTF-8");
    assert!(!empty.contains("project people:") && !empty.contains("forge people:"));
    let (whole, error) = crate::settings_repository_answer(
        &answer,
        0,
        None,
        &[],
        None,
        &repositories,
        &people,
        &forge_people,
        &offer,
        8192,
    )
    .into_parts();
    assert!(!error);
    let expected = whole.strip_suffix(b"end\n").expect("terminal marker");
    assert!(expected.ends_with(b"project repositories:\n  forge forge.example:3847 repository org/repo\nproject people:\n  person 1 owner: role 0, requester, forge identity unknown: cannot count for approvals\n  person 4 ada: role 1, forge forge.example:3847 user 3\n  person 5: role 2, forge forge.example:3847 user 7\n  person 6: role 2, forge forge.example:3847 user 2, temper's own forge writer: never counts as a person\n  person 7 grace: role 2, forge forge.example:3847 user 8, forge other.example unknown: cannot count for approvals there\nforge people:\n  forge 127.0.0.1:3000 configured owned-forge account ada; project membership unverified by this row\nmodels you can offer:\n  claude-opus-5-5: anthropic; efforts low, high; default high\n"), "{}", core::str::from_utf8(expected).expect("inventory UTF-8"));
    let mut combined = skein_lib::Writer::new(expected.len());
    let mut from = 0;
    let mut pages = 0_u32;
    loop {
        let (body, error) = crate::settings_repository_answer(
            &answer,
            from,
            None,
            &[],
            None,
            &repositories,
            &people,
            &forge_people,
            &offer,
            256,
        )
        .into_parts();
        assert!(!error);
        assert!(body.len() <= 256);
        let text = core::str::from_utf8(&body).expect("page UTF-8");
        pages += 1;
        if let Some((text, next)) = text.rsplit_once("next: ") {
            let next = u32::try_from(crate::decimal(next.trim().as_bytes(), b"next").expect("offset")).expect("u32");
            let length = skein_lib::usize_of(next.checked_sub(from).expect("offset advances"));
            combined.put(text.as_bytes().get(..length).expect("original bytes")).expect("whole capacity");
            from = next;
        } else {
            combined.put(body.strip_suffix(b"end\n").expect("end marker")).expect("last page");
            break;
        }
    }
    assert!(pages > 1);
    assert_eq!(combined.finish().as_ref(), expected);
    let (unknown, error) = crate::settings_repository_answer(
        &SettingsAnswer::Unknown,
        0,
        None,
        &[],
        None,
        &repositories,
        &people,
        &forge_people,
        &offer,
        256,
    )
    .into_parts();
    assert!(error);
    assert_eq!(unknown.as_ref(), b"not read: no such setting at that revision. This says nothing about the settings: read the inventory; project-ways, project-questions, project-classes, project-words: project, no number; charter-words, template-goal: number, no project.\nend\n", "context belongs only to an inventory");
}

#[test]
fn a_settings_note_source_requires_one_positive_revision_and_no_inline_items() {
    let call = decided(b"propose_settings", br#"{"against":2,"reason":"whole","note":{"name":8,"revision":3}}"#)
        .expect("note source");
    assert_eq!(
        call.tool,
        engine::Tool::ProposeSettingsNote { against: 2, note: 8, revision: 3, reason: b"whole".as_slice().into() }
    );
    for (input, why) in [
        (
            br#"{"against":1,"reason":"whole","note":{"name":8,"revision":3},"items":[]}"#.as_slice(),
            Problem::Range(b"items"),
        ),
        (br#"{"against":1,"reason":"whole"}"#, Problem::Missing(b"items")),
        (br#"{"against":1,"reason":"whole","note":{"name":0,"revision":3}}"#, Problem::Range(b"name")),
        (br#"{"against":1,"reason":"whole","note":{"name":8,"revision":0}}"#, Problem::Range(b"revision")),
        (br#"{"against":1,"reason":"whole","note":{"name":8}}"#, Problem::Missing(b"revision")),
        (br#"{"against":1,"reason":"whole","note":{"name":8,"revision":3,"body":"x"}}"#, Problem::Range(b"note")),
    ] {
        assert_eq!(decided(b"propose_settings", input).unwrap_err(), why);
    }
}

#[test]
fn settings_document_refusal_words_name_member_and_bound() {
    use jig_domain_people::{Malformed, Refusal, SettingDetail, SettingFault, SettingMember, SettingRule};
    for (member, client_member, supplied, bound, expected) in [
        (
            SettingMember::Documents,
            c::SettingMember::Documents,
            3,
            2,
            "out of shape; item 0; charter 5: shape; documents: supplied 3; maximum tasks.charter_documents 2",
        ),
        (
            SettingMember::StoredPolicy,
            c::SettingMember::StoredPolicy,
            70112,
            65536,
            "out of shape; item 0; charter 5: shape; stored policy: supplied 70112; maximum root.journal.run_bytes 65536",
        ),
    ] {
        let refusal = Refusal::NotWellFormed {
            at: 0,
            why: Malformed::Settings {
                setting: s::Setting::Charter(5),
                fault: SettingFault::Shape,
                detail: Some(SettingDetail {
                    member,
                    document: 0,
                    segment: 0,
                    rule: SettingRule::Maximum,
                    supplied,
                    bound,
                }),
            },
        };
        let client = c::Refusal::Malformed {
            at: 0,
            why: c::Malformed::Settings(c::MalformedSettings {
                setting: c::Setting::Charter(c::SettingNumber { number: 5 }),
                fault: c::SettingFault::Shape,
                detail: Some(c::SettingDetail {
                    member: client_member,
                    document: 0,
                    segment: 0,
                    rule: c::SettingRule::Maximum,
                    supplied,
                    bound,
                }),
            }),
        };
        assert_eq!(
            crate::settings::refusal(&refusal),
            temper_client_tools::jig::render::settings_refusal(&client, temper_client_tools::limits::TEXT)
        );
        assert_eq!(crate::settings::refusal(&refusal).as_ref(), expected.as_bytes());
    }
}

#[test]
fn settings_proposals_decode_more_and_refuse_it_with_a_note() {
    for more in ["", ",\"more\":false", ",\"more\":true"] {
        let input = format!(
            r#"{{"against":1,"reason":"why","items":[{{"setting":{{"kind":"charter","number":1}},"edit":"reset"}}]{more}}}"#
        );
        let call = decided(b"propose_settings", input.as_bytes()).expect("part decodes as a call");
        let engine::Tool::ProposeSettings { more: decoded_more, .. } = call.tool else { panic!("proposal") };
        assert_eq!(decoded_more, more.contains("true"));
    }
    for more in ["false", "true"] {
        let input = format!(r#"{{"against":1,"reason":"why","note":{{"name":1,"revision":1}},"more":{more}}}"#);
        assert_eq!(
            decoded(b"propose_settings", input.as_bytes()).expect_err("note takes no more"),
            Problem::Range(b"more")
        );
    }
}

#[test]
fn settings_parts_answers_use_the_contracts_words_and_error_flag() {
    use engine::PartsAnswer as P;
    for (parts, expected, error) in [
        (
            P::Holding { count: 3, total: 12, against: 7, dropped: None },
            "propose_settings: holding 3 items for revision 7, 12 in all; nothing is proposed until a call without more.",
            false,
        ),
        (
            P::Holding { count: 1, total: 1, against: 8, dropped: Some((1, 7)) },
            "propose_settings: holding 1 item for revision 8, 1 in all; the 1 held for revision 7 were dropped. Nothing is proposed until a call without more.",
            false,
        ),
        (
            P::Holding { count: 3, total: 3, against: 8, dropped: Some((9, 7)) },
            "propose_settings: holding 3 items for revision 8, 3 in all; the 9 held for revision 7 were dropped. Nothing is proposed until a call without more.",
            false,
        ),
        (
            P::Held { setting: s::Setting::Charter(4) },
            "propose_settings: charter 4 is held already, from an earlier part; nothing was added.",
            true,
        ),
        (
            P::Held { setting: s::Setting::Application { kind: 2, scope: 7, number: 4 } },
            "propose_settings: landing-rule 4 --project 7 is held already, from an earlier part; nothing was added.",
            true,
        ),
        (
            P::PastBound { most: 129, would: 140 },
            "propose_settings: a proposal holds at most 129 items, and this call would make 140; nothing was added.",
            true,
        ),
        (
            P::Revision { held: 9, against: 7, named: 8 },
            "propose_settings: the 9 items held are for revision 7, and this call names 8; nothing was proposed, and they were dropped. Give the whole set again against 8.",
            true,
        ),
        (
            P::Revision { held: 1, against: 7, named: 8 },
            "propose_settings: the 1 item held are for revision 7, and this call names 8; nothing was proposed, and they were dropped. Give the whole set again against 8.",
            true,
        ),
        (
            P::Restarted,
            "propose_settings: temper restarted during this run, so parts it gave may be lost; nothing was proposed. Give the whole set again, in parts if it is large.",
            true,
        ),
        (
            P::NotOffered,
            "propose_settings: more is not offered to this charter; give the whole set in one call, or from a note.",
            true,
        ),
        (
            P::Approvals { why: b"project 1: refused.".as_slice().into() },
            "propose_settings: not proposed, as acceptance would refuse it: project 1: refused.",
            true,
        ),
    ] {
        let (body, failed) = crate::answer(&engine::CallAnswer::Parts(Box::new(parts))).into_parts();
        assert_eq!(body.as_ref(), expected.as_bytes());
        assert_eq!(failed, error);
    }
}
