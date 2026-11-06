//! Stock settings are checked by jig and document members name their store replacement.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary step tests generate documents and compare embedded instruction bytes"
)]
use super::fixtures;
use jig_protocol_config as config;

/// The complete startup document emitted by init for fixed machine facts.
pub(super) fn generated() -> Box<[u8]> {
    use core::num::NonZeroU16;
    let answer = temper_init::files(
        &temper_init::Facts {
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"ada"),
            ports: temper_init::Ports {
                clients: NonZeroU16::new(8367).unwrap(),
                owned: NonZeroU16::new(8368).unwrap(),
                forge: NonZeroU16::new(8369).unwrap(),
                llm: NonZeroU16::new(8370).unwrap(),
                webhooks: NonZeroU16::new(8371).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
    )
    .unwrap();
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("document bytes") };
            return bytes;
        }
    }
    panic!("init writes temper.json")
}
fn single_model() -> Box<[u8]> {
    let mut document = generated();
    for (path, first) in [
        (b"engine.llm.providers".as_slice(), b"engine.llm.providers[0]".as_slice()),
        (b"engine.llm.providers[0].models", b"engine.llm.providers[0].models[0]"),
        (b"engine.llm.endpoints", b"engine.llm.endpoints[0]"),
        (b"engine.llm.models", b"engine.llm.models[0]"),
    ] {
        let one = [b"[".as_slice(), &fixtures::value(&document, first), b"]"].concat();
        document = fixtures::with(&document, path, &one);
    }
    fixtures::with(&document, b"application.demo", b"null")
}
fn engine(document: &[u8]) -> config::Engine {
    fixtures::parse_engine(&fixtures::value(document, b"engine"))
}
fn without_projects() -> Box<[u8]> {
    let mut document = generated();
    for (path, value) in [(b"engine.projects".as_slice(), b"[]".as_slice()), (b"application.demo", b"null")] {
        document = fixtures::with(&document, path, value);
    }
    document
}
/// Replace one engine override, keeping every other generated entry.
pub(super) fn limited(document: &[u8], name: &[u8], value: u64) -> Box<[u8]> {
    let original = fixtures::value(document, b"engine.limits.overrides");
    let parsed =
        config::document(&original, &config::DocumentLimits { bytes: 16_384, tokens: 4096, depth: 4 }).unwrap();
    let mut overrides = format!("{{\"{}\":{value}", core::str::from_utf8(name).unwrap());
    for at in (1..parsed.len().checked_sub(1).unwrap()).step_by(2) {
        let key = parsed.text(parsed.token(at).unwrap()).unwrap();
        if key != name {
            let value = config::read::integer(&parsed, at.checked_add(1).unwrap(), u64::MAX).unwrap();
            overrides.push_str(",\"");
            overrides.push_str(core::str::from_utf8(key).unwrap());
            overrides.push_str("\":");
            overrides.push_str(&value.to_string());
        }
    }
    overrides.push('}');
    fixtures::with(document, b"engine.limits.overrides", overrides.as_bytes())
}

#[test]
fn stock_chats_can_delegate_message_and_read_the_forge() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let policy = &defaults.policies.get(&1).expect("stock policy 1").policy;
    let families = jig_domain_core::ToolFamilies::JIG;
    for authority in [&defaults.rules.ceiling, &policy.ceiling, &policy.roles[0].authority] {
        assert_eq!(
            authority.tools,
            jig_domain_authority::Tools(
                families.delegate.0
                    | families.message.0
                    | families.control.0
                    | families.note.0
                    | families.recall.0
                    | families.read.0
                    | families.settings
            ),
            "a stock chat's authority holds notes and recall"
        );
        assert_eq!(authority.delegation.tasks, 384);
        assert_eq!(authority.delegation.depth, 32);
    }
}

#[test]
fn the_stock_holds_a_goals_note_scope_and_an_observer_none() {
    use jig_domain_authority::{Scopes, Tools};
    let engine = engine(&single_model());
    let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let policy = &stock.policies.get(&1).expect("stock policy 1").policy;
    for authority in [&stock.rules.ceiling, &policy.ceiling]
        .into_iter()
        .chain(policy.roles.iter().filter(|role| role.authority.budget.spend != 0).map(|role| &role.authority))
    {
        assert_eq!(authority.notes, Scopes::GOAL);
        assert_eq!(authority.tools, Tools(1927));
        assert!(authority.note_resources.is_empty());
    }
    let observer = &policy.roles.iter().find(|role| role.number == 3).expect("observer").authority;
    assert_eq!(observer.notes, Scopes(0));
    assert!(observer.note_resources.is_empty());
}

// temper/docs/design/protocol/configuration.md, 13.9: reference settings widen stock authority.
#[test]
fn configure_reference_authority_allows_coordinator_tools() {
    use jig_domain_authority::{Answer, Call, NoteScope, Scopes};
    let engine = engine(&generated());
    let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let policy = stock.policies.get(&1).unwrap().policy.clone();
    let before = reference_domain(stock.rules.clone(), policy.clone(), engine.limits.core.authority);
    for family in [8, 32] {
        assert_eq!(reference_call(&before, &policy.roles[0].authority, family, Call::Tool), Answer::Refuse);
    }
    for family in [128, 256] {
        assert_eq!(reference_call(&before, &policy.roles[0].authority, family, Call::Tool), Answer::Allow);
    }
    let required = reference_requirements();
    assert_eq!(required.tools, 440);
    assert_eq!(required.notes, 3);
    let mut rules = stock.rules.clone();
    let mut configured = policy.clone();
    widen_reference(&mut rules.ceiling, &required);
    widen_reference(&mut configured.ceiling, &required);
    widen_reference(&mut configured.roles[0].authority, &required);
    for (old, new) in [
        (&stock.rules.ceiling, &rules.ceiling),
        (&policy.ceiling, &configured.ceiling),
        (&policy.roles[0].authority, &configured.roles[0].authority),
    ] {
        assert_reference_authority(old, new);
    }
    let after = reference_domain(rules, configured.clone(), engine.limits.core.authority);
    let caller = &configured.roles[0].authority;
    for family in [8, 16, 32, 128, 256] {
        assert_eq!(reference_call(&after, caller, family, Call::Tool), Answer::Allow);
    }
    for scope in [NoteScope::Goal, NoteScope::Project] {
        assert_eq!(reference_call(&after, caller, 128, Call::Note(scope.clone())), Answer::Allow);
        let mut without_scope = caller.clone();
        without_scope.notes = Scopes(0);
        assert_eq!(reference_call(&after, &without_scope, 128, Call::Note(scope)), Answer::Propose);
    }
    // Stock chats already hold recall before the project's policy widens.
    assert_eq!(reference_call(&after, &policy.roles[0].authority, 256, Call::Tool), Answer::Allow);
}

