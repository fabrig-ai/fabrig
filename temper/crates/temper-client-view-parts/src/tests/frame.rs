use super::fixtures;

#[test]
fn the_project_chosen_is_the_first_of_me_at_start() {
    let root = fixtures::root_in_project(None);
    assert_eq!(root.pages().project(), Some(fixtures::PROJECT));
}

// A second source lets the deciding-source test keep a later needing source.

// Pat in fixture_me.

#[test]
fn known_sign_in_ways_ignore_unknown_words_and_duplicates() {
    for (content, google) in
        [("", false), ("unknown", false), ("google", true), ("future google google", true), ("Google", false)]
    {
        assert_eq!(
            crate::sign_in::Ways::parse(content, b"https://temper.example.com"),
            crate::sign_in::Ways { google, origin: Some(Box::from(b"https://temper.example.com".as_slice())) },
            "{content}"
        );
    }
    for origin in [b"".as_slice(), b"null", b"file://", b"chrome-extension://x"] {
        assert_eq!(crate::sign_in::Ways::parse("google", origin), crate::sign_in::Ways::NONE);
    }
}
