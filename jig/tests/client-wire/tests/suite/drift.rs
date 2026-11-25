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

#[test]
fn opened_keeps_its_tag_and_layout_head_bytes() {
    use jig_client_protocol_wire as w;
    use skein_lib::{Duration, List, Reader};
    // Tag, hello (two served owners and a heartbeat), and two layouts, ahead of me.
    #[rustfmt::skip]
    let head: &[u8] = &[
        4,
        165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165, 165,
        0, 0, 0, 2,
        0, 0, 0, 0, 43, 0, 0, 0, 43,
        1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1,
        0, 0, 0, 0, 0, 0, 0, 5,
        0, 0, 0, 2,
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
           1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    ];
    let me = w::Owned { owner: w::Owner::Jig, version: 43, record: Box::from([0xca]) };
    let bytes = [head, &me.encode(&w::CEILINGS).expect("fixture me envelope")].concat();
    let event = w::StreamEvent::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("fixed Opened");
    let family = w::Owner::Family(w::Family { number: 1 });
    assert_eq!(
        event,
        w::StreamEvent::Opened(w::Opened {
            hello: w::Hello {
                stream: [165; 16],
                served: List::from_boxed(Box::new([
                    w::Served { owner: w::Owner::Jig, first: 43, last: 43 },
                    w::Served { owner: family.clone(), first: 1, last: 1 },
                ])),
                heartbeat: Duration::from_nanos(5),
            },
            layouts: List::from_boxed(Box::new([
                w::Layout { owner: w::Owner::Jig, layout: [1; 32] },
                w::Layout { owner: family, layout: [2; 32] },
            ])),
            me,
        })
    );
    assert_eq!(event.encode(&w::CEILINGS).expect("Opened fits").as_ref(), bytes);
}
