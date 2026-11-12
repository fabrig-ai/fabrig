//! The reference's settings file (temper/docs/design/protocol/configuration/reference.md).
use settings_form::view::{
    CharterDocument, Setting, SettingEdit, SettingItem, SettingNumber, SettingValue, Symbol, TaskContractDocument,
    TemplateExecutor,
};
use skein_lib::{Duration, List, bytes};

#[path = "../../../../testing/reference_settings.rs"]
mod reference;

const COORDINATOR: u32 = 5;
const CODER: u32 = 2;
const STATUS: u32 = 9;
const GOAL: u32 = 4;
// Two overlays, one template and two landing rules.
const ITEMS: u32 = 5;

fn value(items: &List<SettingItem>, setting: Setting) -> &SettingValue {
    for item in items {
        if item.setting == setting {
            let SettingEdit::Set(set) = &item.edit else { panic!("reference sets its settings") };
            return &set.value;
        }
    }
    panic!("missing reference setting {setting:?}");
}

fn documents(documents: &List<CharterDocument>) {
    assert_eq!(documents.len(), 2);
    for (document, path) in
        documents.iter().zip([b"AGENTS.md".as_slice(), b"skein/docs/foundation/programming-model.md"])
    {
        assert_eq!(document.path.as_ref(), path);
        assert_eq!(document.heading.as_deref(), Some(b"1. In one page".as_slice()));
        assert_eq!(
            document.name.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(),
            [b"forge".as_slice(), b"127.0.0.1:3000", b"acme", b"widget", b"branch", b"main"]
        );
    }
}

#[test]
fn the_reference_settings_file_decodes() {
    let source = include_bytes!("../../../../docs/examples/reference/settings.json");
    let items = settings_form::file::decode(source, &temper_client_tools::limits::form())
        .expect("the reference decodes as a ctl settings file");
    assert_eq!(items.len(), ITEMS);
    let json: serde_json::Value = serde_json::from_slice(source).expect("reference JSON");
    for (number, model, shell) in
        [(COORDINATOR, b"claude-opus-5-5".as_slice(), false), (CODER, b"gpt-6.1-sol".as_slice(), true)]
    {
        let SettingValue::Overlay(overlay) = value(&items, Setting::Charter(SettingNumber { number })) else {
            panic!("reference overlay")
        };
        let raw = json["items"]
            .as_array()
            .expect("items")
            .iter()
            .find(|item| item["setting"]["kind"] == "charter" && item["setting"]["number"] == number)
            .expect("charter item");
        let mut fields: Vec<_> =
            raw["overlay"].as_object().expect("overlay object").keys().map(String::as_str).collect();
        fields.sort_unstable();
        let mut expected =
            vec!["model", "alternatives", "turns", "time", "reserve_turns", "reserve_time", "spend", "shell"];
        if number == COORDINATOR {
            expected.push("documents");
            documents(overlay.documents.as_ref().expect("goal documents"));
        } else {
            assert!(overlay.documents.is_none());
        }
        expected.sort_unstable();
        assert_eq!(fields, expected, "only reference fields, no text or directories");
        assert!(overlay.instructions.bytes().is_empty());
        assert!(overlay.parts.is_empty());
        assert!(overlay.charters.is_empty());
        let configured = overlay.model.as_ref().expect("reference model");
        assert_eq!(configured.name.bytes(), model);
        assert_eq!((configured.dialect, configured.endpoint), (3, 1));
        assert!(configured.program.is_none());
        assert_eq!((configured.window, configured.output), (1_048_576, 48_000));
        assert_eq!(
            (configured.input_price, configured.cached_price, configured.output_price, configured.price_unit),
            (1, 1, 1, 1000)
        );
        assert_eq!(configured.cache_write_price, None);
        assert!(configured.efforts.is_empty());
        assert_eq!(configured.effort, None);
        assert!(overlay.alternatives.as_ref().expect("explicit alternatives").is_empty());
        assert_eq!(overlay.turns, Some(800));
        assert_eq!(overlay.time, Some(Duration::from_secs(28_800)));
        assert_eq!(overlay.reserve_turns, Some(1));
        assert_eq!(overlay.reserve_time, Some(Duration::from_secs(60)));
        assert_eq!(overlay.spend, Some(4228));
        assert_eq!(overlay.shell, Some(shell));
    }
    let SettingValue::Template(goal) = value(&items, Setting::Template(SettingNumber { number: GOAL })) else {
        panic!("goal template")
    };
    assert_eq!(goal.number, GOAL);
    assert_eq!(goal.executor, TemplateExecutor::Charter(COORDINATOR));
    assert_eq!(goal.name.bytes(), b"goal for a draft");
    assert_eq!(goal.tracked, Some(1));
    assert_eq!(goal.label, 2);
    assert_eq!(
        goal.flags.ended_by,
        settings_form::view::EndedBy::Closer(settings_form::view::Closer { connector: 1, code: 6 })
    );
    let llm = goal.llm.as_ref().expect("goal model choice");
    assert_eq!(llm.main.model.bytes(), b"claude-opus-5-5");
    assert_eq!(llm.main.effort.as_ref().expect("coordinator effort").bytes(), b"xhigh");
    let later = llm.later.as_ref().expect("goal later choice");
    assert_eq!(later.main.model.bytes(), b"claude-opus-5-5");
    assert_eq!(later.main.effort.as_ref().expect("later coordinator effort").bytes(), b"high");
    assert_eq!(later.subagents, llm.subagents);
}

