//! Named draws and answer capture (skein/docs/design/testing/fakes/drawn-agent.md, 3–4).
use alloc::boxed::Box;

use skein_lib::{Duration, Env, List, Queue, ReplyTo, Time, Token, Wall, bytes::copy_of};

use crate::api::{
    Answer, Caching, Capture, Cut, Drawn, Error, Failure, Filled, Finish, Key, Line, Message, Part, Query, Role, Rules,
    Scripts, Seeds, Source, Strike, ToolChoice, Turn, Usage,
};
use crate::tests::{CONFIG, query, says, script, user};
use crate::{Config, Domain, Event, Request, fire, step};

fn text(text: &[u8]) -> Part {
    Part::Text { text: copy_of(text) }
}

fn ask(system: &[u8], opening: &[u8]) -> Query {
    Query { system: copy_of(system), ..query(Box::new([user(Box::new([text(opening)]))])) }
}

fn next_turn(query: &Query) -> Query {
    Query {
        messages: Box::new([
            query.messages.first().expect("opening").clone(),
            Message { role: Role::Assistant, parts: Box::new([text(b"previous")]) },
            user(Box::new([text(b"next")])),
        ]),
        ..query.clone()
    }
}

fn menu() -> crate::api::Menu {
    crate::api::Menu { arguments: Box::new([copy_of(b"{\"n\":1}"), copy_of(b"{\"n\":2}")]), invalid: Box::new([]) }
}

fn observe(domain: &mut Domain, config: &Config, query: &Query, now: u64) -> (Time, Result<Answer, Failure>) {
    let env = Env { now: Time::from_nanos(now), wall: Wall::EPOCH, limits: *config };
    let mut out = Queue::uncapped();
    let token = Token::new(1);
    step(domain, &env, Event::Call { reply_to: ReplyTo::new(token), call: token, query: query.clone() }, &mut out);
    assert!(out.pop().is_none());
    let deadline = domain.next_deadline().expect("reply is scheduled");
    fire(domain, &Env { now: deadline, ..env }, &mut out);
    let Some(Request::Reply { result, .. }) = out.pop() else { panic!("a whole reply") };
    assert!(out.pop().is_none());
    domain.reclaim();
    (deadline, result)
}

#[test]
fn one_seed_keeps_the_original_draws() {
    let config = Config {
        latency_min: Duration::from_nanos(4),
        latency_max: Duration::from_nanos(40),
        overloaded: 200,
        refused: 100,
        cut: 300,
        unscoped_reads: 500,
        calls_per_answer: 3,
        tool_rounds: 2,
        ..CONFIG
    };
    let scripts = Scripts {
        cued: Box::new([script(b"cued", Box::new([says(b"first"), says(b"second")]))]),
        asked: Box::new([]),
        otherwise: Box::new([]),
    };
    let providers = [
        Domain::configured(&config, 1, scripts.cued.clone(), menu()),
        Domain::played(&config, 1, scripts.clone(), menu()),
        Domain::played_by_input(&config, 1, scripts.clone(), menu(), rules()),
        Domain::drawn(&config, Seeds::One(1), scripts, menu(), rules(), Capture::none()),
    ];
    let cued = ask(b"cued", b"open the script");
    let random = ask(b"", b"random");
    for provider in providers {
        assert_golden(&mut provider.expect("golden provider"), &config, &cued, &random);
    }
}

