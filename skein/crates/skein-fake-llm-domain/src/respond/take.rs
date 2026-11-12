//! Conversation takes through the domain entrance (testing/fakes/llm.md, 2.6).
use crate::api::{Answer, Error, Failure, Finish, Line, Message, Part, Piece, Query, Role, Rule, Rules, Scripts, Turn};
use crate::tests::{CONFIG, empty_menu, query, script, user};
use crate::{Config, Domain, Event, Request, fire, step};
use alloc::boxed::Box;
use skein_lib::bytes::{concat, copy_of};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

fn text(text: &[u8]) -> Part {
    Part::Text { text: copy_of(text) }
}
fn line(text: &[u8]) -> Line {
    Line::Text { text: copy_of(text) }
}
fn take(name: &[u8], before: &[u8], after: &[u8]) -> Line {
    Line::Take { name: copy_of(name), before: copy_of(before), after: copy_of(after) }
}
fn turn(lines: Box<[Line]>) -> Turn {
    Turn { lines, finish: Finish::Stop, tokens: 1 }
}
fn scripts(lines: Box<[Line]>) -> Scripts {
    Scripts { cued: Box::new([]), asked: Box::new([]), otherwise: Box::new([turn(lines)]) }
}
fn call_line(arguments: &[u8]) -> Line {
    Line::Call { name: copy_of(b"read_pull"), arguments: copy_of(arguments) }
}
fn call(domain: &mut Domain, config: Config, query: &Query) -> Result<Answer, Failure> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config };
    let token = Token::new(7);
    let mut out = Queue::uncapped();
    step(domain, &env, Event::Call { reply_to: ReplyTo::new(token), call: token, query: query.clone() }, &mut out);
    fire(domain, &env, &mut out);
    let Some(Request::Reply { to, result }) = out.pop() else { panic!("one terminal before streaming") };
    assert_eq!(to, ReplyTo::new(token));
    assert!(out.is_empty());
    domain.reclaim();
    result
}
fn arguments(answer: &Answer) -> &[u8] {
    let Some(Part::ToolCall { name, arguments, .. }) = answer.parts.last() else { panic!("last part is a call") };
    assert_eq!(name.as_ref(), b"read_pull");
    arguments
}
fn tool_call(id: &[u8], arguments: &[u8]) -> Part {
    Part::ToolCall { id: copy_of(id), name: copy_of(b"read_pull"), arguments: copy_of(arguments) }
}
fn output(id: &[u8], output: &[u8]) -> Part {
    Part::ToolOutput { id: copy_of(id), output: copy_of(output), is_error: false }
}
fn assistant(parts: Box<[Part]>) -> Message {
    Message { role: Role::Assistant, parts }
}

