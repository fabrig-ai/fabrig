//! Task-child decisions routed to people, hosts and connectors
//! (`domain/engine.md`, section 4.4).

use super::children::append_people;
use super::refusals::{person_message_refusal, person_move_refusal, person_problem_refusal};
use super::runs::call_decision;
use super::view_routes::touch_task;
use crate::flight::{Flight, Internal};
use crate::routing::Work;
use crate::{
    Ask, CallPart, Core, CoreKey, CoreRecord, EscalationDecisionRecord, Event, GoalRoute, Held, Key, Limits, Now,
    PersonProposalRoute, PersonTaskRoute, Record, Request, RoutedCall, SentRoute, Write,
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

#[expect(clippy::too_many_lines, reason = "one exhaustive task vocabulary routes its bounded child decision")]
pub(super) fn tag_task_outputs(core: &mut Core, env: &Env<Limits>, mut child: Queue<tasks::Request>) -> Queue<Request> {
    let mut pending = Queue::uncapped();
    for _ in 0..child.len() {
        pending.push(child.pop().expect("task output count"));
    }
    let mut out = Queue::uncapped();
    for _ in 0..env.limits.routes {
        let Some(next) = pending.pop() else { break };
        let request = match next {
            tasks::Request::ProcedureRefused { task, step, problem } => {
                super::procedure_steps::append_problem(core, task, step, problem, &mut out);
                continue;
            }
            tasks::Request::Lifted { reply_to, released } => {
                let token = reply_to.into_token();
                let Some(Flight::PersonTask { request, route: PersonTaskRoute::Lift(hold) }) = core.land(token) else {
                    continue;
                };
                append_people(
                    core,
                    env,
                    people::Event::Decided { request, outcome: people::Outcome::Lifted { hold, released } },
                    &mut out,
                );
                continue;
            }
            tasks::Request::MovedToNewest { .. } => unreachable!("settings move consumes its counts"),
            tasks::Request::Released { revision } => {
                core.release_revision(env, revision, &mut out);
                continue;
            }
            tasks::Request::Erase { key } => Request::Write(Write::Erase(Key::Tasks(key))),
            tasks::Request::Relay { task, .. } => {
                let Some(relay) = crate::relays::next(core, task, env.limits.tasks.run_inbox) else { continue };
                Request::Held(Box::new(relay))
            }
            tasks::Request::PersonProposed { reply_to, proposal } => {
                let (event, stubs) = core.person_proposed(reply_to.into_token(), proposal);
                for stub in stubs {
                    let mut follow = Queue::uncapped();
                    crate::routing::step::step_tasks(core, env, tasks::Event::RememberStub { stub }, &mut follow);
                    for _ in 0..follow.len() {
                        pending.push(follow.pop().expect("proposal input stub output"));
                    }
                }
                append_people(core, env, event, &mut out);
                continue;
            }
            tasks::Request::PersonProposalDecided { reply_to, proposer, number, outcome } => {
                core.touch_party(proposer);
                if let Some(reply_to) = reply_to {
                    let event =
                        core.person_proposal_decided(reply_to.into_token(), proposer, number, outcome, env.wall);
                    append_people(core, env, event, &mut out);
                }
                continue;
            }
            tasks::Request::EscalationDecided { reply_to, task, revision, outcome } => {
                let token = reply_to.into_token();
                let flight = match core.land(token) {
                    Some(Flight::PersonEscalation(flight)) => flight,
                    Some(Flight::Call {
                        request,
                        route: RoutedCall::Escalation { key, task: named, revision: current, passed_up },
                        ..
                    }) => {
                        assert!(task == named && revision == current, "exact task escalation terminal");
                        call_decision(
                            core,
                            &mut out,
                            ReplyTo::new(request),
                            key,
                            match outcome {
                                tasks::EscalationOutcome::Passed { .. } if passed_up => CallPart::PassedUp { task },
                                tasks::EscalationOutcome::Released
                                | tasks::EscalationOutcome::Rejected
                                | tasks::EscalationOutcome::Passed { .. }
                                | tasks::EscalationOutcome::Stale
                                | tasks::EscalationOutcome::NoFurther
                                | tasks::EscalationOutcome::Limit => {
                                    CallPart::EscalationDecided { task, revision, outcome }
                                }
                            },
                        );
                        continue;
                    }
                    _ => continue,
                };
                assert!(flight.task == task && flight.revision == revision, "exact held choice terminal");
                let choice = match outcome {
                    tasks::EscalationOutcome::Released => people::EscalationChoice::Released,
                    tasks::EscalationOutcome::Rejected => people::EscalationChoice::Rejected,
                    tasks::EscalationOutcome::Passed { .. } => people::EscalationChoice::Passed,
                    tasks::EscalationOutcome::NoFurther => {
                        append_people(
                            core,
                            env,
                            people::Event::Decided {
                                request: flight.request,
                                outcome: people::Outcome::Refused(people::Refusal::NoFurther),
                            },
                            &mut out,
                        );
                        continue;
                    }
                    tasks::EscalationOutcome::Limit => {
                        append_people(
                            core,
                            env,
                            people::Event::Decided {
                                request: flight.request,
                                outcome: people::Outcome::Refused(people::Refusal::Limit {
                                    which: people::LimitKind::Words,
                                }),
                            },
                            &mut out,
                        );
                        continue;
                    }
                    tasks::EscalationOutcome::Stale => {
                        out.push(Request::Now(Box::new(Now::HistoricalEscalation {
                            request: flight.request,
                            person: flight.person,
                            project: flight.project,
                            task,
                            revision,
                        })));
                        continue;
                    }
                };
                out.push(Request::Write(Write::Save(Record::Core(CoreRecord::EscalationDecision(
                    EscalationDecisionRecord {
                        project: flight.project,
                        requester: flight.requester,
                        task,
                        revision,
                        by: flight.person,
                        at: env.wall,
                        decision: flight.decision,
                    },
                )))));
                append_people(
                    core,
                    env,
                    people::Event::Decided {
                        request: flight.request,
                        outcome: people::Outcome::EscalationDecided {
                            task,
                            revision,
                            by: flight.person,
                            at: env.wall,
                            choice,
                        },
                    },
                    &mut out,
                );
                continue;
            }
            tasks::Request::ResolveUncertain { task: _, entry, again } => {
                for connector in &core.connectors {
                    out.push(Request::Ask {
                        connector: *connector,
                        ask: Ask::Effect(crate::connector::Ask::ResolveUncertain { entry, again }),
                    });
                }
                continue;
            }
            tasks::Request::Sent { reply_to, task, word, .. } => {
                let request = reply_to.into_token();
                match core.sent(request, task, &word, env.limits.tasks.inbox_messages, env.limits.tasks.tasks) {
                    SentRoute::Call { request, key, message } => {
                        call_decision(core, &mut out, ReplyTo::new(request), key, CallPart::Sent { message });
                    }
                    SentRoute::Person(event) => append_people(core, env, *event, &mut out),
                }
                continue;
            }
            tasks::Request::RecurringDue { task, period, members } => {
                let Some(context) = core.tasks.delegation(task) else { continue };
                let Some(template) = core.tasks.recurring_template(task) else { continue };
                if members != u32::try_from(template.batch.len()).expect("bounded template") {
                    continue;
                }
                let mut asked = List::with_capacity(env.limits.tasks.batch);
                for member in &template.batch {
                    let executor = match member.executor {
                        tasks::Executor::Agent { charter } => authority::Executor::Charter(charter),
                        tasks::Executor::Procedure { code, .. } => authority::Executor::Procedure(code),
                        tasks::Executor::Person(_) => unreachable!("person template refused at admission"),
                    };
                    asked
                        .push(authority::Delegate {
                            executor,
                            authority: crate::translate::tasks_authority(&member.authority),
                            symbolic: Box::new([]),
                        })
                        .expect("bounded template");
                }
                let checked = core.connector_batch_admit(
                    context.project,
                    &context.authority,
                    tasks::Numbers { budget: context.authority.budget.spend, spent: 0, spent_below: 0, reserved: 0 },
                    context.tasks_left,
                    asked.into_boxed(),
                    &mut Queue::uncapped(),
                );
                if checked.answer != authority::Answer::Allow {
                    continue;
                }
                // No per-occurrence refusal outcome exists. Invalid account hints become
                // typed next-run waits on the created task (accounts.md, 4.3).
                if let Err(why) = core.recurring_accounts(context.project, &template.batch) {
                    if let tasks::Refusal::Account { .. } = why {
                        // Keep this occurrence; run_choice rechecks the same facts for its wait.
                    } else {
                        continue;
                    }
                }
                let mut numbers = List::with_capacity(env.limits.tasks.batch);
                for _ in 0..members {
                    let Some(number) = crate::fresh(&mut core.counters, crate::Family::Task) else { break };
                    numbers.push(number).expect("bounded recurring batch");
                }
                if numbers.len() == members {
                    let mut follow = Queue::uncapped();
                    crate::routing::step::step_tasks(
                        core,
                        env,
                        tasks::Event::RecurringBatch { task, period, numbers: numbers.into_boxed() },
                        &mut follow,
                    );
                    for _ in 0..follow.len() {
                        pending.push(follow.pop().expect("recurring output count"));
                    }
                }
                continue;
            }
            tasks::Request::Adopt { task, attempt, kept } => {
                core.adopt(env, task, attempt, kept);
                continue;
            }
            tasks::Request::Stop { task, attempt } => Request::Held(Box::new(Held::StopRun { task, attempt })),
            tasks::Request::RestoreRefused { .. } => {
                core.start_collect_failure();
                assert!(core.start_failure().is_some(), "tasks refusal names its fit");
                continue;
            }
            tasks::Request::Waiting { .. }
            | tasks::Request::WriterWaiting { task: None, .. }
            | tasks::Request::EscalationsInspected { .. }
            | tasks::Request::EscalationsRechecked { .. } => continue,
            tasks::Request::Taken { task, holdings } => {
                for holding in holdings {
                    match holding {
                        tasks::Holding::Write { resource, .. } => {
                            out.push(Request::Ask { connector: resource.connector, ask: Ask::Hold { task, resource } });
                        }
                        tasks::Holding::Slot { .. } => {}
                    }
                }
                continue;
            }
            tasks::Request::EndTopic { task, subscription, connector } => {
                Request::Ask { connector, ask: Ask::EndTopic { task, subscription } }
            }
            tasks::Request::WriterWaiting { task: Some(task), .. } => {
                if core.claiming.remove(&task).is_some() {
                    core.restore_replaced_proof(task);
                    out.push(Request::Now(Box::new(Now::DropAssignment { task })));
                    let mut follow = Queue::uncapped();
                    crate::routing::step::step_tasks(core, env, tasks::Event::PreparationFailed { task }, &mut follow);
                    for _ in 0..follow.len() {
                        pending.push(follow.pop().expect("writer wait output count"));
                    }
                }
                continue;
            }
            tasks::Request::EscalationNeeded { context } => {
                let mut follow = Queue::uncapped();
                if !core.route_waiting_escalation(env, &context, &mut follow) {
                    out.push(Request::Now(Box::new(Now::RestoreRefused)));
                    continue;
                }
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("escalation output count"));
                }
                continue;
            }
            tasks::Request::EscalationStalled { task, revision, holder } => {
                let Some(context) = core.tasks.escalation(task) else { continue };
                let tasks::Escalation::Waiting { revision: current, holder: old, .. } = context.escalation else {
                    continue;
                };
                if current != revision || old != holder {
                    continue;
                }
                let role = core.people.role(context.requester, context.project);
                let Some(next) = core.escalation_recipient_after(&env.limits, &context, role, Some(holder)) else {
                    continue;
                };
                let Some(entry) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                    out.push(Request::Now(Box::new(Now::RestoreRefused)));
                    continue;
                };
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(
                    core,
                    env,
                    tasks::Event::RoutedEscalation { task, revision, holder: next, entry },
                    &mut follow,
                );
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("stalled escalation output count"));
                }
                continue;
            }
            tasks::Request::ProposalStalled { proposer, proposal, holder } => {
                let Some(row) = core.tasks.proposal(proposer, proposal) else { continue };
                if crate::proposals::placed_release(&row.action) {
                    continue;
                }
                let tasks::ProposalState::Pending { holder: current, .. } = row.state else { continue };
                if current != holder {
                    continue;
                }
                let Some(next) = core.proposal_holder(&env.limits, proposer, &row.action, Some(holder), env.wall)
                else {
                    continue;
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
                continue;
            }
            tasks::Request::ProposalRerouteNeeded { proposer, proposal } => {
                let Some(row) = core.tasks.proposal(proposer, proposal) else { continue };
                if crate::proposals::placed_release(&row.action) {
                    continue;
                }
                let tasks::ProposalState::Pending { holder: current, .. } = row.state else { continue };
                let Some(next) = core.proposal_holder(&env.limits, proposer, &row.action, None, env.wall) else {
                    continue;
                };
                if current == next {
                    continue;
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
                continue;
            }
            tasks::Request::Made { reply_to, tasks } => {
                let request = reply_to.into_token();
                let flight = core.land(request);
                if let Some(Flight::PersonProposal(PersonProposalRoute::Accepting {
                    person, proposer, proposal, ..
                })) = &flight
                {
                    let first = *tasks.first().expect("an accepted batch made tasks");
                    let last = *tasks.last().expect("an accepted batch made tasks");
                    let count = match last.checked_sub(first) {
                        Some(count) => count.checked_add(1),
                        None => None,
                    };
                    assert_eq!(count, u64::try_from(tasks.len()).ok(), "batch numbers are consecutive");
                    let project = core.tasks.task(*proposer).expect("accepted proposer").project;
                    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::ProposalDecision(
                        crate::ProposalDecisionRecord {
                            project,
                            proposer: people::Proposer::Task(*proposer),
                            proposal: *proposal,
                            kind: tasks::ProposalKind::Batch,
                            by: *person,
                            at: env.wall,
                            choice: people::ProposalChoice::Accepted,
                            created: Some((first, last)),
                        },
                    )))));
                }
                match core.made_flight(request, flight, tasks) {
                    crate::MadeRoute::Internal => {}
                    crate::MadeRoute::Tasks(event) => {
                        let mut follow = Queue::uncapped();
                        crate::routing::step::step_tasks(core, env, *event, &mut follow);
                        for _ in 0..follow.len() {
                            pending.push(follow.pop().expect("made output count"));
                        }
                    }
                    crate::MadeRoute::Person { event, stubs } => {
                        for stub in stubs {
                            let mut follow = Queue::uncapped();
                            crate::routing::step::step_tasks(
                                core,
                                env,
                                tasks::Event::RememberStub { stub },
                                &mut follow,
                            );
                            for _ in 0..follow.len() {
                                pending.push(follow.pop().expect("stub output count"));
                            }
                        }
                        append_people(core, env, *event, &mut out);
                    }
                    crate::MadeRoute::Delegated { key, tasks, stubs, given } => {
                        for stub in stubs {
                            let mut follow = Queue::uncapped();
                            crate::routing::step::step_tasks(
                                core,
                                env,
                                tasks::Event::RememberStub { stub },
                                &mut follow,
                            );
                            for _ in 0..follow.len() {
                                pending.push(follow.pop().expect("stub output count"));
                            }
                        }
                        call_decision(
                            core,
                            &mut out,
                            ReplyTo::new(request),
                            key,
                            CallPart::Delegated(crate::as_allowed::made(&tasks, given, env.limits.tasks.batch)),
                        );
                    }
                }
                continue;
            }
            tasks::Request::ProposalDecided { reply_to, proposer, number, outcome } => {
                let token = reply_to.into_token();
                let (token, route) = match core.land(token) {
                    Some(Flight::PersonProposal(route)) => {
                        if let Some(event) =
                            core.task_proposal_decided_for_person(token, proposer, number, outcome, env.wall, route)
                        {
                            append_people(core, env, event, &mut out);
                        }
                        continue;
                    }
                    Some(Flight::Call { request, route, .. }) => (request, route),
                    _ => continue,
                };
                if let RoutedCall::Withdraw { key, moved: Some((task, now)), .. } = route {
                    assert!(outcome == tasks::ProposalOutcome::Withdrawn, "stale proposal withdrawn");
                    call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::Moved { task, now });
                    continue;
                }
                let key = match route {
                    RoutedCall::Decide { key, proposal } | RoutedCall::Withdraw { key, proposal, .. }
                        if proposal == number =>
                    {
                        key
                    }
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
                call_decision(
                    core,
                    &mut out,
                    ReplyTo::new(token),
                    key,
                    CallPart::ProposalDecided { proposal: number, outcome },
                );
                continue;
            }
            tasks::Request::CompoundRefused { reply_to, at, problem } => {
                let request = reply_to.into_token();
                let Some(Flight::PersonTask { request, route: PersonTaskRoute::Compound { .. } }) = core.land(request)
                else {
                    continue;
                };
                let outcome = person_problem_refusal(core, problem);
                let outcome = if let people::Outcome::Refused(why) = outcome {
                    people::Outcome::CompoundRefused { at, why }
                } else {
                    outcome
                };
                append_people(core, env, people::Event::Decided { request, outcome }, &mut out);
                continue;
            }
            tasks::Request::Done { reply_to } => {
                let token = reply_to.into_token();
                match core.land(token) {
                    Some(flight) => tag_tasks(core, env, token, flight, Ok(()), &mut pending, &mut out),
                    None => {
                        if let Some(attempt) = core.claiming.remove(&token.raw()) {
                            core.claim_done(token.raw(), attempt, &mut out);
                        }
                    }
                }
                continue;
            }
            tasks::Request::Save { record } => {
                super::settings::proposal_saved(core, &record, &mut out);
                if let tasks::Stored::Activity(line) = &record {
                    let _retained = core.remember_question(line);
                }
                if let tasks::Stored::Live(task) | tasks::Stored::Ended(task) = &record {
                    core.prune_questions(task.number);
                }
                if let Some(archive) = crate::proposals::decision_record(&record, env.wall) {
                    // Accepted task batches already saved their member numbers
                    // with Made; the history row must not replace that evidence.
                    let by_task = match archive.proposer {
                        people::Proposer::Task(_) => true,
                        people::Proposer::Party(_) => false,
                    };
                    let made = archive.kind == tasks::ProposalKind::Batch
                        && archive.choice == people::ProposalChoice::Accepted
                        && by_task;
                    if !made {
                        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::ProposalDecision(archive)))));
                    }
                }
                let record = match record {
                    tasks::Stored::Ended(mut task) => {
                        let position = crate::fresh(&mut core.counters, crate::Family::Message)
                            .expect("ending position preflighted before mutation");
                        task.result_position = position;
                        assert!(core.ending_positions.insert(task.number, position) == Ok(None), "one ending position");
                        match task.requester {
                            tasks::Party::Person(person) => {
                                core.touch_party(person);
                                core.people.remember_result(
                                    &env.limits.people,
                                    person,
                                    people::ResultRef {
                                        task: task.number,
                                        project: task.project,
                                        at: task.ended_at.expect("ended row has its durable time"),
                                        position,
                                        forgettable: (task.flags.retention == tasks::Retention::ForgottenOnceRead)
                                            .then_some(people::Forgettable {
                                                label: task.label,
                                                listed: task.flags.listed,
                                                turns_read: core
                                                    .people
                                                    .conversation_seen(person, task.number)
                                                    .unwrap_or(0)
                                                    >= task.last_reply,
                                                last_reply: task.flags.conversation.then_some(task.last_reply),
                                            }),
                                    },
                                );
                            }
                            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
                        }
                        tasks::Stored::Ended(task)
                    }
                    tasks::Stored::History(ref row) => {
                        if let Some(proposal) = &row.proposal {
                            let terminal = match proposal.state {
                                tasks::ProposalState::Pending { .. } => false,
                                tasks::ProposalState::Accepted { .. }
                                | tasks::ProposalState::Rejected { .. }
                                | tasks::ProposalState::Withdrawn => true,
                            };
                            if terminal {
                                match proposal.action {
                                    tasks::ProposalAction::Effect { connector, .. }
                                    | tasks::ProposalAction::ProcedureEffect { connector, .. } => {
                                        out.push(Request::Ask {
                                            connector,
                                            ask: Ask::Effect(crate::connector::Ask::DropProposal {
                                                proposal: proposal.number,
                                            }),
                                        });
                                    }
                                    tasks::ProposalAction::Settings { .. }
                                    | tasks::ProposalAction::Batch(_)
                                    | tasks::ProposalAction::Amend { .. }
                                    | tasks::ProposalAction::Widen { .. }
                                    | tasks::ProposalAction::ReleasePlaced { .. }
                                    | tasks::ProposalAction::Release { .. } => {}
                                }
                            }
                        }
                        record
                    }
                    tasks::Stored::Live(_)
                    | tasks::Stored::Milestone(_)
                    | tasks::Stored::Activity(_)
                    | tasks::Stored::Writer(_)
                    | tasks::Stored::Result(_)
                    | tasks::Stored::EndedByParty(_)
                    | tasks::Stored::EndedByProject(_)
                    | tasks::Stored::EndedDelegate(_)
                    | tasks::Stored::Delegate(_)
                    | tasks::Stored::Update(_)
                    | tasks::Stored::Pool(_)
                    | tasks::Stored::ClosedLedger(_)
                    | tasks::Stored::DecidedProposal(_)
                    | tasks::Stored::Ledger(_)
                    | tasks::Stored::PlacedHold(_)
                    | tasks::Stored::Stub(_)
                    | tasks::Stored::PersonProposal(_) => record,
                };
                crate::indexes::save(core, &record, env.wall, &mut out);
                if let tasks::Stored::Live(task) | tasks::Stored::Ended(task) = &record {
                    if let Some(to) = core.conversation_requester(task)
                        && let Some(from) = core.people.chat_owner(task.number)
                        && from != to
                    {
                        append_people(
                            core,
                            env,
                            people::Event::MoveChat { task: task.number, project: task.project, from, to },
                            &mut out,
                        );
                    }
                    let seen = match core.conversation_requester(task) {
                        Some(party) => core.people.conversation_seen(party, task.number),
                        None => None,
                    };
                    if let Some(party) = core.conversation_requester(task) {
                        let live = match &record {
                            tasks::Stored::Live(_) => true,
                            tasks::Stored::Ended(_) => false,
                            tasks::Stored::Activity(_)
                            | tasks::Stored::Milestone(_)
                            | tasks::Stored::PersonProposal(_)
                            | tasks::Stored::History(_)
                            | tasks::Stored::PlacedHold(_)
                            | tasks::Stored::Stub(_)
                            | tasks::Stored::Ledger(_)
                            | tasks::Stored::Writer(_)
                            | tasks::Stored::Result(_)
                            | tasks::Stored::EndedByParty(_)
                            | tasks::Stored::EndedByProject(_)
                            | tasks::Stored::EndedDelegate(_)
                            | tasks::Stored::Delegate(_)
                            | tasks::Stored::Update(_)
                            | tasks::Stored::Pool(_)
                            | tasks::Stored::ClosedLedger(_)
                            | tasks::Stored::DecidedProposal(_) => unreachable!("matched task header"),
                        };
                        append_people(
                            core,
                            env,
                            people::Event::ChatState {
                                party,
                                project: task.project,
                                task: task.number,
                                last_reply: task.last_reply,
                                live,
                                seen: None,
                                replies: crate::inbox::reply_refs(&env.limits, task, party),
                            },
                            &mut out,
                        );
                    }
                    if let tasks::Stored::Ended(_) = &record {
                        core.cache_archived_chat(&env.limits, task, seen);
                    }
                    match task.phase {
                        tasks::Phase::Ended(_) => {
                            drop(core.workspace_credentials.remove(&task.number));
                        }
                        tasks::Phase::Waiting
                        | tasks::Phase::Active(_)
                        | tasks::Phase::Closing(_)
                        | tasks::Phase::Held { .. } => {}
                    }
                    let stale = match core.contexts.get(&task.number) {
                        Some(context) => {
                            task.phase != tasks::Phase::Active(tasks::Active::Preparing)
                                || core.tasks.run_message(task.number) != Some(context.last_message)
                        }
                        None => false,
                    };
                    if stale {
                        super::result_reads::clear(core, task.number);
                        drop(core.contexts.remove(&task.number));
                        drop(core.dependency_results.remove(&task.number));
                        drop(core.transcripts.remove(&task.number));
                        let mut abandoned = Queue::uncapped();
                        crate::routing::step::step_brief(
                            core,
                            env,
                            brief::GatherEvent::Abandon { brief: Token::new(task.number) },
                            &mut abandoned,
                        );
                        for _ in 0..abandoned.len() {
                            match abandoned.pop().expect("abandoned brief output count") {
                                brief::GatherRequest::Drop { connector, section } => {
                                    out.push(Request::Ask { connector, ask: Ask::Drop { section } });
                                }
                                brief::GatherRequest::Gather { .. }
                                | brief::GatherRequest::CutTo { .. }
                                | brief::GatherRequest::Complete { .. }
                                | brief::GatherRequest::Failed { .. }
                                | brief::GatherRequest::Refused { .. } => {
                                    unreachable!("abandon only releases connector sections")
                                }
                            }
                        }
                        if task.phase == tasks::Phase::Active(tasks::Active::Preparing) {
                            let mut follow = Queue::uncapped();
                            crate::routing::step::step_tasks(
                                core,
                                env,
                                tasks::Event::PreparationFailed { task: task.number },
                                &mut follow,
                            );
                            for _ in 0..follow.len() {
                                pending.push(follow.pop().expect("stale preparation output count"));
                            }
                        }
                    }
                    crate::projections::save(core, &env.limits, task, &mut out);
                    touch_task(core, &env.limits, task);
                }
                match &record {
                    tasks::Stored::Ended(task) => touch_task(core, &env.limits, task),
                    tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row) => {
                        core.touch_deployment();
                        core.touch_project_objects(row.project);
                        core.touch_party(row.proposer);
                    }
                    tasks::Stored::Ledger(row) | tasks::Stored::ClosedLedger(row) => match row.funder {
                        tasks::Funder::Pool { project, person, .. } => {
                            core.touch_project_objects(project);
                            core.touch_party(person);
                        }
                        tasks::Funder::Period { project, .. } => core.touch_project_parties(project),
                        tasks::Funder::Recurring { project, .. } => core.touch_project_objects(project),
                        tasks::Funder::Task(_) => {}
                    },
                    tasks::Stored::Live(_)
                    | tasks::Stored::Writer(_)
                    | tasks::Stored::Result(_)
                    | tasks::Stored::EndedByParty(_)
                    | tasks::Stored::EndedByProject(_)
                    | tasks::Stored::EndedDelegate(_)
                    | tasks::Stored::Delegate(_)
                    | tasks::Stored::Update(_)
                    | tasks::Stored::Pool(_)
                    | tasks::Stored::History(_)
                    | tasks::Stored::Milestone(_)
                    | tasks::Stored::Activity(_)
                    | tasks::Stored::PlacedHold(_)
                    | tasks::Stored::Stub(_) => {}
                }
                let waiting = match &record {
                    tasks::Stored::Live(task) | tasks::Stored::Ended(task) => {
                        Some((task.number, core.waiting_entries(&env.limits, task)))
                    }
                    tasks::Stored::PersonProposal(row) | tasks::Stored::DecidedProposal(row) => {
                        if row.first() == tasks::PROPOSAL_WITHOUT_TASK {
                            None
                        } else {
                            Some((row.first(), core.person_proposal_entries(row)))
                        }
                    }
                    tasks::Stored::Ledger(_)
                    | tasks::Stored::Writer(_)
                    | tasks::Stored::Result(_)
                    | tasks::Stored::EndedByParty(_)
                    | tasks::Stored::EndedByProject(_)
                    | tasks::Stored::EndedDelegate(_)
                    | tasks::Stored::Delegate(_)
                    | tasks::Stored::Update(_)
                    | tasks::Stored::Pool(_)
                    | tasks::Stored::ClosedLedger(_)
                    | tasks::Stored::History(_)
                    | tasks::Stored::Milestone(_)
                    | tasks::Stored::Activity(_)
                    | tasks::Stored::PlacedHold(_)
                    | tasks::Stored::Stub(_) => None,
                };
                match &record {
                    tasks::Stored::Milestone(row) => crate::projections::history(
                        core,
                        &env.limits,
                        row.task,
                        crate::ProjectionMilestone {
                            identity: crate::MilestoneId::Lifecycle { task: row.task, position: row.position },
                            phase: Some(row.phase.clone()),
                            change: None,
                            words: Box::new([]),
                        },
                        &mut out,
                    ),
                    tasks::Stored::History(row) => crate::projections::history(
                        core,
                        &env.limits,
                        row.task,
                        crate::ProjectionMilestone {
                            identity: crate::MilestoneId::Revision { task: row.task, revision: row.revision },
                            phase: None,
                            change: Some(row.change),
                            words: row.reason.clone(),
                        },
                        &mut out,
                    ),
                    tasks::Stored::Activity(_)
                    | tasks::Stored::Live(_)
                    | tasks::Stored::Ended(_)
                    | tasks::Stored::PersonProposal(_)
                    | tasks::Stored::Ledger(_)
                    | tasks::Stored::Writer(_)
                    | tasks::Stored::Result(_)
                    | tasks::Stored::EndedByParty(_)
                    | tasks::Stored::EndedByProject(_)
                    | tasks::Stored::EndedDelegate(_)
                    | tasks::Stored::Delegate(_)
                    | tasks::Stored::Update(_)
                    | tasks::Stored::Pool(_)
                    | tasks::Stored::ClosedLedger(_)
                    | tasks::Stored::DecidedProposal(_)
                    | tasks::Stored::PlacedHold(_)
                    | tasks::Stored::Stub(_) => {}
                }
                out.push(Request::Write(Write::Save(Record::Tasks(record))));
                if let Some((task, entries)) = waiting {
                    append_people(core, env, people::Event::Waiting { task, entries }, &mut out);
                }
                continue;
            }
            tasks::Request::Acknowledged { reply_to, task, attempt, .. } => {
                let token = reply_to.into_token();
                match core.land(token) {
                    Some(Flight::Internal(Internal::Acknowledge)) => {}
                    Some(_) | None => out.push(Request::Now(Box::new(Now::DropPayload { payload: token }))),
                }
                if let Some(proof) = core.proofs.get(&task) {
                    if let Some(terminal) = &proof.terminal {
                        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Terminal(terminal.clone())))));
                    }
                    out.push(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof.clone())))));
                }
                out.push(Request::Held(Box::new(Held::TaskTerminalAcknowledged { task, attempt })));
                for &connector in core.connectors.as_ref() {
                    out.push(Request::Ask { connector, ask: Ask::Lost { task, attempt } });
                }
                continue;
            }
            tasks::Request::TurnAcknowledged { reply_to, task, attempt, turn, accepted } => {
                out.push(Request::Now(Box::new(Now::AcceptedTurn {
                    payload: reply_to.into_token(),
                    task,
                    attempt,
                    turn,
                    accepted,
                })));
                continue;
            }
            tasks::Request::Ended { task, requester, ending } => {
                let _pending = core.pending_note_briefs.remove(&task);
                let position = core.ending_positions.remove(&task).expect("ended row assigned its result position");
                let mut ended = super::view_routes::route_view(
                    core,
                    env,
                    jig_domain_views::Event::RunEnded { run: Token::new(task) },
                );
                for _ in 0..ended.len() {
                    let request = ended.pop().expect("ended run outputs");
                    out.push(request);
                }
                for key in core.retire_calls(task, u64::MAX, u32::MAX, env.limits.call_records) {
                    out.push(Request::Write(Write::Erase(Key::Core(CoreKey::Call(key)))));
                }
                drop(core.proofs.remove(&task));
                out.push(Request::Write(Write::Erase(Key::Core(CoreKey::RunProof(task)))));
                match requester {
                    tasks::Party::Person(person) => {
                        out.push(Request::Held(Box::new(Held::Result { person, task, words: ending_words(ending) })));
                    }
                    tasks::Party::Task(parent) => {
                        let (kind, words) = result_notice(ending);
                        let mut follow = Queue::uncapped();
                        crate::routing::step::step_tasks(
                            core,
                            env,
                            tasks::Event::DelegateResult {
                                task: parent,
                                word: tasks::Word {
                                    number: position,
                                    from: tasks::Party::Task(task),
                                    kind: tasks::MessageKind::Result(kind),
                                    words,
                                    at: env.wall,
                                    hits: 1,
                                    eligible: false,
                                },
                            },
                            &mut follow,
                        );
                        for _ in 0..follow.len() {
                            pending.push(follow.pop().expect("delegate result output count"));
                        }
                    }
                    tasks::Party::Deployment { .. } => {}
                }
                continue;
            }
            tasks::Request::Refused { reply_to, problem } => {
                let token = reply_to.into_token();
                if let Some(flight) = core.land(token) {
                    tag_tasks(core, env, token, flight, Err(problem), &mut pending, &mut out);
                    continue;
                }
                if let Some((key, _stubs, _given)) = core.delegating.remove(&token) {
                    call_decision(core, &mut out, ReplyTo::new(token), key, CallPart::DelegationRefused(problem));
                    continue;
                }
                if core.saying.remove(&token).is_some() {
                    append_people(
                        core,
                        env,
                        people::Event::Decided { request: token, outcome: person_message_refusal(core, problem) },
                        &mut out,
                    );
                    continue;
                }
                if core.claiming.remove(&token.raw()).is_some() {
                    core.restore_replaced_proof(token.raw());
                    out.push(Request::Now(Box::new(Now::DropAssignment { task: token.raw() })));
                    let mut follow = Queue::uncapped();
                    crate::routing::step::step_tasks(
                        core,
                        env,
                        tasks::Event::PreparationFailed { task: token.raw() },
                        &mut follow,
                    );
                    for _ in 0..follow.len() {
                        pending.push(follow.pop().expect("failed preparation output count"));
                    }
                    continue;
                }
                Request::Now(Box::new(Now::RefusedPayload { request: token, problem }))
            }
            tasks::Request::PersonStallDue { task } => {
                let message =
                    crate::fresh(&mut core.counters, crate::Family::Message).expect("person stall number admitted");
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(
                    core,
                    env,
                    tasks::Event::PersonStallNotice { task, message },
                    &mut follow,
                );
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("person stall output count"));
                }
                continue;
            }
            tasks::Request::QuestionOwedDue { task } => {
                let message = crate::fresh(&mut core.counters, crate::Family::Message)
                    .expect("question reminder number admitted");
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(
                    core,
                    env,
                    tasks::Event::QuestionOwedNotice { task, message },
                    &mut follow,
                );
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("question notice output count"));
                }
                continue;
            }
            tasks::Request::QuestionEndedDue { task, questions } => {
                let mut notices = List::uncapped();
                for question in questions {
                    let message = crate::fresh(&mut core.counters, crate::Family::Message)
                        .expect("question notice number admitted");
                    notices.push(tasks::QuestionNotice { question, message }).expect("bounded question batch");
                }
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(
                    core,
                    env,
                    tasks::Event::QuestionEndedNotice { task, notices: notices.into_boxed() },
                    &mut follow,
                );
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("question notice output count"));
                }
                continue;
            }
            tasks::Request::MetDue { task, dependency, removed, by } => {
                let message =
                    crate::fresh(&mut core.counters, crate::Family::Message).expect("met notice number admitted");
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(
                    core,
                    env,
                    tasks::Event::MetNotice { task, dependency, removed, by, message },
                    &mut follow,
                );
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("met notice output count"));
                }
                continue;
            }
            tasks::Request::Notify { task, subscription, target, state, words } => {
                let event = core.notice(task, subscription, target, state, words, env.wall);
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(core, env, event, &mut follow);
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("notice output count"));
                }
                continue;
            }
            tasks::Request::Timer { task, subscription } => {
                let event = core.notice_timer(task, subscription, env.wall);
                let mut follow = Queue::uncapped();
                crate::routing::step::step_tasks(core, env, event, &mut follow);
                for _ in 0..follow.len() {
                    pending.push(follow.pop().expect("timer output count"));
                }
                continue;
            }
            tasks::Request::Close { task, ending } => {
                let ancestor_number = core.tasks.root(task).unwrap_or(task);
                for &connector in core.connectors.as_ref() {
                    core.closing_connectors.insert((task, connector), ()).expect("bounded closing connectors");
                    out.push(Request::Ask {
                        connector,
                        ask: Ask::Close { task, root: ancestor_number, ending: ending.clone() },
                    });
                }
                continue;
            }
            tasks::Request::Release { task, ending } => {
                let ancestor_number = core.tasks.root(task).unwrap_or(task);
                let mut entries =
                    List::with_capacity(u32::try_from(core.connectors.len()).expect("connector count fits u32"));
                for &connector in core.connectors.as_ref() {
                    let Some(entry) = crate::fresh(&mut core.counters, crate::Family::ConnectorRow) else {
                        let mut follow = Queue::uncapped();
                        crate::routing::step::step_tasks(
                            core,
                            env,
                            tasks::Event::Hold { task, why: tasks::Hold::Budget },
                            &mut follow,
                        );
                        for _ in 0..follow.len() {
                            pending.push(follow.pop().expect("release budget output count"));
                        }
                        break;
                    };
                    entries.push((connector, entry)).expect("one outbox entry per connector");
                }
                if entries.len() != u32::try_from(core.connectors.len()).expect("connector count fits u32") {
                    continue;
                }
                for &(connector, entry) in &entries {
                    out.push(Request::Ask {
                        connector,
                        ask: Ask::Release { task, root: ancestor_number, ending: ending.clone(), entry },
                    });
                }
                continue;
            }
            tasks::Request::Activate { context } => {
                if core.pending_note_briefs.get(&context.task) == Some(&super::NoteBrief::Activation) {
                    let _pending = core.pending_note_briefs.remove(&context.task);
                    super::children::remember_due(core, context);
                    continue;
                }
                Request::Now(Box::new(Now::Activate { context }))
            }
        };
        out.push(request);
    }
    let holders = core.tasks.live_agent_tasks();
    if holders != core.settings_holders {
        core.settings_holders = holders;
        core.touch_settings();
    }
    let capacity_mark = core.tasks.capacity_mark();
    if capacity_mark != core.task_capacity_mark || capacity_mark == u64::MAX {
        core.task_capacity_mark = capacity_mark;
        core.touch_deployment();
    }
    assert!(pending.is_empty(), "task_routes_drained");
    out
}

