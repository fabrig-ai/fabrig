//! Named run calls, turns and terminals routed to tasks, authority and the root
//! (`domain/engine/vocabulary.md`, section 1.4 and `domain/engine/executors.md`). Delegation and preparation
//! are separate families so each route file stays under 1,500 lines.

use super::{Call, Proposing};
use crate::{
    CallHolder, CallKey, CallPart, CallRecord, Core, CoreRecord, EscalationChoice, Event, Held, Limits, NamedAction,
    PayloadRefusal, ProposalChoice, Record, Request, RoutedCall, ToolKind, Write,
    routing::{NoteRoute, Work, run::delegation::propose_batch},
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_fleet as fleet;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

fn connector_topic(kind: tasks::SubscriptionKind) -> Option<(u16, u64)> {
    match kind {
        tasks::SubscriptionKind::Topic { connector, topic } => Some((connector, topic)),
        tasks::SubscriptionKind::Task { .. } | tasks::SubscriptionKind::Timer { .. } => None,
    }
}

pub(crate) fn call_decision(core: &mut Core, out: &mut Queue<Request>, to: ReplyTo, key: CallKey, part: CallPart) {
    if !core.decide_pending_call(key, part.clone()) {
        return;
    }
    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Call(CallRecord {
        key,
        part: part.clone(),
        settled: core.call_settled.get(&key).cloned(),
    })))));
    out.push(Request::Held(Box::new(Held::CallAnswer { to, key, part })));
}

pub(crate) fn admit_named_call(
    core: &mut Core,
    out: &mut Queue<Request>,
    to: ReplyTo,
    key: CallKey,
) -> Option<ReplyTo> {
    if !core.current_proof(key.task, key.attempt) {
        return None;
    }
    if let Some(part) = core.call_parts.get(&key) {
        out.push(Request::Held(Box::new(Held::CallAnswer { to, key, part: part.clone() })));
        return None;
    }
    if !core.reserve_call(key, CallHolder::Routed) {
        out.push(Request::Now(Box::new(crate::Now::CallBusy { to })));
        return None;
    }
    Some(to)
}

pub(crate) fn escalation_refused(
    core: &mut Core,
    to: ReplyTo,
    key: CallKey,
    task: u64,
    why: tasks::Refusal,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    call_decision(
        core,
        &mut out,
        to,
        key,
        CallPart::EscalationRefused(tasks::Problem { task: Some(task), why, blocked_by: None, capacity: None }),
    );
    out
}

pub(crate) fn proposal_refused(core: &mut Core, to: ReplyTo, key: CallKey, why: tasks::Refusal) -> Queue<Request> {
    let mut out = Queue::uncapped();
    call_decision(
        core,
        &mut out,
        to,
        key,
        CallPart::ProposalRefused(tasks::Problem { task: Some(key.task), why, blocked_by: None, capacity: None }),
    );
    out
}

