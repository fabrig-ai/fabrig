//! Refusal cleanup through the service's actual handler and routed store answers.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary host tests construct and inspect scripted protocol streams"
)]
use super::*;
use crate::tests::fixtures;
use skein_fake_oauth as fake;
use skein_lib::{Duration, bytes};

// temper/docs/design/protocol/credentials.md, 3; skein/docs/design/oauth.md, 6.5.
// The host runs over scripted streams; the independent issuer checks OAuth.
struct SignInWorld {
    service: crate::Service,
    accounts: Accounts,
    issuer: fake::Issuer,
    sockets: Vec<SignInSocket>,
    listeners: Vec<(Token, Token, io::Addr)>,
    events: Queue<root::Event>,
    now: Time,
    exchanges: u32,
    closed_listeners: u32,
}

struct SignInSocket {
    token: Token,
    owner: Token,
    input: skein_lib::Intake,
    demand: Option<(skein_lib::stream::Read, u32)>,
    granted: Option<u32>,
    sent: Vec<u8>,
    answered: bool,
    closed: bool,
    native: bool,
}

impl SignInSocket {
    fn new(token: Token, owner: Token, native: bool) -> Self {
        Self {
            token,
            owner,
            input: skein_lib::Intake::with_capacity(65_536),
            demand: None,
            granted: None,
            sent: Vec::new(),
            answered: false,
            closed: false,
            native,
        }
    }

    fn down(&mut self, down: skein_lib::stream::Down) {
        use skein_lib::stream::{Down, Read};
        match down {
            Down::Demand { read: Read::Nothing, room: 0 } => {
                assert!(self.demand.take().is_some());
            }
            Down::Demand { read, room } => {
                assert!(self.demand.replace((read, room)).is_none());
            }
            Down::Send(value) => {
                assert!(value.len() <= usize::try_from(self.granted.take().expect("granted room")).unwrap());
                self.sent.extend_from_slice(&value);
            }
            Down::Finish => {}
        }
    }

    fn up(&mut self) -> Option<skein_lib::stream::Up> {
        let (read, room) = self.demand?;
        if let Some(value) = self.input.meet(read) {
            self.demand = None;
            Some(skein_lib::stream::Up::Bytes(value))
        } else if room > 0 {
            self.demand = None;
            self.granted = Some(room);
            Some(skein_lib::stream::Up::Room)
        } else {
            None
        }
    }
}

impl SignInWorld {
    fn new() -> Self {
        let (config, runtime) = registered();
        let document = config.limits.driver.client.document;
        let issuer = fake::Issuer::new(
            fake::Config {
                authorization_url: b"http://127.0.0.1:7102/authorize".as_slice().into(),
                token_endpoint: b"http://127.0.0.1:7102/token".as_slice().into(),
                client_id: b"client".as_slice().into(),
                client_secret: None,
                redirect_uri: b"http://127.0.0.1:0/callback".as_slice().into(),
                refresh_token: b"throwaway-refresh".as_slice().into(),
                terms: Some(fake::Terms {
                    issued: Box::new([b"issued-client".as_slice().into()]),
                    omit_issued: false,
                    revocation_endpoint: None,
                    varying_port: true,
                    openid: Some(fake::OpenId {
                        issuer: b"http://127.0.0.1:7102".as_slice().into(),
                        key_set_url: b"http://127.0.0.1:7102/keys".as_slice().into(),
                        subject: b"account-subject".as_slice().into(),
                        lifetime: Duration::from_secs(3600),
                        header_kid: true,
                        fault: None,
                    }),
                }),
            },
            fake::Limits {
                document,
                uri_bytes: 8192,
                request_bytes: 16_384,
                codes: 2,
                rotations: 2,
                plans: 2,
                carried: 4,
            },
        )
        .unwrap();
        let mut service = fixtures::started();
        fixtures::restart(&mut service);
        let mut accounts = host(&config, runtime);
        pump_accounts(&mut accounts, service.engine.as_mut().unwrap());
        Self {
            service,
            accounts,
            issuer,
            sockets: Vec::new(),
            listeners: Vec::new(),
            events: Queue::uncapped(),
            now: Time::from_nanos(1),
            exchanges: 0,
            closed_listeners: 0,
        }
    }

