//! Installation reconciliation through the component boundary and the landed page plan.
use super::fixtures::{self, FORGE, Harness};
use crate::{Account, Change, Event, Request};
use alloc::boxed::Box;
use skein_forge::{
    api,
    github::{
        document,
        plans::{
            Body, Heard,
            start::{Installations, Step},
        },
    },
};
use skein_lib::{Duration, List, Wall};

const SUSPENDED: &[u8] = br#"[{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":"1970-01-01T00:00:02Z"}]"#;

const OTHER: &[u8] = br#"[{"id":2,"account":{"id":23,"login":"other","type":"User"},"created_at":"1970-01-01T00:00:01Z","suspended_at":"1970-01-01T00:00:02Z"}]"#;

const NEXT_PAGE: &[u8] = b"Link: <https://api.github.test/app/installations?per_page=100&page=2>; rel=\"next\"\r\n";

const PAGE_CAP: u32 = skein_forge::github::pin::PAGE_CAP;

fn accepted(h: &mut Harness, ids: &[u64]) {
    h.send(Request::Installations { forge: FORGE, accepted: Box::from(ids) });
}

fn harness(cap: u32, ids: &[u64]) -> Harness {
    let mut h = Harness::with(
        crate::Limits { installations: cap, ..fixtures::limits() },
        fixtures::shared(),
        fixtures::github_settings(),
    );
    accepted(&mut h, ids);
    h.connected(FORGE);
    h
}

fn changes(h: &mut Harness) -> Box<[(u64, Option<Account>, Change)]> {
    h.run(Duration::ZERO);
    let events = h.installation_events.as_mut().expect("GitHub fixture");
    let mut out = List::with_capacity(events.len());
    for event in events.as_slice() {
        match event {
            Event::Installation { installation, account, change, .. } => {
                out.push((*installation, account.clone(), *change)).expect("measured changes");
            }
            Event::Fact { .. }
            | Event::GitToken { .. }
            | Event::Answered(_)
            | Event::Hint(_)
            | Event::Listening { .. }
            | Event::ListenerLost
            | Event::Connected { .. }
            | Event::Stopped { .. }
            | Event::Closed => unreachable!("installation collection"),
        }
    }
    events.clear();
    out.into_boxed()
}

fn account() -> Account {
    Account { id: 22, login: Box::from(b"acme".as_slice()) }
}

fn document_row(id: u64, login_bytes: usize) -> document::Row {
    document::Row::Installation(document::Installation {
        id,
        login: fixtures::filled(login_bytes, b'x'),
        account: 22,
        kind: document::Account::Organization,
        created: Wall::EPOCH,
        suspended: false,
    })
}

fn page_outcome(
    count: u32,
    row: Option<document::Row>,
) -> Result<skein_forge::github::plans::start::InstallationPage, api::Error> {
    let bounds = crate::limits::installation_bounds(&fixtures::limits(), &fixtures::shared()).expect("bounds");
    let mut plan = Installations::new(1);
    drop(plan.begin(bounds).expect("begin page"));
    let head = Heard::Head { status: 200, head: skein_forge::github::header::Head::new() };
    drop(plan.heard(head, bounds).expect("head"));
    for id in 0..count {
        if let Some(step) = plan.heard(Heard::Row(document_row(u64::from(id), 4)), bounds).expect("row") {
            return outcome(step);
        }
    }
    if let Some(row) = row
        && let Some(step) = plan.heard(Heard::Row(row), bounds).expect("extra row")
    {
        return outcome(step);
    }
    outcome(
        plan.heard(Heard::Done(Body::Document(document::Document::Rows)), bounds).expect("done").expect("page result"),
    )
}

fn outcome(
    step: Step<skein_forge::github::plans::start::InstallationPage>,
) -> Result<skein_forge::github::plans::start::InstallationPage, api::Error> {
    match step {
        Step::Done(result) => result,
        Step::Send(_) => unreachable!("single page"),
    }
}

#[test]
fn installation_page_limits_are_not_table_overflow() {
    let mut h = harness(1, &[1]);
    h.answer(FORGE, &fixtures::response_with_fields(200, OTHER, NEXT_PAGE));
    h.answer(FORGE, &fixtures::ok(SUSPENDED));
    assert_eq!(changes(&mut h).as_ref(), [(1, Some(account()), Change::Suspended)]);
    assert_eq!(h.facts.as_ref().expect("facts").len(), 1);
    assert_eq!(page_outcome(PAGE_CAP, None).expect("full page").rows.len(), 100);
}

#[test]
fn installation_page_over_cap_is_a_failed_read() {
    assert_eq!(page_outcome(PAGE_CAP + 1, None).expect_err("too many rows"), api::Error::TooLarge);
}

#[test]
fn installation_page_wrong_row_kind_is_a_failed_read() {
    assert_eq!(
        page_outcome(0, Some(document::Row::Branch(Box::from(b"main".as_slice())))).expect_err("wrong row kind"),
        api::Error::InvalidAnswer
    );
}
