//! A lowering is diagnosed only after the entire row reads at stock.
use super::fixtures;
use crate::{Undecodable, bounds, keys, records};
use jig_protocol_store::codec::Lowered;
use temper_engine_domain_forge as forge;

fn release() -> forge::Stored {
    forge::Stored::Release(forge::ReleaseRow {
        task: 1,
        root: 2,
        ending: forge::ReleaseEnding::Cancelled,
        close_pull: None,
        comment: Some(Box::new([b'x'; 300])),
        pending: None,
        failed: false,
        effects_settled: true,
        releasing: true,
    })
}

#[test]
fn lowered_forge_bytes_name_the_limit_and_held_length() {
    let stock = bounds(&fixtures::LIMITS, 512).unwrap();
    let mut forge = fixtures::LIMITS;
    forge.client.op_bytes = 200;
    let lowered = bounds(&forge, 512).unwrap().with_stock(&stock);
    let row = release();
    let key = keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
    let value = records::encode(&row);
    let error = records::row(&key, &value, &lowered).unwrap_err();
    assert_eq!(error, Undecodable::Lowered(Lowered { name: b"forge.client.op_bytes", configured: 200, held: 300 }));
    assert!(!alloc::fmt::format(format_args!("{error:?}")).contains("xxxxxxxx"));
    assert_eq!(records::row(&key, &value, &stock), Ok(row));
}

#[test]
fn lowered_forge_lists_keep_the_held_item_count() {
    let stock = bounds(&fixtures::LIMITS, 512).unwrap();
    let mut forge = fixtures::LIMITS;
    forge.follow_merge = 1;
    let lowered = bounds(&forge, 512).unwrap().with_stock(&stock);
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
    row.follows = Some(forge::ChangeFollows { follow: 3, commits: Box::new([[1; 32], [2; 32]]) });
    let row = fixtures::change_cold(row);
    let key = keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
    let value = records::encode(&row);
    assert_eq!(
        records::row(&key, &value, &lowered),
        Err(Undecodable::Lowered(Lowered { name: b"forge.follow_merge", configured: 1, held: 2 }))
    );
    assert_eq!(records::row(&key, &value, &stock), Ok(row));
}

#[test]
fn malformed_forge_rows_are_not_lowerings_even_with_an_oversized_field() {
    let stock = bounds(&fixtures::LIMITS, 512).unwrap();
    let mut forge = fixtures::LIMITS;
    forge.client.op_bytes = 200;
    let lowered = bounds(&forge, 512).unwrap().with_stock(&stock);
    let row = release();
    let key = keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
    let encoded = records::encode(&row);
    let trailing = skein_lib::bytes::concat(&[&encoded, &[255]]);
    for value in [Box::from(b"undecodable private value".as_slice()), trailing] {
        assert_eq!(records::row(&key, &value, &lowered), Err(Undecodable::Malformed));
    }
    let wrong = keys::encode(&forge::Key::Release(99), u32::MAX).unwrap();
    assert_eq!(records::row(&wrong, &encoded, &lowered), Err(Undecodable::Malformed));
}
