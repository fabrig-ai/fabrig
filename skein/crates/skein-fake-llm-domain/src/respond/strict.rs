//! Strict coverage failures (testing/fakes/llm.md, 2.5).
use crate::api::{Answer, Error, Failure, Message, Part, Query, Role, Scripts};
use crate::tests::{CONFIG, empty_menu, query, says, script, user};
use crate::{Config, Domain, Event, Request, fire, step};
use alloc::boxed::Box;
use skein_lib::bytes::{concat, copy_of};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

const STRICT: Config = Config { strict: true, ..CONFIG };

fn text(text: &[u8]) -> Part {
    Part::Text { text: copy_of(text) }
}

fn assistant() -> Message {
    Message { role: Role::Assistant, parts: Box::new([text(b"answer")]) }
}

fn call(domain: &mut Domain, query: &Query) -> Result<Answer, Failure> {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: STRICT };
    let token = Token::new(7);
    let mut out = Queue::uncapped();
    step(domain, &env, Event::Call { reply_to: ReplyTo::new(token), call: token, query: query.clone() }, &mut out);
    fire(domain, &env, &mut out);
    let Some(Request::Reply { to, result }) = out.pop() else { panic!("one terminal, before any stream") };
    assert_eq!(to, ReplyTo::new(token));
    assert!(out.is_empty());
    domain.reclaim();
    result
}

fn assert_failure(result: Result<Answer, Failure>, message: &[u8]) {
    let failure = result.expect_err("strict miss fails");
    assert_eq!(failure.error, Error::InvalidRequest);
    assert_eq!(failure.report.code.as_ref(), b"no_script");
    assert!(failure.report.request.is_empty(), "the byte peer supplies its own id");
    assert!(failure.report.field.is_empty());
    assert!(failure.parts.is_empty());
    assert_eq!(failure.message.as_ref(), message);
}

#[test]
fn strict_system_script_fails_past_its_turns_and_again() {
    let mut domain = Domain::configured(
        &STRICT,
        1,
        Box::new([script(b"Plan the widget", Box::new([says(b"first"), says(b"second")]))]),
        empty_menu(),
    )
    .expect("configured scripts");
    let mut input = query(Box::new([user(Box::new([text(b"Build it")]))]));
    input.system = copy_of(b"Plan the widget");
    assert_eq!(call(&mut domain, &input).expect("first turn").parts.as_ref(), &[text(b"first")]);
    input.messages = Box::new([user(Box::new([text(b"Build it")])), assistant(), user(Box::new([text(b"Continue")]))]);
    assert_eq!(call(&mut domain, &input).expect("second turn").parts.as_ref(), &[text(b"second")]);
    input.messages = Box::new([
        user(Box::new([text(b"Build it")])),
        assistant(),
        user(Box::new([text(b"Continue")])),
        assistant(),
        user(Box::new([text(b"Add CSV export")])),
    ]);
    for _ in 0..2_u32 {
        assert_failure(call(&mut domain, &input), b"no script answers: past the 2 turns of the script cued \"Plan the widget\"; last request: \"Add CSV export\"");
    }
}

#[test]
fn strict_request_and_otherwise_scripts_fail_past_their_turns() {
    let rows = [
        (
            Scripts {
                cued: Box::new([script(b"ask", Box::new([says(b"answer")]))]),
                asked: Box::new([]),
                otherwise: Box::new([]),
            },
            b"the script cued \"ask\"".as_slice(),
        ),
        (
            Scripts {
                cued: Box::new([]),
                asked: Box::new([script(b"ask", Box::new([says(b"answer")]))]),
                otherwise: Box::new([]),
            },
            b"the script cued \"ask\"".as_slice(),
        ),
        (
            Scripts { cued: Box::new([]), asked: Box::new([]), otherwise: Box::new([says(b"answer")]) },
            b"otherwise".as_slice(),
        ),
    ];
    for (scripts, source) in rows {
        let mut domain = Domain::played(&STRICT, 1, scripts, empty_menu()).expect("played scripts");
        let input = query(Box::new([user(Box::new([text(b"ask")])), assistant(), user(Box::new([]))]));
        let message = concat(&[b"no script answers: past the 1 turns of ", source, b"; last request: \"ask\""]);
        for _ in 0..2_u32 {
            assert_failure(call(&mut domain, &input), &message);
        }
    }
}

#[test]
fn strict_miss_names_only_the_latest_user_text_unescaped() {
    let rows = [
        (
            Box::<[Message]>::from([
                user(Box::new([text(b"old")])),
                assistant(),
                user(Box::new([
                    text(b"Add \"CSV\""),
                    Part::Opaque { bytes: copy_of(b"ignored") },
                    text(b"export\nnow"),
                ])),
                assistant(),
                user(Box::new([])),
            ]),
            b"Add \"CSV\"\nexport\nnow".as_slice(),
        ),
        (Box::new([user(Box::new([Part::Opaque { bytes: copy_of(b"no text") }]))]), b"".as_slice()),
        (Box::new([user(Box::new([text(b"old")])), user(Box::new([text(b"")]))]), b"".as_slice()),
    ];
    for (messages, text) in rows {
        let mut domain =
            Domain::configured(&STRICT, 1, Box::new([script(b"unmatched", Box::new([says(b"unused")]))]), empty_menu())
                .expect("configured scripts");
        let input = query(messages);
        let message = concat(&[b"no script answers: no cue matches; last request: \"", text, b"\""]);
        assert_failure(call(&mut domain, &input), &message);
    }
}

#[test]
fn strict_request_text_clips_at_512_bytes_on_a_character_boundary() {
    let ascii = [b'x'; 512];
    let before = [b'x'; 511];
    let rows = [
        (Box::<[Part]>::from([text(&ascii)]), copy_of(&ascii)),
        (Box::new([text(&concat(&[&ascii, b"x"]))]), concat(&[&ascii, "…".as_bytes()])),
        (Box::new([text(&concat(&[&before, "é".as_bytes()]))]), concat(&[&before, "…".as_bytes()])),
        (Box::new([text(&before), text(b"")]), concat(&[&before, b"\n"])),
        (Box::new([text(&before), text("é".as_bytes())]), concat(&[&before, b"\n", "…".as_bytes()])),
        (Box::new([text(&ascii), text(b"")]), concat(&[&ascii, "…".as_bytes()])),
    ];
    for (input, expected) in rows {
        let mut domain = Domain::new(&STRICT, 1);
        let input = query(Box::new([user(input)]));
        let message = concat(&[b"no script answers: no cue matches; last request: \"", &expected, b"\""]);
        assert_failure(call(&mut domain, &input), &message);
    }
}
