//! Account and cut transitions (testing/fakes/llm.md, 2.7–2.8).
use crate::api::{Answer, Cut, Error, Failure, Line, Part, Piece, Query, Report, Strike, Turn};
use crate::tests::{CONFIG, query, says, script, user};
use crate::{Config, Domain, Event, Request, fire, step};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall, bytes::copy_of};

const BOUNDS: Config = Config { accounts: 2, unserved: 2, ..CONFIG };

fn configured(config: &Config, lines: Box<[Line]>, seed: u64) -> Domain {
    Domain::scripted(config, seed, Box::new([script(b"cue", Box::new([Turn { lines, ..says(b"") }]))]))
}
fn asked() -> Query {
    let mut query = query(Box::new([user(Box::new([Part::Text { text: copy_of(b"question") }]))]));
    query.system = copy_of(b"cue");
    query.credential = Some(0);
    query
}
fn text() -> Line {
    Line::Text { text: copy_of(b"whole turn") }
}
type Observed = (Result<Answer, Failure>, Box<[Box<[u8]>]>);
fn call(domain: &mut Domain, config: &Config, query: &Query, now: u64) -> Observed {
    let env = Env { now: Time::from_nanos(now), wall: Wall::EPOCH, limits: *config };
    let mut out = Queue::uncapped();
    let token = Token::new(1);
    step(domain, &env, Event::Call { reply_to: ReplyTo::new(token), call: token, query: query.clone() }, &mut out);
    let mut pieces = skein_lib::List::with_capacity(16);
    for _ in 0_u32..16 {
        fire(domain, &env, &mut out);
        while let Some(request) = out.pop() {
            match request {
                Request::Piece { text, .. } => pieces.push(text).expect("piece room"),
                Request::Hold { .. } => panic!("no hold in this fixture"),
                Request::Reply { result, .. } => {
                    domain.reclaim();
                    return (result, pieces.into_boxed());
                }
            }
        }
    }
    panic!("the call must answer");
}

#[test]
fn strikes_precede_chance_and_script_and_clear_preserves_the_turn() {
    let mut domain = configured(&BOUNDS, Box::new([text(), Line::Cut { times: 1 }]), 1);
    let mut query = asked();
    domain.strike(0, Strike::Spent, Time::ZERO).expect("account room");
    domain.strike(0, Strike::SignedOut, Time::ZERO).expect("second kind");
    domain.strike(0, Strike::Unserved { model: query.model.clone() }, Time::ZERO).expect("model room");
    let chances = Config { overloaded: 1000, ..BOUNDS };
    assert_eq!(call(&mut domain, &chances, &query, 0).0.expect_err("struck").error, Error::Spent);
    domain.clear(0, &Strike::Window { tokens: 1, length: Duration::from_secs(1) });
    assert_eq!(call(&mut domain, &BOUNDS, &query, 0).0.expect_err("unknown clear changes nothing").error, Error::Spent);
    domain.clear(0, &Strike::Spent);
    assert_eq!(call(&mut domain, &BOUNDS, &query, 0).0.expect_err("first remaining strike").error, Error::SignedOut);
    domain.clear(0, &Strike::SignedOut);
    assert_eq!(call(&mut domain, &BOUNDS, &query, 0).0.expect_err("model strike").error, Error::Unserved);
    query.model = copy_of(b"other model");
    assert_eq!(
        call(&mut domain, &BOUNDS, &query, 0).0.expect_err("unserved only names one model").error,
        Error::Cut(Cut::MidText)
    );
    query.model = copy_of(b"fake");
    domain.clear(0, &Strike::Unserved { model: query.model.clone() });
    assert_eq!(
        call(&mut domain, &BOUNDS, &query, 0).0.expect("same turn whole").parts.as_ref(),
        &[Part::Text { text: copy_of(b"whole turn") }]
    );
    domain.strike(0, Strike::Spent, Time::ZERO).expect("account room");
    query.credential = Some(1);
    assert!(call(&mut domain, &BOUNDS, &query, 0).0.is_ok(), "another credential is untouched");
    query.credential = None;
    assert!(call(&mut domain, &BOUNDS, &query, 0).0.is_ok(), "uncredentialed calls are untouched");
}

