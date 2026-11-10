//! One protocol driver: a client and an issuer over scripted streams.
//! The stream ordering follows skein/docs/design/oauth/accounts.md, 8.
use crate::sign_in::protocol::Callbacks;
use crate::world::{Client, TlsClose};
use skein_fake_oauth as fake;
use skein_fake_peers::Transport;
use skein_http::server as http;
use skein_io as io;
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as accounts;
use skein_world::stream::{Trace, Wire};
use std::net::Ipv4Addr;

const ROOM: u32 = 64;

pub(crate) struct Rig {
    pub(crate) issuer: fake::Issuer,
    pub(crate) endpoint: Box<[u8]>,
    pub(crate) target: &'static [u8],
    pub(crate) profile: accounts::Limits,
    pub(crate) server_limits: http::Limits,
    /// Reseed and rebuild the server's wire on each connect, or keep one for life.
    pub(crate) rewires: bool,
    pub(crate) callbacks: Option<Callbacks>,
    pub(crate) room: bool,
    pub(crate) expected_posts: Option<u64>,
    pub(crate) turns: u32,
}

pub(crate) fn server_limits(issuer: &fake::Limits) -> http::Limits {
    http::Limits {
        head: 2048,
        headers: 16,
        body: 1024,
        read: 256,
        response: skein_fake_peers::oauth::least_response(issuer).expect("world issuer response floor fits u32"),
        send: 256,
    }
}

