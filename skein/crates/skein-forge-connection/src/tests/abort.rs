//! Abort's call terminals, escalation and io settlement (programming-model.md, 5.2).
use super::fixtures as f;
use crate::{Answered, Component, Event, Limits, Request};
use skein_forge::api;
use skein_io::{self as io, Addr};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, stream};

#[test]
fn abort_while_live_cancels_every_pending_call_once_before_closed() {
    // Both waiting for a forge's start and exchanging calls belong to the component.
    for ready in [false, true] {
        let mut h = if ready { f::known() } else { f::Harness::new(f::settings()) };
        h.run(Duration::ZERO);
        for call in [1, 2] {
            h.call(call, f::REPO, api::Op::Read(api::Read::Settings));
        }
        if ready {
            h.run(Duration::ZERO);
            h.connected(f::FORGE);
        }
        h.send(Request::Abort);
        for call in [1, 2] {
            let Some(Event::Answered(answer)) = h.up.pop() else {
                unreachable!("each held call ends before Closed");
            };
            assert_eq!(
                answer,
                Answered {
                    call: Token::new(call),
                    cost: u32::from(ready),
                    free: 0,
                    rate: None,
                    result: Err(if ready { api::Error::Timeout } else { api::Error::Unavailable }),
                },
            );
        }
        assert!(h.up.is_empty(), "Closed waits for io");
        assert_eq!(h.io.len(), h.peers.len(), "every kept socket is aborted");
        for request in &h.io {
            let io::Request::Abort { .. } = request else { unreachable!("every socket aborted: {request:?}") };
        }
        assert_eq!(h.component.next_deadline(), None, "settlement waits on io");
        h.run(Duration::ZERO);
        assert!(h.closed);
        assert!(h.peers.is_empty());
        h.run(Duration::from_secs(20));
        assert!(h.answers.is_empty(), "no second terminal after io settles");
        assert_eq!(h.component.next_deadline(), None);
    }

    let mut hooks = Hooks::bound();
    let owner = hooks.accept(SOCKET);
    hooks.send(Request::Abort);
    hooks.expect_abort(LISTENER);
    hooks.expect_abort(SOCKET);
    assert!(hooks.io.is_empty());
    assert_eq!(hooks.component.next_deadline(), None);
    hooks.settle(&[hooks.owner, owner]);
}

#[test]
fn abort_after_close_escalates_every_remaining_graceful_entity() {
    let mut h = f::Harness::ready();
    h.run(Duration::ZERO);
    h.send(Request::Close);
    let Some(io::Request::Close { entity }) = h.io.pop() else {
        unreachable!("an idle forge socket closes gracefully");
    };
    assert!(h.io.is_empty());
    h.send(Request::Abort);
    let Some(io::Request::Abort { entity: aborted }) = h.io.pop() else {
        unreachable!("abort escalates the forge socket");
    };
    assert_eq!(aborted, entity);
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    assert_eq!(h.component.next_deadline(), None);
    h.io.push(io::Request::Abort { entity });
    h.run(Duration::ZERO);
    assert!(h.closed);

    let mut hooks = Hooks::bound();
    let owner = hooks.accept(SOCKET);
    // A stream fault puts the webhook into an ordinary graceful io close.
    hooks.event(io::Event::Stream { owner, up: stream::Up::Failed(stream::Fault::Reset) });
    let Some(io::Request::Close { entity }) = hooks.io.pop() else {
        unreachable!("the webhook closes gracefully after its stream fails");
    };
    assert_eq!(entity, SOCKET);
    assert!(hooks.io.is_empty());
    hooks.send(Request::Close);
    let Some(io::Request::Close { entity }) = hooks.io.pop() else {
        unreachable!("the listener closes gracefully");
    };
    assert_eq!(entity, LISTENER);
    assert!(hooks.io.is_empty());
    hooks.send(Request::Abort);
    hooks.expect_abort(LISTENER);
    hooks.expect_abort(SOCKET);
    assert!(hooks.io.is_empty(), "exactly one escalation per graceful entity");
    hooks.settle(&[owner, hooks.owner]);
}

