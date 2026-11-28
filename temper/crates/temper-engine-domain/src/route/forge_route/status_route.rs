//! Home's status procedure and its agent (temper/docs/design/domain/forge/attention.md, 3).
use super::{Box, Domain, Env, Limits, ProcedureStep, RunWorkspace, STATUS_CODE, StepEnd, Work, forge, tasks};

mod milestones;
mod roles;
pub(crate) use roles::{Reading, indexed as notes_indexed, recalled as notes_recalled, refused as notes_refused};

#[expect(clippy::too_many_arguments, reason = "carry the status request's party through the existing root plumbing")]
pub(super) fn start_status_run(
    domain: &mut Domain,
    env: &Env<Limits>,
    project: u32,
    party: u64,
    since: skein_lib::Wall,
    words: Box<[u8]>,
    findings: &[u8],
    signers: &[u64],
) {
    let words = goal_words(domain, env, project, since, &words, findings, signers);
    roles::start(domain, env, project, party, words);
}

pub(crate) fn admission(domain: &Domain, project: u32, party: u64, task: u64) -> forge::Event {
    // ConnectorRepair drains OpenPeriod and Make before the queued StatusTask.
    match domain.engine.core().tasks.task(task) {
        Some(_) => forge::Event::StatusTask { project, party, task },
        None => forge::Event::StatusRefused { project, party },
    }
}

fn start_task(domain: &mut Domain, env: &Env<Limits>, project: u32, party: u64, words: &[u8]) {
    let Some(seed) = domain.engine.core_mut().pool_seed(project, party) else {
        domain.work.push(Work::Forge(forge::Event::StatusRefused { project, party }));
        return;
    };
    let task = seed.number();
    let words = forge::text::fit(words, skein_lib::usize_of(env.limits.tasks.spec_bytes), b"...");
    domain.work.push(Work::Core(jig_domain_core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: STATUS_CODE },
        spec: tasks::Spec { job: None, asks: false, words, parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        holdings: Box::new([]),
    }));
    domain.work.push(Work::Forge(forge::Event::StatusTask { project, party, task }));
}

fn goal_words(
    domain: &Domain,
    env: &Env<Limits>,
    project: u32,
    since: skein_lib::Wall,
    facts: &[u8],
    findings: &[u8],
    signers: &[u64],
) -> super::List<u8> {
    let mut words = super::List::uncapped();
    super::append(&mut words, facts);
    super::append(&mut words, b"Live goals:\n");
    for candidate in domain.engine.core().tasks.view_tasks() {
        let Some(row) = domain.engine.core().tasks.task(candidate.number) else { continue };
        if row.project != project || row.tracked.is_none() || tasks::gathering::executor(row.executor) {
            continue;
        }
        let Ok(chip) = jig_domain_core::documents::chip_from_row(&crate::core_limits(&env.limits), row) else {
            continue;
        };
        if let Some(parent) = milestone_parent(domain, row.requester)
            && let Ok(chip) = jig_domain_core::documents::chip_from_row(&crate::core_limits(&env.limits), parent)
        {
            super::append(&mut words, &chip.title.bytes);
            super::append(&mut words, " › ".as_bytes());
        }
        super::append(&mut words, &chip.title.bytes);
        super::append(&mut words, b": ");
        super::append(&mut words, skein_lib::Decimal::of(u64::from(row.ended_delegates.done)).as_bytes());
        super::append(&mut words, b" of ");
        let total = delegate_total(row);
        super::append(&mut words, skein_lib::Decimal::of(u64::from(total)).as_bytes());
        super::append(&mut words, b" parts done");
        append_open_questions(&mut words, open_to_a_person(domain, row));
        super::append(&mut words, b"\n");
    }
    milestones::append(domain, env, project, since, &mut words);
    if !signers.is_empty() {
        super::append(&mut words, b"People who signed:\n");
        for person in signers {
            super::append(&mut words, b"- person ");
            super::append(&mut words, skein_lib::Decimal::of(*person).as_bytes());
            super::append(&mut words, b": ");
            super::append(&mut words, signer_name(domain, project, *person));
            super::append(&mut words, b"\n");
        }
    }
    super::append(&mut words, findings);
    words
}

