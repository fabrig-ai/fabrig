//! GitHub start transcripts; the only credentials here are artificial fixture values.
use super::fixtures::{self, FORGE, Harness};
use crate::{Event, Request};
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

pub(super) const LIVE: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":null}]"#;

pub(super) const SUSPENDED: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":"1970-01-01T00:00:02Z"}]"#;

pub(super) const FULL: &[u8] = b"artificial-full-api-secret";

pub(super) const CONTENTS: &[u8] = b"artificial-contents-git-secret";

pub(super) const EXPIRES: &[u8] = b"1970-01-01T01:00:00Z";

pub(super) const EXPIRY: Wall = Wall::from_nanos(3_600_000_000_000);

pub(super) const VIEWER: &[u8] = br#"{"data":{"viewer":{"databaseId":9,"login":"test-app[bot]"}}}"#;

pub(super) const NEXT_PAGE: &[u8] =
    b"Link: <https://api.github.test/app/installations?per_page=100&page=2>; rel=\"next\"\r\n";

const SMALL_TOKEN_BOUND: u32 = 8;

#[derive(Clone, Copy, Debug)]
#[expect(dead_code, reason = "the shared start fixture retains phases used by the moved scenarios")]
enum Stage {
    Listing,
    Full,
    Contents,
    Identity,
}

pub(super) fn accepted(h: &mut Harness, ids: &[u64]) {
    h.send(Request::Installations { forge: FORGE, accepted: Box::from(ids) });
}

fn harness() -> Harness {
    let mut h = Harness::new(fixtures::github_settings());
    accepted(&mut h, &[1]);
    h.connected(FORGE);
    h
}

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

fn listed(h: &mut Harness) {
    h.answer(FORGE, &fixtures::ok(LIVE));
}

pub(super) fn full(h: &mut Harness) {
    h.answer(FORGE, &mint(FULL, EXPIRES));
}

pub(super) fn contents(h: &mut Harness) {
    h.answer(FORGE, &mint(CONTENTS, EXPIRES));
}

fn at(stage: Stage) -> Harness {
    let mut h = harness();
    match stage {
        Stage::Listing => {}
        Stage::Full => listed(&mut h),
        Stage::Contents => {
            listed(&mut h);
            full(&mut h);
        }
        Stage::Identity => {
            listed(&mut h);
            full(&mut h);
            contents(&mut h);
        }
    }
    h
}

pub(super) fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    for piece in bytes.windows(needle.len()) {
        if piece == needle {
            return true;
        }
    }
    false
}

pub(super) fn tokens(h: &Harness) -> &[Event] {
    h.git_tokens.as_ref().expect("GitHub harness").as_slice()
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
        Some(Err(error)) => assert_eq!(error, skein_forge::api::Error::TooLarge),
        Some(Ok(_)) | None => unreachable!("oversized token is refused"),
    }
}

#[test]
fn github_full_mint_replaces_levels_and_contents_mint_preserves_them() {
    use skein_forge::github::document::{Level, Levels};
    let mut h = at(Stage::Full);
    let full = br#"{"token":"artificial-full","expires_at":"1970-01-01T01:00:00Z","permissions":{"contents":"write","pull_requests":"write"}}"#;
    h.answer(FORGE, &fixtures::response(201, full));
    let expected = Levels { contents: Level::Write, pull_requests: Level::Write };
    assert_eq!(h.component.test_github().rows.get(&1).expect("installation").permissions, expected);
    let narrowed =
        br#"{"token":"artificial-git","expires_at":"1970-01-01T01:00:00Z","permissions":{"contents":"write"}}"#;
    h.answer(FORGE, &fixtures::response(201, narrowed));
    assert_eq!(h.component.test_github().rows.get(&1).expect("installation").permissions, expected);
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    assert!(h.component.test_github().identity_checked);
    h.component.test_github_mut().rows.get_mut(&1).expect("installation").full.pending = true;
    h.run(Duration::ZERO);
    // An absent permissions object in the next full mint revokes both kept levels.
    h.answer(FORGE, &mint(FULL, EXPIRES));
    assert_eq!(h.component.test_github().rows.get(&1).expect("installation").permissions, Levels::NONE);
}
