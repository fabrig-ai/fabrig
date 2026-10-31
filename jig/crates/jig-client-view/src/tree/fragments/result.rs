//! Result metadata fragments (client/domain/objects.md, 8; view.md, 7 and 10).
//! Linked chips keep their supplied lifecycle; absent facts acquire no defaults.
use crate::JigBindings;
use crate::words::Words;
use crate::words::result_words::ResultPhrase;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::Borrowed;
use jig_client::contract::{Proposer, Requester, ResultDetails, TaskLink, TaskStatus};

fn text(b: &mut Builder, words: &Words, phrase: ResultPhrase, holes: &[Hole]) {
    let saying = words.book().say(words.language(), phrase.id(), holes).expect("result words and typed holes checked");
    b.text(Run::Plain, &saying.text);
}
pub(crate) fn requester(b: &mut Builder, value: Requester, words: &Words) {
    match value {
        Requester::Task(number) => text(b, words, ResultPhrase::RequesterTask, &[Hole::Number(number)]),
        Requester::Person(number) => text(b, words, ResultPhrase::RequesterPerson, &[Hole::Number(number)]),
        Requester::Deployment { project } => {
            text(b, words, ResultPhrase::RequesterDeployment, &[Hole::Number(u64::from(project))]);
        }
    }
}
pub(crate) fn link(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &TaskLink,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match value {
        TaskLink::Live(chip) => {
            text(b, words, ResultPhrase::Live, &[]);
            crate::tree::cards::chip::chip(b, client, chip, words, bindings);
        }
        TaskLink::Retained { chip, at } => {
            text(b, words, ResultPhrase::Retained, &[]);
            crate::tree::cards::chip::chip(b, client, chip, words, bindings);
            match at {
                Some(at) => text(b, words, ResultPhrase::At, &[Hole::Wall(words.adjusted_wall(*at))]),
                None => text(b, words, ResultPhrase::AtUnknown, &[]),
            }
        }
        TaskLink::Stored { number, status } => {
            text(b, words, ResultPhrase::Stored, &[Hole::Number(*number)]);
            let phrase = match status {
                Some(TaskStatus::Done) => ResultPhrase::Done,
                Some(TaskStatus::Failed) => ResultPhrase::Failed,
                Some(TaskStatus::Cancelled) => ResultPhrase::Cancelled,
                None => ResultPhrase::StatusUnknown,
            };
            text(b, words, phrase, &[]);
        }
    }
}
/// Render only supplied metadata. Raw connector references retain their owner meaning.
pub fn render(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &ResultDetails,
    words: &Words,
    bindings: &mut JigBindings,
) {
    b.open(Role::Region);
    let saying = words.book().say(words.language(), ResultPhrase::Details.id(), &[]).expect("result heading checked");
    b.name(&saying.text);
    text(b, words, ResultPhrase::Contract, &[]);
    crate::tree::fragments::proposed_task::contract(b, &value.contract, words, bindings);
    requester(b, value.requester, words);
    crate::tree::fragments::proposed_task::executor(b, value.executor, words);
    match value.chosen_by {
        Some(person) => text(b, words, ResultPhrase::Chooser, &[Hole::Number(person)]),
        None => text(b, words, ResultPhrase::ChooserAbsent, &[]),
    }
    text(b, words, ResultPhrase::Delegates, &[Hole::Number(u64::from(value.delegates.len()))]);
    for delegate in &value.delegates {
        link(b, client, delegate, words, bindings);
    }
    match &value.followups {
        Some(followups) => {
            text(b, words, ResultPhrase::Followups, &[Hole::Number(u64::from(followups.len()))]);
            for followup in followups {
                link(b, client, followup, words, bindings);
            }
        }
        None => text(b, words, ResultPhrase::FollowupsAbsent, &[]),
    }
    match value.proposal {
        Some(key) => match key.proposer {
            Proposer::Task(task) => {
                text(b, words, ResultPhrase::ProposalTask, &[Hole::Number(task), Hole::Number(key.number)]);
            }
            Proposer::Party(party) => {
                text(b, words, ResultPhrase::ProposalParty, &[Hole::Number(party), Hole::Number(key.number)]);
            }
        },
        None => text(b, words, ResultPhrase::ProposalAbsent, &[]),
    }
    if value.cut {
        text(b, words, ResultPhrase::Cut, &[]);
    }
    b.close();
}
/// Configured envelope for both full link lists, contract and scalar metadata.
#[must_use]
pub fn nodes(limits: &crate::Config, client: &jig_client::DomainLimits) -> Option<u32> {
    crate::chip_nodes(limits, client)?
        .checked_add(2)?
        .checked_mul(client.rows)?
        .checked_mul(2)?
        .checked_add(crate::tree::fragments::proposed_task::contract_nodes(limits, client)?)?
        .checked_add(10)
}
/// Reserve saying, escaped opaque byte and linked chip storage before rendering.
#[must_use]
#[cfg(test)]
pub fn limits(view: &crate::Config, client: &jig_client::DomainLimits, words: &Words) -> Option<skein_ui::Limits> {
    let nodes = nodes(view, client)?;
    let phrase = words.jig_capacity()?;
    let text_per_node = phrase.checked_mul(2)?.checked_add(crate::words::literal_segment::max_bytes(client.text)?)?;
    Some(skein_ui::Limits {
        nodes,
        depth: client.rows.checked_add(32)?,
        text_per_node,
        text_bytes: nodes.checked_mul(text_per_node)?,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: nodes,
        slots: nodes,
        markdown_depth: 3,
        scene_marks: 0,
        scene_bytes: 0,
    })
}
