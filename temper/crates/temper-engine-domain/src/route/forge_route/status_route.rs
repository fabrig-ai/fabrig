//! Home's status procedure and its agent (temper/docs/design/domain/forge/attention.md, 3).
use super::{Box, Domain, Env, Limits, ProcedureStep, RunWorkspace, STATUS_CODE, StepEnd, Work, forge, tasks};

pub(super) fn start_status_run(
    domain: &mut Domain,
    env: &Env<Limits>,
    project: u32,
    words: Box<[u8]>,
    signers: &[u64],
) {
    let Some(seed) = domain.core.repair_seed(project) else {
        domain.work.push(Work::Forge(forge::Event::StatusRefused { project }));
        return;
    };
    let task = seed.number();
    let words = goal_words(domain, env, project, &words, signers);
    domain.work.push(Work::Core(jig_domain_core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: STATUS_CODE },
        spec: tasks::Spec { job: None, asks: false, words, parameters: Box::new([]), inputs: Box::new([]) },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        holdings: Box::new([]),
    }));
    domain.work.push(Work::Forge(forge::Event::StatusTask { project, task }));
}

fn goal_words(domain: &Domain, env: &Env<Limits>, project: u32, facts: &[u8], signers: &[u64]) -> Box<[u8]> {
    let mut words = super::List::uncapped();
    super::append(&mut words, facts);
    super::append(&mut words, b"Live goals:\n");
    for candidate in domain.core.tasks.view_tasks() {
        let Some(row) = domain.core.tasks.task(candidate.number) else { continue };
        if row.project != project || row.tracked.is_none() {
            continue;
        }
        let Ok(chip) = jig_domain_core::documents::chip_from_row(&crate::core_limits(&env.limits), row) else {
            continue;
        };
        super::append(&mut words, &chip.title.bytes);
        super::append(&mut words, b": ");
        super::append(&mut words, skein_lib::Decimal::of(u64::from(row.ended_delegates.done)).as_bytes());
        super::append(&mut words, b" of ");
        let ended = row.ended_delegates;
        let total = u32::try_from(row.delegates.len())
            .expect("bounded live delegates")
            .saturating_add(ended.done)
            .saturating_add(ended.failed);
        super::append(&mut words, skein_lib::Decimal::of(u64::from(total)).as_bytes());
        super::append(&mut words, b" delegates done\n");
    }
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
    forge::text::fit(words.as_slice(), skein_lib::usize_of(env.limits.tasks.spec_bytes), b"...")
}

fn signer_name(domain: &Domain, project: u32, person: u64) -> &[u8] {
    if let Some(members) = domain.core.people.project_members(project) {
        for member in members {
            if member.person == person
                && let Some(identity) = domain.core.people.identity(person)
            {
                return if identity.name.is_empty() { &identity.login } else { &identity.name };
            }
        }
    }
    b"no longer a member"
}

pub(crate) fn start_status_step(domain: &mut Domain, env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
    let Some(row) = domain.core.tasks.task(task) else { return };
    let offered = domain.core.tasks.procedure_offered(task).unwrap_or(0);
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

fn delegate(domain: &Domain, env: &Env<Limits>, task: u64) -> Option<super::super::Delegate> {
    let context = domain.core.tasks.delegation(task)?;
    let mut authority = super::child_authority(&context)?;
    authority.tools = tasks::Tools(0);
    authority.grants = Box::new([]);
    authority.notes = tasks::Scopes(0);
    authority.note_resources = Box::new([]);
    Some(super::super::Delegate {
        executor: tasks::Executor::Agent { charter: super::STATUS_CHARTER },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: domain.core.tasks.task(task)?.spec.words.clone(),
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
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}
fn finish(domain: &mut Domain, task: u64, report: Option<Box<[u8]>>) {
    domain.work.push(Work::Forge(forge::Event::StatusHeard { task, report }));
    step(domain, task, ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words: Box::new([]) })));
}

pub(crate) fn refused(domain: &mut Domain, task: u64, why: &jig_domain_core::StepRefusal) -> bool {
    let Some((connector, code, _)) = domain.core.tasks.procedure_due(task) else { return false };
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

pub(super) fn workspace(task: u64) -> RunWorkspace {
    RunWorkspace {
        workspace: super::ForgeWorkspace { repositories: Box::new([]) },
        workstream: task,
        writes: Box::new([]),
        names: Box::new([]),
        designs: Box::new([]),
        own_holds: Box::new([]),
    }
}