fn assert_reference_authority(old: &jig_domain_authority::Authority, new: &jig_domain_authority::Authority) {
    use jig_domain_authority::{Executor, Scopes, Tools};
    assert_eq!(new.tools, Tools(1983));
    assert_eq!(new.notes, Scopes(3));
    let mut restored = new.clone();
    restored.tools = old.tools;
    restored.notes = old.notes;
    restored.delegation.kinds = old.delegation.kinds.clone();
    assert_eq!(&restored, old, "budgets, grants and other authority fields survive");
    for executor in [Executor::Charter(5), Executor::Charter(7), Executor::Role(0)] {
        assert!(new.delegation.kinds.contains(&executor));
    }
}

fn reference_requirements() -> settings_form::view::Authority {
    use settings_form::view::{SettingEdit, SettingValue};
    let help = include_str!("../../../../docs/help/configure.md");
    let json = help
        .split_once("cat > reference-authority.json <<'JSON'\n")
        .expect("configure documents reference authority")
        .1
        .split_once("\nJSON")
        .unwrap()
        .0;
    let mask = |member: &[u8]| {
        let bytes = fixtures::value(json.as_bytes(), member);
        let doc = config::document(&bytes, &config::DocumentLimits { bytes: 1024, tokens: 64, depth: 4 }).unwrap();
        (1..doc.len().checked_sub(1).unwrap())
            .fold(0, |mask, at| mask | config::read::integer(&doc, at, u64::MAX).unwrap())
    };
    let kinds = fixtures::value(json.as_bytes(), b"kinds");
    let file = format!(
        r#"{{"items":[{{"setting":{{"kind":"rules"}},"set":{{
        "ceiling":{{"models":"all","tools":{},"grants":[],
        "delegation":{{"kinds":{},"tasks":0,"depth":0}},"spend":0,"deadline":null,
        "notes":{},"note_resources":[]}},"period_spend":0,"minimum_run_spend":0,
        "maximum_run_spend":0,"implies":[],"requirements":[]}}}}]}}"#,
        mask(b"tools"),
        core::str::from_utf8(&kinds).unwrap(),
        mask(b"notes")
    );
    let items = settings_form::file::decode(file.as_bytes(), &temper_client_tools::limits::form())
        .expect("documented tools, note scopes and executors decode as settings values");
    let SettingEdit::Set(set) = &items.get(0).unwrap().edit else { panic!("set rules") };
    let SettingValue::Rules(rules) = &set.value else { panic!("rules value") };
    rules.ceiling.clone()
}

fn widen_reference(authority: &mut jig_domain_authority::Authority, required: &settings_form::view::Authority) {
    use jig_domain_authority::Executor;
    authority.tools.0 |= required.tools;
    authority.notes.0 |= required.notes;
    let mut kinds = authority.delegation.kinds.to_vec();
    for executor in &required.delegation.kinds {
        let executor = match executor {
            settings_form::view::AuthorityExecutor::Charter(n) => Executor::Charter(*n),
            settings_form::view::AuthorityExecutor::Procedure(n) => Executor::Procedure(*n),
            settings_form::view::AuthorityExecutor::Role(n) => Executor::Role(*n),
            settings_form::view::AuthorityExecutor::Job(n) => Executor::Job(*n),
        };
        if !kinds.contains(&executor) {
            kinds.push(executor);
        }
    }
    authority.delegation.kinds = kinds.into_boxed_slice();
}

fn reference_domain(
    rules: jig_domain_authority::Rules,
    policy: jig_domain_authority::Policy,
    limits: jig_domain_authority::Limits,
) -> jig_domain_authority::Domain {
    use jig_domain_authority::{Domain, Event, PolicyFact, step};
    let mut domain = Domain::new(rules, limits).unwrap();
    let mut facts = skein_lib::Queue::uncapped();
    step(&mut domain, Event::Policy { project: 1, policy }, &mut facts);
    assert_eq!(facts.pop(), Some(PolicyFact::Added { project: 1 }));
    domain
}

fn reference_call(
    domain: &jig_domain_authority::Domain,
    authority: &jig_domain_authority::Authority,
    family: u64,
    call: jig_domain_authority::Call,
) -> jig_domain_authority::Answer {
    let ask = jig_domain_authority::CallAsk {
        project: 1,
        authority: authority.clone(),
        family: jig_domain_authority::Tools(family),
        call,
    };
    jig_domain_authority::check_call(domain, &ask, &mut skein_lib::Queue::uncapped())
}

#[test]
fn stock_chats_inspect_and_delegate_while_coders_modify() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    for charter in &defaults.charters {
        assert_eq!(charter.policy.inspect, charter.number != 9);
        assert!(!charter.policy.shell);
        assert_eq!(charter.policy.modify, charter.number == 2);
        assert_eq!(charter.policy.agents, charter.number == 1 || charter.number == 2);
    }
}

#[test]
fn the_configuring_agent_is_charter_3_and_template_3() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    assert_eq!(
        defaults.charters.iter().map(|charter| charter.number).collect::<Box<[_]>>().as_ref(),
        [1, 2, 3, 4, 6, 9]
    );
    for (charter, tools) in defaults.charters.iter().zip([1023, 514, 1920, 512, 512, 0]) {
        assert_eq!(charter.policy.tools, jig_domain_tasks::Tools(tools));
    }
    let mut expected = defaults.charters[0].clone();
    expected.number = 3;
    expected.policy.instructions = include_bytes!("../stock/configuring.txt").as_slice().into();
    expected.policy.agents = false;
    expected.policy.tools = jig_domain_tasks::Tools(1920);
    assert_eq!(defaults.charters[2], expected);
    assert!(!defaults.charters[2].policy.modify);
    assert!(!defaults.charters[2].policy.agents);
    assert_eq!(defaults.charters[2].policy.instructions.len(), 4807);
    assert_eq!(defaults.templates.iter().map(|template| template.number).collect::<Box<[_]>>().as_ref(), [1, 2, 3]);
    let mut expected = defaults.templates[0].clone();
    expected.number = 3;
    expected.name = b"configuring".as_slice().into();
    expected.executor = jig_domain_people::TemplateExecutor::Charter(3);
    assert_eq!(defaults.templates[2], expected);
}

#[test]
fn the_stock_passes_jigs_one_check() {
    let init = generated();
    let without_demo = fixtures::with(&init, b"application.demo", b"null");
    for (row, document) in [("init", without_demo), ("single model", single_model()), ("demo", init)] {
        crate::named(&document, &config::Directories::working()).expect("stock passes startup parsing");
        let engine = engine(&document);
        let core = config::core(&engine, [1; 16], 7);
        let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
        assert_eq!(defaults.charters[0].policy.instructions.as_ref(), include_bytes!("../stock/chat.txt"));
        assert_eq!(defaults.charters[1].policy.instructions.as_ref(), include_bytes!("../stock/coding.txt"));
        for charter in &defaults.charters {
            assert_eq!(charter.hosts, jig_domain_fleet::Kinds::Workers, "{row}");
        }
        let current = jig_domain_core::settings::Current::new(defaults, &engine.limits.core, &[]);
        assert_eq!(
            jig_domain_core::settings::check(&current, &core.settings, &core.connectors, &engine.limits.core),
            Ok(()),
            "{row}"
        );
    }
}

