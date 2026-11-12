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
            sign_in: false,
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
    let bytes = generated();
    let document = fixtures::Json::new(&bytes);
    let providers = [b"[".as_slice(), document.value(b"engine.llm.providers[0]"), b"]"].concat();
    let endpoints = [b"[".as_slice(), document.value(b"engine.llm.endpoints[0]"), b"]"].concat();
    document.with(&[
        (b"engine.llm.providers", &providers),
        (b"engine.llm.endpoints", &endpoints),
        (b"application.demo", b"null"),
    ])
}
pub(super) fn engine(document: &[u8]) -> config::Engine {
    fixtures::parse_engine(&fixtures::value(document, b"engine"))
}
fn single_engine() -> config::Engine {
    let mut engine = engine(&single_model());
    let sol = engine.llm.models.iter().find(|entry| entry.model.name.as_ref() == b"gpt-6.1-sol").unwrap().clone();
    let defaults = crate::stock::checked_with(Box::new([sol]), engine.limits.core.tasks.result_bytes).unwrap();
    config::pass(&mut engine, defaults).unwrap();
    engine
}
fn without_projects() -> Box<[u8]> {
    let document = generated();
    fixtures::Json::new(&document).with(&[(b"engine.projects", b"[]"), (b"application.demo", b"null")])
}

#[test]
fn stock_chats_can_delegate_message_and_read_the_forge() {
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
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
                    | families.ask.0
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
    let engine = single_engine();
    let stock = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let policy = &stock.policies.get(&1).expect("stock policy 1").policy;
    for authority in [&stock.rules.ceiling, &policy.ceiling]
        .into_iter()
        .chain(policy.roles.iter().filter(|role| role.authority.budget.spend != 0).map(|role| &role.authority))
    {
        assert_eq!(authority.notes, Scopes::GOAL);
        assert_eq!(authority.tools, Tools(3975));
        assert!(authority.note_resources.is_empty());
    }
    let observer = &policy.roles.iter().find(|role| role.number == 3).expect("observer").authority;
    assert_eq!(observer.notes, Scopes(0));
    assert!(observer.note_resources.is_empty());
}

// temper/docs/design/protocol/configuration/reference.md: reference settings widen stock authority.
#[test]
fn configure_reference_authority_allows_coordinator_tools() {
    use jig_domain_authority::{Answer, Call, NoteScope, Scopes};
    let engine = engine(&generated());
    let stock = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
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
    for role in &mut configured.roles {
        role.authority.delegation.kinds = role
            .authority
            .delegation
            .kinds
            .iter()
            .copied()
            .filter(|kind| {
                *kind != jig_domain_authority::Executor::Charter(12)
                    && *kind != jig_domain_authority::Executor::Charter(13)
            })
            .collect();
    }
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
    assert_eq!(new.tools, Tools(4031));
    assert_eq!(new.notes, Scopes(3));
    let mut restored = new.clone();
    restored.tools = old.tools;
    restored.notes = old.notes;
    restored.delegation.kinds = old.delegation.kinds.clone();
    assert_eq!(&restored, old, "budgets, grants and other authority fields survive");
    for executor in [
        Executor::Charter(5),
        Executor::Charter(7),
        Executor::Charter(8),
        Executor::Procedure(6),
        Executor::Job(4),
        Executor::Role(0),
    ] {
        assert!(new.delegation.kinds.contains(&executor));
    }
}

