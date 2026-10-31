//! Owner budget and ledger fragments (client/domain/view.md, sections 7 and 10).
//! All accounting and period facts are supplied by the owner; this view copies
//! them into the selected language, without deriving availability or a calendar.
use crate::tree::fragments::proposed_task;
use crate::words::Words;
use crate::words::budget_words::BudgetPhrase;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client::contract::{Budget, Numbers, PoolLedger};
use skein_lib::Wall;

/// Maximum whole budget region: five numbers, the funder and an optional deadline.
#[must_use]
pub const fn budget_nodes() -> u32 {
    9
}

fn text(b: &mut Builder, words: &Words, phrase: BudgetPhrase, holes: &[Hole]) {
    let text = words
        .book()
        .say(words.language(), phrase.id(), holes)
        .expect("checked budget phrase and typed owner facts")
        .text;
    b.text(Run::Plain, &text);
}
fn numbers(b: &mut Builder, value: &Numbers, left: u64, words: &Words) {
    text(b, words, BudgetPhrase::Budget, &[Hole::Amount(value.budget)]);
    text(b, words, BudgetPhrase::Spent, &[Hole::Amount(value.spent)]);
    text(b, words, BudgetPhrase::SpentBelow, &[Hole::Amount(value.spent_below)]);
    text(b, words, BudgetPhrase::Reserved, &[Hole::Amount(value.reserved)]);
    text(b, words, BudgetPhrase::Left, &[Hole::Amount(left)]);
}
fn time(b: &mut Builder, words: &Words, phrase: BudgetPhrase, at: Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}

/// Render one exact budget with the caller's normal selected Words.
pub fn render(b: &mut Builder, value: &Budget, words: &Words) {
    b.open(Role::Region);
    let label = words
        .book()
        .say(words.language(), BudgetPhrase::Budget.id(), &[Hole::Amount(value.numbers.budget)])
        .expect("checked budget region label")
        .text;
    b.name(&label);
    numbers(b, &value.numbers, value.left, words);
    text(b, words, BudgetPhrase::Funder, &[]);
    proposed_task::funder(b, value.funder, words);
    if let Some(at) = value.deadline {
        time(b, words, BudgetPhrase::Deadline, at);
    }
    b.close();
}
/// Render one exact ledger, retaining its own source/parent period identities.
pub fn ledger(b: &mut Builder, value: &PoolLedger, words: &Words) {
    b.open(Role::Region);
    let label =
        words.book().say(words.language(), BudgetPhrase::Source.id(), &[]).expect("checked ledger region label").text;
    b.name(&label);
    numbers(b, &value.numbers, value.left, words);
    text(b, words, BudgetPhrase::Source, &[]);
    proposed_task::funder(b, value.source, words);
    if let Some(parent) = value.parent {
        text(b, words, BudgetPhrase::Parent, &[]);
        proposed_task::funder(b, parent, words);
    }
    text(b, words, if value.closed { BudgetPhrase::Closed } else { BudgetPhrase::Open }, &[]);
    if let Some(at) = value.began {
        time(b, words, BudgetPhrase::Began, at);
    }
    if let Some(at) = value.next {
        time(b, words, BudgetPhrase::Next, at);
    }
    b.close();
}