#[expect(clippy::too_many_lines, reason = "one exhaustive authenticated task proposal dispatcher")]
pub(crate) fn named_proposal_accept(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    to: ReplyTo,
    key: CallKey,
    proposal: tasks::Proposal,
) -> Queue<Request> {
    let token = to.into_token();
    let to = ReplyTo::new(token);
    if crate::routing::party::proposal_revisions::call(core, work, token, key, &proposal) {
        return Queue::uncapped();
    }
    let result_followups = core.tasks.result_proposal(proposal.proposer, proposal.number);
    if let Err(why) = core.proposal_accept_allowed(
        &env.limits,
        proposal.proposer,
        key.task,
        proposal.project,
        &proposal.action,
        env.wall,
    ) {
        return proposal_refused(core, to, key, why);
    }
    if crate::admission::proposals::placed_release(&proposal.action) {
        let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
            return proposal_refused(core, to, key, tasks::Refusal::Busy);
        };
        let token = core.fly_call(token, RoutedCall::Decide { key, proposal: proposal.number });
        work.push(Work::Event(Event::Tasks(tasks::Event::DecideProposal {
            reply_to: ReplyTo::new(token),
            proposer: proposal.proposer,
            proposal: proposal.number,
            message: Some(message),
            by: tasks::Party::Task(key.task),
            decision: tasks::ProposalDecision::Accept,
        })));
        return Queue::uncapped();
    }
    match proposal.action {
        tasks::ProposalAction::Effect { connector, .. } | tasks::ProposalAction::ProcedureEffect { connector, .. } => {
            let owner = core.effect_owner();
            return crate::effects::start(
                core,
                env,
                owner,
                connector,
                crate::EffectOrigin::Accept {
                    to,
                    key: Some(key),
                    proposal: Box::new(proposal),
                    by: tasks::Party::Task(key.task),
                },
            );
        }
        tasks::ProposalAction::Settings { .. }
        | tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Amend { .. }
        | tasks::ProposalAction::Widen { .. }
        | tasks::ProposalAction::ReleasePlaced { .. }
        | tasks::ProposalAction::Release { .. } => {}
    }
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        return proposal_refused(core, to, key, tasks::Refusal::Busy);
    };
    let token = to.into_token();
    let event = match proposal.action {
        tasks::ProposalAction::Settings { .. } => unreachable!("tasks cannot accept settings"),
        tasks::ProposalAction::Effect { .. } | tasks::ProposalAction::ProcedureEffect { .. } => {
            unreachable!("effect accepted through its connector")
        }
        tasks::ProposalAction::Batch(mut batch) => {
            crate::admission::deadlines::resolve(&mut batch, env.wall);
            let creator =
                if proposal.as_holder { tasks::Party::Task(key.task) } else { tasks::Party::Task(proposal.proposer) };
            for member in &mut batch {
                member.funder = tasks::Funder::Task(key.task);
            }
            if result_followups {
                tasks::Event::MakeResultFollowups {
                    reads_project: false,
                    reply_to: ReplyTo::new(token),
                    proposer: proposal.proposer,
                    proposal: proposal.number,
                    batch,
                }
            } else {
                tasks::Event::Make {
                    reads_project: false,
                    maker: None,
                    places: Box::new([]),
                    reply_to: ReplyTo::new(token),
                    creator,
                    batch,
                }
            }
        }
        tasks::ProposalAction::Amend { task, amendment, .. } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Busy);
            };
            let Some(current) = core.tasks.delegation(task) else {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Unknown);
            };
            let after = crate::translate::tasks_authority(amendment.authority.as_ref().unwrap_or(&current.authority));
            let before = crate::translate::tasks_authority(&current.authority);
            let stop_run = !authority::at_most(&before, &after, &core.authority.rules().implies);
            tasks::Event::Amend {
                reads_project: false,
                operation: tasks::Amend {
                    funder: Some(tasks::Funder::Task(key.task)),
                    task,
                    message,
                    stop_run,
                    amendment: Box::new(amendment),
                },
                places: Box::new([]),
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
            }
        }
        tasks::ProposalAction::Widen { task, authority } => {
            let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                return proposal_refused(core, ReplyTo::new(token), key, tasks::Refusal::Busy);
            };
            tasks::Event::Amend {
                reads_project: false,
                operation: tasks::Amend {
                    funder: Some(tasks::Funder::Task(key.task)),
                    task,
                    message,
                    stop_run: false,
                    amendment: Box::new(tasks::Amendment {
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
                    }),
                },
                places: Box::new([]),
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
            }
        }
        tasks::ProposalAction::ReleasePlaced { task, .. } | tasks::ProposalAction::Release { task } => {
            tasks::Event::Release {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                operation: tasks::Release { task, again: None, message: None },
            }
        }
    };
    let token = core.fly_call(
        token,
        RoutedCall::Accepting { key, proposer: proposal.proposer, proposal: proposal.number, message },
    );
    work.push(Work::Event(Event::Tasks(crate::routing::flight::bind(event, token))));
    Queue::uncapped()
}

pub(crate) fn named_proposal_decide(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    call: Call,
    proposer: u64,
    proposal: u64,
    choice: ProposalChoice,
) -> Queue<Request> {
    let Call { to, key } = call;
    let Some(pending) = core.tasks.proposal(proposer, proposal) else {
        return proposal_refused(core, to, key, tasks::Refusal::Unknown);
    };
    let current = match pending.state {
        tasks::ProposalState::Pending { holder, .. } => holder,
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => {
            return proposal_refused(core, to, key, tasks::Refusal::State);
        }
    };
    if current != tasks::ProposalHolder::Task(key.task) {
        return proposal_refused(core, to, key, tasks::Refusal::Reference);
    }
    let next = match choice {
        ProposalChoice::Accept => return named_proposal_accept(core, env, work, to, key, pending),
        ProposalChoice::Reject { reason } => tasks::ProposalDecision::Reject { reason },
        ProposalChoice::Pass => {
            let Some(holder) = core.proposal_holder(&env.limits, proposer, &pending.action, Some(current), env.wall)
            else {
                return proposal_refused(core, to, key, tasks::Refusal::Reference);
            };
            tasks::ProposalDecision::Pass { holder }
        }
    };
    let message = match next {
        tasks::ProposalDecision::Reject { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(message) => Some(message),
            None => return proposal_refused(core, to, key, tasks::Refusal::Busy),
        },
        tasks::ProposalDecision::Pass { .. } => None,
        tasks::ProposalDecision::Accept => unreachable!("accept routed through action"),
    };
    let token = to.into_token();
    let token = core.fly_call(token, RoutedCall::Decide { key, proposal });
    work.push(Work::Event(Event::Tasks(tasks::Event::DecideProposal {
        reply_to: ReplyTo::new(token),
        proposer,
        proposal,
        message,
        by: tasks::Party::Task(key.task),
        decision: next,
    })));
    Queue::uncapped()
}

