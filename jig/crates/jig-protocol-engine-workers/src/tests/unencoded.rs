use crate::Unencoded;
use skein_codec::Reason;

#[test]
fn unencoded_words_name_each_codec_reason() {
    for (reason, expected) in [
        (Reason::Short, b"assignment cannot encode: Message.words too short".as_slice()),
        (Reason::Bound, b"assignment cannot encode: Message.words too large"),
        (Reason::Tag, b"assignment cannot encode: Message.words bad tag"),
        (Reason::Bool, b"assignment cannot encode: Message.words bad boolean"),
        (Reason::Utf8, b"assignment cannot encode: Message.words not UTF-8"),
        (Reason::Version, b"assignment cannot encode: Message.words unknown version"),
        (Reason::Trailing, b"assignment cannot encode: Message.words trailing bytes"),
        (Reason::Malformed, b"assignment cannot encode: Message.words malformed"),
    ] {
        let refusal = Unencoded { reason: Some(reason), ..Unencoded::field("Message.words") };
        assert_eq!(refusal.words(b"assignment").as_ref(), expected);
    }
}

#[test]
fn unencoded_words_group_figures_with_thousands_commas() {
    let refusal = Unencoded {
        field: "charter.fixed_owned",
        reason: None,
        figure: Some(82_884),
        bound: Some(65_536),
        ceiling: Some(67_108_864),
    };
    assert_eq!(
        refusal.words(b"assignment").as_ref(),
        b"assignment cannot encode: charter.fixed_owned 82,884, bound 65,536, wire ceiling 67,108,864"
    );
    for (figure, expected) in [
        (0, b"job cannot encode: job.bytes 0".as_slice()),
        (7, b"job cannot encode: job.bytes 7"),
        (999, b"job cannot encode: job.bytes 999"),
        (1_000, b"job cannot encode: job.bytes 1,000"),
        (1_000_001, b"job cannot encode: job.bytes 1,000,001"),
        (1_234_567_890, b"job cannot encode: job.bytes 1,234,567,890"),
        (u64::MAX, b"job cannot encode: job.bytes 18,446,744,073,709,551,615"),
    ] {
        let refusal = Unencoded { figure: Some(figure), ..Unencoded::field("job.bytes") };
        assert_eq!(refusal.words(b"job").as_ref(), expected);
    }
}

#[test]
fn unencoded_words_leave_absent_parts_out() {
    for (figure, bound, ceiling, expected) in [
        (None, None, None, b"record cannot encode: grant.value".as_slice()),
        (Some(1), None, None, b"record cannot encode: grant.value 1"),
        (None, Some(2), None, b"record cannot encode: grant.value, bound 2"),
        (None, None, Some(3), b"record cannot encode: grant.value, wire ceiling 3"),
        (Some(1), Some(2), None, b"record cannot encode: grant.value 1, bound 2"),
        (Some(1), None, Some(3), b"record cannot encode: grant.value 1, wire ceiling 3"),
        (None, Some(2), Some(3), b"record cannot encode: grant.value, bound 2, wire ceiling 3"),
    ] {
        let refusal = Unencoded { figure, bound, ceiling, ..Unencoded::field("grant.value") };
        assert_eq!(refusal.words(b"record").as_ref(), expected);
    }
    let refusal =
        Unencoded { field: "field", reason: Some(Reason::Bound), figure: Some(1), bound: Some(2), ceiling: Some(3) };
    assert_eq!(refusal.words(b"record").as_ref(), b"record cannot encode: field too large 1, bound 2, wire ceiling 3");
}
