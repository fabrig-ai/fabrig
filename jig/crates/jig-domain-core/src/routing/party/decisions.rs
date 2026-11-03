//! Party goal, proposal and escalation choices routed to their owner
//! (`domain/people.md`, section 5.1). Each proposal maker selects its own
//! current store, falling back to its exact ended identity. Current escalation
//! revisions take precedence over ended decisions.

use crate::{
    Core, Event, GoalRoute, Limits, Now, PersonProposalRoute, Request,
    routing::{
        PersonEscalation, Work,
        party::dispatch::{Party, Routed},
        refusals::pool_refused,
    },
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use people::{LimitKind, Refusal};
use skein_lib::{Env, ReplyTo};

pub(crate) fn goal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    proposer: u64,
    number: u64,
    choice: people::ProposalDecision,
) {
    let Party { request, person, project, .. } = party;

    let Some(proposal) = core.tasks.person_proposal(proposer, number) else {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    };
    if proposal.project != project {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    }
    if let Err(why) = crate::acts::proposal(
        core,
        &env.limits,
        person,
        proposal_act(&choice),
        crate::documents::Proposer::Party(proposer),
        number,
        env.wall,
    ) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    let event = match choice {
        people::ProposalDecision::Accept => {
            if super::proposal_revisions::party(core, &mut routed.work, request, &proposal) {
                return;
            }
            let (source, _) = core.person_pool(project, person);
            let (period, pool) = core.open_person_pool(project, person);
            if let Some(event) = period {
                routed.work.push(Work::Event(Event::Tasks(event)));
            }
            if let Some(event) = pool {
                routed.work.push(Work::Event(Event::Tasks(event)));
            }
            let event = match proposal.action {
                tasks::PersonAction::Settings { .. } => unreachable!("settings acceptance has its own decision"),
                tasks::PersonAction::Batch(mut batch) => {
                    crate::admission::deadlines::fund(&mut batch, source, env.wall);
                    tasks::Event::Make {
                        reply_to: ReplyTo::new(request),
                        creator: tasks::Party::Person(proposer),
                        batch,
                    }
                }
                tasks::PersonAction::Amend { task, amendment, .. } => {
                    let Some(current) = core.tasks.task(task) else {
                        pool_refused(&mut routed.work, request, Refusal::Unknown);
                        return;
                    };
                    let before = crate::translate::tasks_authority(&current.authority);
                    let after =
                        crate::translate::tasks_authority(amendment.authority.as_ref().unwrap_or(&current.authority));
                    let stop_run = !authority::at_most(&before, &after, &core.authority.rules().implies);
                    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                        pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
                        return;
                    };
                    tasks::Event::Amend {
                        places: Box::new([]),
                        reply_to: ReplyTo::new(request),
                        by: tasks::Party::Person(proposer),
                        funder: Some(source),
                        task,
                        message,
                        stop_run,
                        amendment: *amendment,
                    }
                }
            };
            let request = core.fly_goal(request, GoalRoute::Accepting { proposer, proposal: number, by: person });
            crate::routing::flight::bind(event, request)
        }
        people::ProposalDecision::Reject { reason } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
                return;
            };
            let request = core.fly_goal(request, GoalRoute::Deciding { proposer, proposal: number, by: person });
            tasks::Event::DecidePersonProposal {
                reply_to: ReplyTo::new(request),
                proposer,
                proposal: number,
                by: tasks::Party::Person(person),
                message: Some(message),
                decision: tasks::ProposalDecision::Reject { reason },
                revision: None,
            }
        }
        people::ProposalDecision::Pass => {
            pool_refused(&mut routed.work, request, Refusal::NoFurther);
            return;
        }
    };
    routed.work.push(Work::Event(Event::Tasks(event)));
}

