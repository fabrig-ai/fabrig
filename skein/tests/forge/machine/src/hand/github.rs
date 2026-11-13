use super::{FORGE, Harness, app_access, github_settings, ok, response, shared};
use skein_forge::{api, github::document};
use skein_forge_connection::seam::Access;
use skein_forge_connection::{
    Event,
    seam::{Clock, Read, Row, Tokens},
};
use skein_lib::{Duration, Secret, Time, Wall};
use std::boxed::Box;

pub const GITHUB_REPOSITORY: &[u8] = br#"{"id":1,"owner":{"id":22,"login":"acme","type":"Organization"},"name":"widgets","default_branch":"main","html_url":"https://github.com/acme/widgets","allow_merge_commit":true,"allow_squash_merge":true,"allow_rebase_merge":false}"#;
pub const GITHUB_BRANCH: &[u8] = br#"{"name":"main","commit":{"sha":"0000000000000000000000000000000000000000"}}"#;

// GitHub dialect fixtures: issue/comment, pull, combined status, check runs and GraphQL.
pub const GITHUB_ISSUE: &[u8] = br#"{"number":7,"state":"open","user":{"id":9},"updated_at":"2026-10-20T08:00:00Z","title":"widgets","labels":[],"body":"description"}"#;
pub const GITHUB_PULL: &[u8] = br#"{"number":7,"state":"open","merged":false,"merge_commit_sha":null,"mergeable":true,"head":{"ref":"feature","sha":"0000000000000000000000000000000000000000"},"base":{"ref":"main","sha":"1111111111111111111111111111111111111111"},"requested_reviewers":[]}"#;
pub const GITHUB_COMBINED: &[u8] = br#"{"state":"pending","total_count":0,"statuses":[]}"#;
pub const GITHUB_RUNS: &[u8] = br#"{"check_runs":[]}"#;
pub const GITHUB_GRAPHQL: &[u8] = br#"{"data":{"repository":{"pullRequest":{"id":"opaque-node","mergeCommit":null,"mergeQueueEntry":null,"timelineItems":{"nodes":[]}}}}}"#;

pub const LIVE: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":null}]"#;

pub const SUSPENDED: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":"1970-01-01T00:00:02Z"}]"#;

pub const FULL: &[u8] = b"artificial-full-api-secret";

pub const CONTENTS: &[u8] = b"artificial-contents-git-secret";

pub const EXPIRES: &[u8] = b"1970-01-01T01:00:00Z";

pub const EXPIRY: Wall = Wall::from_nanos(3_600_000_000_000);

pub const VIEWER: &[u8] = br#"{"data":{"viewer":{"databaseId":9,"login":"test-app[bot]"}}}"#;

pub const NEXT_PAGE: &[u8] = b"Link: <https://api.github.test/app/installations?per_page=100&page=2>; rel=\"next\"\r\n";

pub fn accepted(h: &mut Harness, ids: &[u64]) {
    h.send(skein_forge_connection::Request::Installations { forge: FORGE, accepted: Box::from(ids) });
}

#[must_use]
pub fn mint(token: &[u8], expires: &[u8]) -> Box<[u8]> {
    let mut body = skein_lib::Writer::new(
        token.len().checked_add(expires.len()).expect("fixture fields").checked_add(28).expect("JSON framing"),
    );
    body.put(b"{\"token\":\"").expect("measured token");
    body.put(token).expect("measured token");
    body.put(b"\",\"expires_at\":\"").expect("measured expiry");
    body.put(expires).expect("measured expiry");
    body.put(b"\"}").expect("measured mint");
    response(201, &body.finish())
}

pub fn full(h: &mut Harness) {
    h.answer(FORGE, &mint(FULL, EXPIRES));
}

pub fn contents(h: &mut Harness) {
    h.answer(FORGE, &mint(CONTENTS, EXPIRES));
}

#[must_use]
pub fn contains(bytes: &[u8], needle: &[u8]) -> bool {
    for piece in bytes.windows(needle.len()) {
        if piece == needle {
            return true;
        }
    }
    false
}

#[must_use]
pub fn tokens(h: &Harness) -> &[Event] {
    h.git_tokens.as_ref().expect("GitHub harness").as_slice()
}

pub fn github_repository(h: &mut Harness, repository: api::Repository, account: u64) {
    h.component.github_tables().keep_repository(
        repository,
        skein_forge::github::plans::tables::Repository {
            owner: Box::from(b"acme".as_slice()),
            name: Box::from(b"widgets".as_slice()),
            owner_id: account,
            organization: true,
            settings: api::Settings { default_branch: Box::from(*b"main"), merge: true, squash: true, rebase: true },
            merge: Some(skein_forge::github::plans::tables::Method::Merge),
            html_url: Box::from(b"https://github.com/acme/widgets".as_slice()),
        },
    );
}

