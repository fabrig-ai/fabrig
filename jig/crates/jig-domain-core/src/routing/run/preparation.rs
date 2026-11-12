//! Run admission, briefs and workspace preparation across the task, brief,
//! notes and account children (`domain/engine/README.md`, section 4.4, and `domain/engine/executors.md`, section 2).

use crate::{
    Ask, Core, Event, Limits, Now, Request, RunCharter, RunProof,
    routing::{NoteRoute, Work, children::remember_due},
};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_brief as brief;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, ReplyTo, Token};

pub(crate) fn brief_assembled(
    core: &mut Core,
    work: &mut Queue<Work>,
    task: u64,
    failure: Option<brief::GatherFailure>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if let Some(failure) = failure {
        return route_preparation_failed(core, work, task, failure);
    }
    let Some(context) = core.contexts.get(&task).cloned() else {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        return out;
    };
    let Some(attempt) = crate::fresh(&mut core.counters, crate::Family::Run) else {
        return route_preparation_failed(
            core,
            work,
            task,
            brief::GatherFailure {
                fate: brief::GatherFate::Permanent,
                section: b"run",
                words: Box::from(b"no attempt number is left".as_slice()),
            },
        );
    };
    assert!(core.workspace_flight(task).is_none(), "one workspace flight per task");
    core.fly(crate::routing::flight::Flight::Workspace { task, attempt });
    out.push(Request::Now(Box::new(Now::WorkspaceRequest { task, attempt, context })));
    out
}

pub(crate) struct Prepared {
    pub writes: Option<Box<[authority::Write]>>,
    pub holds: Box<[tasks::Name]>,
    pub credentials: Box<[u32]>,
    pub workspace: Option<Box<[u8]>>,
}

