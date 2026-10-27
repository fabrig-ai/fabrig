use super::*;

#[test]
fn find_requires_the_exact_name() {
    assert_eq!(find(b"box").expect("box entry").name, "box");
    for name in [b"Box".as_slice(), b"box ", b"nope", b""] {
        assert!(find(name).is_none(), "{name:?}: no exact entry");
    }
    assert_eq!(KINDS.map(Kind::word), ["index", "concept", "walk", "recipe", "reference", "guide"]);
}

#[test]
fn page_stops_at_the_last_line_end_that_fits() {
    static ENTRY: Page = fixture(Kind::Guide, "ab\nc\nlonger\n");
    assert_eq!(page(&ENTRY, 0, 10), Cut { text: "ab\nc\n", next: Some(5) });
}

#[test]
fn page_cuts_a_long_line_then_continues_at_its_next_offset() {
    static ENTRY: Page = fixture(Kind::Guide, "abcdefgh\nx\n");
    let first = page(&ENTRY, 0, 5);
    assert_eq!(first, Cut { text: "abcde", next: Some(5) });
    assert_eq!(page(&ENTRY, first.next.expect("next page"), 5), Cut { text: "fgh\n", next: Some(9) });
}

#[test]
fn page_cuts_at_character_boundaries_and_counts_non_ascii_bytes_once() {
    static ENTRY: Page = fixture(Kind::Guide, "ab·cd");
    assert_eq!(page(&ENTRY, 0, 3), Cut { text: "ab", next: Some(2) });
    assert_eq!(page(&ENTRY, 0, 4), Cut { text: "ab·", next: Some(4) });
    assert_eq!(page(&ENTRY, 2, 2), Cut { text: "·", next: Some(4) });
}

#[test]
fn page_from_inside_a_character_starts_at_the_next_boundary() {
    static ENTRY: Page = fixture(Kind::Guide, "ab·cd");
    static LAST: Page = fixture(Kind::Guide, "ab·");
    assert_eq!(page(&ENTRY, 3, 2), Cut { text: "cd", next: None });
    assert_eq!(page(&LAST, 3, 2), Cut { text: "", next: None });
}

#[test]
fn page_counts_quotes_backslashes_and_line_ends_twice() {
    static ENTRY: Page = fixture(Kind::Guide, "\"\\\"\\\nend");
    assert_eq!(page(&ENTRY, 0, 5), Cut { text: "\"\\", next: Some(2) });
    assert_eq!(page(&ENTRY, 2, 6), Cut { text: "\"\\\n", next: Some(5) });
}

#[test]
fn page_returns_the_whole_rest_when_it_fits_without_a_final_line_end() {
    static ENTRY: Page = fixture(Kind::Guide, "a\nb");
    assert_eq!(page(&ENTRY, 0, 4), Cut { text: "a\nb", next: None });
}

#[test]
fn page_at_or_past_the_end_is_empty() {
    static ENTRY: Page = fixture(Kind::Guide, "x\n");
    for from in [2, 3, u32::MAX] {
        assert_eq!(page(&ENTRY, from, 2), Cut { text: "", next: None });
    }
}

#[test]
#[should_panic(expected = "a page must fit every rendered byte")]
fn page_requires_room_for_every_rendered_byte() {
    static ENTRY: Page = fixture(Kind::Guide, "x");
    let _page = page(&ENTRY, 0, 1);
}

#[test]
fn following_cuts_reproduces_a_real_page() {
    let entry = find(b"box").expect("box");
    let mut text = String::new();
    let mut from = 0;
    loop {
        let found = page(entry, from, 77);
        text.push_str(found.text);
        match found.next {
            Some(next) => {
                assert!(next > from, "page advances");
                from = next;
            }
            None => break,
        }
    }
    assert_eq!(text, entry.text, "all page bytes");
}

static SEARCH: [Page; 2] = [
    Page {
        name: "z-first",
        title: "First title",
        ..fixture(
            Kind::Guide,
            "# First title\n\nAlpha beta head.\n\n## One\nOnly ALPHA.\n\n## Two\nBeta first.\nalpha first line.\nALPHA later.\n```sh\n## Pretend\nbeta inside fence\n```\n## Three\nalpha beta last.\n",
        )
    },
    Page {
        name: "a-second",
        title: "Second title",
        ..fixture(Kind::Guide, "# Second title\n\nAlpha beta head.\n## Last\nalpha beta section.\n")
    },
];