#[test]
fn each_floor_refuses_one_below_the_stocks_need() {
    use jig_domain_core::settings::{self, Setting};
    let document = without_projects();
    let engine = engine(&document);
    let core = config::core(&engine, [1; 16], 7);
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let mut run = (0, Setting::Charter(1));
    for charter in &defaults.charters {
        let bytes = jig_domain_core::policy_owned_bytes(&charter.policy).unwrap();
        if bytes > run.0 {
            run = (bytes, Setting::Charter(charter.number));
        }
    }
    let current = settings::Current::new(defaults, &engine.limits.core, &[]);
    let rows: &[(&[u8], u64, Setting, &str)] = &[
        (b"core.policy_bytes", 5774, Setting::Policy(1), "the stock's policy 1 needs 5774 bytes"),
        (b"core.run_bytes", run.0, run.1, ""),
        (b"authority.grants", 14, Setting::Rules, "the stock's rules need 14 grants"),
        (b"authority.executors", 9, Setting::Rules, "the stock's rules need 9 executors"),
        (b"authority.segments", 1, Setting::Rules, "the stock's rules need 1 segment"),
        (b"authority.segment_bytes", 5, Setting::Rules, "the stock's rules need 5 bytes"),
        (b"authority.roles", 4, Setting::Policy(1), "the stock's policy 1 needs 4 roles"),
        (b"authority.projects", 1, Setting::Policy(1), "the stock offers 1 policy"),
        (b"tasks.charters", 6, Setting::Charter(1), "the stock offers 6 charters"),
        (b"core.templates", 3, Setting::Template(1), "the stock offers 3 templates"),
        (b"documents.text_bytes", 11, Setting::Template(3), "the stock's template 3 needs 11 bytes"),
        (b"tasks.grace_ms", 5000, Setting::Template(1), "the stock's template 1 needs 5000 ms of grace"),
        (b"core.settings.alternatives", 3, Setting::Charter(1), "the stock's charter 1 needs 3 alternatives"),
        (b"core.settings.permissions", 3, Setting::Policy(1), "the stock's policy 1 needs 3 permissions"),
    ];
    // Every positive scalar bound reached by jig's whole-set check is at its
    // floor together. This catches a stock value acquiring another unmet bound.
    let mut minimum = engine.limits;
    for &(name, needed, _, _) in rows {
        config::overrides::apply(&mut minimum, name, needed).unwrap();
    }
    assert_eq!(settings::check(&current, &core.settings, &core.connectors, &minimum.core), Ok(()));
    assert_eq!(
        crate::stock::floors(&crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes), &minimum),
        Ok(())
    );
    for &(name, needed, setting, words) in rows {
        let mut at = fixtures::parse_engine(&fixtures::value(&document, b"engine"));
        config::overrides::apply(&mut at.limits, name, needed).unwrap();
        let mut problems = crate::startup::Problems::empty();
        crate::startup::checks::stock(&at, &mut problems);
        assert!(problems.is_empty(), "{name:?}");
        assert_eq!(settings::check(&current, &core.settings, &core.connectors, &at.limits.core), Ok(()));
        if let Err(why) = crate::startup::named(&limited(&document, name, needed), &config::Directories::working()) {
            panic!("{name:?} at its floor: {:?}", crate::startup::lines(&why));
        }
        config::overrides::apply(&mut at.limits, name, needed - 1).unwrap();
        assert!(settings::check(&current, &core.settings, &core.connectors, &at.limits.core).is_err(), "{name:?}");
        let found = crate::stock::floors(
            &crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes),
            &at.limits,
        )
        .unwrap_err();
        assert_eq!(found, crate::stock::Floor { limit: name, configured: needed - 1, needed, setting });
        crate::startup::checks::stock(&at, &mut problems);
        let words = if name == b"core.run_bytes" {
            let Setting::Charter(number) = setting else { panic!("run policy's charter") };
            format!("the stock's charter {number} needs {needed} bytes")
        } else {
            words.to_owned()
        };
        let name = core::str::from_utf8(name).unwrap();
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!(
                "engine.limits.overrides.{name}: {name} is {}; {words}; raise it in engine.limits.overrides\n",
                needed - 1
            )
            .as_bytes()
        );
        let refused =
            crate::startup::named(&limited(&document, name.as_bytes(), needed - 1), &config::Directories::working())
                .unwrap_err();
        assert_eq!(crate::startup::lines(&refused), crate::startup::lines(&problems));
    }
}

#[test]
fn each_charters_run_bytes_floor_names_that_charter() {
    use jig_domain_core::settings::Setting;
    let engine = engine(&without_projects());
    for charter in &crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes).charters {
        let mut defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
        // Isolate each run policy from larger stock runs.
        defaults.charters = Box::new([charter.clone()]);
        let needed = jig_domain_core::policy_owned_bytes(&charter.policy).unwrap();
        let mut limits = engine.limits;
        limits.core.run_bytes = u32::try_from(needed).unwrap();
        assert_eq!(crate::stock::floors(&defaults, &limits), Ok(()));
        limits.core.run_bytes -= 1;
        assert_eq!(
            crate::stock::floors(&defaults, &limits),
            Err(crate::stock::Floor {
                limit: b"core.run_bytes",
                configured: needed - 1,
                needed,
                setting: Setting::Charter(charter.number),
            })
        );
    }
}

#[test]
fn a_worlds_own_short_runs_do_not_need_the_stocks_run_bytes() {
    let document = without_projects();
    let engine = engine(&document);
    let mut own = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let mut needed = 0;
    for charter in &mut own.charters {
        charter.policy.instructions = Box::default();
        needed = needed.max(jig_domain_core::policy_owned_bytes(&charter.policy).unwrap());
    }
    let document = limited(&document, b"core.run_bytes", needed);
    let refused = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
    assert!(core::str::from_utf8(&crate::startup::lines(&refused)).unwrap().contains("the stock's charter"));
    crate::startup::named_with(
        &document,
        &config::Directories::working(),
        &crate::stock::Given::Own { jig: own, forge: Box::default() },
    )
    .expect("a world sizes its own runs");
}

#[test]
fn a_stock_policy_past_its_limit_is_refused_while_naming_files() {
    let document = limited(&without_projects(), b"core.policy_bytes", 100);
    // named only parses the document and lists its files. No file read or
    // listener opening can occur before this refusal.
    let found = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(
        crate::startup::lines(&found).as_ref(),
        b"engine.limits.overrides.core.policy_bytes: core.policy_bytes is 100; the stock's policy 1 needs 5774 bytes; raise it in engine.limits.overrides\n"
    );
}
#[test]
fn the_chats_opening_sends_plain_edits_directly_to_change() {
    let words = include_str!("../stock/chat.txt");
    let opening = words.split("If your brief says you are an advisor").next().expect("opening");
    assert!(
        opening.contains(
            "Repository changes must go through change; never implement them by writing your chat workspace."
        )
    );
    assert!(opening.contains("then call change with their request and only what you read"));
}