fn delegate_total(row: &tasks::TaskRecord) -> u32 {
    u32::try_from(row.delegates.len())
        .expect("bounded live delegates")
        .saturating_add(row.ended_delegates.done)
        .saturating_add(row.ended_delegates.failed)
}

fn open_to_a_person(domain: &Domain, row: &tasks::TaskRecord) -> u32 {
    let mut count = 0_u32;
    for number in &row.delegates {
        if let Some(delegate) = domain.engine.core().tasks.task(*number)
            && let tasks::Executor::Person(_) = delegate.executor
        {
            count = count.checked_add(1).expect("bounded live delegates");
        }
    }
    count
}

fn append_open_questions(words: &mut super::List<u8>, count: u32) {
    if count == 0 {
        return;
    }
    super::append(words, b", ");
    super::append(words, skein_lib::Decimal::of(u64::from(count)).as_bytes());
    super::append(words, if count == 1 { b" question open to a person" } else { b" questions open to a person" });
}

fn milestone_parent(domain: &Domain, requester: tasks::Party) -> Option<&tasks::TaskRecord> {
    let number = match requester {
        tasks::Party::Task(number) => number,
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return None,
    };
    let row = domain.engine.core().tasks.task(number)?;
    if tasks::gathering::executor(row.executor) { Some(row) } else { None }
}

fn signer_name(domain: &Domain, project: u32, person: u64) -> &[u8] {
    match domain.engine.core().people.member_name(project, person) {
        Some(name) => name,
        None => b"no longer a member",
    }
}

pub(crate) fn start_status_step(domain: &mut Domain, env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
    let Some(row) = domain.engine.core().tasks.task(task) else { return };
    let offered = domain.engine.core().tasks.procedure_offered(task).unwrap_or(0);
    for word in &row.inbox {
        if word.number > offered {
            continue;
        }
        if let tasks::Party::Task(_) = word.from {
            match &word.kind {
                tasks::MessageKind::Result(tasks::ResultKind::Report) => {
                    let (sentence, _) = forge::status_paragraphs(&word.words);
                    let report = if sentence.is_empty() { None } else { Some(word.words.clone()) };
                    finish(domain, task, report);
                    return;
                }
                tasks::MessageKind::Result(tasks::ResultKind::Failed | tasks::ResultKind::Cancelled) => {
                    finish(domain, task, None);
                    return;
                }
                tasks::MessageKind::Result(
                    tasks::ResultKind::Job
                    | tasks::ResultKind::Approved { .. }
                    | tasks::ResultKind::NoChange
                    | tasks::ResultKind::Verdict { .. }
                    | tasks::ResultKind::Change { .. },
                )
                | tasks::MessageKind::Escalation { .. }
                | tasks::MessageKind::Proposal { .. }
                | tasks::MessageKind::PersonStalled { .. }
                | tasks::MessageKind::ProposalDecision { .. }
                | tasks::MessageKind::SettingsDecision { .. }
                | tasks::MessageKind::Words
                | tasks::MessageKind::EffectNotMade { .. }
                | tasks::MessageKind::Amendment { .. }
                | tasks::MessageKind::FinishRequested { .. }
                | tasks::MessageKind::Question
                | tasks::MessageKind::Answer { .. }
                | tasks::MessageKind::Notice { .. }
                | tasks::MessageKind::Timer { .. }
                | tasks::MessageKind::News { .. }
                | tasks::MessageKind::Met { .. }
                | tasks::MessageKind::QuestionEnded { .. }
                | tasks::MessageKind::PassedQuestion { .. }
                | tasks::MessageKind::PassedAnswer { .. }
                | tasks::MessageKind::QuestionAnswered { .. }
                | tasks::MessageKind::QuestionOwed
                | tasks::MessageKind::NothingAwaited => {}
            }
        }
    }
    let delegated = !row.delegates.is_empty() || row.ended_delegates != tasks::Endings::ZERO;
    let mut decision = ProcedureStep::new(StepEnd::Wait);
    if !delegated {
        let Some(delegate) = delegate(domain, env, task) else {
            finish(domain, task, None);
            return;
        };
        decision.batch = Box::new([delegate]);
    }
    step(domain, task, decision);
}