#[test]
fn close_after_abort_changes_nothing() {
    let mut h = Hooks::bound();
    let owner = h.accept(SOCKET);
    h.send(Request::Abort);
    h.expect_abort(LISTENER);
    h.expect_abort(SOCKET);
    h.send(Request::Close);
    h.component.fire(&h.env, &mut h.up, &mut h.io);
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    assert_eq!(h.component.next_deadline(), None);
    h.settle(&[owner, h.owner]);
}

#[test]
fn a_second_abort_changes_nothing() {
    let mut h = Hooks::bound();
    let owner = h.accept(SOCKET);
    h.send(Request::Abort);
    h.expect_abort(LISTENER);
    h.expect_abort(SOCKET);
    h.send(Request::Abort);
    h.event(io::Event::Failed { owner, error: io::Error::Refused });
    h.component.fire(&h.env, &mut h.up, &mut h.io);
    assert!(h.io.is_empty());
    assert!(h.up.is_empty());
    assert_eq!(h.component.next_deadline(), None);
    h.settle(&[h.owner, owner]);
}

#[test]
fn requests_after_abort_are_ignored_as_after_close() {
    let mut h = f::Harness::new(f::settings());
    h.call(1, f::REPO, api::Op::Read(api::Read::Settings));
    h.send(Request::Abort);
    let Some(Event::Answered(answer)) = h.up.pop() else { unreachable!("cancelled call") };
    assert_eq!(answer.result, Err(api::Error::Unavailable));
    // Take the abort out of the queue while io is still settling.
    let Some(abort @ io::Request::Abort { .. }) = h.io.pop() else { unreachable!("abort socket") };
    assert!(h.io.is_empty());
    for settled in [false, true] {
        let access = f::settings().forges.into_boxed().into_iter().next().expect("forge").access.expect("access");
        h.call(2, f::REPO, api::Op::Read(api::Read::Settings));
        h.send(Request::Connect { forge: f::FORGE, access });
        h.send(Request::Installations { forge: f::FORGE, accepted: Box::new([]) });
        h.send(Request::Refused(answer.clone()));
        h.send(Request::Returned(answer.clone()));
        h.send(Request::RefusedHint(crate::hooks::Hinted {
            connection: Token::new(100),
            hint: api::Hint { repository: f::REPO, change: api::Change::Item(1), key: None },
        }));
        h.send(Request::Offer);
        h.send(Request::Close);
        h.send(Request::Abort);
        assert!(h.up.is_empty(), "ignored requests emit nothing, settled={settled}");
        assert!(h.io.is_empty());
        assert_eq!(h.component.next_deadline(), None);
        if !settled {
            // The only outstanding socket belongs to the start.
            let owner = *h.peers.first().expect("start socket").0;
            h.component.up(&h.env, io::Event::Closed { owner }, &mut h.up, &mut h.io);
            let Some(Event::Closed) = h.up.pop() else { unreachable!("Closed after settlement") };
        }
    }
    drop(abort);
}

#[test]
fn abort_keeps_already_answered_calls_from_receiving_another_terminal() {
    let mut h = f::Harness::new(f::settings());
    let answer = Answered { call: Token::new(1), cost: 2, free: 1, rate: None, result: Err(api::Error::Unavailable) };
    h.send(Request::Refused(answer.clone()));
    h.send(Request::Returned(Answered { call: Token::new(2), ..answer }));
    h.send(Request::Abort);
    assert!(h.up.is_empty(), "refused and returned answers have their terminal already");
    h.run(Duration::ZERO);
    assert!(h.closed);
    assert!(h.answers.is_empty());
}

