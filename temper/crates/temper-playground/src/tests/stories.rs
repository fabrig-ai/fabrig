//! Story data checked before its native walk (temper/docs/design/playground.md, 5.1).
use crate::{
    documents,
    stories::{A, all},
    story,
};
use serde_json::{Value, json};
use skein_fake_llm_domain::{self as model, api};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use skein_shell::Host;
use smith_domain_run::{self as run, charter};
use temper_fake::{forge, llm};

const OTHERWISE: &str = "I am the scripted model of temper's playground, running in your browser: I answer only its three suggested requests: Add CSV export to widget, Add a dark theme to widget and Fix widget's empty search state. A real model comes with temper itself: download it to connect yours.";
const DESCRIPTION: &str = "Try the playground, then ask the three suggested requests: the CSV export is delegated, pushed, passed by CI and landed on main; the dark theme and the empty search are answered in text.";
const CODING: &str = "Read only what the change needs, then write it in its files.\n\n## Task\n\nExport widget's table as CSV\n[Change connector 1, kind 2, at most 4096 bytes]\n[This task: 3, in project 1]\n[Requested by task 2]\n";
const REVIEW: &str = "Review the change with read-only authority.\n\n## Task\n\nReview this change at the named head\nChange words:\nExport widget's table as CSV\n[This task: 4, in project 1]\n[Requested by task 2]\n";
const BRIEFS: [(&str, &str, usize, run::Mode); 5] = [
    ("Add CSV export to widget", "asked", 0, run::Mode::Open),
    ("Add a dark theme to widget", "asked", 1, run::Mode::Open),
    ("Fix widget's empty search state", "asked", 2, run::Mode::Open),
    (CODING, "cued", 0, run::Mode::Finite),
    (REVIEW, "cued", 1, run::Mode::Finite),
];

#[test]
fn story_a_decodes_within_the_fakes_bounds() {
    let llm = llm::Document::decode(A.llm).expect("story LLM within its document and joint script bounds");
    let _domain = llm.domain(7);
    let forge = forge::Document::decode(A.forge).expect("story forge within its document bounds");
    let _seeded = forge.seed(7);
    assert_eq!(forge.owner(), b"playground");
    let [repository] = forge.repositories() else { panic!("one repository") };
    assert_eq!(repository.name.as_ref(), b"widget");
    assert_eq!(repository.landing.as_ref(), b"main");
    assert!(repository.ci.is_none());
    assert!(repository.tree.is_none(), "the box's default seed");
    assert!(repository.id.is_none(), "the box's repository numbering");
    assert_eq!(forge.listen(), documents::FORGE);
    assert_eq!(forge.admin_token(), b"/setup/admin.token");
    assert_eq!(llm.listen(), (std::net::Ipv4Addr::LOCALHOST, 443).into());
    assert_eq!(llm.key(), b"/setup/llm.key");
    let deployment: Value = serde_json::from_slice(A.deployment).expect("deployment JSON");
    assert_eq!(deployment["engine"]["projects"], json!([]));
    assert!(deployment["application"]["forges"].is_null());
    assert_eq!(deployment["application"]["machine"]["fake"]["listen"]["socket"], forge.listen().to_string());
    assert_eq!(deployment["engine"]["llm"]["endpoints"][0]["address"]["socket"], llm.listen().to_string());
    assert_eq!(deployment["application"]["demo"]["llm"]["key"], "/setup/llm.key");
    let [model] = llm.models() else { panic!("one model") };
    assert!(deployment["engine"]["llm"].get("models").is_none());
    assert!(deployment["engine"]["llm"]["providers"][0].get("models").is_none());
    let engine = documents::settings::configuration(&deployment);
    assert_eq!(engine.llm.models[0].model.name.as_ref(), model.name.as_ref());
    assert_eq!(engine.llm.models[0].model.window, model.window);
    assert_eq!(engine.llm.models[0].model.output, model.output);
    assert_eq!(all().len(), 1);
    assert_eq!(all()[0].name, "a");
    assert_eq!(all()[0].deployment, A.deployment);
    assert_eq!(A.description, DESCRIPTION);
}