#[expect(
    clippy::too_many_lines,
    reason = "one fenced preparation builds the exact allowance, grants and claim together"
)]
pub(crate) fn workspace_prepared(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    attempt: u64,
    prepared: Prepared,
) -> Queue<Request> {
    let Prepared { writes, holds, credentials, workspace } = prepared;
    let mut out = Queue::uncapped();
    let Some((token, issued)) = core.workspace_flight(task) else { return out };
    if issued != attempt {
        return out;
    }
    drop(core.land_kind(token, crate::routing::flight::Kind::Workspace));
    let Some(context) = core.contexts.get(&task).cloned() else {
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        return out;
    };
    drop(core.dependency_results.remove(&task));
    let Some(writes) = writes else {
        work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects })));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        return out;
    };
    if credentials.len() > skein_lib::length::usize_of(env.limits.accounts.credentials) {
        work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects })));
        out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
        return out;
    }
    drop(core.workspace_credentials.insert(task, credentials).expect("live task credential room"));
    let Some(current) = super::parts::preparation_facts(core, &context) else {
        drop(core.workspace_credentials.remove(&task));
        return wait_for_account(core, work, context);
    };
    match context.executor {
        tasks::Executor::Job { .. } => {
            if let Some(items) = &workspace
                && items.len() > skein_lib::length::usize_of(env.limits.run_bytes)
            {
                work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task, why: tasks::Hold::Effects })));
                out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
                return out;
            }
            return super::jobs::prepared(
                core,
                env,
                work,
                context,
                attempt,
                writes,
                super::jobs::JobItems { holds, workspace },
            );
        }
        tasks::Executor::Agent { .. } => {}
        tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) => unreachable!("workspace executor"),
    }
    let transcript = core.transcripts.remove(&task).expect("rendered task owns loaded transcript");
    let allowance = match core.run_admission(&current, env.wall, writes) {
        crate::RunAdmission::Allow(allowance) => allowance,
        crate::RunAdmission::Wait(_) => return wait_for_account(core, work, current),
        crate::RunAdmission::Hold(why) => {
            work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task, why })));
            out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
            return out;
        }
    };
    let mut grants = List::with_capacity(env.limits.fleet.grants);
    for binding in &allowance.choice.accounts {
        let fact = core.accounts.source(binding.source).expect("selected source");
        if fact.kind != jig_domain_accounts::Kind::None {
            let Some(grant) = core.accounts.source_grant(binding.source, env.now) else {
                return wait_for_account(core, work, current);
            };
            grants.push(grant).expect("provider grant room");
        }
    }
    for number in core.run_credentials(task) {
        let Some(grant) = core.accounts.credential_grant(*number, env.now) else {
            return wait_for_account(core, work, context);
        };
        let mut present = false;
        for kept in &grants {
            if kept.form.table() == grant.form.table() && kept.account == grant.account {
                present = true;
            }
        }
        if !present {
            grants.push(grant).expect("credential grant room");
        }
    }
    if !core.proofs.contains_key(&task) && core.proofs.len() == core.proofs.capacity() {
        return route_preparation_failed(
            core,
            work,
            task,
            brief::GatherFailure {
                fate: brief::GatherFate::Capacity,
                section: b"run",
                words: Box::from(b"no room for its proof".as_slice()),
            },
        );
    }
    // A task's earlier tries do not close this window (jig/docs/design/domain/engine/executors.md, 2).
    let mut resume = !transcript.closed
        && !transcript.cut
        && core.charter_of(task).policy.resume
        && transcript.bytes <= u64::from(core.settings.resume_bytes);
    let charter = core.charter_of(task);
    let row = core.tasks.task(task).expect("live task");
    let standing = row.flags.ended_by == tasks::EndedBy::Standing;
    let own = row.instructions.as_deref();
    let mut session_end = None;
    let mut reasons = context.reasons;
    if standing {
        if let Some(session) = row.session {
            if context.session_wake {
                let model = &allowance.choice.policy.model;
                session_end = tasks::session_end(
                    session,
                    tasks::SessionWake {
                        at: env.wall,
                        span: tasks::quiet_span(model.cache_life_ms),
                        bytes: transcript.bytes,
                        bound: tasks::context_bound(model.window),
                        kinds: context.session_kinds,
                        offered: super::parts::offered_kinds(charter, own.is_some(), session.reasons, true),
                        resumable: resume && transcript.last.is_some(),
                    },
                );
            }
            if session_end.is_none() {
                reasons = session.reasons;
            }
        }
        if row.session.is_none() || session_end.is_some() {
            resume = false;
        }
    }
    let conversation = if resume && transcript.last.is_some() { transcript.from } else { attempt };
    let instructions = super::parts::run_instructions(charter, own, reasons, standing);
    let offered = super::parts::offered_kinds(charter, own.is_some(), reasons, standing);
    let messages = start_messages(context.inbox, env.limits.tasks.run_inbox, offered);
    let mut proof = start_proof(core, task, attempt, conversation, &messages);
    proof.host = allowance.choice.placement.kind;
    proof.placement = Some(allowance.choice.placement.channel);
    proof.accounts.clone_from(&allowance.choice.accounts);
    proof.window = if resume {
        match transcript.window {
            Some((attempt, turn)) => Some(crate::CurrentWindow { attempt, turn, closed: false }),
            None => None,
        }
    } else {
        None
    };
    proof.credentials = core.workspace_credentials.remove(&task).expect("prepared credentials");
    let previous = core.proofs.insert(task, proof).expect("claim proof reserved before child mutation");
    if let Some(previous) = previous {
        assert!(core.replaced.insert(task, previous) == Ok(None), "one pending replacement per task");
    }
    let turns = if resume { core.resumed_turns(transcript) } else { Box::new([]) };
    let answered = answered_calls(core, task, attempt, env.limits.call_records);
    let mut policy = allowance.choice.policy;
    let mut sources = List::with_capacity(tasks::MODELS.checked_add(1).expect("main source"));
    let mut models = List::with_capacity(tasks::MODELS.checked_add(1).expect("main model"));
    models.push(&mut policy.model).expect("main model");
    for model in &mut policy.alternatives {
        models.push(model).expect("retained alternatives");
    }
    for model in models.into_boxed() {
        let provider = core.provider(model).expect("selected model provider");
        let mut grant = None;
        for binding in &allowance.choice.accounts {
            if binding.provider == provider {
                let kind = core.accounts.source(binding.source).expect("selected source").kind;
                model.endpoint = core.settings.model_endpoint(model, kind).expect("checked model way");
                if kind != jig_domain_accounts::Kind::Program {
                    model.program = None;
                }
                if kind != jig_domain_accounts::Kind::None {
                    grant = Some(binding.source);
                }
            }
        }
        sources.push(grant).expect("one grant per retained model");
    }
    policy.instructions = instructions;
    policy.resume = resume;
    let run = RunCharter {
        revision: core.tasks.task(task).expect("prepared task").settings.run.expect("prepared revision"),
        ended_by: context.ended_by,
        sources: sources.into_boxed(),
        accounts: allowance.choice.accounts,
        placement: allowance.choice.placement,
        policy,
        contract: context.contract,
        authority: context.authority,
        budget: allowance.spend,
    };
    assert!(core.claiming.insert(task, (attempt, offered, session_end)) == Ok(None), "one pending claim per task");
    out.push(Request::Now(Box::new(Now::RunPrepared {
        live: crate::LiveAsk {
            text: core.tasks.task(task).expect("live task").flags.conversation,
            reasoning: false,
            input: core.tasks.task(task).expect("live task").flags.conversation,
        },
        task,
        attempt,
        conversation,
        charter: core.charter_of(task).number,
        run: Box::new(run),
        messages,
        saved: context.saved,
        transcript: turns,
        answered,
        grants: grants.into_boxed(),
    })));
    out
}

