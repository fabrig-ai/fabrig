//! Retained named random regressions, with their complete seeds and heap checks.
use jig_client_world::random::{self, Start};
use skein_world::domain::Verdict;
// Peak 855,962 bytes measured 10-29; testing-strategy.md 6.
const NEWCHAT_HEAP_CEILING: u64 = 1_100_000;
#[test]
fn random_people_smoke_newchat_story_boundaries() {
    let outcome = random::story(71, Start::NewChat, NEWCHAT_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
    assert!(outcome.heap_peak > 0);
    assert!(outcome.heap_peak <= outcome.heap_bound);
}

// Peak 992,032 bytes measured 10-29; testing-strategy.md 6.
const HELD_STORY_HEAP_CEILING: u64 = 1_300_000;
#[test]
fn random_people_smoke_held_story_boundaries() {
    let outcome = random::story(72, Start::Held, HELD_STORY_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
    assert!(outcome.heap_peak > 0);
    assert!(outcome.heap_peak <= outcome.heap_bound);
}

// Peak 555,648 bytes measured 10-29; testing-strategy.md 6.
const RANDOM_WORLD_HEAP_CEILING: u64 = 700_000;
#[test]
fn random_people_smoke_random_world_boundaries() {
    let outcome = random::world(73, RANDOM_WORLD_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
}

// Peak 1,206,464 bytes measured 10-29; testing-strategy.md 6.
const RESTART_RECOVERY_HEAP_CEILING: u64 = 1_600_000;
#[test]
fn a_restart_with_old_due_output_advances_to_recovery_instead_of_spinning() {
    let outcome = random::story(110, Start::NewChat, RESTART_RECOVERY_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.callbacks > 10);
    assert!(outcome.faults > 0);
}

// Peak 940,219 bytes measured 10-29; testing-strategy.md 6.
const SIGNED_OUT_TABS_HEAP_CEILING: u64 = 1_200_000;
#[test]
fn signed_out_tabs_resume_the_link_obligation_only_after_confirmation() {
    let outcome = random::story(107, Start::NewChat, SIGNED_OUT_TABS_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(outcome.faults > 0);
}

// Peak 593,346 bytes measured 10-29; testing-strategy.md 6.
const CURRENT_PAGE_NAVIGATION_HEAP_CEILING: u64 = 750_000;
#[test]
fn selecting_the_current_page_is_still_navigation_for_the_referee() {
    let outcome = random::story(111, Start::Chat, CURRENT_PAGE_NAVIGATION_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
}

// Peak 592,514 bytes measured 10-29; testing-strategy.md 6.
const INBOX_FILTER_HEAP_CEILING: u64 = 750_000;
#[test]
fn a_persons_inbox_filter_changes_visibility_without_deciding_the_proposal() {
    let outcome = random::story(111, Start::Proposal, INBOX_FILTER_HEAP_CEILING);
    assert_eq!(outcome.verdict, Verdict::Passed);
}