pub(crate) fn named_propose(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    proposing: Proposing,
    mut action: tasks::ProposalAction,
    admitted: &mut Option<Token>,
) -> Queue<Request> {
    let Proposing { to, key, reason, as_holder } = proposing;
    if reason.len() > skein_lib::length::usize_of(env.limits.tasks.message_bytes) {
        return proposal_refused(core, to, key, tasks::Refusal::Read);
    }
    let Some(context) = core.tasks.delegation(key.task) else {
        return proposal_refused(core, to, key, tasks::Refusal::Unknown);
    };
    if let tasks::ProposalAction::Widen { task, .. } = &mut action {
        *task = key.task;
    }
    core.keep_proposal_models(key.task, &mut action);
    if let tasks::ProposalAction::ReleasePlaced { task, hold, revision } = &action
        && let Err(why) = core.release_proposal_admit(key.task, *task, *hold, *revision, env.limits.tasks.depth)
    {
        if let tasks::Refusal::HoldRevision { revision } = why {
            let mut out = Queue::uncapped();
            call_decision(
                core,
                &mut out,
                to,
                key,
                CallPart::Moved { task: *task, now: jig_domain_people::Moved::PlacedHold { revision } },
            );
            return out;
        }
        return proposal_refused(core, to, key, why);
    }
    if let tasks::ProposalAction::Amend { task, revision, amendment } = &action
        && let Err(jig_domain_people::Refusal::Moved { now }) = core.party_amend_revision(*task, *revision, amendment)
    {
        let mut out = Queue::uncapped();
        call_decision(core, &mut out, to, key, CallPart::Moved { task: *task, now });
        return out;
    }
    if let Err(why) = core.proposal_admit_action(context.project, &action, env.wall) {
        return proposal_refused(core, to, key, why);
    }
    let Some(holder) = core.proposal_holder(&env.limits, key.task, &action, None, env.wall) else {
        return proposal_refused(core, to, key, tasks::Refusal::Reference);
    };
    if !crate::acts::bounds::proposal_has_recipient(core, holder) {
        return proposal_refused(core, to, key, tasks::Refusal::Unproposable);
    }
    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        return proposal_refused(core, to, key, tasks::Refusal::Busy);
    };
    let token = to.into_token();
    let token = core.fly_call(token, RoutedCall::Propose { key, proposal: number });
    work.push(Work::Event(Event::Tasks(tasks::Event::Propose {
        reply_to: ReplyTo::new(token),
        proposal: tasks::Proposal {
            number,
            proposer: key.task,
            project: context.project,
            action,
            reason,
            as_holder,
            state: tasks::ProposalState::Pending { holder, since: env.wall },
        },
    })));
    *admitted = Some(token);
    Queue::uncapped()
}

pub(crate) fn named_escalation(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    call: Call,
    task: u64,
    revision: u64,
    choice: EscalationChoice,
) -> Queue<Request> {
    let Call { to, key } = call;
    let Some(context) = core.tasks.escalation(task) else {
        return escalation_refused(core, to, key, task, tasks::Refusal::Unknown);
    };
    let (current, holder) = match context.escalation {
        tasks::Escalation::Waiting { revision, holder, .. } => (revision, holder),
        tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. } | tasks::Escalation::Rejected { .. } => {
            return escalation_refused(core, to, key, task, tasks::Refusal::State);
        }
    };
    if let Err(jig_domain_people::Refusal::Moved { now }) = core.hold_revision(task, revision) {
        let mut out = Queue::uncapped();
        call_decision(core, &mut out, to, key, CallPart::Moved { task, now });
        return out;
    }
    if current != revision || holder != tasks::EscalationHolder::Task(key.task) {
        return escalation_refused(core, to, key, task, tasks::Refusal::Reference);
    }
    let role = core.people.role(context.requester, context.project);
    let mut passed_up = false;
    let semantic = match choice {
        EscalationChoice::Release => {
            if core.escalation_recipient(&env.limits, &context, role) == Some(holder) {
                tasks::EscalationDecision::Release
            } else {
                let Some(next) = core.escalation_recipient_after(&env.limits, &context, role, Some(holder)) else {
                    return escalation_refused(core, to, key, task, tasks::Refusal::Funding);
                };
                passed_up = true;
                tasks::EscalationDecision::Pass { holder: next }
            }
        }
        EscalationChoice::Reject { reason } => tasks::EscalationDecision::Reject { reason },
        EscalationChoice::Pass => {
            let Some(next) = core.escalation_recipient_after(&env.limits, &context, role, Some(holder)) else {
                return escalation_refused(core, to, key, task, tasks::Refusal::Reference);
            };
            tasks::EscalationDecision::Pass { holder: next }
        }
    };
    let entry = match semantic {
        tasks::EscalationDecision::Pass { .. } => match crate::fresh(&mut core.counters, crate::Family::Message) {
            Some(entry) => Some(entry),
            None => return escalation_refused(core, to, key, task, tasks::Refusal::Busy),
        },
        tasks::EscalationDecision::Release | tasks::EscalationDecision::Reject { .. } => None,
    };
    let token = to.into_token();
    let token = core.fly_call(token, RoutedCall::Escalation { key, task, revision, passed_up });
    work.push(Work::Event(Event::Tasks(tasks::Event::DecideEscalation {
        reply_to: ReplyTo::new(token),
        task,
        revision,
        by: tasks::EscalationDecider::Task(key.task),
        entry,
        decision: semantic,
    })));
    Queue::uncapped()
}