fn reference_requirements() -> settings_form::view::Authority {
    use settings_form::view::{SettingEdit, SettingValue};
    let readme = include_str!("../../../../docs/examples/reference/README.md");
    let json = readme
        .split_once("cat > reference-authority.json <<'JSON'\n")
        .expect("the reference's README documents its authority")
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
    kinds.retain(|kind| *kind != Executor::Charter(12) && *kind != Executor::Charter(13));
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

#[path = "stock_set.rs"]
mod set;

#[test]
fn the_coordinator_prioritizes_relayed_questions() {
    let set = crate::stock::charter_set();
    assert!(set.goal.unwrap().contains("read your report note and the task tree first"));
    assert!(set.coordinator.unwrap().contains("answer before you go on with anything else"));
    assert!(set.coordinator.unwrap().contains("in the answer form naming its question"));
}

#[test]
fn the_coordinators_charter_makes_no_landing_close_out_or_catch_up() {
    let set = crate::stock::charter_set();
    assert!(set.common.unwrap().contains("No agent merges or lands."));
    let goal = set.goal.unwrap();
    assert!(goal.contains("Make no catch-up of g: the closer takes main as it lands."));
    assert!(goal.contains("A goal never finishes itself: its closer lands g and ends it."));
    assert!(goal.contains("[in flight: change N]"));
    assert!(goal.contains("never to main's"));
    assert!(goal.contains("Then `subscribe` to it"));
    assert!(set.designer.unwrap().contains("Your changes' questions and failures reach you as you work"));
    let coordinator = set.coordinator.unwrap();
    assert!(coordinator.contains("**A failure:** read its report."));
    assert!(coordinator.contains("**Your closer's escalation**"));
    assert!(coordinator.contains("Never write g or land it yourself."));
    assert!(coordinator.contains("pass it up to your person with `decide_escalation`"));
}

#[test]
fn the_coordinators_wake_policy_wakes_on_a_failure_or_the_last_result() {
    let engine = engine(&generated());
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let charter = defaults.charters.iter().find(|charter| charter.number == 5).expect("stock coordinator");
    assert_eq!(charter.wake.results, jig_domain_tasks::ResultsWake::LastOrFailure);
}

#[test]
fn stock_chats_inspect_and_delegate_while_coders_modify() {
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    for charter in &defaults.charters {
        assert_eq!(
            charter.policy.inspect,
            charter.number != 9
                && charter.number != 10
                && charter.number != 11
                && charter.number != 12
                && charter.number != 13
        );
        assert!(!charter.policy.shell);
        assert_eq!(charter.policy.modify, charter.number == 2 || charter.number == 5);
        assert_eq!(
            charter.policy.agents,
            charter.number == 1 || charter.number == 2 || charter.number == 5 || charter.number == 12
        );
    }
}

#[test]
fn ask_temper_is_charter_11_and_template_5() {
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let charter = defaults.charters.iter().find(|charter| charter.number == 11).expect("Ask temper charter");
    let mut expected = defaults.charters.iter().find(|charter| charter.number == 1).unwrap().clone();
    expected.number = 11;
    expected.policy.instructions = include_bytes!("../stock/ask.txt").as_slice().into();
    expected.policy.inspect = false;
    expected.policy.modify = false;
    expected.policy.agents = false;
    expected.policy.tools = jig_domain_tasks::Tools(3072);
    assert_eq!(*charter, expected);
    let words = core::str::from_utf8(&charter.policy.instructions).unwrap();
    assert!(words.contains(temper_engine_smith::ADDRESSES));
    assert!(
        words.contains("project-classes: what agents decide for a person and what asks them, by kind and by path set.")
    );
    ask_design_first_rules(words);
    assert!(words.len() <= 10_240, "Ask temper's instructions fit draft/ask-temper.md, 4.1's 10 KB");
    for heading in [
        "## temper, as you explain it",
        "## Finding out",
        "## Answering",
        "## Configuring",
        "## Showing",
        "## Remembering",
        "## Your voice",
    ] {
        assert!(words.contains(heading), "missing {heading}");
    }
    let (_, configuring) = words.split_once("## Configuring\n\n").expect("Configuring part");
    let (configuring, _) = configuring.split_once("## Showing").expect("Showing follows Configuring");
    assert!(configuring.contains("With it, write in their terms what the pages show: charter-words for each charter, what it does; project-words for the project's own way, a headline and a short lede."));
    let (_, remembering) = words.split_once("## Remembering\n\n").expect("Remembering part");
    let (remembering, _) = remembering.split_once("## Your voice").expect("Your voice follows Remembering");
    assert_eq!(
        remembering,
        r#"remember keeps a few lines about this person across their chats; they see and edit them. Keep only what they ask you to keep ("remember that…"), and say in your reply that you kept it. Never keep what a read answers (settings, their history, proposals and their reasons are records already), a guess about them, or a secret. Others in this project may read this chat, so name a kept line only when the person asks about it. Where a kept line and a read disagree, the read is right. Nothing kept widens what you may do. When the list is full, ask them which line to change or drop.

"#
    );
    let (_, showing) = words.split_once("## Showing\n\n").expect("Showing part");
    let (showing, _) = showing.split_once("## Remembering").expect("Remembering follows Showing");
    assert_eq!(
        showing,
        r"Show a picture when it explains better than words. Draw what exists from a view, a fenced block naming it:
```view
path OWNER/NAME main @revision N
```
Views: path OWNER/NAME [BRANCH] · way NAME · charter N · proposal tN/K; a path or a charter takes @revision N, the newest when left out. Never draw by hand what a view shows.
A way is drawn, never described: way NAME draws one of temper's stock ways over this project, as see ways lists them. A person who wants a way of their own gets a proposal, which draws its path before and after.

"
    );
    for excluded in ["queue", "task N tree", "task N timeline", "```mermaid"] {
        assert!(!showing.contains(excluded), "Showing excludes {excluded} until M3b");
    }
    let mut expected = defaults.templates.iter().find(|template| template.number == 1).unwrap().clone();
    expected.number = 5;
    expected.name = b"ask".as_slice().into();
    expected.executor = jig_domain_people::TemplateExecutor::Charter(11);
    assert_eq!(*defaults.templates.iter().find(|template| template.number == 5).unwrap(), expected);
}

#[test]
fn the_drafter_is_charter_12_and_template_6() {
    use jig_domain_authority::Executor;
    use jig_domain_core::ToolFamilies;
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let charter = defaults.charters.iter().find(|charter| charter.number == 12).expect("drafter charter");
    assert_eq!(charter.policy.instructions.len(), 2_874);
    assert_eq!(
        skein_lib::sha256::digest(&charter.policy.instructions),
        [
            0x1f, 0x44, 0x77, 0x43, 0xf6, 0x7f, 0x32, 0x1b, 0xac, 0x95, 0x16, 0xe4, 0x72, 0x17, 0x3e, 0x9f, 0x88, 0xe0,
            0x5b, 0x40, 0x67, 0xc4, 0xe1, 0xc7, 0xbe, 0xc3, 0xd1, 0x45, 0x7c, 0x5e, 0xd9, 0x00,
        ]
    );
    assert_eq!(charter.policy.tools.0 & ToolFamilies::JIG.delegate.0, 0);
    assert_eq!(charter.policy.tools.0 & ToolFamilies::JIG.settings, 0);
    assert_eq!(charter.policy.tools.0 & ToolFamilies::JIG.ask.0, 0);
    let mut expected = defaults.charters.iter().find(|charter| charter.number == 1).unwrap().clone();
    expected.number = 12;
    expected.policy.instructions = include_bytes!("../stock/drafter.txt").as_slice().into();
    expected.policy.inspect = false;
    expected.policy.modify = false;
    expected.policy.shell = false;
    expected.policy.agents = true;
    expected.policy.tools = jig_domain_tasks::Tools(1022);
    assert_eq!(*charter, expected);
    let mut expected = defaults.templates.iter().find(|template| template.number == 1).unwrap().clone();
    expected.number = 6;
    expected.name = b"design session".as_slice().into();
    expected.executor = jig_domain_people::TemplateExecutor::Charter(12);
    assert_eq!(*defaults.templates.iter().find(|template| template.number == 6).unwrap(), expected);
    let policy = &defaults.policies.get(&1).unwrap().policy;
    for authority in [&defaults.rules.ceiling, &policy.ceiling] {
        assert!(authority.delegation.kinds.contains(&Executor::Charter(12)));
    }
    for role in &policy.roles {
        if role.number == 3 {
            assert!(role.authority.delegation.kinds.is_empty());
        } else {
            assert!(role.authority.delegation.kinds.contains(&Executor::Charter(12)));
        }
    }
}

#[test]
fn the_standing_role_resumes_in_parts_and_keeps_its_period_budget() {
    use jig_domain_people::{TemplateBudget, TemplateDeadline, TemplateExecutor};
    use jig_domain_tasks::{EndedBy, Flags, Reasons, ResultsWake, Retention};
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let charter = defaults.charters.iter().find(|charter| charter.number == 13).expect("standing role charter");
    let text = include_str!("../stock/role.md");
    let mut sections = text.split("## ");
    assert_eq!(sections.next(), Some(""));
    let sections: Box<[_]> = sections.map(|section| section.split_once('\n').expect("heading line")).collect();
    assert_eq!(sections.len(), 3);
    assert_eq!(sections[0].0, "shared");
    assert_eq!(sections[1].0, "part, on: first");
    assert_eq!(charter.policy.instructions.as_ref(), sections[0].1.trim_matches('\n').as_bytes());
    assert_eq!(charter.parts.len(), 2);
    for ((part, reasons), (_, words)) in
        charter.parts.iter().zip([Reasons::FIRST, Reasons(Reasons::ALL.0 & !Reasons::FIRST.0)]).zip(&sections[1..])
    {
        assert_eq!(part.on, reasons);
        assert_eq!(part.instructions.as_ref(), words.trim_matches('\n').as_bytes());
    }
    assert!(charter.policy.resume);
    assert_eq!(charter.policy.waiting, skein_lib::Duration::ZERO);
    assert_eq!(charter.policy.tools.0, 2047);
    assert!(!charter.policy.inspect && !charter.policy.modify && !charter.policy.shell && !charter.policy.agents);
    assert!(charter.documents.is_empty());
    let coordinator = defaults.charters.iter().find(|charter| charter.number == 5).unwrap();
    assert_eq!(charter.wake, jig_domain_tasks::WakePolicy { results: ResultsWake::Each, ..coordinator.wake });
    assert_eq!((charter.policy.turns, charter.policy.time), (coordinator.policy.turns, coordinator.policy.time));
    assert_eq!((charter.policy.reserve_turns, charter.policy.reserve_time), (1, skein_lib::Duration::from_secs(60)));
    let template = defaults.templates.iter().find(|template| template.number == 7).expect("standing role template");
    assert_eq!(template.name.as_ref(), b"standing role");
    assert_eq!(template.executor, TemplateExecutor::Chosen);
    assert_eq!(template.budget, TemplateBudget::Given);
    assert_eq!(template.deadline, TemplateDeadline::None);
    assert_eq!(template.tracked, None);
    assert_eq!(template.label, 3);
    assert_eq!(
        template.flags,
        Flags { ended_by: EndedBy::Standing, conversation: false, listed: true, retention: Retention::Kept }
    );
    assert_eq!(template.contract, defaults.templates[0].contract);
    assert!(template.llm.is_none());
    assert!(defaults.rules.ceiling.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(13)));
}

