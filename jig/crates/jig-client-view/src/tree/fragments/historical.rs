//! Passive whole historical objects (client/domain/client.md, 5;
//! contract.md, 6 and view.md, 7, 10). Original identities and independent
//! sources are retained without acquiring current offers or manufacturing context.
use crate::JigBindings;
use crate::words::Words;
use crate::words::historical_words::HistoricalPhrase as P;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{Borrowed, Limits};
use alloc::boxed::Box;
use jig_client::contract::{
    OwnerEscalationDecision, OwnerGoalHolding, OwnerHistorical, OwnerHistoricalValue, OwnerMessageProposalKind,
    OwnerPersonProposalGoal, OwnerPersonProposalState, OwnerProposalChoice, OwnerProposalDecision, OwnerRequester,
    Proposer, Task, Text,
};
use skein_lib::{Id, Wall};

fn saying(words: &Words, phrase: P, holes: &[Hole]) -> Box<[u8]> {
    words.book().say(words.language(), phrase.id(), holes).expect("checked historical sayings and typed holes").text
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    b.text(Run::Plain, &saying(words, phrase, holes));
}
fn label(b: &mut Builder, words: &Words, phrase: P) {
    text(b, words, phrase, &[]);
}
fn number(b: &mut Builder, words: &Words, phrase: P, number: u64) {
    text(b, words, phrase, &[Hole::Number(number)]);
}
fn at(b: &mut Builder, words: &Words, phrase: P, at: Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}
fn raw(b: &mut Builder, value: &Text) {
    crate::words::literal_segment::render(b, value.bytes());
}
fn region(b: &mut Builder, words: &Words, phrase: P) {
    b.open(Role::Region);
    b.name(&saying(words, phrase, &[]));
}
fn kind(b: &mut Builder, value: OwnerMessageProposalKind, words: &Words) {
    label(
        b,
        words,
        match value {
            OwnerMessageProposalKind::Settings => return,
            OwnerMessageProposalKind::Batch => P::BatchKind,
            OwnerMessageProposalKind::Effect => P::EffectKind,
            OwnerMessageProposalKind::Amend => P::AmendKind,
            OwnerMessageProposalKind::Widen => P::WidenKind,
            OwnerMessageProposalKind::Release => P::ReleaseKind,
        },
    );
}
fn goal(b: &mut Builder, value: &OwnerPersonProposalGoal, words: &Words, bindings: &mut JigBindings, rows: u32) {
    let text_limit = bindings.text_limit();
    number(b, words, P::Goal, value.number);
    number(b, words, P::Project, u64::from(value.project));
    crate::tree::fragments::proposed_task::executor(b, value.executor, words);
    crate::tree::fragments::specification::specification(b, &value.spec, words);
    crate::tree::fragments::proposed_task::contract(b, &value.contract, words, bindings);
    crate::tree::fragments::authority::render_words(b, &value.authority, words, text_limit, rows);
    for (phrase, amount) in [
        (P::Budget, value.numbers.budget),
        (P::Spent, value.numbers.spent),
        (P::SpentBelow, value.numbers.spent_below),
        (P::Reserved, value.numbers.reserved),
    ] {
        text(b, words, phrase, &[Hole::Amount(amount)]);
    }
    crate::tree::fragments::proposed_task::funder(b, value.funder, words);
    for task in &value.dependencies {
        number(b, words, P::Dependency, *task);
    }
    for holding in &value.holdings {
        let (connector, path, kind, phrase) = match holding {
            OwnerGoalHolding::Write { connector, path, kind } => (*connector, path, *kind, P::Write),
            OwnerGoalHolding::Slot { connector, path, kind } => (*connector, path, *kind, P::Slot),
        };
        region(b, words, phrase);
        label(b, words, phrase);
        number(b, words, P::Connector, u64::from(connector));
        number(b, words, P::Kind, u64::from(kind));
        crate::words::literal_segment::path(
            b,
            path.as_slice(),
            words,
            crate::words::literal_segment::PathDisplay { text: bindings.text_limit(), reserved: 0, keep_last: true },
        );
        b.close();
    }
    crate::tree::fragments::wake::policy(b, &value.wake, words.book(), words.language());
    match value.tracked {
        Some(number_) => number(b, words, P::Tracked, u64::from(number_)),
        None => label(b, words, P::TrackedAbsent),
    }
}
fn proposal_decision(b: &mut Builder, value: &OwnerProposalDecision, words: &Words) {
    number(b, words, P::Project, u64::from(value.project));
    match value.proposer {
        Proposer::Task(task) => crate::tree::fragments::result::requester(b, OwnerRequester::Task(task), words),
        Proposer::Party(person) => crate::tree::fragments::result::requester(b, OwnerRequester::Person(person), words),
    }
    number(b, words, P::Number, value.proposal);
    kind(b, value.kind, words);
    number(b, words, P::DecisionBy, value.by);
    at(b, words, P::DecisionAt, value.at);
    label(
        b,
        words,
        match value.choice {
            OwnerProposalChoice::Accepted => P::AcceptChoice,
            OwnerProposalChoice::Rejected => P::RejectChoice,
            OwnerProposalChoice::Passed => P::Pass,
            OwnerProposalChoice::Withdrawn => P::Withdrawn,
            OwnerProposalChoice::Stale => P::Stale,
        },
    );
    match value.created {
        Some((first, last)) => {
            number(b, words, P::FirstCreated, first);
            number(b, words, P::LastCreated, last);
        }
        None => label(b, words, P::CreatedAbsent),
    }
}
fn task(b: &mut Builder, client: &Borrowed<'_>, value: &Task, words: &Words, bindings: &mut JigBindings) {
    number(b, words, P::Project, u64::from(value.project));
    crate::tree::cards::chip::chip(b, client, &value.chip, words, bindings);
    crate::tree::sections::task::overview(b, client, value, words, bindings);
    crate::tree::sections::task::budget(b, value, words, bindings, client.limits.rows);
}
fn outer(b: &mut Builder, value: &OwnerHistorical, words: &Words) {
    match value.at {
        Some(time) => at(b, words, P::At, time),
        None => label(b, words, P::AtAbsent),
    }
    match value.by {
        Some(by) => {
            label(b, words, P::By);
            crate::tree::fragments::result::requester(b, by, words);
        }
        None => label(b, words, P::ByAbsent),
    }
}
/// Render each genuine source separately. No current card controls are added.
pub fn render(
    b: &mut Builder,
    client: &Borrowed<'_>,
    value: &OwnerHistorical,
    words: &Words,
    bindings: &mut JigBindings,
) {
    region(b, words, P::Historical);
    outer(b, value, words);
    match &value.value {
        OwnerHistoricalValue::Task { document, result } => {
            label(b, words, P::Task);
            task(b, client, document, words, bindings);
            crate::tree::cards::result::facts(b, client, result, words, bindings);
        }
        OwnerHistoricalValue::Proposal(value) => {
            label(b, words, P::TaskProposal);
            crate::tree::fragments::history::proposal(b, value, words, bindings, client.limits.rows);
        }
        OwnerHistoricalValue::PersonProposal { proposal, decision } => {
            label(b, words, P::PersonProposal);
            number(b, words, P::Number, proposal.number);
            number(b, words, P::Proposer, proposal.proposer);
            number(b, words, P::Project, u64::from(proposal.project));
            goal(b, &proposal.goal, words, bindings, client.limits.rows);
            match &proposal.state {
                OwnerPersonProposalState::Pending { since } => {
                    label(b, words, P::Pending);
                    at(b, words, P::Since, *since);
                }
                OwnerPersonProposalState::Accepted { by } => {
                    label(b, words, P::Accepted);
                    crate::tree::fragments::result::requester(b, *by, words);
                }
                OwnerPersonProposalState::Rejected { by, reason, message, at: time } => {
                    label(b, words, P::Rejected);
                    crate::tree::fragments::result::requester(b, *by, words);
                    raw(b, reason);
                    number(b, words, P::Message, *message);
                    at(b, words, P::DecisionAt, *time);
                }
            }
            match decision {
                Some(value) => proposal_decision(b, value, words),
                None => label(b, words, P::DecisionAbsent),
            }
        }
        OwnerHistoricalValue::Escalation { task: document, decision, hold } => {
            label(b, words, P::Escalation);
            task(b, client, document, words, bindings);
            number(b, words, P::Project, u64::from(decision.project));
            number(b, words, P::Requester, decision.requester);
            number(b, words, P::Number, decision.task);
            number(b, words, P::Revision, decision.revision);
            number(b, words, P::DecisionBy, decision.by);
            at(b, words, P::DecisionAt, decision.at);
            match &decision.decision {
                OwnerEscalationDecision::Release => label(b, words, P::Release),
                OwnerEscalationDecision::Pass => label(b, words, P::Pass),
                OwnerEscalationDecision::Reject { reason } => {
                    label(b, words, P::Rejected);
                    raw(b, reason);
                }
            }
            match hold {
                Some(value) => crate::tree::fragments::milestone::render(b, value, client, words, bindings),
                None => label(b, words, P::HoldAbsent),
            }
        }
        OwnerHistoricalValue::Question { question, answer, ending } => {
            label(b, words, P::Question);
            crate::tree::cards::chip::chip(b, client, &question.task, words, bindings);
            number(b, words, P::Message, question.message);
            if let Some(by) = question.from {
                crate::tree::fragments::result::requester(b, by, words);
            }
            at(b, words, P::Since, question.since);
            crate::tree::fragments::document_text::plain(b, &question.words, words);
            if let Some(how) = &question.ending {
                words.ending(b, how);
            }
            match answer {
                Some(value) => crate::tree::fragments::activity::render(b, value, words),
                None => label(b, words, P::AnswerAbsent),
            }
            match ending {
                Some(value) => crate::tree::cards::result::facts(b, client, value, words, bindings),
                None => label(b, words, P::EndingAbsent),
            }
        }
        OwnerHistoricalValue::PersonTask { document, result } => {
            label(b, words, P::PersonTask);
            crate::tree::fragments::person_task::render(b, client, document, words, bindings);
            crate::tree::cards::result::facts(b, client, result, words, bindings);
        }
    }
    b.close();
}
/// Rich departure facts take precedence over later compact/body refreshes.
pub(crate) fn card(
    b: &mut Builder,
    client: &Borrowed<'_>,
    object: Id<crate::state::objects::Object>,
    place: &crate::tree::cards::Place,
    words: &Words,
    bindings: &mut JigBindings,
) -> bool {
    use crate::state::objects::{Body, Card, Left};
    let Some(fact) = client.objects().get(object) else {
        return false;
    };
    let value = match &fact.card {
        Card::Leaving { how: Left::Historical { facts }, .. } => Some(facts.as_ref()),
        Card::Open
        | Card::Deciding { .. }
        | Card::NotSent { .. }
        | Card::Refused { .. }
        | Card::Unknown { .. }
        | Card::Leaving {
            how: Left::Looking | Left::Answered { .. } | Left::Ended { .. } | Left::OwnerEnded { .. } | Left::Unknown,
            ..
        } => match &fact.body {
            Some(Body::Historical(value)) => Some(value.as_ref()),
            Some(
                Body::Chip(_)
                | Body::Task(_)
                | Body::OwnerProposal(_)
                | Body::OwnerEscalation(_)
                | Body::Question(_)
                | Body::OwnerPersonTask(_)
                | Body::Result(_)
                | Body::Note { .. },
            )
            | None => None,
        },
    };
    let Some(value) = value else {
        return false;
    };
    if !crate::tree::cards::open(b, client, object, place, &saying(words, P::Historical, &[]), words, bindings) {
        return false;
    }
    render(b, client, value, words, bindings);
    crate::tree::cards::close(b);
    true
}