fn answered_calls(core: &Core, task: u64, attempt: u64, limit: u32) -> Box<[crate::CallKey]> {
    let mut answered = List::with_capacity(limit);
    let mut after = None;
    for _ in 0..core.call_parts.len() {
        let mut next = None;
        for (&key, _) in &core.call_parts {
            if key.task == task
                && key.attempt < attempt
                && match after {
                    Some(previous) => key > previous,
                    None => true,
                }
                && match next {
                    Some(previous) => key < previous,
                    None => true,
                }
            {
                next = Some(key);
            }
        }
        let Some(key) = next else { break };
        answered.push(key).expect("retained call bound");
        after = Some(key);
    }
    answered.into_boxed()
}

fn start_messages(inbox: Box<[tasks::Word]>, most: u32, offered: tasks::Reasons) -> Box<[tasks::Word]> {
    let mut messages = List::with_capacity(most);
    for word in inbox {
        if !tasks::offered(&word, offered) {
            continue;
        }
        if messages.len() == most {
            break;
        }
        messages.push(word).expect("start inbox bound");
    }
    messages.into_boxed()
}

pub(crate) fn start_proof(
    _core: &Core,
    task: u64,
    attempt: u64,
    conversation: u64,
    messages: &[tasks::Word],
) -> RunProof {
    let mut offered = None;
    let mut outstanding = List::with_capacity(u32::try_from(messages.len()).expect("bounded start inbox"));
    for word in messages {
        offered = Some(word.number);
        outstanding.push(word.number).expect("one id per assigned word");
    }
    let mut proof = RunProof::claimed(task, attempt, conversation, offered, jig_domain_fleet::HostKind::Worker);
    proof.outstanding = outstanding.into_boxed();
    proof
}

pub(crate) fn wait_for_account(
    core: &mut Core,
    work: &mut Queue<Work>,
    context: Box<tasks::RunContext>,
) -> Queue<Request> {
    let task = context.task;
    drop(core.job_workspaces.remove(&task));
    super::results::clear(core, task);
    drop(core.contexts.remove(&task));
    work.push(Work::Event(Event::Tasks(tasks::Event::PreparationWaiting { task })));
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
    out
}

