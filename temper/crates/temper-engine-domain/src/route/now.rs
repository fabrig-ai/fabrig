//! The core's immediate outputs: payload reads, preparation and the door
//! (jig's `domain/root.md`, 4; `testing/jig-test-domain/src/routes.rs`).
use super::{
    CallAnswer, Decision, Delivery, Domain, Env, HostRequest, Limits, List, PreparedWorkspace, Read, Request, Work,
    emit, escalation, forge_route, proposals, relay_call,
};
use crate::Range;
use crate::assemble::{brief_outputs, finish_brief_plan, take_payload};
use crate::boundary::{EndedRead, Payload, WorkspacePreparation};
use alloc::boxed::Box;
use jig_domain_brief as brief;
use jig_domain_core as core;
use jig_domain_link::Grant;
use jig_domain_tasks as tasks;
use skein_lib::Queue;

#[expect(clippy::too_many_lines, reason = "one exhaustive dispatcher routes the core's immediate vocabulary")]
pub(crate) fn now(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, now: core::Now) {
    match now {
        core::Now::Link(value) => link_now(domain, value),
        core::Now::RelayBusy { channel, run, attempt, call } => {
            let delivery = super::host_route::busy(channel, run, attempt, call);
            assert!(super::now(domain, crate::boundary::delivery_output(delivery)), "busy answer door reserved");
        }
        core::Now::CallBusy { to } => crate::stopping::call(domain, to),
        core::Now::Call { to, run, attempt, call } => {
            if domain.settling.active() {
                crate::stopping::call(domain, to);
            } else {
                super::host_route::decode(domain, to, run.raw(), attempt.raw(), call);
            }
        }
        core::Now::DropCall { call } => {
            domain.hub_link.forget_call(&call.name);
            let request = Request::Worker(crate::WorkerRequest::Host(Box::new(HostRequest::Dropped { call })));
            let _sent = super::now(domain, request);
        }
        core::Now::Undelivered { message, .. } => drop(take_payload(domain, message.words)),
        core::Now::SettledCallRefused { to, name, .. } => {
            super::host_route::refused(domain, &env.limits, decision, to, name);
        }
        core::Now::SignInRefused { to } => {
            let _sent = super::now(domain, Request::Party(crate::PartyRequest::SignInRefused { to }));
        }
        core::Now::WatchRefused { watcher, why } => {
            let _sent = super::now(domain, Request::Party(crate::PartyRequest::WatchRefused { watcher, why }));
        }
        core::Now::RunText { watcher, turn, from, to, text, cut, call, writing } => {
            let request =
                Request::Party(crate::PartyRequest::RunText { watcher, turn, from, to, text, cut, call, writing });
            let _sent = super::now(domain, request);
        }
        core::Now::Activate { context } => {
            let ready = domain.ready() && crate::domain::within_budget(domain, &env.limits);
            domain.work.push(Work::Activate { context, ready });
        }
        core::Now::StartPreparation { task, from } => {
            start_preparation(domain, env, decision, task, from);
        }
        core::Now::EndedTask { request, task } => {
            ended(domain, env, decision, EndedRead::EndedTask { request, task });
        }
        core::Now::SeenConversation { request, task } => {
            ended(domain, env, decision, EndedRead::SeenConversation { request, task });
        }
        core::Now::LoadEndedRead { request, task, party } => {
            ended(domain, env, decision, EndedRead::Position { request, task, party });
        }
        core::Now::PartyInputs { request, tasks, .. } => ended(
            domain,
            env,
            decision,
            EndedRead::PartyInput { request, task: *tasks.first().expect("historical input") },
        ),
        core::Now::EndedPersonTask { request, person, project, task } => {
            ended(domain, env, decision, EndedRead::PersonTask { request, person, project, task });
        }
        core::Now::DecidedProposal { to, key, proposal } => {
            proposals::task_begin(domain, env, decision, to, key, proposal);
        }
        core::Now::EndedProposal { request, person, project, proposer, proposal } => {
            proposals::historical_begin(domain, env, decision, request, person, project, proposer, proposal);
        }
        core::Now::EndedEscalation { request, person, project, task, revision } => {
            escalation::historical_begin(domain, env, decision, request, person, project, task, revision);
        }
        core::Now::BriefCorePlanned { task, parent, sections, documents } => {
            if waiting_for_design(domain, task) {
                domain
                    .run_workspaces
                    .insert(task, WorkspacePreparation::Brief { parent, sections, documents })
                    .expect("one brief plan per live task");
            } else {
                finish_brief_plan(domain, env, task, parent, sections, documents);
            }
        }
        core::Now::CompleteBrief { brief, order } => {
            let mut child = Queue::with_capacity(1);
            child.push(brief::GatherRequest::Complete { brief, order });
            brief_outputs(domain, env, decision, &mut child);
        }
        core::Now::WorkspaceRequest { task, attempt, context } => workspace(domain, env, task, attempt, context),
        core::Now::BriefFailed { task, failure } => {
            let counts = match failure.fate {
                brief::GatherFate::Capacity => "capacity",
                brief::GatherFate::Transient => "transient",
                brief::GatherFate::Infrastructure => "infrastructure",
                brief::GatherFate::Permanent => "permanent",
            };
            super::diagnostics::emit(domain, crate::Fact::BriefFailed { task, counts, reason: failure.reason() });
        }
        core::Now::RunPreparationFailed { task } => {
            drop(domain.brief_document_titles.remove(&task));
            if let Some(sections) = crate::link::take_sections(domain, task) {
                domain.release_bytes(
                    crate::budget::section_bytes(&sections),
                    "failed preparation's completed sections were held",
                );
            }
            drop(domain.run_workspaces.remove(&task));
            drop(domain.job_workspaces.remove(&task));
        }
        core::Now::RunPrepared {
            task,
            attempt,
            conversation,
            live,
            charter,
            run,
            messages,
            saved,
            transcript,
            answered,
            grants,
        } => {
            let prepared = Prepared {
                task,
                attempt,
                conversation,
                live,
                charter,
                run,
                messages,
                saved,
                transcript,
                answered,
                grants,
            };
            run_prepared(domain, env, prepared);
        }
        core::Now::DropAssignment { task } => {
            drop(domain.run_workspaces.remove(&task));
            drop(domain.job_workspaces.remove(&task));
            if let Some(assignment) = crate::link::take_assignment(domain, task) {
                domain.release_bytes(
                    crate::budget::assignment_bytes(&assignment),
                    "dropped assignment turns and sections were held",
                );
            }
        }
        core::Now::TaskRead { to, key: _, answer } => super::host_route::read(domain, to, CallAnswer::TaskRead(answer)),
        core::Now::ProposalRead { to, key: _, answer } => super::see::proposal::answered(domain, to, answer),
        core::Now::SettingsAnswer { to, key: _, answer } => super::host_route::settings(domain, to, answer),
        core::Now::PersonAnswer { to, key: _, answer } => super::see::person::answered(domain, to, *answer),
        core::Now::EffectAnswer { to, key: _, part } => {
            let answer = CallAnswer::from_core(&part).expect("effect wait answer");
            relay_call(domain, &env.limits, decision, to, answer);
        }
        core::Now::NoteBusy { to, key: _ } => {
            relay_call(domain, &env.limits, decision, to, CallAnswer::NoteRefused(jig_domain_notes::Refusal::Busy));
        }
        core::Now::RestoreRefused => {
            domain.stop_core();
        }
        core::Now::Account(request) => {
            assert!(super::now(domain, Request::Account(request)), "admitted account output fits the door");
        }
        core::Now::NotesIndexed { owner, lines, more } => {
            forge_route::status_route::notes_indexed(domain, env, owner, lines, more);
        }
        core::Now::NotesRecalled { owner, entries, more: _ } => {
            forge_route::status_route::notes_recalled(domain, env, owner, entries);
        }
        core::Now::NotesRefused { owner, why: _ } => {
            forge_route::status_route::notes_refused(domain, env, owner);
        }
    }
}