    fn turn(&mut self) {
        let engine = self.service.engine.as_mut().unwrap();
        while let Some(event) = engine.store.above.pop() {
            engine.store.sources.push(event);
        }
        self.accounts.iterate(
            self.now,
            Wall::from_nanos(self.now.as_nanos()),
            &mut engine.values,
            &mut self.events,
            &mut engine.store,
        );
        // Each turn consumes only the io it began with.
        for _ in 0..self.accounts.io.len() {
            match self.accounts.io.pop().unwrap() {
                io::Request::Listen { owner, addr } => {
                    let io::Addr::Ip(mut address) = addr else { panic!("IP listener") };
                    address.set_port(41000_u16.checked_add(u16::try_from(self.listeners.len()).unwrap()).unwrap());
                    let listener =
                        Token::new(100_u64.checked_add(u64::try_from(self.listeners.len()).unwrap()).unwrap());
                    let addr = io::Addr::Ip(address);
                    self.listeners.push((owner, listener, addr));
                    self.accounts.up(io::Event::Listening { owner, listener, addr });
                }
                io::Request::Connect { owner, .. } => {
                    let socket = Token::new(1000_u64.checked_add(u64::try_from(self.sockets.len()).unwrap()).unwrap());
                    self.sockets.push(SignInSocket::new(socket, owner, false));
                    self.accounts.up(io::Event::Connecting { owner, socket });
                    self.accounts.up(io::Event::Connected { owner });
                }
                io::Request::Bind { socket, owner } => {
                    self.sockets.iter_mut().find(|peer| peer.token == socket).unwrap().owner = owner;
                }
                io::Request::Stream { stream, down } => {
                    self.sockets.iter_mut().find(|peer| peer.token == stream).unwrap().down(down);
                }
                io::Request::Close { entity } | io::Request::Abort { entity } => {
                    let owner = if let Some(peer) = self.sockets.iter_mut().find(|peer| peer.token == entity) {
                        assert!(!peer.closed);
                        peer.closed = true;
                        peer.owner
                    } else {
                        self.closed_listeners = self.closed_listeners.checked_add(1).unwrap();
                        self.listeners.iter().find(|(_, token, _)| *token == entity).unwrap().0
                    };
                    self.accounts.up(io::Event::Closed { owner });
                }
                io::Request::Reject { socket } => {
                    self.sockets.iter_mut().find(|peer| peer.token == socket).unwrap().closed = true;
                }
                io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. } => panic!("only sign-in socket io"),
            }
        }
        for peer in &mut self.sockets {
            if !peer.closed {
                if !peer.native
                    && !peer.answered
                    && let Some(request) = issuer_request(&peer.sent, self.now)
                {
                    if request.method == oauth::Method::Post {
                        self.exchanges = self.exchanges.checked_add(1).unwrap();
                    }
                    let mut out = Queue::uncapped();
                    let event = match request.method {
                        oauth::Method::Get => fake::Event::Get {
                            id: request.id,
                            endpoint: request.endpoint,
                            now: self.now,
                            wall: Wall::from_nanos(self.now.as_nanos()),
                        },
                        oauth::Method::Post => {
                            fake::Event::Post { request, now: self.now, wall: Wall::from_nanos(self.now.as_nanos()) }
                        }
                    };
                    self.issuer.step(event, &mut out);
                    let Some(fake::Request::Http(answer)) = out.pop() else { panic!("issuer HTTP answer") };
                    assert_eq!(answer.status, 200, "issuer accepts the host's request");
                    let head = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        answer.body.len()
                    );
                    peer.input.append(head.as_bytes()).unwrap();
                    peer.input.append(&answer.body).unwrap();
                    peer.answered = true;
                }
                if let Some(up) = peer.up() {
                    self.accounts.up(io::Event::Stream { owner: peer.owner, up });
                }
            }
        }
    }

    fn settle(&mut self) {
        // Each bounded HTTP document and its body take fewer than 512 host steps.
        for _ in 0_u32..512 {
            self.turn();
        }
        assert!(!self.accounts.work_pending(self.now), "the host settles at the supplied instant");
    }

    fn approve(&mut self, source: u32) -> Box<[u8]> {
        let visit = self.visit(source);
        self.approved_callback(visit)
    }

    fn visit(&mut self, source: u32) -> Box<[u8]> {
        self.accounts.request(domain::Request::Signing(domain::Signing::SignIn { source, provider: 3 }));
        self.settle();
        let mut visit = None;
        while let Some(event) = self.events.pop() {
            if let root::Event::Account(domain::Event::Visited { source: visited, address }) = event {
                assert_eq!(visited, source);
                visit = Some(address);
            }
        }
        visit.expect("visit before approval")
    }

    fn approved_callback(&mut self, visit: Box<[u8]>) -> Box<[u8]> {
        let mut out = Queue::uncapped();
        self.issuer.step(fake::Event::Authorize { url: visit, now: self.now }, &mut out);
        let Some(fake::Request::Redirect { uri, state, code, client_id }) = out.pop() else {
            panic!("issuer approval")
        };
        let mut callback = uri.to_vec();
        callback.extend_from_slice(b"?state=");
        callback.extend_from_slice(&state);
        callback.extend_from_slice(b"&code=");
        callback.extend_from_slice(&code);
        callback.extend_from_slice(b"&client_id=");
        callback.extend_from_slice(&client_id.expect("issued client"));
        callback.into_boxed_slice()
    }

    fn paste(&mut self, source: u32, callback: Box<[u8]>) -> u64 {
        let engine = self.service.engine.as_mut().unwrap();
        let number = carried(engine, store::Secrets::Hold { value: Secret::new(callback) });
        self.accounts.request(domain::Request::Signing(domain::Signing::Redirected { source, carried: number }));
        number
    }

    fn forget_paste(&mut self, number: u64) {
        let engine = self.service.engine.as_mut().unwrap();
        // The clients component owns this erase after the finish's terminal.
        let _erased = engine
            .store
            .secrets(Token::new(123), store::Secrets::Erase { owner: store::SecretOwner::Application, number });
        let Some(store::Event::Erased { .. }) = engine.store.above.pop() else { panic!("held address erased") };
        assert!(engine.store.above.is_empty());
        let Some(store::Event::Secret { value: None, .. }) = engine
            .store
            .secrets(Token::new(124), store::Secrets::Get { owner: store::SecretOwner::Application, number })
        else {
            panic!("no carried address remains")
        };
        assert!(!self.accounts.finishes.pending());
        assert!(self.accounts.finish_out.is_empty());
    }

    fn grant_plan(&mut self) {
        self.issuer
            .queue(fake::Plan {
                status: 200,
                body: fake::Body::Token(oauth::TokenResponse {
                    access_token: b"throwaway-access".as_slice().into(),
                    refresh_token: None,
                    expires_in: 3600,
                    scope: Some(b"openid read".as_slice().into()),
                    id_token: None,
                    earliest_refresh_at: None,
                }),
                delay: Duration::ZERO,
                retry_after: Duration::ZERO,
            })
            .unwrap();
    }

    fn native_socket(&mut self) -> Token {
        let (owner, _, addr) = self.listeners.last().unwrap();
        let socket = Token::new(1000_u64.checked_add(u64::try_from(self.sockets.len()).unwrap()).unwrap());
        self.sockets.push(SignInSocket::new(socket, *owner, true));
        self.accounts.up(io::Event::Accepted { owner: *owner, socket, peer: *addr });
        self.settle();
        socket
    }

    fn native_return(&mut self, socket: Token, callback: &[u8]) {
        // The already accepted browser may send after the listener stops waiting.
        let rest = callback.strip_prefix(b"http://").unwrap();
        let slash = bytes::find(rest, b"/").unwrap();
        let mut head = b"GET ".to_vec();
        head.extend_from_slice(&rest[slash..]);
        head.extend_from_slice(b" HTTP/1.1\r\nHost: ");
        head.extend_from_slice(&rest[..slash]);
        head.extend_from_slice(b"\r\nConnection: close\r\n\r\n");
        self.sockets.iter_mut().find(|peer| peer.token == socket).unwrap().input.append(&head).unwrap();
    }

    fn nothing_held(&self) {
        assert!(!self.accounts.finishes.pending());
        assert!(self.accounts.finish_out.is_empty());
        assert!(self.accounts.above.is_empty());
        assert!(self.accounts.io.is_empty());
        assert!(self.sockets.iter().all(|peer| peer.closed));
        assert_eq!(usize::try_from(self.closed_listeners).unwrap(), self.listeners.len());
    }
}

