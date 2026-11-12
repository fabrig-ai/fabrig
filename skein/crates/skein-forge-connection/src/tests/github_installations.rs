//! Installation reconciliation through the component boundary and the landed page plan.
use super::fixtures;
use crate::seam::Access;
use crate::{Account, Change, Component, Event};
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
use skein_lib::{Env, List, Queue, Time, Wall};

const PAGE_CAP: u32 = skein_forge::github::pin::PAGE_CAP;

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
    let limits = crate::Limits { installations: 1, ..fixtures::limits() };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut component = Component::new(fixtures::github_settings(), &limits, &fixtures::shared()).expect("fixture");
    let mut up = Queue::uncapped();
    let gh = component.test_github_mut();
    gh.accept(&[1]);
    gh.begin_read(limits.start_backoff);
    let other = document::Installation {
        id: 2,
        account: 23,
        login: Box::from(b"other".as_slice()),
        kind: document::Account::User,
        created: Wall::from_nanos(1_000_000_000),
        suspended: true,
    };
    component.test_paged(Box::from([other]), true, &env, &mut up);
    component.test_local_step(&env, &mut up);
    let mut suspended = fixtures::installation();
    suspended.suspended = true;
    component.test_paged(Box::from([suspended]), false, &env, &mut up);
    component.test_local_step(&env, &mut up);
    let mut changes = List::with_capacity(1);
    let mut facts = 0_u32;
    while let Some(event) = up.pop() {
        if let Event::Installation { installation, account, change, .. } = event {
            changes.push((installation, account, change)).expect("one change");
        } else if let Event::Fact { .. } = event {
            facts = facts.checked_add(1).expect("fixture fact count");
        }
    }
    assert_eq!(changes.as_slice(), [(1, Some(account()), Change::Suspended)]);
    assert_eq!(facts, 1);
    assert_eq!(page_outcome(PAGE_CAP, None).expect("full page").rows.len(), 100);
}

#[test]
fn installation_page_over_cap_is_a_failed_read() {
    assert_eq!(
        page_outcome(PAGE_CAP + 1, None).expect_err("too many rows"),
        api::Error::TooLarge(api::Excess::unnamed())
    );
}

#[test]
fn installation_page_wrong_row_kind_is_a_failed_read() {
    assert_eq!(
        page_outcome(0, Some(document::Row::Branch(Box::from(b"main".as_slice())))).expect_err("wrong row kind"),
        api::Error::InvalidAnswer
    );
}