fn assert_golden(domain: &mut Domain, config: &Config, cued: &Query, random: &Query) {
    let call = |id: &[u8], arguments: &[u8]| Part::ToolCall {
        id: copy_of(id),
        name: copy_of(b"ls"),
        arguments: copy_of(arguments),
    };
    let reply = |part, finish, output, cache_write| {
        Ok(Answer {
            parts: Box::new([part]),
            finish,
            usage: Usage {
                input: Some(0),
                cache_read: Some(0),
                cache_write: Some(cache_write),
                output: Some(output),
                reasoning: Some(0),
            },
        })
    };
    let mut cut = Failure::from(Error::Cut(Cut::MidCall));
    cut.parts = Box::new([call(b"call_0000000000000002", b"{\"n")]);
    let second = next_turn(cued);
    for (now, request, deadline, expected) in [
        (0, cued, 20, reply(text(b"first"), Finish::Stop, 2, 5)),
        (100, random, 120, reply(call(b"call_0000000000000001", b"{\"n\":2}"), Finish::ToolCalls, 8, 2)),
        (200, &second, 234, Err(Error::Overloaded.into())),
        (300, random, 330, Err(cut)),
        (400, cued, 426, Err(Error::Overloaded.into())),
        (500, random, 535, reply(call(b"call_0000000000000003", b"{\"n\":2}"), Finish::ToolCalls, 8, 2)),
        (600, &second, 635, reply(text(b"second"), Finish::Stop, 2, 8)),
    ] {
        assert_eq!(observe(domain, config, request, now), (Time::from_nanos(deadline), expected));
    }
}

const SEEDS: Seeds = Seeds::Parts { agent: 17, faults: 3, scheduling: 19 };
const NAMED: Config = Config {
    conversations: 4,
    capture_answers: 8,
    capture_bytes: 4096,
    calls_per_answer: 3,
    answer_tokens: 9,
    ..CONFIG
};

fn scripts() -> Scripts {
    Scripts { cued: Box::new([]), asked: Box::new([]), otherwise: Box::new([]) }
}

fn rules() -> Rules {
    Rules { cued: Box::new([]), asked: Box::new([]) }
}

fn named(config: &Config) -> Domain {
    Domain::drawn(config, SEEDS, scripts(), menu(), rules(), Capture::none()).expect("named provider")
}

fn reply(domain: &mut Domain, config: &Config, request: &Query) -> Answer {
    observe(domain, config, request, 0).1.expect("request answers")
}

fn turn(answer: &Answer) -> Turn {
    let mut lines = List::with_capacity(u32::try_from(answer.parts.len()).expect("bounded reply"));
    for part in &answer.parts {
        lines
            .push(match part {
                Part::Text { text } => Line::Text { text: text.clone() },
                Part::ToolCall { name, arguments, .. } => {
                    Line::Call { name: name.clone(), arguments: arguments.clone() }
                }
                Part::ToolOutput { .. } | Part::Opaque { .. } => panic!("not a drawn answer"),
            })
            .expect("room for each line");
    }
    Turn { lines: lines.into_boxed(), finish: answer.finish, tokens: answer.usage.output.expect("output usage") }
}

#[test]
fn identical_parts_repeat_answers_faults_and_fire_times() {
    let config = Config {
        overloaded: 300,
        latency_min: Duration::from_nanos(4),
        latency_max: Duration::from_nanos(40),
        ..NAMED
    };
    let a = ask(b"", b"a");
    let b = ask(b"", b"b");
    let mut left = named(&config);
    let mut right = named(&config);
    let other_schedule = Seeds::Parts { agent: 17, faults: 3, scheduling: 29 };
    let mut rescheduled = Domain::drawn(&config, other_schedule, scripts(), menu(), rules(), Capture::none())
        .expect("rescheduled provider");
    let mut saw_failure = false;
    let mut saw_success = false;
    let mut first_fire = None;
    let mut reseeded_latency = false;
    let mut different_latency = false;
    for request in [&a, &b, &a, &b, &next_turn(&a), &a] {
        let observed = observe(&mut left, &config, request, 0);
        assert_eq!(observed, observe(&mut right, &config, request, 0));
        let (deadline, result) = observe(&mut rescheduled, &config, request, 0);
        assert_eq!(observed.1, result, "scheduling cannot change an answer or a fault");
        reseeded_latency |= observed.0 != deadline;
        if let Some(first) = first_fire {
            different_latency |= first != observed.0;
        } else {
            first_fire = Some(observed.0);
        }
        saw_success |= observed.1.is_ok();
        saw_failure |= observed.1.is_err();
    }
    assert!(saw_success && saw_failure && different_latency && reseeded_latency);
}

