//! Quiet-point controls for the session inspection (smith/docs/design/testing.md, 6).

use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_world::inspect::{Inspection, Moment, Release, Wake, check_tables, check_waits};
use smith_domain_session::{self as session, llm};
use smith_domain_tools::Effect;
use smith_inspect::{KINDS, RoomSums, check_room_sum, room, room_sums};
use smith_session_world::{
    Settings,
    recorded::{Scripted, opening},
};

const SEED: u64 = 7;
const MOMENT: Moment = Moment { seed: SEED, at: Time::ZERO, iteration: 1 };

#[test]
fn inspection_reads_the_provider_requests_own_deadline() {
    let mut world = Scripted::new(SEED);
    world.open(opening(None, world.env.limits.spend));
    world.domain.reclaim();
    let deadline = world.deadlines.expect("completion request").whole;
    let mut reading = Inspection::new("smith", KINDS);
    smith_inspect::session(&world.domain, &mut reading);
    let [wait] = reading.waits() else { panic!("one provider wait") };
    assert_eq!(wait.waiter, "run 31");
    assert_eq!(wait.kind, "a session's call waiting on its provider's answer");
    assert!(KINDS.contains(&wait.kind.as_str()));
    assert_eq!(wait.bound, Some(deadline));
    assert_eq!(wait.wakes, [Wake::Party("provider".into())]);
    check_waits(&[&reading], MOMENT).expect("provider wait has a wake within its own bound");
    world.close();
}

#[test]
fn inspection_reads_one_open_call_and_balanced_room_mid_batch() {
    let limits = Settings::calm(SEED).agent;
    let env = Env { now: MOMENT.at, wall: Wall::EPOCH, limits };
    let mut domain = session::Domain::new(&limits, SEED);
    let mut out = Queue::uncapped();
    let waiting = start(&mut domain, &env, &mut out, 31);
    let tooling = start(&mut domain, &env, &mut out, 32);
    session::step(
        &mut domain,
        &env,
        session::Event::Completed {
            owner: tooling,
            completion: llm::Completion {
                served: llm::ServedSpeed::Unreported,
                reasoning_dropped: Box::default(),
                content: Box::new([read(b"first"), read(b"second")]),
                stop: llm::Stop::ToolUse,
                usage: llm::Usage::ZERO,
            },
        },
        &mut out,
    );
    let mut delegates = Vec::new();
    while let Some(request) = out.pop() {
        if let session::Request::Delegate { owner, .. } = request {
            delegates.push(owner);
        }
    }
    let &[first, second] = delegates.as_slice() else { panic!("two reads in one batch") };
    answer(&mut domain, &env, &mut out, first);
    assert!(matches!(out.pop(), Some(session::Request::Priced { .. })));
    while domain.pop_fact().is_some() {}
    domain.reclaim();
    assert!(out.is_empty() && !domain.is_ready() && !domain.is_due(env.now), "quiet mid-batch");

    let mut reading = Inspection::new("smith", KINDS);
    smith_inspect::session(&domain, &mut reading);
    let open: Vec<_> =
        reading.tables().iter().filter(|table| table.name == "open calls").flat_map(|table| &table.entries).collect();
    let [call] = open.as_slice() else { panic!("only the second call is open: {open:?}") };
    assert_eq!(call.release, Release::Pending);
    assert_eq!(call.key, format!("session {} call {}", tooling.raw(), second.raw()));
    check_tables(&[&reading], MOMENT).expect("answered call no longer occupies its slot");
    room(&domain, MOMENT).expect("parallel sessions' reservations balance at the quiet point");
    let provider = room_sums(&domain, waiting, MOMENT).expect("provider sums fit").expect("live provider session");
    assert!(provider.provider.is_some_and(|credit| credit > 0));
    assert_eq!(
        (provider.reserved, provider.provider),
        domain.provider_credit_for_test(waiting).expect("provider credit")
    );
    let batch = room_sums(&domain, tooling, MOMENT).expect("batch sums fit").expect("live tooling session");
    assert!(batch.batch > 0);
    assert_eq!(
        (batch.reserved, batch.provider, batch.batch, batch.program),
        domain.result_credit_for_test(tooling).expect("result credit")
    );

    answer(&mut domain, &env, &mut out, second);
    for owner in [waiting, tooling] {
        session::step(&mut domain, &env, session::Event::Close { session: owner }, &mut out);
        session::step(&mut domain, &env, session::Event::Cancelled { owner }, &mut out);
    }
    domain.reclaim();
    assert_eq!(domain.sessions(), 0);
}

#[test]
fn inspection_names_an_unbalanced_room_sum_in_its_finding() {
    let sums = RoomSums { reserved: 11, provider: Some(5), batch: 3, program: 2 };
    let moment = Moment { seed: SEED, at: Time::from_nanos(120_000_000), iteration: 48 };
    assert_eq!(check_room_sum(moment, Token::new(31), sums), Err(
        "seed 7: at 120000000 ns, iteration 48: smith session 31: reserved room 11 bytes does not equal provider 5 + batch 3 + program 2 bytes".into()
    ));
}

fn start(
    domain: &mut session::Domain,
    env: &Env<session::Limits>,
    out: &mut Queue<session::Request>,
    opener: u64,
) -> Token {
    let mut opening = opening(None, env.limits.spend);
    opening.spec.delegated[0].effect = Effect::Read;
    session::step(domain, env, session::Event::Open { opener: Token::new(opener), opening: Box::new(opening) }, out);
    let mut completing = None;
    while let Some(request) = out.pop() {
        if let session::Request::Complete { owner, .. } = request {
            assert!(completing.replace(owner).is_none());
        }
    }
    completing.expect("one request per opener")
}

fn read(id: &[u8]) -> llm::Block {
    llm::Block::ToolCall(llm::ToolCall {
        id: id.into(),
        name: b"lookup".as_slice().into(),
        input: b"{}".as_slice().into(),
        replay: None,
        call: llm::Decoded::Delegated {
            descriptor: Token::new(19),
            ticket: Token::new(19),
            source: session::ToolSource::Run,
            effect: Effect::Read,
        },
    })
}

fn answer(domain: &mut session::Domain, env: &Env<session::Limits>, out: &mut Queue<session::Request>, owner: Token) {
    session::step(
        domain,
        env,
        session::Event::Answered {
            owner,
            text: b"present".as_slice().into(),
            error: false,
            spent: 0,
            after: session::After::Complete,
        },
        out,
    );
}