fn link_now(domain: &mut Domain, value: core::LinkNow) {
    match value {
        core::LinkNow::Relayed { channel, run, attempt, call, answer } => {
            let delivery = super::host_route::relayed(domain, channel, run, attempt, call, answer);
            assert!(super::now(domain, crate::boundary::delivery_output(delivery)), "read answer door reserved");
        }
        core::LinkNow::DropPayload { payload } => {
            if let Some(payload) = take_payload(domain, payload) {
                domain.release_bytes(crate::budget::payload_bytes(&payload), "dropped turn payload was held");
            }
        }
        core::LinkNow::TurnPayload { run, attempt, turn, body } => {
            crate::link::route(domain, jig_domain_hub_link::Event::TurnPayload { run, attempt, turn, body });
        }
        core::LinkNow::AnswerPayload { run, attempt, payload } => crate::link::answer(domain, run, attempt, payload),
        core::LinkNow::AcceptedTurn { payload, task, attempt, turn, accepted } => {
            crate::link::route(
                domain,
                jig_domain_hub_link::Event::AcceptedTurn { payload, task, attempt, turn, accepted },
            );
        }
        core::LinkNow::RefusedPayload { request, problem } => {
            if let Some(value) = domain.hub_link.payload(request) {
                let bytes = crate::budget::payload_bytes(value);
                let answer =
                    if let Payload::Answer { task, attempt, .. } = value { Some((*task, *attempt)) } else { None };
                domain.release_bytes(bytes, "refused turn payload was held");
                if let Some(key) = answer {
                    domain.forge_left.remove(&key);
                }
            }
            domain.answer_parts.remove(&request);
            crate::link::route(domain, jig_domain_hub_link::Event::RefusedPayload { request, problem });
        }
    }
}