#[test]
fn the_chats_words_send_plain_changes_to_change() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.starts_with("Help the person with their work.\n"));
    assert!(words.contains("then call change with their request and only what you read"));
    assert!(words.contains("Call goal only when they want to hear a plan or give a go-ahead before anything changes"));
    assert!(words.contains("never call change for work they want agreed first"));
    assert!(!words.contains("never call change yourself"));
    assert!(words.contains("Only if your brief says the work is to be agreed"));
    assert!(words.contains("Otherwise do the work without asking."));
    assert!(words.contains(
        "If your brief says you are a goal, use this goal paragraph; the chat and document paragraphs are for chats alone."
    ));
    assert!(words.contains(
        "If neither line is in your brief, you are the person's chat, and the following document rule is also yours."
    ));
}

#[test]
fn the_goals_charter_asks_for_one_report_revised_in_place() {
    let words = include_str!("../stock/chat.txt");
    let goal = words
        .split("\n\n")
        .find(|paragraph| paragraph.starts_with("If your brief says you are a goal"))
        .expect("goal paragraph");
    for phrase in [
        "After planning",
        "report: ",
        "Left: ",
        "Risk: ",
        "Decided: ",
        "naming the task and project your brief's task section names",
        "in the first person",
        "Revise it in place, naming the note and its revision from your brief's index",
        "A message that changes the plan becomes an amendment you show your person first.",
    ] {
        assert!(goal.contains(phrase), "{phrase}");
    }
}

// temper/docs/design/protocol/configuration.md, 13.4; domain/agent.md, 4.3.
#[test]
fn a_goal_is_told_to_call_decided_before_it_asks() {
    let words = include_str!("../stock/chat.txt");
    let goal = words
        .split("\n\n")
        .find(|paragraph| paragraph.starts_with("If your brief says you are a goal"))
        .expect("goal paragraph");
    assert!(goal.contains("Otherwise do the work without asking. Before you ask your requester anything, call decided with the question, its options, the one you'd take, why, and its kind; on Decided carry on with it and ask no one; on Ask, ask them with message, as a question."), "the goal calls decided before asking and follows its answer");
    assert!(
        goal.contains("Funds, sign-ins, machines, a plan's go-ahead and anything hard to reverse are kind other."),
        "these decisions wait for the person"
    );
    assert!(
        goal.contains("If they tell you a decision was overturned, ask it with message, as a question, and wait."),
        "an overturned decision waits for the person's answer"
    );
}

#[test]
fn asks_always_is_told_never_to_call_it() {
    let words = include_str!("../stock/chat.txt");
    let goal = words
        .split("\n\n")
        .find(|paragraph| paragraph.starts_with("If your brief says you are a goal"))
        .expect("goal paragraph");
    assert!(
        goal.contains(
            "If your words hold the line Asks: always, never call decided: ask them with message, as a question."
        ),
        "Asks: always bypasses decided and asks the person"
    );
}

#[test]
fn the_chats_words_tell_the_person_when_its_change_or_goal_holds() {
    let words = include_str!("../stock/chat.txt");
    let chat = words.split("\n\n").find(|paragraph| paragraph.starts_with("If neither line")).expect("chat paragraph");
    assert!(chat.contains("When a change or goal you started holds, tell the person at once in words what it waits on and that it waits in their inbox for them to decide: release it to go on, or cancel it; then wait."));
}

#[test]
fn the_chats_words_send_work_agreed_first_to_a_goal() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("If your brief says you are a goal"), "a goal is told apart by its root brief");
    assert!(
        words.contains("Only if your brief says the work is to be agreed: before anything is changed, send your requester your plan with message, as a question asking whether to go ahead, and call wait until it is answered: never call finish before that answer, nor before every agreed change has landed; if the answer declines, report that and change nothing."),
        "the goal calls wait after asking and finishes only after the answer and all landings"
    );
    assert!(
        words.contains("Call goal only when they want to hear a plan or give a go-ahead before anything changes, with their words whole at once and agree set, even where they ask for an advisor first, since the goal asks one itself: the goal tells its plan and asks them; or when they want a survey that changes nothing. Never write a plan yourself, never ask for their go-ahead in your own reply, never set advisor, and never call change for work they want agreed first. Tell them a question will wait in their inbox."),
        "the chat sends work agreed first to goal with the person's words"
    );
    assert!(
        words.contains("Tell them a question will wait in their inbox"),
        "the chat names the waiting inbox question"
    );
    assert!(
        words.contains("Never answer the work's question yourself: your person answers it in their inbox, and the goal hears it there."),
        "the person answers the question in their inbox and the goal hears it there"
    );
    assert!(words.contains("When the person answers your goal's question, the goal has the answer and does the work: never call change or goal for it yourself; tell the person the work goes ahead."));
}

#[test]
fn the_chats_words_mark_an_advisors_read_only_report() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.split("\n\n").any(|paragraph| paragraph == "If your brief says you are an advisor, do only that: read what it asks with the forge's reads, change nothing, ask no one, and report what you found."), "the advisor has its own paragraph");
}

#[test]
fn only_the_goals_paragraph_ends_with_the_questions_line_form() {
    // domain/agent.md, 8.2: a goal writes its question in the client's line form.
    const SENTENCE: &str = " Write a question to your requester as one line asking it, a blank line, what they need to know, then a line starting Saw: with what you saw, and a line starting Choice: for each option, the one you recommend ending (recommended).";
    let words = include_str!("../stock/chat.txt");
    for (role, expected) in [
        ("If your brief says you are a goal", true),
        ("If neither line is in your brief", false),
        ("If your brief says you are an advisor", false),
    ] {
        let paragraph = words.split("\n\n").find(|paragraph| paragraph.starts_with(role)).expect("role instructions");
        assert_eq!(paragraph.contains(SENTENCE.trim_start()), expected, "{role}: question line form");
        assert_eq!(paragraph.ends_with(SENTENCE), expected, "{role}: question line form ends the paragraph");
    }
}

#[test]
fn the_goals_words_survey_before_planning_and_sequence_changes() {
    let words = include_str!("../stock/chat.txt");
    assert!(
        words.contains("The work and its changes are yours, so call goal only with advisor set"),
        "the goal owns its work and changes and delegates only advisors with goal"
    );
    assert!(words.contains("if your words ask for an advisor, or you need the code read before you can plan, call goal with advisor set and words saying what to read and report; wait for its report, and name what it found in your plan."), "the goal surveys before planning");
    assert!(words.contains("Make each change with change, one at a time, waiting for its result: when one builds on another, call change for it only after the first's result says it landed, and say in its words what to reuse."), "the goal waits for landing before dependent changes");
    assert!(
        words.contains(
            "Report once every change has landed, each as its result's Landed line says it: that it landed, its pull request and its merge commit; name no check or test it doesn't."
        ),
        "the goal reports after every change lands"
    );
}

#[test]
fn the_chats_words_say_a_delegate_is_started_until_it_reports() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("After calling change, say only that work started until the delegate reports."));
    assert!(words.contains("For successful NoChange, tell the person nothing changed and why from the delegate's conclusion; claim no landing, implementation or tests."));
}

