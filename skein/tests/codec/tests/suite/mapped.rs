//! The maps fixture crosses the domain, records and wire (data/codec/maps.md, 8).

use skein_codec::{Reason, Record};
use skein_codec_tests::{mapped_domain as d, mapped_records as r, mapped_translation as maps};
use skein_lib::{Duration, List, Reader, Wall};

fn values() -> [d::Packet; 2] {
    let mut references = List::with_capacity(2);
    references.push(3).expect("room");
    references.push(5).expect("room");
    let mut events = List::with_capacity(8);
    for event in [
        d::Event::Idle,
        d::Event::Inline { total: d::Positive(2) },
        d::Event::Alternate { other: d::Positive(3) },
        d::Event::Item(Box::new(d::Item { number: 4 })),
        d::Event::UnboxedItem(d::Item { number: 5 }),
        d::Event::Label("payload".into()),
        d::Event::Renamed,
        d::Event::Checked(d::Checked(7)),
    ] {
        events.push(event).expect("room for every variant");
    }
    let full = d::Packet {
        changed: Wall::from_nanos(123),
        caption: "é".into(),
        maybe: Some(Wall::from_nanos(456)),
        times: vec![Wall::from_nanos(1), Wall::from_nanos(2)].into_boxed_slice(),
        references,
        labels: vec!["list".into(), "".into()].into_boxed_slice(),
        items: vec![d::Item { number: 42 }].into_boxed_slice(),
        counts: List::from_boxed(vec![1, 2].into_boxed_slice()),
        padding: Box::new([d::Empty {}]),
        primary: Box::new(d::Item { number: 21 }),
        optional_items: Some(vec![d::Item { number: 9 }].into_boxed_slice()),
        optional_primary: Some(Box::new(d::Item { number: 8 })),
        events,
        checked: d::Checked(7),
        choice: d::HandChoice::Some,
        enabled: true,
        nonce: [1, 2],
        age: Duration::from_nanos(17),
    };
    let empty = d::Packet {
        changed: Wall::EPOCH,
        caption: "".into(),
        maybe: None,
        times: Box::new([]),
        references: List::with_capacity(0),
        labels: Box::new([]),
        items: Box::new([]),
        counts: List::with_capacity(0),
        padding: Box::new([]),
        primary: Box::new(d::Item { number: 0 }),
        optional_items: None,
        optional_primary: None,
        events: List::with_capacity(0),
        checked: d::Checked(7),
        choice: d::HandChoice::None,
        enabled: false,
        nonce: [0, 0],
        age: Duration::ZERO,
    };
    [full, empty]
}

#[test]
fn mapped_values_round_trip_through_records_and_wire() {
    let context = d::Context { permitted: 7 };
    for value in values() {
        let record = maps::packet_to_record(&value, &context).expect("to record");
        assert_eq!(record.when, value.changed.as_nanos());
        assert_eq!(record.items.len(), u32::try_from(value.items.len()).expect("fixture count"));
        let bytes = record.encode(&r::CEILINGS).expect("encode");
        let decoded = r::Packet::decode(&r::CEILINGS, &mut Reader::new(&bytes)).expect("decode");
        assert_eq!(maps::packet_to_domain(decoded, &context), Ok(value.clone()));
        let bytes = d::encode(&value, &context).expect("owner root encode");
        assert_eq!(d::decode(&bytes, &context), Ok(value));
    }
    for value in [d::LongChoice::Slot00, d::LongChoice::Slot98] {
        let record = maps::long_choice_to_record(&value);
        let bytes = record.encode(&r::CEILINGS).expect("long enum encode");
        let decoded = r::LongChoice::decode(&r::CEILINGS, &mut Reader::new(&bytes)).expect("long enum decode");
        assert_eq!(maps::long_choice_to_domain(decoded), value);
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn mapped_slice_count_past_u32_refuses_before_iteration() {
    let context = d::Context { permitted: 7 };
    let mut value = values().into_iter().next().expect("full fixture");
    // Zero-sized items exercise the count without allocating its many bytes.
    value.padding = Box::new([d::Empty {}; usize::MAX]);
    assert_eq!(maps::packet_to_record(&value, &context), Err(r::paths::PACKET_PADDING.problem(Reason::Bound)));
}

#[test]
fn mapped_converter_refusals_keep_the_record_field_path() {
    let context = d::Context { permitted: 7 };
    let value = values().into_iter().next().expect("full fixture");
    let mut record = maps::packet_to_record(&value, &context).expect("record");
    record.caption = Box::new([0xff]);
    assert_eq!(maps::packet_to_domain(record, &context), Err(r::paths::PACKET_CAPTION.problem(Reason::Utf8)));
    assert_eq!(
        maps::event_to_domain(r::Event::Inline(r::Shared { count: 0 }), &context),
        Err(r::paths::SHARED_COUNT.problem(Reason::Bound))
    );
    assert_eq!(
        maps::event_to_record(&d::Event::Alternate { other: d::Positive(0) }, &context),
        Err(r::paths::SHARED_COUNT.problem(Reason::Bound))
    );
    assert_eq!(
        maps::event_to_domain(r::Event::Label(r::Label { text: Box::new([0xff]) }), &context),
        Err(r::paths::LABEL_TEXT.problem(Reason::Utf8))
    );
}

#[test]
fn by_hand_refusals_reach_the_owner_root_unchanged_in_both_directions() {
    let permitted = d::Context { permitted: 7 };
    let refused = d::Context { permitted: 8 };
    let value = values().into_iter().next().expect("full fixture");
    let problem = r::paths::CHECKED_CODE.problem(Reason::Tag);
    assert_eq!(d::encode(&value, &refused), Err(problem));
    let bytes = d::encode(&value, &permitted).expect("permitted root");
    assert_eq!(d::decode(&bytes, &refused), Err(problem));
    assert_eq!(maps::event_to_record(&d::Event::Checked(d::Checked(7)), &refused), Err(problem));
    assert_eq!(maps::event_to_domain(r::Event::Checked(r::Checked { code: 7 }), &refused), Err(problem));
}