fn route_proposal(core: &mut Core, env: &Env<Limits>, event: tasks::Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let tasks::Event::Make { batch, .. } | tasks::Event::MakeResultFollowups { batch, .. } = &event
        && let Err(problem) = core.batch_accounts(batch)
    {
        let (tasks::Event::Make { reply_to, .. } | tasks::Event::MakeResultFollowups { reply_to, .. }) = event else {
            unreachable!("checked batch event")
        };
        out.push(tasks::Request::Refused { reply_to, problem });
        return tag_task_outputs(core, env, out);
    }
    if let tasks::Event::Amend { task, amendment, funder, .. } = &event
        && let Err(why) = core.amendment_account(*task, amendment, *funder)
    {
        let problem = tasks::Problem::new(Some(*task), why);
        let tasks::Event::Amend { reply_to, .. } = event else { unreachable!("checked amendment event") };
        out.push(tasks::Request::Refused { reply_to, problem });
        return tag_task_outputs(core, env, out);
    }
    crate::routing::step::step_tasks(core, env, event, &mut out);
    tag_task_outputs(core, env, out)
}

#[expect(clippy::boxed_local, reason = "the moved route retains its owned event payload")]
pub(super) fn route_connector_repair(
    core: &mut Core,
    work: &mut Queue<Work>,
    seed: Box<crate::connector::RepairSeed>,
    executor: tasks::Executor,
    spec: tasks::Spec,
    contract: tasks::Contract,
    holdings: Box<[tasks::Holding]>,
) -> Queue<Request> {
    if seed.open_period {
        work.push(Work::Event(Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(core.internal(Internal::Admission)),
            project: seed.project,
            period: seed.period,
            budget: seed.period_budget,
        })));
    }
    work.push(Work::Event(Event::Tasks(tasks::Event::Make {
        reply_to: ReplyTo::new(core.internal(Internal::Repair)),
        creator: tasks::Party::Deployment { project: seed.project },
        batch: Box::new([tasks::New {
            deadline: jig_domain_tasks::Deadline::Named,
            label: 0,
            flags: jig_domain_tasks::Flags::PLAIN,
            number: seed.number,
            project: seed.project,
            executor,
            llm: None,
            spec,
            contract,
            numbers: tasks::Numbers { budget: seed.budget, spent: 0, spent_below: 0, reserved: 0 },
            authority: seed.authority,
            funder: tasks::Funder::Period { project: seed.project, period: seed.period },
            dependencies: Box::new([]),
            holdings,
            wake: tasks::WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        }]),
    })));
    Queue::uncapped()
}

