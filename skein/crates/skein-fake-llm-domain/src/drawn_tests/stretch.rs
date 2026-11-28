//! Drawn stretches of a scripted conversation (skein/docs/design/testing/fakes/drawn-agent.md, 5).

use super::{NAMED, SEEDS, ask, drawn, observe, reply, rules, script, scripts, turn, user};
use crate::api::{
    Capture, Error, Finish, Line, Menu, Message, Part, Query, Role, Rule, Scripts, ToolChoice, ToolSpec, Turn,
};
use crate::{Config, Domain};
use skein_lib::{List, bytes::copy_of};

const ROUNDS: u32 = 2;
const CONFIG: Config = Config { tool_rounds: 0, ..NAMED };
const BODY: &[u8] = br#"{"body":1}"#;
const OPENING: &[u8] = b"open the story";

fn tool(name: &[u8]) -> ToolSpec {
    ToolSpec { name: copy_of(name), description: Box::new([]), parameters: copy_of(b"{}") }
}

fn request() -> Query {
    Query { tools: Box::new([tool(b"read"), tool(b"write")]), ..ask(b"", OPENING) }
}

fn call(name: &[u8]) -> Turn {
    Turn {
        lines: Box::new([Line::Call { name: copy_of(name), arguments: copy_of(b"{}") }]),
        finish: Finish::ToolCalls,
        tokens: 8,
    }
}

fn stretch(rounds: u32, names: &[&[u8]]) -> Turn {
    let mut tools = List::with_capacity(u32::try_from(names.len()).expect("fixture names"));
    for name in names {
        tools.push(copy_of(name)).expect("room for all names");
    }
    Turn { lines: Box::new([Line::Drawn { rounds, tools: tools.into_boxed() }]), finish: Finish::ToolCalls, tokens: 0 }
}

fn cued(cue: &[u8], turns: Box<[Turn]>) -> Scripts {
    Scripts { cued: Box::new([script(cue, turns)]), ..scripts() }
}

fn story(names: &[&[u8]]) -> Scripts {
    cued(b"", Box::new([call(b"a"), stretch(ROUNDS, names), call(b"b")]))
}

fn menu(body: &[u8]) -> Menu {
    Menu { arguments: Box::new([copy_of(body)]), invalid: Box::new([]), slots: Box::new([]) }
}

fn domain(config: &Config, scripts: Scripts, body: &[u8], capture: Capture) -> Domain {
    Domain::drawn(config, SEEDS, scripts, menu(body), rules(), capture).expect("admitted story")
}

fn append(query: &mut Query, parts: Box<[Part]>) {
    let mut outputs = List::with_capacity(u32::try_from(parts.len()).expect("fixture parts"));
    for part in &parts {
        let Part::ToolCall { id, .. } = part else { panic!("a tool reply") };
        let part = Part::ToolOutput { id: id.clone(), output: copy_of(b"result"), is_error: false };
        outputs.push(part).expect("room for each output");
    }
    let mut messages = List::uncapped();
    for message in core::mem::replace(&mut query.messages, Box::new([])) {
        messages.push(message).expect("uncapped fixture messages");
    }
    messages.push(Message { role: Role::Assistant, parts }).expect("uncapped fixture messages");
    messages.push(user(outputs.into_boxed())).expect("uncapped fixture messages");
    query.messages = messages.into_boxed();
}

fn after_a(domain: &mut Domain, config: &Config) -> Query {
    let mut request = request();
    let a = reply(domain, config, &request);
    append(&mut request, a.parts);
    request
}

fn assert_calls(answer: &crate::api::Answer, name: &[u8]) {
    assert_eq!(answer.finish, Finish::ToolCalls);
    assert!(!answer.parts.is_empty());
    for part in &answer.parts {
        let Part::ToolCall { name: called, .. } = part else { panic!("a call to {name:?}") };
        assert_eq!(called.as_ref(), name);
    }
}

#[test]
fn stretch_spans_two_turns_and_selects_only_named_tools() {
    let mut domain = domain(&CONFIG, story(&[b"read"]), BODY, Capture::none());
    let mut request = request();
    for name in [b"a".as_slice(), b"read", b"read", b"b"] {
        let answer = reply(&mut domain, &CONFIG, &request);
        assert_calls(&answer, name);
        append(&mut request, answer.parts);
    }
    let end = reply(&mut domain, &CONFIG, &request);
    assert_eq!(end.parts.as_ref(), &[Part::Text { text: copy_of(b"done") }]);
    assert_eq!(end.finish, Finish::Stop);
}

