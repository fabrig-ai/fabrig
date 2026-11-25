//! Opaque cursor headers (jig/docs/design/protocol/client/README.md, 4).

use super::{CURSOR_VERSION, c, decode, encode, encode_tree};
use jig_client_protocol_wire::CEILINGS;
use jig_domain_core_vocabulary::{documents, reads};
use skein_lib::{Reader, Writer};

const TASK: u64 = 7;
const TREE_ROWS: u32 = 3;
const OWNER: reads::Cursor = reads::Cursor::Store { scope: reads::Scope::Task { task: TASK }, after: None };
const LAYOUT_BYTES: u32 = 32; // The fixed layout prefix in Held.
const VERSION_BYTES: usize = size_of::<u16>();

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