#[test]
fn abort_before_io_announces_entities_waits_and_aborts_their_late_handles() {
    let settings = f::hook_settings();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: f::limits() };
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    let address = settings.webhooks.as_ref().expect("hooks").listener;
    let mut component = Component::new(settings, &env.limits, &f::shared()).expect("fixture limits");
    component.begin(&env, &mut up, &mut io);
    let Some(io::Request::Listen { owner: listener_owner, .. }) = io.pop() else { unreachable!("listen") };
    let Some(io::Request::Connect { owner: forge_owner, .. }) = io.pop() else { unreachable!("connect") };
    component.down(&env, Request::Abort, &mut up, &mut io);
    assert!(up.is_empty());
    assert!(io.is_empty(), "no handles announced yet");
    assert_eq!(component.next_deadline(), None);
    component.up(&env, io::Event::Connecting { owner: forge_owner, socket: SOCKET }, &mut up, &mut io);
    let Some(io::Request::Abort { entity }) = io.pop() else { unreachable!("late socket aborted") };
    assert_eq!(entity, SOCKET);
    component.up(
        &env,
        io::Event::Listening { owner: listener_owner, listener: LISTENER, addr: address },
        &mut up,
        &mut io,
    );
    let Some(io::Request::Abort { entity }) = io.pop() else { unreachable!("late listener aborted") };
    assert_eq!(entity, LISTENER);
    assert!(up.is_empty(), "no Listening after abort");
    assert!(io.is_empty());
    component.up(&env, io::Event::Closed { owner: listener_owner }, &mut up, &mut io);
    assert!(up.is_empty(), "the late socket still settles");
    component.up(&env, io::Event::Closed { owner: forge_owner }, &mut up, &mut io);
    let Some(Event::Closed) = up.pop() else { unreachable!("Closed after settlement") };
    assert!(up.is_empty());
    assert!(io.is_empty());
}

#[test]
fn an_accept_racing_abort_is_bound_aborted_and_settled() {
    let mut h = Hooks::bound();
    h.send(Request::Abort);
    h.expect_abort(LISTENER);
    let owner = h.accept(SOCKET);
    h.expect_abort(SOCKET);
    assert!(h.io.is_empty());
    h.settle(&[h.owner, owner]);
}

#[test]
fn abort_of_an_accept_held_by_a_full_slab_does_not_wait_for_room() {
    for close_first in [false, true] {
        let mut h = Hooks::bound();
        let mut owners =
            skein_lib::List::with_capacity(h.env.limits.hooks.checked_add(2).expect("hooks plus accept and listener"));
        for number in 0..h.env.limits.hooks {
            let socket = Token::new(100_u64.checked_add(u64::from(number)).expect("fixture socket"));
            let owner = h.accept(socket);
            owners.push(owner).expect("bounded hooks");
        }
        let pending_socket = Token::new(200);
        h.event(io::Event::Accepted {
            owner: h.owner,
            socket: pending_socket,
            peer: Addr::Ip(([127, 0, 0, 1], 7201).into()),
        });
        assert!(h.io.is_empty(), "the live listener waits for room");
        h.send(if close_first { Request::Close } else { Request::Abort });
        if close_first {
            h.expect_close(LISTENER);
        } else {
            h.expect_abort(LISTENER);
        }
        let Some(io::Request::Bind { owner, socket }) = h.io.pop() else {
            unreachable!("bind the pending accept");
        };
        assert_eq!(socket, pending_socket);
        if close_first {
            h.expect_close(pending_socket);
        } else {
            h.expect_abort(pending_socket);
        }
        for number in 0..h.env.limits.hooks {
            h.expect_abort(Token::new(100_u64.checked_add(u64::from(number)).expect("fixture socket")));
        }
        if close_first {
            h.send(Request::Abort);
            h.expect_abort(LISTENER);
            h.expect_abort(pending_socket);
        }
        assert!(h.io.is_empty(), "every socket is aborted before any settles");
        assert_eq!(h.component.next_deadline(), None);
        owners.push(h.owner).expect("listener fits");
        owners.push(owner).expect("pending accept fits");
        h.settle(owners.as_slice());
    }
}