// Decode only a complete host HTTP request, without formatting its body.
fn issuer_request(sent: &[u8], now: Time) -> Option<oauth::HttpRequest> {
    let head_end = bytes::find(sent, b"\r\n\r\n")?.checked_add(4).unwrap();
    let head = core::str::from_utf8(&sent[..head_end]).unwrap();
    let mut lines = head.split("\r\n");
    let mut first = lines.next().unwrap().split(' ');
    let method = match first.next().unwrap() {
        "GET" => oauth::Method::Get,
        "POST" => oauth::Method::Post,
        _ => panic!("issuer GET or POST"),
    };
    let target = first.next().unwrap();
    let mut length = 0;
    for line in lines {
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = value.trim().parse::<usize>().unwrap();
        }
    }
    if sent.len() < head_end.checked_add(length).unwrap() {
        return None;
    }
    Some(oauth::HttpRequest {
        id: 1,
        method,
        endpoint: format!("http://127.0.0.1:7102{target}").into_bytes().into_boxed_slice(),
        content_type: b"application/x-www-form-urlencoded",
        body: sent[head_end..].into(),
        deadline: now,
    })
}

#[test]
fn restarting_after_approval_then_pasting_the_new_callback_succeeds() {
    let mut world = SignInWorld::new();
    let old = world.approve(71);
    world.accounts.request(domain::Request::Signing(domain::Signing::CancelSignIn { source: 71 }));
    let fresh = world.approve(72);
    let old_paste = world.paste(71, old);
    world.settle();
    assert_eq!(world.exchanges, 0, "the approved superseded code is never exchanged");
    let Some(root::Event::Account(domain::Event::FinishRefused {
        source: 71,
        why: domain::SignInRedirectWhy::NotWaiting,
    })) = world.events.pop()
    else {
        panic!("the superseded callback is refused")
    };
    assert!(world.events.is_empty());
    world.forget_paste(old_paste);
    world.grant_plan();
    let fresh_paste = world.paste(72, fresh);
    world.settle();
    let mut signed_in = 0_u32;
    let mut redirects = 0_u32;
    while let Some(event) = world.events.pop() {
        if let root::Event::Account(domain::Event::SignedIn { source: 72, .. }) = event {
            signed_in += 1;
        }
        if let root::Event::Account(domain::Event::Finishing { source: 72 }) = event {
            redirects += 1;
        }
    }
    assert_eq!(signed_in, 1);
    assert_eq!(redirects, 1);
    assert_eq!(world.exchanges, 1, "only the replacement callback reaches the issuer");
    assert_eq!(world.issuer.posts(), 1);
    world.forget_paste(fresh_paste);
    world.nothing_held();
}

#[test]
fn expiry_and_native_paste_race_take_at_most_one_redirect() {
    const SEED: u64 = 21;
    // 0: deadline tick, 1: native return, 2: paste. Shuffle ties from this test's seed.
    let mut orders = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
    let mut rng = skein_lib::Rng::new(SEED);
    for index in (1..orders.len()).rev() {
        let other = usize::try_from(rng.below(u64::try_from(index.checked_add(1).unwrap()).unwrap())).unwrap();
        orders.swap(index, other);
    }
    let mut schedules = 0_u32;
    for native_offset in [-1_i8, 0, 1] {
        for paste_offset in [-1_i8, 0, 1] {
            for order in orders {
                let context = format!("seed={SEED}, native={native_offset}, paste={paste_offset}, order={order:?}");
                let mut world = SignInWorld::new();
                let visit = world.visit(71);
                let deadline = world.accounts.next_deadline().expect("waiting sign-in's deadline").as_nanos();
                // Approve near the attempt's deadline so the issuer's shorter code
                // lifetime cannot expire first. The attempt's deadline stays fixed.
                world.now = Time::from_nanos(deadline.checked_sub(Duration::from_secs(1).as_nanos()).unwrap());
                let callback = world.approved_callback(visit);
                let socket = world.native_socket();
                world.grant_plan();
                let at = |offset| match offset {
                    -1 => deadline.checked_sub(1).unwrap(),
                    0 => deadline,
                    1 => deadline.checked_add(1).unwrap(),
                    _ => unreachable!(),
                };
                let times = [deadline, at(native_offset), at(paste_offset)];
                let mut schedule = order;
                // Stable ordering preserves the seeded permutation at equal instants.
                schedule.sort_by_key(|action| times[*action]);
                let mut held = None;
                for action in schedule {
                    world.now = Time::from_nanos(times[action]);
                    world.accounts.clock(world.now, Wall::from_nanos(world.now.as_nanos()));
                    match action {
                        0 => {}
                        1 => world.native_return(socket, &callback),
                        2 => held = Some(world.paste(71, callback.clone())),
                        _ => unreachable!(),
                    }
                    world.settle();
                }
                let mut pasted_redirects = 0;
                let mut signed_in = 0_u32;
                let mut expired = 0_u32;
                let mut refused = 0;
                while let Some(event) = world.events.pop() {
                    if let root::Event::Account(domain::Event::Finishing { source: 71 }) = event {
                        pasted_redirects += 1;
                    } else if let root::Event::Account(domain::Event::SignedIn { source: 71, .. }) = event {
                        signed_in += 1;
                    } else if let root::Event::Account(domain::Event::SignInFailed { source: 71, failure }) = event {
                        // The host translates the driver's TimedOut to Later (translate.rs).
                        assert_eq!(failure, domain::Failure::Later { after: None }, "{context}");
                        expired += 1;
                    } else if let root::Event::Account(domain::Event::FinishRefused { source: 71, why }) = event {
                        assert_eq!(why, domain::SignInRedirectWhy::NotWaiting, "{context}");
                        refused += 1;
                    }
                }
                let native_redirects =
                    world.sockets.iter().filter(|peer| peer.native && peer.sent.starts_with(b"HTTP/1.1 200")).count();
                let redirects = native_redirects.checked_add(pasted_redirects).unwrap();
                assert!(redirects <= 1, "{context}");
                assert!(world.exchanges <= 1, "{context}");
                assert_eq!(world.issuer.posts(), u64::from(world.exchanges), "{context}");
                if native_offset < 0 || paste_offset < 0 {
                    assert_eq!((redirects, world.exchanges, signed_in, expired), (1, 1, 1, 0), "{context}");
                    assert_eq!(pasted_redirects + refused, 1, "{context}");
                } else {
                    assert_eq!((redirects, world.exchanges, signed_in, expired, refused), (0, 0, 0, 1, 1), "{context}");
                }
                world.forget_paste(held.expect("one paste per schedule"));
                world.nothing_held();
                schedules += 1;
            }
        }
    }
    assert_eq!(schedules, 54);
}

