//! Attempts and held phases (client/domain/contract/README.md, 4; pages.md, 5.2).
use crate::words::{Words, lifecycle_words::LifecyclePhrase as P};
use crate::{Borrowed, Config, JigBindings};
use jig_client::contract::{AttemptLine, HoldLine};
use skein_lib::Wall;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

/// Whole row envelope, including retained result follow-ups and raw parts.
#[must_use]
pub fn max_nodes(limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    32_u32
        .checked_add(crate::tree::fragments::proposed_task::part_nodes(limits, client)?)?
        .checked_add(skein_ui::markdown::max_nodes(client.text)?.checked_mul(2)?)?
        .checked_add(crate::result_nodes(limits, client)?)
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying =
        words.book().say(words.language(), phrase.id(), holes).expect("checked lifecycle words and typed holes");
    b.text(Run::Plain, &saying.text);
}
fn region(b: &mut Builder, words: &Words, phrase: P, task: u64) {
    b.open(Role::Region);
    let caption = words.book().say(words.language(), phrase.id(), &[]).expect("checked lifecycle region name");
    b.name(&caption.text);
    text(b, words, phrase, &[]);
    text(b, words, P::Task, &[Hole::Number(task)]);
}
fn wall(b: &mut Builder, words: &Words, value: Option<Wall>, present: P, absent: P) {
    match value {
        Some(at) => text(b, words, present, &[Hole::Wall(words.adjusted_wall(at))]),
        None => text(b, words, absent, &[]),
    }
}
pub fn attempt(b: &mut Builder, value: &AttemptLine, words: &Words) {
    region(b, words, P::Attempt, value.task);
    text(b, words, P::Number, &[Hole::Number(value.attempt)]);
    wall(b, words, value.claimed, P::Claimed, P::ClaimedAbsent);
    wall(b, words, value.started, P::Started, P::StartedAbsent);
    wall(b, words, value.ended, P::Ended, P::EndedAbsent);
    match value.end {
        Some(end) => super::activity::end(b, end, words),
        None => text(b, words, P::EndAbsent, &[]),
    }
    match value.phases {
        Some(phases) => text(b, words, P::Phases, &[Hole::Number(u64::from(phases))]),
        None => text(b, words, P::PhasesAbsent, &[]),
    }
    b.close();
}
pub fn hold(b: &mut Builder, value: &HoldLine, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    region(b, words, P::Held, value.task);
    text(b, words, P::Position, &[Hole::Number(value.position)]);
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(value.at))]);
    super::chip::prior(b, value.before, words);
    super::chip::reason(b, value.why, words);
    match &value.source {
        Some(source) => super::proposed_task::part(b, source, words, bindings),
        None => text(b, words, P::SourceAbsent, &[]),
    }
    match &value.ending {
        Some(ending) => super::chip::ending(b, ending, client, words, bindings),
        None => text(b, words, P::EndingAbsent, &[]),
    }
    b.close();
}