#[test]
fn the_goal_and_chat_report_only_the_checks_named_in_the_result() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("Report the goal's landed pull requests and merge commits."));
    assert!(!words.contains("When the goal reports, tell them what landed."));
    assert!(!words.contains(
        "Report once every change has landed, each as its result says it, naming no check or test it doesn't."
    ));
    assert!(
        words.contains(
            "Report once every change has landed, each as its result's Landed line says it: that it landed, its pull request and its merge commit; name no check or test it doesn't."
        ),
        "the goal names only the checks in the change's result"
    );
    assert!(
        words.contains("Name only checks or tests the result names."),
        "the chat names only the checks in the result it hears"
    );
    let reporting =
        words.split("\n\n").find(|paragraph| paragraph.starts_with("Reports:")).expect("shared reporting rules");
    for rule in [
        "CI status cannot prove compilation, tests or execution",
        "even if a producer could not compile or run",
        "Starter echo checks run no code",
        "Use result or forge evidence",
        "else say unknown",
    ] {
        assert!(reporting.contains(rule), "missing reporting instruction: {rule}");
    }
    for role in ["If your brief says you are a goal", "If neither line is in your brief"] {
        let paragraph = words.split("\n\n").find(|paragraph| paragraph.starts_with(role)).expect("role instructions");
        assert!(paragraph.contains("Use Reports."), "{role} must retain reporting rules");
    }
}

#[test]
fn each_charter_offers_every_configured_model() {
    for (document, names) in [
        (single_model(), [b"gpt-6.1-sol".as_slice()].as_slice()),
        (generated(), [b"gpt-6.1-sol".as_slice(), b"gpt-6-luna", b"claude-opus-5-5", b"demo"].as_slice()),
    ] {
        let engine = engine(&document);
        let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
        for charter in &defaults.charters {
            assert_eq!(charter.policy.model.name.as_ref(), names[0]);
            assert_eq!(charter.policy.alternatives.len(), names.len() - 1);
            for (model, name) in charter.policy.alternatives.iter().zip(&names[1..]) {
                assert_eq!(model.name.as_ref(), *name);
            }
        }
    }
}
#[test]
fn each_member_is_refused_at_startup_naming_temper_ctl_settings() {
    let init = generated();
    for member in ["charters", "templates", "policies", "rules"] {
        let original = fixtures::value(&init, b"engine");
        let mut engine = format!("{{\"{member}\":null,").into_bytes();
        engine.extend_from_slice(&original[1..]);
        let document = fixtures::with(&init, b"engine", &engine);
        let problems = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!("engine.{member}: settings live in the store; read them with temper ctl DIR settings show\n")
                .as_bytes()
        );
    }
}

#[test]
fn an_inline_seed_policy_is_refused_naming_its_path() {
    let document = fixtures::with(&generated(), b"engine.projects", fixtures::INLINE_PROJECTS);
    let problems = crate::startup::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(
        crate::startup::lines(&problems).as_ref(),
        b"engine.projects[0].policy: name a policy, policy: 1; change it with temper ctl settings apply\n"
    );
}

#[test]
fn stock_startup_requires_a_model_before_building_its_charters() {
    let mut document = generated();
    for path in [b"engine.llm.providers".as_slice(), b"engine.llm.endpoints", b"engine.llm.models"] {
        document = fixtures::with(&document, path, b"[]");
    }
    let found = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(crate::startup::lines(&found).as_ref(), b"engine.llm.models: past its bound\n");
}

#[test]
fn a_worlds_own_defaults_cannot_restore_document_members() {
    let init = generated();
    let stock = engine(&init);
    let mut own = crate::stock::defaults(&stock.llm, stock.limits.core.tasks.result_bytes);
    own.charters[0].policy.turns = 9;
    let given = crate::stock::Given::Own { jig: own, forge: Box::default() };
    for member in ["charters", "templates", "policies", "rules"] {
        let original = fixtures::value(&init, b"engine");
        let mut engine = format!("{{\"{member}\":null,").into_bytes();
        engine.extend_from_slice(&original[1..]);
        let document = fixtures::with(&init, b"engine", &engine);
        let parsed =
            config::document(&document, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 })
                .unwrap();
        let at = config::read::Object::new(&parsed, 0).unwrap().member(&parsed, b"engine").unwrap();
        let found = crate::startup::engine_with(&parsed, at, &config::Directories::working(), &given).unwrap_err();
        let mut problems = crate::startup::Problems::empty();
        problems.append(found, &config::Path::root().member(b"engine"));
        assert_eq!(problems.problems.len(), 1);
        assert_eq!(
            crate::startup::lines(&problems).as_ref(),
            format!("engine.{member}: settings live in the store; read them with temper ctl DIR settings show\n")
                .as_bytes()
        );
    }
}

#[test]
fn the_reviewers_words_say_to_fail_when_it_cannot_review() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let reviewer = defaults.charters.iter().find(|charter| charter.number == 4).expect("review charter");
    let words = core::str::from_utf8(&reviewer.policy.instructions).expect("review instructions");
    for phrase in [
        "read its pull request, the files it changes and its checks",
        "approve it, or request changes, saying what to change and why",
        "You only read: never edit, push or run anything.",
        "If you cannot read the change, or cannot judge it for any other reason",
        "finish failed and say why: never request changes for that.",
    ] {
        assert!(words.contains(phrase), "missing review instruction: {phrase}");
    }
}

#[test]
fn the_foremans_charter_reads_and_gives_a_verdict() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    assert_eq!(
        defaults.charters.iter().map(|charter| charter.number).collect::<Box<[_]>>().as_ref(),
        [1, 2, 3, 4, 6, 9]
    );
    let foreman = defaults.charters.iter().find(|charter| charter.number == 6).expect("foreman charter");
    let mut expected = defaults.charters[3].clone();
    expected.number = 6;
    expected.policy.instructions = include_bytes!("../stock/foreman.txt").as_slice().into();
    assert_eq!(*foreman, expected);
    assert_eq!(foreman.wake, jig_domain_tasks::WakePolicy::DEFAULT);
    assert_eq!(foreman.policy.instructions.as_ref(), include_bytes!("../stock/foreman.txt"));
    assert!(foreman.policy.inspect);
    assert!(!foreman.policy.modify);
    assert!(!foreman.policy.shell);
    assert!(!foreman.policy.agents);
    for authority in [
        &defaults.rules.ceiling,
        &defaults.policies.get(&1).expect("policy 1").policy.ceiling,
        &defaults.policies.get(&1).expect("policy 1").policy.roles[0].authority,
    ] {
        assert!(authority.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(6)));
    }
}

#[test]
fn the_foremans_words_name_the_three_verdicts() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let foreman = defaults.charters.iter().find(|charter| charter.number == 6).expect("foreman charter");
    let words = core::str::from_utf8(&foreman.policy.instructions).expect("foreman instructions");
    for phrase in ["answer:", "finish:", "escalate:", "never edit, push or run anything"] {
        assert!(words.contains(phrase), "missing foreman instruction: {phrase}");
    }
}