pub(crate) fn start_brief(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Work>, task: u64) -> Queue<Request> {
    if let Some(transcript) = core.transcripts.get_mut(&task) {
        transcript.loaded = true;
    }
    let out = Queue::uncapped();
    let Some(context) = core.contexts.get(&task) else {
        return out;
    };
    if core.tasks.task(task).is_none() {
        return out;
    }
    if !super::parts::preparation_current(core, context) {
        return out;
    }
    if !core.dependency_results.contains_key(&task)
        && let Some(ids) = core.preparation_ids(task)
        && !ids.is_empty()
    {
        return super::results::begin(core, work, task, ids);
    }
    if core.notes.busy() {
        crate::routing::effect::note::wait_for_notes(core, work, task, crate::routing::NoteBrief::Operation);
        return out;
    }
    if super::preferences::gather(core, env, work, task) || super::now::gather(core, env, work, task) {
        return out;
    }
    let context = core.contexts.get(&task).expect("current preparation context");
    let current = core.tasks.task(task).expect("current preparation task");
    let mut scopes = List::with_capacity(env.limits.notes.scopes);
    if let Some(goal) = core.note_goal(current) {
        scopes.push(notes::Scope::Goal { project: context.project, goal }).expect("goal scope room");
    }
    scopes.push(notes::Scope::Project { project: context.project }).expect("project scope room");
    scopes.push(notes::Scope::Deployment).expect("deployment scope room");
    for resource in &context.authority.note_resources {
        let last = match &resource.pattern.last {
            tasks::Last::Exact(value) => notes::Last::Exact(value.clone()),
            tasks::Last::Open(value) => notes::Last::Open(value.clone()),
        };
        scopes
            .push(notes::Scope::Resources {
                project: context.project,
                connector: resource.connector,
                pattern: notes::Pattern { segments: resource.pattern.segments.clone(), last },
            })
            .expect("admitted note resource scope room");
    }
    let owner = core.begin_note_route(NoteRoute::Brief { task });
    work.push(Work::Event(Event::Notes(notes::Event::Index { owner, scopes, most: env.limits.notes.lines })));
    out
}

#[expect(clippy::too_many_lines, reason = "one ordered plan places charter documents after the core's sections")]
pub(crate) fn plan_brief(
    core: &Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    lines: List<notes::Line>,
    more: u32,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let Some(context) = core.contexts.get(&task) else {
        return out;
    };
    if !super::parts::preparation_current(core, context) {
        return out;
    }
    let text_limits = crate::BriefTextLimits {
        parts: env.limits.brief_parts,
        read_bytes: env.limits.brief.read_bytes,
        brief_bytes: env.limits.brief.brief_bytes,
    };
    let mut wanted = List::with_capacity(env.limits.brief.sections);
    let Some(task_text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Spec) else {
        work.push(Work::PreparationFailed {
            task,
            failure: brief::GatherFailure {
                fate: brief::GatherFate::Permanent,
                section: b"task",
                words: Box::from(b"could not be read within its bounds".as_slice()),
            },
        });
        return out;
    };
    wanted
        .push(brief::Planned::Core {
            kind: brief::Core::Task,
            text: task_text,
            limit: env.limits.brief_core_budgets.task,
            priority: 0,
            required: true,
        })
        .expect("task brief room");
    if !super::now::plan(core, &env.limits, work, task, &mut wanted) {
        return out;
    }
    if !context.dependencies.is_empty() || !context.spec.inputs.is_empty() {
        let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Dependencies) else {
            work.push(Work::PreparationFailed {
                task,
                failure: brief::GatherFailure {
                    fate: brief::GatherFate::Permanent,
                    section: b"results",
                    words: Box::from(b"could not be read within its bounds".as_slice()),
                },
            });
            return out;
        };
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Results,
                text,
                limit: env.limits.brief_core_budgets.dependencies,
                priority: 1,
                required: true,
            })
            .expect("dependency result section room");
    }
    if tail_wanted(core, task, context.tries.transient != 0) {
        let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::TranscriptTail) else {
            work.push(Work::PreparationFailed {
                task,
                failure: brief::GatherFailure {
                    fate: brief::GatherFate::Permanent,
                    section: b"transcript",
                    words: Box::from(b"could not be read within its bounds".as_slice()),
                },
            });
            return out;
        };
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::TranscriptTail,
                text,
                limit: env.limits.brief_core_budgets.tail,
                priority: 2,
                required: true,
            })
            .expect("tail brief room");
    }
    if !super::preferences::plan(core, &env.limits, work, task, &mut wanted) {
        return out;
    }
    plan_note_index(&mut wanted, &env.limits, &lines, more);
    if !context.delegates.is_empty()
        && wanted.room() > 0
        && let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Delegates)
    {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Plan,
                text,
                limit: env.limits.brief_core_budgets.plan,
                priority: 3,
                required: false,
            })
            .expect("delegate section room");
    }
    if context.tries != tasks::Tries::NONE
        && wanted.room() > 0
        && let Some(text) = crate::read_brief_part(core, text_limits, task, crate::BriefPart::Attempts)
    {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::Attempts,
                text,
                limit: env.limits.brief_core_budgets.attempts,
                priority: 4,
                required: false,
            })
            .expect("attempt brief room");
    }
    let charter = core.charter_of(task);
    let mut documents = List::with_capacity(env.limits.tasks.charter_documents.min(wanted.room()));
    for (index, document) in charter.documents.iter().enumerate() {
        if wanted.room() == 0 {
            break;
        }
        let token = Token::new(u64::try_from(index).expect("document index").checked_add(1).expect("document token"));
        wanted
            .push(brief::Planned::Connector {
                name: b"document",
                connector: document.connector,
                kind: crate::DOCUMENT_KIND,
                token,
                size: 0,
                limit: env.limits.brief.read_bytes,
                priority: crate::DOCUMENT_PRIORITY,
                required: false,
            })
            .expect("document section room checked");
        documents.push(crate::PlannedDocument { token, document: document.clone() }).expect("charter document bound");
    }
    let parent = brief_parent(context.requester);
    out.push(Request::Now(Box::new(Now::BriefCorePlanned {
        task,
        parent,
        sections: wanted,
        documents: documents.into_boxed(),
    })));
    out
}

