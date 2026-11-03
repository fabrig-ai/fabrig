//! The reference's settings file (temper/docs/design/protocol/configuration.md, 13.9).
use settings_form::view::{
    CharterDocument, ResultsWake, Setting, SettingEdit, SettingItem, SettingNumber, SettingValue, Symbol,
    TaskContractDocument, TemplateExecutor, WakePolicy, WakeRule,
};
use skein_lib::{Duration, List, bytes};

const COORDINATOR: u32 = 5;
const DESIGNER: u32 = 7;
const CODER: u32 = 2;
const FOREMAN: u32 = 6;
const TRIAGE: u32 = 8;
const GOAL: u32 = 4;
// The reference supplies five charters, one template and two landing rules.
const ITEMS: u32 = 8;
// Checked against the reference's profile.policy.max_turns below.
const TURNS: u32 = 64;
const INIT_TURNS: u32 = 16;

fn value(items: &List<SettingItem>, setting: Setting) -> &SettingValue {
    for item in items {
        if item.setting == setting {
            let SettingEdit::Set(set) = &item.edit else { panic!("reference sets its settings") };
            return &set.value;
        }
    }
    panic!("missing reference setting {setting:?}");
}

fn documents(documents: &List<CharterDocument>, first_path: &[u8], first_heading: Option<&[u8]>, second_path: &[u8]) {
    assert_eq!(documents.len(), 2);
    for (document, (path, heading)) in
        documents.iter().zip([(first_path, first_heading), (second_path, Some(b"1. In one page".as_slice()))])
    {
        assert_eq!(document.path.as_ref(), path);
        assert_eq!(document.heading.as_deref(), heading);
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one reference fixture checks designer choices and catch-up instructions together"
)]
fn the_reference_settings_file_decodes() {
    let items = settings_form::file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("the reference decodes as a ctl settings file");
    assert_eq!(items.len(), ITEMS);
    for (number, model) in [
        (COORDINATOR, b"claude-opus-5-5".as_slice()),
        (DESIGNER, b"claude-opus-5-5".as_slice()),
        (CODER, b"gpt-6.1-sol".as_slice()),
        (FOREMAN, b"gpt-6.1-sol".as_slice()),
        (TRIAGE, b"gpt-6.1-sol".as_slice()),
    ] {
        let SettingValue::Charter(charter) = value(&items, Setting::Charter(SettingNumber { number })) else {
            panic!("reference charter")
        };
        assert_eq!(charter.number, number);
        assert_eq!(charter.model.name.bytes(), model);
        assert!(charter.alternatives.is_empty());
        assert_eq!(charter.hosts, settings_form::view::SettingHosts::Workers);
        assert_eq!(charter.turns, if matches!(number, FOREMAN | TRIAGE) { INIT_TURNS } else { TURNS });
        assert_eq!(charter.time, Duration::from_secs(1800));
        if number != CODER && number != FOREMAN {
            assert!(!charter.resume());
            assert_eq!(charter.waiting, Duration::ZERO);
        }
    }
    let SettingValue::Charter(coordinator) = value(&items, Setting::Charter(SettingNumber { number: COORDINATOR }))
    else {
        panic!("coordinator charter")
    };
    documents(
        &coordinator.documents,
        b"AGENTS.md",
        Some(b"1. In one page"),
        b"skein/docs/foundation/programming-model.md",
    );
    assert!(bytes::find(coordinator.instructions.bytes(), b"\"producer\": 7").is_some());
    assert!(bytes::find(coordinator.instructions.bytes(), b"close-out").is_some());
    assert!(
        std::str::from_utf8(coordinator.instructions.bytes())
            .expect("coordinator instructions are UTF-8")
            .contains(r#""branch": "mine""#)
    );
    let instructions =
        std::str::from_utf8(coordinator.instructions.bytes()).expect("coordinator instructions are UTF-8");
    assert!(instructions.contains(r#""branch": "mine""#));
    assert!(instructions.contains(r#""questions": "foreman""#));
    assert!(instructions.contains(r#""questions": "up""#));
    assert!(instructions.contains("delegate its triage to charter 8"));
    assert!(instructions.contains("decide"));
    assert!(instructions.contains(r#""repairs""#));
    assert!(instructions.contains("a red of your goal branch's tier"));
    assert!(instructions.contains("a live repair owning its failing tests"));
    assert!(instructions.contains("then the draft it was built from, by path,"));
    assert!(instructions.contains("with `note`"));
    assert!(coordinator.instructions.bytes().ends_with(
        b"If you finish without a landing, say why in your report; your goal branch is kept for a person to read."
    ));
    assert!(!coordinator.resume());
    assert_eq!(coordinator.waiting, Duration::ZERO);
    assert!(core::str::from_utf8(coordinator.instructions.bytes()).expect("coordinator instructions").contains(
        "When main moves under your goal, you hear a landing into main that touches your goal's \
         open work as news, at most once in 30 minutes. Catch up with one `change`: `\"landing\": \
         \"goal\"`, `\"merge\"` the landing's merge commit, its words naming the landing and why your \
         goal needs it now. Keep one catch-up live at a time: a later landing's commit carries the \
         earlier ones, so on news while one is live, wait for its result. On your first run, once \
         your first batch is made, subscribe with `subscribe_forge` to the landings on main in your \
         repository."
    ));
    assert_eq!(
        coordinator.wake,
        WakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Immediate,
            news: WakeRule::Paced { span: Duration::from_secs(1800) },
            results: ResultsWake::Each,
            questions: true,
            answers: true,
            timers: true,
        }
    );
    let SettingValue::Charter(designer) = value(&items, Setting::Charter(SettingNumber { number: DESIGNER })) else {
        panic!("designer charter")
    };
    assert_eq!(designer.model, coordinator.model);
    assert!(designer.inspect());
    assert!(designer.modify());
    assert!(!designer.shell());
    assert!(designer.agents());
    assert_eq!(designer.reserve_turns, coordinator.reserve_turns);
    assert_eq!(designer.reserve_time, coordinator.reserve_time);
    assert_eq!(designer.call_timeout, coordinator.call_timeout);
    assert_eq!(designer.spend, coordinator.spend);
    assert_eq!(designer.wake, WakePolicy::DEFAULT);
    documents(&designer.documents, b"DESIGN.md", None, b"AGENTS.md");
    assert!(bytes::find(designer.instructions.bytes(), b"Not yet").is_some());
    let SettingValue::Charter(coder) = value(&items, Setting::Charter(SettingNumber { number: CODER })) else {
        panic!("coding charter")
    };
    assert_eq!(coder.wake, WakePolicy::DEFAULT);
    assert!(coder.documents.is_empty());
    assert!(coder.shell());
    let stock = std::str::from_utf8(include_bytes!("../../../temper-service/src/stock/coding.txt"))
        .expect("stock coding instructions are UTF-8");
    let checks = "Before each delivery, run with the shell the lints and the focused tests of each package \
                  you changed: `cargo clippy -p PKG --all-targets -- -D warnings`, then `cargo nextest run -p \
                  PKG`, naming each package; fix what they find in your own files, and list a failure outside \
                  them under Left.";
    let instructions = stock.replace("You can run nothing.", checks);
    assert!(coder.instructions.bytes().starts_with(instructions.as_bytes()));
    assert!(
        std::str::from_utf8(coder.instructions.bytes())
            .expect("coding instructions are UTF-8")
            .contains("in the question form")
    );
    let SettingValue::Charter(foreman) = value(&items, Setting::Charter(SettingNumber { number: FOREMAN })) else {
        panic!("foreman charter")
    };
    assert!(foreman.inspect());
    assert!(!foreman.modify());
    assert!(!foreman.shell());
    assert!(!foreman.agents());
    assert!(foreman.instructions.bytes().starts_with(include_bytes!("../../../temper-service/src/stock/foreman.txt")));
    let SettingValue::Charter(triage) = value(&items, Setting::Charter(SettingNumber { number: TRIAGE })) else {
        panic!("triage charter")
    };
    assert!(triage.inspect());
    assert!(!triage.modify());
    assert!(!triage.shell());
    assert!(!triage.agents());
    assert_eq!(triage.model, foreman.model);
    assert_eq!(triage.reserve_turns, foreman.reserve_turns);
    assert_eq!(triage.reserve_time, foreman.reserve_time);
    assert_eq!(triage.call_timeout, foreman.call_timeout);
    assert_eq!(triage.spend, foreman.spend);
    assert_eq!(triage.wake, WakePolicy::DEFAULT);
    assert!(triage.documents.is_empty());
    assert!(bytes::find(triage.instructions.bytes(), b"You never edit, push or run anything.").is_some());
    assert!(
        core::str::from_utf8(coder.instructions.bytes())
            .expect("coding instructions")
            .contains("Push as you go: deliver a first head as soon as the change builds, then again as it grows.")
    );
    let SettingValue::Template(goal) = value(&items, Setting::Template(SettingNumber { number: GOAL })) else {
        panic!("goal template")
    };
    assert_eq!(goal.number, GOAL);
    assert_eq!(goal.executor, TemplateExecutor::Charter(COORDINATOR));
    assert_eq!(goal.name.bytes(), b"goal for a draft");
    assert_eq!(goal.tracked, Some(1));
    assert_eq!(goal.label, 2);
    let llm = goal.llm.as_ref().expect("goal model choice");
    assert_eq!(llm.main.model.bytes(), b"claude-opus-5-5");
    assert_eq!(llm.main.effort.as_ref().expect("coordinator effort").bytes(), b"xhigh");
}

#[test]
fn the_reference_settings_fit_the_reference_profile() {
    let items = settings_form::file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("reference settings");
    let limits: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/limits.json"))
            .expect("reference limits");
    let agent: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/reference/agent.json"))
            .expect("reference agent profile");
    let init_agent: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/agent.json"))
            .expect("init agent configuration");
    let max_turns = agent["profile"]["policy"]["max_turns"].as_u64().expect("reference turn limit");
    assert_eq!(u64::from(TURNS), max_turns);
    assert_eq!(Some(u64::from(INIT_TURNS)), init_agent["profile"]["policy"]["max_turns"].as_u64());
    let overrides = &limits["engine"]["limits"]["overrides"];
    let mut charters = 4_u64;
    for item in &items {
        if let SettingEdit::Set(set) = &item.edit
            && let SettingValue::Charter(charter) = &set.value
        {
            charters += 1;
            assert!(charter.documents.len() <= 2, "small's tasks.charter_documents");
            assert!(u64::from(charter.turns) <= max_turns, "charter {}", charter.number);
        }
    }
    assert!(charters <= overrides["tasks.charters"].as_u64().expect("charter limit"));
    let SettingValue::Template(goal) = value(&items, Setting::Template(SettingNumber { number: GOAL })) else {
        panic!("goal template")
    };
    let TaskContractDocument::Report { words } = goal.contract else { panic!("goal report") };
    let result_bytes = overrides["tasks.result_bytes"].as_u64().expect("result limit");
    assert!(u64::from(words) <= result_bytes);
    assert!(result_bytes <= 4096, "small's tasks.message_bytes");
    assert_eq!(goal.resources.len(), 1);
    assert_eq!(goal.resources.iter().next(), Some(&Symbol { connector: 1, symbol: 1 }));
    let SettingValue::Charter(coordinator) = value(&items, Setting::Charter(SettingNumber { number: COORDINATOR }))
    else {
        panic!("coordinator charter")
    };
    assert!(bytes::find(coordinator.instructions.bytes(), b"A `change` call makes at most").is_some());
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
    engine["llm"]["models"][0]["prices"]["unit"] = 1000.into();
    let mut designer = engine["llm"]["models"][0].clone();
    designer["name"] = "claude-opus-5-5".into();
    engine["llm"]["models"].as_array_mut().expect("models").push(designer);
    engine["llm"]["providers"][0]["models"] = serde_json::json!(["gpt-6-luna", "claude-opus-5-5"]);
    let bytes = serde_json::to_vec(&engine).expect("engine JSON");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16384, depth: 32 },
    )
    .expect("engine document");
    jig_protocol_config::engine(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
    )
    .expect("reference engine")
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
    let jig_client_protocol_wire::v19::SettingValue::Rules(wire) = wire else { panic!("rules") };
    jig_client_protocol_link::translate::settings::restore_rules(&wire, &link).expect("stock rules")
}

fn reference_charters() -> Vec<jig_domain_core::settings::Item> {
    use jig_client::contract::{Setting, SettingNumber};
    use jig_domain_core::settings::{Edit, Value};
    let reference = temper_client_tools::jig::settings_file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::tools(),
    )
    .expect("reference settings");
    let mut charters: Vec<_> = reference
        .iter()
        .filter(|item| matches!(item.setting, Setting::Charter(SettingNumber { number: COORDINATOR | DESIGNER })))
        .map(core_item)
        .collect();
    // Ctl resolves the reference's forge document names to connector 1 before sending.
    for item in &mut charters {
        if let Edit::Set(Value::Charter(charter)) = &mut item.edit {
            for document in &mut charter.documents {
                document.connector = 1;
            }
        }
    }
    charters
}

fn reference_delegations() -> [Vec<jig_client::contract::AuthorityExecutor>; 4] {
    use jig_client::contract::AuthorityExecutor as Executor;
    let nine = vec![
        Executor::Charter(1),
        Executor::Charter(2),
        Executor::Charter(4),
        Executor::Charter(COORDINATOR),
        Executor::Charter(FOREMAN),
        Executor::Charter(DESIGNER),
        Executor::Procedure(2),
        Executor::Procedure(3),
        Executor::Procedure(4),
    ];
    let eight = nine.iter().copied().filter(|kind| *kind != Executor::Charter(DESIGNER)).collect();
    let mut eleven = nine.clone();
    eleven.extend([Executor::Job(3), Executor::Role(0)]);
    [nine.clone(), eight, nine, eleven]
}

fn check_reference_delegation(mut engine: jig_protocol_config::Engine) {
    use jig_client::contract::{Setting, SettingEdit, SettingItem, SettingSet, SettingValue};
    use jig_domain_core::settings;
    let stock = temper_service::stock::defaults(&engine.llm);
    jig_protocol_config::pass(&mut engine, stock).expect("stock settings");
    let config = jig_protocol_config::core(&engine, [0; 16], 1);
    let limits = &engine.limits.core;
    let current = settings::Current::new(config.defaults, limits, &[]);
    let charters = reference_charters();
    assert_eq!(charters.len(), 2);
    assert_eq!(settings::check_change(&current, &charters, &config.settings, &config.connectors, limits), Ok(()));
    let mut rules = reference_rules(&current);
    for kinds in reference_delegations() {
        let count = u32::try_from(kinds.len()).expect("reference kind count");
        rules.ceiling.delegation.kinds = List::with_capacity(count);
        for kind in kinds {
            rules.ceiling.delegation.kinds.push(kind).expect("delegation count");
        }
        let mut items = charters.clone();
        items.push(core_item(&SettingItem {
            setting: Setting::Rules,
            edit: SettingEdit::Set(SettingSet { value: SettingValue::Rules(Box::new(rules.clone())) }),
        }));
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
