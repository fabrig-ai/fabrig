use super::*;
fn fake() -> Box<[u8]> {
    [b"sample-".as_slice(), b"value"].concat().into()
}
fn has(original: &Original, bytes: &[u8]) -> bool {
    original.forms().iter().any(|form| form.as_ref() == bytes)
}

#[test]
fn a_given_text_is_an_original() {
    let value = fake();
    let originals = Originals::new(Vec::new(), std::slice::from_ref(&value));
    assert_eq!(originals.count(Kind::Given), 1);
    assert!(originals.sought().iter().any(|one| one.kind() == Kind::Given && has(one, &value)));
}

#[test]
fn debug_prints_no_bytes() {
    let originals = Originals::new(Vec::new(), &[fake()]);
    assert_eq!(originals.sought().len(), 1);
    for one in originals.sought() {
        let printed = format!("{one:?}");
        assert!(printed == "Original { kind: Given, len: 12 }");
    }
}

#[test]
fn candidate_kinds_share_forms_and_count_separately() {
    let kinds = [Kind::Approval, Kind::Credential, Kind::Bearer];
    let text = [b"a/ & ".as_slice(), b"\"<>\\\n"].concat();
    let binary = [b"\xff".as_slice(), b"<"].concat();
    let rows: &[(&[u8], &[&[u8]])] = &[
        (
            &text,
            &[
                &text,
                b"a%2F%20%26%20%22%3C%3E%5C%0A",
                b"a%2f%20%26%20%22%3c%3e%5c%0a",
                b"a/ &amp; &quot;&lt;&gt;\\\n",
                b"a/ & \\\"<>\\\\\\n",
            ],
        ),
        (&binary, &[&binary, b"%FF%3C", b"%ff%3c", b"\xff&lt;"]),
    ];
    for kind in kinds {
        for other in kinds.into_iter().filter(|other| *other != kind) {
            for (value, forms) in rows {
                let originals = Originals::new(
                    vec![(kind, (*value).into()), (kind, (*value).into()), (other, (*value).into())],
                    &[b" given-text ".as_slice().into()],
                );
                assert_eq!(originals.count(kind), 1, "{kind:?} counted once");
                assert_eq!(originals.count(other), 1, "{other:?} counted separately");
                assert_eq!(originals.count(Kind::Given), 1);
                assert_eq!(originals.sought().len(), 2, "shared bytes and the given text");
                let one = originals.sought().iter().find(|one| one.kind() == kind).expect("candidate original");
                assert_eq!(one.len(), value.len(), "candidate whitespace stands");
                assert_eq!(one.forms()[0].as_ref(), *value);
                assert_eq!(one.forms().len(), forms.len());
                for form in *forms {
                    assert_eq!(one.forms().iter().filter(|found| found.as_ref() == *form).count(), 1, "{kind:?}");
                }
                assert_eq!(one.digests(), &[(digest(value), digest(&vec![b'x'; value.len()]))]);
                assert!(originals.sought().iter().any(|one| has(one, b"given-text")));
            }
        }
    }
}