#[test]
fn a_revoked_record_waits_for_erased_and_coalesces_a_sign_out() {
    for (notifications, expected_signed_out, removing) in
        [(&[][..], None, false), (&[true, true][..], Some(true), true), (&[false, false][..], Some(false), true)]
    {
        let mut service = fixtures::ready();
        let engine = service.engine.as_mut().expect("engine");
        let (config, runtime) = registered();
        let mut accounts = host(&config, runtime);
        accounts.request(domain::Request::Signing(domain::Signing::Restore {
            source: 1,
            provider: 3,
            kind: domain::Kind::SignIn,
        }));
        let candidate = accounts.env.limits.client.document;
        let records::Next::Store(old_keep) = accounts.records.keep(1, record(1, &candidate), &candidate) else {
            panic!("old record");
        };
        replace(&mut engine.store, old_keep);
        accounts.records.kept(1);
        while engine.store.above.pop().is_some() {}
        let mut events = Queue::with_capacity(16);
        let old = 1;
        let fresh = 9001;
        let candidate = accounts.env.limits.client.document;
        let mut record = record(fresh, &candidate);
        let records::Next::Store(keep) = accounts.records.keep(fresh, record.clone(), &candidate) else {
            panic!("fresh source is kept");
        };
        replace(&mut engine.store, keep);
        assert!(accounts.records.kept(fresh).is_some());
        // Hold unrelated store answers apart from this erase's terminal.
        while engine.store.above.pop().is_some() {}
        engine
            .values
            .accounts
            .granted(
                fresh,
                2,
                table::Value { token: Secret::new(bytes::copy_of(b"fresh-token")), scheme: table::Scheme::Bearer },
                Time::from_nanos(Duration::from_secs(60).as_nanos()),
            )
            .expect("fresh value fits");
        accounts.above.push(driver::Event::Failed {
            account: old,
            ends: driver::Ends::Refresh,
            failure: driver::Failure::Exchange(oauth::Failure::Refused),
        });
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        let Some(root::Event::Account(domain::Event::Component(domain::FromComponent::Failed {
            account: 1,
            failure: domain::Failure::SignInAgain,
        }))) = events.pop()
        else {
            panic!("revocation fails needs sign-in immediately");
        };
        assert!(engine.values.accounts.value(old, 1, Time::from_nanos(1)).is_none());
        let Some(Erasing::Revoked) = accounts.erasing.get(&old) else {
            panic!("erase awaits durability");
        };
        record.key = old;
        let records::Next::Store(before) = accounts.records.keep(old, record.clone(), &candidate) else {
            panic!("the old slot remains admitted");
        };
        assert!(before.replaces.is_some(), "submission did not clear the held digest");
        assert!(accounts.records.stale(old).is_some());
        for confirmed in notifications {
            accounts.above.push(driver::Event::SignedOut { account: old, revoked: *confirmed });
        }
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        assert_eq!(engine.store.above.len(), 1, "one erase for revocation and concurrent sign-out");
        assert!(events.is_empty(), "sign-out awaits erasure");
        let terminal = engine.store.above.pop().expect("held erase acknowledgement");
        let store::Event::Erased { .. } = terminal else {
            panic!("held erase acknowledgement");
        };
        engine.store.sources.push(terminal);
        accounts.answers(&mut engine.store, &mut events);
        // A duplicate acknowledgement and an unknown old-source acknowledgement do nothing.
        engine.store.sources.push(store::Event::Erased { owner: Token::new(PREFIX | u64::from(old)) });
        engine.store.sources.push(store::Event::Erased { owner: Token::new(PREFIX | u64::from(8000_u32)) });
        accounts.answers(&mut engine.store, &mut events);
        assert_eq!(signed_out(&mut events, old), expected_signed_out);
        let records::Next::Store(after) = accounts.records.keep(old, record, &candidate) else {
            panic!("erasure retained the slot");
        };
        assert_eq!(after.replaces, None, "only the acknowledgement cleared the digest");
        assert!(accounts.required.contains_key(&old), "only Removed frees the driver's place");
        assert_eq!(accounts.erasing.contains_key(&old), removing);
        assert_eq!(accounts.above.is_empty(), !removing, "only sign-out asks the driver to remove its account");
        assert!(engine.values.accounts.value(fresh, 2, Time::from_nanos(1)).is_some());
        let Some(store::Event::Secret { value: Some(_), .. }) = engine.store.secrets(
            Token::new(99),
            store::Secrets::Get { owner: store::SecretOwner::Source, number: u64::from(fresh) },
        ) else {
            panic!("the fresh different source keeps its record");
        };
    }
}

fn record(source: u32, limits: &oauth::Limits) -> oauth::SavedToken {
    let record = oauth::SavedToken {
        key: source,
        generation: 2,
        access_token: bytes::copy_of(b"fresh-token"),
        refresh_token: Some(bytes::copy_of(b"fresh-refresh-token")),
        metadata: None,
        expires_at: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        issued_client_id: None,
        granted_scopes: Some(bytes::copy_of(b"openid read")),
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
    };
    assert!(oauth::encode_record(&record, limits).is_ok(), "record fits");
    record
}
fn signed_out(events: &mut Queue<root::Event>, old: u32) -> Option<bool> {
    let event = events.pop()?;
    let root::Event::Account(domain::Event::SignedOut { source, confirmed }) = event else {
        panic!("only a sign-out terminal follows erasure");
    };
    assert_eq!(source, old);
    assert!(events.is_empty(), "one terminal despite duplicate acknowledgements");
    Some(confirmed)
}