pub(crate) fn person_proposal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    proposer: u64,
    number: u64,
    choice: people::ProposalDecision,
) {
    let Party { request, person, project, .. } = party;

    let Some(proposal) = core.tasks.proposal(proposer, number) else {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    };
    if proposal.project != project {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    }
    if let Err(why) = crate::acts::proposal(
        core,
        &env.limits,
        person,
        proposal_act(&choice),
        crate::documents::Proposer::Task(proposer),
        number,
        env.wall,
    ) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    let current = match proposal.state {
        tasks::ProposalState::Pending { holder, .. } => holder,
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => unreachable!("common judgment checked pending holder"),
    };
    let next = match choice {
        people::ProposalDecision::Accept => {
            person_proposal_accept(core, routed, party, proposal, env.wall);
            return;
        }
        people::ProposalDecision::Reject { reason } => tasks::ProposalDecision::Reject { reason },
        people::ProposalDecision::Pass => match current {
            tasks::ProposalHolder::Person(_) => tasks::ProposalDecision::Pass {
                holder: tasks::ProposalHolder::Policy { project, kind: Core::proposal_kind(&proposal.action).0 },
            },
            tasks::ProposalHolder::Policy { .. } => {
                pool_refused(&mut routed.work, request, Refusal::NoFurther);
                return;
            }
            tasks::ProposalHolder::Task(_) => unreachable!("person standing checked"),
        },
    };
    let message = match next {
        tasks::ProposalDecision::Reject { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(message) => Some(message),
            None => {
                pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
                return;
            }
        },
        tasks::ProposalDecision::Pass { .. } => None,
        tasks::ProposalDecision::Accept => unreachable!("accept routed separately"),
    };
    let request = core.fly_person_proposal(PersonProposalRoute::Deciding {
        request,
        proposer,
        proposal: number,
        by: person,
        moved: None,
    });
    routed.work.push(Work::Event(Event::Tasks(tasks::Event::DecideProposal {
        reply_to: ReplyTo::new(request),
        proposer,
        proposal: number,
        message,
        by: tasks::Party::Person(person),
        decision: next,
    })));
}

