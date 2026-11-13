//! Cursor versions follow their header and generated codec (jig/docs/design/protocol/README.md, 7).

use super::{CURSOR_VERSION, decode, encode, encode_tree};
use jig_domain_core_vocabulary::{documents, reads};
use skein_lib::{Reader, Writer};

#[test]
#[expect(clippy::disallowed_macros, reason = "the committed schema is a version test input")]
fn cursor_version_equals_the_header_and_the_codec() {
    let schema =
        skein_codec::schema::parse(include_bytes!("../../../schema/cursor-v1.schema")).expect("cursor family parses");
    assert_eq!(schema.spelled(schema.family()), b"jig-engine-cursor");
    assert_eq!(CURSOR_VERSION, schema.version());

    let cursor = reads::Cursor::Store { scope: reads::Scope::Task { task: 7 }, after: None };
    let bytes = encode(&cursor, 1024).expect("owner cursor");
    assert_eq!(Reader::new(&bytes).u16(), Some(CURSOR_VERSION), "generated codec writes the header's version");
    assert_eq!(decode(&bytes), Some(cursor));
    let tree = encode_tree(&documents::Cursor::Number { number: 7 }, 7, 3, 1024).expect("tree cursor");
    assert_eq!(Reader::new(&tree).u16(), Some(CURSOR_VERSION));

    let mut wrong = Writer::new(bytes.len());
    wrong.put(&CURSOR_VERSION.checked_add(1).expect("test version").to_be_bytes()).expect("version fits");
    let mut reader = Reader::new(&bytes);
    reader.u16().expect("version prefix");
    wrong.put(reader.bytes(reader.remaining()).expect("cursor payload")).expect("cursor payload fits");
    assert_eq!(decode(&wrong.finish()), None, "another cursor version is refused");
}
