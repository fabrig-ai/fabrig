use super::*;
use crate::make::{Refusal, kind_busy, make};
use crate::record as r;
use skein_lib::List;

fn new(seed: &[Forge]) -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config { namespace: b"make".as_slice().into(), writers: Box::new([]) },
        seed,
        false,
    )
    .unwrap()
}

fn started(seed: &[Forge]) -> Domain {
    let mut d = new(seed);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    d
}

fn record(number: u16, identity: u32) -> Forge {
    let mut row = records::forge();
    row.number = number;
    row.identity.number = identity;
    row
}

fn setup(mode: r::SetupMode) -> r::Setup {
    r::Setup {
        directory: b"forge".as_slice().into(),
        bot: r::Account { login: b"temper".as_slice().into(), email: Box::new([]) },
        orgs: Box::new([]),
        mode,
        asked: true,
    }
}

fn owned() -> Forge {
    Forge {
        kind: Kind::Forgejo,
        setup: Some(setup(r::SetupMode::Owned(r::OwnedEntry {
            public: None,
            fetch: r::Fetch { forgejo: true, runner: false, mirror: None },
            forgejo: b"forgejo".as_slice().into(),
            runner: None,
            listen: core::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
            person: r::Account { login: b"person".as_slice().into(), email: Box::new([]) },
        }))),
        ..records::forge()
    }
}

fn playground() -> Forge {
    Forge {
        kind: Kind::Forgejo,
        playground: true,
        setup: Some(setup(r::SetupMode::Fake(r::FakeEntry {
            listen: core::net::SocketAddr::from(([127, 0, 0, 1], 3001)),
            git: b"fake".as_slice().into(),
            owner: b"playground".as_slice().into(),
            repositories: Box::new([]),
        }))),
        ..records::forge()
    }
}

fn refused(d: &mut Domain, forge: Forge, refusal: Refusal) {
    let next = d.next_forge;
    let mut rows = List::with_capacity(LIMITS.forges);
    let mut configured = List::with_capacity(LIMITS.forges);
    let mut identities = List::with_capacity(LIMITS.forges);
    for (_, row) in &d.forge_records {
        rows.push(row.clone()).unwrap();
    }
    for (_, value) in &d.configured {
        configured.push(value.clone()).unwrap();
    }
    for (number, identity) in &d.identities {
        identities.push((*number, *identity)).unwrap();
    }
    let mut out = Queue::uncapped();
    out.push(Request::MarkedForges);
    assert_eq!(make(d, &env(), forge, &mut out), Err(refusal));
    assert_eq!(d.next_forge, next);
    assert_eq!(d.forge_records.len(), rows.len());
    assert_eq!(d.configured.len(), configured.len());
    assert_eq!(d.identities.len(), identities.len());
    for row in &rows {
        assert_eq!(d.forge_record(row.number), Some(row));
    }
    for value in &configured {
        assert_eq!(d.configured(value.number), Some(value));
    }
    for (number, identity) in &identities {
        assert_eq!(d.identities.get(number), Some(identity));
    }
    assert_eq!(out.pop(), Some(Request::MarkedForges));
    assert!(out.is_empty());
}

#[test]
fn a_first_make_is_forge_one() {
    let mut d = started(&[]);
    let mut out = Queue::uncapped();
    assert_eq!(make(&mut d, &env(), record(u16::MAX, u32::MAX), &mut out), Ok(1));
    assert_eq!(d.forge_record(1), Some(&record(1, 1)));
    assert_eq!(d.next_forge(), 2);
}

#[test]
fn numbers_follow_the_highest_ever_kept() {
    let seeded = started(&[record(3, 3), record(1, 1), record(2, 2)]);
    let mut restored = new(&[]);
    outputs(&mut restored, Event::Restore { record: Stored::Forges { next: 7 } });
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    for (mut d, number, next) in [(seeded, 4, 5), (restored, 7, 8)] {
        let mut out = Queue::uncapped();
        assert_eq!(make(&mut d, &env(), records::forge(), &mut out), Ok(number));
        assert_eq!(d.next_forge(), next);
    }
}

#[test]
fn a_made_forge_is_configured_and_told() {
    let mut d = started(&[]);
    let mut out = Queue::uncapped();
    let forge = Forge { number: 1, identity: r::Identity { number: 1, ..owned().identity }, ..owned() };
    assert_eq!(make(&mut d, &env(), owned(), &mut out), Ok(1));
    assert_eq!(d.forge_record(1), Some(&forge));
    assert_eq!(d.configured(1), Some(&Configured::from_record(&forge)));
    assert_eq!(d.identities.get(&1), Some(&1));
    for request in [
        Request::Save { record: Stored::Forge(forge.clone()), release: None, read_afresh: None },
        Request::Save { record: Stored::Forges { next: 2 }, release: None, read_afresh: None },
        Request::MarkedForges,
        Request::ForgeKept { forge: Box::new(forge) },
    ] {
        assert_eq!(out.pop(), Some(request));
    }
    assert!(out.is_empty());
}

