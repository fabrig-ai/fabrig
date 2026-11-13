use super::fixtures;

#[test]
fn the_browser_words_fit_the_views_limits() {
    assert!(crate::Limits::of(&fixtures::domain_limits(), &fixtures::browser_words()).is_some());
}

// A second source lets the deciding-source test keep a later needing source.

// Pat in fixture_me.