fn assert_no_script(result: Result<crate::api::Answer, crate::api::Failure>, message: &[u8]) {
    let error = result.expect_err("the stretch cannot draw");
    assert_eq!(error.error, Error::InvalidRequest);
    assert_eq!(error.report.code.as_ref(), b"no_script");
    assert_eq!(error.message.as_ref(), message);
    assert!(error.parts.is_empty());
}

#[test]
fn stretch_without_named_offers_fails_at_each_round() {
    let scripts = cued(b"stage", Box::new([call(b"a"), stretch(ROUNDS, &[b"not offered"]), call(b"b")]));
    let mut domain = domain(&CONFIG, scripts, BODY, Capture::none());
    let mut request = Query { system: copy_of(b"stage"), ..request() };
    for round in 1..=ROUNDS {
        // A conversation with a previous assistant call for each preceding turn.
        let answer = Part::ToolCall {
            id: Box::new([u8::try_from(round).expect("fixture rounds")]),
            name: copy_of(b"a"),
            arguments: copy_of(b"{}"),
        };
        append(&mut request, Box::new([answer]));
        assert_no_script(
            observe(&mut domain, &CONFIG, &request, 0).1,
            b"no script answers: the stretch of the script cued \"stage\" offers none of its tools; last request: \"open the story\"",
        );
    }
    assert_eq!(domain.capture(), Capture::none());
}

#[test]
fn stretch_without_a_body_names_otherwise_in_the_failure() {
    let scripts = Scripts { otherwise: Box::new([stretch(ROUNDS, &[b"read"])]), ..scripts() };
    let empty_menu = Menu { arguments: Box::new([]), ..menu(BODY) };
    let mut domain = Domain::played(&CONFIG, 17, scripts, empty_menu).expect("admitted stretch");
    assert_no_script(
        observe(&mut domain, &CONFIG, &request(), 0).1,
        b"no script answers: the stretch of otherwise offers none of its tools; last request: \"open the story\"",
    );
}

#[test]
fn stretch_applies_outside_choice_within_its_tool_pool() {
    let scripts = cued(b"", Box::new([stretch(ROUNDS, &[b"read", b"write"])]));
    let request = Query {
        choice: ToolChoice::Only(Box::new([copy_of(b"read")])),
        tools: Box::new([tool(b"read"), tool(b"write"), tool(b"unrelated")]),
        ..request()
    };
    for (outside_choice, expected) in [(0, b"read".as_slice()), (1000, b"write")] {
        let config = Config { outside_choice, ..CONFIG };
        let mut domain = domain(&config, scripts.clone(), BODY, Capture::none());
        assert_calls(&reply(&mut domain, &config, &request), expected);
    }
}

#[test]
fn provider_failure_leaves_a_stretch_rounds_draw_unchanged() {
    let mut reference = domain(&CONFIG, story(&[b"read"]), BODY, Capture::none());
    let mut retried = domain(&CONFIG, story(&[b"read"]), BODY, Capture::none());
    let request = after_a(&mut reference, &CONFIG);
    let retry = after_a(&mut retried, &CONFIG);
    let failed = Config { overloaded: 1000, ..CONFIG };
    assert_eq!(observe(&mut retried, &failed, &retry, 0).1, Err(Error::Overloaded.into()));
    assert_eq!(retried.capture(), Capture::none());
    assert_eq!(reply(&mut retried, &CONFIG, &retry), reply(&mut reference, &CONFIG, &request));
    let capture = retried.capture();
    assert_eq!(capture, reference.capture());
    assert_eq!(drawn(&capture, 0).key.attempt, 0);
}

fn record_rounds(config: &Config) -> (Query, Query, Capture) {
    let mut recording = domain(config, story(&[b"read"]), BODY, Capture::none());
    let first = after_a(&mut recording, config);
    let mut second = first.clone();
    append(&mut second, reply(&mut recording, config, &first).parts);
    let _second = reply(&mut recording, config, &second);
    (first, second, recording.capture())
}

#[test]
fn stretch_rounds_replay_under_strict_by_their_keys() {
    let config = Config { strict: true, ..CONFIG };
    let (first, second, capture) = record_rounds(&config);
    let mut replay = domain(&config, story(&[b"read"]), b"{}", capture.clone());
    for (index, query) in [(1, second), (0, first)] {
        assert_eq!(turn(&reply(&mut replay, &config, &query)), drawn(&capture, index).answer);
    }
}

