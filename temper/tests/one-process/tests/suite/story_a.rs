//! Story A in the composed world (temper/docs/design/playground.md, 5.2 and 9).
use skein_world::domain::Verdict;
use temper_one_process_world::{fixtures, world};

#[test]
#[ignore = "Story A stalls at Starting after reading playground-1/admin_token; no project or account appears"]
fn story_a_is_walked_from_the_welcome_to_the_csv_change_landed() {
    let outcome = world::run(fixtures::calm(fixtures::SEED), world::Story::PlaygroundA);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.signed_in, 1);
    assert_eq!(outcome.adopted, 1);
    let playground = outcome.playground.expect("Story A's observations");
    assert!(playground.csv, "main contains the scripted CSV edit");
    assert!(playground.green, "CI is green at main's head");
    assert!(outcome.pushed.iter().any(|push| push.commit == playground.main), "main holds the pushed change");
    assert_eq!(playground.answered, 3, "all three template replies were seen in chat");
    assert!(outcome.drained);
}
