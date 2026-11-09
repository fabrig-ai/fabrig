use crate::{
    pages::only_you::{self, Item, Why},
    tests::{fixtures, playground},
};
use core::slice::from_ref;
use jig_client::contract as c;
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};

const PROVIDER: u32 = 17;
const ACCOUNT: u32 = 29;
const OTHER: u32 = 11;
const SOURCE: u32 = 43;
const SIGNED_OUT: Item = Item { account: ACCOUNT, source: SOURCE, kind: c::AccountKind::SignIn, why: Why::SignedOut };
const EXPIRING: Item = Item { why: Why::Expiring, ..SIGNED_OUT };

fn account(number: u32, kind: c::AccountKind, state: c::AccountState, expiring: bool) -> c::AccountLine {
    let document = playground::accounts(Some(PROVIDER), true, false);
    let mut account = document.accounts.get(0).unwrap().clone();
    account.number = number;
    let source = account.sources.get_mut(0).unwrap();
    source.number = SOURCE;
    source.kind = kind;
    source.state = state;
    source.expiring = expiring;
    account
}

fn document(accounts: &[c::AccountLine], cut: bool) -> c::Accounts {
    let mut document = playground::accounts(Some(PROVIDER), true, false);
    document.accounts = List::with_capacity(u32::try_from(accounts.len()).unwrap());
    for account in accounts {
        document.accounts.push(account.clone()).unwrap();
    }
    document.cut = cut;
    document
}

type Row<'a> = (&'a str, c::Accounts, Option<u32>, &'a [Item], bool);

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the deciding-source rule and its exclusions")]
fn only_you_reads_the_persons_deciding_login_sources_in_document_order() {
    let granted = account(ACCOUNT, c::AccountKind::SignIn, c::AccountState::Granted, false);
    let signed_out = account(ACCOUNT, c::AccountKind::SignIn, c::AccountState::SignInAgain, false);
    let expiring = account(ACCOUNT, c::AccountKind::SignIn, c::AccountState::Granted, true);
    let mut deployment = signed_out.clone();
    deployment.holder = c::AccountHolder::Deployment;
    let mut other_party = signed_out.clone();
    other_party.holder = c::AccountHolder::Party(fixtures::PARTY + 1);
    let mut no_provider = signed_out.clone();
    no_provider.provider = None;
    let mut no_source = signed_out.clone();
    no_source.sources.clear();
    let mut later = granted.clone();
    later.sources = List::with_capacity(2);
    later.sources.push(granted.sources.get(0).unwrap().clone()).unwrap();
    let mut second = signed_out.sources.get(0).unwrap().clone();
    second.number = SOURCE + 1;
    later.sources.push(second).unwrap();
    let other = account(OTHER, c::AccountKind::Program, c::AccountState::SignInAgain, false);
    let rows: &[Row<'_>] = &[
        ("none delivered", document(&[], false), None, &[], false),
        ("granted", document(&[granted], false), None, &[], false),
        ("signed out", document(from_ref(&signed_out), false), None, &[SIGNED_OUT], false),
        ("expiring", document(from_ref(&expiring), false), None, &[EXPIRING], false),
        (
            "signed out and expiring",
            document(&[account(ACCOUNT, c::AccountKind::SignIn, c::AccountState::SignInAgain, true)], false),
            None,
            &[SIGNED_OUT],
            false,
        ),
        (
            "program signed out",
            document(&[account(ACCOUNT, c::AccountKind::Program, c::AccountState::SignInAgain, false)], false),
            None,
            &[Item { kind: c::AccountKind::Program, ..SIGNED_OUT }],
            false,
        ),
        (
            "key signed out",
            document(&[account(ACCOUNT, c::AccountKind::Key, c::AccountState::SignInAgain, false)], false),
            None,
            &[],
            false,
        ),
        (
            "no credential signed out",
            document(&[account(ACCOUNT, c::AccountKind::None, c::AccountState::SignInAgain, false)], false),
            None,
            &[],
            false,
        ),
        ("deployment", document(&[deployment], false), None, &[], false),
        ("other party", document(&[other_party], false), None, &[], false),
        ("playground", document(from_ref(&signed_out), false), Some(PROVIDER), &[], false),
        ("other provider", document(from_ref(&signed_out), false), Some(PROVIDER + 1), &[SIGNED_OUT], false),
        ("no provider", document(from_ref(&no_provider), false), None, &[SIGNED_OUT], false),
        ("no provider with playground", document(&[no_provider], false), Some(PROVIDER), &[SIGNED_OUT], false),
        ("no source", document(&[no_source], false), None, &[], false),
        ("later source signed out", document(&[later], false), None, &[], false),
        (
            "document order",
            document(&[other, expiring], false),
            None,
            &[Item { account: OTHER, kind: c::AccountKind::Program, ..SIGNED_OUT }, EXPIRING],
            false,
        ),
        ("cut", document(&[signed_out], true), None, &[SIGNED_OUT], true),
        ("empty cut", document(&[], true), None, &[], true),
    ];
    for (case, document, playground, expected, cut) in rows {
        let read = only_you::items(document, fixtures::PARTY, *playground);
        assert_eq!(read.items.as_slice(), *expected, "{case}");
        assert_eq!(read.cut, *cut, "{case}: cut is the document's");
        assert_eq!(read.items.room(), 0, "{case}: capacity is the counted items");

        let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
        let (me, accounts) = fixtures::start_accounts(&mut root, List::with_capacity(0));
        assert!(only_you::read(&root).items.is_empty(), "{case}: no document");
        assert!(!only_you::read(&root).cut, "{case}: no document is not cut");
        super::accounts::deliver(
            &mut root,
            accounts,
            c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(document.clone()))),
        );
        assert!(only_you::read(&root).items.is_empty(), "{case}: no me");
        assert!(!only_you::read(&root).cut, "{case}: no me is not cut");
        if let Some(provider) = playground {
            let mut parts =
                wire::decode_playground_answer(&playground::answer(b"A change"), &w::CEILINGS).unwrap().into_parts();
            parts.provider = *provider;
            playground::deliver_playground_read(
                &mut root,
                jig_client::Readout::Family {
                    family: wire::FORGE,
                    record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
                },
            );
        }
        super::accounts::deliver(&mut root, me, fixtures::frame(fixtures::PARTY));
        assert_eq!(only_you::read(&root), read, "{case}: read gathers the root's facts");
        assert_eq!(
            crate::pages::parts::needs_facts(&root, skein_lib::Wall::EPOCH).only,
            u32::try_from(expected.len()).unwrap(),
            "{case}: the page counts Only you separately"
        );
    }
}
