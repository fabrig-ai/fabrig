use super::*;
fn samples() -> Vec<(Vec<u8>, What)> {
    use Shape::{Bearer, Code, Jwt, Key, RefreshToken};
    [
        (["s", "k", "-", "Ab0123456789_-cdefGHI"].concat(), What::Shape(Key)),
        (["ey", "JabcDEF.Abcdef12.Zyxwvu98"].concat(), What::Shape(Jwt)),
        (["Bear", "er ", "Ab0123456789_+/-="].concat(), What::Shape(Bearer)),
        (["co", "de=", "Ab0123%+"].concat(), What::Shape(Code)),
        (["refresh", "_token", "\":\"", "Ab0123%+"].concat(), What::Shape(RefreshToken)),
        (["refresh", "_token", "=", "Ab0123%+"].concat(), What::Shape(RefreshToken)),
        (["private", "<&\"value>"].concat(), What::Original),
    ]
    .into_iter()
    .map(|(value, what)| (value.into_bytes(), what))
    .collect()
}
fn originals() -> Originals {
    Originals::new(Vec::new(), &[["private", "<&\"value>"].concat().into_bytes().into()])
}

#[test]
fn every_length_prefix_before_a_key() {
    let originals = originals();
    let (key, what) = samples().remove(0);
    let mut failures = 0;
    for byte in 0..=u8::MAX {
        if bytes(&[&[0, 0, 0, byte], key.as_slice()].concat(), &originals) != Some(what) {
            eprintln!("missing key after prefix {byte}");
            failures += 1;
        }
    }
    assert_eq!(failures, 0);
}
#[test]
fn a_51_byte_keys_own_prefix() {
    let key = [["s", "k", "-"].concat().as_bytes(), "A".repeat(48).as_bytes()].concat();
    assert_eq!(bytes(&[b"\0\0\0\x33".as_slice(), &key].concat(), &originals()), Some(What::Shape(Shape::Key)));
}
#[test]
fn these_pass() {
    let originals = originals();
    let mut values = vec![
        "task-pages-and-more-than-twenty-bytes".to_owned(),
        "temper-task-pages-a-long-name".into(),
        "risk-a-long-hyphenated-word-here".into(),
        "code_challenge=abcdefghijkl".into(),
        "status_code=200".into(),
        ["Bear", "er tokens"].concat(),
        ["s", "k", "- short"].concat(),
        ["ey", "J"].concat(),
        ["refresh", "_token is"].concat(),
    ];
    values.extend([
        ["s", "k", "-", &"x".repeat(20)].concat(),
        ["Bear", "er ", &"x".repeat(16)].concat(),
        ["co", "de=", &"x".repeat(8)].concat(),
        ["refresh", "_token=", &"x".repeat(8)].concat(),
        ["refresh", "_token", "\":\"", &"x".repeat(8)].concat(),
        ["xxxxxxxx", ".", "xxxxxxxx", ".", "xxxxxxxx"].concat(),
        "x".repeat(40),
    ]);
    for (row, value) in values.iter().enumerate() {
        assert!(bytes(value.as_bytes(), &originals).is_none(), "safe row {row}");
    }
}
#[test]
fn these_are_found_at_their_edges() {
    let originals = originals();
    let cases = [
        (["s", "k", "-"].concat(), 20, Shape::Key),
        (["Bear", "er "].concat(), 16, Shape::Bearer),
        (["co", "de="].concat(), 8, Shape::Code),
        (["error_co", "de="].concat(), 8, Shape::Code),
        (["device_co", "de="].concat(), 8, Shape::Code),
        (["refresh", "_token="].concat(), 8, Shape::RefreshToken),
    ];
    for (prefix, bound, shape) in cases {
        assert_eq!(bytes(format!("{prefix}{}", "A".repeat(bound)).as_bytes(), &originals), Some(What::Shape(shape)));
        assert!(bytes(format!("{prefix}{}", "A".repeat(bound - 1)).as_bytes(), &originals).is_none());
    }
    for mask in 0..64 {
        let prefix: String = "bearer"
            .chars()
            .enumerate()
            .map(|(i, c)| if mask & (1 << i) == 0 { c } else { c.to_ascii_uppercase() })
            .collect();
        assert_eq!(
            bytes(format!("{prefix} {}", "A".repeat(16)).as_bytes(), &originals),
            Some(What::Shape(Shape::Bearer))
        );
    }
    for lengths in [[8, 8, 8], [7, 8, 8], [8, 7, 8], [8, 8, 7]] {
        let text = ["ey", "J", &"A".repeat(lengths[0] - 3), ".", &"A".repeat(lengths[1]), ".", &"A".repeat(lengths[2])]
            .concat();
        assert_eq!(bytes(text.as_bytes(), &originals).is_some(), lengths == [8, 8, 8]);
    }
}
#[test]
fn original_forms_and_digest_halves_are_found() {
    let originals = Originals::new(Vec::new(), &[[" \tprivate", "<&\"\nvalue>\r "].concat().into_bytes().into()]);
    for original in originals.sought() {
        for form in original.forms() {
            assert_eq!(bytes(form, &originals), Some(What::Original));
        }
        for (digest, placeholder) in original.digests() {
            assert_eq!(bytes(digest, &originals), Some(What::Original));
            assert!(bytes(placeholder, &originals).is_none());
        }
    }
}
#[test]
fn refresh_separators_and_code_boundaries_hold() {
    let originals = originals();
    for separators in ["", "=", "\":\"", "\\\": "] {
        let text = ["refresh", "_token", separators, "Abcdef12"].concat();
        assert_eq!(bytes(text.as_bytes(), &originals), Some(What::Shape(Shape::RefreshToken)));
    }
    let text = ["refresh", "_token", "     ", "Abcdef12"].concat();
    assert!(bytes(text.as_bytes(), &originals).is_none());
    for prefix in ["a", "Z", "0", "9"] {
        assert!(bytes([prefix, "co", "de=Abcdef12"].concat().as_bytes(), &originals).is_none());
    }
}