#[test]
fn the_reference_settings_fit_the_reference_profile() {
    let items = settings_form::file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("reference settings");
    let mut init: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/temper.json"))
            .expect("init configuration");
    let reference: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/limits.json"))
            .expect("reference limits");
    init["engine"]["limits"]["overrides"]
        .as_object_mut()
        .expect("init overrides")
        .extend(reference["engine"]["limits"]["overrides"].as_object().expect("reference overrides").clone());
    let engine = reference_engine(&init["engine"]["limits"]);
    let limits = &engine.limits.core.tasks;
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), limits.result_bytes);
    let init_agent: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/agent.json"))
            .expect("init agent configuration");
    let max_turns = init_agent["profile"]["policy"]["max_turns"].as_u64().expect("init turn limit");
    assert_eq!(items.iter().filter(|item| matches!(item.setting, Setting::Charter(_))).count(), 2);
    for number in [CODER, COORDINATOR] {
        let charter = reference::charter(&stock, number);
        assert!(
            charter.documents.len() <= usize::try_from(limits.charter_documents).expect("document limit"),
            "tasks.charter_documents"
        );
        assert!(u64::from(charter.policy.turns) <= max_turns, "charter {number}");
    }
    assert!(stock.charters.len() <= usize::try_from(limits.charters).expect("charter limit"));
    for charter in &stock.charters {
        assert!(u64::from(charter.policy.turns) <= max_turns, "stock charter {}", charter.number);
    }
    let SettingValue::Template(goal) = value(&items, Setting::Template(SettingNumber { number: GOAL })) else {
        panic!("goal template")
    };
    let TaskContractDocument::Report { words } = goal.contract else { panic!("goal report") };
    assert!(words <= limits.result_bytes);
    assert_eq!(goal.resources.len(), 1);
    assert_eq!(goal.resources.iter().next(), Some(&Symbol { connector: 1, symbol: 1 }));
}

