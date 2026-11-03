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
const CODING: &str = "Read only what the change needs, then write it in its files.\n\n## Task\n\nExport widget's table as CSV\n[Change connector 1, kind 2, at most 4096 bytes]\n[Requested by task 2]\n";
const REVIEW: &str = "Review the change with read-only authority.\n\n## Task\n\nReview this change at the named head\nChange words:\nExport widget's table as CSV\n[Requested by task 2]\n";
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
    assert_eq!(deployment["engine"]["llm"]["models"][0]["name"], std::str::from_utf8(&model.name).expect("model name"));
    assert_eq!(deployment["engine"]["llm"]["models"][0]["window"], model.window);
    assert_eq!(deployment["engine"]["llm"]["models"][0]["output"], model.output);
    assert_eq!(all().len(), 1);
    assert_eq!(all()[0].name, "a");
    assert_eq!(all()[0].deployment, A.deployment);
    assert_eq!(A.description, DESCRIPTION);
}

#[test]
fn story_as_scripts_are_the_templates() {
    let story: Value = serde_json::from_slice(A.llm).expect("story LLM");
    let template: Value =
        serde_json::from_slice(include_bytes!("../../../temper-init/template/temper.json")).expect("template");
    for scripts in ["asked", "cued"] {
        assert_eq!(story[scripts], template["application"]["demo"]["llm"][scripts], "{scripts}");
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
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge::LIMITS };
    let head =
        domain::advance(&mut forge.domain, &env, b"playground/widget", b"main", b"ci/result", b"green", forge::OWNER)
            .expect("seeded repository");
    let due = Time::ZERO.saturating_add(Duration::from_millis(4000));
    let mut out = Queue::with_capacity(128);
    env.now = Time::ZERO.saturating_add(Duration::from_millis(3999));
    domain::fire(&mut forge.domain, &env, &mut out);
    let f::Answer::Statuses(statuses) = forge
        .domain
        .inspect(&forge::LIMITS, b"playground/widget", &f::Read::Statuses { commit: head })
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
        .inspect(&forge::LIMITS, b"playground/widget", &f::Read::Statuses { commit: head })
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
        mode,
        resume: false,
        waiting: Duration::ZERO,
        instructions: b"Help the person plan their work.".as_slice().into(),
        brief: run::Brief {
            sections: Box::new([run::Section { title: b"Task".as_slice().into(), text: words.as_bytes().into() }]),
        },
        conventions: None,
        grants: charter::Grants {
            wait: true,
            deliver: None,
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
