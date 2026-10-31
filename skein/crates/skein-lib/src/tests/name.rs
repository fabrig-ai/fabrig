//! Names at the length and grammar boundaries (skein/docs/design/lib.md, 4).

use crate::{Grammar, Name};

#[test]
fn names_refuse_empty_and_over_capacity_input() {
    assert!(Name::<3>::new(b"", Grammar::LowercaseDigitsHyphen).is_none());
    assert!(Name::<3>::new(b"abcd", Grammar::LowercaseDigitsHyphen).is_none());
    assert!(Name::<0>::new(b"", Grammar::PrintableAscii).is_none());
    assert!(Name::<0>::new(b"a", Grammar::PrintableAscii).is_none());
    let name = Name::<3>::new(b"a-1", Grammar::LowercaseDigitsHyphen).expect("name at capacity");
    assert_eq!(name.as_bytes(), b"a-1");
    let longest = [b'z'; 255];
    assert_eq!(Name::<255>::new(&longest, Grammar::LowercaseDigitsHyphen).expect("u8 length").as_bytes(), longest);
    assert!(Name::<255>::new(&[b'z'; 256], Grammar::LowercaseDigitsHyphen).is_none());
}

#[test]
fn source_spelling_admits_only_lowercase_digits_and_hyphens() {
    for bytes in [b"a".as_slice(), b"source-123", b"-", b"0123456789abcdefghijklmnopqrstuvwxyz-"] {
        let name = Name::<40>::new(bytes, Grammar::LowercaseDigitsHyphen).expect("source spelling");
        assert_eq!(name.as_bytes(), bytes);
    }
    for bytes in [b"A".as_slice(), b"a_b", b"../a", b"two words", b"a\x80", b"a\0"] {
        assert!(Name::<40>::new(bytes, Grammar::LowercaseDigitsHyphen).is_none(), "{bytes:?}");
    }
}

#[test]
fn method_spelling_admits_printable_ascii_without_spaces() {
    for byte in 0_u8..=255 {
        let bytes = [byte];
        let name = Name::<1>::new(&bytes, Grammar::PrintableAscii);
        assert_eq!(name.is_some(), (b'!'..=b'~').contains(&byte), "{byte}");
        if let Some(name) = name {
            assert_eq!(name.as_bytes(), bytes);
        }
    }
    assert_eq!(
        Name::<32>::new(b"method.example", Grammar::PrintableAscii).expect("method").as_bytes(),
        b"method.example"
    );
}

#[test]
fn stored_names_check_only_the_carrier_length() {
    for bytes in [b"".as_slice(), b"A\0\x80", b"a-1"] {
        assert_eq!(Name::<3>::from_checked(bytes).expect("fits carrier").as_bytes(), bytes);
    }
    assert!(Name::<3>::from_checked(b"abcd").is_none());
    assert_eq!(Name::<0>::from_checked(b"").expect("empty carrier").as_bytes(), b"");
    let checked = Name::<3>::new(b"a-1", Grammar::LowercaseDigitsHyphen).expect("checked name");
    assert_eq!(Name::<3>::from_checked(b"a-1"), Some(checked));
}
