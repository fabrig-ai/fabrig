//! Typed reads are ready for the worker's check-start caller (domain/forge/reds.md, 5.4).
use crate::worker::translate;
use skein_lib::Writer;
use temper_protocol_worker_wire::v1;
use temper_worker_domain::wire;

#[test]
fn reds_answers_validate_heads_and_consume_the_whole_payload() {
    assert_eq!(translate::ask(wire::Ask::Reds).as_ref(), &[0]);
    for length in [0_usize, 19, 20, 21, 31, 32, 33] {
        let mut writer = Writer::new(525_usize.checked_add(length).unwrap());
        writer.put(&[0]).unwrap(); // Answer::Reds.
        writer.put(&1_u32.to_be_bytes()).unwrap();
        writer.put(&512_u32.to_be_bytes()).unwrap();
        writer.put(&[b'k'; 512]).unwrap();
        writer.put(&u32::try_from(length).unwrap().to_be_bytes()).unwrap();
        let mut head = skein_lib::bytes::zeroed(length);
        head.fill(3);
        writer.put(&head).unwrap();
        let bytes = writer.finish();
        let decoded = translate::answer(&bytes, &v1::CEILINGS);
        if length == 20 || length == 32 {
            assert_eq!(
                decoded,
                Some(wire::Answer::Reds { reds: Box::new([wire::Red { key: Box::new([b'k'; 512]), head }]) })
            );
            let trailing = skein_lib::bytes::concat(&[&bytes, &[0]]);
            assert_eq!(translate::answer(&trailing, &v1::CEILINGS), None);
            let short = bytes.get(..bytes.len().checked_sub(1).unwrap()).unwrap();
            assert_eq!(translate::answer(short, &v1::CEILINGS), None);
        } else {
            assert_eq!(decoded, None, "head length {length}");
        }
    }
}

#[test]
fn landed_asks_and_answers_keep_their_wire_forms() {
    let head = [37; 32];
    let bytes = translate::ask(wire::Ask::Landed { head });
    let expected = v1::Ask::Landed(v1::Head::new(&v1::CEILINGS, v1::HeadParts { head }).unwrap());
    assert_eq!(v1::Ask::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)), Ok(expected));
    for (encoded, typed) in [(v1::Answer::Landed, wire::Answer::Landed), (v1::Answer::Unlanded, wire::Answer::Unlanded)]
    {
        let bytes = temper_protocol_worker_wire::answer(&encoded);
        assert_eq!(translate::answer(&bytes, &v1::CEILINGS), Some(typed));
        let trailing = skein_lib::bytes::concat(&[&bytes, &[0]]);
        assert_eq!(translate::answer(&trailing, &v1::CEILINGS), None);
        assert_eq!(translate::answer(&[], &v1::CEILINGS), None);
    }
}
