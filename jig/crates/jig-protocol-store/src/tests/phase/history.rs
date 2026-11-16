//! Every commit routes history to logs (protocol/store/logs.md, 3).
use super::*;

#[test]
fn history_routes_while_the_first_commits_done_row_waits_for_sync() {
    let mut disk = Disk::new(&[]);
    let mut store = jig_store(&disk, crate::JIG_STORE_PATCH_SET);
    assert_opened(&disk.open(&mut store), 0);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(1, Box::new([fixtures::header(1)])),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    let row = fixtures::attempt_row(5, 1);
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(2, Box::new([fixtures::header(2), fixtures::save(row.clone())])),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    disk.drive(&mut below, |io, below| crate::up(&mut store, &fixtures::env(), io, &mut above, below));
    let events = fixtures::events(&mut above);
    let [Event::Committed { number: 1, .. }, Event::Committed { number: 2, .. }] = events.as_ref() else {
        panic!("{events:?}")
    };
    let key = crate::keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap();
    assert!(crate::view::raw_get(&mut store, key).unwrap().is_none());
    assert!(store.stores.store.log_end(crate::logs::history(5).unwrap()) > 0);
}

#[test]
fn a_fresh_stores_first_commits_put_history_only_in_logs_and_read_it_through_the_view() {
    for registered in [false, true] {
        let mut disk = Disk::new(&[]);
        let mut store = if registered { jig_store(&disk, crate::JIG_STORE_PATCH_SET) } else { disk.store() };
        assert_opened(&disk.open(&mut store), 0);
        for number in [1, 2] {
            let record = fixtures::history(5, number);
            let events = disk.request(
                &mut store,
                fixtures::commit(number, Box::new([fixtures::header(number), fixtures::save(record.clone())])),
            );
            assert!({ if let [Event::Committed { .. }] = events.as_ref() { true } else { false } }, "{events:?}");
            let key = crate::keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap();
            assert_eq!(fixtures::get_kv(&mut store, &key), None);
            let events = disk.request(&mut store, Request::Get { owner: Token::new(9), key: Get::Jig(record.key()) });
            let [Event::Got { row: Some(Row::Jig(found)), .. }] = events.as_ref() else { panic!("{events:?}") };
            assert_eq!(found, &record);
        }
    }
}
