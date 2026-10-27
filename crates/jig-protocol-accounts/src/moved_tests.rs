use crate::{
    moved::{self, Moved, Unread},
    v1,
};
use jig_domain_accounts::{DISPLAY, SUBJECT};
use skein_lib::{Reader, Wall, Writer, bytes};
use skein_oauth::{Limits, SavedToken};

pub(crate) fn limits() -> Limits {
    Limits {
        document_bytes: 512,
        string_bytes: 128,
        token_bytes: 128,
        client_bytes: 64,
        issued_client_bytes: 64,
        granted_bytes: 64,
        subject_bytes: 64,
        id_token_bytes: 64,
        detail_bytes: 64,
        record_bytes: 512,
        depth: 4,
        tokens: 32,
    }
}

fn fixture(full: bool) -> Moved {
    let mut record = SavedToken {
        key: 7,
        generation: 9,
        access_token: bytes::copy_of(b"a"),
        refresh_token: None,
        metadata: None,
        expires_at: Wall::from_nanos(123),
        issued_client_id: None,
        granted_scopes: None,
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
    };
    if full {
        record.refresh_token = Some(bytes::copy_of(b"refresh"));
        record.metadata = Some(bytes::copy_of(b"metadata"));
        record.issued_client_id = Some(bytes::copy_of(b"issued-client"));
        record.granted_scopes = Some(bytes::copy_of(b"scope one"));
        record.subject = Some(bytes::copy_of(b"subject"));
        record.id_token = Some(bytes::copy_of(b"id-token"));
        record.earliest_refresh_at = Some(Wall::from_nanos(100));
    }
    Moved {
        application: bytes::copy_of(b"a"),
        issuer: bytes::copy_of(b"i"),
        client_id: bytes::copy_of(b"c"),
        subject: bytes::copy_of(b"s"),
        display: bytes::copy_of(if full { b"Display" } else { b"" }),
        record,
    }
}

fn assert_same(expected: &Moved, actual: &Moved) {
    assert_eq!(expected.application, actual.application);
    assert_eq!(expected.issuer, actual.issuer);
    assert_eq!(expected.client_id, actual.client_id);
    assert_eq!(expected.subject, actual.subject);
    assert_eq!(expected.display, actual.display);
    assert_eq!(expected.record.key, actual.record.key);
    assert_eq!(expected.record.generation, actual.record.generation);
    assert_eq!(expected.record.expires_at, actual.record.expires_at);
    assert_eq!(expected.record.access_token, actual.record.access_token);
    assert_eq!(expected.record.refresh_token, actual.record.refresh_token);
    assert_eq!(expected.record.metadata, actual.record.metadata);
    assert_eq!(expected.record.issued_client_id, actual.record.issued_client_id);
    assert_eq!(expected.record.granted_scopes, actual.record.granted_scopes);
    assert_eq!(expected.record.subject, actual.record.subject);
    assert_eq!(expected.record.id_token, actual.record.id_token);
    assert_eq!(expected.record.earliest_refresh_at, actual.record.earliest_refresh_at);
}

// The version plus six length prefixes (protocol/llm.md, 5.6).
const HEADER: u32 = 2;
const PARTS: usize = 6;
const OUTER_CAPS: [(&str, u32); 5] = [
    ("application", moved::APPLICATION),
    ("issuer", moved::ISSUER),
    ("client id", moved::CLIENT_ID),
    ("subject", SUBJECT),
    ("display", DISPLAY),
];

#[test]
fn full_and_smallest_moved_records_round_trip() {
    for full in [true, false] {
        let original = fixture(full);
        let encoded = moved::encode(&original, &limits()).expect("valid record");
        assert_eq!(&encoded.expose()[..2], b"\0\x01");
        let decoded = moved::decode(encoded.expose(), &limits()).expect("valid file");
        assert_same(&original, &decoded);
        assert_eq!(moved::encode(&decoded, &limits()).expect("decoded record fits"), encoded);
    }
}

#[test]
fn schema_ceilings_match_the_parts_owners() {
    assert_eq!(v1::CEILINGS.moved_application, moved::APPLICATION);
    assert_eq!(v1::CEILINGS.moved_issuer, moved::ISSUER);
    assert_eq!(v1::CEILINGS.moved_client_id, moved::CLIENT_ID);
    assert_eq!(v1::CEILINGS.moved_subject, SUBJECT);
    assert_eq!(v1::CEILINGS.moved_display, DISPLAY);
    assert_eq!(
        u64::from(v1::CEILINGS.moved_record),
        skein_oauth::v3::SavedToken::worst_case_bytes(&skein_oauth::v3::CEILINGS).expect("saved token ceiling")
    );
}

