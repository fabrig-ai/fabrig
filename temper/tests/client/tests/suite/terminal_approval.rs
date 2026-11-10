//! Browser approval across the client roots and engine (temper/docs/design/testing/checks.md, 3).
use jig_client_kit::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use temper_client_domain::{self as client, Action, Place, approve};
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::Joint,
};

const TERMINAL: ClientId = ClientId(0);
const BROWSER: ClientId = ClientId(1);
const OTHER: ClientId = ClientId(2);
// The chat world's two-client pump settles within 24 rounds.
const ROUNDS: usize = 24;
// The approval's span (jig/docs/design/protocol/client/sign-in.md).
const SPAN: Duration = Duration::from_secs(600);

struct Probe {
    root: Root,
    out: Queue<client::Request>,
    reads: Vec<jig_client::ApprovalState>,
    read_acts: usize,
    decisions: usize,
}
fn environment(now: Time) -> Env<client::Limits> {
    Env { now, wall: Wall::from_nanos(now.as_nanos()), limits: fixtures::domain_limits() }
}
fn event(probe: &mut Probe, now: Time, event: client::Event) {
    Temper::step(&mut probe.root, &environment(now), event.into(), &mut probe.out);
}
fn probe(seed: u64) -> Probe {
    let mut probe = Probe {
        root: Root::Person(Box::new(client::unlinked(&fixtures::domain_limits(), seed))),
        out: Queue::uncapped(),
        reads: Vec::new(),
        read_acts: 0,
        decisions: 0,
    };
    Temper::step(
        &mut probe.root,
        &environment(Time::ZERO),
        Temper::start(fixtures::inbox(), Vec::new(), 0),
        &mut probe.out,
    );
    probe
}
fn pump(front: &mut Joint, probes: &mut [Probe], now: Time) {
    let env = environment(now);
    for _ in 0..ROUNDS {
        for (index, probe) in probes.iter_mut().enumerate() {
            if Temper::next_deadline(&probe.root).is_some_and(|deadline| deadline <= now) {
                Temper::fire(&mut probe.root, &env, &mut probe.out);
            }
            Temper::take(&mut probe.root, &env, &mut probe.out);
            while let Some(request) = probe.out.pop() {
                if let client::Request::Unlinked { request: jig_client::UnlinkedRequest::ApprovalAct(act) } = &request {
                    match act {
                        jig_client::ApprovalAct::Read { .. } => probe.read_acts += 1,
                        jig_client::ApprovalAct::Decide { .. } => probe.decisions += 1,
                        jig_client::ApprovalAct::Ask => {}
                    }
                }
                if let Duty::Wire(request) = Temper::duty(request) {
                    front.wire(ClientId(index), env.now, env.wall, request);
                }
            }
        }
        for (who, answer) in front.poll(env.now, env.wall) {
            let probe = &mut probes[who.0];
            if let Some(client::Event::Client {
                event: jig_client::Event::ApprovalAnswer { answer: jig_client::Approval::Read { state, .. } },
            }) = answer.root()
            {
                probe.reads.push(*state);
            }
            front.received(who, &answer);
            Temper::step(&mut probe.root, &env, answer, &mut probe.out);
        }
    }
}
fn setup(others: bool) -> (Joint, Vec<Probe>) {
    let mut front = Joint::with_approval(fixtures::SEED, fixtures::domain_limits());
    front.bind(BROWSER, fixtures::PARTY);
    front.as_browser(BROWSER);
    let mut probes = vec![probe(fixtures::SEED), probe(fixtures::SEED + 1)];
    if others {
        front.bind(OTHER, fixtures::PARTY);
        probes.push(probe(fixtures::SEED + 2));
    }
    event(
        &mut probes[TERMINAL.0],
        Time::ZERO,
        client::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } },
    );
    pump(&mut front, &mut probes, Time::ZERO);
    (front, probes)
}
fn open(probe: &mut Probe, code: jig_client::ApprovalCode) {
    event(probe, Time::ZERO, client::Event::Went { place: Place::Approve { code: Some(code) } });
}
fn approve(probe: &mut Probe) {
    event(probe, Time::ZERO, client::Event::Act { action: Action::Approve(approve::Act::Approve) });
}
fn state(probe: &Probe) -> approve::State {
    probe.root.person().approval().expect("the approval page").state()
}
fn me(probe: &Probe) -> &jig_client::contract::Me {
    let root = probe.root.person();
    root.pages().me(root.client()).expect("the root draws me")
}

