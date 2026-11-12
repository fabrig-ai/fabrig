use super::*;
use crate::journey::{Original, placeholder};
use jig_client_protocol_wire as w;
use skein_lib::{Reader, Writer, sha256::digest};
use skein_ui::Act;

fn fake() -> Box<[u8]> {
    [b"sample-".as_slice(), b"value"].concat().into()
}

fn input(value: &[u8]) -> Line {
    Line::Act {
        act: Act::Input { value: value.into() },
        event: b"AccountKey { value: Secret { len: 12 } }".as_slice().into(),
        name: b"Key".as_slice().into(),
    }
}

fn kept(record: &[u8]) -> Box<[u8]> {
    let value = w::Kept::new(&w::CEILINGS, w::KeptParts { store_version: 1, record: record.into() })
        .expect("bounded fake record");
    let mut writer = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured fake record");
    writer.finish()
}

fn has(original: &Original, bytes: &[u8]) -> bool {
    original.forms().iter().any(|form| form.as_ref() == bytes)
}

#[test]
fn the_links_value_is_an_original() {
    let value = fake();
    let originals = crate::journey::originals(&[Line::Secret(value.clone())], &[]);
    assert_eq!(originals.count(Kind::Link), 1);
    assert_eq!(originals.sought().len(), 1);
    assert!(originals.sought().iter().any(|one| one.kind() == Kind::Link && has(one, &value)));
    let spaced = [b" ".as_slice(), &value, b"\n"].concat();
    let originals = crate::journey::originals(&[Line::Secret(spaced.clone().into())], &[]);
    assert!(originals.sought().iter().any(|one| has(one, &spaced)));
}

#[test]
fn a_stored_values_kept_record_is_the_original_and_its_head_stands() {
    let record = [b" \t".as_slice(), &fake(), b"\r\n"].concat();
    let value = kept(&record);
    let head = kept_head(&value).expect("kept head");
    assert_eq!(head, 8);
    let originals =
        crate::journey::originals(&[Line::Stored { key: b"store".as_slice().into(), value: value.clone() }], &[]);
    assert_eq!(originals.count(Kind::Stored), 1);
    assert!(originals.sought().iter().any(|one| has(one, &record) && !has(one, &value)));
    let mut replaced = value[..head].to_vec();
    replaced.extend(std::iter::repeat_n(b'x', record.len()));
    let mut reader = Reader::new(&replaced);
    let decoded = w::Kept::decode(&w::CEILINGS, &mut reader).expect("head still decodes");
    assert_eq!(decoded.record().len(), record.len());
    assert!(decoded.record().iter().all(|byte| *byte == b'x'));
    assert!(reader.is_empty());
}

#[test]
fn a_stored_value_that_isnt_kept_is_an_original_whole() {
    let value = [b" ".as_slice(), &fake(), b"\n"].concat();
    assert!(kept_head(&value).is_none());
    let originals = crate::journey::originals(
        &[Line::Stored { key: b"store".as_slice().into(), value: value.clone().into() }],
        &[],
    );
    assert!(originals.sought().iter().any(|one| has(one, &value)));
    let mut trailing = kept(&fake()).into_vec();
    trailing.push(0);
    assert!(kept_head(&trailing).is_none(), "a kept prefix is not a whole kept value");
    let mut cut = kept(&fake()).into_vec();
    cut.pop();
    assert!(kept_head(&cut).is_none(), "a cut record is not kept");
}

#[test]
fn a_save_of_the_stream_is_an_original() {
    let value = fake();
    let lines = [
        Line::Save { key: jig_client::STREAM_KEY.into(), value: kept(&value) },
        Line::Save { key: [7; 16].into(), value: kept(b"request body") },
    ];
    let originals = crate::journey::originals(&lines, &[]);
    assert_eq!(originals.count(Kind::Stored), 1);
    assert_eq!(originals.sought().len(), 1);
    assert!(originals.sought().iter().any(|one| has(one, &value)));
}

#[test]
fn a_typed_value_needs_the_secret_in_its_event() {
    let value = fake();
    let secret = input(&value);
    assert!(typed(&secret));
    let plain = Line::Act {
        act: Act::Input { value: value.clone() },
        event: b"Filter { value: plain }".as_slice().into(),
        name: b"Secret { len: 12 }".as_slice().into(),
    };
    let press = Line::Act { act: Act::Press, event: SECRET_EVENT.into(), name: Box::default() };
    assert!(!typed(&plain));
    assert!(typed(&press));
    assert!(!typed(&Line::Secret(value.clone())));
    let originals = crate::journey::originals(&[plain, press], &[]);
    assert_eq!(originals.count(Kind::Typed), 0);
    assert!(originals.sought().is_empty());
    let originals = crate::journey::originals(&[secret], &[]);
    assert_eq!(originals.count(Kind::Typed), 1);
    assert!(originals.sought().iter().any(|one| has(one, &value)));
}

