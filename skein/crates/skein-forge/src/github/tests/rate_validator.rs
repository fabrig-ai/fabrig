use super::{Head, Wall, api, header, limits};

const RATE_HEADERS: [(&[u8], &[u8]); 3] =
    [(b"x-ratelimit-limit", b"5000"), (b"x-ratelimit-remaining", b"4999"), (b"x-ratelimit-reset", b"1792483260")];

#[test]
fn github_rate_maps_all_three_headers() {
    for order in [[0, 1, 2], [2, 0, 1], [1, 2, 0]] {
        let mut head = Head::new();
        for index in order {
            let (name, value) = RATE_HEADERS[index];
            header::field(&mut head, name, value, &limits());
        }
        assert_eq!(
            header::rate(&head),
            Some(api::Rate { limit: 5000, remaining: 4999, reset_at: Wall::from_nanos(1_792_483_260_000_000_000) })
        );
    }
}

#[test]
fn rate_extraction_requires_three_valid_numeric_header_values() {
    skein_rows::check(
        [
            ("github_rate_is_none_when_one_header_is_absent: missing 0", Box::from([RATE_HEADERS[1], RATE_HEADERS[2]])),
            ("github_rate_is_none_when_one_header_is_absent: missing 1", Box::from([RATE_HEADERS[0], RATE_HEADERS[2]])),
            ("github_rate_is_none_when_one_header_is_absent: missing 2", Box::from([RATE_HEADERS[0], RATE_HEADERS[1]])),
            (
                "github_rate_is_none_when_one_header_is_malformed: malformed 0",
                Box::<[(&[u8], &[u8])]>::from([
                    RATE_HEADERS[0],
                    RATE_HEADERS[1],
                    RATE_HEADERS[2],
                    (RATE_HEADERS[0].0, b"invalid".as_slice()),
                ]),
            ),
            (
                "github_rate_is_none_when_one_header_is_malformed: malformed 1",
                Box::<[(&[u8], &[u8])]>::from([
                    RATE_HEADERS[0],
                    RATE_HEADERS[1],
                    RATE_HEADERS[2],
                    (RATE_HEADERS[1].0, b"invalid".as_slice()),
                ]),
            ),
            (
                "github_rate_is_none_when_one_header_is_malformed: malformed 2",
                Box::<[(&[u8], &[u8])]>::from([
                    RATE_HEADERS[0],
                    RATE_HEADERS[1],
                    RATE_HEADERS[2],
                    (RATE_HEADERS[2].0, b"invalid".as_slice()),
                ]),
            ),
        ],
        |fields| {
            let mut head = Head::new();
            for (name, value) in fields {
                header::field(&mut head, name, value, &limits());
            }
            assert_eq!(header::rate(&head), None);
        },
    );
}

#[test]
fn github_rate_is_none_for_a_jwt_call_head() {
    let mut head = Head::new();
    header::field(&mut head, b"Date", b"Tue, 20 Oct 2026 08:00:00 GMT", &limits());
    assert_eq!(header::rate(&head), None);
    assert_eq!(header::validator(&head), None);
}

#[test]
fn printable_bounded_etags_retain_their_complete_original_spelling() {
    skein_rows::check(
        [
            ("github_validator_maps_a_printable_etag: ordinary", b"\"example-app\"".as_slice()),
            ("github_validator_maps_a_printable_etag: bound", &[b'a'; api::VALIDATOR_BYTES][..]),
            ("github_validator_keeps_a_weak_etag_as_sent", b"W/\"example-app\"".as_slice()),
        ],
        |bytes| {
            let mut head = Head::new();
            header::field(&mut head, b"ETag", bytes, &limits());
            assert_eq!(header::validator(&head).expect("printable bounded tag").as_bytes(), bytes);
        },
    );
}

#[test]
#[expect(clippy::manual_map, reason = "the workspace subset uses explicit matches instead of Option::map")]
fn invalid_etags_never_become_usable_conditional_request_validators() {
    skein_rows::check(
        [
            (
                "github_validator_drops_a_129_byte_etag",
                (
                    Box::<[(&[u8], &[u8])]>::from([
                        (b"ETag".as_slice(), b"\"example-app\"".as_slice()),
                        (b"ETag".as_slice(), &[b'a'; 129][..]),
                    ]),
                    None,
                ),
            ),
            (
                "github_validator_drops_empty_and_nonprintable_etags: empty",
                (Box::<[(&[u8], &[u8])]>::from([(b"ETag".as_slice(), b"".as_slice())]), Some(b"".as_slice())),
            ),
            (
                "github_validator_drops_empty_and_nonprintable_etags: line break",
                (Box::<[(&[u8], &[u8])]>::from([(b"ETag".as_slice(), b"\"example-app\"\r\n".as_slice())]), None),
            ),
            (
                "github_validator_drops_empty_and_nonprintable_etags: control",
                (Box::<[(&[u8], &[u8])]>::from([(b"ETag".as_slice(), b"\x1f".as_slice())]), None),
            ),
            (
                "github_validator_drops_empty_and_nonprintable_etags: delete",
                (Box::<[(&[u8], &[u8])]>::from([(b"ETag".as_slice(), b"\x7f".as_slice())]), None),
            ),
            (
                "github_validator_drops_empty_and_nonprintable_etags: nonascii",
                (Box::<[(&[u8], &[u8])]>::from([(b"ETag".as_slice(), b"\x80".as_slice())]), None),
            ),
        ],
        |(fields, expected_etag)| {
            let mut head = Head::new();
            for (name, value) in fields {
                header::field(&mut head, name, value, &limits());
            }
            let etag = match &head.etag {
                Some(etag) => Some(etag.as_bytes()),
                None => None,
            };
            assert_eq!(etag, expected_etag);
            assert_eq!(header::validator(&head), None);
        },
    );
}
