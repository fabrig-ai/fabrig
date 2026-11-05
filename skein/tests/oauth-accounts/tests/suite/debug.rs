//! Credential and carrier debug views (skein/docs/design/oauth.md, 4; skein/docs/design/lib.md, 6).

use std::fmt::Debug;

use skein_fake_oauth as fake;
use skein_lib::{Duration, Queue, Secret, Time, Wall, bytes};
use skein_oauth as oauth;
use skein_oauth_accounts as accounts;
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

fn token() -> oauth::TokenResponse {
    oauth::TokenResponse {
        access_token: secret(),
        refresh_token: Some(secret()),
        expires_in: 30,
        scope: None,
        id_token: Some(secret()),
        earliest_refresh_at: None,
    }
}

fn post() -> oauth::HttpRequest {
    oauth::HttpRequest {
        id: 1,
        method: oauth::Method::Post,
        endpoint: bytes::copy_of(b"https://issuer.example/token"),
        content_type: b"application/x-www-form-urlencoded",
        body: bytes::copy_of(CANARY.as_bytes()),
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

#[test]
fn debug_hides_every_oauth_credential_and_carrier() {
    let mut client = oauth::Client::new(limits().client).expect("client");
    let mut out = Queue::uncapped();
    client.step(sign_in(), &mut out);
    assert!(matches!(out.pop(), Some(oauth::Request::Http(_))), "sign-in holds its visit while fetching keys");

    let rows: &[(&str, &dyn Debug)] = &[
        ("oauth::SavedToken", &record()),
        ("oauth::RefreshState", &record().refresh_state().expect("refresh state")),
        ("oauth::TokenResponse", &token()),
        (
            "oauth::RefreshRequest",
            &oauth::RefreshRequest { client_id: bytes::copy_of(b"client"), refresh_token: secret() },
        ),
        ("oauth::Registration", &registration()),
        (
            "oauth::Redirect",
            &oauth::Redirect {
                uri: bytes::copy_of(b"https://app.example/callback"),
                state: secret(),
                code: Some(secret()),
                error: None,
                client_id: None,
            },
        ),
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
        ("oauth::Event::SignIn", &sign_in()),
        (
            "oauth::Event::Redirected",
            &oauth::Event::Redirected {
                uri: bytes::copy_of(b"https://app.example/callback"),
                state: secret(),
                code: Some(secret()),
                error: None,
                client_id: None,
                now: Time::ZERO,
            },
        ),
        ("oauth::Request::Visit", &oauth::Request::Visit { url: bytes::copy_of(CANARY.as_bytes()) }),
        ("accounts::Request::HandIn", &accounts::Request::HandIn { account: 0, record: record() }),
        (
            "accounts::Request::Redirected",
            &accounts::Request::Redirected { account: 0, uri: bytes::copy_of(CANARY.as_bytes()) },
        ),
        (
            "accounts::Event::Granted",
            &accounts::Event::Granted { account: 0, token: secret(), generation: 1, valid: Duration::ZERO },
        ),
        ("accounts::Event::Keep", &accounts::Event::Keep { account: 0, record: record() }),
        ("accounts::Event::Visit", &accounts::Event::Visit { account: 0, url: bytes::copy_of(CANARY.as_bytes()) }),
        ("accounts::Locked::Held", &accounts::Locked::Held { record: Some(record()) }),
        (
            "fake::Config",
            &fake::Config {
                authorization_url: bytes::copy_of(b"https://issuer.example/authorize"),
                token_endpoint: bytes::copy_of(b"https://issuer.example/token"),
                client_id: bytes::copy_of(b"client"),
                client_secret: Some(secret()),
                redirect_uri: bytes::copy_of(b"https://app.example/callback"),
                refresh_token: secret(),
                terms: None,
            },
        ),
        ("fake::Body::Token", &fake::Body::Token(token())),
        ("fake::Body::Raw", &fake::Body::Raw(bytes::copy_of(CANARY.as_bytes()))),
        (
            "fake::Plan",
            &fake::Plan {
                status: 200,
                body: fake::Body::Token(token()),
                delay: Duration::ZERO,
                retry_after: Duration::ZERO,
            },
        ),
        ("fake::Event::Authorize", &fake::Event::Authorize { url: bytes::copy_of(CANARY.as_bytes()), now: Time::ZERO }),
        ("fake::Event::Post", &fake::Event::Post { request: post(), now: Time::ZERO, wall: Wall::from_nanos(0) }),
        (
            "fake::Request::Redirect",
            &fake::Request::Redirect {
                uri: bytes::copy_of(b"https://app.example/callback"),
                state: secret(),
                code: secret(),
                client_id: None,
            },
        ),
        ("oauth::Client", &client),
    ];
    for (name, value) in rows {
        assert_hidden(name, value);
    }
}
