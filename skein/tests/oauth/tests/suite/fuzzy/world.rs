//! Refresh scenarios vary by case seed (`skein/docs/design/testing/sweeps.md`, 4.2–4.5).

use core::convert::Infallible;
use skein_fake_oauth as fake;
use skein_lib::Duration;
use skein_oauth as oauth;
use skein_oauth_world::{boxed as box_of, client_step, documents, fake_step, time as at, wall};
use skein_world::{sweep::cases, this_test};

fn registration(seed: u64) -> oauth::Registration {
    oauth::Registration {
        authorization_url: box_of(b"https://issuer.example/authorize"),
        token_endpoint: box_of(b"https://issuer.example/token"),
        client_id: box_of(b"client"),
        redirect_uri: box_of(b"http://127.0.0.1:1234/callback"),
        scope: box_of(b""),
        wire: if seed.is_multiple_of(2) { oauth::WireFormat::Json } else { oauth::WireFormat::Form },
        client_secret: None,
        pkce_for_confidential: false,
        metadata_claim: None,
        terms: None,
    }
}

fn delay_and_mode(seed: u64) -> (u64, u64) {
    (seed % 5, seed % 3)
}

fn scenario(seed: u64) {
    let document = documents();
    let mut client = oauth::Client::new(oauth::ClientLimits {
        document,
        scope_bytes: 32,
        state_bytes: 32,
        code_bytes: 32,
        url_bytes: 512,
        request_bytes: 512,
        skew: Duration::from_secs(60),
        sign_in_time: Duration::from_secs(60),
        backoff_base: Duration::from_secs(1),
        backoff_ceiling: Duration::from_secs(4),
        max_attempts: 2,
        ..oauth::ClientLimits::TEST
    })
    .expect("client");
    let mut issuer = fake::Issuer::new(
        fake::Config {
            authorization_url: box_of(b"https://issuer.example/authorize"),
            token_endpoint: box_of(b"https://issuer.example/token"),
            client_id: box_of(b"client"),
            client_secret: None,
            redirect_uri: box_of(b"http://127.0.0.1:1234/callback"),
            refresh_token: skein_lib::Secret::new(box_of(b"seed")),
            terms: None,
        },
        fake::Limits { document, uri_bytes: 256, request_bytes: 512, codes: 2, rotations: 2, plans: 2, carried: 0 },
    )
    .expect("issuer");
    let (delay, mode) = delay_and_mode(seed);
    let plan = plan(mode, delay);
    issuer.queue(plan).expect("plan");
    let Some(oauth::Request::Http(request)) = client_step(
        &mut client,
        oauth::Event::Refresh {
            registration: registration(seed),
            prior: oauth::RefreshState {
                key: 1,
                generation: 3,
                refresh_token: skein_lib::Secret::new(box_of(b"seed")),
                issued_client_id: None,
                granted_scopes: None,
                subject: None,
                id_token: None,
            },
            now: at(1),
        },
    ) else {
        panic!("request at seed {seed}")
    };
    let first = fake_step(&mut issuer, fake::Event::Post { request, now: at(2), wall: wall(100) });
    let answer = if delay == 0 {
        first
    } else {
        assert!(first.is_none(), "issuer waits at seed {seed}");
        fake_step(&mut issuer, fake::Event::Tick { now: at(2 + delay), wall: wall(100 + delay) })
    };
    let Some(fake::Request::Http(answer)) = answer else { panic!("answer at seed {seed}") };
    let terminal = client_step(&mut client, oauth::Event::Http(answer));
    match (mode, terminal) {
        (0, Some(oauth::Request::Tokens { record, .. })) => {
            assert_eq!(
                record.refresh_token.as_ref().map(skein_lib::Secret::expose).expect("refresh token"),
                b"next",
                "seed {seed}"
            );
            assert_eq!(issuer.generation(), 1, "seed {seed}");
        }
        (1, Some(oauth::Request::Tokens { record, .. })) => {
            assert_eq!(
                record.refresh_token.as_ref().map(skein_lib::Secret::expose).expect("refresh token"),
                b"seed",
                "seed {seed}"
            );
            assert_eq!(issuer.generation(), 0, "seed {seed}");
        }
        (2, Some(oauth::Request::Failed { failure: oauth::Failure::Refused })) => {}
        _ => panic!("unexpected terminal at seed {seed}"),
    }
    assert!(client.is_done(), "one terminal at seed {seed}");
}

#[test]
fn varied_delay_rotation_and_error_body_keep_one_terminal_per_refresh() {
    cases(this_test!("varied_delay_rotation_and_error_body_keep_one_terminal_per_refresh"), 0..192, |seed| {
        scenario(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn refresh_draws_reduce_full_width_seeds_before_time_arithmetic() {
    assert_eq!(delay_and_mode(u64::MAX), (0, 0));
    assert_eq!(delay_and_mode(u64::MAX - 1), (4, 2));
}

fn plan(mode: u64, delay: u64) -> fake::Plan {
    match mode {
        0 => fake::Plan {
            status: 200,
            body: fake::Body::Token(oauth::TokenResponse {
                access_token: skein_lib::Secret::new(box_of(b"access")),
                refresh_token: Some(skein_lib::Secret::new(box_of(b"next"))),
                expires_in: 30,
                scope: None,
                id_token: None,
                earliest_refresh_at: None,
            }),
            delay: Duration::from_secs(delay),
            retry_after: Duration::ZERO,
        },
        1 => fake::Plan {
            status: 200,
            body: fake::Body::Token(oauth::TokenResponse {
                access_token: skein_lib::Secret::new(box_of(b"access")),
                refresh_token: None,
                expires_in: 30,
                scope: None,
                id_token: None,
                earliest_refresh_at: None,
            }),
            delay: Duration::from_secs(delay),
            retry_after: Duration::ZERO,
        },
        _ => fake::Plan {
            status: 400,
            body: fake::Body::Error(oauth::OAuthError { code: box_of(b"invalid_grant"), detail: box_of(b"revoked") }),
            delay: Duration::from_secs(delay),
            retry_after: Duration::ZERO,
        },
    }
}
