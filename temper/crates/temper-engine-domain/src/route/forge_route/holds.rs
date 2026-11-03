//! Retained accounts of procedure decisions and resource holds
//! (temper/docs/design/domain/forge.md, 8.4 and 11).
use super::{
    Domain, EffectDescription, Env, Limits, List, ProcedureStep, StepEnd, Work, append, append_change_replies,
    authority, decimal, forge, tasks,
};
use alloc::boxed::Box;

pub(super) fn source(domain: &Domain, entry: Option<u64>, words: Box<[u8]>) -> tasks::HoldSource {
    tasks::HoldSource::Account { connector: domain.config.forge_connector, entry, words }
}

pub(super) fn decision(domain: &Domain, env: &Env<Limits>, task: u64, words: Box<[u8]>) -> ProcedureStep {
    decision_as(domain, env, task, tasks::Hold::Procedure, words)
}

pub(super) fn decision_as(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    why: tasks::Hold,
    words: Box<[u8]>,
) -> ProcedureStep {
    let mut decision =
        ProcedureStep::new(StepEnd::HoldFrom { why, source: Box::new(source(domain, None, words.clone())) });
    if let Some(row) = domain.core.tasks.task(task)
        && let tasks::Party::Task(target) = row.requester
    {
        decision.messages = Box::new([jig_domain_core::StepMessage {
            target,
            kind: tasks::MessageKind::Words,
            words: forge::text::fit(
                &words,
                usize::try_from(env.limits.tasks.message_bytes).expect("bounded message"),
                b"...",
            ),
        }]);
    }
    decision
}

pub(super) fn findings(words: &mut List<u8>, findings: &[authority::Finding]) {
    for finding in findings {
        append(words, b"; ");
        append(
            words,
            match finding {
                authority::Finding::Executor { .. } => b"executor outside its delegation authority",
                authority::Finding::Tasks { .. } => b"no remaining task authority",
                authority::Finding::Oversized => b"request exceeds its configured bounds",
                authority::Finding::UnknownProject => b"project policy missing",
                authority::Finding::UnknownRole => b"role missing",
                authority::Finding::Authority { lacks, .. } => {
                    for (missing, name) in [
                        (lacks.tools, b"tool authority".as_slice()),
                        (lacks.grants, b"resource grants"),
                        (lacks.executors, b"executor authority"),
                        (lacks.tasks, b"task capacity"),
                        (lacks.depth, b"delegation depth"),
                        (lacks.spend, b"spend"),
                        (lacks.deadline, b"deadline authority"),
                        (lacks.notes, b"note scopes"),
                        (lacks.models, b"models"),
                    ] {
                        if missing {
                            append(words, b"lacks ");
                            append(words, name);
                            append(words, b"; ");
                        }
                    }
                    continue;
                }
                authority::Finding::Spend { .. } | authority::Finding::RunBudget => {
                    b"insufficient spend for the delegate"
                }
                authority::Finding::Grant { .. } => b"operation has no authority grant",
                authority::Finding::Required { .. } => b"required fact missing or pending",
                authority::Finding::Failed { .. } => b"required fact failed",
                authority::Finding::Unguarded { .. } => b"required fact cannot be guarded",
                authority::Finding::ResourceAccess => b"resource access refused",
                authority::Finding::ResourceUnavailable => b"resource unavailable",
                authority::Finding::Tool => b"tool family not granted",
                authority::Finding::Arithmetic => b"authority accounting overflow",
                authority::Finding::RunCap => b"run budget exceeds its cap",
                authority::Finding::Deadline => b"deadline passed",
                authority::Finding::Account => b"model account unusable",
                authority::Finding::Writer => b"resource has another writer",
                authority::Finding::Reference => b"task reference not granted",
                authority::Finding::Scope { .. } => b"note scope not granted",
                authority::Finding::Price { .. } => b"effect price exceeds its funding",
                authority::Finding::Unpermitted => b"request not permitted",
                authority::Finding::Undecidable => b"proposal decision not permitted",
                authority::Finding::PeriodSpend => b"period spend exceeds its ceiling",
            },
        );
    }
}

