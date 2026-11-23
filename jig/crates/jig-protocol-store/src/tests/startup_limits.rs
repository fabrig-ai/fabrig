//! Runtime codecs change only before restore, never the stock envelopes.
use super::fixtures;
use crate::{Event, InstallRefusal, Request, Write};
use jig_domain_core_vocabulary::{Record, RestoreRange};
use skein_lib::Token;

fn opened() -> crate::Store {
    let mut limits = fixtures::store_limits();
    limits.commits.writes = 3;
    let mut store = crate::Store::over(
        &crate::check(&limits).expect("fixture store limits"),
        fixtures::APPLICATION,
        &[],
        fixtures::stores(),
    );
    let events = fixtures::run(
        &mut store,
        Request::Open {
            purpose: crate::Purpose::Serve,
            owner: Token::new(1),
            root: fixtures::ROOT,
            installation: fixtures::INSTALLATION,
        },
    );
    let Some(Event::Opened { .. }) = events.first() else {
        panic!("store opens: {events:?}");
    };
    store
}
#[test]
fn the_runtime_check_is_the_installed_and_reopened_view() {
    let stock = opened().checked();
    let mut custom = stock.limits().core;
    custom.notes.body_bytes = 8;
    for core in [stock.limits().core, custom] {
        let mut store = opened();
        let expected = crate::runtime_limits(stock.limits(), &core, fixtures::APPLICATION, 64).unwrap();
        assert_eq!(expected.codec().core.turn_record_transcript, 64);
        store.install_limits(&core, fixtures::APPLICATION, 64).unwrap();
        assert_eq!(store.checked(), expected);
        let reopened = fixtures::reopen(store);
        assert_eq!(reopened.checked(), expected);
    }
}

#[test]
fn installed_codecs_encode_only_records_within_the_resolved_ceiling() {
    let mut store = opened();
    let stock = store.limits;
    let (jig, application) = store.limit_settings(Box::new([41])).unwrap();
    assert!(jig.is_none() && application.is_none());
    let mut core = stock.core;
    core.notes.body_bytes = 8;
    store.install_limits(&core, fixtures::APPLICATION, 64).unwrap();
    assert_eq!(store.limits.store, stock.store);
    assert_eq!(store.limits.commits, stock.commits);
    let mut entry = fixtures::note_entry();
    entry.body = Box::new([b'a'; 8]);
    let good = fixtures::run(
        &mut store,
        fixtures::commit(
            1,
            Box::new([fixtures::header(1), Write::Save(Record::Notes(jig_domain_notes::Record::Entry(entry.clone())))]),
        ),
    );
    let [Event::Committed { number: 1, .. }] = good.as_ref() else {
        panic!("bounded note commits: {good:?}");
    };
    entry.body = Box::new([b'a'; 9]);
    let bad = fixtures::run(
        &mut store,
        fixtures::commit(
            2,
            Box::new([fixtures::header(2), Write::Save(Record::Notes(jig_domain_notes::Record::Entry(entry)))]),
        ),
    );
    let [Event::Failed { why: crate::CommitFailure::Check(crate::CommitCheck::Encoding), .. }] = bad.as_ref() else {
        panic!("oversized note refused: {bad:?}");
    };
}
#[test]
fn installation_refuses_after_restore_or_a_non_setting_read() {
    for request in [
        Request::Restore {
            owner: Token::new(1),
            range: crate::Restore::Jig(Box::new([RestoreRange::Deployment])),
            after: None,
            rows: 1,
            bytes: 1_048_576,
        },
        Request::Get { owner: Token::new(1), key: crate::Get::Application(Box::new([41])) },
    ] {
        let mut store = opened();
        let core = store.limits.core;
        fixtures::run(&mut store, request);
        assert_eq!(store.install_limits(&core, fixtures::APPLICATION, 64), Err(InstallRefusal::NotOpening));
    }
}
#[test]
fn installation_names_the_stock_figure_it_would_pass() {
    let mut store = opened();
    let core = store.limits.core;
    let bound = store.limits.store.value;
    let application = crate::sizes::Application { value: bound + 1, ..fixtures::APPLICATION };
    assert_eq!(
        store.install_limits(&core, application, 64),
        Err(InstallRefusal::Store(crate::sizes::SizeRefusal::Value { needs: bound + 1, has: bound }))
    );
}
#[test]
fn an_open_without_installation_keeps_todays_codec() {
    let mut store = opened();
    let mut entry = fixtures::note_entry();
    entry.body = Box::new([b'a'; 9]);
    let events = fixtures::run(
        &mut store,
        fixtures::commit(
            1,
            Box::new([fixtures::header(1), Write::Save(Record::Notes(jig_domain_notes::Record::Entry(entry)))]),
        ),
    );
    let [Event::Committed { number: 1, .. }] = events.as_ref() else {
        panic!("stock note commits: {events:?}");
    };
}
