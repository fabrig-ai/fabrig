//! Fixture selection on smith's fresh Open request (smith's domain/run.md, 3.4).
use crate::support::{Scratch, cluster, github_land};
use serde_json::{Value, json};
use skein_fake_llm_domain::{self as fake, api};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use smith_domain_run::{self as run, charter};
use std::{fs, path::Path};

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
    assert!(matches!(out.pop(), Some(run::Request::Admitted { .. })), "run admitted: {out:?}");
    let Some(run::Request::Open { opening, .. }) = out.pop() else { panic!("main opening: {out:?}") };
    assert!(out.is_empty());
    opening
}

fn answer(value: &Value, words: &str, mode: run::Mode) -> api::Answer {
    // Retain the fixture's selection and turns; only its deployment addresses
    // are replaced for this domain-only request.
    let document = json!({
        "listen":{"socket":"127.0.0.1:0"}, "key":"/fixture/llm.key", "tls":null,
        "models":[{"name":"fake","window":1_048_576,"output":8192}],
        "cued":value["cued"], "asked":value["asked"], "otherwise":value["otherwise"]
    });
    let document = temper_fake::llm::Document::decode(&serde_json::to_vec(&document).expect("fixture JSON"))
        .expect("fake LLM fixture");
    let opening = opening(words, mode);
    if mode == run::Mode::Open {
        assert!(skein_lib::bytes::find(&opening.system, words.as_bytes()).is_none(), "brief belongs to the user");
        assert!(skein_lib::bytes::find(&opening.prompt, words.as_bytes()).is_some(), "whole opening request");
    }
    let mut domain = document.domain(17);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: temper_fake::llm::LIMITS };
    let mut out = Queue::uncapped();
    fake::step(
        &mut domain,
        &env,
        fake::Event::Call {
            reply_to: ReplyTo::new(Token::new(1)),
            call: Token::new(1),
            query: api::Query {
                credential: None,
                caching: api::Caching::Unscoped,
                model: b"fake".as_slice().into(),
                system: opening.system,
                tools: Box::new([]),
                choice: api::ToolChoice::Auto,
                messages: Box::new([api::Message {
                    role: api::Role::User,
                    parts: Box::new([api::Part::Text { text: opening.prompt }]),
                }]),
                max_tokens: 8192,
            },
        },
        &mut out,
    );
    fake::fire(&mut domain, &env, &mut out);
    let Some(fake::Request::Reply { result, .. }) = out.pop() else { panic!("script reply: {out:?}") };
    assert!(out.is_empty());
    result.expect("fixture request accepted")
}

fn calls(answer: &api::Answer, name: &str) {
    assert!(
        answer
            .parts
            .iter()
            .any(|part| matches!(part, api::Part::ToolCall { name: got, .. } if got.as_ref() == name.as_bytes())),
        "expected {name}, got {answer:?}"
    );
}
fn says(answer: &api::Answer, prefix: &str) {
    assert!(
        answer.parts.iter().any(|part| matches!(part, api::Part::Text { text } if text.starts_with(prefix.as_bytes()))),
        "expected {prefix}, got {answer:?}"
    );
}
fn cluster_fixture(name: &str, source: &str, configure: bool) -> Value {
    let scratch = Scratch::new(name);
    let directory = skein_scratch::paths::package().join(source);
    let mut description = cluster::Cluster::description_from(scratch.path(), &directory);
    description.bind("proc.forge.ready.address", "127.0.0.1:12345");
    description.prepare("llm");
    let path = description.dir("llm").join("llm.json");
    if configure {
        github_land::configure_llm(&path);
    }
    let value = read(&path);
    scratch.remove();
    value
}
fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("fixture document")).expect("fixture JSON")
}

#[test]
fn github_chat_fixture_selects_its_delegate_from_the_opening_user_message() {
    let value = cluster_fixture("github_opening", "tests/support/github", false);
    calls(&answer(&value, "@forge_chat Update README", run::Mode::Open), "delegate");
}
#[test]
fn github_landing_fixture_selects_its_change_from_the_opening_user_message() {
    let value = cluster_fixture("github_landing_opening", "tests/support/github", true);
    calls(&answer(&value, "@forge_chat Update README", run::Mode::Open), "change");
}
#[test]
fn cluster_chat_fixtures_select_their_opening_requests() {
    let value = cluster_fixture("cluster_opening", "tests/fixtures/cluster", false);
    for (words, expected) in
        [("cluster-first-chat", "First worker replied."), ("cluster-second-chat", "Second worker replied.")]
    {
        says(&answer(&value, words, run::Mode::Open), expected);
    }
    calls(&answer(&value, "Add a file NOTICE to widget", run::Mode::Open), "delegate");
}
#[test]
fn recorded_walk_chat_fixtures_select_their_opening_requests() {
    let tree = skein_scratch::paths::tree();
    for walk in ["b0", "b1"] {
        let directory = tree.join("tests/real-loop/tests/fixtures").join(walk);
        let words = read(&directory.join("person.json"));
        calls(
            &answer(
                &read(&directory.join("llm.json")),
                words["request"].as_str().expect("person request"),
                run::Mode::Open,
            ),
            "goal",
        );
    }
}
#[test]
fn demo_chat_fixtures_select_their_opening_requests() {
    let value = read(&skein_scratch::paths::tree().join("crates/temper-init/template/temper.json"));
    let value = &value["application"]["demo"]["llm"];
    for (words, prefix) in [
        ("Add CSV export to widget", "I’ll delegate this change"),
        ("Add a dark theme to widget", "I would give widget a dark theme"),
        ("Fix widget's empty search state", "I would have widget say"),
    ] {
        says(&answer(value, words, run::Mode::Open), prefix);
    }
}
#[test]
fn github_coder_fixture_keeps_its_finite_system_cue() {
    let value = cluster_fixture("github_coder_opening", "tests/support/github", false);
    calls(&answer(&value, "@coding Fix", run::Mode::Finite), "read");
}