#[expect(clippy::too_many_lines, reason = "one exhaustive authenticated person proposal dispatcher")]
pub(crate) fn person_proposal_accept(
    core: &mut Core,
    routed: &mut Routed,
    party: Party,
    proposal: tasks::Proposal,
    decision: skein_lib::Wall,
) {
    let Party { request, person, .. } = party;

    if super::proposal_revisions::person(core, &mut routed.work, request, person, &proposal) {
        return;
    }
    if crate::admission::proposals::placed_release(&proposal.action) {
        let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
            pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
            return;
        };
        let request = core.fly_person_proposal(PersonProposalRoute::Deciding {
            request,
            proposer: proposal.proposer,
            proposal: proposal.number,
            by: person,
            moved: None,
        });
        routed.work.push(Work::Event(Event::Tasks(tasks::Event::DecideProposal {
            reply_to: ReplyTo::new(request),
            proposer: proposal.proposer,
            proposal: proposal.number,
            message: Some(message),
            by: tasks::Party::Person(person),
            decision: tasks::ProposalDecision::Accept,
        })));
        return;
    }
    let source = tasks::Funder::Pool { project: proposal.project, person, period: core.period(proposal.project) };
    if let Err(why) = core.action_accounts(&proposal.action, source) {
        pool_refused(&mut routed.work, request, core.account_refusal(why));
        return;
    }
    match &proposal.action {
        tasks::ProposalAction::Amend { .. } | tasks::ProposalAction::Widen { .. } => {
            let (period, pool) = core.open_person_pool(proposal.project, person);
            if let Some(event) = period {
                routed.work.push(Work::Event(Event::Tasks(event)));
            }
            if let Some(event) = pool {
                routed.work.push(Work::Event(Event::Tasks(event)));
            }
        }
        tasks::ProposalAction::Settings { .. }
        | tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Effect { .. }
        | tasks::ProposalAction::ProcedureEffect { .. }
        | tasks::ProposalAction::ReleasePlaced { .. }
        | tasks::ProposalAction::Release { .. } => {}
    }
    let result_followups = core.tasks.result_proposal(proposal.proposer, proposal.number);
    match proposal.action {
        tasks::ProposalAction::Effect { connector, .. } | tasks::ProposalAction::ProcedureEffect { connector, .. } => {
            routed.work.push(Work::Event(Event::EffectStart {
                owner: core.effect_owner(),
                connector,
                origin: crate::EffectOrigin::Accept {
                    to: ReplyTo::new(request),
                    key: None,
                    proposal: Box::new(proposal),
                    by: tasks::Party::Person(person),
                },
            }));
            return;
        }
        tasks::ProposalAction::Settings { .. }
        | tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Amend { .. }
        | tasks::ProposalAction::Widen { .. }
        | tasks::ProposalAction::ReleasePlaced { .. }
        | tasks::ProposalAction::Release { .. } => {}
    }
    let event = match proposal.action {
        tasks::ProposalAction::Settings { .. } => unreachable!("settings has its own acceptance"),
        tasks::ProposalAction::Effect { .. } | tasks::ProposalAction::ProcedureEffect { .. } => {
            unreachable!("effect accepted through its connector")
        }
        tasks::ProposalAction::Batch(mut batch) => {
            crate::admission::deadlines::resolve(&mut batch, decision);
            let held = match core.people.role(person, proposal.project) {
                Some(role) => core.authority.escalations_held(proposal.project, role.number()),
                None => false,
            };
            if proposal.as_holder && !held {
                pool_refused(&mut routed.work, request, Refusal::NoEscalationHolder);
                return;
            }
            let creator =
                if proposal.as_holder { tasks::Party::Person(person) } else { tasks::Party::Task(proposal.proposer) };
            let (period, pool) = core.open_person_pool(proposal.project, person);
            if let Some(event) = period {
                routed.work.push(Work::Event(Event::Tasks(event)));
            }
            if let Some(event) = pool {
                routed.work.push(Work::Event(Event::Tasks(event)));
            }
            for member in &mut batch {
                member.funder =
                    tasks::Funder::Pool { project: proposal.project, person, period: core.period(proposal.project) };
            }
            if result_followups {
                tasks::Event::MakeResultFollowups {
                    reply_to: ReplyTo::new(request),
                    proposer: proposal.proposer,
                    proposal: proposal.number,
                    batch,
                }
            } else {
                tasks::Event::Make { reply_to: ReplyTo::new(request), creator, batch }
            }
        }
        tasks::ProposalAction::Amend { task, amendment, .. } => {
            if let Err(why) = core.amendment_accept_revision(proposal.proposer, task, amendment.release) {
                pool_refused(&mut routed.work, request, why);
                return;
            }
            let Some(current) = core.tasks.delegation(task) else {
                pool_refused(&mut routed.work, request, Refusal::Unknown);
                return;
            };
            let Some(amend_message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
                return;
            };
            let after = crate::translate::tasks_authority(amendment.authority.as_ref().unwrap_or(&current.authority));
            let before = crate::translate::tasks_authority(&current.authority);
            let stop_run = !authority::at_most(&before, &after, &core.authority.rules().implies);
            tasks::Event::Amend {
                places: Box::new([]),
                funder: Some(source),
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                message: amend_message,
                stop_run,
                amendment,
            }
        }
        tasks::ProposalAction::Widen { task, authority } => {
            let Some(amend_message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
                return;
            };
            tasks::Event::Amend {
                places: Box::new([]),
                funder: Some(source),
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                message: amend_message,
                stop_run: false,
                amendment: tasks::Amendment {
                    tracked: None,
                    addressed: None,
                    release: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(authority),
                    llm: None,
                    reason: proposal.reason,
                },
            }
        }
        tasks::ProposalAction::ReleasePlaced { task, .. } | tasks::ProposalAction::Release { task } => {
            tasks::Event::Control {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                control: tasks::Control::Release { again: None, message: None },
            }
        }
    };
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
        return;
    };
    let request = core.fly_person_proposal(PersonProposalRoute::Accepting {
        request,
        person,
        proposer: proposal.proposer,
        proposal: proposal.number,
        message,
    });
    routed.work.push(Work::Event(Event::Tasks(crate::routing::flight::bind(event, request))));
}

pub(crate) fn person_escalation_decide(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    revision: u64,
    decision: people::EscalationDecision,
) {
    let Party { request, person, project, .. } = party;

    if let people::EscalationDecision::Reject { reason } = &decision
        && reason.len() > skein_lib::length::usize_of(env.limits.escalation_reason_bytes)
    {
        pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Words });
        return;
    }
    let Some(context) = core.tasks.escalation(task) else {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    };
    if context.project != project {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    }
    let pending = match context.escalation {
        tasks::Escalation::Waiting { revision: current, .. } => current == revision,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            false
        }
    };
    if !pending {
        pool_refused(&mut routed.work, request, Refusal::NotWaiting);
        return;
    }
    let act = match &decision {
        people::EscalationDecision::Release => crate::acts::Act::Release,
        people::EscalationDecision::Reject { .. } => crate::acts::Act::LeaveHeld,
        people::EscalationDecision::Pass => crate::acts::Act::PassUp,
    };
    if let Err(why) = crate::acts::escalation(core, &env.limits, person, act, task) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    let semantic = match &decision {
        people::EscalationDecision::Release => tasks::EscalationDecision::Release,
        people::EscalationDecision::Reject { reason } => tasks::EscalationDecision::Reject { reason: reason.clone() },
        people::EscalationDecision::Pass => {
            let Some(holder) = core.escalation_fallback(project) else {
                pool_refused(&mut routed.work, request, Refusal::NoFurther);
                return;
            };
            tasks::EscalationDecision::Pass { holder }
        }
    };
    let entry = match semantic {
        tasks::EscalationDecision::Pass { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(entry) => Some(entry),
            None => {
                pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
                return;
            }
        },
        tasks::EscalationDecision::Release | tasks::EscalationDecision::Reject { .. } => None,
    };
    let request = core.fly(crate::routing::flight::Flight::PersonEscalation(PersonEscalation {
        request,
        requester: context.requester,
        person,
        project,
        task,
        revision,
        decision,
    }));
    routed.work.push(Work::Event(Event::Tasks(tasks::Event::DecideEscalation {
        reply_to: ReplyTo::new(request),
        task,
        revision,
        by: person,
        entry,
        decision: semantic,
    })));
}