#[test]
fn the_reference_charters_fit_a_stock_worker_and_the_run_bound() {
    use jig_domain_fleet::{CharterNeeds, HostKind, ModelNeed, Needs, Version};

    let room: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/room.json"))
            .expect("reference room");
    let run_bytes =
        room["application"]["limits"]["overrides"]["root.journal.run_bytes"].as_u64().expect("reference run bound");
    let bounds = reference_worker_bounds(&room["agent"]);
    let init: serde_json::Value = serde_json::from_slice(include_bytes!("../../../temper-init/template/temper.json"))
        .expect("init configuration");
    let engine = reference_engine(&init["engine"]["limits"]);
    let config = jig_protocol_config::core(&engine, [0; 16], 1);
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    for number in [CODER, COORDINATOR] {
        let charter = reference::charter(&stock, number);
        let original = stock.charters.iter().find(|charter| charter.number == number).expect("stock charter");
        assert_eq!(charter.policy.instructions, original.policy.instructions, "stock words reach the run");
        assert_eq!(charter.parts.len(), original.parts.len(), "stock parts reach the run");
        assert_eq!(charter.parts, original.parts, "stock parts reach the run");
        if number == COORDINATOR {
            let common = include_str!("../../../../docs/charters/stock/common.md").trim();
            let goal = include_str!("../../../../docs/charters/stock/goal.md").trim();
            assert_eq!(charter.policy.instructions.as_ref(), format!("{common}\n\n{goal}").as_bytes());
            assert_eq!(charter.parts.len(), 4);
            assert_eq!(charter.policy.model.name.as_ref(), b"claude-opus-5-5");
        }
        let policy = &charter.policy;
        let models = std::iter::once(&policy.model)
            .chain(policy.alternatives.iter())
            .map(|model| ModelNeed {
                endpoint: model.endpoint,
                name: model.name.clone(),
                window: model.window,
                output: model.output,
            })
            .collect();
        let needs = Needs {
            charters: Box::new([CharterNeeds {
                version: Version { charter: number, revision: 1 },
                kinds: charter.hosts,
                models,
                turns: policy.turns,
                time: policy.time,
                waiting: policy.waiting,
                change_reserve: if policy.modify { Some(policy.reserve_time) } else { None },
            }]),
            spend: config.defaults.rules.maximum_run_spend,
            message_bytes: engine.limits.core.tasks.message_bytes,
            start_messages: engine.limits.core.tasks.run_inbox,
            unacknowledged: config.hub_turns,
        };
        assert_eq!(
            jig_domain_fleet::shortfall(&needs, &bounds, HostKind::Worker),
            None,
            "charter {number}: stock worker bounds {bounds:?}"
        );
        let stored = jig_domain_core::policy_owned_bytes(policy)
            .and_then(|bytes| bytes.checked_add(jig_domain_core::parts_owned_bytes(&charter.parts)?))
            .unwrap_or_else(|| panic!("charter {number}: stored policy byte count overflow"));
        eprintln!("reference charter {number}: stored policy and parts {stored}/{run_bytes} bytes");
        assert!(
            stored <= run_bytes,
            "charter {number}: stored policy {stored} exceeds root.journal.run_bytes {run_bytes}"
        );
    }
}

fn reference_worker_bounds(room: &serde_json::Value) -> jig_domain_link::Bounds {
    let mut agent: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/agent.json"))
            .expect("init agent configuration");
    agent["profile"]["declared"]["tool_payload"] = room["tool_payload"].clone();
    // Only addresses and template placeholders change; no endpoint is contacted.
    for endpoint in agent["endpoints"].as_array_mut().expect("init endpoints") {
        endpoint["limits"]["output"] = room["output"].clone();
        endpoint["address"] = "127.0.0.1:1".into();
    }
    let scratch = skein_scratch::Scratch::new("reference-worker");
    agent["programs"] = serde_json::json!({"claude_code": {
        "path": scratch.path().join("claude"), "directory": scratch.path().join("programs")
    }});
    let path = scratch.path().join("agent.json");
    std::fs::write(&path, serde_json::to_vec(&agent).expect("agent JSON")).expect("scratch agent");
    temper_worker_shell::agent::read(&path).expect("stock worker with reference room").bounds
}

#[test]
fn the_reference_sets_main_and_goal_branch_landing_rules() {
    use temper_engine_domain_forge::{GateLast, settings};
    let items = settings_form::file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("reference settings");
    for (number, last, checks) in
        [(1, GateLast::Exact(b"main".as_slice().into()), true), (2, GateLast::Open(b"temper".as_slice().into()), false)]
    {
        let SettingValue::Application(bytes) = value(&items, Setting::Application { kind: 1, scope: 0, number }) else {
            panic!("reference landing rule")
        };
        let value = temper_engine_smith::kinds::decode(
            1,
            bytes,
            &temper_engine_smith::kinds::Bounds { segments: 8, segment_bytes: 64, gates: 4, approvals: 4 },
        )
        .expect("reference rule codec");
        let settings::Value::Rule(rule) = value.value else { panic!("rule value") };
        assert_eq!(
            rule.pattern.segments.as_ref(),
            [b"forge".as_slice(), b"127.0.0.1:3000", b"acme", b"widget", b"branch"].map(Box::from)
        );
        assert_eq!(rule.pattern.last, last);
        assert!(rule.enforce);
        assert_eq!((rule.ci, rule.up_to_date), (checks, checks));
        assert!(rule.gates.is_empty());
        assert!(rule.approvals.is_empty());
    }
}

