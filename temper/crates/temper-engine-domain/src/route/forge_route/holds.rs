//! Retained accounts of procedure decisions and resource holds
//! (temper/docs/design/domain/forge/change/repairs.md and temper/docs/design/domain/forge/change/drift.md, 1 and temper/docs/design/domain/forge/branches.md).
use super::forge::hold_words::{append_recovery, drift_change};
pub(super) use super::forge::hold_words::{effect, findings};
use super::{
    Domain, EffectDescription, Env, Limits, List, ProcedureStep, StepEnd, Work, append, append_change_replies,
    authority, decimal, forge, tasks,
};
use alloc::boxed::Box;

mod change;
pub(super) use change::{
    account as change_account, fallback as change_fallback, fallback_reason as change_fallback_reason,
};

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
    if let Some(row) = domain.engine.core().tasks.task(task)
        && let tasks::Party::Task(target) = row.requester
    {
        decision.messages = Box::new([jig_domain_core::StepMessage {
            notice: None,
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

/// Keep the operation and admission refusal together on the task's account.
pub(super) fn effect_refusal(
    domain: &Domain,
    task: u64,
    description: &EffectDescription,
    why: jig_domain_core::effects::connector::EffectRefusal,
    findings_: &[authority::Finding],
) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    effect(&mut words, description);
    append(
        &mut words,
        match why {
            jig_domain_core::effects::connector::EffectRefusal::Authority => b" was vetoed by authority",
            jig_domain_core::effects::connector::EffectRefusal::Obsolete => {
                b" was refused: its procedure step or entry is no longer valid"
            }
            jig_domain_core::effects::connector::EffectRefusal::Description => {
                b" was refused: its connector description is invalid"
            }
            jig_domain_core::effects::connector::EffectRefusal::Capacity => {
                b" was refused: effect admission has no room"
            }
            jig_domain_core::effects::connector::EffectRefusal::Dependencies => {
                b" was refused: its dependency is still pending"
            }
            jig_domain_core::effects::connector::EffectRefusal::Proposal => {
                b" was refused: its proposal could not be admitted or addressed"
            }
            jig_domain_core::effects::connector::EffectRefusal::Unavailable => {
                b" was refused: effect admission is unavailable"
            }
        },
    );
    findings(&mut words, findings_);
    recovery(domain, task, &mut words);
    words.into_boxed()
}

pub(super) fn recovery(domain: &Domain, task: u64, words: &mut List<u8>) {
    let revision = domain.engine.core().tasks.task(task).expect("held procedure").revisions.hold.saturating_add(1);
    append_recovery(task, revision, words);
}

pub(crate) fn refused_step(
    domain: &mut Domain,
    env: &Env<Limits>,
    task: u64,
    step: u64,
    findings_: &[authority::Finding],
) {
    let Some((connector, code @ (2 | 3), current)) = domain.engine.core().tasks.procedure_due(task) else { return };
    if current != step || connector != domain.config.forge_connector {
        return;
    }
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, if code == 3 { b"Landing prepare delegate refused" } else { b"Delegate refused" });
    findings(&mut words, findings_);
    recovery(domain, task, &mut words);
    let decision = decision(domain, env, task, words.into_boxed());
    let decision = append_change_replies(domain, env, task, decision);
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
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

/// Retain the resource and its recorded observation in the goal's drift account.
pub(super) fn drift(domain: &mut Domain, task: u64, resource: &forge::Name) {
    let recorded = match domain.forge.hold(resource) {
        Some(hold) => hold.drift.as_ref(),
        None => None,
    };
    let words = drift_words(domain, resource, recorded);
    hold_as(domain, task, tasks::Hold::Drift, words.as_slice());
}

fn drift_words(domain: &Domain, resource: &forge::Name, recorded: Option<&forge::Drift>) -> List<u8> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, b"Resource drift");
    if let Some(drift) = recorded {
        append(&mut words, b" at ");
        append(&mut words, &decimal(drift.at.as_nanos()));
        append(&mut words, b" ns since Unix epoch: ");
        drift_change(&mut words, &drift.change);
        if let forge::DriftChange::Branch { .. } = drift.change
            && let Some(entry) = domain.forge.nearest_write(resource)
        {
            change::last_write(&mut words, entry);
        }
    }
    append(&mut words, b"; ");
    resource_words(domain, resource, &mut words);
    words
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
    task: Option<u64>,
    operation: &[u8],
    resource: Option<&forge::Name>,
    cause: forge::RefusalCause,
) -> Box<[u8]> {
    let mut context = None;
    if let forge::RefusalCause::Limit { .. } = cause
        && let Some(task) = task
        && let Some(row) = domain.forge.change(task)
    {
        let use_branch = match resource {
            None => true,
            Some(name) => match name.what {
                forge::What::Repository => {
                    name.forge == row.repository.forge && name.repository == row.repository.repository
                }
                forge::What::Branch(_) | forge::What::Pull(_) | forge::What::Issue(_) => false,
            },
        };
        if use_branch {
            context = Some(forge::branch_name(row.repository, &row.branch));
        }
    }
    let resource = context.as_ref().or(resource);
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, operation);
    if let Some(resource) = resource {
        append(&mut words, b" on ");
        resource_words(domain, resource, &mut words);
    }
    let reason = cause.words();
    if let forge::RefusalCause::Limit { .. } = cause {
        let room = tasks::HOLD_ACCOUNT_BYTES
            .saturating_sub(u32::try_from(reason.len()).expect("bounded refusal words"))
            .saturating_sub(2);
        if words.len() > room {
            words.truncate(room.saturating_sub(3));
            append(&mut words, b"...");
        }
    }
    append(&mut words, b": ");
    append(&mut words, &reason);
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
    if domain.engine.core().tasks.task(task).is_none() {
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
    } else if let Some(context) = domain.engine.core().tasks.run_context(task)
        && let Some(words) = super::close::refusal(domain, &context)
    {
        hold(domain, task, words);
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
    fn a_long_resource_keeps_both_heads_in_the_drift_account() {
        let (domain, _, _) = crate::tests::live_task(Some(1), tasks::Executor::Agent { charter: 1 });
        let mut resource = goal_branch();
        let branch = [b'x'; 240];
        resource.what = forge::What::Branch(Box::new([branch.as_slice().into()]));
        let expected = [0xab; 32];
        let observed = [0xcd; 32];
        let recorded = forge::Drift {
            at: skein_lib::Wall::from_nanos(u64::MAX),
            change: forge::DriftChange::Branch { expected, observed: Some(observed) },
        };
        let mut words = drift_words(&domain, &resource, Some(&recorded));
        append_recovery(u64::MAX, u64::MAX, &mut words);
        for text in [
            super::super::commit_text(&expected),
            super::super::commit_text(&observed),
            b"18446744073709551615".as_slice().into(),
            b"branch ".as_slice().into(),
        ] {
            let mut found = false;
            for part in words.as_slice().windows(text.len()) {
                if part == text.as_ref() {
                    found = true;
                    break;
                }
            }
            assert!(found, "whole field {text:?} in {words:?}");
        }
        assert!(words.len() <= tasks::HOLD_ACCOUNT_BYTES);
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
        tasks::step(&mut domain.engine.core_mut().tasks, &env.with(env.limits.tasks), event, &mut Queue::uncapped());
        let tasks::Phase::Held { why, source: Some(source), .. } =
            &domain.engine.core().tasks.task(task).unwrap().phase
        else {
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