#[expect(clippy::too_many_lines, reason = "one generic named-call vocabulary routes its finite task continuations")]
pub(crate) fn named_action(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    to: ReplyTo,
    key: CallKey,
    action: NamedAction,
) -> Queue<Request> {
    if !core.current_proof(key.task, key.attempt) {
        return Queue::uncapped();
    }
    let action = match action {
        NamedAction::ReadTask { task, field, from, room } => {
            return crate::reads::run_task::run(
                core,
                env,
                to,
                key,
                crate::reads::run_task::Read { task, field, from, room, joined: false },
            );
        }
        NamedAction::PersonRead(read) => return crate::reads::person::run(core, env, to, key, *read),
        NamedAction::Settings(read) => return crate::reads::settings::run(core, env, to, key, read),
        NamedAction::ReadProposal { proposer, number } => {
            return crate::reads::settings::run_proposal(core, env, to, key, proposer, number);
        }
        action @ (NamedAction::ProposeSettings { .. }
        | NamedAction::Message { .. }
        | NamedAction::MarkRead { .. }
        | NamedAction::Introduce { .. }
        | NamedAction::Subscribe { .. }
        | NamedAction::Unsubscribe { .. }
        | NamedAction::Control { .. }
        | NamedAction::Release { .. }
        | NamedAction::Measure { .. }
        | NamedAction::Amend { .. }
        | NamedAction::DecideEscalation { .. }
        | NamedAction::WithdrawProposal { .. }
        | NamedAction::DecideProposal { .. }
        | NamedAction::Propose { .. }
        | NamedAction::ProposeBatch { .. }
        | NamedAction::ProposeGoal { .. }
        | NamedAction::Note { .. }
        | NamedAction::NoteLines { .. }
        | NamedAction::NoteDelete { .. }
        | NamedAction::Recall { .. }) => action,
    };
    let mut out = Queue::uncapped();
    let Some(to) = admit_named_call(core, &mut out, to, key) else { return out };
    if let NamedAction::NoteDelete { scope, name, recalled } = action {
        return crate::routing::party::delete(core, work, Call { to, key }, scope, name, recalled);
    }
    if let Some(part) = core.authorize_tool(key, named_tool_kind(&action)) {
        call_decision(core, &mut out, to, key, part);
        return out;
    }
    let token = to.into_token();
    if malformed_recording(&action, env.limits.tasks.inbox_messages) {
        call_decision(
            core,
            &mut out,
            ReplyTo::new(token),
            key,
            CallPart::MessageRefused(tasks::Problem::new(Some(key.task), tasks::Refusal::Read)),
        );
        return out;
    }
    match action {
        NamedAction::ReadTask { .. } => unreachable!("task reads use the unrecorded path"),
        NamedAction::PersonRead(_) => unreachable!("person reads use the unrecorded path"),
        NamedAction::ReadProposal { .. } | NamedAction::Settings(_) => {
            unreachable!("settings reads use the unrecorded path")
        }
        NamedAction::ProposeSettings { against, items, reason } => {
            return crate::routing::settings::dispatch::propose_run(
                core,
                env,
                work,
                ReplyTo::new(token),
                key,
                crate::settings::Proposed { against, items, reason },
            );
        }
        NamedAction::MarkRead { messages } => {
            let token = core.fly_call(token, RoutedCall::MarkRead(key));
            work.push(Work::Event(Event::Tasks(tasks::Event::RecordWords {
                reply_to: ReplyTo::new(token),
                task: key.task,
                messages,
            })));
        }
        NamedAction::Message { target, kind, words } => {
            if core.person_answers
                && let tasks::MessageKind::Answer { question } = kind
                && core.people.question_person(key.task, question).is_some()
            {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::MessageRefused(tasks::Problem::new(
                        Some(target),
                        tasks::Refusal::PersonAnswers { question },
                    )),
                );
                return out;
            }
            let Some(context) = core.tasks.delegation(key.task) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::MessageRefused(tasks::Problem {
                        task: Some(key.task),
                        why: tasks::Refusal::Unknown,
                        blocked_by: None,
                        capacity: None,
                    }),
                );
                return out;
            };
            let project = context.project;
            let Some(number) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::MessageRefused(tasks::Problem {
                        task: Some(target),
                        why: tasks::Refusal::Busy,
                        blocked_by: None,
                        capacity: None,
                    }),
                );
                return out;
            };
            let token = core.fly_call(token, RoutedCall::Message(key));
            work.push(Work::Event(Event::Tasks(tasks::Event::Message {
                reply_to: ReplyTo::new(token),
                project,
                operation: tasks::Message {
                    task: target,
                    word: tasks::Word {
                        number,
                        from: tasks::Party::Task(key.task),
                        kind,
                        words,
                        at: env.wall,
                        hits: 1,
                        eligible: false,
                    },
                },
            })));
        }
        NamedAction::Introduce { left, right } => {
            let token = core.fly_call(token, RoutedCall::Introduce(key));
            work.push(Work::Event(Event::Tasks(tasks::Event::Introduce {
                reply_to: ReplyTo::new(token),
                by: key.task,
                left,
                right,
            })));
        }
        NamedAction::Subscribe { kind } => {
            let Some(subscription) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::SubscriptionRefused(tasks::Problem {
                        task: Some(key.task),
                        why: tasks::Refusal::Busy,
                        blocked_by: None,
                        capacity: None,
                    }),
                );
                return out;
            };
            let topic = connector_topic(kind);
            let token = core.fly_call(token, RoutedCall::Subscribe { key, subscription, topic });
            work.push(Work::Event(Event::Tasks(tasks::Event::Subscribe {
                reads_project: false,
                reply_to: ReplyTo::new(token),
                task: key.task,
                subscription: tasks::Subscription { number: subscription, kind },
            })));
        }
        NamedAction::Unsubscribe { subscription } => {
            let mut route = RoutedCall::Unsubscribe { key, subscription, topic: None };
            if let Some(task) = core.tasks.task(key.task) {
                for interest in &task.subscriptions {
                    if interest.number == subscription {
                        route = RoutedCall::Unsubscribe {
                            key,
                            subscription: interest.number,
                            topic: connector_topic(interest.kind),
                        };
                        break;
                    }
                }
            }
            let token = core.fly_call(token, route);
            work.push(Work::Event(Event::Tasks(tasks::Event::Unsubscribe {
                reply_to: ReplyTo::new(token),
                task: key.task,
                subscription,
            })));
        }
        NamedAction::Release { target, revision, again } => {
            let message = match core.release_admit(target, revision, again) {
                Ok(message) => message,
                Err(why) => {
                    let part = match why {
                        jig_domain_people::Refusal::Moved { now } => CallPart::Moved { task: target, now },
                        jig_domain_people::Refusal::Authority {
                            lacking: jig_domain_people::Lack::Again { required },
                            ..
                        } => CallPart::Again { task: target, required },
                        why @ (jig_domain_people::Refusal::Unknown | jig_domain_people::Refusal::Limit { .. }) => {
                            CallPart::ControlRefused(tasks::Problem {
                                task: Some(target),
                                why: if why == jig_domain_people::Refusal::Unknown {
                                    tasks::Refusal::Unknown
                                } else {
                                    tasks::Refusal::Busy
                                },
                                blocked_by: None,
                                capacity: None,
                            })
                        }
                        jig_domain_people::Refusal::Forgotten { .. }
                        | jig_domain_people::Refusal::Judged { .. }
                        | jig_domain_people::Refusal::Stale { .. }
                        | jig_domain_people::Refusal::Full { .. }
                        | jig_domain_people::Refusal::LiveTasks { .. }
                        | jig_domain_people::Refusal::Circle { .. }
                        | jig_domain_people::Refusal::Dependency { .. }
                        | jig_domain_people::Refusal::Kept { .. }
                        | jig_domain_people::Refusal::Role { .. }
                        | jig_domain_people::Refusal::Authority { .. }
                        | jig_domain_people::Refusal::Unlinked { .. }
                        | jig_domain_people::Refusal::QuestionAnswered { .. }
                        | jig_domain_people::Refusal::Concluded { .. }
                        | jig_domain_people::Refusal::Ended { .. }
                        | jig_domain_people::Refusal::Taken { .. }
                        | jig_domain_people::Refusal::NotWaiting
                        | jig_domain_people::Refusal::NoFurther
                        | jig_domain_people::Refusal::Provider
                        | jig_domain_people::Refusal::SignInRedirect { .. }
                        | jig_domain_people::Refusal::AccountMove { .. }
                        | jig_domain_people::Refusal::NoTerminal
                        | jig_domain_people::Refusal::LoginUnderWay
                        | jig_domain_people::Refusal::NotLending
                        | jig_domain_people::Refusal::NameTaken
                        | jig_domain_people::Refusal::ProgramNameReserved
                        | jig_domain_people::Refusal::NotOffered
                        | jig_domain_people::Refusal::NotWellFormed { .. }
                        | jig_domain_people::Refusal::NoEscalationHolder
                        | jig_domain_people::Refusal::Llm { .. }
                        | jig_domain_people::Refusal::Account { .. }
                        | jig_domain_people::Refusal::KeyConflict => unreachable!(
                            "shared release admission returns revision, again, unknown or arithmetic refusals"
                        ),
                    };
                    call_decision(core, &mut out, ReplyTo::new(token), key, part);
                    return out;
                }
            };
            let token = core.fly_call(token, RoutedCall::Control { key, target, revised: true });
            work.push(Work::Event(Event::Tasks(tasks::Event::Release {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                operation: tasks::Release { task: target, again, message },
            })));
        }
        NamedAction::Measure { target, revision } => {
            if let Some(part) = measure_refusal(core, &env.limits.tasks, key.task, target, revision) {
                call_decision(core, &mut out, ReplyTo::new(token), key, part);
                return out;
            }
            let token = core.fly_call(token, RoutedCall::Control { key, target, revised: true });
            work.push(Work::Event(Event::Tasks(tasks::Event::Measure {
                reply_to: ReplyTo::new(token),
                by: tasks::Party::Task(key.task),
                task: target,
            })));
        }
        NamedAction::Control { cancel } => {
            return super::controls::control(core, work, ReplyTo::new(token), key, cancel);
        }
        NamedAction::Amend { keep_parameters, target, revision, amendment } => {
            return named_amend(
                core,
                env,
                work,
                AgentAmend {
                    token,
                    key,
                    target,
                    revision,
                    amendment: keep_spec_parameters(core, target, amendment, keep_parameters),
                },
                crate::routing::party::amendment_reads::Evidence::new(&env.limits.tasks),
            );
        }
        NamedAction::DecideEscalation { task, revision, choice } => {
            return named_escalation(core, env, work, Call { to: ReplyTo::new(token), key }, task, revision, choice);
        }
        NamedAction::WithdrawProposal { proposal } => {
            if core.tasks.proposal(key.task, proposal).is_none() {
                core.handoff_call(key, CallHolder::Routed, CallHolder::DecidedProposal(proposal));
                let point = crate::loads::point::Point::Proposal { to: ReplyTo::new(token), key, proposal };
                if !crate::loads::try_point(core, &env.limits, &mut out, point) {
                    return crate::routing::party::ended::decided_proposal(
                        core,
                        ReplyTo::new(token),
                        key,
                        proposal,
                        None,
                        true,
                    );
                }
                return out;
            }
            let token = core.fly_call(token, RoutedCall::Withdraw { key, proposal, moved: None });
            work.push(Work::Event(Event::Tasks(tasks::Event::WithdrawProposal {
                reply_to: ReplyTo::new(token),
                proposer: key.task,
                proposal,
            })));
        }
        NamedAction::DecideProposal { proposer, proposal, choice } => {
            let call = Call { to: ReplyTo::new(token), key };
            return named_proposal_decide(core, env, work, call, proposer, proposal, choice);
        }
        NamedAction::Propose { action, reason, as_holder } => {
            let proposing = Proposing { to: ReplyTo::new(token), key, reason, as_holder };
            return named_propose(core, env, work, proposing, action, &mut None);
        }
        NamedAction::ProposeBatch { batch, reason, as_holder } => {
            let proposing = Proposing { to: ReplyTo::new(token), key, reason, as_holder };
            return propose_batch(core, env, work, proposing, batch, None);
        }
        NamedAction::ProposeGoal { goal, priority, reason } => {
            let proposing = Proposing { to: ReplyTo::new(token), key, reason, as_holder: true };
            return propose_batch(core, env, work, proposing, Box::new([*goal]), Some(priority));
        }
        NamedAction::Note { mut entry, recalled, records } => {
            if recalled.is_none() && entry.name != 0 {
                call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::NoteRefused(notes::Refusal::Exists));
                return out;
            }
            if recalled.is_some() && entry.name == 0 {
                call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::NoteRefused(notes::Refusal::Missing));
                return out;
            }
            if entry.name == 0 {
                let Some(name) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                    call_decision(
                        core,
                        &mut out,
                        ReplyTo::new(token),
                        key,
                        CallPart::NoteRefused(notes::Refusal::Full),
                    );
                    return out;
                };
                entry.name = name;
            }
            entry.author = notes::Author::Task { task: key.task, attempt: key.attempt };
            let owner = core.begin_note_route(NoteRoute::Call { to: ReplyTo::new(token), key, records });
            work.push(Work::Event(Event::Notes(notes::Event::Write { owner, entry, recalled })));
        }
        NamedAction::NoteLines { mut entry, recalled, range, records } => {
            if entry.name == 0 {
                call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::NoteRefused(notes::Refusal::Missing));
                return out;
            }
            entry.author = notes::Author::Task { task: key.task, attempt: key.attempt };
            let owner = core.begin_note_route(NoteRoute::Call { to: ReplyTo::new(token), key, records });
            work.push(Work::Event(Event::Notes(notes::Event::WriteLines { owner, entry, recalled, range })));
        }
        NamedAction::NoteDelete { .. } => unreachable!("party deletion routed after replay"),
        NamedAction::Recall { by, page } => {
            let records = Box::new([]);
            let owner = core.begin_note_route(NoteRoute::Call { to: ReplyTo::new(token), key, records });
            work.push(Work::Event(Event::Notes(notes::Event::Recall { owner, by, page })));
        }
    }
    out
}