// Walk the wire's lengths to edit a named part's prefix.
fn prefix(input: &[u8], part: usize) -> usize {
    let mut reader = Reader::new(input);
    reader.skip(HEADER).expect("version");
    for _ in 0..part {
        let len = reader.u32().expect("part length");
        reader.skip(len).expect("part bytes");
    }
    input.len().checked_sub(usize::try_from(reader.remaining()).expect("remaining fits")).expect("cursor")
}

fn length(input: &mut [u8], part: usize, len: u32) {
    let offset = prefix(input, part);
    for (index, byte) in len.to_be_bytes().into_iter().enumerate() {
        input[offset.checked_add(index).expect("prefix byte")] = byte;
    }
}

fn refused(input: &[u8], limits: &Limits, expected: Unread, name: &str) {
    assert_eq!(moved::decode(input, limits).err(), Some(expected), "{name}");
}

// Literal output of the encoder at 365afd554^ for fixture(false), retained
// from the J4 compatibility-refusal test, including its version-3 SavedToken.
const OLD: &[u8] = b"JIGM\0\x01\0\0\0\x01a\0\0\0\x01i\0\0\0\x01c\0\0\0\x01s\0\0\0\0\0\0\0\x22\0\x03\0\0\0\x07\0\0\0\0\0\0\0\x09\0\0\0\0\0\0\0\x7b\0\0\0\x01a\0\0\0\0\0\0\0";
const MAGIC_BYTES: usize = 4;

#[test]
fn old_jigm_literal_decodes_to_its_saved_account() {
    assert_same(&fixture(false), &moved::decode(OLD, &limits()).expect("old JIGM form"));
}

#[test]
fn an_old_file_reencodes_only_in_the_generated_form() {
    let decoded = moved::decode(OLD, &limits()).expect("old JIGM form");
    let encoded = moved::encode(&decoded, &limits()).expect("new form");
    assert_eq!(&encoded.expose()[..2], b"\0\x01");
    assert_same(&decoded, &moved::decode(encoded.expose(), &limits()).expect("new form decodes"));
}

#[test]
fn every_truncation_of_an_old_file_is_refused() {
    for end in MAGIC_BYTES..OLD.len() {
        refused(&OLD[..end], &limits(), Unread::Malformed, "truncated old file");
    }
}

#[test]
fn an_old_file_refuses_a_trailing_byte() {
    refused(&append_byte(OLD), &limits(), Unread::Trailing, "old trailing byte");
}

#[test]
fn an_old_file_refuses_a_version_after_its_magic() {
    let mut wrong_version = bytes::copy_of(OLD);
    for (index, byte) in 2_u16.to_be_bytes().into_iter().enumerate() {
        wrong_version[MAGIC_BYTES.checked_add(index).expect("version byte")] = byte;
    }
    refused(&wrong_version, &limits(), Unread::Version, "old version 2");
}

// Build boundary cases from the kept old bytes, never with today's encoder.
fn old_part(part: usize, replacement: &[u8]) -> Box<[u8]> {
    let replaced = replace_part(&OLD[MAGIC_BYTES..], part, replacement);
    with_magic(&replaced)
}

fn with_magic(input: &[u8]) -> Box<[u8]> {
    let mut out = Writer::new(MAGIC_BYTES.checked_add(input.len()).expect("small old file"));
    out.put(b"JIGM").expect("room");
    out.put(input).expect("room");
    out.finish()
}

fn append_byte(input: &[u8]) -> Box<[u8]> {
    let mut out = Writer::new(input.len().checked_add(1).expect("one extra byte"));
    out.put(input).expect("room");
    out.put(b"x").expect("room");
    out.finish()
}

#[test]
fn old_parts_at_their_bounds_decode() {
    for (part, (_name, cap)) in OUTER_CAPS.into_iter().enumerate() {
        let mut expected = fixture(false);
        *outer_field(&mut expected, part) = filled(cap);
        assert_same(&expected, &moved::decode(&old_part(part, &filled(cap)), &limits()).expect("old part at bound"));
    }
    let limits = record_limits(&fixture(false));
    assert_same(&fixture(false), &moved::decode(OLD, &limits).expect("old record at bound"));
}

