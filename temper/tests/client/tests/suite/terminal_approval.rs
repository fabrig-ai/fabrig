//! Browser approval across the client roots and engine (temper/docs/design/testing/checks.md, 3).
use jig_client_kit::harness::{App, ClientId, Duty, Front};
use jig_protocol_engine_clients::Via;
use skein_lib::{Duration, Env, Queue, Secret, Time, Wall, base64};
use skein_ui::Role;
use temper_client_domain::{self as client, Action, approve};
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
    linked: bool,
    out: Queue<client::Request>,
    reads: Vec<jig_client::ApprovalState>,
    read_acts: usize,
    decisions: usize,
    leaves: Vec<(Box<[u8]>, bool)>,
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
        linked: false,
        out: Queue::uncapped(),
        reads: Vec::new(),
        read_acts: 0,
        decisions: 0,
        leaves: Vec::new(),
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
            if probe.linked {
                Temper::settle(&mut probe.root, &env, &mut probe.out);
                Temper::take(&mut probe.root, &env, &mut probe.out);
            }
            while let Some(request) = probe.out.pop() {
                if let client::Request::Unlinked { request: jig_client::UnlinkedRequest::ApprovalAct(act) } = &request {
                    match act {
                        jig_client::ApprovalAct::Read { .. } => probe.read_acts += 1,
                        jig_client::ApprovalAct::Decide { .. } => probe.decisions += 1,
                        jig_client::ApprovalAct::Ask { .. } => {}
                    }
                }
                if let client::Request::Leave { to, replace } = &request {
                    probe.leaves.push((to.clone(), *replace));
                }
                if let Duty::Wire(request) = Temper::duty(request) {
                    if probe.linked {
                        let client::Request::Client { request } = request else { panic!("linked terminal request") };
                        front.carry(ClientId(index), now, request);
                    } else {
                        front.wire(ClientId(index), env.now, env.wall, request);
                    }
                }
            }
        }
        for (index, probe) in probes.iter_mut().enumerate() {
            if probe.linked {
                for answer in front.heard(ClientId(index), now) {
                    event(probe, now, client::Event::Client { event: answer });
                }
            }
        }
        for (who, answer) in front.poll(env.now, env.wall) {
            let probe = &mut probes[who.0];
            if let Some(client::Event::Client {
                event: jig_client::Event::ApprovalAnswer { answer: jig_client::Approval::Read { state, .. } },
            }) = answer.root()
            {
                probe.reads.push(state.clone());
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
    open_path(probe, &client::address::print_approve(Some(code)));
}
fn open_path(probe: &mut Probe, path: &[u8]) {
    event(probe, Time::ZERO, client::Event::Went { place: client::address::parse(path) });
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
    event(&mut probes[TERMINAL.0], Time::ZERO, client::Event::AskApproval { port: None });
    pump(&mut front, &mut probes, Time::ZERO);
    let Some(client::asked::State::Asked { code, expires, .. }) = probes[TERMINAL.0].root.person().asked() else {
        panic!("the signed-out root holds the approval code and expiry");
    };
    assert_eq!(expires, Wall::from_nanos(SPAN.as_nanos()));
    open(&mut probes[BROWSER.0], code);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[BROWSER.0].read_acts, 1);
    assert_eq!(
        probes[BROWSER.0].reads,
        [jig_client::ApprovalState::Waiting {
            asked: Wall::EPOCH,
            kind: jig_client::ApprovalKind::Listener,
            source: None,
            reader: None
        }]
    );
    assert_eq!(
        state(&probes[BROWSER.0]),
        approve::State::Asking {
            asked: Wall::EPOCH,
            kind: jig_client::ApprovalKind::Listener,
            source: None,
            reader: None
        }
    );
    approve(&mut probes[BROWSER.0]);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[BROWSER.0].decisions, 1);
    assert_eq!(state(&probes[BROWSER.0]), approve::State::Approved);
    let again = Time::ZERO.saturating_add(approve::AGAIN);
    pump(&mut front, &mut probes, again);
    assert_eq!(probes[BROWSER.0].read_acts, 2);
    assert_eq!(probes[BROWSER.0].reads.last(), Some(&jig_client::ApprovalState::Approved));
    assert_eq!(state(&probes[BROWSER.0]), approve::State::Approved);
    front.take_approval(TERMINAL, None);
    pump(&mut front, &mut probes, again);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[TERMINAL.0]).party.number, fixtures::PARTY);
    assert_eq!(probes[TERMINAL.0].root.person().asked(), None);
    assert_eq!(probes[BROWSER.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[BROWSER.0]), &browsers_me);
    pump(&mut front, &mut probes, again.saturating_add(approve::AGAIN));
    assert_eq!(probes[BROWSER.0].reads.last(), Some(&jig_client::ApprovalState::Taken));
    assert_eq!(state(&probes[BROWSER.0]), approve::State::SignedIn);
    front.take_approval(TERMINAL, None);
    assert_eq!(front.taking(TERMINAL), Some(jig_client::Taking::Used));
}