fn carried(engine: &mut crate::engine::Engine, request: store::Secrets) -> u64 {
    let _later = engine.store.secrets(Token::new(123), request);
    let Some(store::Event::Carried { number, .. }) = engine.store.above.pop() else { panic!("carried LLM key") };
    number
}
fn carry_llm_key(engine: &mut crate::engine::Engine, value: &[u8]) -> u64 {
    carried(engine, store::Secrets::Carry { kind: store::CarriedKind::Key, value: Secret::new(value.into()) })
}
fn held_address(engine: &mut crate::engine::Engine) -> u64 {
    carried(
        engine,
        store::Secrets::Hold {
            value: Secret::new(b"http://127.0.0.1:7103/callback?state=fake&code=fake".as_slice().into()),
        },
    )
}
fn pump_accounts(accounts: &mut Accounts, engine: &mut crate::engine::Engine) -> List<domain::FromComponent> {
    let mut events = Queue::with_capacity(64);
    let mut heard = List::with_capacity(64);
    for _ in 0_u32..32 {
        while let Some(event) = engine.store.above.pop() {
            engine.store.sources.push(event);
        }
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
        while let Some(event) = events.pop() {
            if let root::Event::Account(domain::Event::Component(event)) = event {
                heard.push(event).expect("bounded component events");
            }
        }
    }
    heard
}
#[test]
fn llm_key_work_reaches_the_store_and_the_workers_table_through_the_service() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    let mut accounts = engine.accounts.take().expect("accounts component");
    let carried = carry_llm_key(engine, b"first-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: false }));
    assert!(has_llm_key_grant(&pump_accounts(&mut accounts, engine), 1));
    assert_eq!(
        engine.values.accounts.value(7, 1, Time::from_nanos(1)).expect("worker value").token.expose(),
        b"first-llm-key"
    );
    let carried = carry_llm_key(engine, b"next-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: true }));
    assert!(has_llm_key_grant(&pump_accounts(&mut accounts, engine), 2));
    accounts.request(domain::Request::Signing(domain::Signing::Forget { source: 7 }));
    pump_accounts(&mut accounts, engine);
    assert!(engine.values.accounts.value(7, 1, Time::from_nanos(1)).is_none());
    assert!(engine.values.accounts.value(7, 2, Time::from_nanos(1)).is_none());
    let Some(store::Event::Secret { value: None, .. }) =
        engine.store.secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Source, number: 7 })
    else {
        panic!("removed LLM key value")
    };
}
#[test]
fn service_restart_restores_an_llm_key_before_want_and_sweeps_only_unnamed_sources() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    let mut accounts = engine.accounts.take().expect("accounts component");
    let carried = carry_llm_key(engine, b"kept-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: false }));
    pump_accounts(&mut accounts, engine);
    engine.values.accounts.remove(7);
    let carried_orphan = carry_llm_key(engine, b"orphaned-carried-value");
    let _later = engine.store.secrets(
        Token::new(123),
        store::Secrets::Keep {
            owner: store::SecretOwner::Source,
            number: 8,
            value: jig_protocol_store::SecretValue::Record(Secret::new(b"orphaned-source-value".as_slice().into())),
        },
    );
    while engine.store.above.pop().is_some() {}
    let mut fresh_service = fixtures::started();
    let mut restarted = fresh_service.engine.as_mut().expect("engine").accounts.take().expect("new component");
    restarted.request(domain::Request::Signing(domain::Signing::Restore {
        source: 7,
        provider: 1,
        kind: domain::Kind::Key,
    }));
    restarted.request(domain::Request::Component(domain::ToComponent::Want { account: 7 }));
    restarted.request(domain::Request::Signing(domain::Signing::Restored));
    assert!(has_llm_key_grant(&pump_accounts(&mut restarted, engine), 1));
    assert_eq!(
        engine.values.accounts.value(7, 1, Time::from_nanos(1)).expect("restored value").token.expose(),
        b"kept-llm-key"
    );
    for (owner, number, exists) in [
        (store::SecretOwner::Source, 1, false),
        (store::SecretOwner::Source, 7, true),
        (store::SecretOwner::Source, 8, false),
        (store::SecretOwner::Application, carried_orphan, true),
    ] {
        let Some(store::Event::Secret { value, .. }) =
            engine.store.secrets(Token::new(123), store::Secrets::Get { owner, number })
        else {
            panic!("stored value")
        };
        assert_eq!(value.is_some(), exists, "{owner:?} {number}");
    }
}

fn has_llm_key_grant(heard: &List<domain::FromComponent>, wanted: u64) -> bool {
    for event in heard {
        if let domain::FromComponent::Granted {
            account: 7,
            generation,
            form: domain::Form::Credential(domain::Scheme::Key),
            ..
        } = event
            && *generation == wanted
        {
            return true;
        }
    }
    false
}

#[test]
fn aborting_does_not_submit_an_llm_key_adoption_to_the_closing_store() {
    let mut service = fixtures::ready();
    let engine = service.engine.as_mut().expect("engine");
    let mut accounts = engine.accounts.take().expect("accounts component");
    let carried = carry_llm_key(engine, b"aborted-llm-key");
    accounts.request(domain::Request::Signing(domain::Signing::Keep { source: 7, carried, replaces: false }));
    let mut events = Queue::with_capacity(64);
    accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
    assert!(!accounts.key_out.is_empty(), "adoption has not been submitted yet");
    accounts.abort();
    let heard = pump_accounts(&mut accounts, engine);
    assert!(!has_llm_key_grant(&heard, 1));
    assert!(accounts.is_empty(), "abort settles the component");
    let Some(store::Event::Secret { value: None, .. }) =
        engine.store.secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Source, number: 7 })
    else {
        panic!("unadopted LLM key")
    };
    let Some(store::Event::Secret { value: Some(_), .. }) = engine
        .store
        .secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Application, number: carried })
    else {
        panic!("carried value remains an orphan")
    };
}