#[test]
fn old_parts_refuse_oversized_prefixes_before_reading_bytes() {
    for (part, (name, cap)) in OUTER_CAPS.into_iter().chain([("record", limits().record_bytes)]).enumerate() {
        let mut changed = bytes::copy_of(OLD);
        let offset = MAGIC_BYTES.checked_add(prefix(&OLD[MAGIC_BYTES..], part)).expect("old prefix");
        let end = offset.checked_add(4).expect("prefix end");
        for (index, byte) in cap.checked_add(1).expect("small cap").to_be_bytes().into_iter().enumerate() {
            changed[offset.checked_add(index).expect("prefix byte")] = byte;
        }
        refused(&changed, &limits(), Unread::TooLarge, name);
        refused(&changed[..end], &limits(), Unread::TooLarge, name);
    }
    let mut limits = record_limits(&fixture(false));
    limits.record_bytes = limits.record_bytes.checked_sub(1).expect("nonzero record");
    refused(OLD, &limits, Unread::TooLarge, "old record one past bound");
}

#[test]
fn old_required_parts_refuse_empty_bytes() {
    for part in [0, 1, 2, 3, 5] {
        refused(&old_part(part, b""), &limits(), Unread::Malformed, "old empty required part");
    }
}

#[test]
fn an_old_file_refuses_an_invalid_nested_record() {
    refused(&old_part(5, b"invalid record"), &limits(), Unread::Record, "old nested record");
}

#[test]
fn old_file_at_the_whole_file_bound_decodes() {
    let mut old = bytes::copy_of(OLD);
    for (part, (_name, cap)) in OUTER_CAPS.into_iter().enumerate() {
        let replaced = replace_part(&old[MAGIC_BYTES..], part, &filled(cap));
        old = with_magic(&replaced);
    }
    let limits = record_limits(&fixture(false));
    assert_eq!(u64::try_from(old.len()).expect("size"), moved::bound(&limits));
    let decoded = moved::decode(&old, &limits).expect("old whole-file bound decodes");
    let mut expected = fixture(false);
    for (part, (_name, cap)) in OUTER_CAPS.into_iter().enumerate() {
        *outer_field(&mut expected, part) = filled(cap);
    }
    assert_same(&expected, &decoded);
    refused(&append_byte(&old), &limits, Unread::TooLarge, "old whole-file bound before trailing");
}

#[test]
fn another_version_is_refused_by_name() {
    let valid = moved::encode(&fixture(false), &limits()).expect("fixture");
    let mut changed = bytes::copy_of(valid.expose());
    for (index, byte) in 2_u16.to_be_bytes().into_iter().enumerate() {
        changed[index] = byte;
    }
    refused(&changed, &limits(), Unread::Version, "version 2");
}

#[test]
fn decode_refuses_each_part_one_past_its_bound() {
    let valid = moved::encode(&fixture(true), &limits()).expect("fixture");
    for (part, (name, cap)) in OUTER_CAPS.into_iter().chain([("record", limits().record_bytes)]).enumerate() {
        let mut changed = bytes::copy_of(valid.expose());
        length(&mut changed, part, cap.checked_add(1).expect("small cap"));
        refused(&changed, &limits(), Unread::TooLarge, name);
        // Only the prefix remains: oversize wins before trying to read bytes.
        let end = prefix(&changed, part).checked_add(4).expect("prefix end");
        refused(&changed[..end], &limits(), Unread::TooLarge, name);
    }
}

#[test]
fn every_truncation_of_a_valid_file_is_refused() {
    let valid = moved::encode(&fixture(true), &limits()).expect("fixture");
    for end in 0..valid.len() {
        refused(&valid.expose()[..end], &limits(), Unread::Malformed, "truncated file");
    }
}

#[test]
fn a_trailing_byte_is_refused() {
    let valid = moved::encode(&fixture(true), &limits()).expect("fixture");
    let mut trailing = Writer::new(valid.len().checked_add(1).expect("one extra byte"));
    trailing.put(valid.expose()).expect("room");
    trailing.put(b"x").expect("room");
    refused(&trailing.finish(), &limits(), Unread::Trailing, "trailing byte");
}

#[test]
fn whole_file_bound_is_checked_before_its_version() {
    let oversized =
        bytes::zeroed(usize::try_from(moved::bound(&limits()).checked_add(1).expect("bound plus one")).expect("fits"));
    refused(&oversized, &limits(), Unread::TooLarge, "whole-file bound");
}

fn filled(len: u32) -> Box<[u8]> {
    let mut field = bytes::zeroed(usize::try_from(len).expect("small bound"));
    field.fill(b'x');
    field
}

