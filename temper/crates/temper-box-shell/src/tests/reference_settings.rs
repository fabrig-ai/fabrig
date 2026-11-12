//! The reference's settings file (temper/docs/design/protocol/configuration/reference.md).
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
const STATUS: u32 = 9;
const GOAL: u32 = 4;
// The reference supplies five charters, one template, its work and two landing rules.
const ITEMS: u32 = 9;

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
    reason = "one reference fixture checks designer choices and coordinator prohibitions together"
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
        assert_eq!(charter.turns, 800);
        assert_eq!(charter.time, Duration::from_secs(28_800));
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
    coordinator_rules(coordinator);
    assert!(!coordinator.resume());
    assert_eq!(coordinator.waiting, Duration::ZERO);
    assert_eq!(
        coordinator.wake,
        WakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Immediate,
            news: WakeRule::Paced { span: Duration::from_secs(1800) },
            results: ResultsWake::LastOrFailure,
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
    let designer_text = std::str::from_utf8(designer.instructions.bytes()).expect("designer UTF-8");
    assert!(!designer_text.contains("close-out"));
    assert!(designer_text.contains("Tag each Not yet entry a change of your family will close with `[in flight: change N]` once `change` has answered its number; your requester tags those made after your report."));
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
    let coder_instructions = std::str::from_utf8(coder.instructions.bytes()).expect("coding instructions are UTF-8");
    assert!(coder_instructions.contains("in the question form"));
    assert!(coder_instructions.contains("After asking, call wait, not finish: the answer reaches this run."));
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
        coder_instructions
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
    let SettingValue::Application(bytes) = value(&items, Setting::Application { kind: 11, scope: 0, number: GOAL })
    else {
        panic!("goal template's work")
    };
    let decoded = temper_engine_smith::kinds::decode(
        11,
        bytes,
        &temper_engine_smith::kinds::Bounds { segments: 16, segment_bytes: 4096, gates: 64, approvals: 64 },
    )
    .expect("reference goal template's work");
    let temper_engine_domain_forge::settings::Value::Goal(work) = decoded.value else {
        panic!("goal template's work record")
    };
    assert_eq!((work.design, work.close_out), (true, false));
}

// Rules pin the shared instructions or the wake parts selected by their on names.
fn coordinator_rules(coordinator: &settings_form::view::Charter) {
    let shared = "shared";
    let cut = "first";
    let questions = "question question_answered question_ended question_owed words";
    let results = "escalation proposal person_stalled result notice nothing_awaited";
    let news = "news timer";
    let mut texts = vec![(shared.to_owned(), std::str::from_utf8(coordinator.instructions.bytes()).expect("shared"))];
    for part in &coordinator.parts {
        let on = settings_form::view::REASON_NAMES
            .iter()
            .enumerate()
            .filter(|(bit, _)| part.on & (1 << bit) != 0)
            .map(|(_, name)| std::str::from_utf8(name).expect("reason name"))
            .collect::<Vec<_>>()
            .join(" ");
        texts.push((on, std::str::from_utf8(part.instructions.bytes()).expect("part")));
    }
    assert_eq!(texts.iter().map(|(on, _)| on.as_str()).collect::<Vec<_>>(), [shared, cut, questions, results, news]);
    coordinator_shared_rules(&texts);
    coordinator_part_rules(&texts);
    coordinator_news_rules(&texts);
    coordinator_dependencies(&texts);
    for (on, text) in &texts {
        assert!(
            !text.contains("On your first run, once your first batch is made, subscribe with `subscribe_forge` to the landings on main in your repository."),
            "{on}: the core already subscribes the goal to its changes' landings"
        );
    }
    let prohibition = "Never land your goal, make a close-out or a catch-up, or finish once your changes have ended: your closer settles your goal's status, lands it and reports, and comes to you only through an escalation.";
    assert_eq!(texts[0].1.matches(prohibition).count(), 1);
    let without_prohibition = texts[0].1.replace(prohibition, "");
    for (on, text) in &texts {
        let rest = if on == shared { without_prohibition.as_str() } else { *text }.to_ascii_lowercase();
        for forbidden in [r#""branch": "mine""#, r#""merge""#, "catch", "close-out", "read_task"] {
            assert!(!rest.contains(forbidden), "{on}: {forbidden}");
        }
    }
}