pub(crate) fn check_account(domain: &mut Domain, procedure: u64) {
    let Some(row) = domain.engine.core().tasks.task(procedure) else { return };
    let Some(delegate) = row.delegates.first() else { return };
    let Err(jig_domain_core::RunWait::Account { account: None, .. }) = domain.engine.core().run_choice(*delegate)
    else {
        return;
    };
    let Some((party, project)) = domain.forge.status_owner(procedure) else { return };
    domain.work.push(Work::Forge(forge::Event::StatusRefused { project, party }));
    domain.work.push(Work::Tasks(tasks::Event::Cancel {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(procedure)),
        by: tasks::Party::Deployment { project },
        operation: tasks::Cancel {
            task: procedure,
            reason: b"no account pays for the person's brief".as_slice().into(),
            grace: None,
            message: None,
        },
    }));
}

fn delegate(domain: &Domain, env: &Env<Limits>, task: u64) -> Option<super::super::Delegate> {
    let context = domain.engine.core().tasks.delegation(task)?;
    let mut authority = super::child_authority(&context)?;
    authority.tools = tasks::Tools(0);
    authority.grants = Box::new([]);
    authority.notes = tasks::Scopes(0);
    authority.note_resources = Box::new([]);
    Some(super::super::Delegate {
        charter_scope: jig_domain_tasks::CharterScope::Project,
        executor: tasks::Executor::Agent { charter: super::STATUS_CHARTER },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: domain.engine.core().tasks.task(task)?.spec.words.clone(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        authority,
        deadline: tasks::Deadline::Named,
        llm: None,
        symbolic_grants: Box::new([]),
        dependencies: Box::new([]),
        wake: Some(tasks::WakePolicy::DEFAULT),
        tracked: None,
        label: 0,
        flags: tasks::Flags::PLAIN,
        budget: super::people::Budget::Named,
    })
}

fn step(domain: &mut Domain, task: u64, decision: ProcedureStep) {
    let Some((connector, code, step)) = domain.engine.core().tasks.procedure_due(task) else { return };
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}
fn finish(domain: &mut Domain, task: u64, report: Option<Box<[u8]>>) {
    domain.work.push(Work::Forge(forge::Event::StatusHeard { task, report }));
    step(domain, task, ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words: Box::new([]) })));
}

pub(crate) fn refused(domain: &mut Domain, task: u64, why: &jig_domain_core::StepRefusal) -> bool {
    let Some((connector, code, _)) = domain.engine.core().tasks.procedure_due(task) else { return false };
    if connector != domain.config.forge_connector || code != STATUS_CODE {
        return false;
    }
    match why {
        jig_domain_core::StepRefusal::Moved { .. } => false,
        jig_domain_core::StepRefusal::Denied { .. } | jig_domain_core::StepRefusal::Refused(_) => {
            finish(domain, task, None);
            true
        }
    }
}

pub(super) fn workspace(domain: &Domain, context: &tasks::RunContext) -> RunWorkspace {
    RunWorkspace {
        workspace: super::ForgeWorkspace {
            hook_timeout_max_seconds: super::hook_timeout_max_seconds(domain, context.project),
            ..super::ForgeWorkspace::default()
        },
        workstream: context.task,
        writes: Box::new([]),
        names: Box::new([]),
        designs: Box::new([]),
        own_holds: Box::new([]),
    }
}