fn reference_engine(limits: &serde_json::Value) -> jig_protocol_config::Engine {
    let owned: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("owned JSON");
    let mut engine = owned["engine"].clone();
    engine["limits"] = limits.clone();
    let bytes = serde_json::to_vec(&engine).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16384, depth: 32 },
    )
    .expect("engine document");
    let mut engine = jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
    )
    .expect("reference engine");
    let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let mut sol =
        stock.models.into_vec().into_iter().find(|entry| entry.model.name.as_ref() == b"gpt-6.1-sol").expect("Sol");
    sol.number = 1;
    sol.shown = None;
    sol.model.input_price = 1;
    sol.model.cached_price = 1;
    sol.model.output_price = 1;
    sol.model.price_unit = 1000;
    sol.model.efforts = Box::default();
    sol.model.effort = None;
    let mut designer = sol.clone();
    designer.number = 2;
    designer.model.name = b"claude-opus-5-5".as_slice().into();
    let defaults = temper_service::stock::with_models(Box::new([sol, designer]), engine.limits.core.tasks.result_bytes);
    jig_protocol_config::pass(&mut engine, defaults).expect("reference defaults");
    engine
}

fn core_item(item: &jig_client::contract::SettingItem) -> jig_domain_core::settings::Item {
    let preset = temper_client_tools::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("ctl limits");
    let wire = jig_client_protocol_link::translate::settings::wire_setting_item(item, link.wire())
        .expect("reference item fits ctl");
    let size = wire.measure();
    let mut bytes = skein_lib::Writer::new(usize::try_from(size).expect("item size") + 4);
    bytes.put(&size.to_be_bytes()).expect("length prefix");
    wire.encode(&mut bytes).expect("reference item encoding");
    jig_protocol_engine_clients::translate::settings::items(bytes.finish(), &[], link.wire())
        .expect("reference item translation")
        .into_iter()
        .next()
        .expect("one item")
}

fn reference_rules(current: &jig_domain_core::settings::Current) -> jig_client::contract::Rules {
    let preset = temper_client_tools::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("ctl limits");
    let value = jig_domain_core::settings::Value::Rules(Box::new(current.rules().clone()));
    let wire =
        jig_protocol_engine_clients::translate::settings::value(&value, &[], link.wire()).expect("stock rules fit ctl");
    let jig_client_protocol_wire::v20::SettingValue::Rules(wire) = wire else { panic!("rules") };
    jig_client_protocol_link::translate::settings::restore_rules(&wire, &link).expect("stock rules")
}

fn reference_charters() -> Vec<jig_domain_core::settings::Item> {
    use jig_domain_core::settings::{Edit, Value};
    let mut charters = reference::items().into_vec();
    // Ctl resolves the reference's forge document names to connector 1 before sending.
    for item in &mut charters {
        if let Edit::Set(Value::Overlay(overlay)) = &mut item.edit
            && let Some(documents) = &mut overlay.documents
        {
            for document in documents {
                document.connector = 1;
            }
        }
    }
    charters
}

fn reference_delegations() -> [Vec<jig_client::contract::AuthorityExecutor>; 1] {
    use jig_client::contract::AuthorityExecutor as Executor;
    let fifteen = vec![
        Executor::Charter(1),
        Executor::Charter(2),
        Executor::Charter(4),
        Executor::Charter(COORDINATOR),
        Executor::Charter(STATUS),
        Executor::Charter(11),
        Executor::Procedure(2),
        Executor::Procedure(3),
        Executor::Procedure(4),
        Executor::Procedure(6),
        Executor::Job(3),
        Executor::Job(4),
        Executor::Role(0),
        Executor::Role(1),
        Executor::Role(2),
    ];
    [fifteen]
}

