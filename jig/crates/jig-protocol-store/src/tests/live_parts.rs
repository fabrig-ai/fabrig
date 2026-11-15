//! Live rows' writes, joins and missing parts (protocol/store/parts.md, 2 and 5).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "ordinary Rust live-parts assertions"
)]
use super::fixtures;
use crate::{Event, Get, Request, Restore, Row, Write, codec, commits, keys, restore, tasks_v13 as wire};
use jig_domain_core_vocabulary::{Key, Record, RestoreRange, reads};
use jig_domain_tasks::{self as tasks, Stored};
use skein_lib::{Reader, Token};

fn live() -> tasks::TaskRecord {
    let mut row = fixtures::task_record();
    row.number = 7;
    row.proposal = None;
    row.recurring = None;
    row.authority.models = tasks::Models::All;
    row
}
fn raw(key: tasks::Key) -> Box<[u8]> {
    keys::encode(&Key::Tasks(key), fixtures::KEY_LIMIT).unwrap()
}
fn hot(row: &tasks::TaskRecord) -> Write {
    Write::Save(Record::Tasks(Stored::Live(Box::new(row.clone()))))
}
fn cold(row: &tasks::TaskRecord) -> Write {
    Write::Save(fixtures::cold_task(row))
}
fn get(store: &mut crate::Store) -> Record {
    let events =
        fixtures::run(store, Request::Get { owner: Token::new(9), key: Get::Jig(Key::Tasks(tasks::Key::Live(7))) });
    let [Event::Got { row: Some(Row::Jig(record)), .. }] = events.as_ref() else { panic!("get: {events:?}") };
    record.clone()
}
fn operations(store: &mut crate::Store, number: u64, writes: Box<[Write]>) -> Box<[skein_kv::Op]> {
    commits::operations(store, number, writes, false).unwrap().ops
}
fn put_value(ops: &[skein_kv::Op], wanted: tasks::Key) -> wire::Value {
    let key = raw(wanted);
    let value = ops
        .iter()
        .find_map(|op| match op {
            skein_kv::Op::Put { key: found, value } if *found == key => Some(value),
            _ => None,
        })
        .expect("part put");
    wire::Row::decode(&wire::CEILINGS, &mut Reader::new(value)).unwrap().into_parts().value
}

#[test]
fn making_claiming_and_ending_put_only_their_parts_and_reads_join_whole() {
    let mut store = fixtures::opened();
    let mut row = live();
    let made = operations(&mut store, 1, Box::new([hot(&row), cold(&row)]));
    assert_eq!(made.len(), 2);
    assert!(match put_value(&made, tasks::Key::Live(7)) {
        wire::Value::LiveHot(_) => true,
        _ => false,
    });
    assert!(match put_value(&made, tasks::Key::LiveCold(7)) {
        wire::Value::LiveCold(_) => true,
        _ => false,
    });
    assert_eq!(raw(tasks::Key::LiveCold(7)).as_ref(), &[1, 1, 24, 0, 0, 0, 0, 0, 0, 0, 7]);
    fixtures::plant(&mut store.stores, &made);
    let before_cold = fixtures::get_kv(&mut store, &raw(tasks::Key::LiveCold(7)));
    row.attempt = 8;
    row.phase = tasks::Phase::Active(tasks::Active::Claimed { attempt: 8 });
    let claimed = operations(&mut store, 2, Box::new([hot(&row)]));
    assert_eq!(claimed.len(), 1);
    assert!(match put_value(&claimed, tasks::Key::Live(7)) {
        wire::Value::LiveHot(_) => true,
        _ => false,
    });
    fixtures::plant(&mut store.stores, &claimed);
    assert_eq!(fixtures::get_kv(&mut store, &raw(tasks::Key::LiveCold(7))), before_cold);
    let expected = Record::Tasks(Stored::Live(Box::new(row.clone())));
    assert_eq!(get(&mut store), expected);
    let loaded = restore::page(&mut store, Restore::Jig(Box::new([RestoreRange::Tasks])), None, 8, u64::MAX).unwrap();
    let [restore::Item::Row(Row::Jig(restored))] = loaded.items.as_ref() else { panic!("restart: {loaded:?}") };
    assert_eq!(restored, &expected);
    let events = fixtures::run(
        &mut store,
        Request::Read { owner: Token::new(9), range: fixtures::read(fixtures::ALICE, reads::Scope::Task { task: 7 }) },
    );
    let [Event::Read { result: reads::Loaded::Page { rows, next: None }, .. }] = events.as_ref() else {
        panic!("point: {events:?}")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].value, expected);
    let ended = Record::Tasks(Stored::Ended(Box::new(row)));
    let ended_ops = operations(
        &mut store,
        3,
        Box::new([
            Write::Erase(Key::Tasks(tasks::Key::Live(7))),
            Write::Erase(Key::Tasks(tasks::Key::LiveCold(7))),
            Write::Save(ended.clone()),
        ]),
    );
    assert_eq!(ended_ops.len(), 3);
    for key in [tasks::Key::Live(7), tasks::Key::LiveCold(7)] {
        assert!(ended_ops.iter().any(|op| match op {
            skein_kv::Op::Erase { key: found } => *found == raw(key),
            skein_kv::Op::Put { .. } => false,
        }));
    }
    assert!(match put_value(&ended_ops, tasks::Key::Ended(7)) {
        wire::Value::EndedTalk(_) => true,
        _ => false,
    });
    fixtures::plant(&mut store.stores, &ended_ops);
    assert_eq!(fixtures::get_kv(&mut store, &raw(tasks::Key::Live(7))), None);
    assert_eq!(fixtures::get_kv(&mut store, &raw(tasks::Key::LiveCold(7))), None);
    let bytes = fixtures::get_kv(&mut store, &raw(tasks::Key::Ended(7))).unwrap();
    assert_eq!(codec::decode(&ended.key(), &bytes, &store.codec_limits, &[]), Some(ended));
}