pub(super) fn route_tasks(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    mut event: tasks::Event,
) -> Queue<Request> {
    if let Some(token) = crate::flight::request_token(&mut event)
        && let Some(Flight::PersonProposal(_)) = core.flights.get(skein_lib::Id::from_token(token))
    {
        return route_proposal(core, env, event);
    }
    if let tasks::Event::Procedure { task, step, .. } = &event {
        return super::procedure_steps::apply(core, env, *task, *step, event);
    }
    let mut out = Queue::uncapped();
    if let tasks::Event::Amend { task, amendment, funder, .. } = &event
        && let Err(why) = core.amendment_account(*task, amendment, *funder)
    {
        let problem = tasks::Problem::new(Some(*task), why);
        let tasks::Event::Amend { reply_to, .. } = event else { unreachable!("checked amendment event") };
        out.push(tasks::Request::Refused { reply_to, problem });
        return tag_task_outputs(core, env, out);
    }
    if let tasks::Event::Amend { task, amendment, .. } = &event
        && let Some(tasks::Tracking::Tracked { .. }) = amendment.tracked
        && let Some(row) = core.tasks.task(*task)
        && row.tracked.is_none()
        && core.projections.len() >= env.limits.tasks.tasks
    {
        let tasks::Event::Amend { task, reply_to, .. } = event else { unreachable!("checked amendment") };
        out.push(tasks::Request::Refused { reply_to, problem: tasks::Problem::new(Some(task), tasks::Refusal::Goals) });
        return tag_task_outputs(core, env, out);
    }
    if let tasks::Event::Make { batch, .. } = &event {
        let mut tracked = 0_u32;
        for row in batch {
            if row.tracked.is_some() {
                tracked = tracked.checked_add(1).expect("bounded make batch");
            }
        }
        if tracked > env.limits.tasks.tasks.saturating_sub(core.projections.len()) {
            let tasks::Event::Make { reply_to, creator, batch } = event else { unreachable!("checked creation event") };
            let capacity = tasks::Capacity::batch(
                creator,
                &batch,
                None,
                tasks::CapacityCause::Tasks { limit: env.limits.tasks.tasks },
            );
            core.tasks.remember_capacity(env.wall, capacity);
            out.push(tasks::Request::Refused {
                reply_to,
                problem: tasks::Problem::capacity(None, tasks::Refusal::Live, capacity),
            });
            return tag_task_outputs(core, env, out);
        }
    }
    let event = if let tasks::Event::Make { reply_to, creator, mut batch } = event {
        if let Err(problem) = core.resolve_llm(&mut batch) {
            out.push(tasks::Request::Refused { reply_to, problem });
            return tag_task_outputs(core, env, out);
        }
        match crate::resources::defer(
            core,
            env,
            reply_to,
            crate::resources::Kind::Creation { creator },
            batch,
            &mut out,
        ) {
            Some(event) => event,
            None => {
                return tag_task_outputs(core, env, out);
            }
        }
    } else if let tasks::Event::MakeResultFollowups { reply_to, proposer, proposal, mut batch } = event {
        if let Err(problem) = core.resolve_llm(&mut batch) {
            out.push(tasks::Request::Refused { reply_to, problem });
            return tag_task_outputs(core, env, out);
        }
        match crate::resources::defer(
            core,
            env,
            reply_to,
            crate::resources::Kind::Result { proposer, proposal },
            batch,
            &mut out,
        ) {
            Some(event) => event,
            None => {
                return tag_task_outputs(core, env, out);
            }
        }
    } else {
        event
    };
    let event = question_event(core, event);
    crate::routing::step::step_tasks(core, env, event, &mut out);
    crate::resources::resume(core, work);
    tag_task_outputs(core, env, out)
}
// Direct-answer authority stays inside the core's question admission.
fn question_event(core: &Core, event: tasks::Event) -> tasks::Event {
    if let tasks::Event::Message { reply_to, project, task, word } = event {
        let person = match core.tasks.task(task) {
            Some(row) => match row.requester {
                tasks::Party::Person(person) => Some(person),
                tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
            },
            None => None,
        };
        if core.person_answers
            && word.kind == tasks::MessageKind::Question
            && let Some(person) = person
        {
            tasks::Event::PersonQuestion { reply_to, project, task, person, word }
        } else {
            tasks::Event::Message { reply_to, project, task, word }
        }
    } else {
        event
    }
}