#[test]
fn seeded_failures_and_account_strikes_leave_the_answer_unchanged() {
    let clean = Config { accounts: 2, ..NAMED };
    let failing = Config { overloaded: 300, ..clean };
    let a = Query { credential: Some(0), ..ask(b"", b"a") };
    let b = Query { credential: Some(1), ..ask(b"", b"b") };
    let mut reference = named(&clean);
    let mut seeded = named(&failing);
    let mut struck = named(&clean);
    struck.strike(0, Strike::SignedOut, Time::from_nanos(0)).expect("account strike fits");
    assert_eq!(observe(&mut struck, &clean, &a, 0).1, Err(Error::SignedOut.into()));
    assert_eq!(struck.capture(), Capture::none());
    struck.clear(0, &Strike::SignedOut);
    let mut failures = 0_u32;
    for request in [&a, &b, &next_turn(&a), &next_turn(&b)] {
        let expected = reply(&mut reference, &clean, request);
        assert_eq!(reply(&mut struck, &clean, request), expected);
        let mut answered = false;
        for _ in 0_u32..8 {
            match observe(&mut seeded, &failing, request, 0).1 {
                Ok(answer) => {
                    assert_eq!(answer, expected);
                    answered = true;
                    break;
                }
                Err(failure) => {
                    assert_eq!(failure.error, Error::Overloaded);
                    failures = failures.checked_add(1).expect("bounded fixture");
                }
            }
        }
        assert!(answered, "a retry must reach the answer");
    }
    assert!(failures > 0);
    assert_eq!(struck.capture(), reference.capture());
    assert_eq!(seeded.capture(), reference.capture());
}

#[test]
fn reversing_two_conversations_preserves_each_draw() {
    let a = ask(b"", b"a");
    let b = ask(b"", b"b");
    let mut forward = named(&NAMED);
    let first_a = turn(&reply(&mut forward, &NAMED, &a));
    let first_b = turn(&reply(&mut forward, &NAMED, &b));
    assert_ne!(first_a, first_b, "this seed distinguishes the two conversation keys");
    let mut backward = named(&NAMED);
    assert_eq!(turn(&reply(&mut backward, &NAMED, &b)), first_b);
    assert_eq!(turn(&reply(&mut backward, &NAMED, &a)), first_a);
}

fn drawn(capture: &Capture, index: usize) -> &Drawn {
    capture.answers.get(index).expect("captured answer")
}

fn text_only(query: Query) -> Query {
    Query { choice: ToolChoice::None, ..query }
}

#[test]
fn attempts_count_draws_and_reset_at_turns_and_eviction() {
    let a = text_only(ask(b"s", b"t"));
    let q = text_only(ask(b"s", b"q"));
    let mut domain = named(&NAMED);
    let fail = Config { overloaded: 1000, ..NAMED };
    let cut = Config { cut: 1000, ..NAMED };
    assert_eq!(observe(&mut domain, &fail, &a, 0).1, Err(Error::Overloaded.into()));
    assert_eq!(domain.capture(), Capture::none());
    assert_eq!(observe(&mut domain, &cut, &a, 0).1.expect_err("cut answer").error, Error::Cut(Cut::BeforeOutput));
    // Agent 17 draws 2 tokens at attempt 0 and 3 at attempt 1 for this key.
    assert_eq!(reply(&mut domain, &NAMED, &a).usage.output, Some(3));
    let _next = reply(&mut domain, &NAMED, &next_turn(&a));
    assert_eq!(reply(&mut domain, &NAMED, &a).usage.output, Some(2));
    let captured = domain.capture();
    assert_eq!(drawn(&captured, 0).key.attempt, 0);
    assert_eq!(drawn(&captured, 0).answer.tokens, 2);
    assert_eq!(drawn(&captured, 1).key.attempt, 1);
    assert_eq!(drawn(&captured, 2).key.turn, 1);
    assert_eq!(drawn(&captured, 2).key.attempt, 0);
    assert_eq!(drawn(&captured, 3).key, drawn(&captured, 0).key);

    let zero = Config { conversations: 0, ..NAMED };
    let mut domain = named(&zero);
    assert_eq!(reply(&mut domain, &zero, &a), reply(&mut domain, &zero, &a));
    let captured = domain.capture();
    assert_eq!(drawn(&captured, 0), drawn(&captured, 1));

    let one = Config { conversations: 1, ..NAMED };
    let mut domain = named(&one);
    for request in [&a, &q, &a, &q] {
        let _answer = reply(&mut domain, &one, request);
    }
    let captured = domain.capture();
    assert_eq!(drawn(&captured, 0).key.attempt, 0);
    assert_eq!(drawn(&captured, 1).key.attempt, 0);
    assert_eq!(drawn(&captured, 2), drawn(&captured, 0));
    assert_eq!(drawn(&captured, 3), drawn(&captured, 1));

    let two = Config { conversations: 2, ..NAMED };
    let mut domain = named(&two);
    let c = text_only(ask(b"s", b"c"));
    for request in [&a, &q, &a, &c, &a, &q] {
        let _answer = reply(&mut domain, &two, request);
    }
    for (answer, attempt) in domain.capture().answers.iter().zip([0, 0, 1, 0, 2, 0]) {
        assert_eq!(answer.key.attempt, attempt, "the least recently asked conversation is evicted");
    }
}