fn measure_refusal(core: &Core, limits: &tasks::Limits, by: u64, task: u64, revision: u64) -> Option<CallPart> {
    let why = match core.tasks.task(task) {
        None => tasks::Refusal::Unknown,
        Some(_) if !core.tasks.may_control_task(task, by, limits.tasks) => tasks::Refusal::Reference,
        Some(row) if !row.offers_measure() => tasks::Refusal::State,
        Some(row) => match row.phase {
            tasks::Phase::Held { why, .. } if row.revisions.hold != revision => {
                let now = crate::settings::revisions::hold_change(row.revisions.hold, why);
                return Some(CallPart::Moved { task, now });
            }
            tasks::Phase::Held { .. } => return None,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {
                tasks::Refusal::State
            }
        },
    };
    Some(CallPart::ControlRefused(tasks::Problem { task: Some(task), why, blocked_by: None, capacity: None }))
}

fn malformed_recording(action: &NamedAction, limit: u32) -> bool {
    let messages = match action {
        NamedAction::MarkRead { messages } => {
            if messages.is_empty() {
                return true;
            }
            messages
        }
        NamedAction::Note { records, .. } | NamedAction::NoteLines { records, .. } => records,
        NamedAction::ReadTask { .. }
        | NamedAction::PersonRead(_)
        | NamedAction::Settings(_)
        | NamedAction::ReadProposal { .. }
        | NamedAction::ProposeSettings { .. }
        | NamedAction::Message { .. }
        | NamedAction::Introduce { .. }
        | NamedAction::Subscribe { .. }
        | NamedAction::Unsubscribe { .. }
        | NamedAction::Control { .. }
        | NamedAction::Release { .. }
        | NamedAction::Measure { .. }
        | NamedAction::Amend { .. }
        | NamedAction::DecideEscalation { .. }
        | NamedAction::WithdrawProposal { .. }
        | NamedAction::DecideProposal { .. }
        | NamedAction::Propose { .. }
        | NamedAction::ProposeBatch { .. }
        | NamedAction::ProposeGoal { .. }
        | NamedAction::NoteDelete { .. }
        | NamedAction::Recall { .. } => return false,
    };
    messages.len() > skein_lib::usize_of(limit) || messages.contains(&0)
}