fn outer_field(original: &mut Moved, part: usize) -> &mut Box<[u8]> {
    match part {
        0 => &mut original.application,
        1 => &mut original.issuer,
        2 => &mut original.client_id,
        3 => &mut original.subject,
        4 => &mut original.display,
        _ => unreachable!("five outer fields"),
    }
}

#[test]
fn each_outer_part_accepts_its_exact_bound() {
    for (part, (_name, cap)) in OUTER_CAPS.into_iter().enumerate() {
        let mut original = fixture(false);
        *outer_field(&mut original, part) = filled(cap);
        let encoded = moved::encode(&original, &limits()).expect("field at bound");
        let decoded = moved::decode(encoded.expose(), &limits()).expect("field at bound decodes");
        assert_same(&original, &decoded);
    }
}

#[test]
fn encode_refuses_each_outer_part_one_past_its_bound() {
    for (part, (name, cap)) in OUTER_CAPS.into_iter().enumerate() {
        let mut original = fixture(false);
        *outer_field(&mut original, part) = filled(cap.checked_add(1).expect("small cap"));
        assert_eq!(moved::encode(&original, &limits()).err(), Some(Unread::TooLarge), "{name}");
    }
}

fn record_limits(original: &Moved) -> Limits {
    let mut limits = limits();
    limits.record_bytes =
        u32::try_from(skein_oauth::encode_record(&original.record, &limits).expect("record").len()).expect("size");
    limits
}

#[test]
fn a_record_at_its_runtime_bound_round_trips() {
    let original = fixture(true);
    let limits = record_limits(&original);
    let encoded = moved::encode(&original, &limits).expect("record at bound");
    assert_same(&original, &moved::decode(encoded.expose(), &limits).expect("record at bound decodes"));
}

#[test]
fn a_record_one_past_its_runtime_bound_is_refused() {
    let original = fixture(true);
    let mut limits = record_limits(&original);
    let encoded = moved::encode(&original, &limits).expect("fixture");
    limits.record_bytes = limits.record_bytes.checked_sub(1).expect("nonzero record");
    assert_eq!(moved::encode(&original, &limits).err(), Some(Unread::Record));
    refused(encoded.expose(), &limits, Unread::TooLarge, "record one past");
}

// Replace a whole part, retaining the following fields and their boundaries.
fn replace_part(input: &[u8], part: usize, replacement: &[u8]) -> Box<[u8]> {
    let mut reader = Reader::new(input);
    let version = reader.u16().expect("version");
    let replaced_length = Reader::new(&input[prefix(input, part)..]).u32().expect("replaced part length");
    let size = input
        .len()
        .checked_sub(usize::try_from(replaced_length).expect("length fits"))
        .expect("replaced part fits input")
        .checked_add(replacement.len())
        .expect("replacement capacity");
    let mut out = Writer::new(size);
    out.put(&version.to_be_bytes()).expect("room");
    for index in 0..PARTS {
        let len = reader.u32().expect("part length");
        let field = reader.bytes(len).expect("part bytes");
        let field = if index == part { replacement } else { field };
        out.put(&u32::try_from(field.len()).expect("length").to_be_bytes()).expect("room");
        out.put(field).expect("room");
    }
    out.finish()
}

#[test]
fn decode_refuses_every_empty_required_part() {
    let valid = moved::encode(&fixture(false), &limits()).expect("fixture");
    for part in [0, 1, 2, 3, 5] {
        let changed = replace_part(valid.expose(), part, b"");
        refused(&changed, &limits(), Unread::Malformed, "empty required part");
    }
}

#[test]
fn encode_refuses_every_empty_required_outer_part() {
    for part in 0..4 {
        let mut original = fixture(false);
        *outer_field(&mut original, part) = bytes::copy_of(b"");
        assert_eq!(moved::encode(&original, &limits()).err(), Some(Unread::Malformed));
    }
}

#[test]
fn an_empty_display_is_accepted() {
    let original = fixture(false);
    let encoded = moved::encode(&original, &limits()).expect("empty display allowed");
    let decoded = moved::decode(encoded.expose(), &limits()).expect("empty display allowed");
    assert!(decoded.display.is_empty());
}

#[test]
fn an_invalid_nested_record_is_refused_by_name() {
    let valid = moved::encode(&fixture(true), &limits()).expect("fixture");
    refused(&replace_part(valid.expose(), 5, b"invalid record"), &limits(), Unread::Record, "nested record");
}

