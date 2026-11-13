use super::fixtures;
use jig_client::words::{DecisionWord, Word};
use skein_ui::phrases::Hole;

const FUNDS: u64 = 40;
const LEFT: u64 = 120;
const AFTER: u64 = 80;

#[test]
fn proposal_funding_uses_the_mockups_pool_sentence() {
    let words = fixtures::words();
    assert_eq!(
        words
            .say_word(
                Word::Decision(DecisionWord::Funds),
                &[Hole::Amount(FUNDS), Hole::Amount(LEFT), Hole::Amount(AFTER)]
            )
            .as_ref(),
        b"Funds 40 from your pool \xc2\xb7 120 left, 80 after",
    );
}
