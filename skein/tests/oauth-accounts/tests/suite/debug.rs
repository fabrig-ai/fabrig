//! Credential and carrier debug views (skein/docs/design/oauth.md, 4; skein/docs/design/lib.md, 6).

use std::fmt::Debug;

use skein_fake_oauth as fake;
use skein_lib::stream::Up;
use skein_lib::{CryptoRng, Duration, Env, List, Queue, Secret, Time, Token, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts::{self as accounts, Component, Event, Limits, Lower, LowerEvent};
use skein_oauth_accounts_world::{sign_in, world::limits};

const CANARY: &str = "canary-oauth-credential-that-must-stay-hidden";

fn secret() -> Secret {
    Secret::new(bytes::copy_of(CANARY.as_bytes()))
}

fn registration() -> oauth::Registration {
    oauth::Registration {
        client_secret: Some(secret()),
        pkce_for_confidential: true,
        ..sign_in::registration(sign_in::Story::OpenIdAgain)
    }
}

fn record() -> oauth::SavedToken {
    oauth::SavedToken {
        key: 0,
        generation: 1,
        access_token: secret(),
        refresh_token: Some(secret()),
        metadata: None,
        expires_at: Wall::from_nanos(1),
        issued_client_id: None,
        granted_scopes: None,
        subject: None,
        id_token: Some(secret()),
        earliest_refresh_at: None,
    }
}

fn post() -> oauth::HttpRequest {
    let form = format!("grant_type=refresh_token&refresh_token={CANARY}&client_id=client&client_secret={CANARY}");
    oauth::HttpRequest {
        id: 1,
        method: oauth::Method::Post,
        endpoint: bytes::copy_of(b"https://issuer.example/token"),
        content_type: b"application/x-www-form-urlencoded",
        body: form.into_bytes().into_boxed_slice(),
        deadline: Time::ZERO,
    }
}

fn sign_in() -> oauth::Event {
    oauth::Event::SignIn {
        registration: registration(),
        key: 0,
        generation: 0,
        state: secret(),
        verifier: Some(secret()),
        nonce: Some(secret()),
        port: None,
        issued_client_id: None,
        id_token: Some(secret()),
        subject: None,
        now: Time::ZERO,
    }
}

fn assert_hidden(name: &str, value: &dyn Debug) {
    let bytes = format!("{:?}", CANARY.as_bytes()).replace(' ', "");
    let sequence = bytes.trim_matches(['[', ']']);
    for printed in [format!("{value:?}"), format!("{value:#?}")] {
        assert!(!printed.contains(CANARY), "{name}: canary text in Debug: {printed}");
        assert!(!printed.replace([' ', '\n'], "").contains(sequence), "{name}: canary bytes in Debug: {printed}");
    }
}

fn advance(component: &mut Component, env: &Env<Limits>, events: &mut Queue<Event>, io: &mut Queue<Lower>) {
    while component.has_work() {
        component.fire(env, events, io, &mut Queue::uncapped());
    }
}

#[test]
fn debug_hides_oauth_secret_and_hand_written_views() {
    let mut client = oauth::Client::new(limits().client).expect("client");
    let mut out = Queue::uncapped();
    client.step(sign_in(), &mut out);
    assert!(matches!(out.pop(), Some(oauth::Request::Http(_))), "sign-in holds its visit while fetching keys");

    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let address = ([127, 0, 0, 1], 31000).into();
    let mut component = accounts::Component::new(
        List::from_boxed(Box::new([accounts::Account::SignIn {
            registration: registration(),
            endpoint: accounts::Endpoint { address, transport: accounts::Transport::Plaintext },
            keeper: accounts::Keeper::Owner { kept: Some(record()) },
            shared: false,
            origins: List::with_capacity(0),
        }])),
        &env.limits,
        CryptoRng::new(&Secret::new(bytes::copy_of(&[7; 32])), 0).expect("seed"),
    )
    .expect("component");
    let mut events = Queue::uncapped();
    let mut io = Queue::uncapped();
    component.down(&env, accounts::Request::Grant { account: 0 }, &mut events, &mut io, &mut Queue::uncapped());
    advance(&mut component, &env, &mut events, &mut io);
    let Some(accounts::Lower::Connect { owner, .. }) = io.pop() else { panic!("refresh opens its socket") };
    let partial = format!("HTTP/1.1 200 OK\r\ncontent-length: 1024\r\n\r\n{{\"access_token\":\"{CANARY}");
    let response = partial.split_inclusive('\n');
    let pieces = response.map(|piece| LowerEvent::Stream { owner, up: Up::Bytes(bytes::copy_of(piece.as_bytes())) });
    for event in [
        LowerEvent::Connecting { owner, socket: Token::new(100) },
        LowerEvent::Connected { owner },
        LowerEvent::Stream { owner, up: Up::Room },
        LowerEvent::Stream { owner, up: Up::Room },
    ]
    .into_iter()
    .chain(pieces)
    {
        component.up(&env, event, &mut events, &mut io, &mut Queue::uncapped());
        advance(&mut component, &env, &mut events, &mut io);
    }
    assert!(events.is_empty(), "partial answer neither grants nor fails the exchange: {events:?}");

    let mut issuer = fake::Issuer::new(
        fake::Config {
            authorization_url: bytes::copy_of(b"https://issuer.example/authorize"),
            token_endpoint: bytes::copy_of(b"https://issuer.example/token"),
            client_id: bytes::copy_of(b"client"),
            client_secret: Some(secret()),
            redirect_uri: bytes::copy_of(b"https://app.example/callback"),
            refresh_token: secret(),
            terms: None,
        },
        fake::Limits {
            document: env.limits.client.document,
            uri_bytes: env.limits.client.uri_bytes,
            request_bytes: env.limits.client.request_bytes,
            codes: 1,
            rotations: 2,
            plans: 2,
            carried: 0,
        },
    )
    .expect("issuer");

    let body = fake::Body::Raw(secret());
    let delay = Duration::from_secs(1);
    issuer.queue(fake::Plan { status: 200, body, delay, retry_after: Duration::ZERO }).expect("delayed answer");
    let mut replies = Queue::uncapped();
    issuer.step(fake::Event::Post { request: post(), now: Time::ZERO, wall: Wall::EPOCH }, &mut replies);
    assert!(replies.is_empty(), "issuer keeps its pending answer");
    assert_eq!(issuer.posts(), 1);

    let rows: &[(&str, &dyn Debug)] = &[
        ("oauth::SavedToken", &record()),
        ("oauth::HttpRequest", &post()),
        (
            "oauth::HttpResponse",
            &oauth::HttpResponse {
                id: 1,
                status: 200,
                body: bytes::copy_of(CANARY.as_bytes()),
                retry_after: Duration::ZERO,
                evidence: oauth::HttpEvidence::Response,
                now: Time::ZERO,
                wall: Wall::from_nanos(0),
            },
        ),
        ("oauth::Client", &client),
        ("accounts::Component", &component),
        ("fake::Issuer", &issuer),
    ];
    for (name, value) in rows {
        assert_hidden(name, value);
    }
}