fn view(probe: &Probe) -> temper_client_view::View {
    let limits = fixtures::domain_limits();
    let words = temper_client_view::words::Words::english(
        fixtures::FORGE,
        limits.client.domain.text,
        temper_client_view::words::Shell::Browser,
    )
    .expect("browser words fit");
    let bounds = fixtures::view_bounds(&limits, &words);
    let mut view = temper_client_view::View::new(&bounds, &limits, words, fixtures::FORGE);
    temper_client_view::build(&mut view, probe.root.person(), Wall::EPOCH, &mut Queue::uncapped());
    view
}
fn text(view: &temper_client_view::View) -> String {
    let bytes: Vec<u8> =
        view.tree().nodes().iter().flat_map(|node| node.text.iter().flat_map(|run| run.text.iter().copied())).collect();
    String::from_utf8(bytes).expect("view text")
}
fn type_code(probe: &mut Probe, code: jig_client::ApprovalCode) {
    event(
        probe,
        Time::ZERO,
        client::Event::Act { action: Action::Approve(approve::Act::Words { words: code.shown().into() }) },
    );
    event(probe, Time::ZERO, client::Event::Act { action: Action::Approve(approve::Act::Check) });
}

#[test]
fn one_press_returns_a_grant_to_the_terminals_loopback_before_it_can_sign_in() {
    let (mut front, mut probes) = setup(false);
    let source = Some("203.0.113.7".parse().expect("terminal source"));
    front.link_client(TERMINAL, fixtures::domain_limits().client.domain);
    front.exchange_from(TERMINAL, Via::Public, source);
    probes[TERMINAL.0].root = Root::Person(Box::new(client::new(&fixtures::domain_limits(), fixtures::SEED)));
    probes[TERMINAL.0].linked = true;
    event(
        &mut probes[TERMINAL.0],
        Time::ZERO,
        client::Event::Start { address: fixtures::inbox(), kept: skein_lib::List::with_capacity(0), with: None },
    );
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::SignedOut);
    event(&mut probes[TERMINAL.0], Time::ZERO, client::Event::AskApproval { port: Some(43210) });
    pump(&mut front, &mut probes, Time::ZERO);
    let asked = probes[TERMINAL.0].root.person().asked();
    let Some(client::asked::State::Asked { code, port: Some(43210), .. }) = asked else {
        panic!("the terminal root's port ask was not answered: {asked:?}");
    };
    assert_eq!(
        probes[TERMINAL.0].root.person().client().link().not_signed_in,
        Some(jig_client::NotSignedIn::Approval(jig_client::Taking::Waiting))
    );
    front.exchange_from(BROWSER, Via::Public, source);
    // Extra address parameters cannot choose the hand-off host or port.
    let mut path = client::address::print_approve(Some(code)).into_vec();
    path.extend_from_slice(b"&host=elsewhere.example&port=32123");
    open_path(&mut probes[BROWSER.0], &path);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[BROWSER.0].read_acts, 1);
    assert_eq!(probes[BROWSER.0].reads.len(), 1);
    assert_eq!(
        state(&probes[BROWSER.0]),
        approve::State::Asking { asked: Wall::EPOCH, kind: jig_client::ApprovalKind::Returned, source, reader: source }
    );
    assert!(text(&view(&probes[BROWSER.0])).contains("from 203.0.113.7"));
    assert_eq!(probes[BROWSER.0].decisions, 0);
    assert_eq!(probes[BROWSER.0].leaves.len(), 0);
    let browsers_me = me(&probes[BROWSER.0]).clone();
    approve(&mut probes[BROWSER.0]);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[BROWSER.0].decisions, 1);
    assert_eq!(state(&probes[BROWSER.0]), approve::State::Handing);
    assert_eq!(probes[BROWSER.0].leaves.len(), 1);
    let (to, replace) = &probes[BROWSER.0].leaves[0];
    assert!(*replace);
    let encoded = to
        .strip_prefix(b"http://127.0.0.1:43210/approved?grant=")
        .expect("engine's port, fixed loopback host and path");
    assert_eq!(encoded.len(), 43);
    let grant = base64::decode(base64::Alphabet::Url, encoded, 32).expect("base64url grant");
    assert_eq!(grant.len(), 32);
    let again = Time::ZERO.saturating_add(jig_client::APPROVAL_AGAIN);
    pump(&mut front, &mut probes, again);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::SignedOut);
    assert_eq!(
        probes[TERMINAL.0].root.person().client().link().not_signed_in,
        Some(jig_client::NotSignedIn::Approval(jig_client::Taking::Waiting))
    );
    event(&mut probes[TERMINAL.0], again, client::Event::Granted { grant: Secret::new(grant) });
    pump(&mut front, &mut probes, again);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[TERMINAL.0]).party.number, browsers_me.party.number);
    assert_eq!(probes[TERMINAL.0].root.person().asked(), None);
    assert_eq!(me(&probes[BROWSER.0]), &browsers_me);
    assert_eq!(probes[BROWSER.0].decisions, 1);
    assert_eq!(probes[BROWSER.0].leaves.len(), 1);
    // The wait for the terminal's next take allowed one more browser read.
    assert_eq!(probes[BROWSER.0].read_acts, 2);
}