fn registered() -> (config::Engine, AccountsRuntime) {
    let mut config = fixtures::engine().jig;
    config.llm.providers = Box::new([config::llm::Provider {
        number: 3,
        name: b"issuer".as_slice().into(),
        signed_in_plan: false,
        manage_usage: None,
        models: Box::new([]),
        keys: true,
        programs: Box::new([]),
        sign_in: Some(config::llm::Registration {
            issuer: b"http://127.0.0.1:7102".as_slice().into(),
            client: b"client".as_slice().into(),
            scopes: Box::new([b"openid".as_slice().into(), b"read".as_slice().into()]),
            name_hint: None,
            authorization: b"http://127.0.0.1:7102/authorize".as_slice().into(),
            token: b"http://127.0.0.1:7102/token".as_slice().into(),
            revocation: None,
            keys: b"http://127.0.0.1:7102/keys".as_slice().into(),
            endpoint_transport: config::Transport::Plaintext,
            must_grant: Box::new([b"read".as_slice().into()]),
            resource: None,
            redirect_path: b"callback".as_slice().into(),
            redirect_port: None,
        }),
    }]);
    let endpoint =
        || driver::Endpoint { address: ([127, 0, 0, 1], 7102).into(), transport: driver::Transport::Plaintext };
    let mut origins = List::with_capacity(1);
    origins.push(driver::Origin { uri: b"http://127.0.0.1:7102".as_slice().into(), endpoint: endpoint() }).unwrap();
    let mut providers = List::with_capacity(1);
    providers.push((3, endpoint(), origins)).unwrap();
    (config, AccountsRuntime { providers })
}
fn host(config: &config::Engine, runtime: AccountsRuntime) -> Accounts {
    Accounts::new(
        crate::system::ACCOUNT_APPLICATION.into(),
        config,
        &fixtures::seeds(),
        Some(runtime),
        &config.limits.store.secrets,
    )
    .expect("public registration fits")
}
#[test]
fn dynamic_sources_use_provider_templates_and_restore_their_kept_records() {
    let (config, runtime) = registered();
    let mut accounts = host(&config, runtime);
    let mut service = fixtures::started();
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().expect("engine");
    let _started = pump_accounts(&mut accounts, engine);
    // Source numbers are independent of the provider and configured source.
    accounts.request(domain::Request::Signing(domain::Signing::SignIn { source: 9001, provider: 3 }));
    let _started = pump_accounts(&mut accounts, engine);
    assert!(accounts.required.contains_key(&9001));
    assert_eq!(&**accounts.required.get(&9001).expect("required scopes"), b"read".as_slice());
    let mut saved = record(9001, &config.limits.driver.client.document);
    saved.metadata = Some(b"provider-subject".as_slice().into());
    let records::Next::Store(keep) = accounts.records.keep(9001, saved, &config.limits.driver.client.document) else {
        panic!("record reaches durable store");
    };
    replace(&mut engine.store, keep);
    let _kept = pump_accounts(&mut accounts, engine);
    let (config, runtime) = registered();
    let mut restored = host(&config, runtime);
    restored.request(domain::Request::Signing(domain::Signing::Restore {
        source: 9001,
        provider: 3,
        kind: domain::Kind::SignIn,
    }));
    restored.request(domain::Request::Signing(domain::Signing::Restored));
    let _restored = pump_accounts(&mut restored, engine);
    assert_eq!(&**restored.required.get(&9001).expect("restored scopes"), b"read".as_slice());
    assert!(!restored.inventory.active());
    restored.above.push(driver::Event::Failed {
        account: 9001,
        ends: driver::Ends::Refresh,
        failure: driver::Failure::Exchange(oauth::Failure::Refused),
    });
    let _revoked = pump_accounts(&mut restored, engine);
    let Some(store::Event::Secret { value: None, .. }) =
        engine.store.secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Source, number: 9001 })
    else {
        panic!("revoked row erased")
    };
    let (config, runtime) = registered();
    let mut restarted = host(&config, runtime);
    restarted.request(domain::Request::Signing(domain::Signing::Restore {
        source: 9001,
        provider: 3,
        kind: domain::Kind::SignIn,
    }));
    restarted.request(domain::Request::Signing(domain::Signing::Restored));
    restarted.request(domain::Request::Component(domain::ToComponent::Want { account: 9001 }));
    let heard = pump_accounts(&mut restarted, engine);
    let mut unloaded = false;
    for event in &heard {
        if let domain::FromComponent::Failed { account: 9001, failure: domain::Failure::Unloaded { .. } } = event {
            unloaded = true;
        }
    }
    assert!(unloaded);
    assert!(engine.values.accounts.value(9001, 2, Time::from_nanos(1)).is_none());
}
#[test]
fn missing_registration_runtime_is_refused_before_start() {
    let (config, _) = registered();
    assert!(
        Accounts::new(
            crate::system::ACCOUNT_APPLICATION.into(),
            &config,
            &fixtures::seeds(),
            None,
            &config.limits.store.secrets
        )
        .is_err()
    );
}

#[test]
fn a_restored_sign_ins_metadata_leaves_its_grant_a_bearer() {
    const SOURCE: u32 = 71;
    let (config, runtime) = registered();
    let mut saved = record(SOURCE, &config.limits.driver.client.document);
    saved.metadata = Some(bytes::copy_of(b"private-account-id"));
    saved.expires_at = Wall::from_nanos(Duration::from_secs(3_600).as_nanos());
    saved.issued_client_id = Some(bytes::copy_of(b"issued-client"));
    let mut service = fixtures::started();
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().expect("engine");
    let _kept = engine.store.secrets(
        Token::new(123),
        store::Secrets::Replace {
            owner: store::SecretOwner::Source,
            number: u64::from(SOURCE),
            value: jig_protocol_store::SecretValue::Record(Secret::new(
                oauth::encode_record(&saved, &config.limits.driver.client.document).expect("record fits"),
            )),
            replaces: None,
        },
    );
    while engine.store.above.pop().is_some() {}
    let mut accounts = host(&config, runtime);
    accounts.request(domain::Request::Signing(domain::Signing::Restore {
        source: SOURCE,
        provider: 3,
        kind: domain::Kind::SignIn,
    }));
    accounts.request(domain::Request::Signing(domain::Signing::Restored));
    accounts.request(domain::Request::Component(domain::ToComponent::Want { account: SOURCE }));
    let heard = pump_accounts(&mut accounts, engine);
    let mut granted = false;
    for event in &heard {
        if let domain::FromComponent::Granted { account: SOURCE, form, .. } = event {
            assert_eq!(*form, domain::Form::Credential(domain::Scheme::Bearer));
            granted = true;
        }
    }
    assert!(granted, "the sign-in announces a bearer grant: {heard:?}");
    let value =
        engine.values.accounts.value(SOURCE, saved.generation, Time::from_nanos(1)).expect("restored sign-in lent");
    let table::Scheme::Bearer = &value.scheme else { panic!("the sign-in lends a bearer") };
    assert_eq!(value.token.expose(), saved.access_token.as_ref());
}

fn pump_move_events(accounts: &mut Accounts, engine: &mut crate::engine::Engine) -> Queue<root::Event> {
    let mut events = Queue::with_capacity(64);
    for _ in 0_u32..32 {
        while let Some(event) = engine.store.above.pop() {
            engine.store.sources.push(event);
        }
        accounts.iterate(Time::from_nanos(1), Wall::from_nanos(1), &mut engine.values, &mut events, &mut engine.store);
    }
    events
}