/// Load the current transcript window before the brief.
fn start_preparation(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, task: u64, from: (u64, u32)) {
    let waiter = domain
        .archive_reads
        .insert(Some(Read::Transcript { task }))
        .expect("the activation's step found read room")
        .token();
    let range = Range::Core(crate::CoreRange::TaskTranscript { task, from });
    emit(decision, &env.limits, Delivery::Load { waiter, range, after: None });
}

/// Load one ended owner row the core asked for; a full read table answers busy.
fn ended(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, read: EndedRead) {
    let range = match &read {
        EndedRead::Position { task, party, .. } => crate::CoreRange::EndedRead { task: *task, party: *party },
        EndedRead::EndedTask { task, .. }
        | EndedRead::SeenConversation { task, .. }
        | EndedRead::PartyInput { task, .. }
        | EndedRead::PersonTask { task, .. } => crate::CoreRange::TaskResult { task: *task },
    };
    match domain.archive_reads.insert(Some(Read::Ended(read))) {
        Ok(waiter) => emit(
            decision,
            &env.limits,
            Delivery::Load { waiter: waiter.token(), range: Range::Core(range), after: None },
        ),
        Err(Some(Read::Ended(read))) => crate::archive::ended_answer(domain, read, None, true),
        Err(
            Some(
                Read::Escalation(_)
                | Read::Proposal(_)
                | Read::TaskProposal { .. }
                | Read::Transcript { .. }
                | Read::GoalWords(_)
                | Read::InputCheck(_)
                | Read::Document(_)
                | Read::Remember(_)
                | Read::SettingsNote { .. }
                | Read::Notes { .. },
            )
            | None,
        ) => unreachable!("the refused read returns unchanged"),
    }
}

fn workspace(domain: &mut Domain, env: &Env<Limits>, task: u64, attempt: u64, context: Box<tasks::RunContext>) {
    if waiting_for_design(domain, task) {
        domain
            .run_workspaces
            .insert(task, WorkspacePreparation::Workspace { attempt, context })
            .expect("one workspace request per live task");
        return;
    }
    let mut credentials = List::with_capacity(env.limits.forge.resources_per_task);
    let workspace = forge_route::run_workspace(domain, env, &context, attempt);
    let writes = match workspace {
        Some(workspace) => {
            match forge_route::claim_names(domain, &env.limits, task, &workspace.names, &workspace.designs) {
                Some(claim_names) => {
                    // Each repository's git identity is an application credential
                    // the run is granted (domain/root.md, 6.1; jig's domain/engine/README.md, 6.1).
                    for repository in &workspace.workspace.repositories {
                        if !credentials.as_slice().contains(&repository.identity) {
                            credentials.push(repository.identity).expect("one identity per repository");
                        }
                    }
                    let writes = workspace.writes.clone();
                    let inserted = match domain
                        .run_workspaces
                        .insert(task, WorkspacePreparation::Ready(PreparedWorkspace { workspace, claim_names }))
                    {
                        Ok(None) => true,
                        Ok(Some(_)) | Err(_) => false,
                    };
                    assert!(inserted, "one workspace flight per task");
                    Some(writes)
                }
                None => None,
            }
        }
        None => None,
    };
    let credentials = match writes {
        Some(_) => credentials.into_boxed(),
        None => Box::new([]),
    };
    let job = match context.executor {
        tasks::Executor::Job { .. } => true,
        tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => false,
    };
    let job_workspace = if job
        && match domain.run_workspaces.get(&task) {
            Some(prepared) => !prepared.prepared().workspace.workspace.repositories.is_empty(),
            None => false,
        } {
        Some(Box::from(task.to_le_bytes().as_slice()))
    } else {
        None
    };
    let holds = if job {
        match domain.run_workspaces.get(&task) {
            Some(prepared) => workspace_holds(domain, env, &prepared.prepared().workspace),
            None => Box::new([]),
        }
    } else {
        Box::new([])
    };
    domain.work.push(Work::Core(core::Event::WorkspacePrepared {
        task,
        attempt,
        holds,
        writes,
        credentials,
        workspace: job_workspace,
    }));
}

fn waiting_for_design(domain: &Domain, task: u64) -> bool {
    let Some(context) = domain.core.tasks.run_context(task) else { return false };
    let charter = match context.executor {
        tasks::Executor::Agent { charter } => charter,
        tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => return false,
    };
    let Some(charter) = domain.core.charter(charter) else { return false };
    charter.policy.modify && !crate::no_workspace(&charter.policy) && domain.forge.design_workspace_waiting(task)
}