#[test]
fn newest_take_searches_messages_parts_occurrences_then_system() {
    let history = Box::new([
        user(Box::new([text(b"request")])),
        assistant(Box::new([tool_call(b"old", b"{}")])),
        user(Box::new([output(b"old", b"pull request #3 opened")])),
        assistant(Box::new([tool_call(b"new", b"{}")])),
        user(Box::new([
            text(b"pull request #5 opened"),
            output(b"new", b"pull request #6 opened; pull request #7 opened; pull request # opened"),
            Part::Opaque { bytes: copy_of(b"pull request #9 opened") },
        ])),
    ]);
    let mut from_system = query(Box::new([user(Box::new([]))]));
    from_system.system = copy_of(b"pull request #8 opened");
    let mut newest = query(history);
    newest.system = copy_of(b"pull request #1 opened");
    for (query, expected) in [(newest, b"{\"number\":7}".as_slice()), (from_system, b"{\"number\":8}")] {
        let mut domain = Domain::played(
            &CONFIG,
            1,
            scripts(Box::new([take(b"pull", b"pull request #", b" opened"), call_line(br#"{"number":"{{#pull}}"}"#)])),
            empty_menu(),
        )
        .expect("take script");
        let answer = call(&mut domain, CONFIG, &query).expect("taken number");
        assert_eq!(answer.parts.len(), 1, "a take emits no part");
        assert_eq!(arguments(&answer), expected);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "test values straddle the capture byte bound")]
fn takes_skip_invalid_values_and_never_read_calls_or_span_parts() {
    let long = [b'x'; 257];
    let text_with_invalid = concat(&[b"<good><><line\nfeed><", &long, b"><unfinished"]);
    let rows = [
        (Box::<[Message]>::from([user(Box::new([text(&text_with_invalid)]))]), b"good".as_slice()),
        (Box::new([user(Box::new([text(b"<split"), text(b"value>")]))]), b"system"),
        (
            Box::new([
                assistant(Box::new([tool_call(b"id", b"<ignored>")])),
                user(Box::new([text(b"request"), output(b"id", b"nothing")])),
            ]),
            b"system",
        ),
    ];
    for (messages, expected) in rows {
        let mut input = query(messages);
        input.system = copy_of(b"<system>");
        let mut domain =
            Domain::played(&CONFIG, 1, scripts(Box::new([take(b"v", b"<", b">"), line(b"{{v}}")])), empty_menu())
                .expect("script");
        assert_eq!(call(&mut domain, CONFIG, &input).expect("taken text").parts.as_ref(), &[text(expected)]);
    }
    for len in [1_usize, 256] {
        let value = vec![b'x'; len];
        let input = query(Box::new([user(Box::new([text(&concat(&[b"<", &value, b">"]))]))]));
        let mut domain =
            Domain::played(&CONFIG, 1, scripts(Box::new([take(b"v", b"<", b">"), line(b"{{v}}")])), empty_menu())
                .expect("script");
        assert_eq!(call(&mut domain, CONFIG, &input).expect("valid value bound").parts.as_ref(), &[text(&value)]);
    }
}

#[test]
fn uses_are_ordered_raw_in_text_and_json_escaped_in_arguments() {
    let value = b"a\"\\\x1f";
    let input = query(Box::new([user(Box::new([text(&concat(&[b"<", value, b"> n=0;"]))]))]));
    let mut domain = Domain::played(
        &CONFIG,
        1,
        scripts(Box::new([
            line(b"{{v}}"),
            call_line(br#"{"before":"{{v}}"}"#),
            take(b"v", b"<", b">"),
            take(b"n", b"n=", b";"),
            line(b"raw {{v}} / {{other}} / {{v}}"),
            Line::Opaque { bytes: copy_of(b"{{v}}") },
            Line::Call { name: copy_of(b"{{v}}"), arguments: copy_of(b"{}") },
            call_line(br#"{"value":"{{v}}","number":"{{#n}}","unknown":"{{other}}"}"#),
        ])),
        empty_menu(),
    )
    .expect("script");
    let answer = call(&mut domain, CONFIG, &input).expect("expanded");
    assert_eq!(answer.parts.first(), Some(&text(b"{{v}}")));
    assert_eq!(answer.parts.get(1), Some(&tool_call(b"call_0000000000000001", br#"{"before":"{{v}}"}"#)));
    assert_eq!(answer.parts.get(2), Some(&text(&concat(&[b"raw ", value, b" / {{other}} / ", value]))));
    assert_eq!(answer.parts.get(3), Some(&Part::Opaque { bytes: copy_of(b"{{v}}") }));
    assert_eq!(
        answer.parts.get(4),
        Some(&Part::ToolCall {
            id: copy_of(b"call_0000000000000002"),
            name: copy_of(b"{{v}}"),
            arguments: copy_of(b"{}")
        })
    );
    assert_eq!(arguments(&answer), br#"{"value":"a\"\\\u001f","number":0,"unknown":"{{other}}"}"#);
}

#[test]
fn no_take_miss_and_non_number_fail_again_with_or_without_strict() {
    for strict in [false, true] {
        let config = Config { strict, ..CONFIG };
        let cases = [
            (
                b"absent".as_slice(),
                b"no value taken for v: \"<\" \xe2\x80\xa6 \">\" is nowhere in the conversation".as_slice(),
            ),
            (b"<01>", b"the value taken for v, \"01\", is no number"),
            (b"<+1>", b"the value taken for v, \"+1\", is no number"),
            (b"<1.0>", b"the value taken for v, \"1.0\", is no number"),
            (b"<123456789012345678901>", b"the value taken for v, \"123456789012345678901\", is no number"),
        ];
        for (system, reason) in cases {
            let mut input = query(Box::new([
                user(Box::new([text(b"old")])),
                user(Box::new([text(b"last \"request\""), text(b"next\nline")])),
            ]));
            input.system = copy_of(system);
            let mut domain = Domain::played(
                &config,
                1,
                scripts(Box::new([line(b"not emitted"), take(b"v", b"<", b">"), call_line(br#"{"n":"{{#v}}"}"#)])),
                empty_menu(),
            )
            .expect("script");
            for _ in 0..2_u32 {
                let failure = call(&mut domain, config, &input).expect_err("take fails");
                assert_eq!(failure.error, Error::InvalidRequest);
                assert_eq!(failure.report.code.as_ref(), b"no_take");
                assert!(failure.report.request.is_empty());
                assert!(failure.report.field.is_empty());
                assert!(failure.parts.is_empty(), "no earlier line is emitted");
                assert_eq!(failure.message, concat(&[reason, b"; last request: \"last \"request\"\nnext\nline\""]));
            }
        }
    }
    let input = query(Box::new([user(Box::new([text(b"<12345678901234567890>")]))]));
    let mut domain = Domain::played(
        &CONFIG,
        1,
        scripts(Box::new([take(b"v", b"<", b">"), call_line(br#"{"n":"{{#v}}"}"#)])),
        empty_menu(),
    )
    .expect("script");
    assert_eq!(arguments(&call(&mut domain, CONFIG, &input).expect("twenty digits")), b"{\"n\":12345678901234567890}");
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary test builds refused line lists across admission bounds"
)]
fn take_admission_checks_names_markers_count_duplicates_and_pieces() {
    let valid = vec![take(b"aZ_09", &[b'x'; 256], &[b'y'; 256]), take(&[b'n'; 32], b"<", b">")];
    Domain::played(&CONFIG, 1, scripts(valid.into_boxed_slice()), empty_menu()).expect("inclusive bounds");
    let mut eight = Vec::new();
    for name in b'a'..=b'h' {
        eight.push(take(&[name], b"<", b">"));
    }
    Domain::played(&CONFIG, 1, scripts(eight.clone().into_boxed_slice()), empty_menu()).expect("eight takes");
    let mut nine = eight;
    nine.push(take(b"i", b"<", b">"));
    let mut cases = vec![
        nine,
        vec![take(b"v", b"<", b">"), take(b"v", b"[", b"]")],
        vec![
            Line::Pieces { pieces: Box::new([Piece { text: copy_of(b"piece"), hold: None }]) },
            take(b"v", b"<", b">"),
        ],
    ];
    for name in [b"".as_slice(), &[b'n'; 33], b"a-b", "é".as_bytes()] {
        cases.push(vec![take(name, b"<", b">")]);
    }
    for marker in [b"".as_slice(), &[b'x'; 257]] {
        cases.push(vec![take(b"v", marker, b">")]);
        cases.push(vec![take(b"v", b"<", marker)]);
    }
    for lines in cases {
        assert_eq!(
            Domain::played(&CONFIG, 1, scripts(lines.into_boxed_slice()), empty_menu()).err(),
            Some(Error::InvalidRequest)
        );
    }
}

#[test]
fn take_bytes_share_the_script_cap() {
    let scripts = scripts(Box::new([take(b"v", b"<", b">")]));
    // No menu, rules or cue payload; fallback is retained as one Script.
    let held = size_of::<crate::api::Script>()
        .checked_add(size_of::<Turn>())
        .expect("turn wrapper")
        .checked_add(size_of::<Line>())
        .expect("line wrapper")
        .checked_add(3)
        .expect("name and two markers");
    let held = u64::try_from(held).expect("small fixture");
    for (cap, fits) in [(held, true), (held.checked_sub(1).expect("nonempty script"), false)] {
        let config = Config { script_bytes: u32::try_from(cap).expect("small fixture"), ..CONFIG };
        assert_eq!(Domain::played(&config, 1, scripts.clone(), empty_menu()).is_ok(), fits);
    }
}

#[test]
fn expanded_answer_is_bounded_before_calls_are_copied() {
    let lines = Box::new([call_line(b"{}"), take(b"v", b"<", b">"), call_line(br#"{"v":"{{v}}"}"#)]);
    let script = scripts(lines);
    let mut short = query(Box::new([user(Box::new([text(b"<x>")]))]));
    let bound = size_of::<Part>()
        .checked_mul(2)
        .expect("two call wrappers")
        .checked_add(42)
        .expect("two generated ids")
        .checked_add(18)
        .expect("two read_pull names")
        .checked_add(11)
        .expect("empty object and expanded x argument");
    let config = Config { answer_bytes: u32::try_from(bound).expect("small answer"), ..CONFIG };
    let mut domain = Domain::played(&config, 1, script, empty_menu()).expect("script");
    let large = concat(&[b"<", &[0x1f; 64], b">"]);
    short.messages = Box::new([user(Box::new([text(&large)]))]);
    assert_eq!(call(&mut domain, config, &short).expect_err("sixfold escaped expansion").error, Error::ContextTooLong);
    short.messages = Box::new([user(Box::new([text(b"<x>")]))]);
    let answer = call(&mut domain, config, &short).expect("exactly at bound");
    assert_eq!(
        answer.parts.first(),
        Some(&tool_call(b"call_0000000000000001", b"{}")),
        "refused expansion minted no call"
    );
    assert_eq!(arguments(&answer), b"{\"v\":\"x\"}");
}

#[test]
fn input_selected_turn_plays_its_takes() {
    let scripts = Scripts {
        cued: Box::new([script(
            b"plan",
            Box::new([turn(Box::new([line(b"wrong")])), turn(Box::new([take(b"v", b"<", b">"), line(b"{{v}}")]))]),
        )]),
        asked: Box::new([]),
        otherwise: Box::new([]),
    };
    let rules = Rules {
        cued: Box::new([Box::new([Rule { cue: copy_of(b"ready"), turn: 1 }, Rule { cue: copy_of(b""), turn: 0 }])]),
        asked: Box::new([]),
    };
    let mut domain = Domain::played_by_input(&CONFIG, 1, scripts, empty_menu(), rules).expect("input-selected script");
    let mut input = query(Box::new([user(Box::new([text(b"ready <taken>")]))]));
    input.system = copy_of(b"plan");
    assert_eq!(call(&mut domain, CONFIG, &input).expect("selected turn").parts.as_ref(), &[text(b"taken")]);
}
