//! Browser-approved sign-ins across the real links, clients' component and core
//! (jig/docs/design/protocol/client.md, 8; jig/docs/design/client/domain/testing.md, 5).
use jig_client::{Approval, ApprovalAct, ApprovalState, Client, Decided, Event, Link, Taking, UnlinkedRequest};
use jig_client_protocol_link::Signing;
use jig_client_world::{Front, Test, harness::ClientId, joint::Joint};
use skein_lib::{Duration, List, Queue, Secret, Time, Wall};
use std::collections::BTreeMap;

const TERMINAL: ClientId = ClientId(0);
const BROWSER: ClientId = ClientId(1);
const BEARER: ClientId = ClientId(2);
const SPAN: Duration = Duration::from_secs(600);

fn wall(now: Time) -> Wall {
    Wall::from_nanos(1_000_000_000_000 + now.as_nanos())
}

struct Story {
    front: Joint,
    clients: BTreeMap<ClientId, Client>,
    now: Time,
}
impl Story {
    fn new() -> Self {
        let mut front = jig_client_world::joint::front::fixture(&jig_test_profile::limits());
        front.session.as_browser(BROWSER);
        Front::<Test>::bind(&mut front, BROWSER, 1);
        // This unsigned provider identity is different from the browser's.
        // The expected party is the browser's; no provider exchange is made.
        front.session.bind_unsigned(
            TERMINAL,
            1,
            jig_domain_people::Identity {
                key: jig_domain_people::IdentityKey { provider: 0, subject: Box::new([99]) },
                login: Box::from(b"terminal".as_slice()),
                name: Box::from(b"Terminal".as_slice()),
            },
            Time::ZERO,
            wall(Time::ZERO),
        );
        let mut story = Self { front, clients: BTreeMap::new(), now: Time::ZERO };
        story.add_client(TERMINAL);
        story.add_client(BROWSER);
        story.settle();
        story.assert_me(BROWSER, 1);
        assert_eq!(story.clients[&TERMINAL].link().state, Link::SignedOut);
        story
    }

    fn add_client(&mut self, id: ClientId) {
        let limits = jig_client::Limits { domain: jig_client_world::test::limits().client.domain, exchanges: 8 };
        let client = Client::unlinked(&limits, 8517).expect("bounded client");
        self.clients.insert(id, client);
        self.step(id, Event::StartUnlinked { kept: List::with_capacity(0) });
    }

    fn step(&mut self, id: ClientId, event: Event) {
        let mut out = Queue::uncapped();
        let client = self.clients.get_mut(&id).expect("story client");
        client.step_unlinked(self.now, wall(self.now), event, &mut out);
        client.reclaim();
        while let Some(request) = out.pop() {
            match request {
                UnlinkedRequest::Down { down } => {
                    self.front.down(id, self.now, wall(self.now), down).expect("real joint front");
                }
                UnlinkedRequest::Told { .. } => {}
                other @ (UnlinkedRequest::SignOut
                | UnlinkedRequest::ApprovalAct(_)
                | UnlinkedRequest::Approval(_)
                | UnlinkedRequest::Authorized(_)) => panic!("unexpected client work: {other:?}"),
            }
        }
    }

    fn settle(&mut self) {
        for _ in 0..20 {
            let callbacks = self.front.up(self.now, wall(self.now), &[]);
            if callbacks.is_empty() {
                return;
            }
            for (id, up) in callbacks {
                self.step(id, Event::Up { up });
            }
        }
        panic!("joint approval story did not settle");
    }

    fn approval(&mut self, id: ClientId, act: ApprovalAct) -> Approval {
        self.front.session.approval(id, self.now, wall(self.now), act);
        self.settle();
        let answer = self.front.session.take_approval_answer(id).expect("one approval answer");
        assert!(self.front.session.take_approval_answer(id).is_none(), "answer handed out once");
        answer
    }

    fn ask(&mut self) -> (Secret, jig_client::ApprovalCode) {
        let Approval::Asked { secret, code, expires } = self.approval(TERMINAL, ApprovalAct::Ask) else {
            panic!("an unsigned client can ask an approval");
        };
        assert_eq!(secret.len(), 32);
        assert_eq!(expires, wall(self.now.saturating_add(SPAN)));
        assert_eq!(self.front.session.component_deadline(), Some(self.now.saturating_add(SPAN)));
        (secret, code)
    }

    fn read(&mut self, id: ClientId, code: jig_client::ApprovalCode, state: ApprovalState) {
        assert_eq!(self.approval(id, ApprovalAct::Read { code }), Approval::Read { code, state });
    }