#[test]
fn a_terminal_approved_by_a_browser_is_signed_in_as_the_browsers_party() {
    let (mut front, mut probes) = setup(false);
    let browsers_me = me(&probes[BROWSER.0]).clone();
    event(&mut probes[TERMINAL.0], Time::ZERO, client::Event::AskApproval);
    pump(&mut front, &mut probes, Time::ZERO);
    let Some(client::asked::State::Asked { code, expires }) = probes[TERMINAL.0].root.person().asked() else {
        panic!("the signed-out root holds the approval code and expiry");
    };
    assert_eq!(expires, Wall::from_nanos(SPAN.as_nanos()));
    open(&mut probes[BROWSER.0], code);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[BROWSER.0].read_acts, 1);
    assert_eq!(probes[BROWSER.0].reads, [jig_client::ApprovalState::Waiting { asked: Wall::EPOCH }]);
    assert_eq!(state(&probes[BROWSER.0]), approve::State::Asking { asked: Wall::EPOCH });
    approve(&mut probes[BROWSER.0]);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[BROWSER.0].decisions, 1);
    assert_eq!(state(&probes[BROWSER.0]), approve::State::Approved);
    let again = Time::ZERO.saturating_add(approve::AGAIN);
    pump(&mut front, &mut probes, again);
    assert_eq!(probes[BROWSER.0].read_acts, 2);
    assert_eq!(probes[BROWSER.0].reads.last(), Some(&jig_client::ApprovalState::Approved));
    assert_eq!(state(&probes[BROWSER.0]), approve::State::Approved);
    front.take_approval(TERMINAL);
    pump(&mut front, &mut probes, again);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[TERMINAL.0]).party.number, fixtures::PARTY);
    assert_eq!(probes[TERMINAL.0].root.person().asked(), None);
    assert_eq!(probes[BROWSER.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[BROWSER.0]), &browsers_me);
    pump(&mut front, &mut probes, again.saturating_add(approve::AGAIN));
    assert_eq!(probes[BROWSER.0].reads.last(), Some(&jig_client::ApprovalState::Taken));
    assert_eq!(state(&probes[BROWSER.0]), approve::State::SignedIn);
    front.take_approval(TERMINAL);
    assert_eq!(front.taking(TERMINAL), Some(jig_client::Taking::Used));
}

#[test]
fn a_page_in_a_client_that_isnt_a_browser_is_refused() {
    let (mut front, mut probes) = setup(true);
    let others_me = me(&probes[OTHER.0]).clone();
    let (code, _) = front.ask_approval(TERMINAL);
    open(&mut probes[OTHER.0], code);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[OTHER.0].reads, [jig_client::ApprovalState::NotABrowser]);
    assert_eq!(state(&probes[OTHER.0]), approve::State::SignedOut);
    assert_eq!(probes[OTHER.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[OTHER.0]), &others_me);
    front.take_approval(TERMINAL);
    assert_eq!(front.taking(TERMINAL), Some(jig_client::Taking::Waiting));
    open(&mut probes[BROWSER.0], code);
    pump(&mut front, &mut probes, Time::ZERO);
    approve(&mut probes[BROWSER.0]);
    pump(&mut front, &mut probes, Time::ZERO);
    front.take_approval(TERMINAL);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[TERMINAL.0]).party.number, fixtures::PARTY);
}