fn tail_wanted(core: &Core, task: u64, transient: bool) -> bool {
    let context = core.contexts.get(&task).expect("brief owns a preparation context");
    let standing = context.ended_by == tasks::EndedBy::Standing;
    if standing && core.tasks.task(task).expect("preparing task").retry_notice.is_none() {
        return false;
    }
    let transcript = core.transcripts.get(&task).expect("brief owns a loaded transcript");
    if transcript.closed {
        return transcript.last.is_some();
    }
    standing || core.transcript_oversized(task) || transient || !core.charter_of(task).policy.resume
}

pub(crate) fn brief_parent(requester: tasks::Party) -> Option<u64> {
    match requester {
        tasks::Party::Task(parent) => Some(parent),
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
    }
}

pub(crate) fn plan_note_index(
    wanted: &mut List<brief::Planned>,
    limits: &Limits,
    lines: &List<notes::Line>,
    more: u32,
) {
    if wanted.room() > 0 {
        wanted
            .push(brief::Planned::Core {
                kind: brief::Core::NotesIndex,
                text: crate::note_index_text(lines, more, limits.brief.read_bytes.min(limits.brief_core_budgets.notes)),
                limit: limits.brief_core_budgets.notes,
                priority: 4,
                required: false,
            })
            .expect("note index section room");
    }
}

/// Load the saved window for resume, or its tail when closed (jig/docs/design/domain/engine/executors.md, 2).
fn transcript_from(core: &Core, context: &tasks::RunContext) -> Option<(u64, u32)> {
    let proof = core.proofs.get(&context.task)?;
    let window = proof.window?;
    Some((window.attempt, window.turn))
}

fn prepare_agent(
    core: &mut Core,
    work: &mut Queue<Work>,
    context: Box<tasks::RunContext>,
    from: Option<(u64, u32)>,
) -> Queue<Request> {
    let mut out = Queue::uncapped();
    let number = context.task;
    if let Some((token, _)) = core.workspace_flight(number) {
        drop(core.land_kind(token, crate::routing::flight::Kind::Workspace));
    }
    let previous_attempt = context.previous_attempt;
    super::results::clear(core, number);
    assert!(core.contexts.insert(number, context).is_ok(), "bounded activation context");
    core.begin_transcript(number, previous_attempt, from);
    work.push(Work::Event(Event::Tasks(tasks::Event::Prepare {
        reply_to: ReplyTo::new(core.internal(crate::routing::flight::Internal::Acknowledge)),
        task: number,
    })));
    if let Some(from) = from {
        out.push(Request::Now(Box::new(Now::StartPreparation { task: number, from })));
    } else {
        work.push(Work::StartBrief { task: number });
    }
    out
}

