mod layout;
mod read;

use crate::flow::{self, Direction, Node};
use crate::look;

#[test]
fn a_flows_data_reads_back() {
    let cases: &[(Direction, &[Node<'_>], &[u8])] = &[
        (Direction::LeftToRight, &[Node { name: b"a", caption: None, strong: false }], b"\0\0\x01\0a"),
        (Direction::TopToBottom, &[Node { name: b"b", caption: None, strong: true }], b"\x01\x02\x01\0b"),
        (
            Direction::LeftToRight,
            &[
                Node { name: b"a", caption: Some(b"c"), strong: false },
                Node { name: b"b", caption: Some(b"d"), strong: true },
                Node { name: b"\xff\0", caption: None, strong: false },
            ],
            b"\0\x01\x01\0a\x01\0c\x03\x01\0b\x01\0d\0\x02\0\xff\0",
        ),
    ];
    for (direction, nodes, expected) in cases {
        let bound = u32::try_from(expected.len()).expect("fixture fits u32");
        let data = flow::encode(*direction, nodes, bound).expect("data fits exactly");
        assert_eq!(data.as_slice(), *expected);
        let (found_direction, found_nodes) = flow::decode(data.as_slice()).expect("encoded flow decodes");
        assert_eq!(found_direction, *direction);
        assert!(found_nodes.clone().eq(nodes.iter().copied()));
        assert!(found_nodes.eq(nodes.iter().copied()));
    }
}

#[test]
fn data_encode_cannot_write_is_refused() {
    let cases: &[(&[u8], &str)] = &[
        (b"", "no direction"),
        (b"\0", "no node"),
        (b"\x02\0\x01\0a", "unknown direction"),
        (b"\0\x04\x01\0a", "unknown flag"),
        (b"\0\x80\x01\0a", "high flag"),
        (b"\0\0", "no name length"),
        (b"\0\0\x01", "short name length"),
        (b"\0\0\0\0", "empty name"),
        (b"\0\0\x02\0a", "name past end"),
        (b"\0\0\0\x01a", "little-endian length past end"),
        (b"\0\x01\x01\0a", "caption flag without caption"),
        (b"\0\x01\x01\0a\x01", "short caption length"),
        (b"\0\x01\x01\0a\0\0", "empty caption"),
        (b"\0\x01\x01\0a\x02\0b", "caption past end"),
        (b"\0\0\x01\0a\0", "incomplete second node"),
        (b"\0\0\x01\0a\x04\x01\0b", "invalid second node"),
    ];
    for (data, reason) in cases {
        assert!(flow::decode(data).is_none(), "{reason}");
    }
}

#[test]
fn scene_bounds_refuse_overflow() {
    // Four marks and 25 arrow bytes per node (ui.md, 11.5).
    for (nodes, expected) in [(0, Some(0)), (4, Some(16)), (1_073_741_823, Some(4_294_967_292)), (1_073_741_824, None)]
    {
        assert_eq!(flow::marks(nodes), expected, "marks for {nodes}");
    }
    for (nodes, data, expected) in [
        (0, 0, Some(0)),
        (4, 100, Some(200)),
        (171_798_691, 20, Some(u32::MAX)),
        (171_798_691, 21, None),
        (171_798_692, 0, None),
    ] {
        assert_eq!(flow::scene_bytes(nodes, data), expected, "bytes for {nodes}, {data}");
    }
}

#[test]
fn encoding_refuses_data_outside_its_bounds() {
    // Text's length is a u16; one node costs direction, flags, length, then text.
    const MAX_TEXT: usize = 65_535;
    const PAST_TEXT: usize = 65_536;
    const MAX_NAME_DATA: u32 = 65_539;
    const MAX_CAPTION_DATA: u32 = 65_542;
    let max = skein_lib::bytes::zeroed(MAX_TEXT);
    let past = skein_lib::bytes::zeroed(PAST_TEXT);
    let cases: &[(&str, &[Node<'_>], u32, bool)] = &[
        ("no nodes", &[], u32::MAX, false),
        ("empty name", &[Node { name: b"", caption: None, strong: false }], u32::MAX, false),
        ("empty caption", &[Node { name: b"a", caption: Some(b""), strong: false }], u32::MAX, false),
        ("exact bound", &[Node { name: b"a", caption: None, strong: false }], 5, true),
        ("past bound", &[Node { name: b"a", caption: None, strong: false }], 4, false),
        ("no room", &[Node { name: b"a", caption: None, strong: false }], 0, false),
        ("max name", &[Node { name: &max, caption: None, strong: false }], MAX_NAME_DATA, true),
        ("long name", &[Node { name: &past, caption: None, strong: false }], u32::MAX, false),
        ("max caption", &[Node { name: b"a", caption: Some(&max), strong: false }], MAX_CAPTION_DATA, true),
        ("long caption", &[Node { name: b"a", caption: Some(&past), strong: false }], u32::MAX, false),
    ];
    for (reason, nodes, bound, accepted) in cases {
        let data = flow::encode(Direction::TopToBottom, nodes, *bound);
        assert_eq!(data.is_some(), *accepted, "{reason}");
        if let Some(data) = data {
            let (direction, decoded) = flow::decode(data.as_slice()).expect("bounded encoding decodes");
            assert_eq!(direction, Direction::TopToBottom, "{reason}");
            assert!(decoded.eq(nodes.iter().copied()), "{reason}");
        }
    }
}

#[test]
fn the_flows_names_belong_to_their_tokens() {
    assert_eq!(look::check(&[flow::NAMES]), None);
    assert_eq!(flow::OWNER.number, u32::MAX);
    assert_eq!(flow::KIND.owner, flow::OWNER);
    assert_eq!(flow::KIND.kind, 0);
    for (style, expected) in [
        (flow::BOX, b"skein-ui-flow-box".as_slice()),
        (flow::NAME, b"skein-ui-flow-name"),
        (flow::CAPTION, b"skein-ui-flow-caption"),
        (flow::ARROW, b"skein-ui-flow-arrow"),
        (flow::STRONG, b"skein-ui-flow-strong"),
    ] {
        assert_eq!(flow::NAMES.style(style), Some(expected));
    }
}
