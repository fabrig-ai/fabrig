use super::fixtures;
use crate::{jig::key::Name, key};
#[test]
fn a_named_key_is_the_first_16_bytes_of_the_salted_digest() {
    let digest = skein_lib::sha256::digest(b"temper-tools\0chat-to-shop");
    assert_eq!(key::engine(&fixtures::name(b"chat-to-shop"))[..], digest[..16]);
}
/// client/tools.md, 7: names contain one to 64 bytes.
const NAME_MOST: usize = 64;
const NAMES: &[(&[u8], bool)] = &[
    (b"", false),
    (b"a", true),
    (b"a-z09", true),
    (b"A", false),
    (b"a_b", false),
    (b"a b", false),
    (b"a.b", false),
    (b"\xff", false),
    (&[b'a'; NAME_MOST], true),
    (&[b'a'; NAME_MOST + 1], false),
];
#[test]
fn a_key_name_is_1_to_64_bytes_of_lower_case_digits_and_hyphens() {
    for (bytes, accepted) in NAMES {
        assert_eq!(Name::new(bytes).is_some(), *accepted, "{bytes:?}");
    }
}