#[test]
fn a_worlds_own_defaults_replace_the_stock_without_document_members() {
    let bytes = fixtures::with(&generated(), b"engine.projects", br#"[{"number":1,"policy":1}]"#);
    let document =
        config::document(&bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    let at = config::read::Object::new(&document, 0).unwrap().member(&document, b"engine").unwrap();
    let stock = crate::startup::engine(&document, at, &config::Directories::working()).unwrap();
    let mut own = crate::stock::defaults(&stock.llm, stock.limits.core.tasks.result_bytes);
    own.charters[0].policy.waiting = skein_lib::Duration::from_millis(1);
    own.templates[0].flags.listed = true;
    let policy = &mut own.policies.get_mut(&1).unwrap().policy;
    policy.period_spend = 41;
    // The project copies this policy; every role allowance fits its period.
    for role in &mut policy.roles {
        role.period_spend = role.period_spend.min(policy.period_spend);
    }
    own.rules.maximum_run_spend = 2;
    let given = crate::stock::Given::Own { jig: own, forge: Box::default() };
    let engine = crate::startup::engine_with(&document, at, &config::Directories::working(), &given).unwrap();
    assert_eq!(stock.charters[0].policy.waiting, skein_lib::Duration::from_secs(10));
    assert_eq!(engine.charters[0].policy.waiting, skein_lib::Duration::from_millis(1));
    assert!(engine.templates[0].flags.listed);
    assert_eq!(engine.policies.iter().find(|policy| policy.number == 1).unwrap().policy.period_spend, 41);
    assert_eq!(engine.projects[0].policy.period_spend, 41);
    assert_eq!(engine.rules.maximum_run_spend, 2);
}

#[test]
fn init_inbox_holds_three_message_credits_within_the_store_codec() {
    let engine = engine(&generated());
    let limits = &engine.limits.core;
    assert_eq!(limits.tasks.result_bytes, 10_660);
    assert_eq!(limits.tasks.message_bytes, 10_660);
    assert_eq!(limits.tasks.inbox_bytes, 82 * limits.tasks.message_bytes);
    // protocol/configuration.md, 13.4: news and answer credits, four live
    // delegates' result reservations, and one unread result coexist.
    let required = 2 * limits.tasks.message_bytes + 5 * limits.tasks.result_bytes;
    assert_eq!(required, 74_620);
    assert!(limits.tasks.inbox_bytes >= required);
    assert!(jig_protocol_store::codec::limits(limits).is_some(), "the three-credit inbox fits the pinned store codec");
}

#[test]
fn stock_budgets_follow_the_default_models_completion_and_run_turn_floor() {
    let mut engine = engine(&generated());
    let init = engine.llm.models[0].clone();
    let mut cheap = init.clone();
    cheap.window = 1501;
    cheap.output = 503;
    cheap.input_price = 1;
    cheap.cached_price = 2;
    cheap.cache_write_price = Some(3);
    cheap.output_price = 4;
    cheap.price_unit = 1000;
    for (row, model, c, k) in [("init", init, 1057, 4228), ("cheap with costliest prices and rounding", cheap, 7, 256)]
    {
        engine.llm.models[0] = model;
        let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
        assert_eq!(stock.rules.minimum_run_spend, 2 * c, "{row}");
        assert_eq!(stock.rules.maximum_run_spend, k, "{row}");
        for (charter, turns, seconds) in
            [(&stock.charters[0], 64, 3600), (&stock.charters[1], 800, 28_800), (&stock.charters[5], 4, 300)]
        {
            assert_eq!(charter.policy.turns, turns, "{row}: charter {}", charter.number);
            assert_eq!(charter.policy.time, skein_lib::Duration::from_secs(seconds), "{row}");
            assert_eq!(charter.policy.reserve_turns, 1, "{row}");
            assert_eq!(charter.policy.reserve_time, skein_lib::Duration::from_secs(60), "{row}");
        }
        for charter in &stock.charters {
            assert_eq!(charter.spend, k, "{row}");
        }
        let policy = &stock.policies.get(&1).expect("policy 1").policy;
        assert_eq!(policy.roles[0].authority.budget.spend, 24 * k, "{row}");
        for spend in [
            stock.rules.ceiling.budget.spend,
            policy.ceiling.budget.spend,
            policy.roles[0].period_spend,
            policy.roles[1].period_spend,
        ] {
            assert_eq!(spend, 80 * 24 * k, "{row}");
        }
        assert_eq!(policy.roles[2].period_spend, 24 * k, "{row}");
        for spend in [stock.rules.period_spend, policy.period_spend] {
            assert_eq!(spend, 81 * 24 * k, "{row}");
        }
    }
}

#[test]
fn reference_coding_run_matches_stock() {
    let engine = engine(&generated());
    let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let coding = &stock.charters.iter().find(|charter| charter.number == 2).expect("coding charter 2").policy;
    let reference = include_str!("../../../../docs/design/draft/reference-code-production.md");
    let configuration = reference.split_once("## 14. Configuration\n").expect("reference configuration").1;
    let row = configuration
        .split("\n## ")
        .next()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("| the coding agent's charter |"))
        .expect("reference coding-charter row");
    let run = format!(
        "retain the stock run: {} turns and {} s, with a reserve of {} turn and {} s",
        coding.turns,
        coding.time.as_nanos() / 1_000_000_000,
        coding.reserve_turns,
        coding.reserve_time.as_nanos() / 1_000_000_000,
    );
    assert!(row.replace(',', "").contains(&run.replace(',', "")), "coding charter must {run}; documented row: {row}");
}

#[test]
fn the_stock_funds_eighty_chats_in_one_period() {
    let mut engine = engine(&generated());
    let init = engine.llm.models[0].clone();
    let mut cheap = init.clone();
    cheap.window = 1501;
    cheap.output = 503;
    cheap.input_price = 1;
    cheap.cached_price = 2;
    cheap.cache_write_price = Some(3);
    cheap.output_price = 4;
    cheap.price_unit = 1000;
    for (row, model) in [("init", init), ("cheap with costliest prices and rounding", cheap)] {
        engine.llm.models[0] = model;
        let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
        let policy = &stock.policies.get(&1).expect("policy 1").policy;
        let chat = policy.roles[0].authority.budget.spend;
        let pool = policy.roles[0].period_spend;
        assert!(80 * chat <= pool, "{row}: eighty open chats fit the pool");
        assert!(81 * chat > pool, "{row}: an eighty-first open chat exceeds the pool");
    }
}

#[test]
fn startup_refuses_stock_budget_overflow_before_constructing_defaults() {
    let document = fixtures::with(&single_model(), b"engine.llm.models[0].prices.input", b"18446744073709551615");
    let document = fixtures::with(&document, b"engine.llm.models[0].prices.unit", b"1");
    let found = crate::named(&document, &config::Directories::working()).unwrap_err();
    assert_eq!(crate::startup::lines(&found).as_ref(), b"engine.llm.models[0].prices: past its bound\n");
}

#[test]
fn the_reviewer_keeps_the_coding_run_with_read_only_workspace_families() {
    let engine = engine(&generated());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let mut expected = defaults.charters[1].clone();
    expected.number = 4;
    expected.policy.instructions = include_bytes!("../stock/review.txt").as_slice().into();
    expected.policy.modify = false;
    expected.policy.agents = false;
    expected.policy.tools = jig_domain_tasks::Tools(512);
    assert_eq!(defaults.charters[3], expected);
    for authority in [
        &defaults.rules.ceiling,
        &defaults.policies.get(&1).expect("policy 1").policy.ceiling,
        &defaults.policies.get(&1).expect("policy 1").policy.roles[0].authority,
    ] {
        assert!(authority.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(4)));
    }
}