#[test]
fn only_the_framed_opening_names_the_conversation() {
    let a = ask(b"s", b"t");
    let scoped = Query { caching: Caching::Scope([0; 16]), ..a.clone() };
    let other_system = ask(b"q", b"t");
    let mut domain = named(&NAMED);
    for request in [&a, &scoped, &other_system, &next_turn(&a)] {
        let _answer = reply(&mut domain, &NAMED, request);
    }
    let captured = domain.capture();
    let first = drawn(&captured, 0).key;
    // Start at the FNV basis and fold 01 00 00 00 00 00 00 00 73, then
    // 01 00 00 00 00 00 00 00 74. There is no length field for an absent scope.
    assert_eq!(first, Key { conversation: 0xfd64_8de6_68ad_a3e0, turn: 0, attempt: 0 });
    assert_ne!(drawn(&captured, 1).key.conversation, first.conversation);
    assert_ne!(drawn(&captured, 2).key.conversation, first.conversation);
    assert_eq!(drawn(&captured, 3).key, Key { turn: 1, ..first });
}

#[test]
fn capture_keeps_whole_draws_and_stops_at_its_first_bound() {
    let a = ask(b"", b"a");
    let b = text_only(ask(b"", b"b"));
    let c = text_only(ask(b"", b"c"));
    let cut_request = ask(b"", b"cut");
    let scripts = Scripts { cued: Box::new([script(b"cued", Box::new([says(b"script")]))]), ..scripts() };
    let mut domain =
        Domain::drawn(&NAMED, SEEDS, scripts, menu(), rules(), Capture::none()).expect("script and capture");
    let calls = turn(&reply(&mut domain, &NAMED, &a));
    assert_eq!(calls.finish, Finish::ToolCalls);
    assert_eq!(calls.lines.len(), 2);
    assert_eq!(calls.tokens, 16);
    let scripted = ask(b"cued", b"read the script");
    assert_eq!(turn(&reply(&mut domain, &NAMED, &scripted)), says(b"script"));
    let fail = Config { overloaded: 1000, ..NAMED };
    assert_eq!(observe(&mut domain, &fail, &b, 0).1, Err(Error::Overloaded.into()));
    let refused = Config { refused: 1000, ..NAMED };
    assert_eq!(reply(&mut domain, &refused, &b).finish, Finish::ContentFilter);
    let done = turn(&reply(&mut domain, &NAMED, &b));
    assert_eq!(done, Turn { tokens: 5, ..says(b"done") });
    let cut = Config { cut: 1000, ..NAMED };
    assert_eq!(observe(&mut domain, &cut, &cut_request, 0).1.expect_err("cut answer").error, Error::Cut(Cut::MidCall));
    let whole = turn(&reply(&mut named(&NAMED), &NAMED, &cut_request));
    let captured = domain.capture();
    assert!(!captured.full);
    assert_eq!(captured.answers.len(), 3);
    let keys = [0xba8f_d8a2_3d5c_6cdf, 0xba8f_d9a2_3d5c_6e92, 0xdf40_5f10_4f72_171e];
    for ((record, expected), conversation) in captured.answers.iter().zip([calls, done, whole]).zip(keys) {
        assert_eq!(record.key, Key { conversation, turn: 0, attempt: 0 });
        assert_eq!(record.answer, expected);
        assert!(record.slots.is_empty());
    }

    // A one-line text draw holds the Drawn wrapper, its one Line, and four text bytes.
    let price = size_of::<Drawn>().checked_add(size_of::<Line>()).expect("small wrappers");
    let price = u32::try_from(price).expect("small wrappers").checked_add(4).expect("four text bytes");
    let two = price.checked_mul(2).expect("two small draws");
    let short = two.checked_sub(1).expect("one byte short");
    for (config, prefix, full) in [
        (Config { capture_answers: 2, ..NAMED }, 2, true),
        (Config { capture_bytes: short, ..NAMED }, 1, true),
        (Config { capture_bytes: two, ..NAMED }, 2, true),
        (Config { capture_answers: 0, ..NAMED }, 0, false),
    ] {
        let mut bounded = named(&config);
        let a = text_only(a.clone());
        let mut reference = named(&NAMED);
        for request in [&a, &b, &c, &a] {
            let _answer = reply(&mut bounded, &config, request);
            let _answer = reply(&mut reference, &NAMED, request);
        }
        let observed = bounded.capture();
        assert_eq!(observed.full, full);
        assert_eq!(Some(&*observed.answers), reference.capture().answers.get(..prefix));
    }
}