pub(crate) fn current_person_escalation(core: &Core, task: u64, revision: u64) -> bool {
    let Some(context) = core.tasks.escalation(task) else { return false };
    match context.escalation {
        tasks::Escalation::Waiting { revision: current, .. } => current == revision,
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            false
        }
    }
}

pub(crate) fn proposal_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    choice: super::dispatch::Proposal,
) {
    let super::dispatch::Proposal { proposer, number: proposal, decision, running } = choice;
    let Party { request, person, project, .. } = party;

    match proposer {
        people::Proposer::Party(number) => {
            if let Some(row) = core.tasks.person_proposal(number, proposal) {
                let settings = match row.action {
                    tasks::PersonAction::Settings { .. } => true,
                    tasks::PersonAction::Batch(_) | tasks::PersonAction::Amend { .. } => false,
                };
                let accepts = decision == people::ProposalDecision::Accept;
                if running.is_some() != (settings && accepts) {
                    pool_refused(
                        &mut routed.work,
                        request,
                        Refusal::NotWellFormed { at: 0, why: people::Malformed::Running },
                    );
                    return;
                }
                if settings && accepts {
                    if row.project != project {
                        pool_refused(&mut routed.work, request, Refusal::Unknown);
                        return;
                    }
                    if let Err(why) = crate::acts::proposal(
                        core,
                        &env.limits,
                        person,
                        crate::acts::Act::Accept,
                        crate::documents::Proposer::Party(number),
                        proposal,
                        env.wall,
                    ) {
                        pool_refused(&mut routed.work, request, why);
                        return;
                    }
                    let key =
                        (people::Proposer::Party(number), u32::try_from(proposal).expect("settings proposal identity"));
                    let outcome = crate::routing::settings::dispatch::accept(
                        core,
                        env,
                        routed,
                        person,
                        request,
                        key,
                        running.expect("settings acceptance"),
                    );
                    match outcome {
                        Ok(Some(revision)) => {
                            let request =
                                core.fly_goal(request, GoalRoute::Deciding { proposer: number, proposal, by: person });
                            routed.work.push(Work::Event(Event::Tasks(tasks::Event::DecidePersonProposal {
                                reply_to: ReplyTo::new(request),
                                proposer: number,
                                proposal,
                                by: tasks::Party::Person(person),
                                message: None,
                                revision: Some(revision),
                                decision: tasks::ProposalDecision::Accept,
                            })));
                        }
                        Ok(None) => return,
                        Err(why) => routed.work.push(Work::Event(Event::People(people::Event::Decided {
                            request,
                            outcome: people::Outcome::Refused(why),
                        }))),
                    }
                    return;
                }
                goal_decide(core, env, routed, party, number, proposal, decision);
                return;
            }
        }
        people::Proposer::Task(number) => {
            if let Some(row) = core.tasks.proposal(number, proposal)
                && let tasks::ProposalAction::Settings { .. } = row.action
            {
                task_settings_decide(core, env, routed, party, row, (decision, running));
                return;
            }
            if running.is_some() {
                pool_refused(
                    &mut routed.work,
                    request,
                    Refusal::NotWellFormed { at: 0, why: people::Malformed::Running },
                );
                return;
            }
            if core.tasks.proposal(number, proposal).is_some() {
                person_proposal_decide(core, env, routed, party, number, proposal, decision);
                return;
            }
        }
    }
    routed.out.push(Request::Now(Box::new(Now::EndedProposal { request, person, project, proposer, proposal })));
}