#[test]
fn stock_and_init_admit_the_landing_procedure_and_its_effects() {
    let document = fixtures::with(&generated(), b"engine.projects", br#"[{"number":1,"policy":1}]"#);
    let engine_bytes = fixtures::value(&document, b"engine");
    let engine = fixtures::parse_engine(&engine_bytes);
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let core = config::core(&engine, [1; 16], 7);
    for ceiling in [
        &defaults.rules.ceiling,
        &defaults.policies.get(&1).unwrap().policy.ceiling,
        &core.seed_projects[0].policy.policy.ceiling,
    ] {
        use jig_domain_authority::Executor;
        assert_eq!(
            ceiling.delegation.kinds.as_ref(),
            [
                Executor::Charter(1),
                Executor::Charter(2),
                Executor::Charter(3),
                Executor::Charter(4),
                Executor::Charter(6),
                Executor::Charter(9),
                Executor::Procedure(2),
                Executor::Procedure(3),
                Executor::Procedure(4)
            ]
        );
        assert_eq!(ceiling.grants.len(), 14);
        for (grant, kind) in ceiling.grants.iter().zip([1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 16, 17]) {
            assert_eq!(grant.kind, kind);
        }
    }
}

#[path = "stock_brief.rs"]
mod brief;

#[test]
fn the_chats_words_read_a_changes_code_before_briefing_it() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("When they ask for a change to a repository, first read the code it touches with the forge's reads (read_branch for its head, then read_file), then call change with their request and only what you read: never describe code you have not read."));
    assert!(words.contains("When they ask you to fix something and what they call broken is not there, change nothing and tell them so in words, saying what the code has."));
}

#[test]
fn the_chats_words_report_only_the_landed_results_or_pull_requests_change() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("When a change lands, tell them so with the pull request and merge commit its Landed line names, and what it changed only as its result or its pull request says, never by restating their request as done."));
}

#[test]
fn the_chats_words_route_repository_edits_through_change_and_wait_for_landing() {
    let words = include_str!("../stock/chat.txt");
    assert!(
        words.contains(
            "Repository changes must go through change; never implement them by writing your chat workspace."
        )
    );
    assert!(words.contains("After calling change, say only that work started until the delegate reports."));
    assert!(words.contains("Name only checks or tests the result names."));
}

#[test]
fn the_coders_words_put_its_report_in_finish() {
    let words = include_str!("../stock/coding.txt");
    assert!(words.contains("Summarize the change in finish; message your change only in the question form."));
}

#[test]
fn the_coders_words_wait_after_asking() {
    let engine = engine(&single_model());
    let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let coder = stock.charters.iter().find(|charter| charter.number == 2).expect("coding charter 2");
    let words = core::str::from_utf8(&coder.policy.instructions).expect("coding instructions");
    assert!(words.contains("After asking, call wait, not finish: the answer reaches this run."));
}

#[test]
fn the_coders_words_refuse_an_absent_fix_instead_of_adding_a_feature() {
    let words = include_str!("../stock/coding.txt");
    assert!(words.contains("Change only what was asked. If a requested fix is absent from the code, write nothing and finish with no_change, explaining what the code has; never add a feature in its place."));
}

#[test]
fn the_coders_words_build_additions_in_the_code_there_is() {
    let words = include_str!("../stock/coding.txt");
    assert!(words.contains("If the description differs from the code, build the requested addition using the code's names; use no_change only if it cannot be built there."));
}

#[test]
fn the_coders_words_preserve_both_sides_in_repairs_and_resolutions() {
    let words = include_str!("../stock/coding.txt");
    assert!(words.contains("In repairs and resolutions, preserve the requested change and unrelated base content, including independent appended lines and existing blank lines."));
    assert!(words.contains("Restore losses named in review; never remove content to pass checks."));
    assert!(words.contains("Initial production has no pull request yet:"));
}

#[test]
fn kinds_are_declared_with_their_bounds() {
    const CONNECTOR: u16 = 5;
    use jig_domain_core::settings::{Kind, KindScope};
    use temper_engine_domain_forge::{self as forge, settings};
    let engine = fixtures::engine();
    let mut limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    // Distinct counts exercise every bound, including the smaller name bound.
    limits.authority.segments = 3;
    limits.authority.segment_bytes = 13;
    limits.forge.name_bytes = 9;
    limits.forge.change_policy.gates = 2;
    limits.forge.roles = 3;
    limits.forge.landing_rules = 7;
    let rule = settings::Value::Rule(settings::LandingRule {
        pattern: forge::GatePattern {
            segments: Box::new([
                b"forge".as_slice().into(),
                b"123456789".as_slice().into(),
                b"abcdefghi".as_slice().into(),
            ]),
            last: forge::GateLast::Open(b"123456789".as_slice().into()),
        },
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: Box::new([
            settings::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Exact },
            settings::RuleGate { number: 2, blocking: false, freshness: forge::JudgeFreshness::Clean },
        ]),
        approvals: Box::new([
            settings::RuleApproval { role: 0, people: 1, freshness: forge::JudgeFreshness::Exact },
            settings::RuleApproval { role: 1, people: 2, freshness: forge::JudgeFreshness::Clean },
            settings::RuleApproval { role: 2, people: 3, freshness: forge::JudgeFreshness::Exact },
        ]),
    });
    let rule_bytes = temper_engine_smith::kinds::size(&rule);
    assert_eq!(
        crate::stock::kinds(&limits, CONNECTOR).as_ref(),
        &[
            Kind {
                number: 1,
                scope: KindScope::Deployment,
                connector: CONNECTOR,
                value_bytes: rule_bytes,
                per_scope: 7,
                charters: 0
            },
            Kind {
                number: 2,
                scope: KindScope::Project,
                connector: CONNECTOR,
                value_bytes: rule_bytes,
                per_scope: 7,
                charters: 0
            },
            Kind {
                number: 3,
                scope: KindScope::Deployment,
                connector: CONNECTOR,
                value_bytes: temper_engine_smith::kinds::size(&settings::Value::Charter(2)),
                per_scope: 2,
                charters: 1
            },
            Kind {
                number: 6,
                scope: KindScope::Project,
                connector: CONNECTOR,
                value_bytes: 1,
                per_scope: 1,
                charters: 0
            },
            Kind {
                number: 7,
                scope: KindScope::Project,
                connector: CONNECTOR,
                value_bytes: 8,
                per_scope: 1,
                charters: 0
            },
        ]
    );
    limits.authority.segment_bytes = 6;
    assert_eq!(crate::stock::kinds(&limits, CONNECTOR)[0].value_bytes, rule_bytes - 3 * (9 - 6));
}