#[test]
fn a_hot_task_without_its_cold_part_is_a_decode_failure() {
    let mut store = fixtures::opened();
    let ops = operations(&mut store, 1, Box::new([hot(&live())]));
    fixtures::plant(&mut store.stores, &ops);
    let error =
        restore::page(&mut store, Restore::Jig(Box::new([RestoreRange::Tasks])), None, 8, u64::MAX).unwrap_err();
    assert_eq!(error.cause, restore::Cause::Decode(restore::DecodeFailure::Value));
    assert_eq!(error.row.unwrap().key, raw(tasks::Key::Live(7)));
    let events = fixtures::run(
        &mut store,
        Request::Get { owner: Token::new(9), key: Get::Jig(Key::Tasks(tasks::Key::Live(7))) },
    );
    assert!(match events.as_ref() {
        [Event::Corrupt { .. }] => true,
        _ => false,
    });
}

#[test]
fn generated_live_records_split_and_join_with_each_parts_own_model_names() {
    let names = fixtures::configured_model_names(&[b"z", b"a", b"m"]);
    let now = fixtures::configured_model_names(&[b"m", b"z", b"a"]);
    let mut rng = skein_lib::Rng::new(0x5701_0016);
    for _ in 0_u32..64 {
        let mut row = fixtures::task_record();
        row.number = rng.next_u64();
        row.spec.words = vec![b'w'; 4096].into();
        row.instructions = Some(vec![b'i'; usize::try_from(rng.below(100)).unwrap()].into());
        row.authority.models = tasks::Models::Named(1);
        for member in &mut row.recurring.as_mut().unwrap().template.batch {
            member.authority.models = tasks::Models::Named(2);
        }
        let stored = Stored::Live(Box::new(row.clone()));
        let Record::Tasks(cold) = fixtures::cold_task(&row) else { unreachable!() };
        let hot_bytes = codec::tasks::encode(&stored, &wire::CEILINGS, &names).unwrap();
        let cold_bytes = codec::tasks::encode(&cold, &wire::CEILINGS, &names).unwrap();
        assert_eq!(codec::tasks::decode_cold(row.number, &cold_bytes, &wire::CEILINGS, &names), Ok(cold.clone()));
        assert_eq!(cold.key(), tasks::Key::LiveCold(row.number));
        assert_eq!(codec::tasks::decode_live(&hot_bytes, &cold_bytes, &wire::CEILINGS, &names), Ok(stored.clone()));
        let mut expected = stored;
        let mut moved = [0; 64];
        moved[0] = 2;
        moved[1] = 4;
        moved[2] = 1;
        expected.remap_models(&moved);
        assert_eq!(codec::tasks::decode_live(&hot_bytes, &cold_bytes, &wire::CEILINGS, &now), Ok(expected));
        let hot = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(&hot_bytes)).unwrap();
        let cold = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(&cold_bytes)).unwrap();
        assert_eq!(cold.models().len(), 1);
        assert_eq!(hot.models().len(), 1, "nested recurring authorities stay hot");
        assert_eq!(hot.models().as_slice()[0].value(), b"a");
        assert_eq!(cold.models().as_slice()[0].value(), b"z");
        // EndedTalk keeps the whole payload and the same-width tag as LiveTalk.
        let whole = codec::tasks::encode(&Stored::Ended(Box::new(row)), &wire::CEILINGS, &names).unwrap();
        assert!(hot_bytes.len() < whole.len());
    }
}
