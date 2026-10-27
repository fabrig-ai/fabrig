#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "step tests read files and assemble expected text with ordinary Rust"
)]
#![expect(clippy::arithmetic_side_effects, reason = "test fixture offsets and counts are small")]
use super::{
    HEADING_BYTES, Kind, LINE_BYTES, NAME_BYTES, Page, Refused, SUMMARY_BYTES, TITLE_BYTES, TOPIC_BYTES, TOPICS,
    TOPICS_MAX, Topic, WORD_BYTES, find, page, search,
};
use alloc::{string::String, vec::Vec};
use std::{fs, path::Path};

fn head(text: &str) -> (&str, &str, String) {
    let mut paragraphs = text.split("\n\n");
    let title = paragraphs.next().expect("title").strip_prefix("# ").expect("title prefix");
    let kind = paragraphs.next().expect("kind").strip_prefix("Kind: ").expect("kind prefix");
    let summary = paragraphs.next().expect("summary").lines().collect::<Vec<_>>().join(" ");
    (title, kind, summary)
}

fn sections(topic: &Topic) -> Vec<(&str, usize, &str)> {
    let mut starts = Vec::new();
    starts.push((topic.title, 0_usize));
    let mut fenced = false;
    let mut offset = 0;
    for line in topic.text.split_inclusive('\n') {
        if line.starts_with("```") {
            fenced = !fenced;
        } else if !fenced && let Some(heading) = line.strip_prefix("## ") {
            starts.push((heading.trim_end_matches('\n'), offset));
        }
        offset += line.len();
    }
    let mut out = Vec::new();
    for (i, &(heading, start)) in starts.iter().enumerate() {
        let end = starts.get(i + 1).map_or(topic.text.len(), |&(_, at)| at);
        out.push((heading, start, &topic.text[start..end]));
    }
    out
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the step test locates the topics through CARGO_MANIFEST_DIR")]
fn the_table_is_the_directory() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/help");
    let mut names = Vec::new();
    for entry in fs::read_dir(directory).expect("guide directory") {
        let path = entry.expect("guide file").path();
        if path.extension().is_some_and(|extension| extension == "md") {
            let name: String = path.file_stem().expect("topic stem").to_str().expect("ASCII name").into();
            names.push(name);
            let topic = find(path.file_stem().expect("topic stem").as_encoded_bytes()).expect("file in table");
            assert_eq!(topic.text.as_bytes(), fs::read(&path).expect("topic bytes"), "{}: file bytes", topic.name);
        }
    }
    names.sort();
    let mut table_names = Vec::new();
    let mut previous = None;
    for topic in &TOPICS {
        table_names.push(String::from(topic.name));
        let kind: u8 = match topic.kind {
            Kind::Walk => 0,
            Kind::Concept => 1,
            Kind::Recipe => 2,
            Kind::Limits => 3,
        };
        let key = (kind, topic.name);
        if let Some(previous) = previous {
            assert!(previous < key, "{}: kind and name order", topic.name);
        }
        previous = Some(key);
    }
    table_names.sort();
    assert_eq!(names, table_names, "table names equal the directory");
}

#[test]
fn each_topic_is_within_its_bounds() {
    assert!(TOPICS.len() <= TOPICS_MAX, "table: TOPICS_MAX");
    for topic in &TOPICS {
        let name = topic.name;
        assert!(!name.is_empty() && name.len() <= NAME_BYTES, "{name}: NAME_BYTES");
        assert!(
            name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
            "{name}: name characters"
        );
        assert!(topic.text.len() <= TOPIC_BYTES, "{name}: TOPIC_BYTES");
        assert!(
            topic.text.bytes().all(|b| b == b'\n' || (0x20..=0x7e).contains(&b)),
            "{name}: printable ASCII and line ends"
        );
        let (title, kind, summary) = head(topic.text);
        assert_eq!(title, topic.title, "{name}: title line");
        assert!(title.len() <= TITLE_BYTES, "{name}: TITLE_BYTES");
        assert_eq!(kind, topic.kind.word(), "{name}: kind line");
        assert_eq!(summary, topic.summary, "{name}: summary paragraph");
        assert!(summary.len() <= SUMMARY_BYTES, "{name}: SUMMARY_BYTES");
        assert!(summary.ends_with('.'), "{name}: summary ends in a period");
        for (heading, _, _) in sections(topic) {
            assert!(heading.len() <= HEADING_BYTES, "{name}: HEADING_BYTES: {heading}");
        }
    }
}

#[test]
fn each_link_names_a_topic() {
    for topic in &TOPICS {
        for after in topic.text.split("temper help ").skip(1) {
            let name = after
                .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'))
                .next()
                .expect("link name");
            if name != "--search" {
                assert!(find(name.as_bytes()).is_some(), "{}: link {name}", topic.name);
            }
        }
    }
}