#[test]
fn stock_models_keep_a_five_minute_cache_except_the_scripted_one() {
    let models = crate::stock::models(&engine(&generated()).llm);
    assert_eq!(models.len(), 8);
    for model in models {
        assert_eq!(model.model.cache_life_ms, if model.model.name.as_ref() == b"demo" { 0 } else { 300_000 });
    }
}

fn ask_design_first_rules(words: &str) {
    assert!(words.contains("confirm all required design landed into g before dispatching coding"));
    assert!(words.contains("A push, delivery or designer report is insufficient"));
    assert!(words.contains("Use after only for live prerequisites"));
    assert!(words.contains("cite design's repository path and section in words; omit its ended task from after"));
}

#[test]
fn each_charters_run_bytes_floor_names_that_charter() {
    use jig_domain_core::settings::Setting;
    let engine = engine(&without_projects());
    for charter in &crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes).charters
    {
        let mut defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
        // Isolate each run policy from larger stock runs.
        defaults.charters = Box::new([charter.clone()]);
        let resolved =
            jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core).unwrap();
        let needed = jig_domain_core::charter_owned_bytes(&resolved).unwrap();
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
fn the_chats_opening_sends_plain_edits_directly_to_change() {
    let words = include_str!("../stock/chat.txt");
    let opening = words.split("If your brief says you are an advisor").next().expect("opening");
    assert!(
        opening.contains(
            "Repository changes must go through change; never implement them by writing your chat workspace."
        )
    );
    assert!(opening.contains("then call change with their request and only what you read"));
    assert!(opening.contains("call read_branch with its repository before you say it isn't there"));
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
        "If your brief says [Requested by person N], your requester is that person, who has no task to message: wherever you would ask them with message, call delegate instead with one task: executor {kind: person, person: N}, N from that line and no one else; spec words your question; contract {kind: report, words: 4096}; authority {tools: 0, grants: [], delegation: {kinds: [], tasks: 0, depth: 0}, budget: {spend: 0}, notes: 0}. Then call wait: its result is their answer.",
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

// temper/docs/design/protocol/configuration/stock.md; domain/agent/tools.md.
#[test]
fn a_goal_is_told_to_call_decided_before_it_asks() {
    for (words, prefixes) in
        [(include_str!("../stock/chat.txt"), &["If your brief says you are a goal", "If neither line"][..])]
    {
        for prefix in prefixes {
            let paragraph =
                words.split("\n\n").find(|paragraph| paragraph.starts_with(prefix)).expect("asking paragraph");
            assert!(paragraph.contains("Before asking the person anything, call decided with the question, its options, the one you'd take, why, its kind, the paths the choice would change, and under where an item of the project's classes with words covers it (see settings project-classes); on Decided carry on and ask no one; on Ask, ask as the answer says. Use money for funds and budgets, irreversible for rewriting history, deleting work that hasn't landed or a store's data, rules for a change to the project's classes, a landing rule or a standing role's rules, and other for what fits none of the ten; these always wait too. Sign-ins, machines and a plan's go-ahead always wait."));
        }
    }
    let goal = crate::stock::charter_set().goal.unwrap();
    assert!(goal.contains("`decide_escalation`, which passes an escalation up"));
    for words in [
        "Before you ask anything, call `decided`",
        "the question, its options, the one you'd take, why, its kind, the paths",
        "see settings project-classes",
        "On Decided, carry on with it and ask no one; on Ask, ask as the answer says.",
        "If you're told a decision was overturned, ask it with `message`",
    ] {
        assert!(goal.contains(words), "{words}");
    }

    assert!(
        include_str!("../stock/chat.txt")
            .contains("If they tell you a decision was overturned, ask it with message, as a question, and wait.")
    );
}

#[test]
fn asks_always_is_told_every_answer_is_ask() {
    for (words, prefixes) in
        [(include_str!("../stock/chat.txt"), &["If your brief says you are a goal", "If neither line"][..])]
    {
        for prefix in prefixes {
            let paragraph =
                words.split("\n\n").find(|paragraph| paragraph.starts_with(prefix)).expect("asking paragraph");
            assert!(
                paragraph.contains("If your words hold the line Asks: always, every answer is Ask."),
                "Asks: always calls decided and records why it asks"
            );
        }
    }
    assert!(
        crate::stock::charter_set()
            .goal
            .unwrap()
            .contains("If your words hold the line `Asks: always`, every answer is Ask.")
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
    // domain/agent/agents.md, 2: a goal writes its question in the client's line form.
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
fn every_stock_charter_names_its_first_choices_and_the_rest() {
    let mut unnamed = engine(&generated());
    unnamed.llm.models = unnamed
        .llm
        .models
        .into_iter()
        .filter(|model| {
            model.model.name.as_ref() != b"claude-opus-5-5"
                && model.model.name.as_ref() != b"gpt-6.1-sol"
                && model.model.name.as_ref() != b"gpt-6-astra"
        })
        .collect();
    unnamed.llm.models.reverse();
    for (llm, ordinary, asking) in [
        (single_engine().llm, vec!["gpt-6.1-sol"], vec!["gpt-6.1-sol"]),
        (
            engine(&generated()).llm,
            vec![
                "claude-opus-5-5",
                "gpt-6.1-sol",
                "claude-sonnet-5-5",
                "claude-fable-5-1",
                "gpt-6-astra",
                "gpt-6.1-terra",
                "gpt-6-luna",
                "demo",
            ],
            vec![
                "claude-opus-5-5",
                "gpt-6-astra",
                "gpt-6.1-sol",
                "claude-sonnet-5-5",
                "claude-fable-5-1",
                "gpt-6.1-terra",
                "gpt-6-luna",
                "demo",
            ],
        ),
        (
            engine(include_bytes!("../../../temper-init/template/bare.json")).llm,
            vec![
                "claude-opus-5-5",
                "gpt-6.1-sol",
                "claude-sonnet-5-5",
                "claude-fable-5-1",
                "gpt-6-astra",
                "gpt-6.1-terra",
                "gpt-6-luna",
            ],
            vec![
                "claude-opus-5-5",
                "gpt-6-astra",
                "gpt-6.1-sol",
                "claude-sonnet-5-5",
                "claude-fable-5-1",
                "gpt-6.1-terra",
                "gpt-6-luna",
            ],
        ),
        (
            unnamed.llm,
            vec!["claude-sonnet-5-5", "claude-fable-5-1", "gpt-6.1-terra", "gpt-6-luna", "demo"],
            vec!["claude-sonnet-5-5", "claude-fable-5-1", "gpt-6.1-terra", "gpt-6-luna", "demo"],
        ),
    ] {
        let defaults = crate::stock::checked_with(llm.models.clone(), 10_660).expect("representable stock");
        for charter in &defaults.charters {
            let names = if charter.number == 11 { &asking } else { &ordinary };
            assert!(charter.rest);
            let first_choices: &[&[u8]] = if charter.number == 11 {
                &[b"claude-opus-5-5", b"gpt-6-astra", b"gpt-6.1-sol"]
            } else {
                &[b"claude-opus-5-5", b"gpt-6.1-sol"]
            };
            let choices: Box<[_]> =
                core::iter::once(&charter.policy.model).chain(&charter.policy.alternatives).collect();
            if choices.iter().any(|model| first_choices.contains(&model.name.as_ref())) {
                assert!(choices.iter().all(|model| first_choices.contains(&model.name.as_ref())));
            } else {
                assert_eq!(choices.len(), 1);
                assert_eq!(choices[0], &defaults.models.iter().min_by_key(|entry| entry.number).unwrap().model);
            }
            let charter = jig_domain_core::settings::charters::resolved(
                charter,
                &defaults.models,
                &jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core,
            )
            .unwrap();
            let offered: Box<[_]> = core::iter::once(&charter.policy.model)
                .chain(charter.policy.alternatives.iter())
                .map(|model| core::str::from_utf8(&model.name).unwrap())
                .collect();
            assert_eq!(offered.as_ref(), names, "charter {}", charter.number);
        }
    }
}

#[test]
fn the_reviewers_words_say_to_fail_when_it_cannot_review() {
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
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
fn init_inbox_holds_three_message_credits_within_the_store_codec() {
    let engine = engine(&generated());
    let limits = &engine.limits.core;
    assert_eq!(limits.tasks.result_bytes, 10_660);
    assert_eq!(limits.tasks.message_bytes, 10_660);
    assert_eq!(limits.tasks.inbox_bytes, 82 * limits.tasks.message_bytes);
    // protocol/configuration/stock.md: news and answer credits, four live
    // delegates' result reservations, and one unread result coexist.
    let required = 2 * limits.tasks.message_bytes + 5 * limits.tasks.result_bytes;
    assert_eq!(required, 74_620);
    assert!(limits.tasks.inbox_bytes >= required);
    assert!(jig_protocol_store::codec::limits(limits).is_some(), "the three-credit inbox fits the pinned store codec");
}

#[test]
fn stock_budgets_follow_the_costliest_completion_and_run_turn_floor() {
    let engine = engine(&generated());
    let init = engine.llm.models.clone();
    let mut cheap = init[0].clone();
    cheap.model.window = 1501;
    cheap.model.output = 503;
    cheap.model.input_price = 1;
    cheap.model.cached_price = 2;
    cheap.model.cache_write_price = Some(3);
    cheap.model.output_price = 4;
    cheap.model.price_unit = 1000;
    for (row, models, c, k) in [
        ("init", init.clone(), 10_896, 43_584),
        ("cheap with costliest prices and rounding", Box::new([cheap.clone()]), 7, 256),
        ("cheap before Fable", Box::new([cheap, init[2].clone()]), 10_896, 43_584),
    ] {
        let stock = crate::stock::checked_with(models, engine.limits.core.tasks.result_bytes).expect("world stock");
        assert_eq!(stock.rules.minimum_run_spend, 2 * c, "{row}");
        assert_eq!(stock.rules.maximum_run_spend, k, "{row}");
        for (number, turns, seconds) in [(1, 64, 3600), (2, 800, 28_800), (9, 4, 300)] {
            let charter = stock.charters.iter().find(|charter| charter.number == number).unwrap();
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
    let stock = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
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
    let engine = engine(&generated());
    let init = engine.llm.models.clone();
    let mut cheap = init[0].clone();
    cheap.model.window = 1501;
    cheap.model.output = 503;
    cheap.model.input_price = 1;
    cheap.model.cached_price = 2;
    cheap.model.cache_write_price = Some(3);
    cheap.model.output_price = 4;
    cheap.model.price_unit = 1000;
    for (row, models) in [("init", init), ("cheap with costliest prices and rounding", Box::new([cheap]))] {
        let stock = crate::stock::checked_with(models, engine.limits.core.tasks.result_bytes).expect("world stock");
        let policy = &stock.policies.get(&1).expect("policy 1").policy;
        let chat = policy.roles[0].authority.budget.spend;
        let pool = policy.roles[0].period_spend;
        assert!(80 * chat <= pool, "{row}: eighty open chats fit the pool");
        assert!(81 * chat > pool, "{row}: an eighty-first open chat exceeds the pool");
    }
}

#[test]
fn startup_refuses_stock_budget_overflow_before_constructing_defaults() {
    let mut models = engine(&generated()).llm.models;
    models[0].model.input_price = u64::MAX;
    models[0].model.price_unit = 1;
    assert!(crate::stock::checked_with(models, 10_660).is_none());
    assert!(crate::stock::checked_with(Box::default(), 10_660).is_none());
}

#[test]
fn the_reviewer_keeps_the_coding_run_with_read_only_workspace_families() {
    let engine = engine(&generated());
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let mut expected = defaults.charters.iter().find(|charter| charter.number == 2).unwrap().clone();
    expected.number = 4;
    expected.policy.instructions = include_bytes!("../stock/review.txt").as_slice().into();
    expected.policy.modify = false;
    expected.policy.agents = false;
    expected.policy.tools = jig_domain_tasks::Tools(512);
    assert_eq!(*defaults.charters.iter().find(|charter| charter.number == 4).unwrap(), expected);
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
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let core = config::core(&engine, [1; 16], 7);
    for ceiling in [
        &defaults.rules.ceiling,
        &defaults.policies.get(&1).unwrap().policy.ceiling,
        &core.seed_projects[0].policy.policy.ceiling,
    ] {
        use jig_domain_authority::Executor;
        for executor in [Executor::Procedure(6), Executor::Job(4)] {
            assert!(ceiling.delegation.kinds.contains(&executor));
        }
        for kind in [16, 17] {
            assert!(ceiling.grants.iter().any(|grant| grant.kind == kind));
        }
    }
    let policy = &defaults.policies.get(&1).expect("stock policy").policy;
    for (authority, expected) in [
        (&policy.roles[0].authority, &[0][..]),
        (&policy.roles[1].authority, &[1][..]),
        (&policy.roles[2].authority, &[2][..]),
        (&policy.roles[3].authority, &[][..]),
    ] {
        let roles: Box<[_]> = authority
            .delegation
            .kinds
            .iter()
            .filter_map(|kind| match kind {
                jig_domain_authority::Executor::Role(role) => Some(*role),
                jig_domain_authority::Executor::Job(_)
                | jig_domain_authority::Executor::Charter(_)
                | jig_domain_authority::Executor::Procedure(_) => None,
            })
            .collect();
        assert_eq!(roles.as_ref(), expected);
        if expected.is_empty() {
            assert!(authority.delegation.kinds.is_empty());
        } else {
            let executors: Box<[_]> = policy
                .ceiling
                .delegation
                .kinds
                .iter()
                .copied()
                .filter(|kind| match kind {
                    jig_domain_authority::Executor::Role(_) => false,
                    jig_domain_authority::Executor::Job(_)
                    | jig_domain_authority::Executor::Charter(_)
                    | jig_domain_authority::Executor::Procedure(_) => true,
                })
                .chain(expected.iter().map(|role| jig_domain_authority::Executor::Role(*role)))
                .collect();
            assert_eq!(authority.delegation.kinds.as_ref(), executors.as_ref());
        }
    }
}

#[test]
fn the_chats_words_read_a_changes_code_before_briefing_it() {
    let words = include_str!("../stock/chat.txt");
    assert!(words.contains("For a repository change, first read touched code through forge reads (read_branch for its head, then read_file), then call change with their request and only what you read: never describe unread code."));
    assert!(words.contains("If a requested fix's fault is absent, change nothing: tell the person what the code has."));
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
fn the_coders_words_name_which_task_is_its_change() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(words.contains("then call `message` to your requester"));
    assert!(words.contains("`[Requested by task N]` line names and never `[This task: N]`"));
    assert!(words.contains("in the question form"));
}

#[test]
fn the_coders_words_put_its_report_in_finish() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(words.contains("Written whole in one `finish`"));
    assert!(words.contains("The body holds what your spec's `## Report` asks, whole"));
    assert!(words.contains("list what you cut under `## Left`, and finish."));
}

#[test]
fn the_coders_words_explain_what_a_delivery_keeps() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(words.contains("**Your checkout is detached:** `deliver` commits its files on the host's own head"));
    assert!(words.contains("name no commit from an earlier run: it may not be on the branch."));
}

#[test]
fn the_coders_words_say_what_a_resume_holds() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(words.contains("temper merged your branch's tip into your work before this run"));
    assert!(words.contains("Settle a merge's conflicts before you ask"));
    assert!(words.contains("keeps none of its edits."));
}

#[test]
fn the_coders_words_wait_after_asking() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(words.contains("Then call `wait`, not `finish`: the answer reaches this run."));
}

#[test]
fn the_coders_words_build_additions_in_the_code_there_is() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(words.contains("Where the spec's words differ from the code, build what it asks with the code's names."));
}

#[test]
fn the_coders_words_preserve_both_sides_in_repairs_and_resolutions() {
    let words = crate::stock::charter_set().implementer.unwrap();
    assert!(
        words.contains("keep the requested change and unrelated base content, appended lines and blank lines included")
    );
    assert!(words.contains("restore losses a review names; never remove content to pass checks."));
    assert!(words.contains("Until then it has no pull request:"));
}

#[test]
#[expect(clippy::too_many_lines, reason = "each setting kind is checked against its admission bounds")]
fn kinds_are_declared_with_their_bounds() {
    const CONNECTOR: u16 = 5;
    // Two list lengths, twenty 127-byte items and six 553-byte path sets.
    const CLASSES_BYTES: u32 = 8 + 20 * 127 + 6 * 553;
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
    limits.templates = 9;
    let rule = settings::Value::Rule(settings::LandingRule {
        sets: Box::new([1, 2, 3, 4, 5, 6]),
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
                value_bytes: 7,
                per_scope: 1,
                charters: 0
            },
            Kind {
                number: 8,
                scope: KindScope::Project,
                connector: CONNECTOR,
                value_bytes: CLASSES_BYTES,
                per_scope: 1,
                charters: 0
            },
            Kind {
                number: 9,
                scope: KindScope::Deployment,
                connector: CONNECTOR,
                value_bytes: 124,
                per_scope: limits.tasks.charters,
                charters: 0
            },
            Kind {
                number: 10,
                scope: KindScope::Project,
                connector: CONNECTOR,
                value_bytes: 608,
                per_scope: 1,
                charters: 0,
            },
            Kind {
                number: 11,
                scope: KindScope::Deployment,
                connector: CONNECTOR,
                value_bytes: 2,
                per_scope: 9,
                charters: 0
            },
            Kind {
                number: 12,
                scope: KindScope::Project,
                connector: CONNECTOR,
                // Repositories, branch length prefixes, paths, owner/start/options, producer and check.
                value_bytes: 6
                    + 4
                    + 9
                    + 4
                    + limits.forge.follow_paths * (4 + 256)
                    + 6
                    + 4
                    + 9
                    + 9
                    + 1
                    + 1
                    + 4
                    + 8192
                    + 4
                    + 256
                    + 4
                    + 256
                    + 1,
                per_scope: limits.forge.follows,
                charters: 1
            },
            Kind {
                number: 13,
                scope: KindScope::Project,
                connector: CONNECTOR,
                value_bytes: 4,
                per_scope: limits.forge.dependencies,
                charters: 0
            },
        ]
    );
    limits.authority.segment_bytes = 6;
    assert_eq!(crate::stock::kinds(&limits, CONNECTOR)[0].value_bytes, rule_bytes - 3 * (9 - 6));
}

