//! Proposal terminals and recipient changes
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::flight::Flight;
use crate::routing::children::append_people;
use crate::routing::runs::call_decision;
use crate::{CallPart, Core, Limits, Request, RoutedCall};
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo};

pub(super) fn person_proposed(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    proposal: u64,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let (event, stubs) = core.person_proposed(reply_to.into_token(), proposal);
    for stub in stubs {
        let mut follow = Queue::uncapped();
        crate::routing::step::step_tasks(core, env, tasks::Event::RememberStub { stub }, &mut follow);
        for _ in 0..follow.len() {
            pending.push(follow.pop().expect("proposal input stub output"));
        }
    }
    append_people(core, env, event, out);
}

pub(super) fn person_proposal_decided(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: Option<ReplyTo>,
    proposer: u64,
    number: u64,
    outcome: tasks::ProposalOutcome,
    out: &mut Queue<Request>,
) {
    core.touch_party(proposer);
    if let Some(reply_to) = reply_to {
        let event = core.person_proposal_decided(reply_to.into_token(), proposer, number, outcome, env.wall);
        append_people(core, env, event, out);
    }
}

pub(super) fn proposal_stalled(
    core: &mut Core,
    env: &Env<Limits>,
    proposer: u64,
    proposal: u64,
    holder: tasks::ProposalHolder,
    pending: &mut Queue<tasks::Request>,
) {
    let Some(row) = core.tasks.proposal(proposer, proposal) else { return };
    if crate::proposals::placed_release(&row.action) {
        return;
    }
    let tasks::ProposalState::Pending { holder: current, .. } = row.state else { return };
    if current != holder {
        return;
    }
    let Some(next) = core.proposal_holder(&env.limits, proposer, &row.action, Some(holder), env.wall) else {
        return;
    };
    let revision = core.tasks.task(proposer).expect("pending proposer remains live").revision;
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(
        core,
        env,
        tasks::Event::StalledProposal { proposer, proposal, from: holder, revision, holder: next },
        &mut follow,
    );
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("stalled proposal output count"));
    }
}

pub(super) fn proposal_reroute_needed(
    core: &mut Core,
    env: &Env<Limits>,
    proposer: u64,
    proposal: u64,
    pending: &mut Queue<tasks::Request>,
) {
    let Some(row) = core.tasks.proposal(proposer, proposal) else { return };
    if crate::proposals::placed_release(&row.action) {
        return;
    }
    let tasks::ProposalState::Pending { holder: current, .. } = row.state else { return };
    let Some(next) = core.proposal_holder(&env.limits, proposer, &row.action, None, env.wall) else {
        return;
    };
    if current == next {
        return;
    }
    let revision = core.tasks.task(proposer).expect("pending proposer remains live").revision;
    let mut follow = Queue::uncapped();
    crate::routing::step::step_tasks(
        core,
        env,
        tasks::Event::StalledProposal { proposer, proposal, from: current, revision, holder: next },
        &mut follow,
    );
    for _ in 0..follow.len() {
        pending.push(follow.pop().expect("rerouted proposal output count"));
    }
}

pub(super) fn proposal_decided(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    proposer: u64,
    number: u64,
    outcome: tasks::ProposalOutcome,
    out: &mut Queue<Request>,
) {
    let token = reply_to.into_token();
    let (token, route) = match core.land_task(token) {
        Some(Flight::PersonProposal(route)) => {
            if let Some(event) =
                core.task_proposal_decided_for_person(token, proposer, number, outcome, env.wall, route)
            {
                append_people(core, env, event, out);
            }
            return;
        }
        Some(Flight::Call { request, route, .. }) => (request, route),
        _ => return,
    };
    if let RoutedCall::Withdraw { key, moved: Some((task, now)), .. } = route {
        assert!(outcome == tasks::ProposalOutcome::Withdrawn, "stale proposal withdrawn");
        call_decision(core, out, ReplyTo::new(token), key, CallPart::Moved { task, now });
        return;
    }
    let key = match route {
        RoutedCall::Decide { key, proposal } | RoutedCall::Withdraw { key, proposal, .. } if proposal == number => key,
        RoutedCall::Propose { .. }
        | RoutedCall::Accepting { .. }
        | RoutedCall::Decide { .. }
        | RoutedCall::Escalation { .. }
        | RoutedCall::Withdraw { .. }
        | RoutedCall::Message(_)
        | RoutedCall::Introduce(_)
        | RoutedCall::Subscribe { .. }
        | RoutedCall::Unsubscribe(_)
        | RoutedCall::Control { .. } => unreachable!("matching proposal decision"),
    };
    assert!(key.task == proposer || outcome != tasks::ProposalOutcome::Withdrawn, "withdrawal by proposer");
    call_decision(core, out, ReplyTo::new(token), key, CallPart::ProposalDecided { proposal: number, outcome });
}