#[test]
fn story_as_chats_keep_the_stock_requester_ending() {
    let document: Value = serde_json::from_slice(A.deployment).expect("story deployment");
    let temper_service::stock::Given::Own { jig, .. } = documents::given(&document) else {
        panic!("the composition supplies its settings")
    };
    let engine = documents::settings::configuration(&document);
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    assert_eq!(jig.templates[0].flags.ended_by, stock.templates[0].flags.ended_by);
    assert_eq!(jig.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Requester);
}

#[test]
fn harness_chats_keep_the_result_ending() {
    let temper_service::stock::Given::Own { jig, .. } = documents::given(&documents::world_document()) else {
        panic!("the composition supplies its settings")
    };
    assert_eq!(jig.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Result);
}

#[test]
fn story_as_forge_keeps_the_product_stock() {
    use temper_engine_domain_forge::settings::Value;
    let document: serde_json::Value = serde_json::from_slice(A.deployment).expect("story deployment");
    let temper_service::stock::Given::Own { forge, .. } = documents::given(&document) else {
        panic!("the composition supplies its settings")
    };
    let engine = documents::settings::configuration(&document);
    let limits = temper_service::limits::engine(
        &engine.limits,
        &temper_service::limits::profile(temper_service::limits::Profile::Small),
    )
    .expect("playground engine limits");
    assert_eq!(forge, temper_service::stock::forge(&limits));
    let charters: Vec<_> = forge.iter().filter(|item| item.kind == 3).map(|item| &item.value.value).collect();
    assert_eq!(charters, [&Value::Charter(2), &Value::Charter(4)]);
}

#[test]
fn harness_forge_keeps_its_settings() {
    let document = documents::world_document();
    let temper_service::stock::Given::Own { forge, .. } = documents::given(&document) else {
        panic!("the composition supplies its settings")
    };
    assert_eq!(forge, documents::settings::forge(&document));
    let change = forge.iter().find(|item| item.kind == 3 && item.number == 1).expect("change charter");
    assert_eq!(change.value.value, temper_engine_domain_forge::settings::Value::Charter(1));
}

#[test]
fn story_as_documents_hold_the_product_change_and_review() {
    use jig_domain_tasks::{Contract, Parameter, Spec, Verdict};
    // The chat, change, producer and review coexist during the landing.
    const TASKS: u32 = 4;
    let deployments = [
        serde_json::from_slice(A.deployment).expect("story deployment"),
        documents::world_document_with(documents::configure),
    ];
    let spec = Spec {
        words: b"Review this change at the named head\nPull request: #1\nBranch: temper/1/c2\nBase: main\nHead: 0000000000000000000000000000000000000000\n".as_slice().into(),
        parameters: Box::new([
            Parameter::Resource { name: 1, connector: 1, resource: 4_294_967_297 },
            Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
            Parameter::Bytes { name: 3, value: b"temper/1/c2".as_slice().into() },
            Parameter::Number { name: 7, value: 1 },
            Parameter::Bytes { name: 8, value: b"0000000000000000000000000000000000000000".as_slice().into() },
        ]),
        job: None,
        asks: false,
        inputs: Box::new([]),
    };
    let contract = Contract::Verdict {
        choices: Box::new([
            Verdict { code: 1, words: 128, followups: 0, fields: Box::new([]) },
            Verdict { code: 2, words: 128, followups: 0, fields: Box::new([]) },
        ]),
    };
    for document in deployments {
        let engine = documents::settings::configuration(&document);
        let limits = &engine.limits.core.tasks;
        assert!(limits.tasks >= TASKS, "the producer and reviewer fit beside the chat and change");
        assert!(engine.limits.core.people.inbox_tasks >= 2 * TASKS, "people's inbox follows task admission");
        assert!(jig_domain_tasks::valid_spec(limits, &spec), "the review's identity and five parameters fit");
        assert!(jig_domain_tasks::valid_contract(limits, &contract), "the review can approve or request changes");
    }
}