pub(super) fn result_words(result: tasks::TaskResult) -> Box<[u8]> {
    match result {
        tasks::TaskResult::Report { words }
        | tasks::TaskResult::Verdict { words, .. }
        | tasks::TaskResult::Change { words, .. } => words,
        tasks::TaskResult::Approved { words, .. } => match words {
            Some(words) => words,
            None => Box::new([]),
        },
        tasks::TaskResult::Failure { reason } => reason,
    }
}

pub(super) fn ending_words(ending: tasks::Ending) -> Box<[u8]> {
    match ending {
        tasks::Ending::Done(result) => result_words(result),
        tasks::Ending::Failed { reason } => reason,
        tasks::Ending::Cancelled { reason, result, .. } => match result {
            Some(result) => result_words(result),
            None => reason,
        },
    }
}

pub(super) fn result_notice(ending: tasks::Ending) -> (tasks::ResultKind, Box<[u8]>) {
    match ending {
        tasks::Ending::Done(result) => match result {
            tasks::TaskResult::Approved { state, words } => {
                let words_present = words.is_some();
                let words = match words {
                    Some(words) => words,
                    None => Box::new([]),
                };
                (tasks::ResultKind::Approved { state, words_present }, words)
            }
            tasks::TaskResult::Report { words } => (tasks::ResultKind::Report, words),
            tasks::TaskResult::Verdict { code, words, fields } => (tasks::ResultKind::Verdict { code, fields }, words),
            tasks::TaskResult::Change { connector, kind, resource, words } => {
                (tasks::ResultKind::Change { connector, kind, resource }, words)
            }
            tasks::TaskResult::Failure { reason } => (tasks::ResultKind::Failed, reason),
        },
        tasks::Ending::Failed { reason } => (tasks::ResultKind::Failed, reason),
        tasks::Ending::Cancelled { reason, .. } => (tasks::ResultKind::Cancelled, reason),
    }
}

