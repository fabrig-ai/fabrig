//! Given scripts and strict coverage (protocol/setup/playground.md, document form).
use super::*;

fn decode(value: &Value, scripts: &Value) -> llm::Document {
    llm::Document::decode_with_scripts(
        &serde_json::to_vec(value).expect("document JSON"),
        Some(&serde_json::to_vec(scripts).expect("scripts JSON")),
    )
    .expect("given scripts")
}

#[test]
fn given_scripts_replace_all_document_scripts_before_checks() {
    let mut value = llm_document();
    value["cued"] = json!([{"cue":"Plan a farewell line for hello.","turns":["Farewell."]}]);
    value["asked"] = json!([{"cue":"old request","turns":["old answer"]}]);
    let given = decode(
        &value,
        &json!({
            "cued":[{"cue":"Plan the widget","turns":["Planned."]}],
            "asked":[{"cue":"new request","turns":["Asked."]}],
            "otherwise":["Given fallback."]
        }),
    );
    for (system, request, text) in [
        ("Plan the widget", "", "Planned."),
        ("", "new request", "Asked."),
        ("Plan a farewell line for hello.", "old request", "Given fallback."),
    ] {
        let result = answer(&given, 23, query(system.as_bytes(), request.as_bytes()));
        assert_eq!(result.parts.as_ref(), &[api::Part::Text { text: text.as_bytes().into() }]);
    }
    // Replaced members are not checked; absent replacement members are empty, not inherited.
    value["asked"] = json!("not a script list");
    let given = decode(&value, &json!({"cued":[{"cue":"Plan the widget","turns":["Planned."]}]}));
    assert_eq!(
        answer(&given, 23, query(b"Plan the widget", b"")).parts.as_ref(),
        &[api::Part::Text { text: b"Planned.".as_slice().into() }],
    );
    let miss = call(&given.with_strict(true), query(b"Plan a farewell line for hello.", b"old request"))
        .expect_err("no inherited scripts or fallback");
    assert_eq!(miss.report.code.as_ref(), b"no_script");
}

#[test]
fn given_scripts_use_document_checks_and_allow_empty_otherwise() {
    let mut value = llm_document();
    value["otherwise"] = json!([]);
    llm::Document::decode(&serde_json::to_vec(&value).expect("JSON")).expect("empty document fallback");
    for given in [json!({}), json!({"otherwise":[]})] {
        let document = decode(&value, &given).with_strict(true);
        assert_eq!(
            call(&document, query(b"", b"unmatched")).expect_err("empty fallback").report.code.as_ref(),
            b"no_script"
        );
    }
    for (given, expected) in [
        (json!({"mystery":[]}), "mystery: unknown member\n"),
        (json!({"cued":[{"cue":"x","turns":[]}]}), "cued[0].turns: must be non-empty\n"),
        (
            json!({"asked":[{"cue":"x","turns":["yes"],"inputs":[{"cue":"","turn":1}]}]}),
            "asked[0].inputs[0].turn: must index this script's turns\n",
        ),
        (
            json!({"otherwise":[{"lines":[{"pieces":[{"text":"held","hold":"gate"}]}],"finish":"stop","tokens":1}]}),
            "control: required when a piece has a hold\n",
        ),
    ] {
        let error = llm::Document::decode_with_scripts(
            &serde_json::to_vec(&value).expect("document JSON"),
            Some(&serde_json::to_vec(&given).expect("scripts JSON")),
        )
        .expect_err("document checks apply to given scripts");
        assert_eq!(error.line().as_ref(), expected.as_bytes());
    }
}

fn call(document: &llm::Document, query: api::Query) -> Result<api::Answer, api::Failure> {
    let configuration = llm::configuration(document, b"key", 23);
    let mut domain = configuration.domain;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: configuration.limits };
    let mut out = Queue::uncapped();
    skein_fake_llm_domain::step(
        &mut domain,
        &env,
        skein_fake_llm_domain::Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: Token::new(2), query },
        &mut out,
    );
    skein_fake_llm_domain::fire(&mut domain, &env, &mut out);
    let Some(skein_fake_llm_domain::Request::Reply { result, .. }) = out.pop() else {
        panic!("one whole reply");
    };
    result
}

#[test]
fn strict_document_fails_when_a_script_runs_out() {
    let document = decode(&llm_document(), &json!({"cued":[{"cue":"Plan the widget","turns":["Planned."]}]}));
    let mut query = query(b"Plan the widget", b"Build it");
    call(&document, query.clone()).expect("first scripted turn");
    let mut history = query.messages.into_vec();
    history.push(api::Message {
        role: api::Role::Assistant,
        parts: Box::new([api::Part::Text { text: b"Planned.".as_slice().into() }]),
    });
    history.push(api::Message {
        role: api::Role::User,
        parts: Box::new([api::Part::Text { text: b"Continue".as_slice().into() }]),
    });
    query.messages = history.into_boxed_slice();
    call(&document, query.clone()).expect("strict is off by default");
    let document = document.with_strict(true);
    assert!(document.limits().strict);
    let failure = call(&document, query).expect_err("strict exhausted script");
    assert_eq!(failure.error, api::Error::InvalidRequest);
    assert_eq!(failure.report.code.as_ref(), b"no_script");
    assert!(failure.message.starts_with(b"no script answers: past the 1 turns"));
}