pub(crate) fn route_activate(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    context: Box<tasks::RunContext>,
    ready: bool,
    busy: bool,
) -> Queue<Request> {
    let number = context.task;
    let mut out = Queue::uncapped();
    let Some(context) = core.tasks.run_context(number) else {
        return out;
    };
    if core.tasks.task(number).expect("current activation").phase != tasks::Phase::Active(tasks::Active::Due) {
        return out;
    }
    if !ready || !core.start_opened() || core.start_stopped() {
        remember_due(core, context);
    } else if core.tasks.recurring_template(number).is_none() {
        match context.executor {
            tasks::Executor::Person(_) => {}
            tasks::Executor::Job { .. } => match super::jobs::admission(core, &context, env.wall, Box::new([])) {
                Ok(()) => return super::jobs::activate(core, work, context),
                Err(crate::RunAdmission::Wait(_)) => remember_due(core, context),
                Err(crate::RunAdmission::Hold(why)) => {
                    work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task: number, why })));
                }
                Err(crate::RunAdmission::Allow(_)) => unreachable!("job admission has no run allowance"),
            },
            tasks::Executor::Procedure { connector, .. } => {
                if let Some(step) = context.previous_attempt.checked_add(1) {
                    out.push(Request::Ask { connector, ask: Ask::StartProcedure { context, step } });
                } else {
                    work.push(Work::Event(Event::Tasks(tasks::Event::Hold {
                        task: number,
                        why: tasks::Hold::Effects,
                    })));
                }
            }
            tasks::Executor::Agent { .. } => match core.run_admission(&context, env.wall, Box::new([])) {
                crate::RunAdmission::Allow(_) => {
                    let from = transcript_from(core, &context);
                    if busy && from.is_some() {
                        remember_due(core, context);
                    } else {
                        return prepare_agent(core, work, context, from);
                    }
                }
                crate::RunAdmission::Wait(_) => remember_due(core, context),
                crate::RunAdmission::Hold(why) => {
                    work.push(Work::Event(Event::Tasks(tasks::Event::Hold { task: number, why })));
                }
            },
        }
    }
    out
}

pub(crate) fn route_preparation_failed(
    core: &mut Core,
    work: &mut Queue<Work>,
    task: u64,
    failure: brief::GatherFailure,
) -> Queue<Request> {
    drop(core.job_workspaces.remove(&task));
    super::results::clear(core, task);
    drop(core.contexts.remove(&task));
    drop(core.dependency_results.remove(&task));
    drop(core.transcripts.remove(&task));
    let mut out = Queue::uncapped();
    let event = preparation_failure(task, failure, &mut out);
    work.push(Work::Event(Event::Tasks(event)));
    out.push(Request::Now(Box::new(Now::RunPreparationFailed { task })));
    out
}

/// One classification and log route for every brief failure (domain/engine/briefs.md, 1.1).
pub(crate) fn preparation_failure(task: u64, failure: brief::GatherFailure, out: &mut Queue<Request>) -> tasks::Event {
    let reason = failure.reason();
    let fate = failure.fate;
    out.push(Request::Now(Box::new(Now::BriefFailed { task, failure })));
    match fate {
        brief::GatherFate::Capacity => tasks::Event::PreparationPaused { task, reason },
        brief::GatherFate::Transient => {
            tasks::Event::PreparationCounted { task, class: tasks::Class::Transient, reason }
        }
        brief::GatherFate::Permanent => {
            tasks::Event::PreparationCounted { task, class: tasks::Class::Permanent, reason }
        }
    }
}

pub(crate) fn route_claim_refused(core: &mut Core, work: &mut Queue<Work>, task: u64) -> Queue<Request> {
    let mut out = Queue::uncapped();
    if core.claiming.remove(&task).is_some() {
        drop(core.job_workspaces.remove(&task));
        core.restore_replaced_proof(task);
        out.push(Request::Now(Box::new(Now::DropAssignment { task })));
        let event = preparation_failure(
            task,
            brief::GatherFailure {
                fate: brief::GatherFate::Capacity,
                section: b"claim",
                words: Box::from(b"refused".as_slice()),
            },
            &mut out,
        );
        work.push(Work::Event(Event::Tasks(event)));
    }
    out
}