#[test]
fn search_requires_every_word_case_folded_in_each_section_in_table_order() {
    let found = search(&SEARCH, b" \tALpha\nbEtA\r ", 20).expect("search words");
    let mut got = Vec::new();
    for hit in found.hits.as_slice() {
        got.push((hit.page, hit.heading));
    }
    assert_eq!(
        got,
        [
            ("z-first", "First title"),
            ("z-first", "Two"),
            ("z-first", "Three"),
            ("a-second", "Second title"),
            ("a-second", "Last")
        ]
    );
    assert!(!found.more, "all matches fit");
    for entry in &SEARCH {
        for (heading, offset, _) in sections(entry) {
            for hit in found.hits.as_slice() {
                if hit.page == entry.name && hit.heading == heading {
                    assert_eq!(hit.offset, u32::try_from(offset).expect("small fixture"), "section first byte");
                }
            }
        }
    }
}

#[test]
fn search_takes_the_first_line_with_the_first_word() {
    let found = search(&SEARCH, b"alpha beta", 20).expect("words");
    assert_eq!(found.hits.as_slice()[0].line, "Alpha beta head.");
    assert_eq!(found.hits.as_slice()[1].line, "alpha first line.");
    let reversed = search(&SEARCH, b"beta alpha", 20).expect("words");
    assert_eq!(reversed.hits.as_slice()[1].line, "Beta first.");
}

#[test]
fn search_cuts_the_matching_line_to_its_byte_bound() {
    static LONG: [Page; 1] = [fixture(
        Kind::Guide,
        concat!(
            "# Fixture\n",
            "alpha ",
            "012345678901234567890123456789012345678901234567890123456789",
            "012345678901234567890123456789012345678901234567890123456789",
            "tail\n"
        ),
    )];
    let found = search(&LONG, b"alpha", 1).expect("words");
    let line = found.hits.as_slice()[0].line;
    assert_eq!(line.len(), LINE_BYTES, "line is cut to LINE_BYTES");
    assert_eq!(line, &LONG[0].text.lines().nth(1).expect("long line")[..LINE_BYTES]);
}

#[test]
fn search_cuts_before_a_character_straddling_its_byte_bound() {
    static LONG: [Page; 1] = [fixture(
        Kind::Guide,
        concat!(
            "# Fixture\n",
            "alpha ",
            "012345678901234567890123456789012345678901234567890123456789",
            "01234567890123456789012345678901234567890123456789012",
            "·tail\n"
        ),
    )];
    let source = LONG[0].text.lines().nth(1).expect("long line");
    assert_eq!(source.find('·'), Some(LINE_BYTES - 1), "character crosses byte 120");
    let found = search(&LONG, b"alpha", 1).expect("words");
    assert_eq!(found.hits.as_slice()[0].line, &source[..LINE_BYTES - 1]);
}

#[test]
fn search_sets_more_only_when_another_section_matches() {
    for (limit, count, more) in [(0, 0, true), (1, 1, true), (5, 5, false), (6, 5, false)] {
        let found = search(&SEARCH, b"alpha beta", limit).expect("words");
        assert_eq!((found.hits.len(), found.more), (count, more), "limit {limit}");
    }
    let found = search(&SEARCH, b"absent", 0).expect("words");
    assert_eq!((found.hits.len(), found.more), (0, false), "no matching sections");
}

#[test]
fn a_heading_inside_a_fence_stays_in_its_section() {
    let found = search(&SEARCH, b"Pretend", 20).expect("words");
    assert_eq!(found.hits.len(), 1, "one containing section");
    assert_eq!(found.hits.as_slice()[0].heading, "Two");
}

#[test]
fn search_refuses_no_words_and_words_past_either_bound() {
    for words in [b"".as_slice(), b" \t\r\n"] {
        assert_eq!(search(&SEARCH, words, 1).expect_err("no words"), Refused::NoWords);
    }
    assert!(search(&SEARCH, b"a a a a a a a a", 1).is_ok(), "WORDS words pass");
    assert_eq!(search(&SEARCH, b"a a a a a a a a a", 1).expect_err("nine words"), Refused::TooManyWords);
    assert!(search(&SEARCH, &[b'a'; WORD_BYTES], 1).is_ok(), "WORD_BYTES passes");
    assert_eq!(search(&SEARCH, &[b'a'; WORD_BYTES + 1], 1).expect_err("long word"), Refused::WordTooLong);
    assert_eq!(Refused::NoWords.says(), "nothing to search for");
    assert_eq!(Refused::TooManyWords.says(), "at most 8 words");
    assert_eq!(Refused::WordTooLong.says(), "a word is at most 32 bytes");
}
