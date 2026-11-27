use super::*;
use skein_codec::layout::{Plan, Why};
use skein_codec::schema::{Schema, parse};

// Big-endian version, number (or length and digits), then length and UTF-8 label.
const NUMBER_SEVEN: &[u8] = b"\0\x01\0\0\0\x07\0\0\0\x05caf\xc3\xa9";
const TEXT_SEVEN: &[u8] = b"\0\x01\0\0\0\x017\0\0\0\x05caf\xc3\xa9";
const NUMBER_FORTY_TWO: &[u8] = b"\0\x01\0\0\0\x2a\0\0\0\x05caf\xc3\xa9";
const TEXT_FORTY_TWO: &[u8] = b"\0\x01\0\0\0\x0242\0\0\0\x05caf\xc3\xa9";
const NUMBER_MAX: &[u8] = b"\0\x01\xff\xff\xff\xff\0\0\0\x10abcdefghijklmnop";
const TEXT_MAX: &[u8] = b"\0\x01\0\0\0\x0a4294967295\0\0\0\x10abcdefghijklmnop";
const NUMBER_ZERO: &[u8] = b"\0\x01\0\0\0\0\0\0\0\0";
// The largest text-layout record: version, two lengths, 10 digits and 16 label bytes.
const CAPACITY: u32 = 2 + 4 + 10 + 4 + 16;

fn fixtures() -> (Schema, Schema) {
    (parse(WRITTEN_NUMBER.as_bytes()).unwrap(), parse(WRITTEN_TEXT.as_bytes()).unwrap())
}

fn side<'a>(text: &'a Schema, name: &[u8]) -> Side<'a> {
    Side { text, root: text.root(name, None).unwrap(), held: &[] }
}

fn retyped_side(text: &Schema) -> Side<'_> {
    side(text, b"Retyped")
}

fn filled(capacity: u32) -> Buffer {
    let mut out = Buffer::with_capacity(capacity);
    out.put(b"stale").unwrap();
    out
}

fn assert_refusal(result: Result<(), Refusal>, out: &Buffer, declaration: &str, part: &str, why: &str) {
    let refusal = result.unwrap_err();
    assert_eq!((&*refusal.declaration, &*refusal.part, &*refusal.why), (declaration, part, why));
    assert!(out.is_empty());
}

#[test]
fn neither_direction_derives_a_plan_in_either_mode() {
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    for mode in [Mode::Read, Mode::Keep] {
        for (from, to) in [(&number, &text), (&text, &number)] {
            let error = Plan::between(mode, from.text, from.root, from.held, to.text, to.root, to.held).unwrap_err();
            assert_eq!(error.why, Why::Retyped);
        }
    }
}

#[test]
fn both_modes_write_each_layout_with_its_label() {
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    let rows = [
        (&number, &text, NUMBER_SEVEN, TEXT_SEVEN),
        (&text, &number, TEXT_FORTY_TWO, NUMBER_FORTY_TWO),
        (&number, &text, NUMBER_MAX, TEXT_MAX),
        (&text, &number, TEXT_MAX, NUMBER_MAX),
    ];
    for mode in [Mode::Read, Mode::Keep] {
        for (from, to, value, expected) in rows {
            let mut out = filled(u32::try_from(expected.len()).unwrap());
            convert(mode, from, to, value, &mut out).unwrap();
            assert_eq!(out.as_slice(), expected);
        }
    }
}

#[test]
fn nondecimal_text_is_refused_at_the_number_field() {
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    let rows: &[&[u8]] = &[b"\0\x01\0\0\0\x02+1\0\0\0\0", b"\0\x01\0\0\0\x021x\0\0\0\0", b"\0\x01\0\0\0\0\0\0\0\0"];
    for &value in rows {
        let mut out = filled(CAPACITY);
        let result = convert(Mode::Keep, &text, &number, value, &mut out);
        assert_refusal(result, &out, "Retyped", "n", "expected decimal digits");
    }
}

#[test]
fn decimal_past_u32_is_refused_at_the_number_field() {
    const PAST_U32: &[u8] = b"\0\x01\0\0\0\x0a4294967296\0\0\0\0";
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    let mut out = filled(CAPACITY);
    let result = convert(Mode::Read, &text, &number, PAST_U32, &mut out);
    assert_refusal(result, &out, "Retyped", "n", "decimal number exceeds u32");
}

#[test]
fn a_result_one_byte_past_capacity_is_refused() {
    const ONE_SHORT: u32 = CAPACITY - 1;
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    let mut out = filled(ONE_SHORT);
    let result = convert(Mode::Keep, &number, &text, NUMBER_MAX, &mut out);
    assert_refusal(result, &out, "Retyped", "label", "result exceeds output capacity");
}

#[test]
fn another_family_or_root_declaration_is_refused() {
    const OTHER_FAMILY: &str = "family fixture-other 1\nversioned record Retyped { n: u32, label: text 16 }\n";
    const OTHER_ROOT: &str = "family fixture-written 1\nversioned record Other { n: u32, label: text 16 }\n";
    let (number, _) = fixtures();
    let from = retyped_side(&number);
    let rows: &[(&str, &[u8])] = &[(OTHER_FAMILY, b"Retyped"), (OTHER_ROOT, b"Other")];
    for &(text, root) in rows {
        let other = parse(text.as_bytes()).unwrap();
        let mut out = filled(CAPACITY);
        let result = convert(Mode::Read, &from, &side(&other, root), NUMBER_SEVEN, &mut out);
        assert_refusal(result, &out, "Retyped", "", "unsupported target value type");
    }
}

#[test]
fn a_pair_with_no_written_change_is_refused() {
    let (number, _) = fixtures();
    let number = retyped_side(&number);
    let mut out = filled(CAPACITY);
    let result = convert(Mode::Read, &number, &number, NUMBER_SEVEN, &mut out);
    assert_refusal(result, &out, "Retyped", "", "unsupported layout pair");
}

#[test]
fn a_number_round_trips_through_the_text_layout() {
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    let mut encoded = Buffer::with_capacity(CAPACITY);
    let mut decoded = Buffer::with_capacity(CAPACITY);
    convert(Mode::Keep, &number, &text, NUMBER_ZERO, &mut encoded).unwrap();
    convert(Mode::Read, &text, &number, encoded.as_slice(), &mut decoded).unwrap();
    assert_eq!(decoded.as_slice(), NUMBER_ZERO);
}

#[test]
fn malformed_records_are_refused_at_the_source_part() {
    let (number, text) = fixtures();
    let (number, text) = (retyped_side(&number), retyped_side(&text));
    let rows: &[(&[u8], &str, &str)] = &[
        (b"\0\x02\0\0\0\x07\0\0\0\0", "", "missing or wrong source version"),
        (b"\0\x01\0\0\0", "n", "short number"),
        (b"\0\x01\0\0\0\x07\0\0\0\x01\xff", "label", "text is not UTF-8"),
        (b"\0\x01\0\0\0\x07\0\0\0\x11abcdefghijklmnopq", "label", "short or oversized text"),
        (b"\0\x01\0\0\0\x07\0\0\0\0\0", "", "trailing bytes"),
    ];
    for &(value, part, why) in rows {
        let mut out = filled(CAPACITY);
        let result = convert(Mode::Read, &number, &text, value, &mut out);
        assert_refusal(result, &out, "Retyped", part, why);
    }
}