    fn decide(&mut self, id: ClientId, code: jig_client::ApprovalCode, decided: Decided) {
        assert_eq!(
            self.approval(id, ApprovalAct::Decide { code, approve: true }),
            Approval::Decided { code, decided }
        );
    }

    fn take(&mut self, secret: &Secret, expected: Signing) {
        self.front.session.take_approval(TERMINAL, self.now, wall(self.now), secret.clone());
        self.settle();
        assert_eq!(self.front.session.take_approval_signing(TERMINAL), Some(expected));
        assert!(self.front.session.take_approval_signing(TERMINAL).is_none(), "one take answer");
    }

    fn assert_me(&self, id: ClientId, party: u64) {
        let me = self.clients[&id].link();
        assert_eq!((me.state, me.party), (Link::Live, Some(party)), "the client's real me watch");
    }
}

#[test]
fn a_terminals_approval_approved_by_a_browser_signs_it_in_as_the_browsers_party() {
    let mut story = Story::new();
    let before = story.front.session.sign_ins();
    let browser_credential = story.front.session.credential(BROWSER).expect("browser signed in").clone();
    let (secret, code) = story.ask();
    story.read(BROWSER, code, ApprovalState::Waiting { asked: wall(story.now) });
    story.decide(BROWSER, code, Decided::Approved);
    story.read(BROWSER, code, ApprovalState::Approved);
    assert_eq!(story.front.session.sign_ins(), before, "approving makes no sign-in");
    story.take(&secret, Signing::SignedIn { party: 1 });
    story.assert_me(TERMINAL, 1);
    story.assert_me(BROWSER, 1);
    let credential = story.front.session.credential(TERMINAL).expect("take installs the credential").clone();
    assert_eq!(credential.len(), 40);
    assert_ne!(credential, browser_credential);
    assert_eq!(story.front.session.credential(BROWSER), Some(&browser_credential));
    let after = story.front.session.sign_ins();
    assert_eq!(after.len(), before.len() + 1);
    assert!(before.iter().all(|row| after.contains(row)));
    assert!(after.iter().all(|(_, party)| *party == 1));
    story.take(&secret, Signing::Approval(Taking::Used));
    assert_eq!(story.front.session.credential(TERMINAL), Some(&credential));
    assert_eq!(story.front.session.sign_ins(), after);
    story.read(BROWSER, code, ApprovalState::Taken);
}

#[test]
fn a_decision_from_a_client_that_isnt_a_browser_is_refused() {
    let mut story = Story::new();
    Front::<Test>::bind(&mut story.front, BEARER, 1);
    story.add_client(BEARER);
    story.settle();
    story.assert_me(BEARER, 1);
    assert!(story.front.session.credential(BEARER).is_some(), "the non-browser holds a valid bearer");
    let before = story.front.session.sign_ins();
    let (secret, code) = story.ask();
    story.read(BEARER, code, ApprovalState::NotABrowser);
    story.decide(BEARER, code, Decided::NotABrowser);
    story.take(&secret, Signing::Approval(Taking::Waiting));
    assert_eq!(story.front.session.credential(TERMINAL), None);
    assert_eq!(story.clients[&TERMINAL].link().state, Link::SignedOut);
    assert_eq!(story.front.session.sign_ins(), before);
    story.decide(BROWSER, code, Decided::Approved);
    story.take(&secret, Signing::SignedIn { party: 1 });
    story.assert_me(TERMINAL, 1);
}

#[test]
fn an_approval_never_taken_makes_no_sign_in() {
    let mut story = Story::new();
    let before = story.front.session.sign_ins();
    let (secret, code) = story.ask();
    story.decide(BROWSER, code, Decided::Approved);
    assert_eq!(story.front.session.sign_ins(), before);
    story.now = story.now.saturating_add(SPAN).saturating_add(Duration::from_nanos(1));
    story.settle();
    assert_eq!(story.front.session.driver.world.clocks(), (story.now, wall(story.now)));
    assert_ne!(story.front.session.component_deadline(), Some(Time::ZERO.saturating_add(SPAN)));
    story.take(&secret, Signing::Approval(Taking::Expired));
    story.read(BROWSER, code, ApprovalState::Expired);
    assert_eq!(story.front.session.credential(TERMINAL), None);
    assert_eq!(story.clients[&TERMINAL].link().state, Link::SignedOut);
    assert_eq!(story.front.session.sign_ins(), before, "never taking creates no real people sign-in");
    story.assert_me(BROWSER, 1);
}
