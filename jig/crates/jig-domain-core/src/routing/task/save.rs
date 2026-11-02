//! Task records, their projections, and the people entries they update
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::{
    Ask, Core, CoreRecord, Limits, Record, Request, Write,
    routing::{children::append_people, view::routes::touch_task},
};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Token};

pub(crate) fn save(
    core: &mut Core,
    env: &Env<Limits>,
    record: tasks::Stored,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    crate::routing::settings::dispatch::proposal_saved(core, &record, out);
    if let tasks::Stored::Activity(line) = &record {
        let _retained = core.remember_question(line);
    }
    if let tasks::Stored::Live(task) | tasks::Stored::Ended(task) = &record {
        core.prune_questions(task.number);
    }
    if let Some(archive) = crate::admission::proposals::decision_record(&record, env.wall) {
        // Accepted task batches already saved their member numbers
        // with Made; the history row must not replace that evidence.
        let by_task = match archive.proposer {
            people::Proposer::Task(_) => true,
            people::Proposer::Party(_) => false,
        };
        let made =
            archive.kind == tasks::ProposalKind::Batch && archive.choice == people::ProposalChoice::Accepted && by_task;
        if !made {
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::ProposalDecision(archive)))));
        }
    }
    let record = prepare_record(core, env, record, out);
    crate::reads::indexes::save(core, &record, env.wall, out);
    task_saved(core, env, &record, pending, out);
    touch_record(core, env, &record);
    let waiting = waiting(core, env, &record);
    history(core, env, &record, out);
    out.push(Request::Write(Write::Save(Record::Tasks(record))));
    if let Some((task, entries)) = waiting {
        append_people(core, env, people::Event::Waiting { task, entries }, out);
    }
}

fn prepare_record(
    core: &mut Core,
    env: &Env<Limits>,
    record: tasks::Stored,
    out: &mut Queue<Request>,
) -> tasks::Stored {
    match record {
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
                            forgettable: (task.flags.retention == tasks::Retention::ForgottenOnceRead).then_some(
                                people::Forgettable {
                                    label: task.label,
                                    listed: task.flags.listed,
                                    turns_read: core.people.conversation_seen(person, task.number).unwrap_or(0)
                                        >= task.last_reply,
                                    last_reply: task.flags.conversation.then_some(task.last_reply),
                                },
                            ),
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
                                ask: Ask::Effect(crate::effects::connector::Ask::DropProposal {
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
    }
}

fn task_saved(
    core: &mut Core,
    env: &Env<Limits>,
    record: &tasks::Stored,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    if let tasks::Stored::Live(task) | tasks::Stored::Ended(task) = record {
        if let Some(to) = core.conversation_requester(task)
            && let Some(from) = core.people.chat_owner(task.number)
            && from != to
        {
            append_people(
                core,
                env,
                people::Event::MoveChat { task: task.number, project: task.project, from, to },
                out,
            );
        }
        let seen = match core.conversation_requester(task) {
            Some(party) => core.people.conversation_seen(party, task.number),
            None => None,
        };
        if let Some(party) = core.conversation_requester(task) {
            let live = match record {
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
                out,
            );
        }
        if let tasks::Stored::Ended(_) = &record {
            core.cache_ended_chat(&env.limits, task, seen);
        }
        match task.phase {
            tasks::Phase::Ended(_) => {
                drop(core.job_workspaces.remove(&task.number));
                drop(core.workspace_credentials.remove(&task.number));
            }
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {}
        }
        clear_stale_preparation(core, env, task, pending, out);
        crate::effects::projections::save(core, &env.limits, task, out);
        touch_task(core, &env.limits, task);
    }
}

fn touch_record(core: &mut Core, env: &Env<Limits>, record: &tasks::Stored) {
    match record {
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
}

fn waiting(core: &Core, env: &Env<Limits>, record: &tasks::Stored) -> Option<(u64, Box<[people::Entry]>)> {
    match record {
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
    }
}

fn history(core: &mut Core, env: &Env<Limits>, record: &tasks::Stored, out: &mut Queue<Request>) {
    match record {
        tasks::Stored::Milestone(row) => crate::effects::projections::history(
            core,
            &env.limits,
            row.task,
            crate::ProjectionMilestone {
                identity: crate::MilestoneId::Lifecycle { task: row.task, position: row.position },
                phase: Some(row.phase.clone()),
                change: None,
                words: Box::new([]),
            },
            out,
        ),
        tasks::Stored::History(row) => crate::effects::projections::history(
            core,
            &env.limits,
            row.task,
            crate::ProjectionMilestone {
                identity: crate::MilestoneId::Revision { task: row.task, revision: row.revision },
                phase: None,
                change: Some(row.change),
                words: row.reason.clone(),
            },
            out,
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
}

fn clear_stale_preparation(
    core: &mut Core,
    env: &Env<Limits>,
    task: &tasks::TaskRecord,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    let stale = match core.contexts.get(&task.number) {
        Some(context) => {
            task.phase != tasks::Phase::Active(tasks::Active::Preparing)
                || core.tasks.run_message(task.number) != Some(context.last_message)
        }
        None => false,
    };
    if stale {
        crate::routing::run::results::clear(core, task.number);
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
}
