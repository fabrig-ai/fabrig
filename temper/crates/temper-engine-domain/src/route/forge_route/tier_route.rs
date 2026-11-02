//! Deployment-started tier run tasks (temper/docs/design/domain/after-landing.md, 3, 10).
use super::{
    AFTER_LANDING_CODE, Box, Domain, EffectDescription, EffectForm, Env, Family, Limits, List, ProcedureStep, Recovery,
    StepEnd, Work, authority, effect_access, forge, resource_name, tasks,
};

pub(super) fn start_tier_run(
    domain: &mut Domain,
    env: &Env<Limits>,
    repository: skein_forge::api::Repository,
    tier: Box<[u8]>,
    number: u64,
    tip: skein_forge::api::Commit,
) {
    if !make_tier_run(domain, env, repository, &tier, number, tip) {
        domain.work.push(Work::Forge(forge::Event::TierRefused { repository, tier, number }));
    }
}

fn make_tier_run(
    domain: &mut Domain,
    env: &Env<Limits>,
    repository: skein_forge::api::Repository,
    tier: &[u8],
    number: u64,
    tip: skein_forge::api::Commit,
) -> bool {
    let Some(adopted) = domain.forge.repository(repository) else { return false };
    let project = adopted.project;
    let mut words = List::with_capacity(env.limits.tasks.spec_bytes);
    let digits = skein_lib::Decimal::of(number);
    for part in [b"Tier ".as_slice(), tier, b" run ", digits.as_bytes()] {
        for byte in part {
            if words.push(*byte).is_err() {
                return false;
            }
        }
    }
    let Some(seed) = domain.core.procedure_seed(project) else { return false };
    let task = seed.number();
    domain.work.push(Work::Core(jig_domain_core::Event::ConnectorRepair {
        seed: Box::new(seed),
        executor: tasks::Executor::Procedure { connector: domain.config.forge_connector, code: AFTER_LANDING_CODE },
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: words.into_boxed(),
            parameters: Box::new([
                tasks::Parameter::Resource {
                    name: 1,
                    connector: domain.config.forge_connector,
                    resource: forge::tag(repository.forge, repository.repository),
                },
                tasks::Parameter::Bytes { name: 8, value: super::commit_text(&tip) },
                tasks::Parameter::Bytes { name: 10, value: Box::from(tier) },
                tasks::Parameter::Number { name: 11, value: number },
            ]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
        holdings: Box::new([]),
    }));
    domain.work.push(Work::Forge(forge::Event::TierTask { repository, tier: Box::from(tier), number, task }));
    true
}

pub(crate) fn start_tier_step(domain: &mut Domain, _env: &Env<Limits>, context: &tasks::RunContext) {
    let task = context.task;
    let Some(entry) = crate::fresh(&mut domain.core.counters, Family::ConnectorRow) else { return };
    let released = match domain.core.tasks.task(task) {
        Some(row) => row.due_reason == tasks::Why::Released,
        None => false,
    };
    domain.forge.offered_tier(task);
    domain.work.push(Work::Forge(forge::Event::StepTier {
        task,
        entry,
        heard: forge::TierHeard { released, cancelled: false },
    }));
}

pub(super) fn tier_decision(domain: &mut Domain, env: &Env<Limits>, task: u64, choice: forge::TierDecision) {
    let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };
    if connector != domain.config.forge_connector || code != AFTER_LANDING_CODE {
        return;
    }
    domain.forge.offered_tier(task);
    let decision = match choice {
        forge::TierDecision::Wait { until } => {
            domain.forge.wait_tier(&super::super::environment_forge(env), task, until);
            ProcedureStep::new(StepEnd::Wait)
        }
        forge::TierDecision::Held { words } => super::holds::decision(domain, env, task, words),
        forge::TierDecision::Done { words } => ProcedureStep::new(StepEnd::Result(tasks::TaskResult::Report { words })),
    };
    domain.work.push(Work::Core(jig_domain_core::Event::ProcedureStep { task, step, connector, code, decision }));
}

pub(super) fn describe_tier_effect(
    domain: &Domain,
    env: &Env<Limits>,
    task: u64,
    entry: u64,
) -> Option<EffectDescription> {
    let (provider, what, described) = domain.forge.describe_tier_effect(task, entry)?;
    let repository = domain.forge.repository(provider)?;
    let context = domain.core.tasks.delegation(task)?;
    if repository.project != context.project {
        return None;
    }
    Some(EffectDescription {
        waits_for_dependencies: false,
        connector: domain.config.forge_connector,
        purpose: entry,
        effect: authority::Effect {
            connector: domain.config.forge_connector,
            kind: described.kind,
            name: resource_name(repository, &what, env.limits.authority.segments)?,
            state: described.state,
            price: None,
            access: effect_access(repository, &what, described.kind),
            additional: Box::new([]),
            guards: Box::new([]),
        },
        form: match described.form {
            forge::EffectForm::Creation => EffectForm::Creation,
            forge::EffectForm::Transition => EffectForm::Transition,
            forge::EffectForm::Set => EffectForm::Set,
        },
        recovery: match described.recovery {
            forge::Recovery::Keyed => Recovery::Keyed,
            forge::Recovery::Conditional => Recovery::Conditional,
            forge::Recovery::Idempotent => Recovery::Idempotent,
            forge::Recovery::Unrecoverable => Recovery::Unrecoverable,
        },
    })
}
