//! Renewal fixtures for component and repository-probe step tests.
use super::{
    fixtures::{self, FORGE, Harness},
    github_start::{CONTENTS, EXPIRES, FULL, LIVE, VIEWER, accepted, mint},
};
use skein_lib::{Duration, Time};
// One-hour Full and seventy-minute Contents, with the fixture's sixty-second lead.
pub(crate) const FULL_DUE: u64 = 3540;
const CONTENTS_DUE: u64 = 4140;
const CONTENTS_EXPIRY: &[u8] = b"1970-01-01T01:10:00Z";
const LATER: &[u8] = b"1970-01-01T03:00:00Z";
const NEW_FULL: &[u8] = b"artificial-renewed-full";
const NEW_CONTENTS: &[u8] = b"artificial-renewed-contents";
pub(crate) fn ready() -> Harness {
    initial(CONTENTS_EXPIRY)
}

fn initial(contents_expiry: &[u8]) -> Harness {
    let mut h = minted_pair(EXPIRES, contents_expiry);
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    // Let the idle socket settle before crossing the renewal timer.
    h.run(h.env.limits.idle);
    h
}

fn minted_pair(full_expiry: &[u8], contents_expiry: &[u8]) -> Harness {
    let mut h = Harness::new(fixtures::github_settings());
    accepted(&mut h, &[1]);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(LIVE));
    h.answer(FORGE, &mint(FULL, full_expiry));
    h.answer(FORGE, &mint(CONTENTS, contents_expiry));
    h
}

pub(crate) fn advance(h: &mut Harness, seconds: u64) {
    h.run(Time::from_nanos(Duration::from_secs(seconds).as_nanos()).saturating_since(h.env.now));
}

pub(crate) fn renewing() -> Harness {
    let mut h = ready();
    advance(&mut h, FULL_DUE);
    h.connected(FORGE);
    h
}

pub(crate) fn contents_renewing() -> Harness {
    let mut h = renewing();
    renewed_full(&mut h);
    h.run(h.env.limits.idle);
    advance(&mut h, CONTENTS_DUE);
    h.connected(FORGE);
    h
}

pub(crate) fn renewed_full(h: &mut Harness) {
    h.answer(FORGE, &mint(NEW_FULL, LATER));
    h.run(Duration::ZERO);
}

pub(crate) fn renewed_contents(h: &mut Harness) {
    h.answer(FORGE, &mint(NEW_CONTENTS, LATER));
    h.run(Duration::ZERO);
}