#[test]
fn abort_ends_github_renewal_without_a_renewal_or_poller_deadline() {
    let mut h = super::github_renewal::renewing();
    assert!(h.component.next_deadline().is_some(), "renewal is active");
    h.send(Request::Abort);
    assert_eq!(h.io.len(), h.peers.len());
    for request in &h.io {
        let io::Request::Abort { .. } = request else { unreachable!("abort each GitHub socket") };
    }
    assert!(h.up.is_empty());
    assert_eq!(h.component.next_deadline(), None);
    h.run(Duration::ZERO);
    assert!(h.closed);
    h.run(Duration::from_secs(7200));
    assert_eq!(h.component.next_deadline(), None);
    assert!(h.peers.is_empty());
}

const LISTENER: Token = Token::new(90);
const SOCKET: Token = Token::new(100);

/// The close harness's boundary, with io's Closed events held back for inspection.
struct Hooks {
    component: Component,
    env: Env<Limits>,
    up: Queue<Event>,
    io: Queue<io::Request>,
    owner: Token,
}
impl Hooks {
    fn bound() -> Self {
        let mut settings = f::hook_settings();
        settings.forges.clear();
        let address = settings.webhooks.as_ref().expect("hooks").listener;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: f::limits() };
        let mut component = Component::new(settings, &env.limits, &f::shared()).expect("fixture limits");
        let mut up = Queue::uncapped();
        let mut io = Queue::uncapped();
        component.begin(&env, &mut up, &mut io);
        let Some(io::Request::Listen { owner, .. }) = io.pop() else { unreachable!("listen") };
        assert!(io.is_empty());
        component.up(&env, io::Event::Listening { owner, listener: LISTENER, addr: address }, &mut up, &mut io);
        let Some(Event::Listening { .. }) = up.pop() else { unreachable!("Listening after bind") };
        assert!(up.is_empty());
        Self { component, env, up, io, owner }
    }
    fn event(&mut self, event: io::Event) {
        self.component.up(&self.env, event, &mut self.up, &mut self.io);
    }
    fn send(&mut self, request: Request) {
        self.component.down(&self.env, request, &mut self.up, &mut self.io);
    }
    fn accept(&mut self, socket: Token) -> Token {
        self.event(io::Event::Accepted { owner: self.owner, socket, peer: Addr::Ip(([127, 0, 0, 1], 7201).into()) });
        let Some(io::Request::Bind { owner, socket: bound }) = self.io.pop() else { unreachable!("bind accept") };
        assert_eq!(bound, socket);
        // An active hook states its read demand; an abort races directly to Abort.
        if let Some(io::Request::Stream { .. }) = self.io.iter().next() {
            drop(self.io.pop());
        }
        owner
    }
    fn expect_close(&mut self, expected: Token) {
        let Some(io::Request::Close { entity }) = self.io.pop() else { unreachable!("io close") };
        assert_eq!(entity, expected);
    }
    fn expect_abort(&mut self, expected: Token) {
        let Some(io::Request::Abort { entity }) = self.io.pop() else { unreachable!("io abort") };
        assert_eq!(entity, expected);
    }
    fn settle(&mut self, owners: &[Token]) {
        assert!(self.up.is_empty(), "Closed waits for io");
        for (at, owner) in owners.iter().enumerate() {
            self.event(io::Event::Closed { owner: *owner });
            if at.checked_add(1) == Some(owners.len()) {
                let Some(Event::Closed) = self.up.pop() else { unreachable!("Closed after last settlement") };
            }
            assert!(self.up.is_empty(), "only the last settlement reports Closed");
        }
        self.env.now = self.env.now.saturating_add(Duration::from_secs(20));
        self.component.fire(&self.env, &mut self.up, &mut self.io);
        assert!(self.up.is_empty());
        assert!(self.io.is_empty());
        assert_eq!(self.component.next_deadline(), None);
    }
}