fn check_reference_delegation(mut engine: jig_protocol_config::Engine) {
    use jig_client::contract::{Setting, SettingEdit, SettingItem, SettingSet, SettingValue};
    use jig_domain_core::settings;
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    jig_protocol_config::pass(&mut engine, stock).expect("stock settings");
    let config = jig_protocol_config::core(&engine, [0; 16], 1);
    let limits = &engine.limits.core;
    // The stock fits twenty-one executor places; the reference replaces its delegation.
    assert_eq!(limits.tasks.executor_kinds, 21);
    assert_eq!(limits.authority.executors, 21);
    let current = settings::Current::new(config.defaults, limits, &[]);
    let charters = reference_charters();
    assert_eq!(charters.len(), 2);
    assert_eq!(settings::check_change(&current, &charters, &config.settings, &config.connectors, limits), Ok(()));
    let mut rules = reference_rules(&current);
    for kinds in reference_delegations() {
        let count = u32::try_from(kinds.len()).expect("reference kind count");
        assert_eq!(count, 15);
        rules.ceiling.delegation.kinds = List::with_capacity(count);
        for kind in kinds {
            rules.ceiling.delegation.kinds.push(kind).expect("delegation count");
        }
        let mut items = charters.clone();
        let rules_item = core_item(&SettingItem {
            setting: Setting::Rules,
            edit: SettingEdit::Set(SettingSet { value: SettingValue::Rules(Box::new(rules.clone())) }),
        });
        let settings::Edit::Set(settings::Value::Rules(core_rules)) = &rules_item.edit else { panic!("rules") };
        let mut policy = current.policy(1).expect("stock policy").clone();
        policy.policy.ceiling.delegation.kinds.clone_from(&core_rules.ceiling.delegation.kinds);
        for role in &mut policy.policy.roles {
            if role.number == 0 {
                role.authority.delegation.kinds.clone_from(&core_rules.ceiling.delegation.kinds);
            } else {
                role.authority.delegation.kinds = role
                    .authority
                    .delegation
                    .kinds
                    .iter()
                    .copied()
                    .filter(|kind| {
                        *kind != jig_domain_authority::Executor::Charter(12)
                            && *kind != jig_domain_authority::Executor::Charter(13)
                            && *kind != jig_domain_authority::Executor::Charter(14)
                            && *kind != jig_domain_authority::Executor::Charter(15)
                            && *kind != jig_domain_authority::Executor::Charter(16)
                    })
                    .collect();
            }
        }
        items.push(rules_item);
        items.push(settings::Item {
            setting: settings::Setting::Policy(1),
            edit: settings::Edit::Set(settings::Value::Policy(Box::new(policy))),
        });
        let change = settings::Change {
            against: current.newest(),
            items: items.into_boxed_slice(),
            running: settings::Running::Stays,
        };
        assert_eq!(
            settings::check_change(&current, &change.items, &config.settings, &config.connectors, limits),
            Ok(()),
            "{count} reference executors against NEWEST"
        );
        assert!(count <= limits.tasks.executor_kinds);
    }
}

#[test]
fn init_limits_hold_reference_delegation() {
    for bytes in [
        include_bytes!("../../../temper-init/template/temper.json").as_slice(),
        include_bytes!("../../../../docs/examples/owned.json").as_slice(),
    ] {
        let document: serde_json::Value = serde_json::from_slice(bytes).expect("shipped configuration JSON");
        check_reference_delegation(reference_engine(&document["engine"]["limits"]));
    }
}