#[expect(clippy::too_many_lines, reason = "one protocol loop exposes both stream boundaries and the issuer")]
pub(crate) fn drive(seed: u64, mut client: Client, mut rig: Rig) -> (Client, Trace, TlsClose) {
    let profile = rig.profile;
    let server_limits = rig.server_limits;
    let mut issuer = rig.issuer;
    let mut server = http::Server::new(&server_limits);
    let mut server_events = Queue::with_capacity(ROOM);
    let mut server_requests = Queue::with_capacity(ROOM);
    let mut server_down = Queue::with_capacity(ROOM);
    let mut issuer_out = Queue::with_capacity(ROOM);
    let mut client_wire = Wire::new(seed);
    let mut server_wire = Wire::new(seed.wrapping_add(1));
    let mut tls = (client.transport == Transport::Tls).then(|| {
        skein_tls_kit::drive::Wire::new(skein_tls_kit::server::Server::new(
            skein_tls_kit::pki::Server::plain().config(),
        ))
    });
    let mut close = TlsClose::default();
    let epoch = if tls.is_some() { skein_tls_kit::pki::VALID } else { Wall::EPOCH };
    client.address = Some((Ipv4Addr::LOCALHOST, 31000).into());
    let mut now = Time::ZERO;
    client.start(&Env { now, wall: epoch, limits: profile });
    let mut owner = None;
    let mut body = Vec::new();
    let mut response: Option<(Box<[u8]>, usize)> = None;
    let mut server_closed = false;
    let mut document = None;
    let mut document_paths = Vec::new();
    let mut trace = Trace::new();
    for _ in 0..rig.turns {
        let wall = Wall::from_nanos(epoch.as_nanos().saturating_add(now.as_nanos()));
        let env = Env { now, wall, limits: profile };
        let server_env = Env { now, wall, limits: server_limits };
        if rig.callbacks.is_some() {
            crate::sign_in::restart(&mut client, &env);
        }
        let component = client.component.as_mut().expect("component");
        if component.has_work() || component.next_deadline().is_some_and(|due| due <= now) {
            component.fire(&env, &mut client.above, &mut client.requests, &mut client.files);
        }
        client.observe(&env);
        if let Some(callbacks) = &mut rig.callbacks {
            callbacks.script(&mut client, &env, &mut issuer, &mut issuer_out);
        } else {
            client.protocol_turn(&env, tls.is_some(), &mut close);
        }
        if let Some(request) = client.requests.pop() {
            if rig.callbacks.is_none() {
                client.protocol_socket(&env, &request);
            }
            match request {
                io::Request::Connect { owner: identity, .. } => {
                    assert!(owner.replace(identity).is_none(), "one exchange socket");
                    client.deliver(&env, io::Event::Connecting { owner: identity, socket: Token::new(9) });
                    client.deliver(&env, io::Event::Connected { owner: identity });
                    if rig.rewires {
                        server = http::Server::new(&server_limits);
                        server_closed = false;
                        server_events = Queue::with_capacity(ROOM);
                        server_requests = Queue::with_capacity(ROOM);
                        server_down = Queue::with_capacity(ROOM);
                        client_wire = Wire::new(seed.wrapping_add(4 + issuer.posts()));
                        server_wire = Wire::new(seed.wrapping_add(5 + issuer.posts()));
                        body.clear();
                        response = None;
                    }
                    server_requests.push(http::Request::Next);
                }
                io::Request::Listen { owner: identity, addr } => {
                    if let Some(callbacks) = &mut rig.callbacks {
                        callbacks.listen(&mut client, &env, identity, addr);
                    } else {
                        assert!(owner.replace(identity).is_none(), "one listener");
                        client.deliver(&env, io::Event::Listening { owner: identity, listener: Token::new(9), addr });
                    }
                }
                io::Request::Bind { owner: identity, socket } => {
                    rig.callbacks.as_mut().expect("callback listeners").bind(&mut client, identity, socket);
                }
                io::Request::Stream { stream, down } if rig.callbacks.is_some() && stream == Token::new(11) => {
                    rig.callbacks.as_mut().expect("callback stream").wire.take(down);
                }
                io::Request::Stream { down, .. } if tls.is_some() => {
                    let tls = tls.as_mut().expect("TLS stream");
                    tls.take(down);
                    server_wire.write(&std::mem::take(&mut tls.server.received));
                }
                io::Request::Stream { down, .. } => {
                    match &down {
                        Down::Send(bytes) => server_wire.write(bytes),
                        Down::Finish => server_wire.eof = true,
                        Down::Demand { .. } => {}
                    }
                    client_wire.take(down);
                }
                request @ (io::Request::Close { entity } | io::Request::Abort { entity }) => {
                    if matches!(request, io::Request::Abort { .. }) {
                        close.aborted = Some(now);
                    }
                    let identity = if rig.rewires && entity != Token::new(9) {
                        rig.callbacks.as_mut().expect("callback listeners").close(&mut client, entity, &mut trace)
                    } else {
                        if rig.rewires {
                            trace.append(&mut client_wire.trace);
                            trace.append(&mut server_wire.trace);
                            server_requests.push(http::Request::Close);
                        }
                        owner.take().expect("socket owner")
                    };
                    client.deliver(&env, io::Event::Closed { owner: identity });
                    if !rig.rewires {
                        server_requests.push(http::Request::Close);
                    }
                }
                other @ (io::Request::Reject { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::SpawnAlone { .. }
                | io::Request::Signal { .. }
                | io::Request::Window { .. }
                | io::Request::Usage { .. }) => panic!("token exchange emits only socket requests: {other:?}"),
            }
        }
        if let Some(callbacks) = &mut rig.callbacks {
            callbacks.answer(&mut client, &env);
        }
        let answer = if owner.is_some() {
            match &mut tls {
                Some(tls) if !rig.room && close.started.is_some() && tls.demand.is_some_and(|(_, room)| room > 0) => {
                    None
                }
                Some(tls) => tls.answer(),
                None => client_wire.answer(),
            }
        } else {
            None
        };
        if let Some(identity) = owner
            && let Some(up) = answer
        {
            client.deliver(&env, io::Event::Stream { owner: identity, up });
        }
        if let Some(request) = server_requests.pop() {
            http::down(&mut server, &server_env, request, &mut server_events, &mut server_down);
        }
        if !server_closed && let Some(up) = server_wire.answer() {
            http::up(&mut server, &server_env, up, &mut server_events, &mut server_down);
        }
        if let Some(down) = server_down.pop() {
            match &down {
                Down::Send(bytes) => match &mut tls {
                    Some(tls) => {
                        tls.server.write(bytes);
                        tls.pull();
                    }
                    None => client_wire.write(bytes),
                },
                Down::Finish => client_wire.eof = true,
                Down::Demand { .. } => {}
            }
            server_wire.take(down);
        }
        if let Some(event) = server_events.pop() {
            match event {
                http::Event::Call(call) => {
                    if rig.callbacks.is_some() && call.method == skein_http::Method::Get {
                        document_paths.push(call.target.clone());
                        let mut uri = crate::sign_in::ISSUER.to_vec();
                        uri.extend_from_slice(&call.target);
                        document = Some(uri.into_boxed_slice());
                    } else {
                        assert_eq!(call.method, skein_http::Method::Post);
                        assert_eq!(call.target.as_ref(), rig.target);
                        assert_eq!(call.header(b"content-type"), Some(b"application/x-www-form-urlencoded".as_slice()));
                    }
                    server_requests.push(read_body(body.len(), profile.client.document.document_bytes));
                }
                http::Event::Body(Up::Bytes(bytes)) => {
                    body.extend_from_slice(&bytes);
                    server_requests.push(read_body(body.len(), profile.client.document.document_bytes));
                }
                http::Event::Body(Up::End) if document.is_some() => issuer.step(
                    fake::Event::Get { id: 0, endpoint: document.take().expect("document URL"), now, wall },
                    &mut issuer_out,
                ),
                http::Event::Body(Up::End) => issuer.step(
                    fake::Event::Post {
                        request: oauth::HttpRequest {
                            method: oauth::Method::Post,
                            id: 0,
                            endpoint: rig.endpoint.clone(),
                            content_type: b"application/x-www-form-urlencoded",
                            body: bytes::copy_of(&body),
                            deadline: now.saturating_add(Duration::from_secs(10)),
                        },
                        now,
                        wall,
                    },
                    &mut issuer_out,
                ),
                http::Event::Reply(Up::Room) => {
                    let (bytes, offset) = response.as_mut().expect("response body");
                    let end = offset.saturating_add(256).min(bytes.len());
                    server_requests.push(http::Request::Reply(Down::Send(bytes::copy_of(&bytes[*offset..end]))));
                    *offset = end;
                    if end == bytes.len() {
                        server_requests.push(http::Request::Reply(Down::Finish));
                    } else {
                        server_requests.push(http::Request::Reply(Down::Demand {
                            read: Read::Nothing,
                            room: u32::try_from(bytes.len() - end).expect("bounded body").min(256),
                        }));
                    }
                }
                http::Event::Closed => server_closed = true,
                http::Event::Done(_) => {}
                other @ (http::Event::Ended
                | http::Event::Body(_)
                | http::Event::Reply(_)
                | http::Event::Refused(_)
                | http::Event::Failed(_)) => panic!("positive independent issuer HTTP event: {other:?}"),
            }
        }
        issuer.step(fake::Event::Tick { now, wall }, &mut issuer_out);
        if let Some(output) = issuer_out.pop() {
            match output {
                fake::Request::Http(answer) => {
                    server_requests.push(http::Request::Respond(http::Response {
                        status: answer.status,
                        headers: Box::new([]),
                        body: http::Body::Length(skein_lib::len_u64(answer.body.len())),
                        close: true,
                    }));
                    server_requests.push(http::Request::Reply(Down::Demand {
                        read: Read::Nothing,
                        room: u32::try_from(answer.body.len()).expect("bounded response").min(256),
                    }));
                    response = Some((answer.body, 0));
                }
                fake::Request::Redirect { .. } | fake::Request::Refused => panic!("valid rotating refresh"),
            }
        }
        if client.closed
            && client.requests.is_empty()
            && client.above.is_empty()
            && owner.is_none()
            && rig.callbacks.as_ref().is_none_or(Callbacks::settled)
        {
            if let Some(expected) = rig.expected_posts {
                assert_eq!(issuer.posts(), expected);
            } else {
                assert!(issuer.posts() <= 1);
            }
            if let Some(callbacks) = &mut rig.callbacks {
                callbacks.check(&document_paths);
            }
            trace.append(&mut client_wire.trace);
            trace.append(&mut server_wire.trace);
            if let Some(callbacks) = &mut rig.callbacks {
                trace.append(&mut callbacks.wire.trace);
            }
            close.peer_closed = tls.is_some_and(|tls| tls.server.closed);
            return (client, trace, close);
        }
        now = now.saturating_add(Duration::from_millis(1));
    }
    panic!("protocol world settles");
}

pub(crate) fn issuer_config(registration: &oauth::Registration, terms: Option<fake::Terms>) -> fake::Config {
    fake::Config {
        authorization_url: registration.authorization_url.clone(),
        token_endpoint: registration.token_endpoint.clone(),
        client_id: registration.client_id.clone(),
        client_secret: registration.client_secret.clone(),
        redirect_uri: registration.redirect_uri.clone(),
        refresh_token: skein_lib::Secret::new(bytes::copy_of(b"seed")),
        terms,
    }
}

pub(crate) fn token_plan(
    status: u16,
    access: Box<[u8]>,
    refresh: Option<Box<[u8]>>,
    scope: Option<Box<[u8]>>,
) -> fake::Plan {
    fake::Plan {
        status,
        body: fake::Body::Token(oauth::TokenResponse {
            access_token: skein_lib::Secret::new(access),
            refresh_token: refresh.map(skein_lib::Secret::new),
            expires_in: 30,
            scope,
            id_token: None,
            earliest_refresh_at: None,
        }),
        delay: Duration::from_millis(5),
        retry_after: Duration::ZERO,
    }
}

fn read_body(bytes: usize, maximum: u32) -> http::Request {
    http::Request::Body(Down::Demand {
        read: Read::Arrived { max: maximum.saturating_sub(u32::try_from(bytes).expect("bounded request body")).max(1) },
        room: 0,
    })
}
