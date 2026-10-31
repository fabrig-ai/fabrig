//! Withdraw proposals whose named spec has moved before answering the accepter.
use crate::routing::Work;
use crate::{CallKey, Core, Event, GoalRoute, PersonProposalRoute, RoutedCall};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Queue, ReplyTo, Token};

fn moved(core: &Core, proposal: &tasks::Proposal) -> Option<(u64, people::Moved)> {
    if let tasks::ProposalAction::Amend { task, revision, amendment } = &proposal.action
        && let Err(people::Refusal::Moved { now }) = core.party_amend_revision(*task, *revision, amendment)
    {
        return Some((*task, now));
    }
    if let tasks::ProposalAction::ReleasePlaced { task, hold, revision } = proposal.action
        && let Some(hold) = core.tasks.placed_hold(hold)
        && hold.revision != revision
    {
        return Some((task, people::Moved::PlacedHold { revision: hold.revision }));
    }
    None
}
pub(super) fn call(
    core: &mut Core,
    work: &mut Queue<Work>,
    token: Token,
    key: CallKey,
    proposal: &tasks::Proposal,
) -> bool {
    let Some(moved) = moved(core, proposal) else {
        return false;
    };
    assert!(core.pending_calls.insert(key, true).is_ok(), "stale acceptance call room");
    let token = core.fly_call(token, RoutedCall::Withdraw { key, proposal: proposal.number, moved: Some(moved) });
    work.push(Work::Event(Event::Tasks(tasks::Event::WithdrawProposal {
        reply_to: ReplyTo::new(token),
        proposer: proposal.proposer,
        proposal: proposal.number,
    })));
    true
}
pub(super) fn person(
    core: &mut Core,
    work: &mut Queue<Work>,
    request: Token,
    by: u64,
    proposal: &tasks::Proposal,
) -> bool {
    let Some((_, now)) = moved(core, proposal) else {
        return false;
    };
    let request = core.fly_person_proposal(PersonProposalRoute::Deciding {
        request,
        proposer: proposal.proposer,
        proposal: proposal.number,
        by,
        moved: Some(now),
    });
    work.push(Work::Event(Event::Tasks(tasks::Event::WithdrawProposal {
        reply_to: ReplyTo::new(request),
        proposer: proposal.proposer,
        proposal: proposal.number,
    })));
    true
}
pub(super) fn party(core: &mut Core, work: &mut Queue<Work>, request: Token, proposal: &tasks::PersonProposal) -> bool {
    let tasks::PersonAction::Amend { task, revision, amendment } = &proposal.action else {
        return false;
    };
    let Err(people::Refusal::Moved { now }) = core.party_amend_revision(*task, *revision, amendment) else {
        return false;
    };
    let request = core.fly_goal(
        request,
        GoalRoute::Withdrawing { proposer: proposal.proposer, proposal: proposal.number, moved: Some(now) },
    );
    work.push(Work::Event(Event::Tasks(tasks::Event::WithdrawPersonProposal {
        reply_to: Some(ReplyTo::new(request)),
        proposer: proposal.proposer,
        proposal: proposal.number,
    })));
    true
}