/// Retry only after the forge changes: a taking, its confirmation or a drift release.
pub(super) fn resume_workspaces(domain: &mut Domain, env: &Env<Limits>) {
    let mut ready = List::with_capacity(env.limits.tasks.tasks);
    for (task, preparation) in &domain.run_workspaces {
        let waiting = match preparation {
            WorkspacePreparation::Ready(_) => false,
            WorkspacePreparation::Brief { .. } | WorkspacePreparation::Workspace { .. } => true,
        };
        if waiting && !waiting_for_design(domain, *task) {
            ready.push(*task).expect("waiting workspaces bounded by live tasks");
        }
    }
    for task in &ready {
        let pending = domain.run_workspaces.remove(task).expect("collected preparation wait");
        if domain.core.tasks.run_context(*task).is_none() {
            continue;
        }
        match pending {
            WorkspacePreparation::Ready(_) => unreachable!("only waiting preparations were collected"),
            WorkspacePreparation::Brief { parent, sections, documents } => {
                finish_brief_plan(domain, env, *task, parent, sections, documents);
            }
            WorkspacePreparation::Workspace { attempt, context } => workspace(domain, env, *task, attempt, context),
        }
    }
}

struct Prepared {
    task: u64,
    attempt: u64,
    conversation: u64,
    live: core::LiveAsk,
    charter: u32,
    run: Box<core::RunCharter>,
    messages: Box<[tasks::Word]>,
    saved: Box<[tasks::SavedResource]>,
    transcript: Box<[Box<[u8]>]>,
    answered: Box<[core::CallKey]>,
    grants: Box<[Grant]>,
}

fn run_prepared(domain: &mut Domain, env: &Env<Limits>, prepared: Prepared) {
    let Prepared { task, attempt, conversation, live, charter, run, messages, saved, transcript, answered, grants } =
        prepared;
    for body in &transcript {
        domain.hold_bytes(skein_lib::len_u64(body.len()));
    }
    let brief = domain.hub_link.take_brief(task).expect("prepared brief sections");
    let PreparedWorkspace { workspace, claim_names } =
        domain.run_workspaces.remove(&task).expect("prepared connector workspace").into_prepared();
    let mut records = List::with_capacity(domain.limits.call_records);
    for key in answered {
        let answer = super::call_answer(domain, key).expect("live call part has its connector answer");
        records
            .push(crate::CallRecord { key, answer, settled: domain.core.call_settled.get(&key).cloned() })
            .expect("retained call bound");
    }
    let parts = crate::boundary::AssignmentParts {
        saved: forge_route::saved_tags(&saved, domain.config.forge_connector)
            .expect("task saved names were admitted by the root"),
        workspace: workspace.workspace.clone(),
        workstream: workspace.workstream,
        answered: records.into_boxed(),
    };
    domain.assignment_parts.insert(task, parts).expect("one temper assignment per task");
    let assignment = jig_domain_hub_link::Assignment {
        task,
        attempt,
        conversation,
        live,
        charter,
        run,
        brief,
        messages,
        transcript,
        answered: domain.core.settled_calls(task, attempt),
        grants,
    };
    let budget = assignment.run.budget;
    domain.hub_link.keep_assignment(assignment);
    let writes = claim_workspace(domain, env, task, &workspace, claim_names);
    domain.work.push(Work::Core(core::Event::ClaimPrepared { task, attempt, budget, writes }));
}

pub(super) fn claim_workspace(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    workspace: &forge_route::RunWorkspace,
    claim_names: Box<[super::forge::Name]>,
) -> Box<[tasks::Name]> {
    if workspace.names.is_empty() && workspace.designs.is_empty() {
        return Box::new([]);
    }
    let hub_writes = workspace_holds(domain, env, workspace);
    domain.work.push(Work::Forge(super::forge::Event::Names { task, resources: claim_names }));
    for name in &workspace.own_holds {
        domain.work.push(Work::Forge(super::forge::Event::Hold { entry: 0, task, resource: name.clone(), from: None }));
    }
    hub_writes
}

fn workspace_holds(domain: &Domain, env: &Env<Limits>, workspace: &forge_route::RunWorkspace) -> Box<[tasks::Name]> {
    let mut hub_writes = List::with_capacity(env.limits.tasks.holdings);
    for name in &workspace.names {
        let resource = forge_route::hub_name(domain.config.forge_connector, name, &env.limits.tasks)
            .expect("admitted forge branch name fits the hub's bound");
        hub_writes.push(resource).expect("workspace write bound fits hub");
    }
    hub_writes.into_boxed()
}