pub(crate) fn named_tool_kind(action: &NamedAction) -> ToolKind {
    match action {
        NamedAction::ReadTask { .. } => ToolKind::Read,
        NamedAction::PersonRead(_) => ToolKind::Ask,
        NamedAction::Settings(_) | NamedAction::ReadProposal { .. } | NamedAction::ProposeSettings { .. } => {
            ToolKind::Settings
        }
        NamedAction::MarkRead { .. } => ToolKind::MarkRead,
        NamedAction::Message { target, kind: tasks::MessageKind::Answer { question }, .. } => {
            ToolKind::AnswerQuestion { target: *target, question: *question }
        }
        NamedAction::Message { target, .. } => ToolKind::Message { target: *target },
        NamedAction::Introduce { left, .. } => ToolKind::Message { target: *left },
        NamedAction::Subscribe { .. } | NamedAction::Unsubscribe { .. } => ToolKind::Subscribe,
        NamedAction::Control { .. }
        | NamedAction::Amend { .. }
        | NamedAction::Release { .. }
        | NamedAction::Measure { .. } => ToolKind::Control,
        NamedAction::DecideEscalation { .. }
        | NamedAction::WithdrawProposal { .. }
        | NamedAction::DecideProposal { .. } => ToolKind::Decide,
        NamedAction::Propose { .. } | NamedAction::ProposeBatch { .. } | NamedAction::ProposeGoal { .. } => {
            ToolKind::Propose
        }
        NamedAction::Note { entry, .. } | NamedAction::NoteLines { entry, .. } => {
            ToolKind::Note { scope: entry.scope.clone() }
        }
        NamedAction::NoteDelete { scope, .. } => ToolKind::Note { scope: scope.clone() },
        NamedAction::Recall { by, .. } => ToolKind::Recall { by: by.clone() },
    }
}

