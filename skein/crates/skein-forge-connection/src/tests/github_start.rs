//! GitHub mint plan and component completion step tests.
use super::fixtures;
use crate::Component;
use crate::seam::Access;
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Time, Wall};
const SMALL_TOKEN_BOUND: u32 = 8;
const FULL: &[u8] = b"artificial-full-api-secret";
const EXPIRES: &[u8] = b"1970-01-01T01:00:00Z";
pub(super) fn mint(token: &[u8], expires: &[u8]) -> Box<[u8]> {
    let mut body = skein_lib::Writer::new(
        token.len().checked_add(expires.len()).expect("fixture fields").checked_add(28).expect("JSON framing"),
    );
    body.put(b"{\"token\":\"").expect("measured token");
    body.put(token).expect("measured token");
    body.put(b"\",\"expires_at\":\"").expect("measured expiry");
    body.put(expires).expect("measured expiry");
    body.put(b"\"}").expect("measured mint");
    fixtures::response(201, &body.finish())
}

#[test]
fn github_mint_one_past_a_small_token_bound_is_too_large() {
    use skein_forge::github::{
        plans::{
            plan::Bounds,
            start::{Minting, Step},
        },
        request::Scope,
    };
    let mut h = fixtures::ConnectionHarness::github();
    h.env.limits.token_bytes = SMALL_TOKEN_BOUND;
    let bounds = Bounds { inner: 8, page_size: 100, rows: 1, answer_bytes: 1024, body_bytes: fixtures::BODY_ROOM };
    let mut plan = Minting::new(1, Scope::Full);
    let request = match plan.begin(bounds).expect("mint begins") {
        Step::Send(exchange) => exchange.request,
        Step::Done(_) => unreachable!("mint sends"),
    };
    h.start_github(
        skein_forge::github::plans::plan::Exchange {
            request,
            retention: None,
            comparison: skein_forge::github::plans::plan::Comparison::Full,
        },
        Some(skein_lib::Secret::new(Box::from(b"Bearer artificial-jwt".as_slice()))),
    )
    .expect("mint request");
    let token = fixtures::filled(usize::try_from(SMALL_TOKEN_BOUND + 1).expect("bound"), b'x');
    let heard = h.github_answer(&mint(&token, EXPIRES));
    let mut outcome = None;
    for event in heard.into_boxed() {
        if let Some(step) = plan.heard(event, bounds).expect("ordered response") {
            match step {
                Step::Done(result) => outcome = Some(result),
                Step::Send(_) => unreachable!("one mint exchange"),
            }
        }
    }
    match outcome {
        Some(Err(error)) => assert_eq!(error, skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed())),
        Some(Ok(_)) | None => unreachable!("oversized token is refused"),
    }
}

#[test]
fn github_full_mint_replaces_levels_and_contents_mint_preserves_them() {
    use skein_forge::github::document::{Level, Levels, Minted, Viewer};
    let limits = fixtures::limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
    let mut up = Queue::uncapped();
    let gh = component.test_github_mut();
    gh.accept(&[1]);
    gh.begin_read(limits.start_backoff);
    component.test_paged(Box::from([fixtures::installation()]), false, &env, &mut up);
    component.test_local_step(&env, &mut up);
    let expected = Levels { contents: Level::Write, pull_requests: Level::Write };
    let minted = |token: &[u8], permissions| {
        Ok(Minted {
            token: skein_lib::Secret::new(Box::from(token)),
            expires: Wall::from_nanos(3_600_000_000_000),
            permissions,
        })
    };
    component.test_minted(minted(b"artificial-full", expected), &env, &mut up);
    component.test_local_step(&env, &mut up);
    assert_eq!(component.test_github().rows.get(&1).expect("installation").permissions, expected);
    let narrowed = Levels { contents: Level::Write, pull_requests: Level::None };
    component.test_minted(minted(b"artificial-git", narrowed), &env, &mut up);
    component.test_local_step(&env, &mut up);
    assert_eq!(component.test_github().rows.get(&1).expect("installation").permissions, expected);
    component.test_identified(Viewer { id: 9, login: Box::from(b"test-app[bot]".as_slice()) }, &env, &mut up);
    component.test_local_step(&env, &mut up);
    assert!(component.test_github().identity_checked);
    component.test_github_mut().rows.get_mut(&1).expect("installation").full.pending = true;
    component.test_local_step(&env, &mut up);
    // An absent permissions object in the next full mint revokes both kept levels.
    component.test_minted(minted(FULL, Levels::NONE), &env, &mut up);
    assert_eq!(component.test_github().rows.get(&1).expect("installation").permissions, Levels::NONE);
}