#[test]
fn a_public_ask_without_a_return_needs_its_code_typed_on_either_origin() {
    let mut origins = 0;
    for via in [Via::Public, Via::Listener] {
        let (mut front, mut probes) = setup(false);
        let (code, _) = front.ask_approval(TERMINAL, None, Via::Public, None);
        front.exchange_from(BROWSER, via, None);
        open(&mut probes[BROWSER.0], code);
        pump(&mut front, &mut probes, Time::ZERO);
        assert_eq!(probes[BROWSER.0].read_acts, 1);
        assert_eq!(probes[BROWSER.0].reads.len(), 1);
        assert_eq!(state(&probes[BROWSER.0]), approve::State::Typing { unread: false });
        let page = probes[BROWSER.0].root.person().approval().expect("approval page");
        assert_eq!(page.code(), None);
        assert_eq!(page.words(), b"");
        // The web shell replaces history when this code is dropped; this world stops at the domain.
        assert_eq!(client::address::print_approve(page.code()).as_ref(), b"/terminal");
        let drawn = view(&probes[BROWSER.0]);
        let field = drawn
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == Role::TextField && node.name.as_deref() == Some(b"Code"))
            .expect("Code field");
        assert!(field.value.as_ref().expect("field value").text.is_empty());
        assert!(
            !drawn.tree().nodes().iter().any(|node| node.role == Role::Alert
                || node.role == Role::Button && node.name.as_deref() == Some(b"Approve"))
        );
        approve(&mut probes[BROWSER.0]);
        pump(&mut front, &mut probes, Time::ZERO);
        assert_eq!(probes[BROWSER.0].decisions, 0, "an address code offers no approval");
        type_code(&mut probes[BROWSER.0], code);
        pump(&mut front, &mut probes, Time::ZERO);
        assert_eq!(probes[BROWSER.0].read_acts, 2);
        assert_eq!(probes[BROWSER.0].reads.len(), 2);
        assert_eq!(
            state(&probes[BROWSER.0]),
            approve::State::Asking {
                asked: Wall::EPOCH,
                kind: jig_client::ApprovalKind::Typed,
                source: None,
                reader: None
            }
        );
        assert!(
            view(&probes[BROWSER.0])
                .tree()
                .nodes()
                .iter()
                .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Approve"))
        );
        approve(&mut probes[BROWSER.0]);
        pump(&mut front, &mut probes, Time::ZERO);
        assert_eq!(probes[BROWSER.0].decisions, 1);
        assert_eq!(state(&probes[BROWSER.0]), approve::State::Approved);
        assert_eq!(probes[BROWSER.0].leaves.len(), 0);
        front.take_approval(TERMINAL, None);
        pump(&mut front, &mut probes, Time::ZERO);
        assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::Live);
        assert_eq!(me(&probes[TERMINAL.0]).party.number, me(&probes[BROWSER.0]).party.number);
        assert_eq!(probes[BROWSER.0].read_acts, 2);
        assert_eq!(probes[BROWSER.0].decisions, 1);
        origins += 1;
    }
    assert_eq!(origins, 2);
}