#[test]
fn encode_refuses_a_record_skein_will_not_encode() {
    let mut original = fixture(true);
    original.record.access_token = bytes::copy_of(b"invalid bearer!");
    assert_eq!(moved::encode(&original, &limits()).err(), Some(Unread::Record));
}

#[test]
fn bound_includes_four_more_bytes_than_the_generated_form() {
    let mut original = fixture(true);
    for (part, (_name, cap)) in OUTER_CAPS.into_iter().enumerate() {
        *outer_field(&mut original, part) = filled(cap);
    }
    let limits = record_limits(&original);
    let encoded = moved::encode(&original, &limits).expect("all parts at bounds");
    assert_eq!(moved::bound(&limits), u64::try_from(encoded.len()).expect("size").checked_add(4).expect("old magic"));
    assert_same(&original, &moved::decode(encoded.expose(), &limits).expect("largest file decodes"));
}

#[test]
fn a_runtime_allowance_above_the_schema_covers_old_files() {
    let limits = Limits { record_bytes: u32::MAX, ..limits() };
    assert_eq!(
        moved::bound(&limits),
        30_u64
            + u64::from(moved::APPLICATION)
            + u64::from(moved::ISSUER)
            + u64::from(moved::CLIENT_ID)
            + u64::from(SUBJECT)
            + u64::from(DISPLAY)
            + u64::from(u32::MAX)
    );
}

// Kept bytes transcribed from jig 365afd554^'s outer encoder and skein
// cad332f2 record.rs: big-endian header, key/generation/expiry, three
// u32-length fields, four marked fields and marked wall for v3. V2 ends
// after metadata. The full row is R5's 109-byte literal; absent refresh
// and metadata are zero u32 lengths, not option marks. No current encoder
// produces these fixtures.
pub(crate) const SKOT_FULL: &[u8] = b"JIGM\x00\x01\x00\x00\x00\x01a\x00\x00\x00\x01i\x00\x00\x00\x01c\x00\x00\x00\x01s\x00\x00\x00\x01d\x00\x00\x00JSKOT\x00\x03\x00\x00\x00\x07\x00\x00\x00\x00\x00\x00\x00\x09\x00\x00\x00\x00\x00\x00\x00{\x00\x00\x00\x01a\x00\x00\x00\x01r\x00\x00\x00\x01m\x01\x00\x00\x00\x01c\x01\x00\x00\x00\x01g\x01\x00\x00\x00\x01s\x01\x00\x00\x00\x01i\x01\x00\x00\x00\x00\x00\x00\x00d";
const SKOT_ABSENT: &[u8] = b"JIGM\x00\x01\x00\x00\x00\x01a\x00\x00\x00\x01i\x00\x00\x00\x01c\x00\x00\x00\x01s\x00\x00\x00\x01d\x00\x00\x00,SKOT\x00\x03\x00\x00\x00\x07\x00\x00\x00\x00\x00\x00\x00\x09\x00\x00\x00\x00\x00\x00\x00{\x00\x00\x00\x01a\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
const SKOT_TWO: &[u8] = b"JIGM\x00\x01\x00\x00\x00\x01a\x00\x00\x00\x01i\x00\x00\x00\x01c\x00\x00\x00\x01s\x00\x00\x00\x01d\x00\x00\x00)SKOT\x00\x02\x00\x00\x00\x07\x00\x00\x00\x00\x00\x00\x00\x09\x00\x00\x00\x00\x00\x00\x00{\x00\x00\x00\x01a\x00\x00\x00\x01r\x00\x00\x00\x01m";

fn skot_fixture(version: u16, full: bool) -> Moved {
    let mut value = fixture(false);
    value.display = bytes::copy_of(b"d");
    if full {
        value.record.refresh_token = Some(bytes::copy_of(b"r"));
        value.record.metadata = Some(bytes::copy_of(b"m"));
        if version == 3 {
            value.record.issued_client_id = Some(bytes::copy_of(b"c"));
            value.record.granted_scopes = Some(bytes::copy_of(b"g"));
            value.record.subject = Some(bytes::copy_of(b"s"));
            value.record.id_token = Some(bytes::copy_of(b"i"));
            value.record.earliest_refresh_at = Some(Wall::from_nanos(100));
        }
    }
    value
}