#[test]
fn windows_share_charges_and_keep_fixed_boundaries_and_a_lone_large_call() {
    let mut domain = configured(&BOUNDS, Box::new([text()]), 1);
    let query = asked();
    let answer = call(&mut domain, &BOUNDS, &query, 0).0.expect("calibrate usage");
    let input = answer.usage.input.expect("input")
        + answer.usage.cache_read.expect("read")
        + answer.usage.cache_write.expect("write");
    assert!(input > 0);
    let window = Strike::Window { tokens: input, length: Duration::from_nanos(10) };
    domain.strike(0, window.clone(), Time::from_nanos(5)).expect("window room");
    call(&mut domain, &BOUNDS, &query, 6).0.expect("first call");
    let mut other = query.clone();
    other.messages = Box::new([user(Box::new([Part::Text { text: copy_of(b"separate conversation") }]))]);
    assert_eq!(
        call(&mut domain, &BOUNDS, &other, 14).0.expect_err("shared charge").error,
        Error::Window { reset: Duration::from_nanos(1), limit: input, remaining: 0 }
    );
    assert!(call(&mut domain, &BOUNDS, &other, 15).0.is_ok(), "lone call beyond allowance is served");
    assert_eq!(
        call(&mut domain, &BOUNDS, &query, 16).0.expect_err("large call charged").error,
        Error::Window { reset: Duration::from_nanos(9), limit: input, remaining: 0 }
    );
    assert!(call(&mut domain, &BOUNDS, &query, 25).0.is_ok(), "reset stays anchored at the strike");
    domain.clear(0, &window);
    domain
        .strike(0, Strike::Window { tokens: input * 2, length: Duration::from_nanos(10) }, Time::from_nanos(26))
        .expect("partial window");
    call(&mut domain, &BOUNDS, &query, 26).0.expect("first charge");
    assert_eq!(
        call(&mut domain, &BOUNDS, &other, 27).0.expect_err("remaining allowance").error,
        Error::Window { reset: Duration::from_nanos(9), limit: input * 2, remaining: input }
    );
    domain.clear(0, &window);
    domain
        .strike(0, Strike::Window { tokens: 1, length: Duration::from_nanos(10) }, Time::from_nanos(30))
        .expect("new window");
    call(&mut domain, &BOUNDS, &query, 31).0.expect("lone large call");
    call(&mut domain, &BOUNDS, &query, 32).0.expect_err("large call charged");
    call(&mut domain, &BOUNDS, &query, 40).0.expect("new fixed window");
}

#[test]
fn strike_admission_refuses_credentials_models_and_bytes_without_dropping_them() {
    let config = Config { accounts: 1, unserved: 1, ..CONFIG };
    let mut domain = Domain::new(&config, 1);
    assert_eq!(domain.strike(1, Strike::Spent, Time::ZERO), Err(Error::ContextTooLong));
    let first = Strike::Unserved { model: copy_of(b"a") };
    domain.strike(0, first.clone(), Time::ZERO).expect("one model");
    domain.strike(0, first.clone(), Time::ZERO).expect("idempotent strike");
    assert_eq!(domain.strike(0, Strike::Unserved { model: copy_of(b"b") }, Time::ZERO), Err(Error::ContextTooLong));
    domain.clear(0, &first);
    domain.strike(0, Strike::Unserved { model: copy_of(b"b") }, Time::ZERO).expect("clear releases model room");
    let price = crate::accounts::Accounts::price(&config).expect("small account reservation");
    let config = Config { script_bytes: u32::try_from(price).expect("small price"), ..config };
    let mut domain = Domain::new(&config, 1);
    assert_eq!(domain.strike(0, first, Time::ZERO), Err(Error::ContextTooLong));
    domain.strike(0, Strike::Spent, Time::ZERO).expect("non-owning strike fits reservation");
}