#[test]
fn a_typed_ask_says_when_it_came_from_another_network_than_the_browser() {
    let source = Some("203.0.113.7".parse().expect("terminal source"));
    let mut networks = 0;
    for (reader, other) in [("198.51.100.9", true), ("203.0.113.7", false)] {
        let (mut front, mut probes) = setup(false);
        let reader = Some(reader.parse().expect("browser source"));
        let (code, _) = front.ask_approval(TERMINAL, None, Via::Public, source);
        front.exchange_from(BROWSER, Via::Public, reader);
        open_path(&mut probes[BROWSER.0], b"/terminal");
        type_code(&mut probes[BROWSER.0], code);
        pump(&mut front, &mut probes, Time::ZERO);
        assert_eq!(probes[BROWSER.0].read_acts, 1);
        assert_eq!(probes[BROWSER.0].reads.len(), 1);
        let column = text(&view(&probes[BROWSER.0]));
        assert!(column.contains("from 203.0.113.7"));
        assert_eq!(column.contains("It was asked from another network than this browser's."), other);
        assert_eq!(
            state(&probes[BROWSER.0]),
            approve::State::Asking { asked: Wall::EPOCH, kind: jig_client::ApprovalKind::Typed, source, reader }
        );
        assert_eq!(probes[BROWSER.0].decisions, 0);
        networks += 1;
    }
    assert_eq!(networks, 2);
}

#[test]
fn a_page_in_a_client_that_isnt_a_browser_is_refused() {
    let (mut front, mut probes) = setup(true);
    let others_me = me(&probes[OTHER.0]).clone();
    let (code, _) = front.ask_approval(TERMINAL, None, Via::Listener, None);
    open(&mut probes[OTHER.0], code);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[OTHER.0].reads, [jig_client::ApprovalState::NotABrowser]);
    assert_eq!(state(&probes[OTHER.0]), approve::State::SignedOut);
    assert_eq!(probes[OTHER.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[OTHER.0]), &others_me);
    front.take_approval(TERMINAL, None);
    assert_eq!(front.taking(TERMINAL), Some(jig_client::Taking::Waiting));
    open(&mut probes[BROWSER.0], code);
    pump(&mut front, &mut probes, Time::ZERO);
    approve(&mut probes[BROWSER.0]);
    pump(&mut front, &mut probes, Time::ZERO);
    front.take_approval(TERMINAL, None);
    pump(&mut front, &mut probes, Time::ZERO);
    assert_eq!(probes[TERMINAL.0].root.person().client().link().state, jig_client::Link::Live);
    assert_eq!(me(&probes[TERMINAL.0]).party.number, fixtures::PARTY);
}