#[test]
fn story_as_stock_authority_funds_its_scripted_delegate() {
    // The tree includes the chat and its immediate delegate as well as the delegate's descendants.
    const CREATORS: u64 = 2;
    let (engine, defaults) = story_a_stock();
    let llm: Value = serde_json::from_slice(A.llm).expect("story LLM");
    let authority = &llm["asked"][0]["turns"][0]["lines"][1]["arguments"]["batch"][0]["authority"];
    let spend = authority["budget"]["spend"].as_u64().expect("delegate spend");
    let policy = &defaults.policies.get(&1).expect("playground policy").policy;
    assert!(playground_owner(&defaults).budget.spend >= spend, "the chat funds its delegate");
    assert!(policy.ceiling.budget.spend >= spend, "the policy funds the delegate");
    assert!(defaults.rules.ceiling.budget.spend >= spend, "the rules fund the delegate");
    assert!(
        u64::from(engine.limits.core.tasks.tree_tasks)
            >= authority["delegation"]["tasks"].as_u64().expect("delegate task count") + CREATORS,
        "the task tree holds the delegate's authority"
    );
}

#[test]
fn story_as_stock_runs_fit_the_hosted_agents_bounds() {
    let (_, defaults) = story_a_stock();
    assert!(documents::agent().bounds.spend >= defaults.rules.maximum_run_spend);
    let budget = documents::agent_limits().domain.run.budget;
    for charter in &defaults.charters {
        assert!(budget.turns >= charter.policy.turns, "the hosted agent admits the stock turns");
        assert!(budget.time >= charter.policy.time, "the hosted agent admits the stock time");
    }
}

#[test]
fn story_as_project_has_room_for_the_chat_and_its_delegate() {
    // The chat and the script's immediate batch coexist in the same project.
    const CHAT: u64 = 1;
    let llm: Value = serde_json::from_slice(A.llm).expect("story LLM");
    let batch = llm["asked"][0]["turns"][0]["lines"][1]["arguments"]["batch"].as_array().expect("delegate batch");
    let tasks = CHAT + u64::try_from(batch.len()).expect("batch size");
    let document: Value = serde_json::from_slice(A.deployment).expect("story deployment");
    let engine = documents::settings::configuration(&document);
    assert!(u64::from(engine.limits.core.tasks.project_tasks) >= tasks);
}

#[test]
fn story_as_delegates_fit_the_result_contract_bound() {
    let llm: Value = serde_json::from_slice(A.llm).expect("story LLM");
    let contract = &llm["asked"][0]["turns"][0]["lines"][1]["arguments"]["batch"][0]["contract"];
    for (engine, _) in configured_deployments() {
        assert!(jig_domain_tasks::valid_contract(
            &engine.limits.core.tasks,
            &jig_domain_tasks::Contract::Change {
                connector: contract["connector"].as_u64().expect("connector").try_into().expect("connector fits"),
                kind: contract["change_kind"].as_u64().expect("change kind").try_into().expect("kind fits"),
                words: contract["words"].as_u64().expect("words").try_into().expect("words fit"),
            }
        ));
    }
}

#[test]
fn story_as_deployment_offers_its_llm_cues_in_order() {
    let deployment: Value = serde_json::from_slice(A.deployment).expect("story deployment");
    let llm: Value = serde_json::from_slice(A.llm).expect("story LLM");
    let cues: Vec<_> =
        llm["asked"].as_array().expect("asked scripts").iter().map(|script| json!({"cue": script["cue"]})).collect();
    assert_eq!(deployment["application"]["demo"]["llm"]["asked"], json!(cues));
}