#[test]
fn the_binary_gives_the_forges_stock_at_every_start() {
    // protocol/configuration/stock.md: every project's rule 1, both forge charters and each charter's words and template 4's work.
    const ITEMS: usize = 13;
    use temper_engine_domain_forge::{self as forge, settings};
    let engine = fixtures::engine();
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let stock = crate::stock::forge(&limits);
    assert_eq!(stock.len(), ITEMS);
    assert_eq!((stock[0].kind, stock[0].scope, stock[0].number), (2, 0, 1));
    assert_eq!(
        stock[0].value.value,
        settings::Value::Rule(settings::LandingRule {
            sets: Box::new([]),
            pattern: forge::GatePattern {
                segments: Box::new([b"branch".as_slice().into()]),
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
    for (item, kind, number, value) in [
        (&stock[1], 3, 1, settings::Value::Charter(2)),
        (&stock[2], 3, 2, settings::Value::Charter(4)),
        (&stock[3], 11, 4, settings::Value::Goal(settings::TemplateGoal { design: false, close_out: false })),
    ] {
        assert_eq!((item.kind, item.scope, item.number), (kind, 0, number));
        assert_eq!(item.value.value, value);
    }
    for (item, (number, line)) in stock[4..].iter().zip([
        (1, b"talks with you and asks for the changes".as_slice()),
        (2, b"makes each change, repairs it and resolves its conflicts".as_slice()),
        (4, b"reviews each change at its head, and holds it until it approves".as_slice()),
        (5, b"designs a goal's draft, cuts it into changes, and answers what they ask".as_slice()),
        (9, b"writes the few lines at the top of Home, from what landed and what waits".as_slice()),
        (10, b"answers one word, so that a new model is known to run".as_slice()),
        (11, b"explains how work runs here, and proposes changes to it".as_slice()),
        (12, b"writes a draft with you, one question at a time, and starts nothing".as_slice()),
    ]) {
        assert_eq!((item.kind, item.scope, item.number), (9, 0, number));
        assert_eq!(item.value.value, settings::Value::CharterWords(line.into()), "charter {number}");
    }
    for item in &stock {
        assert_eq!(item.value.size, temper_engine_smith::kinds::size(&item.value.value));
        assert_eq!(
            usize::try_from(item.value.size).unwrap(),
            temper_engine_smith::kinds::encode(&item.value.value).len()
        );
    }
}

#[test]
fn every_stock_charter_has_exactly_one_fitting_line() {
    use temper_engine_domain_forge::settings::{LINE_BYTES, Value, words_fit};
    let engine = fixtures::engine();
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let defaults = crate::stock::defaults(&engine.jig.llm, engine.jig.limits.core.tasks.result_bytes);
    let stock = crate::stock::forge(&limits);
    for charter in &defaults.charters {
        let lines: Box<[_]> = stock.iter().filter(|item| item.kind == 9 && item.number == charter.number).collect();
        assert_eq!(lines.len(), 1, "charter {} has exactly one stock line", charter.number);
        assert_eq!(lines[0].scope, 0, "charter {} has deployment words", charter.number);
        let Value::CharterWords(line) = &lines[0].value.value else { panic!("charter {} has words", charter.number) };
        assert!(words_fit(line, LINE_BYTES), "charter {} has a fitting line", charter.number);
    }
}

#[test]
fn every_stock_role_keeps_its_whole_line_in_who() {
    use temper_engine_domain_forge::settings::Value;
    use temper_engine_domain_ways::{WHO, role_who};
    let engine = fixtures::engine();
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let stock = crate::stock::forge(&limits);
    for (number, when) in [
        (1, b"For each chat".as_slice()),
        (11, b"When you ask"),
        (12, b"When you start a draft"),
        (5, b"For each goal"),
        (2, b"For each change"),
        (4, b"For each change"),
        (9, b"When you look at Home after landings"),
    ] {
        let item = stock.iter().find(|item| item.kind == 9 && item.number == number).expect("role's stock words");
        let Value::CharterWords(line) = &item.value.value else { panic!("role's words") };
        let who = role_who(when, line);
        assert_eq!(who.as_ref(), [when, b": ", line].concat(), "charter {number}");
        assert!(who.len() <= WHO, "charter {number}");
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "stock way rows keep their words and settings together")]
fn the_stock_ways_hold_their_words_and_settings() {
    use temper_engine_domain_forge::{
        self as forge,
        settings::{LandingRule, RuleApproval, RuleGate, Value},
    };
    use temper_engine_domain_ways::Laid;
    use temper_protocol_reaches::{Classes, Way};
    let all = forge::GatePattern {
        segments: Box::new([b"branch".as_slice().into()]),
        last: forge::GateLast::Open(Box::default()),
    };
    let review = LandingRule {
        sets: Box::new([]),
        pattern: all.clone(),
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: Box::new([RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Clean }]),
        approvals: Box::default(),
    };
    let rows = [
        (
            b"You approve".as_slice(),
            b"You approve every change".as_slice(),
            b"An agent makes each change; nothing reaches main until you approve it.".as_slice(),
            vec![(
                1,
                LandingRule {
                    sets: Box::new([]),
                    pattern: all,
                    enforce: true,
                    ci: true,
                    up_to_date: true,
                    gates: Box::default(),
                    approvals: Box::new([RuleApproval { role: 0, people: 1, freshness: forge::JudgeFreshness::Clean }]),
                },
            )],
            None,
            Some(Classes::stock(Way::YouApprove)),
            false,
        ),
        (
            b"Agents review".as_slice(),
            b"Agents review every change, and code lands it".as_slice(),
            b"An agent makes each change, another reviews it, and code lands it.".as_slice(),
            vec![(1, review)],
            None,
            Some(Classes::stock(Way::AgentsReview)),
            false,
        ),
        (
            b"Draft first".as_slice(),
            b"Draft first, then agents build it on its own branch".as_slice(),
            b"You and an agent write a design draft; agents build it on its own branch, and it lands whole.".as_slice(),
            vec![
                (
                    1,
                    LandingRule {
                        sets: Box::new([]),
                        pattern: forge::GatePattern {
                            segments: Box::new([b"branch".as_slice().into()]),
                            last: forge::GateLast::Exact(b"main".as_slice().into()),
                        },
                        enforce: true,
                        ci: true,
                        up_to_date: true,
                        gates: Box::default(),
                        approvals: Box::default(),
                    },
                ),
                (
                    2,
                    LandingRule {
                        sets: Box::new([]),
                        pattern: forge::GatePattern {
                            segments: Box::new([b"branch".as_slice().into()]),
                            last: forge::GateLast::Open(b"temper".as_slice().into()),
                        },
                        enforce: true,
                        ci: false,
                        up_to_date: false,
                        gates: Box::default(),
                        approvals: Box::default(),
                    },
                ),
            ],
            Some(Laid { drafts: true, goals: true, way: 4 }),
            Some(Classes::stock(Way::DraftFirst)),
            true,
        ),
    ];
    let stock = crate::stock::ways();
    assert_eq!(stock.len(), rows.len());
    for (way, (name, headline, line, rules, ways, classes, delegation)) in stock.iter().zip(rows) {
        assert_eq!(way.name.as_ref(), name);
        assert_eq!(way.headline.as_ref(), headline, "{name:?}");
        assert_eq!(way.line.as_ref(), line, "{name:?}");
        assert_eq!(way.rules.as_ref(), rules.as_slice(), "{name:?}");
        assert_eq!(way.ways, ways, "{name:?}");
        assert_eq!(way.classes, classes, "{name:?}");
        assert_eq!(way.delegation, delegation, "{name:?}");
    }
    let engine = fixtures::engine();
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let forge_stock = crate::stock::forge(&limits);
    let rule = forge_stock.iter().find(|item| (item.kind, item.number) == (2, 1)).unwrap();
    assert_eq!(Value::Rule(stock[1].rules[0].1.clone()), rule.value.value);
}

#[test]
fn the_stock_pool_funds_eighty_chats_at_once() {
    let engine = engine(&generated());
    let stock = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    assert_eq!(stock.rules.minimum_run_spend, 2 * 10_896);
    assert_eq!(stock.rules.maximum_run_spend, 43_584);
    let policy = &stock.policies.get(&1).expect("policy 1").policy;
    assert_eq!(stock.templates[0].budget, jig_domain_people::TemplateBudget::AsAllowed { cap: None });
    let chat_spend = policy.roles[0].authority.budget.spend;
    assert_eq!(chat_spend, 24 * 43_584);
    let pool = policy.roles[0].period_spend;
    assert!(pool >= 80 * chat_spend);
    assert!(pool < 81 * chat_spend);
}

#[test]
fn the_stock_reports_follow_the_deployment_result_bound_and_keep_its_floor() {
    let mut engine = engine(&generated());
    let stock = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let coordinator = stock.charters.iter().find(|charter| charter.number == 5).unwrap();
    let mut expected = stock.charters.iter().find(|charter| charter.number == 2).unwrap().clone();
    expected.number = 5;
    expected.policy.instructions = coordinator.policy.instructions.clone();
    expected.parts = coordinator.parts.clone();
    expected.policy.resume = false;
    expected.policy.waiting = skein_lib::Duration::ZERO;
    expected.policy.tools = jig_domain_tasks::Tools(1023);
    expected.wake = jig_domain_tasks::WakePolicy {
        words: jig_domain_tasks::WakeRule::Immediate,
        notices: jig_domain_tasks::WakeRule::Immediate,
        news: jig_domain_tasks::WakeRule::Paced { span: skein_lib::Duration::from_secs(1800) },
        results: jig_domain_tasks::ResultsWake::LastOrFailure,
        questions: true,
        answers: true,
        timers: true,
    };
    assert_eq!(coordinator, &expected);
    let mut expected = stock.templates.iter().find(|template| template.number == 2).unwrap().clone();
    expected.number = 4;
    expected.name = b"goal for a draft".as_slice().into();
    expected.executor = jig_domain_people::TemplateExecutor::Charter(5);
    expected.resources = Box::new([jig_domain_people::Symbol { connector: 1, symbol: 1 }]);
    expected.flags.ended_by = jig_domain_tasks::EndedBy::Closer(jig_domain_tasks::Closer { connector: 1, code: 6 });
    assert_eq!(stock.templates.iter().find(|template| template.number == 4).unwrap(), &expected);
    for result_bytes in [32, 10_660, 262_144] {
        engine.limits.core.tasks.result_bytes = result_bytes;
        let stock = crate::stock::with_models(engine.llm.models.clone(), result_bytes);
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
    assert!(
        document
            .contains("Use join true to continue the line before first without a newline when a line exceeds one call")
    );
    assert_document_rule(document);
}

#[test]
fn stock_words_hold_no_placeholder_a_run_cannot_fill() {
    for words in [
        include_str!("../stock/ask.txt"),
        include_str!("../stock/chat.txt"),
        include_str!("../../../../docs/charters/stock/common.md"),
        include_str!("../../../../docs/charters/stock/goal.md"),
        include_str!("../../../../docs/charters/stock/designer.md"),
        include_str!("../../../../docs/charters/stock/coordinator.md"),
        include_str!("../../../../docs/charters/stock/implementer.md"),
        include_str!("../stock/review.txt"),
        include_str!("../stock/status.txt"),
        include_str!("../stock/trial.txt"),
    ] {
        assert!(!words.contains("PROJECT"));
        assert!(!words.contains("DATE"));
    }
}

fn assert_document_rule(words: &str) {
    for phrase in [
        "scope goal, project P and goal T from your brief's [This task: T, in project P]",
        "description its title",
        "Write its first version whole, in Markdown: # TITLE",
        "muted line _Draft, with the user._",
        "one ## section per topic",
        "open questions under ## Open questions",
        "Then change it by lines",
        "revision you read from your brief, recall or your last write's answer",
        "To append, first is the note's lines + 1 and remove 0",
        "Never rewrite it whole past a few KB",
        "build a long document in parts",
        "A refused note says the revision or the line count to use",
        "Say in one sentence what changed",
        "do not repeat the document in your reply",
        "Where a sequence says more drawn, write a flow",
        "one path joined by -->",
    ] {
        assert!(words.contains(phrase), "missing document instruction: {phrase}");
    }
    assert!(!words.contains("Mermaid is not drawn"), "obsolete document instruction: Mermaid is not drawn");
}

#[test]
fn the_status_charter_is_small_and_reaches_nothing() {
    let engine = engine(&generated());
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
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
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let status = defaults.charters.iter().find(|charter| charter.number == 9).expect("status charter");
    assert_eq!(status.policy.instructions.as_ref(), words.as_bytes());
}

#[test]
fn the_rules_delegate_to_the_status_charter() {
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let policy = &defaults.policies.get(&1).expect("policy 1").policy;
    for authority in [&defaults.rules.ceiling, &policy.ceiling]
        .into_iter()
        .chain(policy.roles.iter().filter(|role| role.authority.budget.spend != 0).map(|role| &role.authority))
    {
        assert!(authority.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(9)));
    }
}

// temper/docs/design/protocol/configuration/stock.md; temper/docs/design/domain/trial.md, 2.
#[test]
fn the_trial_charter_is_one_working_turn_and_reaches_nothing() {
    let engine = engine(&generated());
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let trial = defaults.charters.iter().find(|charter| charter.number == 10).expect("trial charter");
    let mut expected = defaults.charters[0].clone();
    expected.number = 10;
    expected.policy.instructions = include_bytes!("../stock/trial.txt").as_slice().into();
    expected.policy.turns = 2;
    expected.policy.time = skein_lib::Duration::from_secs(120);
    expected.policy.resume = false;
    expected.policy.inspect = false;
    expected.policy.agents = false;
    expected.policy.tools = jig_domain_tasks::Tools(0);
    assert_eq!(*trial, expected);
    assert_eq!(trial.policy.reserve_turns, 1);
    assert_eq!(trial.policy.reserve_time, skein_lib::Duration::from_secs(60));
    assert!(!trial.policy.modify);
    assert!(!trial.policy.shell);
}

#[test]
fn the_trial_charters_words_ask_for_one_word() {
    let words = include_str!("../stock/trial.txt");
    assert_eq!(words, "Say one word and nothing else. Use no tools.");
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let trial = defaults.charters.iter().find(|charter| charter.number == 10).expect("trial charter");
    assert_eq!(trial.policy.instructions.as_ref(), words.as_bytes());
}

#[test]
fn no_policy_delegates_to_the_trial_charter() {
    let engine = single_engine();
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let authorities =
        core::iter::once(&defaults.rules.ceiling).chain(defaults.policies.iter().flat_map(|(_, value)| {
            core::iter::once(&value.policy.ceiling).chain(value.policy.roles.iter().map(|role| &role.authority))
        }));
    for authority in authorities {
        assert!(!authority.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(10)));
    }
}

#[test]
fn stock_numbers_each_model_for_good() {
    let mut llm = engine(&generated()).llm;
    // A typed caller's leftover table cannot replace the binary's catalogue.
    llm.models = Box::new([fixtures::small_model()]);
    let models = crate::stock::models(&llm);
    assert_eq!(models.len(), 8);
    for (entry, (number, provider, name)) in models.iter().zip([
        (1, 2, "claude-opus-5-5"),
        (2, 2, "claude-sonnet-5-5"),
        (3, 2, "claude-fable-5-1"),
        (4, 1, "gpt-6.1-sol"),
        (5, 1, "gpt-6-astra"),
        (6, 1, "gpt-6.1-terra"),
        (7, 1, "gpt-6-luna"),
        (8, 3, "demo"),
    ]) {
        assert_eq!(
            (entry.number, entry.model.provider, entry.model.name.as_ref()),
            (number, provider, name.as_bytes())
        );
        assert_eq!(entry.source, jig_domain_core::settings::ModelSource::Stock);
        assert!(entry.picker);
        assert!(!entry.withdrawn);
        assert!(!entry.estimated);
        assert_eq!(entry.model.window, 1_048_576);
        assert_eq!(entry.model.output, 8192);
        assert_eq!(entry.model.cache_write_price, None);
        assert!(entry.description.is_none());
    }
    assert_eq!(models[7].model.name.as_ref(), b"demo");
}

#[test]
fn a_row_whose_provider_isnt_configured_isnt_a_default() {
    let bare = engine(include_bytes!("../../../temper-init/template/bare.json"));
    let models = crate::stock::models(&bare.llm);
    assert_eq!(models.iter().map(|entry| entry.number).collect::<Box<[_]>>().as_ref(), [1, 2, 3, 4, 5, 6, 7]);
    let mut llm = engine(&generated()).llm;
    llm.providers = llm.providers.into_vec().into_iter().filter(|provider| provider.number == 3).collect();
    let models = crate::stock::models(&llm);
    assert_eq!(models.len(), 1);
    assert_eq!((models[0].number, models[0].model.provider, models[0].model.name.as_ref()), (8, 3, b"demo".as_slice()));
    let defaults = crate::stock::checked(&llm, 10_660).unwrap();
    for charter in &defaults.charters {
        assert_eq!(charter.policy.model.name.as_ref(), b"demo");
        assert!(charter.rest);
    }
}

#[test]
fn the_stock_has_no_foreman() {
    use temper_engine_domain_forge::settings::Value;
    let engine = fixtures::engine();
    let limits = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
    let defaults = crate::stock::defaults(&engine.jig.llm, engine.jig.limits.core.tasks.result_bytes);
    assert!(!defaults.charters.iter().any(|charter| charter.number == 6));
    assert!(!defaults.rules.ceiling.delegation.kinds.contains(&jig_domain_authority::Executor::Charter(6)));
    let stock = crate::stock::forge(&limits);
    assert_eq!(
        stock.iter().filter(|item| item.kind == 3).map(|item| item.number).collect::<Box<[_]>>().as_ref(),
        [1, 2]
    );
    assert!(!stock.iter().any(|item| item.kind == 9 && item.number == 6));
    let goal = stock.iter().find(|item| item.kind == 9 && item.number == 5).expect("goal's words");
    assert_eq!(
        goal.value.value,
        Value::CharterWords(
            b"designs a goal's draft, cuts it into changes, and answers what they ask".as_slice().into()
        )
    );
    assert!(!include_str!("../stock/ask.txt").contains("answer change questions"));
}