#[test]
fn find_requires_the_exact_name() {
    assert_eq!(find(b"box").expect("box topic").name, "box");
    for name in [b"Box".as_slice(), b"box ", b"nope", b""] {
        assert!(find(name).is_none(), "{name:?}: no exact topic");
    }
    assert_eq!(
        [Kind::Walk.word(), Kind::Concept.word(), Kind::Recipe.word(), Kind::Limits.word()],
        ["walk", "concept", "recipe", "limits"]
    );
}

const fn fixture(text: &'static str) -> Topic {
    Topic { name: "fixture", kind: Kind::Walk, title: "Fixture", summary: "Fixture.", text }
}

#[test]
fn page_stops_at_the_last_line_end_that_fits() {
    static TOPIC: Topic = fixture("ab\nc\nlonger\n");
    assert_eq!(page(&TOPIC, 0, 10), Page { text: "ab\nc\n", next: Some(5) });
}

#[test]
fn page_cuts_a_long_line_then_continues_at_its_next_offset() {
    static TOPIC: Topic = fixture("abcdefgh\nx\n");
    let first = page(&TOPIC, 0, 5);
    assert_eq!(first, Page { text: "abcde", next: Some(5) });
    assert_eq!(page(&TOPIC, first.next.expect("next page"), 5), Page { text: "fgh\n", next: Some(9) });
}

#[test]
fn page_counts_quotes_backslashes_and_line_ends_twice() {
    static TOPIC: Topic = fixture("\"\\\"\\\nend");
    assert_eq!(page(&TOPIC, 0, 5), Page { text: "\"\\", next: Some(2) });
    assert_eq!(page(&TOPIC, 2, 6), Page { text: "\"\\\n", next: Some(5) });
}

#[test]
fn page_returns_the_whole_rest_when_it_fits_without_a_final_line_end() {
    static TOPIC: Topic = fixture("a\nb");
    assert_eq!(page(&TOPIC, 0, 4), Page { text: "a\nb", next: None });
}

#[test]
fn page_at_or_past_the_end_is_empty() {
    static TOPIC: Topic = fixture("x\n");
    for from in [2, 3, u32::MAX] {
        assert_eq!(page(&TOPIC, from, 2), Page { text: "", next: None });
    }
}

#[test]
#[should_panic(expected = "a page must fit every rendered byte")]
fn page_requires_room_for_every_rendered_byte() {
    static TOPIC: Topic = fixture("x");
    let _page = page(&TOPIC, 0, 1);
}

#[test]
fn following_pages_reproduces_a_real_topic() {
    let topic = find(b"box").expect("box");
    let mut text = String::new();
    let mut from = 0;
    loop {
        let found = page(topic, from, 77);
        text.push_str(found.text);
        match found.next {
            Some(next) => {
                assert!(next > from, "page advances");
                from = next;
            }
            None => break,
        }
    }
    assert_eq!(text, topic.text, "all topic bytes");
}

static SEARCH: [Topic; 2] = [
    Topic {
        name: "z-first",
        title: "First title",
        ..fixture(
            "# First title\n\nAlpha beta head.\n\n## One\nOnly ALPHA.\n\n## Two\nBeta first.\nalpha first line.\nALPHA later.\n```sh\n## Pretend\nbeta inside fence\n```\n## Three\nalpha beta last.\n",
        )
    },
    Topic {
        name: "a-second",
        title: "Second title",
        ..fixture("# Second title\n\nAlpha beta head.\n## Last\nalpha beta section.\n")
    },
];

#[test]
fn search_requires_every_word_case_folded_in_each_section_in_table_order() {
    let found = search(&SEARCH, b" \tALpha\nbEtA\r ", 20).expect("search words");
    let mut got = Vec::new();
    for hit in found.hits.as_slice() {
        got.push((hit.topic, hit.heading));
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
    for topic in &SEARCH {
        for (heading, offset, _) in sections(topic) {
            for hit in found.hits.as_slice() {
                if hit.topic == topic.name && hit.heading == heading {
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
    static LONG: [Topic; 1] = [fixture(concat!(
        "# Fixture\n",
        "alpha ",
        "012345678901234567890123456789012345678901234567890123456789",
        "012345678901234567890123456789012345678901234567890123456789",
        "tail\n"
    ))];
    let found = search(&LONG, b"alpha", 1).expect("words");
    let line = found.hits.as_slice()[0].line;
    assert_eq!(line.len(), LINE_BYTES, "line is cut to LINE_BYTES");
    assert_eq!(line, &LONG[0].text.lines().nth(1).expect("long line")[..LINE_BYTES]);
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
