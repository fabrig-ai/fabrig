//! The client records' drift (skein's data/codec/golden.md, 1).
use jig_client_protocol_wire::Record as _;
#[path = "../../../codec-families.rs"]
mod families;

#[test]
fn codec_families_match_their_schemas() {
    families::check();
}

#[test]
fn the_hello_alone_keeps_version_27_bytes() {
    use jig_client_protocol_wire as w;
    use skein_lib::{Reader, Writer};
    let full: &[u8] = &[
        0, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 0, 0, 0, 1, 0, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    ];
    for (expected, fixed_value) in [
        (full, super::client_values::fixed_value("enum_stream_event_hello_full.bin")),
        ([0_u8; 29].as_slice(), super::client_values::fixed_value("enum_stream_event_hello_smallest.bin")),
    ] {
        assert_eq!(fixed_value, expected);
        let value = w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(expected)).expect("version 27 hello");
        assert!(matches!(value, w::StreamEvent::Hello(_)));
        let mut writer = Writer::new(
            value
                .measure(&jig_client_protocol_wire::CEILINGS)
                .expect("bounded fixture")
                .try_into()
                .expect("short hello"),
        );
        value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("hello fits");
        assert_eq!(writer.finish().as_ref(), expected);
    }
}
