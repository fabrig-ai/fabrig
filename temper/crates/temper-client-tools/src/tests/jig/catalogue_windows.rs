use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;

const ROWS: u32 = 4;
const PAST_ROWS: u32 = 5;
#[test]
fn accounts_fit_at_the_row_window() {
    let printed = accounts_at(ROWS);
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":false}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"number\":"), usize::try_from(ROWS).expect("rows"));
}
#[test]
fn accounts_are_cut_past_the_row_window() {
    let printed = accounts_at(PAST_ROWS);
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"number\":"), usize::try_from(ROWS).expect("rows"));
}
fn accounts_at(count: u32) -> crate::jig::Printed {
    let mut party = f::accounts();
    let mut row = party.accounts.get(0).expect("account").clone();
    row.sources = List::with_capacity(0);
    row.uses = None;
    party.accounts = List::with_capacity(count);
    for number in 0..count {
        row.number = number;
        party.accounts.push(row.clone()).expect("account room");
    }
    let mut root = f::account_root(false);
    root.limits.tools.rows = ROWS;
    f::account_read(&mut root, party, None)
}
#[derive(Clone, Copy)]
enum Side {
    Party,
    Deployment,
}
#[test]
fn party_readiness_fits_at_its_window() {
    let printed = readiness_at(Side::Party, ROWS);
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":false}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"payer\":"), usize::try_from(ROWS).expect("rows"));
}
#[test]
fn party_readiness_is_cut_past_its_window() {
    let printed = readiness_at(Side::Party, PAST_ROWS);
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"payer\":"), usize::try_from(ROWS).expect("rows"));
}
#[test]
fn deployment_readiness_fits_at_its_window() {
    let printed = readiness_at(Side::Deployment, ROWS);
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":false}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"payer\":"), usize::try_from(ROWS).expect("rows"));
}
#[test]
fn deployment_readiness_is_cut_past_its_window() {
    let printed = readiness_at(Side::Deployment, PAST_ROWS);
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"payer\":"), usize::try_from(ROWS).expect("rows"));
}
fn readiness(count: u32, value: c::Readiness) -> List<c::PayerReadiness> {
    let mut rows = List::with_capacity(count);
    for provider in 0..count {
        rows.push(c::PayerReadiness {
            provider,
            payer: c::OwnerAccountUse::Party(f::PARTY),
            readiness: value,
            settled: None,
        })
        .expect("readiness room");
    }
    rows
}
fn readiness_at(side: Side, count: u32) -> crate::jig::Printed {
    let mut party = f::accounts();
    let mut deployment = f::deployment();
    party.accounts = List::with_capacity(0);
    deployment.accounts = List::with_capacity(0);
    match side {
        Side::Party => party.readiness = readiness(count, c::Readiness::Unknown),
        Side::Deployment => deployment.readiness = readiness(count, c::Readiness::Usable),
    }
    let mut root = f::account_root(true);
    root.limits.tools.rows = ROWS;
    f::account_read(&mut root, party, Some(deployment))
}
#[test]
fn readiness_lists_have_independent_windows() {
    let mut party = f::accounts();
    let mut deployment = f::deployment();
    party.accounts = List::with_capacity(0);
    deployment.accounts = List::with_capacity(0);
    party.readiness = readiness(ROWS, c::Readiness::Unknown);
    deployment.readiness = readiness(ROWS, c::Readiness::Usable);
    let mut root = f::account_root(true);
    root.limits.tools.rows = ROWS;
    let printed = f::account_read(&mut root, party, Some(deployment));
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":false}"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"readiness\":\"unknown\""), usize::try_from(ROWS).expect("rows"));
    assert_eq!(f::occurrences(&printed.stdout, b"\"readiness\":\"usable\""), usize::try_from(ROWS).expect("rows"));
}
const PROVIDERS: u32 = 2;
const PROGRAMS: u32 = 2;
#[test]
fn me_offers_fit_at_the_contract_window() {
    for json in [false, true] {
        let printed = me_at(PROGRAMS, json);
        assert!(f::has(&printed, if json { b"\"cut\":false,\"retention\":" } else { b"\ncut: false\n" }));
    }
}
#[test]
fn me_offers_are_cut_past_the_contract_window() {
    const PAST_PROGRAMS: u32 = 3;
    for json in [false, true] {
        let printed = me_at(PAST_PROGRAMS, json);
        assert!(f::has(&printed, if json { b"\"cut\":true,\"retention\":" } else { b"\ncut: true\n" }));
    }
}
fn me_at(last_programs: u32, json: bool) -> crate::jig::Printed {
    let mut me = f::me(f::PARTY);
    me.providers = List::with_capacity(PROVIDERS);
    for provider in 0..PROVIDERS {
        let mut offer = f::provider_offers().get(0).expect("offer").clone();
        offer.provider = provider;
        let count = if provider == PROVIDERS.checked_sub(1).expect("last provider") { last_programs } else { PROGRAMS };
        offer.programs = List::with_capacity(count);
        for _ in 0..count {
            offer.programs.push(c::Text::new(b"helper".as_slice().into(), 32).expect("program")).expect("program room");
        }
        me.providers.push(offer).expect("provider room");
    }
    let mut root = f::account_root(true);
    root.limits.tools.rows = PROVIDERS;
    root.deliver(c::Delivery::Me(Box::new(me)));
    let call = root.submit(if json { &["me", "--json"] } else { &["me"] });
    root.answer(call).expect("me projection")
}