#[test]
fn replay_looks_up_keys_and_counts_each_answer_it_replays() {
    let a = ask(b"", b"a");
    let b = ask(b"", b"b");
    let missing = ask(b"", b"missing");
    let mut original = named(&NAMED);
    let cut = Config { cut: 1000, ..NAMED };
    assert_eq!(observe(&mut original, &cut, &a, 0).1.expect_err("cut answer").error, Error::Cut(Cut::BeforeOutput));
    let _b = reply(&mut original, &NAMED, &b);
    let _a_retry = reply(&mut original, &NAMED, &a);
    let captured = original.capture();
    let a0 = drawn(&captured, 0);
    let b0 = drawn(&captured, 1);
    let a1 = drawn(&captured, 2);
    assert_ne!(a0.answer, a1.answer);

    let other_agent = Seeds::Parts { agent: 81, faults: 3, scheduling: 19 };
    let provider = |config, replay| {
        Domain::drawn(config, other_agent, scripts(), menu(), rules(), replay).expect("replay provider")
    };
    let mut fresh = provider(&NAMED, Capture::none());
    assert_ne!(turn(&reply(&mut fresh, &NAMED, &b)), b0.answer, "agent 81 would draw a different batch");
    let mut replay = provider(&NAMED, captured.clone());
    for (request, recorded) in [(&b, b0), (&a, a0), (&a, a1)] {
        assert_eq!(turn(&reply(&mut replay, &NAMED, request)), recorded.answer);
    }
    let expected: Box<[Drawn]> = Box::new([b0.clone(), a0.clone(), a1.clone()]);
    assert_eq!(replay.capture().answers, expected);
    assert_eq!(turn(&reply(&mut replay, &NAMED, &missing)), turn(&reply(&mut fresh, &NAMED, &missing)));
    let _drawn_again = reply(&mut replay, &NAMED, &a);
    assert_eq!(drawn(&replay.capture(), 4).key, Key { attempt: 2, ..a0.key });

    let strict = Config { strict: true, ..NAMED };
    let mut replay = provider(&strict, captured);
    let failure = observe(&mut replay, &strict, &missing, 0).1.expect_err("strict replay misses the key");
    assert_eq!(&*failure.report.code, b"no_script");
    assert_eq!(replay.capture(), Capture::none());
    let short = Query { max_tokens: 1, ..b };
    let shortened = reply(&mut replay, &strict, &short);
    assert_eq!(shortened.finish, Finish::Length);
    assert_eq!(shortened.usage.output, Some(1));
    let Some(Part::ToolCall { id, arguments, .. }) = shortened.parts.first() else { panic!("cut first call") };
    assert_eq!(&**id, b"call_0000000000000001");
    assert_eq!(&**arguments, b"{\"n");
    assert_eq!(replay.capture().answers.first().expect("replayed draw").answer.finish, Finish::ToolCalls);
}