#[test]
fn the_reference_project_file_shows_its_goals_and_lands_main_alone() {
    use temper_engine_domain_forge::{GateLast, settings};

    let items = settings_form::file::decode(
        include_bytes!("../../../../docs/examples/reference/project.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("reference project settings");
    assert_eq!(items.len(), 2);
    assert_eq!(
        items.get(0).expect("project ways first").setting,
        Setting::Application { kind: 7, scope: 1, number: 1 }
    );
    let SettingValue::Application(bytes) = value(&items, Setting::Application { kind: 7, scope: 1, number: 1 }) else {
        panic!("project ways")
    };
    let decoded = temper_engine_smith::kinds::decode(
        7,
        bytes,
        &temper_engine_smith::kinds::Bounds { segments: 16, segment_bytes: 4096, gates: 64, approvals: 64 },
    )
    .expect("reference ways");
    let temper_engine_domain_forge::settings::Value::Ways(ways) = decoded.value else { panic!("project ways record") };
    assert_eq!((ways.drafts, ways.goals, ways.changes, ways.way), (true, true, true, 4));

    let setting = Setting::Application { kind: 2, scope: 1, number: 1 };
    assert_eq!(items.get(1).expect("project landing rule second").setting, setting);
    let SettingValue::Application(bytes) = value(&items, setting) else { panic!("project landing rule") };
    let decoded = temper_engine_smith::kinds::decode(
        2,
        bytes,
        &temper_engine_smith::kinds::Bounds { segments: 8, segment_bytes: 64, gates: 4, approvals: 4 },
    )
    .expect("reference project rule codec");
    let bounds = settings::Bounds {
        landing_rules: 2,
        gates: 4,
        roles: 4,
        charters: 12,
        segments: 8,
        segment_bytes: 64,
        templates: 4,
        follows: 2,
        dependencies: 4,
        follow_paths: 8,
        name_bytes: 64,
    };
    for (kind, scope, expected) in [(2, 1, Ok(())), (1, 0, Err(settings::Refusal::Pattern))] {
        let item = settings::Item { kind, scope, number: 1, value: Some(decoded.value.clone()) };
        assert_eq!(settings::judge(&item, &bounds), expected, "reference rule as kind {kind}");
    }
    let settings::Value::Rule(rule) = decoded.value else { panic!("project landing rule value") };
    assert_eq!(rule.pattern.segments.as_ref(), [Box::from(b"branch".as_slice())]);
    assert_eq!(rule.pattern.last, GateLast::Exact(b"main".as_slice().into()));
    assert!(rule.enforce);
    assert!(rule.ci);
    assert!(rule.up_to_date);
    assert!(rule.gates.is_empty());
    assert!(rule.approvals.is_empty());
}

// temper/docs/design/client/README.md, 2: each client receives the stock delegation whole.
#[test]
fn the_stock_settings_document_decodes_in_every_client() {
    use jig_domain_core::{reads, settings};
    let engine = reference_engine(&serde_json::json!({"profile":"standard","overrides":{
        "authority.executors":21,"tasks.executor_kinds":21,"tasks.charters":16
    }}));
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    assert_eq!(stock.rules.ceiling.delegation.kinds.len(), 20);
    let source = reads::SettingDocument {
        overlay: None,
        setting: settings::Setting::Rules,
        revision: 1,
        value: Some(settings::Value::Rules(Box::new(stock.rules))),
        application: None,
        proposed: None,
    };
    let presets = [
        ("terminal", temper_client_shell_terminal::limits::native().client.client),
        ("browser", temper_client_shell_web::limits::client().client),
        ("ctl", temper_client_tools::limits::client().client),
    ];
    for (name, preset) in presets {
        let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
        let wire = jig_protocol_engine_clients::translate::settings::setting_document(&source, &[], link.wire())
            .unwrap_or_else(|why| panic!("{name} receives stock settings: {why:?}"));
        let mut bytes = skein_lib::Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut bytes).unwrap();
        let bytes = bytes.finish();
        let mut reader = skein_lib::Reader::new(&bytes);
        let decoded = jig_client_protocol_wire::SettingDocument::decode(link.wire(), &mut reader).unwrap();
        let restored = jig_client_protocol_link::translate::settings::restore_setting_document(&decoded, &link)
            .unwrap_or_else(|why| panic!("{name} restores stock settings: {why:?}"));
        let Some(jig_client::contract::SettingValue::Rules(rules)) = restored.value else { panic!("stock rules") };
        assert_eq!(rules.ceiling.delegation.kinds.len(), 20, "{name}");
        for form in [temper_client_tools::limits::form(), settings_form::Limits::standard()] {
            let restored = settings_form::wire::restore_setting_document(&decoded, &form).expect("settings form");
            let Some(SettingValue::Rules(rules)) = restored.value else { panic!("form rules") };
            assert_eq!(rules.ceiling.delegation.kinds.len(), 20);
        }
    }
}

#[test]
fn the_reference_has_no_foreman() {
    let source = include_bytes!("../../../../docs/examples/reference/settings.json");
    assert!(bytes::find(source, b"foreman").is_none());
    let items = settings_form::file::decode(source, &temper_client_tools::limits::form()).expect("reference settings");
    assert!(!items.iter().any(|item| item.setting == Setting::Charter(SettingNumber { number: 6 })));
}