#[test]
fn a_typing_runs_heads_are_not_sought() {
    let value = fake();
    let mut lines: Vec<_> = (1..=value.len()).map(|end| input(&value[..end])).collect();
    lines.push(input(&value));
    let originals = crate::journey::originals(&lines, &[]);
    assert_eq!(originals.count(Kind::Typed), 1);
    assert_eq!(originals.sought().len(), 1);
    assert!(originals.sought().iter().all(|one| one.len() == value.len() && has(one, &value)));
}

#[test]
fn a_short_typed_value_is_not_sought() {
    for value in [b"a".as_slice(), b"ab", b"abc"] {
        let originals = crate::journey::originals(&[input(value)], &[]);
        assert_eq!(originals.count(Kind::Typed), 1);
        assert!(originals.sought().is_empty());
    }
    let originals = crate::journey::originals(&[input(b"abcd")], &[]);
    assert_eq!(originals.sought().len(), 1);
}

#[test]
fn surrounding_space_stands_outside_the_original() {
    let value = b" \t\r\nab\n\r\t ";
    assert!(placeholder(value).as_ref() == b" \t\r\nxx\n\r\t ");
    assert!(placeholder(b" ab\n").as_ref() == b" xx\n");
    assert!(placeholder(b" \t\r\n").as_ref() == b" \t\r\n");
    assert!(placeholder(b"a b").as_ref() == b"xxx");
    assert!(placeholder(b"\x0bab\x0c").as_ref() == b"xxxx");
    let originals = crate::journey::originals(&[input(b" \t\r\nabcd\n\r\t ")], &[value.as_slice().into()]);
    assert!(originals.sought().iter().any(|one| one.kind() == Kind::Given && one.len() == 2 && has(one, b"ab")));
    assert!(originals.sought().iter().any(|one| one.kind() == Kind::Typed && one.len() == 4 && has(one, b"abcd")));
}

#[test]
fn an_empty_or_x_original_is_none() {
    for value in [b"".as_slice(), b"x", b"xxxx"] {
        let lines =
            [Line::Secret(value.into()), Line::Stored { key: Box::default(), value: value.into() }, input(value)];
        let originals = crate::journey::originals(&lines, &[value.into()]);
        assert!(originals.sought().is_empty());
        for kind in [Kind::Link, Kind::Stored, Kind::Typed, Kind::Given] {
            assert_eq!(originals.count(kind), 0);
        }
    }
}

#[test]
fn the_forms_are_each_once_bytes_first() {
    let plain = fake();
    let originals = crate::journey::originals(&[], std::slice::from_ref(&plain));
    assert!(originals.sought().iter().all(|one| one.forms().len() == 1 && one.forms()[0] == plain));
    let text = b"a/ & \"<>\\\n";
    let originals = crate::journey::originals(&[Line::Secret(text.as_slice().into())], &[]);
    for one in originals.sought() {
        assert!(one.forms()[0].as_ref() == text);
        for expected in [
            text.as_slice(),
            b"a%2F%20%26%20%22%3C%3E%5C%0A",
            b"a%2f%20%26%20%22%3c%3e%5c%0a",
            b"a/ &amp; &quot;&lt;&gt;\\\n",
            b"a/ & \\\"<>\\\\\\n",
        ] {
            assert_eq!(one.forms().iter().filter(|form| form.as_ref() == expected).count(), 1);
        }
        assert_eq!(one.forms().len(), 5);
    }
    let originals = crate::journey::originals(&[], &[b"\xff<".as_slice().into()]);
    assert_eq!(originals.sought().len(), 1);
    for one in originals.sought() {
        assert_eq!(one.forms().len(), 4, "non-UTF-8 has no Debug text form");
        assert!(has(one, b"\xff<") && has(one, b"%FF%3C") && has(one, b"%ff%3c") && has(one, b"\xff&lt;"));
    }
}

#[test]
fn the_digests_are_of_the_bytes_and_the_value() {
    let value = b" \tabcd\r\n";
    let originals = crate::journey::originals(&[input(value)], &[]);
    assert_eq!(originals.sought().len(), 1);
    for one in originals.sought() {
        assert!(one.digests().contains(&(digest(b"abcd"), digest(b"xxxx"))));
        assert!(one.digests().contains(&(digest(value), digest(b" \txxxx\r\n"))));
        assert_eq!(one.digests().len(), 2);
    }
    let originals = crate::journey::originals(&[], &[b"abcd".as_slice().into(), value.as_slice().into()]);
    assert_eq!(originals.sought().len(), 1);
    assert!(originals.sought().iter().all(|one| one.digests().len() == 2));
}

#[test]
fn longest_first_each_once() {
    let value = fake();
    let originals = crate::journey::originals(
        &[Line::Secret(value.clone()), Line::Secret(value.clone())],
        &[b"ab".as_slice().into(), value.clone(), b"abcdefghijklmno".as_slice().into()],
    );
    assert_eq!(originals.sought().len(), 3);
    assert_eq!(originals.count(Kind::Link), 1);
    assert_eq!(originals.count(Kind::Given), 3);
    assert!(originals.sought().windows(2).all(|pair| pair[0].len() >= pair[1].len()));
    assert!(originals.sought().iter().any(|one| has(one, &value) && one.kind() == Kind::Link));
}
