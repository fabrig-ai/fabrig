//! Opaque cursor headers (jig/docs/design/protocol/client/README.md, 4).

use super::{CURSOR_VERSION, c, decode, decode_window, encode, encode_tree};
use crate::translate::encode_window;
use c::Record as _;
use jig_client_protocol_wire::CEILINGS;
use jig_domain_core_vocabulary::{documents, reads};
use skein_lib::{Reader, Wall, Writer};

const TASK: u64 = 7;
const TREE_ROWS: u32 = 3;
const OWNER: reads::Cursor = reads::Cursor::Store { scope: reads::Scope::Task { task: TASK }, after: None };
const LAYOUT_BYTES: u32 = 32; // The fixed layout prefix in Held.
const VERSION_BYTES: usize = size_of::<u16>();
const NUMBER: documents::Cursor = documents::Cursor::Number { number: TASK };

#[test]
fn cursor_headers_name_the_schema_version_and_layout() {
    let schema = &skein_codegen::values(crate::SCHEMA).schema;
    assert_eq!(schema.family.as_bytes(), b"jig-engine-cursor");
    assert_eq!(CURSOR_VERSION, schema.version);

    let owner = encode(&OWNER, CEILINGS.cursor_value).expect("owner cursor");
    let tree = encode_tree(&documents::Cursor::Number { number: TASK }, TASK, TREE_ROWS, CEILINGS.cursor_value)
        .expect("tree cursor");
    for bytes in [&owner, &tree] {
        let mut reader = Reader::new(bytes);
        assert_eq!(reader.u16(), Some(CURSOR_VERSION));
        assert_eq!(reader.bytes(LAYOUT_BYTES), Some(c::LAYOUT.as_slice()));
    }
    assert_eq!(decode(&owner), Some(OWNER));
}

#[test]
fn cursors_of_another_version_or_layout_are_refused() {
    let bytes = encode(&OWNER, CEILINGS.cursor_value).expect("owner cursor");
    let other_version = CURSOR_VERSION.checked_add(1).expect("another version").to_be_bytes();
    let other_layout = [0; c::LAYOUT.len()];
    for (label, offset, replacement) in
        [("version", 0, other_version.as_slice()), ("layout", VERSION_BYTES, &other_layout)]
    {
        let end = offset.checked_add(replacement.len()).expect("prefix fits");
        let mut foreign = Writer::new(bytes.len());
        foreign.put(bytes.get(..offset).expect("prefix start")).expect("prefix fits");
        foreign.put(replacement).expect("replacement fits");
        foreign.put(bytes.get(end..).expect("prefix end")).expect("suffix fits");
        assert_eq!(decode(&foreign.finish()), None, "another {label}");
    }
}

#[test]
fn window_cursors_round_trip_at_their_exact_bound() {
    let inbox = documents::Cursor::Inbox {
        task: TASK,
        object: documents::InboxObject::Proposal { proposer: jig_domain_people::Proposer::Party(11), number: 13 },
        at: Some(Wall::from_nanos(17)),
    };
    for after in [NUMBER, inbox] {
        let bytes = encode_window(&after, CEILINGS.cursor_value).expect("window cursor");
        assert_eq!(decode_window(&bytes), Some(after));
        let measured = u32::try_from(bytes.len()).expect("bounded cursor");
        assert_eq!(encode_window(&after, measured), Some(bytes));
        assert_eq!(encode_window(&after, measured.checked_sub(1).expect("nonempty cursor")), None);
    }
}

#[test]
fn window_cursors_require_the_current_head_and_the_complete_window_form() {
    let limit = CEILINGS.cursor_value;
    let bytes = encode_window(&NUMBER, limit).expect("window cursor");
    assert_eq!(decode(&bytes), None, "a window is not an owner cursor");

    let mut foreign_layout = bytes.clone();
    *foreign_layout.get_mut(VERSION_BYTES).expect("layout byte") ^= 1;
    let mut foreign_version = bytes.clone();
    *foreign_version.first_mut().expect("version byte") ^= 1;
    let bare = c::DocumentCursor::Number(c::Number { value: TASK });
    for (label, rejected) in [
        ("bare position", bare.encode(&c::CEILINGS).expect("bare position")),
        ("foreign layout", foreign_layout),
        ("foreign version", foreign_version),
        ("tree form", encode_tree(&NUMBER, TASK, TREE_ROWS, limit).expect("tree cursor")),
        ("owner form", encode(&OWNER, limit).expect("owner cursor")),
        ("trailing byte", skein_lib::bytes::concat(&[&bytes, &[0]])),
    ] {
        assert_eq!(decode_window(&rejected), None, "{label}");
    }
    for cut in 0..bytes.len() {
        assert_eq!(decode_window(bytes.get(..cut).expect("proper prefix")), None, "cut at {cut}");
    }
}