fn coordinator_shared_rules(texts: &[(String, &str)]) {
    let shared = "shared";
    for (phrase, place) in [
        (r#""producer": 7"#, shared),
        (r#""questions": "foreman""#, shared),
        (r#""questions": "up""#, shared),
        ("Make a larger batch in several calls", shared),
        ("report: ", shared),
        ("Left: ", shared),
        ("Risk: ", shared),
        ("Decided: ", shared),
        ("your goal's scope", shared),
        ("Revise that note in place", shared),
        ("body only these lines, one item each, each only if it holds", shared),
        ("Write no narrative: your tree and your closer's report hold the rest", shared),
        ("After planning and whenever a decision is taken or a risk found", shared),
        ("[in flight: change N]", shared),
        ("watch with `subscribe` each person task you wait on", shared),
        ("Watch each designer's change with `subscribe` for its result", shared),
        ("Then watch it with `subscribe` and call wait", shared),
        ("Read your report note and the tree first at every wake", shared),
        ("park with `wait` once your decisions are made", shared),
        ("an answer", shared),
        ("a decision on your proposal or settings", shared),
        ("an effect not made", shared),
        ("a `met`", shared),
        ("an amendment changes what you are to do", shared),
        ("a request to finish", shared),
        ("a person's words", shared),
        ("leave it undecided for the run whose part it is", shared),
        ("the lints and the focused tests of the packages it changes", shared),
        ("as charter 2 runs them", shared),
        ("never for a slow tier or a walk, which run after landing", shared),
        ("with `note`", shared),
    ] {
        coordinator_phrase(texts, phrase, place);
    }
}

fn coordinator_part_rules(texts: &[(String, &str)]) {
    let cut = "first";
    let questions = "question question_answered question_ended question_owed words";
    let results = "escalation proposal person_stalled result notice nothing_awaited";
    for (phrase, place) in [
        ("Where a request settles its changes with no shape for the code to follow", cut),
        ("write no design, say so in your report note, and make its changes", cut),
        ("The core subscribes you to your goal branch's landings", cut),
        ("Never subscribe to main's landings", cut),
        ("On a designer's question, answer from the draft", questions),
        ("On a question a change relays", questions),
        ("Never answer what you have not settled.", questions),
        ("On a person task's result, answer the change that waited for it", results),
        ("with decide, saying why", results),
        ("On a task's words, read them and act as on a question", questions),
        ("On a change that failed, read why", results),
        ("On a designer's result, which reaches you because you watched its change with `subscribe`", results),
        ("When your closer escalates (its change settling your status", results),
        ("then release: your closer waits for it and lands the new head", results),
        ("Cancel your closer only to drop the landing", results),
    ] {
        coordinator_phrase(texts, phrase, place);
    }
}

fn coordinator_news_rules(texts: &[(String, &str)]) {
    let news = "news timer";
    for (phrase, place) in [
        ("delegate its triage to charter 8", news),
        (r#""repairs""#, news),
        ("a red of your goal branch's tier", news),
        ("While your goal's landing is verified, when a red after landing reaches you as news", news),
    ] {
        coordinator_phrase(texts, phrase, place);
    }
}

fn coordinator_phrase(texts: &[(String, &str)], phrase: &str, place: &str) {
    for (on, text) in texts {
        assert_eq!(text.contains(phrase), on == place, "{phrase:?}: expected only in {place}, checked {on}");
    }
}

fn coordinator_dependencies(texts: &[(String, &str)]) {
    let first = texts.iter().find(|(on, _)| on == "first").expect("first-run instructions").1;
    assert!(first.contains("Keep coding that uses pending design out of the first batch"));
    assert!(!first.contains("and the changes the design already specifies, after S where they use its shapes"));
    for (on, text) in texts {
        if on == "shared" || on == "first" || on == "escalation proposal person_stalled result notice nothing_awaited" {
            assert!(
                text.contains("Do not issue dependent coding until all required design is confirmed landed into g"),
                "{on}: wait for design landing before coding dispatch"
            );
            assert!(text.contains("A push, delivery or designer report is insufficient"));
        }
        if on == "shared" || on == "escalation proposal person_stalled result notice nothing_awaited" {
            assert!(text.contains("Use after only for live prerequisites;"));
            assert!(text.contains(
                "once design has landed into g, cite its path and section in words and omit its ended task from after"
            ));
        }
        assert!(!text.contains("each \"after\" its dependencies, made or in the same call"));
        assert!(!text.contains("a later call naming an earlier one's changes in \"after\""));
    }
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
    let mut charters = stock.charters.len();
    for item in &items {
        if let SettingEdit::Set(set) = &item.edit
            && let SettingValue::Charter(charter) = &set.value
        {
            if !stock.charters.iter().any(|existing| existing.number == charter.number) {
                charters += 1;
            }
            assert!(charter.documents.len() <= limits.charter_documents, "tasks.charter_documents");
            assert!(u64::from(charter.turns) <= max_turns, "charter {}", charter.number);
        }
    }
    assert!(charters <= usize::try_from(limits.charters).expect("charter limit"));
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
    use jig_domain_core::settings;
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
    let reference = temper_client_tools::jig::settings_file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::tools(),
    )
    .expect("reference settings");
    for number in [CODER, COORDINATOR, FOREMAN, DESIGNER, TRIAGE] {
        let item = reference
            .iter()
            .find(|item| {
                item.setting == jig_client::contract::Setting::Charter(jig_client::contract::SettingNumber { number })
            })
            .expect("reference charter");
        let settings::Edit::Set(settings::Value::Charter(charter)) = core_item(item).edit else {
            panic!("charter {number}")
        };
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
        .filter(|item| {
            matches!(item.setting, Setting::Charter(SettingNumber { number: COORDINATOR | DESIGNER | TRIAGE }))
        })
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

fn reference_delegations() -> [Vec<jig_client::contract::AuthorityExecutor>; 1] {
    use jig_client::contract::AuthorityExecutor as Executor;
    let eighteen = vec![
        Executor::Charter(1),
        Executor::Charter(2),
        Executor::Charter(4),
        Executor::Charter(COORDINATOR),
        Executor::Charter(FOREMAN),
        Executor::Charter(DESIGNER),
        Executor::Charter(TRIAGE),
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
    [eighteen]
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
    assert_eq!(charters.len(), 3);
    assert_eq!(settings::check_change(&current, &charters, &config.settings, &config.connectors, limits), Ok(()));
    let mut rules = reference_rules(&current);
    for kinds in reference_delegations() {
        let count = u32::try_from(kinds.len()).expect("reference kind count");
        assert_eq!(count, 18);
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
    assert_eq!(stock.rules.ceiling.delegation.kinds.len(), 21);
    let source = reads::SettingDocument {
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
        assert_eq!(rules.ceiling.delegation.kinds.len(), 21, "{name}");
        for form in [temper_client_tools::limits::form(), settings_form::Limits::standard()] {
            let restored = settings_form::wire::restore_setting_document(&decoded, &form).expect("settings form");
            let Some(SettingValue::Rules(rules)) = restored.value else { panic!("form rules") };
            assert_eq!(rules.ceiling.delegation.kinds.len(), 21);
        }
    }
}