#[test]
fn strict_draws_a_stretch_key_missing_from_capture() {
    let config = Config { strict: true, ..CONFIG };
    let (_, second, mut capture) = record_rounds(&config);
    capture.answers = Box::new([drawn(&capture, 0).clone()]);
    let mut replay = domain(&config, story(&[b"read"]), b"{}", capture);
    let answer = reply(&mut replay, &config, &second);
    assert_calls(&answer, b"read");
    for part in &answer.parts {
        let Part::ToolCall { arguments, .. } = part else { panic!("a drawn call") };
        assert_eq!(arguments.as_ref(), b"{}");
    }
}

#[test]
fn input_rules_cannot_name_a_stretch_turn() {
    let scripts = story(&[b"read"]);
    for (turn, expected) in [(1, Some(Error::InvalidRequest)), (2, None)] {
        let rules = crate::api::Rules { cued: Box::new([Box::new([Rule { cue: Box::new([]), turn }])]), ..rules() };
        assert_eq!(Domain::played_by_input(&CONFIG, 17, scripts.clone(), menu(BODY), rules).err(), expected);
    }
}

#[test]
fn malformed_stretches_are_refused_at_configuration() {
    const MOST_ROUNDS: u32 = 64; // drawn-agent.md, 5
    const MOST_TOOLS: usize = 8;
    const MOST_NAME_BYTES: usize = 64;
    let long_name = [b'x'; MOST_NAME_BYTES + 1];
    let extra_tools = [b"a".as_slice(), b"b", b"c", b"d", b"e", b"f", b"g", b"h", b"i"];
    let legal = stretch(1, &[b"read"]);
    let mixed = Turn {
        lines: Box::new([
            Line::Text { text: copy_of(b"also says") },
            legal.lines.first().expect("stretch line").clone(),
        ]),
        ..legal.clone()
    };
    for (label, invalid) in [
        ("zero rounds", stretch(0, &[b"read"])),
        ("too many rounds", stretch(MOST_ROUNDS + 1, &[b"read"])),
        ("no tool names", stretch(1, &[])),
        ("too many names", stretch(1, &extra_tools)),
        ("duplicate name", stretch(1, &[b"read", b"read"])),
        ("empty name", stretch(1, &[b""])),
        ("long name", stretch(1, &[&long_name])),
        ("another line", mixed),
        ("wrong finish", Turn { finish: Finish::Stop, ..legal.clone() }),
        ("nonzero tokens", Turn { tokens: 1, ..legal }),
    ] {
        assert_eq!(
            Domain::played(&CONFIG, 17, cued(b"", Box::new([invalid])), menu(BODY)).err(),
            Some(Error::InvalidRequest),
            "{label}"
        );
    }
    let longest = [b'x'; MOST_NAME_BYTES];
    let most_tools: [&[u8]; MOST_TOOLS] = [b"a", b"b", b"c", b"d", b"e", b"f", b"g", &longest];
    let _admitted = domain(&CONFIG, cued(b"", Box::new([stretch(MOST_ROUNDS, &most_tools)])), BODY, Capture::none());
    // A one-round stretch and a one-byte name are also valid at their lower bounds.
    let _admitted = domain(&CONFIG, cued(b"", Box::new([stretch(1, &[b"a"])])), BODY, Capture::none());
}

#[test]
fn stretch_names_charge_the_script_byte_cap() {
    let name = b"read";
    let scripts = cued(b"", Box::new([stretch(ROUNDS, &[name])]));
    let menu = menu(BODY);
    // One script with no cue, one turn with one line and one owned tool name; one menu body.
    let bytes = size_of::<crate::api::Script>()
        + size_of::<Turn>()
        + size_of::<Line>()
        + 2 * size_of::<Box<[u8]>>()
        + name.len()
        + BODY.len();
    let cap = u32::try_from(bytes).expect("small fixture");
    for (script_bytes, expected) in [(cap, None), (cap - 1, Some(Error::ContextTooLong))] {
        let config = Config { script_bytes, ..crate::tests::CONFIG };
        assert_eq!(Domain::played(&config, 17, scripts.clone(), menu.clone()).err(), expected);
    }
}