#[test]
fn the_binary_gives_the_forges_stock_at_every_start() {
    // configuration.md, 13.4: one deployment rule and both forge charters.
    const ITEMS: usize = 3;
    use temper_engine_domain_forge::{self as forge, settings};
    let engine = fixtures::engine();
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let stock = crate::stock::forge(&limits);
    assert_eq!(stock.len(), ITEMS);
    assert_eq!((stock[0].kind, stock[0].scope, stock[0].number), (1, 0, 1));
    assert_eq!(
        stock[0].value.value,
        settings::Value::Rule(settings::LandingRule {
            pattern: forge::GatePattern {
                segments: Box::new([b"forge".as_slice().into()]),
                last: forge::GateLast::Open(Box::default()),
            },
            enforce: true,
            ci: true,
            up_to_date: true,
            gates: Box::new([settings::RuleGate {
                number: 1,
                blocking: true,
                freshness: forge::JudgeFreshness::Clean
            }]),
            approvals: Box::default(),
        })
    );
    for (item, number, charter) in [(&stock[1], 1, 2), (&stock[2], 2, 4)] {
        assert_eq!((item.kind, item.scope, item.number), (3, 0, number));
        assert_eq!(item.value.value, settings::Value::Charter(charter));
    }
    for item in &stock {
        assert_eq!(item.value.size, temper_engine_smith::kinds::size(&item.value.value));
    }
}

#[test]
fn the_stock_pool_funds_eighty_chats_at_once() {
    let engine = engine(&generated());
    let stock = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    assert_eq!(stock.rules.minimum_run_spend, 2 * 1057);
    assert_eq!(stock.rules.maximum_run_spend, 4228);
    let policy = &stock.policies.get(&1).expect("policy 1").policy;
    assert_eq!(stock.templates[0].budget, jig_domain_people::TemplateBudget::AsAllowed { cap: None });
    let chat_spend = policy.roles[0].authority.budget.spend;
    assert_eq!(chat_spend, 24 * 4228);
    let pool = policy.roles[0].period_spend;
    assert!(pool >= 80 * chat_spend);
    assert!(pool < 81 * chat_spend);
}

#[test]
fn the_stock_reports_follow_the_deployment_result_bound_and_keep_its_floor() {
    let mut engine = engine(&generated());
    for result_bytes in [32, 10_660, 262_144] {
        engine.limits.core.tasks.result_bytes = result_bytes;
        let stock = crate::stock::defaults(&engine.llm, result_bytes);
        for template in &stock.templates {
            assert_eq!(template.contract, jig_domain_tasks::Contract::Report { words: result_bytes });
        }
        assert_eq!(crate::stock::floors(&stock, &engine.limits), Ok(()));
        engine.limits.core.tasks.result_bytes -= 1;
        assert_eq!(
            crate::stock::floors(&stock, &engine.limits),
            Err(crate::stock::Floor {
                limit: b"tasks.result_bytes",
                configured: u64::from(result_bytes - 1),
                needed: u64::from(result_bytes),
                setting: jig_domain_core::settings::Setting::Template(1),
            })
        );
    }
}

#[test]
fn chat_txt_tells_a_chat_to_keep_one_document() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("the chat and document paragraphs are for chats alone"));
    assert!(words.contains("the following document rule is also yours"));
    let (chat, document) = words.split_once("\n\nWhen the person asks for a draft").expect("document follows chat");
    assert!(chat.contains("If neither line is in your brief"));
    assert!(document.starts_with(" or says \"write this up\", keep one document with note:"));
    assert_document_rule(document);
}

#[test]
fn configuring_txt_tells_it_how_project_works() {
    let words = include_str!("../stock/configuring.txt");
    assert!(words.contains("Once you understand the person's way of working"));
    assert!(words.contains("one document titled \"How PROJECT works\", current as the conversation changes it"));
    assert!(words.contains("Use note:"));
    assert!(words.contains("Change nothing yourself."));
    assert_document_rule(words);
}

fn assert_document_rule(words: &str) {
    for phrase in [
        "scope goal, project P and goal T from your brief's [This task: T, in project P]",
        "description its title",
        "Write its first version whole, in Markdown: # TITLE",
        "muted line _Draft, DATE, with the user._",
        "one ## section per topic",
        "open questions under ## Open questions",
        "Then change it by lines",
        "revision you read from your brief, recall or your last write's answer",
        "To append, first is the note's lines + 1 and remove 0",
        "Never rewrite it whole past a few KB",
        "build a long document in parts",
        "A refused note says the revision or the line count to use",
        "Use join true to continue the line before first without a newline when a line exceeds one call",
        "Say in one sentence what changed; do not repeat the document in your reply",
        "Mermaid is not drawn, so do not draw diagrams",
    ] {
        assert!(words.contains(phrase), "missing document instruction: {phrase}");
    }
}

#[test]
fn the_status_charter_is_small_and_reaches_nothing() {
    let engine = engine(&generated());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let status = defaults.charters.iter().find(|charter| charter.number == 9).expect("status charter");
    let mut expected = defaults.charters[0].clone();
    expected.number = 9;
    expected.policy.instructions = include_bytes!("../stock/status.txt").as_slice().into();
    expected.policy.turns = 4;
    expected.policy.time = skein_lib::Duration::from_secs(300);
    expected.policy.inspect = false;
    expected.policy.agents = false;
    expected.policy.tools = jig_domain_tasks::Tools(0);
    assert_eq!(*status, expected);
    assert_eq!(status.policy.reserve_turns, 1);
    assert_eq!(status.policy.reserve_time, skein_lib::Duration::from_secs(60));
    assert!(!status.policy.inspect);
    assert!(!status.policy.modify);
    assert!(!status.policy.shell);
    assert!(!status.policy.agents);
    assert_eq!(status.policy.tools, jig_domain_tasks::Tools(0));
}

#[test]
fn the_status_charters_words_are_the_requests() {
    let words = include_str!("../stock/status.txt");
    assert_eq!(
        words,
        "You write the few lines at the top of a project's home page, from the facts in your task. Write one sentence on what landed, starting with the count. Then a blank line, then at most three plain sentences on what stands out: what finished, what is stuck and why. Say only what the facts say. No greetings, no lists, no advice."
    );
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let status = defaults.charters.iter().find(|charter| charter.number == 9).expect("status charter");
    assert_eq!(status.policy.instructions.as_ref(), words.as_bytes());
}

#[test]
fn the_rules_delegate_to_the_status_charter() {
    let engine = engine(&single_model());
    let defaults = crate::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let policy = &defaults.policies.get(&1).expect("policy 1").policy;
    for authority in [&defaults.rules.ceiling, &policy.ceiling]
        .into_iter()
        .chain(policy.roles.iter().filter(|role| role.authority.budget.spend != 0).map(|role| &role.authority))
    {
        assert!(authority.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(9)));
    }
}