pub(crate) fn route_named_answer(core: &mut Core, to: ReplyTo, key: CallKey, part: CallPart) -> Queue<Request> {
    // A translated answer can complete a reserved application call or be the
    // whole decision of a new call (for example, a refusal of its input).
    if core.pending_calls.get(&key) == Some(&CallHolder::Application) {
        return route_pending_answer(core, to, key, part);
    }
    let mut out = Queue::uncapped();
    if let Some(to) = admit_named_call(core, &mut out, to, key) {
        call_decision(core, &mut out, to, key, part);
    }
    out
}

pub(crate) fn route_pending_answer(core: &mut Core, to: ReplyTo, key: CallKey, part: CallPart) -> Queue<Request> {
    let mut out = Queue::uncapped();
    call_decision(core, &mut out, to, key, part);
    out
}

pub(crate) fn route_refused_payload(
    core: &mut Core,
    work: &mut Queue<Work>,
    request: Token,
    problem: tasks::Problem,
    payload: Option<PayloadRefusal>,
) -> Queue<Request> {
    match payload {
        Some(PayloadRefusal::Turn { task, attempt, turn }) if problem.task == Some(task) => {
            work.push(Work::Event(Event::Fleet(fleet::Event::TurnBusy {
                run: Token::new(task),
                attempt: Token::new(attempt),
                turn,
            })));
        }
        Some(PayloadRefusal::Answer { task, attempt })
            if problem.task == Some(task) && core.current_proof(task, attempt) =>
        {
            if core.committed_terminal(task, attempt).is_some() {
                return Queue::uncapped();
            }
            let proof = core.proofs.get_mut(&task).expect("answer proof reserved");
            proof.terminal = Some(crate::TerminalRecord {
                task,
                attempt,
                cumulative: match proof.turn {
                    Some(turn) => turn.cumulative,
                    None => 0,
                },
                end: tasks::End::Failed(tasks::Class::Invalid),
            });
            work.push(Work::Event(Event::Tasks(tasks::Event::Activation {
                reads_project: false,
                reply_to: ReplyTo::new(request),
                task,
                attempt,
                end: tasks::End::Failed(tasks::Class::Invalid),
                saved: None,
                cause: tasks::Cause::Unpriced,
            })));
        }
        Some(PayloadRefusal::Turn { .. } | PayloadRefusal::Answer { .. }) | None => {}
    }
    Queue::uncapped()
}