#[test]
fn identity_numbers_follow_the_highest_remaining_record() {
    let removing = Forge { removing: true, ..record(2, 90) };
    let mut d = started(&[record(1, 40), removing, record(3, 12)]);
    let mut out = Queue::uncapped();
    assert_eq!(make(&mut d, &env(), record(0, u32::MAX), &mut out), Ok(4));
    assert_eq!(d.forge_record(4), Some(&record(4, 41)));
    assert_eq!(d.identities.get(&4), Some(&41));
}

#[test]
fn a_second_owned_or_playground_forge_is_busy() {
    for forge in [owned(), playground()] {
        let mut d = started(core::slice::from_ref(&forge));
        assert!(kind_busy(&d, &forge));
        refused(&mut d, forge, Refusal::Busy);
    }
}

#[test]
fn removing_forges_free_their_box_kind() {
    for forge in [owned(), playground()] {
        let removing = Forge { removing: true, ..forge.clone() };
        let mut d = started(&[removing]);
        assert!(!kind_busy(&d, &forge));
        let mut out = Queue::uncapped();
        assert_eq!(make(&mut d, &env(), forge, &mut out), Ok(3));
    }
}

#[test]
fn different_box_kinds_can_be_kept_together() {
    for (first, second) in [(owned(), playground()), (playground(), owned())] {
        let mut d = started(&[first]);
        assert!(!kind_busy(&d, &second));
        let mut out = Queue::uncapped();
        assert_eq!(make(&mut d, &env(), second, &mut out), Ok(3));
    }
}

#[test]
fn connected_managed_and_other_fake_forges_do_not_take_a_box_kind() {
    let managed = Forge { setup: Some(setup(r::SetupMode::Managed { admin_token: None })), ..owned() };
    let fake = Forge { playground: false, ..playground() };
    for forge in [records::forge(), managed, fake] {
        let mut d = started(core::slice::from_ref(&forge));
        assert!(!kind_busy(&d, &forge));
        let mut out = Queue::uncapped();
        assert_eq!(make(&mut d, &env(), forge, &mut out), Ok(3));
    }
}

#[test]
fn full_and_invalid_records_change_nothing() {
    let mut full = started(&[record(1, 1), record(2, 2), record(3, 3), record(4, 4)]);
    refused(&mut full, records::forge(), Refusal::Full);
    let invalid = Forge { git: skein_lib::bytes::zeroed(r::PATH_BYTES.checked_add(1).unwrap()), ..records::forge() };
    let mut d = started(&[record(1, 1)]);
    refused(&mut d, invalid, Refusal::Invalid);
}

#[test]
fn a_record_at_the_schema_bound_is_kept() {
    let forge = Forge { git: skein_lib::bytes::zeroed(r::PATH_BYTES), ..records::forge() };
    let mut d = started(&[]);
    let mut out = Queue::uncapped();
    assert_eq!(make(&mut d, &env(), forge.clone(), &mut out), Ok(1));
    assert_eq!(d.forge_record(1).unwrap().git, forge.git);
}

#[test]
fn exhausted_numbers_change_nothing() {
    let mut d = started(&[]);
    outputs(&mut d, Event::Restore { record: Stored::Forges { next: u16::MAX - 1 } });
    let mut out = Queue::uncapped();
    assert_eq!(make(&mut d, &env(), records::forge(), &mut out), Ok(u16::MAX - 1));
    assert_eq!(d.next_forge(), u16::MAX);
    refused(&mut d, records::forge(), Refusal::Exhausted);
}

#[test]
fn exhausted_identity_numbers_change_nothing() {
    let mut d = started(&[record(1, u32::MAX)]);
    refused(&mut d, records::forge(), Refusal::Exhausted);
}

#[test]
fn a_made_forge_restores_as_any_record() {
    let mut d = started(&[]);
    let mut out = Queue::uncapped();
    let mut forge = playground();
    forge.first = Some(r::FirstIntent {
        org: b"playground".as_slice().into(),
        repository: b"widget".as_slice().into(),
        project: SetUpProject::Existing(1),
        label: b"Widget".as_slice().into(),
    });
    assert_eq!(make(&mut d, &env(), forge, &mut out), Ok(1));
    let mut restored = new(&[]);
    let mut saves = 0_u32;
    for _ in 0..out.len() {
        if let Request::Save { record, .. } = out.pop().unwrap() {
            outputs(&mut restored, Event::Restore { record });
            saves += 1;
        }
    }
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
    assert_eq!(saves, 2);
    assert_eq!(restored.restore_fit_failure(), None);
    assert_eq!(restored.fit_restored(&LIMITS), Ok(()));
    assert_eq!(restored.forge_record(1), d.forge_record(1));
    assert_eq!(restored.configured(1), d.configured(1));
    assert_eq!(restored.identities.get(&1), Some(&1));
    assert_eq!(make(&mut restored, &env(), records::forge(), &mut out), Ok(2));
    assert_eq!(restored.forge_record(2), Some(&record(2, 2)));
}