#[test]
fn skot_literals_decode_every_field_and_convert_to_schema_records() {
    for (input, version, full) in [(SKOT_FULL, 3, true), (SKOT_ABSENT, 3, false), (SKOT_TWO, 2, true)] {
        let decoded = moved::decode(input, &limits()).expect("JIGM/SKOT");
        assert_same(&skot_fixture(version, full), &decoded);
        let encoded = moved::encode(&decoded, &limits()).expect("schema export");
        assert_eq!(&encoded.expose()[..2], b"\0\x01");
        assert_same(&decoded, &moved::decode(encoded.expose(), &limits()).expect("schema record"));
    }
}

// Six outer parts; retain the nested record's bytes for corruptions.
fn skot_record(input: &[u8]) -> &[u8] {
    let offset = MAGIC_BYTES.checked_add(prefix(&input[MAGIC_BYTES..], 5)).expect("record prefix");
    &input[offset.checked_add(4).expect("record payload")..]
}

#[test]
fn skot_record_corruption_is_refused_inside_the_envelope() {
    const SKOT_HEADER: usize = 6; // four magic bytes and u16 version
    const PRESENCE: [usize; 5] = [41, 47, 53, 59, 65]; // three L(1), then four marked L(1)
    let record = skot_record(SKOT_FULL);
    for end in MAGIC_BYTES..record.len() {
        refused(&old_part(5, &record[..end]), &limits(), Unread::Record, "nested truncation");
    }
    refused(&old_part(5, &append_byte(record)), &limits(), Unread::Record, "nested tail");
    for version in [1_u16, 4] {
        let mut changed = bytes::copy_of(record);
        for (target, byte) in changed[MAGIC_BYTES..SKOT_HEADER].iter_mut().zip(version.to_be_bytes()) {
            *target = byte;
        }
        refused(&old_part(5, &changed), &limits(), Unread::Record, "nested version");
    }
    for offset in PRESENCE {
        let mut changed = bytes::copy_of(record);
        changed[offset] = 2;
        refused(&old_part(5, &changed), &limits(), Unread::Record, "invalid presence");
    }
}

#[test]
fn skot_fields_accept_their_bounds() {
    // Each field of R5's full literal is one byte, all seven caps at one.
    let bounded = Limits {
        token_bytes: 1,
        issued_client_bytes: 1,
        granted_bytes: 1,
        subject_bytes: 1,
        id_token_bytes: 1,
        ..limits()
    };
    assert_same(&skot_fixture(3, true), &moved::decode(SKOT_FULL, &bounded).expect("fields at bounds"));
}

#[test]
fn skot_fields_and_current_record_validation_refuse_overflow() {
    const FIELD_PREFIXES: [usize; 7] = [26, 31, 36, 42, 48, 54, 60]; // fixed header, three L(1), four marked L(1)
    const FIELD_PAYLOADS: [usize; 3] = [30, 35, 40]; // header 26, then three L(1)
    const OVER_CAP: u32 = 129; // one past the largest test cap, token_bytes 128
    const V2_RECORD_BYTES: u32 = 41; // 38 fixed bytes plus a, r and m
    for bounded in [
        Limits { token_bytes: 0, ..limits() },
        Limits { issued_client_bytes: 0, ..limits() },
        Limits { granted_bytes: 0, ..limits() },
        Limits { subject_bytes: 0, ..limits() },
        Limits { id_token_bytes: 0, ..limits() },
    ] {
        refused(SKOT_FULL, &bounded, Unread::Record, "field past limit");
    }
    // Each length is refused before attempting its oversized payload.

    for offset in FIELD_PREFIXES {
        let mut changed = bytes::copy_of(skot_record(SKOT_FULL));
        let end = offset.checked_add(4).expect("prefix end");
        for (target, byte) in changed[offset..end].iter_mut().zip(OVER_CAP.to_be_bytes()) {
            *target = byte;
        }
        refused(&old_part(5, &changed), &limits(), Unread::Record, "oversized nested prefix");
    }
    // V2 is 41 bytes; the equivalent schema record is 44. Current size
    // validation still applies even when the old record fits its allowance.
    refused(SKOT_TWO, &Limits { record_bytes: V2_RECORD_BYTES, ..limits() }, Unread::Record, "current encoded size");
    // The old reader permits any field bytes; the current codec must still
    // refuse a malformed bearer, refresh text and metadata.

    for offset in FIELD_PAYLOADS {
        let mut changed = bytes::copy_of(skot_record(SKOT_FULL));
        changed[offset] = 0xff;
        refused(&old_part(5, &changed), &limits(), Unread::Record, "current semantic validation");
    }
}
