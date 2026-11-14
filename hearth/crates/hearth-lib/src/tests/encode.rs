use super::fixtures::{encode, hex, text};
use crate::encode::{DEPTH, Digest, Encoder, Item};

#[test]
fn a_list_of_texts_encodes_as_store_md_writes_it() {
    let value = [Item::List(2), text(b"hearth/1"), text(b"empty")];
    let expected = hex("06 0000000000000002 04 0000000000000008 6865617274682f31 04 0000000000000005 656d707479");
    assert_eq!(
        encode(&value),
        expected,
        "hearth/docs/design/domain/store/README.md, 3.2: encode(list [bytes hearth/1, bytes empty])"
    );
}

#[test]
fn a_map_key_is_encoded_as_a_whole_bytes_value() {
    let value = [Item::Map(1), text(b"a"), Item::Uint(1)];
    assert_eq!(
        encode(&value),
        hex("07 0000000000000001 04 0000000000000001 61 03 0000000000000001"),
        "hearth/docs/design/domain/store/README.md, 3.2: a key is bytes"
    );
}

const SCALARS: &[(Item, &str)] = &[
    (Item::None, "00"),
    (Item::False, "01"),
    (Item::True, "02"),
    (Item::Uint(0o755), "03 00000000000001ed"),
    (Item::Digest(Digest([1; 32])), "05 0101010101010101010101010101010101010101010101010101010101010101"),
];

#[test]
fn each_scalar_kind_starts_with_a_byte_of_its_own() {
    for (item, expected) in SCALARS {
        assert_eq!(encode(core::slice::from_ref(item)), hex(expected), "scalar {item:?}");
    }
}

#[test]
fn map_keys_order_by_their_bytes_not_their_encodings() {
    let value = [Item::Map(2), text(b"ab"), Item::None, text(b"b"), Item::None];
    assert_eq!(
        encode(&value),
        hex("07 0000000000000002 04 0000000000000002 6162 00 04 0000000000000001 62 00"),
        "keys order by bytes"
    );
}

#[test]
fn a_record_without_its_optional_field_holds_an_empty_map() {
    let value = [Item::Map(2), text(b"adm"), Item::Map(1), text(b"gid"), Item::Uint(4), text(b"staff"), Item::Map(0)];
    let expected = hex(
        "07 0000000000000002 04 0000000000000003 61646d 07 0000000000000001 04 0000000000000003 676964 03 0000000000000004 04 0000000000000005 7374616666 07 0000000000000000",
    );
    assert_eq!(encode(&value), expected, "an absent optional field is left out");
}

#[test]
fn a_value_sixty_four_containers_deep_encodes() {
    let mut encoder = Encoder::measuring();
    for _ in 0..DEPTH {
        encoder.list(1);
    }
    encoder.none();
    assert_eq!(encoder.measured(), 577, "sixty-four list heads and none");
}