pub(crate) fn escalation_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    revision: u64,
    decision: people::EscalationDecision,
) {
    let Party { request, person, project, role, .. } = party;

    if current_person_escalation(core, task, revision) {
        person_escalation_decide(core, env, routed, party, task, revision, decision);
        return;
    }

    if match &decision {
        people::EscalationDecision::Reject { reason } => {
            reason.len() > skein_lib::length::usize_of(env.limits.escalation_reason_bytes)
        }
        people::EscalationDecision::Release | people::EscalationDecision::Pass => false,
    } {
        pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Words });
        return;
    }
    if let Some(context) = core.tasks.escalation(task)
        && (context.project != project || !core.escalation_visible(&context, person, role))
    {
        pool_refused(&mut routed.work, request, Refusal::Unknown);
        return;
    }
    if let Some(context) = core.tasks.escalation(task)
        && context.escalation.revision() != revision
    {
        pool_refused(
            &mut routed.work,
            request,
            crate::settings::revisions::moved_hold(context.escalation.revision(), context.why),
        );
        return;
    }
    routed.out.push(Request::Now(Box::new(Now::EndedEscalation { request, person, project, task, revision })));
}

fn proposal_act(choice: &people::ProposalDecision) -> crate::acts::Act {
    match choice {
        people::ProposalDecision::Accept => crate::acts::Act::Accept,
        people::ProposalDecision::Reject { .. } => crate::acts::Act::Reject,
        people::ProposalDecision::Pass => crate::acts::Act::PassUp,
    }
}

fn task_settings_decide(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    row: tasks::Proposal,
    choice: (people::ProposalDecision, Option<people::Running>),
) {
    let Party { request, person, project, .. } = party;

    let (choice, running) = choice;
    let accepts = choice == people::ProposalDecision::Accept;
    if running.is_some() != accepts {
        return pool_refused(
            &mut routed.work,
            request,
            Refusal::NotWellFormed { at: 0, why: people::Malformed::Running },
        );
    }
    if project != people::DEPLOYMENT {
        return pool_refused(&mut routed.work, request, Refusal::Unknown);
    }
    if let Err(why) = crate::acts::proposal(
        core,
        &env.limits,
        person,
        proposal_act(&choice),
        crate::documents::Proposer::Task(row.proposer),
        row.number,
        env.wall,
    ) {
        return pool_refused(&mut routed.work, request, why);
    }
    let reason = match &choice {
        people::ProposalDecision::Reject { reason } => reason.clone(),
        people::ProposalDecision::Accept => Box::new([]),
        people::ProposalDecision::Pass => return pool_refused(&mut routed.work, request, Refusal::NoFurther),
    };
    if reason.len() > skein_lib::length::usize_of(env.limits.tasks.message_bytes) {
        return pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Words });
    }
    let messages = u64::from(env.limits.tasks.tasks)
        .checked_mul(2)
        .expect("move messages")
        .checked_add(u64::from(env.limits.settings.proposals))
        .expect("stale messages")
        .checked_add(1)
        .expect("outcome message");
    if core.counters.deployment().messages.checked_add(messages).is_none() {
        return pool_refused(&mut routed.work, request, Refusal::Limit { which: LimitKind::Arithmetic });
    }
    let outcome = if accepts {
        let key =
            (people::Proposer::Task(row.proposer), u32::try_from(row.number).expect("settings proposal identity"));
        match crate::routing::settings::dispatch::accept(
            core,
            env,
            routed,
            person,
            request,
            key,
            running.expect("acceptance running choice"),
        ) {
            Ok(Some(revision)) => tasks::SettingsOutcome::Accepted { revision },
            Ok(None) => return,
            Err(why) => return pool_refused(&mut routed.work, request, why),
        }
    } else {
        tasks::SettingsOutcome::Rejected
    };
    let message =
        crate::fresh(&mut core.counters, crate::Family::Message).expect("settings outcome number preflighted");
    let request = core.fly_person_proposal(PersonProposalRoute::Deciding {
        request,
        proposer: row.proposer,
        proposal: row.number,
        by: person,
        moved: None,
    });
    routed.work.push(Work::Event(Event::Tasks(tasks::Event::DecideSettingsProposal {
        reply_to: Some(ReplyTo::new(request)),
        decision: tasks::SettingsDecision {
            proposer: row.proposer,
            proposal: row.number,
            message,
            by: tasks::Party::Person(person),
            outcome,
            reason,
        },
    })));
}