#[test]
fn story_as_documents_can_list_every_offered_charter() {
    for (engine, jig) in configured_deployments() {
        let offered = jig.charters.iter().filter(|charter| charter.number != 10).count();
        assert!(
            engine.limits.core.documents.lines >= u32::try_from(offered).expect("offered charter count"),
            "the creating document lists every eligible charter before offering a chat"
        );
    }
}

#[test]
fn story_as_chat_requests_fit_the_peoples_payload_bound() {
    let llm: Value = serde_json::from_slice(A.llm).expect("story LLM");
    for (engine, jig) in configured_deployments() {
        let authority = jig_domain_core::authority_to_people(playground_owner(&jig)).expect("owner authority");
        let authority_bytes = jig_domain_people::authority_bytes(&authority).expect("bounded authority");
        for script in llm["asked"].as_array().expect("asked scripts") {
            let cue = script["cue"].as_str().expect("suggestion");
            let bytes = authority_bytes + u64::try_from(cue.len()).expect("cue length");
            assert!(
                bytes <= u64::from(engine.limits.core.people.amendment_bytes),
                "{cue:?}: chat payload needs {bytes} bytes, bound {}",
                engine.limits.core.people.amendment_bytes
            );
        }
    }
}

#[test]
fn story_as_task_bounds_hold_the_chat_owners_authority() {
    for (engine, jig) in configured_deployments() {
        let authority = playground_owner(&jig);
        assert!(
            jig_domain_tasks::valid_authority(&engine.limits.core.tasks, &jig_domain_core::task_authority(authority)),
            "the chat inherits its owner's {} grants within task authority bounds",
            authority.grants.len()
        );
    }
}

#[test]
fn story_as_scripts_are_the_templates() {
    let story: Value = serde_json::from_slice(A.llm).expect("story LLM");
    let template: Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/temper.json")).expect("template");
    for scripts in ["asked", "cued"] {
        let templates = template["application"]["demo"]["llm"][scripts].as_array().expect("template scripts");
        for script in story[scripts].as_array().expect("story scripts") {
            let matching = templates.iter().find(|candidate| candidate["cue"] == script["cue"]).expect("template cue");
            let mut expected = matching.clone();
            if scripts == "asked" && script["cue"] == "Add CSV export to widget" {
                // The standalone story prices its delegate with its own model.
                expected["turns"][0]["lines"][1]["arguments"]["batch"][0]["authority"]["budget"]["spend"] =
                    json!(21_140_u64);
                expected["turns"][0]["tokens"] = json!(269_u32);
            }
            assert_eq!(script, &expected, "{scripts}");
        }
    }
    assert_eq!(
        story["otherwise"],
        json!([{"lines":[{"text":OTHERWISE}],"finish":"stop","tokens":OTHERWISE.len().div_ceil(4)}])
    );
}

#[test]
fn story_as_cues_are_matched_apart() {
    let value: Value = serde_json::from_slice(A.llm).expect("story LLM");
    assert_eq!(value["asked"].as_array().expect("asked scripts").len(), 3);
    assert_eq!(value["cued"].as_array().expect("cued scripts").len(), 2);
    for (own, list, index, _) in BRIEFS {
        let script = &value[list][index];
        let cue = script["cue"].as_str().expect("cue");
        assert!(own.contains(cue));
        let mut only = value.clone();
        only["asked"] = json!([]);
        only["cued"] = json!([]);
        only[list] = json!([script]);
        let document =
            llm::Document::decode(&serde_json::to_vec(&only).expect("isolated script")).expect("script document");
        for (words, _, _, mode) in BRIEFS {
            let answer = answer(&document, opening(words, mode));
            if words == own {
                let first = &script["turns"][0]["lines"][0];
                if let Some(text) = first["text"].as_str() {
                    let Some(api::Part::Text { text: said }) = answer.parts.first() else { panic!("script's text") };
                    assert_eq!(said.as_ref(), text.as_bytes());
                } else {
                    let Some(api::Part::ToolCall { name, .. }) = answer.parts.first() else { panic!("script's call") };
                    assert_eq!(name.as_ref(), first["call"].as_str().expect("call name").as_bytes());
                }
            } else {
                assert!(!words.contains(cue), "{cue:?} only selects its own brief");
                assert_eq!(answer.parts.as_ref(), &[api::Part::Text { text: OTHERWISE.as_bytes().into() }]);
                assert_eq!(answer.finish, api::Finish::Stop);
            }
        }
    }
}