pub(crate) fn route_claim_prepared(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    attempt: u64,
    budget: u64,
    writes: Box<[tasks::Name]>,
) -> Queue<Request> {
    if let Some(&(prepared, offered, session_end)) = core.claiming.get(&task)
        && prepared == attempt
    {
        let mut reasons = tasks::Reasons::FIRST;
        if let Some(context) = core.contexts.remove(&task) {
            reasons = context.reasons;
            let valid = match super::parts::preparation_facts(core, &context) {
                Some(current) => match core.run_admission(&current, env.wall, Box::new([])) {
                    crate::RunAdmission::Allow(allowance) => {
                        let proof = core.proofs.get(&task).expect("pending proof");
                        allowance.spend == budget
                            && allowance.choice.accounts == proof.accounts
                            && Some(allowance.choice.placement.channel) == proof.placement
                    }
                    crate::RunAdmission::Wait(_) | crate::RunAdmission::Hold(_) => false,
                },
                None => false,
            };
            if !valid {
                let _: Option<(u64, tasks::Reasons, Option<tasks::SessionEnd>)> = core.claiming.remove(&task);
                core.restore_replaced_proof(task);
                work.push(Work::Event(Event::Tasks(tasks::Event::PreparationWaiting { task })));
                let mut out = Queue::uncapped();
                out.push(Request::Now(Box::new(Now::DropAssignment { task })));
                return out;
            }
        }

        let earlier_transcript = match core.proofs.get(&task) {
            Some(proof) => proof.conversation != attempt,
            None => false,
        };
        let turned = match core.tasks.task(task) {
            Some(row) => row.ever_turned,
            None => false,
        };
        let resumed = earlier_transcript && turned;
        work.push(Work::Event(Event::Tasks(tasks::Event::Claim {
            session_end,
            offered,
            reasons,
            account: match core.proofs.get(&task) {
                Some(proof) => match proof.accounts.first() {
                    Some(binding) => Some(binding.account),
                    None => None,
                },
                None => None,
            },
            reply_to: ReplyTo::new(Token::new(task)),
            task,
            attempt,
            resumed,
            budget,
            writes,
        })));
    }
    Queue::uncapped()
}

pub(crate) fn transcript_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    task: u64,
    load: crate::routing::TranscriptLoad,
) -> Queue<Request> {
    if !core.transcripts.contains_key(&task) {
        return Queue::uncapped();
    }
    match load {
        crate::routing::TranscriptLoad::Failed => route_preparation_failed(
            core,
            work,
            task,
            brief::GatherFailure {
                fate: brief::GatherFate::Transient,
                section: b"transcript",
                words: Box::from(b"its window could not be loaded".as_slice()),
            },
        ),
        crate::routing::TranscriptLoad::Unread => transcript_unread(core, work, task),
        crate::routing::TranscriptLoad::Cut { turn, bytes, done } => {
            if !core.append_cut_transcript(task, turn, bytes) {
                return route_preparation_failed(
                    core,
                    work,
                    task,
                    brief::GatherFailure {
                        fate: brief::GatherFate::Transient,
                        section: b"transcript",
                        words: Box::from(b"its window could not be loaded".as_slice()),
                    },
                );
            }
            if done { start_brief(core, env, work, task) } else { Queue::uncapped() }
        }
        crate::routing::TranscriptLoad::Page { rows, done } => {
            for (row, text) in rows {
                if !core.append_transcript(task, row, text) {
                    return route_preparation_failed(
                        core,
                        work,
                        task,
                        brief::GatherFailure {
                            fate: brief::GatherFate::Transient,
                            section: b"transcript",
                            words: Box::from(b"its window could not be loaded".as_slice()),
                        },
                    );
                }
            }
            if done {
                if !core.transcript_readable(task) {
                    return transcript_unread(core, work, task);
                }
                return start_brief(core, env, work, task);
            }
            Queue::uncapped()
        }
    }
}

pub(crate) fn transcript_unread(core: &mut Core, work: &mut Queue<Work>, task: u64) -> Queue<Request> {
    if let Some(transcript) = core.transcripts.get(&task)
        && transcript.closed
    {
        let previous_attempt = transcript.previous_attempt;
        drop(core.transcripts.remove(&task));
        core.begin_transcript(task, previous_attempt, None);
        work.push(Work::StartBrief { task });
    } else if core.transcripts.remove(&task).is_some() {
        // An unread open window counts once and closes in that decision (domain/engine/executors.md, 2).
        core.close_window(task);
        let mut out = route_preparation_failed(
            core,
            work,
            task,
            brief::GatherFailure {
                fate: brief::GatherFate::Transient,
                section: b"transcript",
                words: Box::from(b"its window could not be read".as_slice()),
            },
        );
        if let Some(proof) = core.proofs.get(&task) {
            out.push(Request::Write(crate::Write::Save(crate::Record::Core(crate::CoreRecord::RunProof(
                proof.clone(),
            )))));
        }
        return out;
    }
    Queue::uncapped()
}