#[test]
fn candidate_originals_and_shapes_are_scanned_separately() {
    use crate::Kind;
    use skein_lib::sha256::digest;

    let value = [b"a/ & ".as_slice(), b"\"<>\\\n"].concat();
    for kind in [Kind::Approval, Kind::Credential, Kind::Bearer] {
        let originals = Originals::new(vec![(kind, value.clone().into())], &[]);
        for form in [
            value.as_slice(),
            b"a%2F%20%26%20%22%3C%3E%5C%0A",
            b"a%2f%20%26%20%22%3c%3e%5c%0a",
            b"a/ &amp; &quot;&lt;&gt;\\\n",
            b"a/ & \\\"<>\\\\\\n",
            &digest(&value),
        ] {
            let field = [b"before ".as_slice(), form, b" after"].concat();
            assert!(crate::original(&field, &originals), "{kind:?}");
            assert_eq!(bytes(&field, &originals), Some(What::Original), "{kind:?}");
            assert_eq!(crate::shape(&field), None, "{kind:?} has no shape");
        }
        assert!(!original(&vec![b'x'; value.len()], &originals), "{kind:?} placeholder");
        assert!(!original(&digest(&vec![b'x'; value.len()]), &originals), "{kind:?} placeholder digest");
    }
    for (value, what) in samples() {
        if let What::Shape(expected) = what {
            let originals = Originals::new(vec![(Kind::Approval, value.clone().into())], &[]);
            assert!(original(&value, &originals));
            assert_eq!(shape(&value), Some(expected));
            assert_eq!(bytes(&value, &originals), Some(What::Original), "original precedes shape {expected:?}");
            assert_eq!(bytes(&value, &Originals::new(Vec::new(), &[])), Some(what));
        }
    }
}