fn answer(document: &llm::Document, opening: run::Opening) -> api::Answer {
    let query = api::Query {
        credential: Some(0),
        model: b"fake".as_slice().into(),
        system: opening.system,
        messages: Box::new([api::Message {
            role: api::Role::User,
            parts: Box::new([api::Part::Text { text: opening.prompt }]),
        }]),
        tools: Box::new([]),
        choice: api::ToolChoice::None,
        max_tokens: 8192,
        caching: api::Caching::Unscoped,
    };
    let mut domain = document.domain(23);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: document.limits() };
    let mut out = Queue::with_capacity(1);
    model::step(
        &mut domain,
        &env,
        model::Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: Token::new(2), query },
        &mut out,
    );
    assert!(out.is_empty(), "reply waits for its pace");
    env.now = domain.next_deadline().expect("paced reply");
    model::fire(&mut domain, &env, &mut out);
    let Some(model::Request::Reply { result, .. }) = out.pop() else { panic!("one whole reply") };
    assert!(out.is_empty());
    result.expect("script's answer")
}

#[test]
fn story_as_pace_is_paced() {
    use skein_fake_forge_domain::{self as domain, api as f};
    let llm = llm::Document::decode(A.llm).expect("LLM document");
    assert_eq!(llm.limits().latency_min, Duration::from_millis(1200));
    assert_eq!(llm.limits().latency_max, Duration::from_millis(1200));
    let forge = forge::Document::decode(A.forge).expect("forge document");
    let (mut forge, _) = forge.seed(7);
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge::limits() };
    let head =
        domain::advance(&mut forge.domain, &env, b"playground/widget", b"main", b"ci/result", b"green", forge::OWNER)
            .expect("seeded repository");
    let due = Time::ZERO.saturating_add(Duration::from_millis(4000));
    let mut out = Queue::with_capacity(128);
    env.now = Time::ZERO.saturating_add(Duration::from_millis(3999));
    domain::fire(&mut forge.domain, &env, &mut out);
    let f::Answer::Statuses(statuses) = forge
        .domain
        .inspect(&forge::limits(), b"playground/widget", &f::Read::Statuses { commit: head })
        .expect("CI status")
    else {
        panic!("statuses")
    };
    assert_eq!(statuses[0].state, f::Check::Pending);
    assert_eq!(forge.domain.next_deadline(), Some(due));
    env.now = due;
    domain::fire(&mut forge.domain, &env, &mut out);
    let f::Answer::Statuses(statuses) = forge
        .domain
        .inspect(&forge::limits(), b"playground/widget", &f::Read::Statuses { commit: head })
        .expect("CI status")
    else {
        panic!("statuses")
    };
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0].state, f::Check::Passed);
}

#[test]
fn story_a_starts_and_prints_its_link() {
    let wall = Wall::from_nanos(1_700_000_000_000_000_000);
    let start = A.start(7, wall);
    assert_eq!(start.links, crate::Links::Print);
    assert_eq!(start.deployment.as_ref(), A.deployment);
    let mut world = story::world(start);
    assert_eq!(world.seed(), 7);
    assert_eq!(world.wall(), wall);
    let mut idle = false;
    for _ in 0..10_000 {
        story::step(&mut world);
        if !world.busy() && world.next_due().is_none_or(|due| due > world.now()) {
            idle = true;
            break;
        }
    }
    assert!(idle, "story waits idle for its visitor");
    let lines = world.printed();
    let [line] = lines.as_slice() else { panic!("one sign-in line: {lines:?}") };
    assert_eq!(line.process.as_ref(), b"temper serve");
    assert_eq!(line.stream, skein_sim_world::Stream::Output);
    let url = std::str::from_utf8(&line.line).expect("sign-in URL");
    let prefix = format!("http://{}/jig/v1/sign-in/link?secret=", documents::CLIENTS);
    let secret = url.strip_prefix(&prefix).expect("service's sign-in URL");
    assert!(!secret.is_empty() && !secret.contains(['\r', '\n']));
    assert!(
        world
            .processes()
            .iter()
            .all(|process| process.exit().is_none_or(|exit| exit == skein_io::kernel::Exit::Code(0))),
        "no process failed"
    );
    let crate::programs::Process::Service(service) = &world.processes()[0] else { panic!("service first") };
    assert!(service.service.end().is_none(), "service awaits its visitor");
}