fn moved_answer(engine: &mut crate::engine::Engine, bearer: &[u8], attempt: u8) -> Secret {
    use crate::tests::fixtures::http as peer;
    use jig_client_protocol_wire::v19 as w;
    use skein_lib::{Reader, Writer};
    let limits = engine.clients.env.limits.wire;
    let change = w::AccountChange::MoveOut(
        w::AccountChangeMoveOut::new(&limits, w::AccountChangeMoveOutParts { account: 19 }).unwrap(),
    );
    let ask = w::Ask::AccountChange(
        w::AskAccountChange::new(&limits, w::AskAccountChangeParts { holder: w::AccountHolder::Own, change }).unwrap(),
    );
    let mut bytes = Writer::new(usize::try_from(ask.measure()).unwrap());
    ask.encode(&mut bytes).unwrap();
    let keyed = w::Keyed::new(
        &limits,
        w::KeyedParts { key: [attempt; 16], owner: w::Owner::Jig, version: 19, request: bytes.finish(), value: None },
    )
    .unwrap();
    let mut bytes = Writer::new(usize::try_from(keyed.measure()).unwrap());
    keyed.encode(&mut bytes).unwrap();
    let mut call = peer::Peer::new(engine, b"POST", b"/jig/v1/request", &bytes.finish(), Some(bearer));
    call.pump(engine, false);
    let Some(root::Event::Party(root::PartyInput::Ask { reply_to, .. })) = engine.events.pop() else {
        panic!("move-out ask reaches the root")
    };
    engine.clients.down(jig_protocol_engine_clients::Event::Reply {
        to: reply_to,
        reply: jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::AccountMovingOut {
            account: 19,
            number: 23,
            source: 71,
        }),
    });
    call.pump(engine, false);
    let at = bytes::find(call.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
    let w::KeyedAnswer::FinalAnswer(answer) =
        w::KeyedAnswer::decode(&limits, &mut Reader::new(&call.received()[at..])).unwrap()
    else {
        panic!("terminal move-out answer")
    };
    let w::Final::Done(done) = w::Final::decode(&limits, &mut Reader::new(answer.record())).unwrap() else {
        panic!("move-out done")
    };
    let w::Outcome::Made(made) = done.outcome() else { panic!("made") };
    let w::Made::MovingOut(moving) = made.what() else { panic!("moving out") };
    assert_eq!((moving.account(), moving.number()), (19, 23));
    moving.record().clone()
}

#[test]
fn a_move_out_answers_with_its_record_once_and_a_move_in_is_adopted() {
    use crate::tests::fixtures::http as peer;
    use jig_client_protocol_wire::v19 as w;

    let mut service = fixtures::ready();
    let link = service.links().pop().expect("sign-in link");
    let engine = service.engine.as_mut().expect("engine");
    let body = peer::link_body(engine, &link);
    let mut login = peer::Peer::new(engine, b"POST", b"/jig/v1/sign-in/link", &body, None);
    login.pump(engine, true);
    let w::SignInAnswer::SignedIn(answer) = login.sign_in_answer(engine) else { panic!("signed in") };
    let bearer = peer::bearer(answer.token().as_ref().expect("credential").expose());

    let (config, runtime) = registered();
    let mut accounts = host(&config, runtime);
    let document = config.limits.driver.client.document;
    let mut saved = record(71, &document);
    saved.subject = Some(bytes::copy_of(b"owner-subject"));
    saved.expires_at = Wall::from_nanos(Duration::from_secs(3_600).as_nanos());
    let encoded = oauth::encode_record(&saved, &document).expect("record fits");
    let _kept = engine.store.secrets(
        Token::new(123),
        store::Secrets::Replace {
            owner: store::SecretOwner::Source,
            number: 71,
            value: jig_protocol_store::SecretValue::Record(Secret::new(encoded)),
            replaces: None,
        },
    );
    while engine.store.above.pop().is_some() {}
    accounts.request(domain::Request::Signing(domain::Signing::Restore {
        source: 71,
        provider: 3,
        kind: domain::Kind::SignIn,
    }));
    accounts.request(domain::Request::Signing(domain::Signing::Restored));
    pump_accounts(&mut accounts, engine);
    accounts.request(domain::Request::Signing(domain::Signing::Withdraw {
        source: 71,
        display: bytes::copy_of(b"moved account"),
    }));
    let mut events = pump_move_events(&mut accounts, engine);
    let mut withdrawn = false;
    while let Some(event) = events.pop() {
        if let root::Event::Account(domain::Event::Withdrawn { source: 71, readable: true }) = event {
            withdrawn = true;
        }
    }
    assert!(withdrawn, "the driver removal and store read reach the child");
    engine.accounts = Some(accounts);
    let mut file = None;
    for attempt in 0_u8..2 {
        let moved_record = moved_answer(engine, &bearer, attempt);
        if attempt == 0 {
            let moved = jig_protocol_accounts::moved::decode(moved_record.expose(), &document).expect("moved file");
            assert_eq!(moved.application.as_ref(), crate::system::ACCOUNT_APPLICATION);
            assert_eq!(moved.record.refresh_token, saved.refresh_token);
            file = Some(moved_record);
        } else {
            assert!(moved_record.expose().is_empty(), "the second exchange has no file");
        }
    }
    adopt_moved_record(engine, file.expect("first moved record"), &document, saved.refresh_token.as_deref().unwrap());
}

fn adopt_moved_record(
    engine: &mut crate::engine::Engine,
    file: Secret,
    document: &oauth::Limits,
    refresh_token: &[u8],
) {
    let carried = carried(engine, store::Secrets::Carry { kind: store::CarriedKind::Moved, value: file });
    let mut accounts = engine.accounts.take().unwrap();
    accounts.request(domain::Request::Signing(domain::Signing::Open { source: 72, carried }));
    let mut events = pump_move_events(&mut accounts, engine);
    let mut opened = false;
    while let Some(event) = events.pop() {
        if let root::Event::Account(domain::Event::Opened { source: 72, provider: 3, .. }) = event {
            opened = true;
        }
    }
    assert!(opened, "the carried file is read and opened");
    accounts.request(domain::Request::Signing(domain::Signing::Adopt { source: 72, carried }));
    events = pump_move_events(&mut accounts, engine);
    let mut adopted = false;
    while let Some(event) = events.pop() {
        if let root::Event::Account(domain::Event::Adopted { source: 72, kept: true }) = event {
            adopted = true;
        }
    }
    assert!(adopted, "adoption is acknowledged by the store");
    let Some(store::Event::Secret { value: Some(store::SecretValue::Record(value)), .. }) =
        engine.store.secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Source, number: 72 })
    else {
        panic!("adopted source record")
    };
    let adopted = oauth::decode_record(value.expose(), document).expect("adopted record");
    assert_eq!(adopted.key, 72);
    assert_eq!(adopted.refresh_token.as_deref(), Some(refresh_token));
    let Some(store::Event::Secret { value: None, .. }) = engine
        .store
        .secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Application, number: carried })
    else {
        panic!("adoption consumes the carried file")
    };
}