const SECOND: Duration = Duration::from_secs(1);
#[must_use]
pub fn row(id: u64, suspended: bool) -> document::Installation {
    document::Installation {
        id,
        account: 22,
        login: Box::from(b"acme".as_slice()),
        kind: document::Account::Organization,
        created: Wall::EPOCH,
        suspended,
    }
}

// Feed the committed reconciliation transition directly to cross a written job
// with loss; socket serialization itself is checked by the wire stories.
pub fn commit(h: &mut Harness, rows: Box<[document::Installation]>) {
    let gh = h.component.test_github_mut();
    let mut reading = skein_forge_connection::seam::Reading::new(gh.rows.capacity(), gh.generation, SECOND);
    reading.page(rows, &gh.rows, &gh.accepted, self::shared().name_bytes).expect("fixture read");
    gh.read = Read::Reading(reading);
    gh.commit();
}

// One-hour Full and seventy-minute Contents, with the fixture's sixty-second lead.
pub const FULL_DUE: u64 = 3540;

pub const CONTENTS_DUE: u64 = 4140;

pub const CONTENTS_EXPIRY: &[u8] = b"1970-01-01T01:10:00Z";

pub const LATER: &[u8] = b"1970-01-01T03:00:00Z";

pub const NEW_FULL: &[u8] = b"artificial-renewed-full";

pub const NEW_CONTENTS: &[u8] = b"artificial-renewed-contents";

#[must_use]
pub fn ready() -> Harness {
    initial(CONTENTS_EXPIRY)
}

#[must_use]
pub fn initial(contents_expiry: &[u8]) -> Harness {
    let mut h = minted_pair(EXPIRES, contents_expiry);
    h.answer(FORGE, &self::ok(VIEWER));
    h.run(Duration::ZERO);
    // Let the idle socket settle before crossing the renewal timer.
    h.run(h.env.limits.idle);
    h
}

#[must_use]
pub fn minted_pair(full_expiry: &[u8], contents_expiry: &[u8]) -> Harness {
    let mut h = Harness::new(self::github_settings());
    accepted(&mut h, &[1]);
    h.connected(FORGE);
    h.answer(FORGE, &self::ok(LIVE));
    h.answer(FORGE, &mint(FULL, full_expiry));
    h.answer(FORGE, &mint(CONTENTS, contents_expiry));
    h
}

pub fn advance(h: &mut Harness, seconds: u64) {
    h.run(Time::from_nanos(Duration::from_secs(seconds).as_nanos()).saturating_since(h.env.now));
}

#[must_use]
pub fn renewing() -> Harness {
    let mut h = ready();
    advance(&mut h, FULL_DUE);
    h.connected(FORGE);
    h
}

#[must_use]
pub fn contents_renewing() -> Harness {
    let mut h = renewing();
    renewed_full(&mut h);
    h.run(h.env.limits.idle);
    advance(&mut h, CONTENTS_DUE);
    h.connected(FORGE);
    h
}

pub fn renewed_full(h: &mut Harness) {
    h.answer(FORGE, &mint(NEW_FULL, LATER));
    h.run(Duration::ZERO);
}

pub fn renewed_contents(h: &mut Harness) {
    h.answer(FORGE, &mint(NEW_CONTENTS, LATER));
    h.run(Duration::ZERO);
}

pub fn github_account(h: &mut Harness, installation: u64, account: u64, login: &[u8], token: &[u8]) {
    let gh = h.component.test_github_mut();
    let _accepted = gh.accepted.insert(installation).expect("fixture accepted");
    drop(
        gh.rows
            .insert(
                installation,
                Row {
                    state: Tokens::Current,
                    revision: 0,
                    token_revision: 1,
                    refused_revision: None,
                    full: Clock { expiry: Time::from_nanos(3_600_000_000_000), ..Clock::empty() },
                    contents: Clock::empty(),
                    account: skein_forge_connection::Account { id: account, login: Box::from(login) },
                    suspended: false,
                    accepted: true,
                    api: Some(Secret::new(Box::from(token))),
                    permissions: skein_forge::github::document::Levels::NONE,
                    api_expires: Wall::from_nanos(3_600_000_000_000),
                    git_expires: Wall::from_nanos(3_600_000_000_000),
                },
            )
            .expect("fixture installation"),
    );
    let account = gh.rows.get(&installation).expect("fixture row").account.clone();
    drop(
        gh.known
            .insert(installation, skein_forge_connection::seam::Accepted { known: Some(account), checked: true })
            .expect("fixture accepted bookkeeping"),
    );
}

/// Seeds checked call state without opening a startup connection.
#[must_use]
pub fn github_calls() -> Harness {
    let mut settings = github_settings();
    settings.forges.get_mut(0).expect("forge").access = None;
    let mut h = Harness::new(settings);
    h.component.test_calls_ready(app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
    github_account(&mut h, 1, 22, b"acme", b"artificial-full-one");
    h
}
