//! Retained page fragments (client/domain/pages.md, 5.2 and 6.5; view.md,
//! 7 and 10). The source owns row order, optional clocks and all counters.
//! Brief delegate identity does not manufacture a Result card; a funding
//! record does not supply pools, task rows or an amount left.
use crate::JigBindings;
use crate::words::Words;
use crate::words::historical_pages_words::HistoricalPhrase as P;
use jig_client::words::{DecisionWord as D, Word};
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{Borrowed, Config};
use alloc::boxed::Box;
use jig_client::contract::{ChipPhase, Funder, OwnerEndedPage, OwnerSpendPage};

/// Brief row identity and phase plus the supplied complete ending/result.
#[must_use]
pub fn ended_nodes(l: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    crate::chip_nodes(l, client)?
        .checked_add(crate::result_nodes(l, client)?)?
        .checked_add(skein_ui::markdown::max_nodes(client.text)?)?
        .checked_add(32)?
        .checked_mul(client.rows)?
        .checked_add(2)
}
/// Every four-counter row, both source identities and supplied period clocks.
#[must_use]
pub fn spend_nodes(_l: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    client.rows.checked_mul(18)?.checked_add(2)
}
fn saying(words: &Words, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    words
        .book()
        .say(words.language(), phrase.id(), holes)
        .expect("checked historical words and typed source holes")
        .text
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    b.text(Run::Plain, &saying(words, phrase, holes));
}
fn wall(b: &mut Builder, words: &Words, phrase: P, at: skein_lib::Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}
/// Display admitted rows as a companion to either existing Plan rendering path.
pub fn ended_render(
    b: &mut Builder,
    client: &Borrowed<'_>,
    page: &OwnerEndedPage,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Region);
    b.name(&saying(words, P::EndedDelegates, &[]));
    for row in &page.rows {
        b.open(Role::Region);
        b.name(&saying(words, P::EndedRow, &[Hole::Number(row.chip.task)]));
        {
            let ChipPhase { phase, .. } = row.chip.details.as_ref();
            crate::tree::cards::chip::head(b, &row.chip, words, bindings);
            crate::tree::fragments::chip_phase::brief(b, phase, words);
        }
        crate::tree::fragments::chip::ending(b, &row.ended, client, words, bindings);
        match row.at {
            Some(at) => wall(b, words, P::At, at),
            None => text(b, words, P::AtAbsent, &[]),
        }
        b.close();
    }
    b.close();
}
fn funder_name(value: Funder, words: &Words) -> Box<[u8]> {
    let saying = match value {
        Funder::Task(task) => {
            words.book().say(words.language(), Word::Decision(D::FunderTask).id(), &[Hole::Number(task)])
        }
        Funder::Pool { project, person, period } => words.book().say(
            words.language(),
            Word::Decision(D::FunderPool).id(),
            &[Hole::Number(u64::from(project)), Hole::Number(person), Hole::Number(period)],
        ),
        Funder::Recurring { project, task, period } => words.book().say(
            words.language(),
            Word::Decision(D::FunderRecurring).id(),
            &[Hole::Number(u64::from(project)), Hole::Number(task), Hole::Number(period)],
        ),
        Funder::Period { project, period } => words.book().say(
            words.language(),
            Word::Decision(D::FunderPeriod).id(),
            &[Hole::Number(u64::from(project)), Hole::Number(period)],
        ),
    };
    saying.expect("checked funder identity and supplied source coordinates").text
}
/// All supplied historical counters; no live balance or ledger projection.
pub fn spend_render(b: &mut Builder, page: &OwnerSpendPage, words: &Words) {
    b.open(Role::Region);
    b.name(&saying(words, P::FundingHistory, &[]));
    for row in &page.rows {
        b.open(Role::Region);
        b.name(&funder_name(row.funder, words));
        crate::tree::fragments::proposed_task::funder(b, row.funder, words);
        match row.parent {
            Some(parent) => {
                text(b, words, P::Parent, &[]);
                crate::tree::fragments::proposed_task::funder(b, parent, words);
            }
            None => text(b, words, P::ParentAbsent, &[]),
        }
        for (phrase, amount) in [
            (P::Budget, row.numbers.budget),
            (P::Spent, row.numbers.spent),
            (P::SpentBelow, row.numbers.spent_below),
            (P::Reserved, row.numbers.reserved),
        ] {
            text(b, words, phrase, &[Hole::Amount(amount)]);
        }
        text(b, words, P::Made, &[Hole::Number(u64::from(row.made))]);
        text(b, words, if row.closed { P::Closed } else { P::Open }, &[]);
        match row.period {
            Some(period) => {
                wall(b, words, P::Began, period.began);
                match period.next {
                    Some(next) => wall(b, words, P::Next, next),
                    None => text(b, words, P::NextAbsent, &[]),
                }
            }
            None => text(b, words, P::PeriodAbsent, &[]),
        }
        b.close();
    }
    b.close();
}
