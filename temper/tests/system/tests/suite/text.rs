//! Prompt checks use the linear byte search rather than comparing every window.

pub fn occurrences(text: &[u8], words: &[u8]) -> usize {
    assert!(!words.is_empty(), "a prompt check names nonempty words");
    let mut count = 0;
    let mut from = 0;
    while let Some(at) = skein_lib::bytes::find_from(text, words, from) {
        count += 1;
        // Keep overlapping occurrences, just as a check of every byte window does.
        from = at + 1;
    }
    count
}

#[test]
fn prompt_counts_preserve_absent_repeated_and_overlapping_bytes() {
    for (text, words, expected) in [
        (b"unrelated".as_slice(), b"word".as_slice(), 0),
        (b"wordword", b"word", 2),
        (b"ababa", b"aba", 2),
        (b"\xff\xff\xff", b"\xff\xff", 2),
    ] {
        assert_eq!(occurrences(text, words), expected, "{text:?}: {words:?}");
    }
}