fn follow(core: &mut Core, env: &Env<Limits>, event: tasks::Event, pending: &mut Queue<tasks::Request>) {
    let (event, procedure) = if let tasks::Event::Procedure { reply_to, task, step, read, decision } = event {
        let token = reply_to.into_token();
        (tasks::Event::Procedure { reply_to: ReplyTo::new(token), task, step, read, decision }, Some(token))
    } else {
        (event, None)
    };
    let mut child = Queue::uncapped();
    crate::routing::step::step_tasks(core, env, event, &mut child);
    for _ in 0..child.len() {
        let request = child.pop().expect("continuation output count");
        if let tasks::Request::ProcedureRefused { .. } = &request
            && let Some(token) = procedure
        {
            drop(core.land(token));
        }
        pending.push(request);
    }
}

fn tag_tasks(
    core: &mut Core,
    env: &Env<Limits>,
    token: Token,
    flight: Flight,
    outcome: Result<(), tasks::Problem>,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    if let Err(problem) = outcome {
        refused(core, env, token, flight, problem, pending, out);
        return;
    }
    match flight {
        Flight::PersonTask { request, route } => {
            let outcome = match route {
                PersonTaskRoute::Compound { project, operations, first_message } => {
                    people::Outcome::Compound { project, operations, first_message }
                }
                PersonTaskRoute::Take(task) => people::Outcome::PersonTaken { task },
                PersonTaskRoute::PoolSet { project, person } => people::Outcome::PoolSet { project, person },
                PersonTaskRoute::HandBack(task) => people::Outcome::PersonHandedBack { task },
                PersonTaskRoute::Answer(task) => people::Outcome::PersonAnswered { task },
                PersonTaskRoute::Stop(task) => people::Outcome::Stopped { task },
                PersonTaskRoute::Cancel(task) => people::Outcome::Cancelled { task },
                PersonTaskRoute::Release(task) => people::Outcome::Released { task },
                PersonTaskRoute::Amended { task, released } => people::Outcome::Amended { task, released },
                PersonTaskRoute::Lift(_) => return,
            };
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::Moving { request, task } => {
            append_people(core, env, people::Event::Decided { request, outcome: people::Outcome::Moved { task } }, out);
        }
        Flight::ProcedureProposal { owner, route } => {
            let event = crate::effects::procedure_proposed(core, out, owner, route, true);
            follow(core, env, event, pending);
        }
        Flight::Goal { request, route: GoalRoute::Accepting { proposer, proposal, by } } => {
            let token = core.fly_goal(request, GoalRoute::Deciding { proposer, proposal, by });
            follow(
                core,
                env,
                tasks::Event::DecidePersonProposal {
                    reply_to: ReplyTo::new(token),
                    proposer,
                    proposal,
                    by: tasks::Party::Person(by),
                    message: None,
                    decision: tasks::ProposalDecision::Accept,
                    revision: None,
                },
                pending,
            );
        }
        Flight::PersonProposal(PersonProposalRoute::Accepting { request, person, proposer, proposal, message }) => {
            let token = core.fly_person_proposal(PersonProposalRoute::Deciding {
                request,
                proposer,
                proposal,
                by: person,
                moved: None,
            });
            follow(
                core,
                env,
                tasks::Event::DecideProposal {
                    reply_to: ReplyTo::new(token),
                    proposer,
                    proposal,
                    message: Some(message),
                    by: tasks::Party::Person(person),
                    decision: tasks::ProposalDecision::Accept,
                },
                pending,
            );
        }
        Flight::Call { request, route, effect } => call_done(core, env, request, route, effect, pending, out),
        Flight::Goal { .. }
        | Flight::PersonProposal(_)
        | Flight::Made { .. }
        | Flight::PersonEscalation(_)
        | Flight::Internal(_)
        | Flight::Retired => {}
    }
}

fn call_done(
    core: &mut Core,
    env: &Env<Limits>,
    request: Token,
    route: RoutedCall,
    effect: Option<(u16, Token)>,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let (key, part) = match route {
        RoutedCall::Propose { key, proposal } => {
            if let Some((connector, owner)) = effect {
                out.push(Request::Ask {
                    connector,
                    ask: Ask::Effect(crate::connector::Ask::KeepProposal {
                        owner,
                        proposal,
                        task: key.task,
                        project: core.tasks.task(key.task).expect("proposing task exists").project,
                    }),
                });
            }
            let tasks::ProposalState::Pending { holder, .. } =
                core.tasks.proposal(key.task, proposal).expect("admitted proposal").state
            else {
                return;
            };
            (key, CallPart::Proposed { proposal, holder })
        }
        RoutedCall::Introduce(key) => (key, CallPart::Introduced),
        RoutedCall::Control { key, target, revised } => {
            let part = if revised {
                let revision = core.tasks.task(target).expect("controlled task exists").revisions.spec;
                CallPart::Revised { task: target, revision }
            } else {
                CallPart::Controlled
            };
            (key, part)
        }
        RoutedCall::Accepting { key, proposer, proposal, message } => {
            let token = core.fly_call(request, RoutedCall::Decide { key, proposal });
            follow(
                core,
                env,
                tasks::Event::DecideProposal {
                    reply_to: ReplyTo::new(token),
                    proposer,
                    proposal,
                    message: Some(message),
                    by: tasks::Party::Task(key.task),
                    decision: tasks::ProposalDecision::Accept,
                },
                pending,
            );
            return;
        }
        RoutedCall::Subscribe { key, subscription } => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::SubscriptionDone { request, key, subscription } });
            }
            return;
        }
        RoutedCall::Unsubscribe(key) => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::UnsubscriptionDone { request, key } });
            }
            return;
        }
        RoutedCall::Message(_)
        | RoutedCall::Decide { .. }
        | RoutedCall::Withdraw { .. }
        | RoutedCall::Escalation { .. } => return,
    };
    call_decision(core, out, ReplyTo::new(request), key, part);
}

