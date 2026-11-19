//! The shared helpers' wire rules, independently of a consuming family.

use crate::{Path, Problem, Reason, Record, field};
use field::Field;
use skein_lib::{Duration, List, Reader, Secret, Writer};

#[rustfmt::skip]
#[path = "lean_fixture.rs"]
mod fixture;

// Only the second family's identity is needed, not a second codec.
mod other_fixture {
    pub struct Limits;
    pub type Path = crate::Path<Limits>;
    pub type Problem = crate::Problem<Limits>;
}

#[test]
fn families_have_distinct_problem_types() {
    use core::any::TypeId;
    assert_ne!(TypeId::of::<fixture::Problem>(), TypeId::of::<other_fixture::Problem>());
    assert_ne!(TypeId::of::<fixture::Path>(), TypeId::of::<other_fixture::Path>());
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test formats Debug without requiring traits from Limits")]
fn positions_and_refusals_need_no_limits_traits_or_storage() {
    use core::{
        fmt::Debug,
        hash::{Hash, Hasher},
        mem::size_of,
    };
    use std::hash::DefaultHasher;

    struct Limits;
    const PATH: Path<Limits> = Path::new(9, 2);
    const PROBLEM: Problem<Limits> = PATH.problem(Reason::Trailing);
    fn traits<T: Clone + Copy + Debug + Eq + Hash>(value: T) -> T {
        value
    }
    fn hash(value: &impl Hash) -> u64 {
        let mut state = DefaultHasher::new();
        value.hash(&mut state);
        state.finish()
    }

    let path = traits(PATH);
    let problem = traits(PROBLEM);
    assert_eq!((path.declaration, path.part), (9, 2));
    assert_eq!(size_of::<Path<Limits>>(), size_of::<(u16, u16)>());
    assert_eq!(size_of::<Problem<Limits>>(), size_of::<Problem<()>>());
    assert_eq!(path, PATH);
    assert_ne!(path, Path::new(8, 2));
    assert_ne!(path, Path::new(9, 1));
    assert_eq!(problem, PROBLEM);
    assert_ne!(problem, Path::new(8, 2).problem(Reason::Trailing));
    assert_ne!(problem, PATH.problem(Reason::Short));
    assert_eq!(hash(&path), hash(&(9_u16, 2_u16)));
    assert_eq!(hash(&problem), hash(&(9_u16, 2_u16, Reason::Trailing)));
    assert_eq!(format!("{path:?}"), "Path { declaration: 9, part: 2 }");
    assert_eq!(format!("{problem:?}"), "Problem { path: Path { declaration: 9, part: 2 }, reason: Trailing }");
}

const PATH: Path<()> = Path::new(9, 2);

fn written<T, F: Field<(), T>>(shape: &F, value: &T) -> Result<Box<[u8]>, Problem<()>> {
    let mut measure = Writer::measure(1024);
    shape.write(value, &(), &mut measure, PATH)?;
    let mut out = Writer::new(measure.written());
    shape.write(value, &(), &mut out, PATH)?;
    Ok(out.finish())
}

fn check<T: core::fmt::Debug + PartialEq, F: Field<(), T>>(shape: &F, value: &T, expected: &[u8]) {
    assert_eq!(written(shape, value).expect("measured and written").as_ref(), expected);
    let decoded = shape.read(&(), &mut Reader::new(expected), PATH).expect("read");
    assert_eq!(&decoded, value);
    assert_eq!(written(shape, &decoded).expect("canonical encoding").as_ref(), expected);
    for cut in 0..expected.len() {
        assert_eq!(
            shape.read(&(), &mut Reader::new(expected.get(..cut).expect("prefix")), PATH),
            Err(PATH.problem(Reason::Short)),
            "cut {cut}"
        );
    }
}

fn round_trip<T: core::fmt::Debug + PartialEq + Clone, F: Field<(), T> + Copy>(shape: F, value: T, expected: &[u8]) {
    check(&shape, &value, expected);
    let mut out = Writer::new(expected.len().checked_add(4).expect("list count"));
    out.put(&[0, 0, 0, 1]).expect("count");
    out.put(expected).expect("item");
    check(
        &field::Sequence { limit: 1, minimum: u32::try_from(expected.len()).expect("small item"), item: shape },
        &list([value.clone()], 1),
        &out.finish(),
    );
    let mut out = Writer::new(expected.len().checked_add(1).expect("option tag"));
    out.put(&[1]).expect("tag");
    out.put(expected).expect("item");
    check(&field::Optional { item: shape }, &Some(value), &out.finish());
}

fn list<T: core::fmt::Debug, const N: usize>(items: [T; N], capacity: u32) -> List<T> {
    let mut list = List::with_capacity(capacity);
    for item in items {
        list.push(item).expect("test capacity");
    }
    list
}

#[test]
fn each_field_kind_has_canonical_bytes_and_short_cuts() {
    round_trip(field::U8, 0xa5, &[0xa5]);
    round_trip(field::U16, 0x0102, &[1, 2]);
    round_trip(field::U32, 0x0102_0304, &[1, 2, 3, 4]);
    round_trip(field::U64, 0x0102_0304_0506_0708, &[1, 2, 3, 4, 5, 6, 7, 8]);
    round_trip(field::Boolean, true, &[1]);
    round_trip(field::Boolean, false, &[0]);
    round_trip(field::Span, Duration::from_nanos(9), &[0, 0, 0, 0, 0, 0, 0, 9]);
    round_trip(field::Fixed::<3>, [0xa5; 3], &[0xa5; 3]);
    round_trip(field::Bytes { limit: 3 }, Box::from(&b"abc"[..]), &[0, 0, 0, 3, 97, 98, 99]);
    round_trip(field::Hidden { limit: 3 }, Secret::new(Box::from(&b"abc"[..])), &[0, 0, 0, 3, 97, 98, 99]);
    round_trip(field::Text { limit: 3 }, Box::from(&b"abc"[..]), &[0, 0, 0, 3, 97, 98, 99]);
    round_trip(field::Sequence { limit: 2, minimum: 2, item: field::U16 }, list([1, 2], 2), &[0, 0, 0, 2, 0, 1, 0, 2]);
    round_trip(field::Optional { item: field::U16 }, Some(2), &[1, 0, 2]);
    round_trip(field::Optional { item: field::U16 }, None, &[0]);
    round_trip(
        field::Sequence { limit: 2, minimum: 1, item: field::Optional { item: field::Text { limit: 3 } } },
        list([Some(Box::from(&b"abc"[..])), None], 2),
        &[0, 0, 0, 2, 1, 0, 0, 0, 3, 97, 98, 99, 0],
    );
    round_trip(
        field::Optional { item: field::Sequence { limit: 2, minimum: 1, item: field::U8 } },
        Some(list([1, 2], 2)),
        &[1, 0, 0, 0, 2, 1, 2],
    );
}

#[test]
fn limits_and_refusals_precede_allocating_contents() {
    let long = Box::from(&b"abcd"[..]);
    assert_eq!(written(&field::Bytes { limit: 3 }, &long), Err(PATH.problem(Reason::Bound)));
    assert_eq!(written(&field::Hidden { limit: 3 }, &Secret::new(long.clone())), Err(PATH.problem(Reason::Bound)));
    assert_eq!(written(&field::Text { limit: 3 }, &long), Err(PATH.problem(Reason::Bound)));
    assert_eq!(written(&field::Text { limit: 3 }, &Box::from(&[0xff][..])), Err(PATH.problem(Reason::Utf8)));
    let shape = field::Sequence { limit: 2, minimum: 1, item: field::U8 };
    assert_eq!(written(&shape, &list([1, 2, 3], 3)), Err(PATH.problem(Reason::Bound)));
    assert_eq!(written(&shape, &list([1, 2], 100)).expect("length, not capacity").as_ref(), &[0, 0, 0, 2, 1, 2]);
    assert_eq!(field::Boolean.read(&(), &mut Reader::new(&[2]), PATH), Err(PATH.problem(Reason::Bool)));
    assert_eq!(
        field::Optional { item: field::U8 }.read(&(), &mut Reader::new(&[2]), PATH),
        Err(PATH.problem(Reason::Tag))
    );
    let allocations = skein_heap::Span::start();
    assert_eq!(
        field::Bytes { limit: 3 }.read(&(), &mut Reader::new(&[0xff; 4]), PATH),
        Err(PATH.problem(Reason::Bound))
    );
    assert_eq!(
        field::Hidden { limit: 3 }.read(&(), &mut Reader::new(&[0xff; 4]), PATH),
        Err(PATH.problem(Reason::Bound))
    );
    assert_eq!(
        field::Text { limit: 3 }.read(&(), &mut Reader::new(&[0xff; 4]), PATH),
        Err(PATH.problem(Reason::Bound))
    );
    assert_eq!(shape.read(&(), &mut Reader::new(&[0xff; 4]), PATH), Err(PATH.problem(Reason::Bound)));
    assert_eq!(shape.read(&(), &mut Reader::new(&[0, 0, 0, 2, 1]), PATH), Err(PATH.problem(Reason::Short)));
    assert_eq!(
        field::Text { limit: 3 }.read(&(), &mut Reader::new(&[0, 0, 0, 1, 0xff]), PATH),
        Err(PATH.problem(Reason::Utf8))
    );
    assert_eq!(allocations.made(), 0, "refused lengths and text allocate no content");
}

fn record_round_trip<T: Record<Limits = fixture::Limits> + core::fmt::Debug + PartialEq + Clone>(
    value: T,
    bytes: &[u8],
) {
    let path = fixture::Path::new(9, 2);
    assert_eq!(value.measure(&fixture::CEILINGS), Ok(u32::try_from(bytes.len()).expect("small")));
    assert_eq!(value.encode(&fixture::CEILINGS).expect("encode").as_ref(), bytes);
    assert_eq!(T::decode(&fixture::CEILINGS, &mut Reader::new(bytes)), Ok(value.clone()));
    let sequence =
        field::Sequence { limit: 1, minimum: u32::try_from(bytes.len()).expect("small record"), item: field::Nested };
    let mut out = Writer::new(bytes.len().checked_add(4).expect("count prefix"));
    sequence.write(&list([value.clone()], 1), &fixture::CEILINGS, &mut out, path).expect("nested list write");
    let wire = out.finish();
    assert_eq!(wire.get(..4), Some(&[0, 0, 0, 1][..]));
    assert_eq!(wire.get(4..), Some(bytes));
    let read: List<T> = sequence.read(&fixture::CEILINGS, &mut Reader::new(&wire), path).expect("nested list read");
    assert_eq!(read.as_slice(), core::slice::from_ref(&value));
    let option = field::Optional { item: field::Nested };
    let mut out = Writer::new(bytes.len().checked_add(1).expect("option tag"));
    option.write(&Some(value.clone()), &fixture::CEILINGS, &mut out, path).expect("nested option write");
    let wire = out.finish();
    assert_eq!(wire.first(), Some(&1));
    assert_eq!(wire.get(1..), Some(bytes));
    let read: Option<T> = option.read(&fixture::CEILINGS, &mut Reader::new(&wire), path).expect("nested option read");
    assert_eq!(read, Some(value));
    for cut in 0..bytes.len() {
        assert_eq!(
            T::decode(&fixture::CEILINGS, &mut Reader::new(bytes.get(..cut).expect("prefix"))).expect_err("cut").reason,
            Reason::Short
        );
    }
    let mut grown = Writer::new(bytes.len().checked_add(1).expect("one trailing byte"));
    grown.put(bytes).expect("original");
    grown.put(&[0]).expect("tail");
    let grown = grown.finish();
    assert_eq!(T::decode(&fixture::CEILINGS, &mut Reader::new(&grown)), Err(T::TAIL.problem(Reason::Trailing)));
}

#[test]
fn records_enumerations_versions_and_nested_paths() {
    use fixture::{Choice, Empty, Nested, Row, Words, paths};
    record_round_trip(Empty {}, &[]);
    record_round_trip(Words { words: Box::from(&b"abc"[..]) }, &[0, 0, 0, 3, 97, 98, 99]);
    record_round_trip(Choice::Unit, &[0]);
    record_round_trip(Choice::Words(Words { words: Box::from(&b"abc"[..]) }), &[1, 0, 0, 0, 3, 97, 98, 99]);
    record_round_trip(
        Choice::BoxedWords(Box::new(Words { words: Box::from(&b"abc"[..]) })),
        &[2, 0, 0, 0, 3, 97, 98, 99],
    );
    record_round_trip(Row { choice: Choice::Unit }, &[0, 7, 0]);
    record_round_trip(
        Nested {
            words: list([None, Some(Box::from(&b"abc"[..]))], 2),
            maybe: Some(list([2], 1)),
            records: list([Words { words: Box::from(&b"abc"[..]) }], 1),
        },
        &[0, 0, 0, 2, 0, 1, 0, 0, 0, 3, 97, 98, 99, 1, 0, 0, 0, 1, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3, 97, 98, 99],
    );
    assert_eq!(
        Words { words: Box::from(&b"abcd"[..]) }.measure(&fixture::CEILINGS),
        Err(paths::WORDS_WORDS.problem(Reason::Bound))
    );
    assert_eq!(
        Words { words: Box::from(&[0xff][..]) }.measure(&fixture::CEILINGS),
        Err(paths::WORDS_WORDS.problem(Reason::Utf8))
    );
    assert_eq!(
        Row::decode(&fixture::CEILINGS, &mut Reader::new(&[0, 8, 0])),
        Err(paths::ROW_VERSION.problem(Reason::Version))
    );
    assert_eq!(
        Row::decode(&fixture::CEILINGS, &mut Reader::new(&[0, 7, 3])),
        Err(paths::CHOICE_TAG.problem(Reason::Tag))
    );
    assert_eq!(
        Row::decode(&fixture::CEILINGS, &mut Reader::new(&[0, 7, 2, 0, 0, 0, 4])),
        Err(paths::WORDS_WORDS.problem(Reason::Bound))
    );
    assert_eq!(fixture::name(paths::WORDS_WORDS), Some("Words.words"));
    assert_eq!(fixture::name(fixture::Path::new(u16::MAX, 0)), None);
    assert_eq!(fixture::name(fixture::Path::new(1, u16::MAX)), None);
    let mut limits = fixture::CEILINGS;
    limits.words_words = 1;
    assert_eq!(Words { words: Box::from(&b"ab"[..]) }.measure(&limits), Err(paths::WORDS_WORDS.problem(Reason::Bound)));
    limits.words_words = 100;
    assert_eq!(
        Words { words: Box::from(&b"abcd"[..]) }.measure(&limits),
        Err(paths::WORDS_WORDS.problem(Reason::Bound)),
        "ceilings still hold with invalid limits"
    );
    assert_eq!(fixture::check_limits(&limits), Err(paths::WORDS_WORDS.problem(Reason::Bound)));
    assert_eq!(Words::encoded_bound(&limits), None);
    let nested = Nested { words: list([Some(Box::from(&b"abcd"[..]))], 1), maybe: None, records: list([], 0) };
    assert_eq!(nested.measure(&fixture::CEILINGS), Err(paths::NESTED_WORDS.problem(Reason::Bound)));
    let nested =
        Nested { words: list([], 0), maybe: None, records: list([Words { words: Box::from(&b"abcd"[..]) }], 1) };
    assert_eq!(nested.measure(&fixture::CEILINGS), Err(paths::WORDS_WORDS.problem(Reason::Bound)));
    let nested = Nested { words: list([None, None, None], 3), maybe: None, records: list([], 0) };
    assert_eq!(nested.measure(&fixture::CEILINGS), Err(paths::NESTED_WORDS.problem(Reason::Bound)));
    assert!(fixture::encoded_bound(&fixture::CEILINGS).is_some());
    assert!(fixture::decoded_bound(&fixture::CEILINGS).is_some());
}