fn opening(words: &str, mode: run::Mode) -> run::Opening {
    let limits = smith_agent_world::LIMITS.run;
    let budget = run::Budget { turns: 4, spend: 10, time: Duration::from_secs(60) };
    let charter = run::Charter {
        check_timeout: None,
        mode,
        resume: false,
        waiting: Duration::ZERO,
        instructions: b"Help the person plan their work.".as_slice().into(),
        brief: run::Brief {
            sections: Box::new([run::Section { title: b"Task".as_slice().into(), text: words.as_bytes().into() }]),
        },
        conventions: None,
        grants: charter::Grants {
            contract: 1,
            wait: true,
            deliver: None,
            save: false,
            tools: charter::Tools { inspect: false, modify: false, shell: false },
            agents: false,
            host_tools: Box::new([]),
        },
        outcome: run::outcome::OutcomeSpec {
            change: None,
            verdicts: Box::new([]),
            report: Some(run::outcome::TextSpec { max: 128, fields: Box::new([]) }),
            failure: None,
        },
        budget,
        reserve: charter::Reserve { turns: 0, time: Duration::ZERO },
        llm: charter::Llm {
            effort: None,
            prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
            account: None,
            endpoint: charter::Endpoint(1),
            model: b"fake".as_slice().into(),
            window: 1_048_576,
            output: 512,
            dialect: 3,
        },
        models: Box::new([]),
    };
    let mut domain = run::Domain::new(&limits, 7);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    run::step(
        &mut domain,
        &env,
        run::Event::Start {
            reply_to: ReplyTo::new(Token::new(1)),
            host_run: Token::new(1),
            activation: 1,
            identity: None,
            window: run::Window { turns: 4, bytes: 65_536, largest_turn: 1 },
            messages: Box::new([]),
            charter,
            workspace: None,
            transcript: None,
            resumed: false,
        },
        &mut out,
    );
    let Some(run::Request::Admitted { .. }) = out.pop() else { panic!("run admitted: {out:?}") };
    let Some(run::Request::Open { opening, .. }) = out.pop() else { panic!("main opening: {out:?}") };
    assert!(out.is_empty());
    opening
}

fn story_a_stock() -> (jig_protocol_config::Engine, jig_domain_core::settings::Defaults) {
    let document: Value = serde_json::from_slice(A.deployment).expect("story deployment");
    let engine = documents::settings::configuration(&document);
    let defaults = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    (engine, defaults)
}

fn configured_deployments() -> [(jig_protocol_config::Engine, jig_domain_core::settings::Defaults); 2] {
    [serde_json::from_slice(A.deployment).expect("story deployment"), documents::world_document()].map(|deployment| {
        let engine = documents::settings::configuration(&deployment);
        let temper_service::stock::Given::Own { jig, .. } = documents::given(&deployment) else {
            panic!("the composition supplies its settings")
        };
        (engine, jig)
    })
}

fn playground_owner(defaults: &jig_domain_core::settings::Defaults) -> &jig_domain_authority::Authority {
    let policy = &defaults.policies.get(&1).expect("playground policy").policy;
    &policy.roles.iter().find(|role| role.number == 0).expect("owner role").authority
}