#[test]
fn malformed_or_oversized_replays_are_refused_at_configuration() {
    let first = Drawn { key: Key { conversation: 1, turn: 0, attempt: 0 }, answer: says(b"ok"), slots: Box::new([]) };
    let call = Line::Call { name: copy_of(b"ls"), arguments: copy_of(b"{}") };
    let text = Line::Text { text: copy_of(b"ok") };
    let config = NAMED;
    let admit =
        |config, answers| Domain::drawn(config, SEEDS, scripts(), menu(), rules(), Capture { answers, full: false });
    assert_eq!(admit(&config, Box::new([first.clone(), first.clone()])).err(), Some(Error::InvalidRequest));
    for answer in [
        Turn { lines: Box::new([]), ..first.answer.clone() },
        Turn { lines: Box::new([Line::Opaque { bytes: copy_of(b"opaque") }]), ..first.answer.clone() },
        Turn { lines: Box::new([call.clone()]), ..first.answer.clone() },
        Turn { finish: Finish::ToolCalls, ..first.answer.clone() },
        Turn { lines: Box::new([text.clone(), call]), finish: Finish::ToolCalls, ..first.answer.clone() },
        Turn { lines: Box::new([text.clone(), text]), ..first.answer.clone() },
    ] {
        let malformed = Drawn { answer, ..first.clone() };
        assert_eq!(admit(&config, Box::new([malformed])).err(), Some(Error::InvalidRequest));
    }
    let slot = Filled {
        id: copy_of(b"call"),
        slot: 9,
        value: copy_of(b"valid"),
        source: Source::Conversation,
        class: copy_of(b"class_1"),
    };
    admit(&config, Box::new([Drawn { slots: Box::new([slot.clone()]), ..first.clone() }]))
        .expect("valid slot metadata");
    for malformed in [
        Filled { id: Box::new([]), ..slot.clone() },
        Filled { value: Box::new([]), ..slot.clone() },
        Filled { value: Box::new([b'x'; 257]), ..slot.clone() },
        Filled { class: Box::new([]), ..slot.clone() },
        Filled { class: Box::new([b'x'; 33]), ..slot.clone() },
        Filled { class: copy_of(b"bad-class"), ..slot },
    ] {
        assert_eq!(
            admit(&config, Box::new([Drawn { slots: Box::new([malformed]), ..first.clone() }])).err(),
            Some(Error::InvalidRequest)
        );
    }

    let scripts = Scripts { cued: Box::new([script(b"cued", Box::new([says(b"script")]))]), ..scripts() };
    let base = crate::limits::scripts(&scripts.cued)
        .expect("script price")
        .checked_add(crate::limits::menu(&menu()).expect("menu price"))
        .expect("script and menu price")
        .checked_add(crate::draws::Draws::attempt_bytes(&config).expect("attempt table price"))
        .expect("shared script bytes");
    let replay = Capture { answers: Box::new([first]), full: false };
    let cap = base.checked_add(crate::limits::replay(&replay).expect("replay price")).expect("small fixture");
    let cap = u32::try_from(cap).expect("small fixture");
    let fits = Config { script_bytes: cap, ..config };
    let short = Config { script_bytes: cap.checked_sub(1).expect("one byte short"), ..config };
    Domain::drawn(&fits, SEEDS, scripts.clone(), menu(), rules(), replay.clone()).expect("replay fits at the bound");
    assert_eq!(Domain::drawn(&short, SEEDS, scripts, menu(), rules(), replay).err(), Some(Error::InvalidRequest));
}