fn refused(
    core: &mut Core,
    env: &Env<Limits>,
    token: Token,
    flight: Flight,
    problem: tasks::Problem,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    match flight {
        Flight::Goal { request, .. } | Flight::PersonTask { request, .. } | Flight::Made { request, .. } => {
            if match &core.settings_offering {
                Some((token, ..)) => *token == request,
                None => false,
            } {
                drop(core.settings_offering.take());
            }
            let outcome = person_problem_refusal(core, problem);
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::Moving { request, .. } => {
            let outcome = person_move_refusal(core, problem);
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::PersonProposal(route) => {
            let request = match route {
                PersonProposalRoute::Deciding { request, .. } | PersonProposalRoute::Accepting { request, .. } => {
                    request
                }
            };
            let outcome = person_move_refusal(core, problem);
            append_people(core, env, people::Event::Decided { request, outcome }, out);
        }
        Flight::ProcedureProposal { owner, route } => {
            let event = crate::effects::procedure_proposed(core, out, owner, route, false);
            follow(core, env, event, pending);
        }
        Flight::Call { request, route, effect } => {
            if let Some((connector, owner)) = effect {
                out.push(Request::Ask {
                    connector,
                    ask: Ask::Effect(crate::connector::Ask::Drop {
                        owner,
                        answer: authority::Answer::Refuse,
                        findings: Box::new([]),
                        why: crate::connector::EffectRefusal::Proposal,
                    }),
                });
            }
            let (key, part) = match route {
                RoutedCall::Message(key) | RoutedCall::Introduce(key) => (key, CallPart::MessageRefused(problem)),
                RoutedCall::Control { key, target, .. } => {
                    let part = if let tasks::Refusal::HoldOwner { hold } = problem.why {
                        let row = core.tasks.placed_hold(hold).expect("owner refusal names retained hold");
                        CallPart::HoldOwner {
                            task: target,
                            hold,
                            revision: row.revision,
                            owner: crate::roles::hold_owner(row.owner),
                        }
                    } else {
                        CallPart::ControlRefused(problem)
                    };
                    (key, part)
                }
                RoutedCall::Escalation { key, .. } => (key, CallPart::EscalationRefused(problem)),
                RoutedCall::Propose { key, .. }
                | RoutedCall::Decide { key, .. }
                | RoutedCall::Withdraw { key, .. }
                | RoutedCall::Accepting { key, .. } => (key, CallPart::ProposalRefused(problem)),
                RoutedCall::Subscribe { key, .. } | RoutedCall::Unsubscribe(key) => {
                    call_decision(core, out, ReplyTo::new(request), key, CallPart::SubscriptionRefused(problem));
                    for &connector in core.connectors.as_ref() {
                        out.push(Request::Ask { connector, ask: Ask::DropSubscription { request } });
                    }
                    return;
                }
            };
            call_decision(core, out, ReplyTo::new(request), key, part);
        }
        Flight::Internal(Internal::Repair) => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::RepairRefused { repair: problem.task } });
            }
        }
        Flight::Internal(Internal::Delegate | Internal::Acknowledge) => {
            for &connector in core.connectors.as_ref() {
                out.push(Request::Ask { connector, ask: Ask::DelegateRefused { task: problem.task } });
            }
        }
        Flight::Internal(Internal::Admission) => {
            out.push(Request::Now(Box::new(Now::RefusedPayload { request: token, problem })));
        }
        Flight::Retired | Flight::PersonEscalation(_) => {}
    }
}
