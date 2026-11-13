//! Frozen forge history retains each changed state (temper/docs/design/testing/real-loop.md).
use super::*;
use skein_lib::{Queue, ReplyTo, Token};

// State grants this fixture user write authority; only one call is outstanding.
const WRITER: u64 = 1;
const CALL: u64 = 1;
const MAIN: &[u8] = b"main";
const TOPIC: &[u8] = b"work";
// The changed state is observed at one tick, then checked again at the next.
const CHANGED: Time = Time::from_nanos(1);
const UNCHANGED: Time = Time::from_nanos(2);

fn call(state: &State, op: api::Op) -> api::Answer {
    let mut forge = state.handle.borrow_mut();
    let mut env = skein_shell::Clock::new().now().with(forge.config.clone());
    let mut out = Queue::uncapped();
    domain::step(
        &mut forge.domain,
        &env,
        domain::Event::Call {
            reply_to: ReplyTo::new(Token::new(CALL)),
            user: WRITER,
            repository: state.repository.clone(),
            op,
        },
        &mut out,
    );
    loop {
        while let Some(request) = out.pop() {
            if let domain::Request::Reply { result, .. } = request {
                forge.domain.reclaim();
                return result.expect("fixture write");
            }
        }
        env.now = forge.domain.next_deadline().expect("fixture answer is pending");
        domain::fire(&mut forge.domain, &env, &mut out);
    }
}

#[test]
fn forge_snapshots_retain_initial_state_and_subsequent_changes() {
    let (state, peer) = State::new(fixture());
    let head = state.handle.borrow().domain.branch(&state.repository, MAIN).expect("seeded main");
    let mut seen = Seen::default();
    seen.collect(Time::ZERO, &peer, &state.handle, &state.repository);
    call(&state, api::Op::Git(api::Git::Create { branch: TOPIC.into(), commit: head }));
    seen.collect(Time::ZERO, &peer, &state.handle, &state.repository);
    let api::Answer::Created(number) = call(
        &state,
        api::Op::Write(api::Write::OpenPull {
            title: b"Change".as_slice().into(),
            body: Box::new([]),
            head: TOPIC.into(),
            base: MAIN.into(),
            opens_empty: true,
        }),
    ) else {
        panic!("created pull");
    };
    seen.collect(Time::ZERO, &peer, &state.handle, &state.repository);
    call(&state, api::Op::Write(api::Write::Close { number }));
    seen.collect(Time::ZERO, &peer, &state.handle, &state.repository);
    call(&state, api::Op::Write(api::Write::Reopen { number }));
    seen.collect(CHANGED, &peer, &state.handle, &state.repository);
    seen.collect(UNCHANGED, &peer, &state.handle, &state.repository);
    let [initial, branched, opened, closed, reopened] = seen.snapshots() else {
        panic!("one initial snapshot and one per branch or pull transition; got {}", seen.snapshots().len());
    };
    assert_eq!(initial.branches.as_ref(), &[(MAIN.into(), head)], "the seed stays frozen");
    assert_eq!(branched.branches.as_ref(), &[(MAIN.into(), head), (TOPIC.into(), head)]);
    assert!(initial.pulls.is_empty());
    assert!(branched.pulls.is_empty());
    assert_eq!(opened.pulls.as_ref(), &[number], "opening a pull changes no branch");
    assert!(closed.pulls.is_empty());
    assert_eq!(reopened.pulls.as_ref(), &[number]);
    assert_eq!(reopened.at, CHANGED, "an unchanged pass keeps the change's time");
    crate::git::share(None);
}

#[test]
fn deliveries_follow_peer_order_without_repeating_unchanged_history() {
    let mut records = vec![Observation::Connection { connection: Token::new(CALL), open: true }];
    let mut seen = Seen::default();
    seen.collect_deliveries(&records);
    assert_eq!(seen.deliveries(), []);
    records.push(Observation::Delivery { post: None, outcome: Delivery::Written });
    seen.collect_deliveries(&records);
    seen.collect_deliveries(&records);
    assert_eq!(seen.deliveries(), &[Delivery::Written]);
    records.push(Observation::Delivery { post: None, outcome: Delivery::Reset });
    seen.collect_deliveries(&records);
    assert_eq!(seen.deliveries(), &[Delivery::Written, Delivery::Reset]);
}