pub(super) fn effect(words: &mut List<u8>, description: &EffectDescription) {
    append(
        words,
        match description.effect.kind {
            3 => b"open pull request",
            4 => b"merge pull request",
            5 => b"post review",
            6 => b"set status",
            7 => b"post comment",
            8 => b"create issue",
            9 => b"create branch",
            10 => b"edit pull request",
            11 => b"update pull request",
            12 => b"close or reopen pull request",
            13 => b"edit issue",
            14 => b"set reviewers",
            15 => b"delete branch",
            _ => b"forge effect",
        },
    );
    append(words, b" on ");
    for (index, segment) in description.effect.name.segments.iter().enumerate() {
        if index > 0 {
            append(words, b"/");
        }
        append(words, segment);
    }
    if words.len() > 180 {
        words.truncate(177);
        append(words, b"...");
    }
}

pub(super) fn recovery(domain: &Domain, task: u64, words: &mut List<u8>) {
    let mut tail = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut tail, b"; resolve this reason, then task release ");
    append(&mut tail, &decimal(task));
    append(&mut tail, b" --hold ");
    let revision = domain.core.tasks.task(task).expect("held procedure").revisions.hold.saturating_add(1);
    append(&mut tail, &decimal(revision));
    append(&mut tail, b"; or task cancel ");
    append(&mut tail, &decimal(task));
    append(&mut tail, b" REASON");
    let room = tasks::HOLD_ACCOUNT_BYTES.checked_sub(tail.len()).expect("recovery fits the account");
    if words.len() > room {
        words.truncate(room.saturating_sub(3));
        append(words, b"...");
    }
    append(words, tail.as_slice());
}

pub(crate) fn refused_step(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    step: u64,
    findings_: &[authority::Finding],
) {
    let Some((connector, code @ 2, current)) = domain.core.tasks.procedure_due(task) else { return };
    if current != step || connector != domain.config.forge_connector {
        return;
    }
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, b"Delegate refused");
    findings(&mut words, findings_);
    recovery(domain, task, &mut words);
    let decision = decision(domain, env, task, words.into_boxed());
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep {
        task,
        step,
        connector,
        code,
        decision: append_change_replies(domain, env, task, decision),
    }));
}

pub(crate) fn hold(domain: &mut Domain, task: u64, reason: &[u8]) {
    hold_as(domain, task, tasks::Hold::Procedure, reason);
}

pub(super) fn taken_words(domain: &Domain, resource: &forge::Name, by: u64) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    resource_words(domain, resource, &mut words);
    append(&mut words, b" is taken by holder ");
    append(&mut words, &decimal(by));
    words.into_boxed()
}

pub(super) fn make_words(domain: &Domain, resource: &forge::Name, reason: &[u8]) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    resource_words(domain, resource, &mut words);
    append(&mut words, b" could not be made: ");
    append(&mut words, reason);
    words.into_boxed()
}

pub(super) fn lost_words(domain: &Domain, task: u64, attempt: u64, resource: &forge::Name, reason: &[u8]) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, b"task ");
    append(&mut words, &decimal(task));
    append(&mut words, b"'s ");
    resource_words(domain, resource, &mut words);
    append(&mut words, b" could not be read again after run ");
    append(&mut words, &decimal(attempt));
    append(&mut words, b" ended: ");
    append(&mut words, reason);
    words.into_boxed()
}

pub(super) fn refused_words(
    domain: &Domain,
    operation: &[u8],
    resource: Option<&forge::Name>,
    cause: forge::RefusalCause,
) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, operation);
    if let Some(resource) = resource {
        append(&mut words, b" on ");
        resource_words(domain, resource, &mut words);
    }
    append(&mut words, b": ");
    append(&mut words, &cause.words());
    words.into_boxed()
}