#[test]
fn a_finish_reads_the_held_address_and_hands_it_to_the_driver() {
    for routed in [false, true] {
        let mut service = fixtures::started();
        fixtures::restart(&mut service);
        let engine = service.engine.as_mut().expect("engine");
        let (config, runtime) = registered();
        let mut accounts = host(&config, runtime);
        pump_accounts(&mut accounts, engine);
        let carried = held_address(engine);
        accounts.request(domain::Request::Signing(domain::Signing::Redirected { source: 71, carried }));
        accounts.start(&mut engine.store, &mut engine.values);
        let mut events = Queue::with_capacity(64);
        if routed {
            let Some(finishes::Next::Get { exchange, carried }) = accounts.finish_out.pop() else {
                panic!("finish asks for the held value");
            };
            let answer = engine.store.secrets(
                Token::new(FINISH_EXCHANGE | exchange.raw()),
                store::Secrets::Get { owner: store::SecretOwner::Application, number: carried },
            );
            engine.store.sources.push(answer.expect("held value answer"));
            accounts.answers(&mut engine.store, &mut events);
        } else {
            accounts.finish_steps(&mut events, &mut engine.store);
        }
        let Some(finishes::Next::Driver(request)) = accounts.finish_out.pop() else {
            panic!("the held value makes a driver request");
        };
        let driver::Request::Redirected { account, .. } = &request else {
            panic!("the driver receives a redirect");
        };
        assert_eq!(*account, 71);
        accounts.finish_out.push(finishes::Next::Driver(request));
        accounts.finish_steps(&mut events, &mut engine.store);
        assert!(accounts.finish_out.is_empty());
        // No sign-in is waiting for this source: the driver's refusal proves
        // that the request went down, without inspecting or formatting its URI.
        let mut events = pump_move_events(&mut accounts, engine);
        let Some(root::Event::Account(domain::Event::FinishRefused {
            source: 71,
            why: domain::SignInRedirectWhy::NotWaiting,
        })) = events.pop()
        else {
            panic!("a finish without a waiting sign-in is refused");
        };
        assert!(events.is_empty());
        let Some(store::Event::Secret { value: Some(_), .. }) = engine
            .store
            .secrets(Token::new(123), store::Secrets::Get { owner: store::SecretOwner::Application, number: carried })
        else {
            panic!("only the clients component erases the held value");
        };
    }
}

#[test]
fn a_finish_whose_held_address_is_gone_is_refused_not_waiting() {
    let mut service = fixtures::started();
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().expect("engine");
    let (config, runtime) = registered();
    let mut accounts = host(&config, runtime);
    accounts.request(domain::Request::Signing(domain::Signing::Redirected { source: 71, carried: 9001 }));
    let mut events = pump_move_events(&mut accounts, engine);
    let Some(root::Event::Account(domain::Event::FinishRefused {
        source: 71,
        why: domain::SignInRedirectWhy::NotWaiting,
    })) = events.pop()
    else {
        panic!("a finish whose held value is gone is refused");
    };
    assert!(events.is_empty());
    assert!(!accounts.finishes.pending());
    assert!(accounts.above.is_empty());
}

#[test]
fn the_drivers_taken_and_refusals_reach_the_child() {
    let mut service = fixtures::started();
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().expect("engine");
    let (config, runtime) = registered();
    let mut accounts = host(&config, runtime);
    pump_accounts(&mut accounts, engine);
    accounts.above.push(driver::Event::Taken { account: 71 });
    let mut events = pump_move_events(&mut accounts, engine);
    let Some(root::Event::Account(domain::Event::Finishing { source: 71 })) = events.pop() else {
        panic!("the driver's taken event reaches the accounts child");
    };
    assert!(events.is_empty());
    for (why, expected) in [
        (driver::Refusal::State, domain::SignInRedirectWhy::State),
        (driver::Refusal::Redirect, domain::SignInRedirectWhy::Address),
        (driver::Refusal::Full { bound: 3 }, domain::SignInRedirectWhy::TooMany),
    ] {
        accounts.above.push(driver::Event::Refused { account: 71, asked: driver::Asked::Redirected, why });
        let mut events = pump_move_events(&mut accounts, engine);
        let Some(root::Event::Account(domain::Event::FinishRefused { source, why })) = events.pop() else {
            panic!("driver refusal reaches the accounts child");
        };
        assert_eq!(source, 71);
        assert_eq!(why, expected);
        assert!(events.is_empty());
    }
}

#[test]
fn aborting_drops_a_finish_in_flight() {
    let mut service = fixtures::started();
    fixtures::restart(&mut service);
    let engine = service.engine.as_mut().expect("engine");
    let (config, runtime) = registered();
    let mut accounts = host(&config, runtime);
    pump_accounts(&mut accounts, engine);
    let carried = held_address(engine);
    accounts.request(domain::Request::Signing(domain::Signing::Redirected { source: 71, carried }));
    accounts.start(&mut engine.store, &mut engine.values);
    let Some(finishes::Next::Get { exchange, carried }) = accounts.finish_out.pop() else {
        panic!("finish waits for its store read");
    };
    let answer = engine.store.secrets(
        Token::new(FINISH_EXCHANGE | exchange.raw()),
        store::Secrets::Get { owner: store::SecretOwner::Application, number: carried },
    );
    assert!(accounts.finishes.pending());
    accounts.abort();
    engine.store.sources.push(answer.expect("late store answer"));
    let mut events = pump_move_events(&mut accounts, engine);
    let Some(root::Event::Account(domain::Event::Component(domain::FromComponent::Closed))) = events.pop() else {
        panic!("the aborted driver closes");
    };
    assert!(events.is_empty(), "the aborted finish has no answer from the driver or child");
    assert!(!accounts.finishes.pending());
    assert!(accounts.finish_out.is_empty());
    assert!(accounts.io.is_empty());
    assert!(accounts.is_empty());
}

#[test]
fn a_restored_sources_other_kind_fails_the_read() {
    const SOURCE: u32 = 7;
    for (kind, value) in [
        (domain::Kind::Key, store::SecretValue::Record(Secret::new(b"record".as_slice().into()))),
        (domain::Kind::SignIn, store::SecretValue::Key(Secret::new(b"key".as_slice().into()))),
    ] {
        let mut service = fixtures::ready();
        let engine = service.engine.as_mut().expect("engine");
        let mut accounts = engine.accounts.take().expect("accounts");
        engine.store.secrets(
            Token::new(123),
            store::Secrets::Keep { owner: store::SecretOwner::Source, number: u64::from(SOURCE), value },
        );
        accounts.dispatch(
            domain::Request::Signing(domain::Signing::Restore { source: SOURCE, provider: 1, kind }),
            &mut engine.store,
            &mut engine.values,
        );
        let mut failed = false;
        while let Some(event) = engine.store.above.pop() {
            if let store::Event::Corrupt { .. } = event {
                failed = true;
            }
        }
        assert!(failed, "the wrong kind follows the failed-read path");
        assert!(accounts.aborting);
    }
}
