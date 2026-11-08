#[path = "../build/commit.rs"]
mod commit;

#[test]
fn the_commit_word_is_twelve_hex_digits_dirty_or_unknown() {
    const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
    for (head, dirty, expected) in [
        (Some(HEAD), false, "0123456789ab"),
        (Some(HEAD), true, "0123456789ab-dirty"),
        (None, false, "unknown"),
        (None, true, "unknown"),
        (Some("0123456789a"), false, "unknown"),
        (Some("0123456789ag"), false, "unknown"),
        (Some("0123456789aB"), false, "unknown"),
        (Some("0123456789abg"), false, "unknown"),
        (Some("0123456789ab"), false, "0123456789ab"),
    ] {
        assert_eq!(commit::word(head, dirty), expected, "head {head:?}, dirty {dirty}");
    }
}

#[test]
fn the_version_line_names_the_binary_its_version_and_commit() {
    let line = super::version();
    let prefix = format!("temper-client {} (", env!("CARGO_PKG_VERSION"));
    let word = line.strip_prefix(&prefix).and_then(|tail| tail.strip_suffix(")\n")).expect("version line");
    let (head, dirty) = match word.strip_suffix("-dirty") {
        Some(head) => (head, true),
        None => (word, false),
    };
    let head = if head == "unknown" { None } else { Some(head) };
    assert_eq!(word, commit::word(head, dirty));
    assert_eq!(line, format!("{prefix}{})\n", env!("TEMPER_CLIENT_COMMIT")));
}