#[test]
fn cuts_keep_their_place_parts_and_count_then_replay_whole() {
    for (before, place, expected) in [
        (None, Cut::BeforeOutput, None),
        (Some(text()), Cut::MidText, Some(Part::Text { text: copy_of(b"whole turn") })),
        (
            Some(Line::Call { name: copy_of(b"ls"), arguments: copy_of(b"1234567") }),
            Cut::MidCall,
            Some(Part::ToolCall { id: copy_of(b"call_1"), name: copy_of(b"ls"), arguments: copy_of(b"123") }),
        ),
        (
            Some(Line::Pieces {
                pieces: Box::new([
                    Piece { text: copy_of(b"whole "), hold: None },
                    Piece { text: copy_of(b"turn"), hold: None },
                ]),
            }),
            Cut::MidText,
            Some(Part::Text { text: copy_of(b"whole turn") }),
        ),
    ] {
        let mut lines = skein_lib::List::with_capacity(3);
        if let Some(before) = before {
            lines.push(before).expect("line room");
        }
        lines.push(Line::Cut { times: 2 }).expect("line room");
        lines.push(Line::Text { text: copy_of(b"after") }).expect("line room");
        let mut domain = configured(&BOUNDS, lines.into(), 1);
        for attempt in 0_u32..2 {
            let (got, pieces) = call(&mut domain, &BOUNDS, &asked(), 0);
            let failure = got.expect_err("cut count");
            assert_eq!(failure.error, Error::Cut(place));
            if let Some(expected) = &expected {
                let part = failure.parts.first().expect("prior part");
                match (part, expected) {
                    (
                        Part::ToolCall { name, arguments, .. },
                        Part::ToolCall { name: expected_name, arguments: expected_arguments, .. },
                    ) => {
                        assert_eq!(name, expected_name);
                        assert_eq!(arguments, expected_arguments);
                    }
                    _ => assert_eq!(part, expected),
                }
            } else {
                assert!(failure.parts.is_empty());
            }
            if !pieces.is_empty() {
                assert_eq!(pieces.as_ref(), &[copy_of(b"whole ")]);
            }
            assert_eq!(failure.parts.len(), usize::from(expected.is_some()), "attempt {attempt}");
        }
        let answer = call(&mut domain, &BOUNDS, &asked(), 0).0.expect("cut passes after its count");
        assert_eq!(answer.parts.last(), Some(&Part::Text { text: copy_of(b"after") }));
        if let Some(Part::ToolCall { arguments, .. }) = answer.parts.first() {
            assert_eq!(arguments.as_ref(), b"1234567");
        }
    }
}

#[test]
fn cut_configuration_refuses_zero_and_failing_cut_classes() {
    for line in [
        Line::Cut { times: 0 },
        Line::Fail { error: Error::Cut(Cut::BeforeOutput), report: Report::none(), credentials: Box::new([]) },
    ] {
        let result = Domain::try_scripted(
            &BOUNDS,
            1,
            Box::new([script(b"cue", Box::new([Turn { lines: Box::new([line]), ..says(b"") }]))]),
        );
        assert_eq!(result.err(), Some(Error::InvalidRequest));
    }
}

#[test]
fn seeded_cuts_draw_places_and_replay_them() {
    let config = Config { cut: 1000, ..BOUNDS };
    let mut places = skein_lib::List::with_capacity(3);
    for seed in 1..=16 {
        let mut results = skein_lib::List::with_capacity(2);
        for _ in 0_u32..2 {
            let mut domain = configured(
                &config,
                Box::new([text(), Line::Call { name: copy_of(b"ls"), arguments: copy_of(b"1234567") }]),
                seed,
            );
            results.push(call(&mut domain, &config, &asked(), 0).0.expect_err("certain cut")).expect("two replays");
        }
        assert_eq!(results.get(0), results.get(1), "seed {seed}");
        let error = results.get(0).expect("first replay").error;
        if !places.as_slice().contains(&error) {
            places.push(error).expect("three places");
        }
    }
    assert!(places.as_slice().contains(&Error::Cut(Cut::BeforeOutput)));
    assert!(places.as_slice().contains(&Error::Cut(Cut::MidText)));
    assert!(places.as_slice().contains(&Error::Cut(Cut::MidCall)));
}