/// Checked envelope for each independent source and retained outer context.
/// The Task helper's eight typed working-set presence families are already
/// bounded by the released escalation envelope. Historical embedded sources
/// remain additional allocations and additional displayed fragments.
#[must_use]
pub fn max_nodes(l: &Limits) -> Option<u32> {
    let result = crate::sizes::sizes::result_facts(l)?;
    let task = crate::sizes::sizes::owner_escalation_nodes(l)?
        .checked_add(result)?
        .checked_add(result.checked_mul(l.rows)?)?;
    let escalation = task.checked_add(crate::tree::fragments::milestone::max_nodes(l)?)?.checked_add(12)?;
    let proposal = crate::tree::fragments::history::proposal_max_nodes(l)?;
    let person_goal = crate::tree::fragments::specification::nodes(l)?
        .checked_add(crate::tree::fragments::proposed_task::contract_nodes(l)?)?
        .checked_add(crate::tree::fragments::authority::document_nodes(l.rows, l.segments)?)?
        .checked_add(crate::tree::fragments::wake::policy_nodes())?
        .checked_add(l.rows.checked_mul(l.segments.checked_add(5)?)?)?
        .checked_add(40)?;
    let question = crate::chip_nodes(l)?
        .checked_add(skein_ui::markdown::max_nodes(l.text)?)?
        .checked_add(result.checked_mul(2)?)?
        .checked_add(l.rows.checked_mul(4)?)?
        .checked_add(120)?;
    let person = crate::tree::fragments::person_task::max_nodes(l)?.checked_add(result)?;
    task.max(escalation).max(proposal).max(person_goal).max(question).max(person).checked_add(12)
}