fn resource_words(domain: &Domain, resource: &forge::Name, words: &mut List<u8>) {
    match &resource.what {
        forge::What::Branch(segments) => {
            append(words, b"branch ");
            for (index, segment) in segments.iter().enumerate() {
                if index > 0 {
                    append(words, b"/");
                }
                append(words, segment);
            }
        }
        forge::What::Repository => append(words, b"repository"),
        forge::What::Pull(number) | forge::What::Issue(number) => {
            append(words, b"item ");
            append(words, &decimal(*number));
        }
    }
    append(words, b" in ");
    let provider = skein_forge::api::Repository { forge: resource.forge, repository: resource.repository };
    if let Some(repository) = domain.forge.repository(provider) {
        append(words, &repository.owner);
        append(words, b"/");
        append(words, &repository.name);
    } else {
        append(words, b"forge ");
        append(words, &decimal(u64::from(resource.forge)));
        append(words, b" repository ");
        append(words, &decimal(u64::from(resource.repository)));
    }
}

pub(crate) fn hold_as(domain: &mut Domain, task: u64, why: tasks::Hold, reason: &[u8]) {
    if domain.core.tasks.task(task).is_none() {
        return;
    }
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, reason);
    if let Some(row) = domain.forge.change(task) {
        append(&mut words, b" for branch ");
        append(&mut words, &row.branch);
        if let Some(repository) = domain.forge.repository(row.repository) {
            append(&mut words, b" in ");
            append(&mut words, &repository.owner);
            append(&mut words, b"/");
            append(&mut words, &repository.name);
        }
    }
    recovery(domain, task, &mut words);
    domain.work.push(Work::Tasks(tasks::Event::HoldFrom {
        task,
        why,
        source: Box::new(source(domain, None, words.into_boxed())),
    }));
}

pub(crate) fn start_refused(domain: &mut Domain, task: u64) {
    if let Some(row) = domain.forge.change(task)
        && row.effect == super::forge_change::EffectResult::Failed
        && let Some(account) = &row.effect_account
    {
        domain.work.push(Work::Tasks(tasks::Event::HoldFrom {
            task,
            why: tasks::Hold::EffectFailed,
            source: Box::new(source(domain, Some(account.entry), account.words.clone())),
        }));
    } else {
        hold(domain, task, b"change procedure cannot start from its repository and branch parameters");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Queue;

    fn goal_branch() -> forge::Name {
        forge::Name {
            forge: 1,
            repository: 2,
            what: forge::What::Branch(Box::new([
                b"temper".as_slice().into(),
                b"7".as_slice().into(),
                b"g7".as_slice().into(),
            ])),
        }
    }

    #[test]
    fn a_failed_goal_branch_making_holds_the_goal_with_the_forge_answer() {
        let (mut domain, env, task) = crate::tests::live_task(Some(1), tasks::Executor::Agent { charter: 1 });
        let mut out = Queue::uncapped();
        out.push(forge::Request::MakeFailed {
            task,
            resource: goal_branch(),
            reason: b"forge refused permission".as_slice().into(),
        });
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        super::super::outputs(&mut domain, &env, &mut decision, &mut out);
        let Some(Work::Tasks(event @ tasks::Event::HoldFrom { .. })) = domain.work.pop() else {
            panic!("the making failure holds its goal");
        };
        tasks::step(
            &mut domain.core.tasks,
            &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
            event,
            &mut Queue::uncapped(),
        );
        let tasks::Phase::Held { why, source: Some(source), .. } = &domain.core.tasks.task(task).unwrap().phase else {
            panic!("the goal retains the making's failure");
        };
        assert_eq!(*why, tasks::Hold::EffectFailed);
        let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("retained forge answer") };
        assert!(
            words
                .starts_with(b"branch temper/7/g7 in forge 1 repository 2 could not be made: forge refused permission")
        );
    }

    #[test]
    fn a_taking_routes_the_cores_entry_to_the_forge() {
        const ENTRY: u64 = 21;
        let (mut domain, env, task) = crate::tests::live_task(Some(1), tasks::Executor::Agent { charter: 1 });
        let connector = domain.config.forge_connector;
        let resource = super::super::hub_name(connector, &goal_branch(), &env.limits.tasks).unwrap();
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::asks::ask(
            &mut domain,
            &env,
            &mut decision,
            connector,
            jig_domain_core::Ask::Hold { task, resource, entry: ENTRY },
        );
        let Some(Work::Forge(event)) = domain.work.pop() else { panic!("the forge hears the taking") };
        assert_eq!(event, forge::Event::Hold { entry: ENTRY, task, resource: goal_branch(), from: None });
    }
}