pub(crate) struct AgentAmend {
    pub(crate) token: Token,
    pub(crate) key: CallKey,
    pub(crate) target: u64,
    pub(crate) revision: u64,
    pub(crate) amendment: tasks::Amendment,
}

pub(crate) fn named_amend(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    amend: AgentAmend,
    evidence: crate::routing::party::amendment_reads::Evidence,
) -> Queue<Request> {
    let AgentAmend { token, key, target, revision, mut amendment } = amend;
    let mut out = Queue::uncapped();
    if let Some(part) = core.authorize_tool(key, ToolKind::Control) {
        call_decision(core, &mut out, ReplyTo::new(token), key, part);
        return out;
    }
    if let Some(next) = crate::routing::party::amendment_reads::missing(
        core,
        &env.limits.tasks,
        tasks::Party::Task(key.task),
        target,
        amendment.dependencies.as_deref(),
        &evidence,
    ) {
        super::results::wait(
            core,
            work,
            &mut out,
            next,
            crate::routing::party::amendment_reads::AmendmentRead {
                replay: crate::routing::party::amendment_reads::Replay::Agent {
                    token,
                    key,
                    target,
                    revision,
                    amendment,
                },
                evidence,
            },
        );
        return out;
    }
    core.keep_models(target, &mut amendment);
    let stop_run = match core.task_amend_admit(key.task, target, &amendment, env.limits.tasks.depth, &env.limits.tasks)
    {
        Ok(stop_run) => stop_run,
        Err(crate::TaskAmendDenied::Refused { task, why }) => {
            call_decision(
                core,
                &mut out,
                ReplyTo::new(token),
                key,
                CallPart::ControlRefused(tasks::Problem { task: Some(task), why, blocked_by: None, capacity: None }),
            );
            return out;
        }
        Err(crate::TaskAmendDenied::Denied(answer)) => {
            call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::ControlDenied { answer });
            return out;
        }
    };
    let moved = match core.party_amend_revision(target, revision, &amendment) {
        Ok(()) => None,
        Err(jig_domain_people::Refusal::Moved { now }) => Some(now),
        Err(_) => unreachable!("admitted amendment target exists"),
    };
    if let Some(now) = moved {
        call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::Moved { task: target, now });
        return out;
    }
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        call_decision(
            core,
            &mut out,
            ReplyTo::new(token),
            key,
            CallPart::ControlRefused(tasks::Problem {
                task: Some(target),
                why: tasks::Refusal::Busy,
                blocked_by: None,
                capacity: None,
            }),
        );
        return out;
    };
    let current = core.tasks.task(target).expect("amend admission found target");
    let funder = crate::routing::amendment_funder(core, &current.authority, &amendment, tasks::Funder::Task(key.task));
    let token = core.fly_call(token, RoutedCall::Control { key, target, revised: true });
    work.push(Work::Event(Event::Tasks(tasks::Event::Amend {
        reads_project: false,
        operation: tasks::Amend { funder, task: target, message, stop_run, amendment: Box::new(amendment) },
        places: evidence.places.into_boxed(),
        reply_to: ReplyTo::new(token),
        by: tasks::Party::Task(key.task),
    })));
    out
}

/// Resolve host omission before the whole amendment is admitted (domain/tasks/changes.md).
fn keep_spec_parameters(core: &Core, target: u64, mut amendment: tasks::Amendment, keep: bool) -> tasks::Amendment {
    if keep
        && let Some(spec) = &mut amendment.spec
        && let Some(current) = core.tasks.task(target)
    {
        spec.parameters.clone_from(&current.spec.parameters);
    }
    amendment
}
