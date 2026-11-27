//! A repair is the standing triager's goal, made with the finding decision (domain/red-repairs.md, 4).
use super::{Decision, Domain, Env, Filer, FindingRefusal, Holder, Limits, Work, forge, forge_route, tasks};
use jig_domain_core::effects::connector::SeededGoal;
use skein_lib::{Decimal, List};
use temper_engine_domain_translate::{
    goal_authority, repair_goal_spec, repair_goal_template, resource_report, stamp_repair,
};

pub(super) fn repair(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    by: Filer,
    project: u32,
    number: u32,
    act: &forge::RepairAct,
) -> Result<u64, FindingRefusal> {
    let repairs = match domain.forge.project_repairs(project) {
        Ok(repairs) => repairs.clone(),
        Err(why) => return Err(FindingRefusal::Words(why)),
    };
    let standing = match domain.forge.project_triager(project) {
        Some(owner) => super::stands(domain, project, owner),
        None => None,
    };
    let triager = match standing {
        Some(Holder::Task(task)) => task,
        Some(Holder::Party(_) | Holder::Role(_)) | None => {
            return Err(words(&[b"no triager stands in project ", Decimal::of(u64::from(project)).as_bytes()]));
        }
    };
    permission(domain, project, number, by)?;
    for &number in &act.more {
        permission(domain, project, number, by)?;
    }
    let mut writes = domain.forge.prepare_repair_findings(project, number, act)?;
    let deadline_ms = repairs.deadline_ms;
    let (goal, made) = prepare_goal(domain, env, decision, triager, repairs, &writes, &act.words)?;
    stamp_repair(&mut writes, goal, env.wall, deadline_ms);
    domain.work.push(Work::Tasks(made));
    domain.work.push(Work::RepairFindings { goal, writes });
    Ok(goal)
}

fn prepare_goal(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    triager: u64,
    repairs: forge::settings::Repairs,
    writes: &[forge::FindingWrite],
    explanation: &[u8],
) -> Result<(u64, tasks::Event), FindingRefusal> {
    let context = domain.engine.core().tasks.delegation(triager).ok_or(refused(tasks::Refusal::Tree))?;
    let project = context.project;
    let left = context
        .numbers
        .budget
        .saturating_sub(context.numbers.spent)
        .saturating_sub(context.numbers.spent_below)
        .saturating_sub(context.numbers.reserved);
    if left < repairs.goal.budget {
        return Err(words(&[
            b"the triager's period has ",
            Decimal::of(left).as_bytes(),
            b" left, short of the repair goal's budget ",
            Decimal::of(repairs.goal.budget).as_bytes(),
        ]));
    }
    let (label, flags) = repair_goal_template(domain.engine.core(), domain.config.forge_connector)?;
    let spec = repair_goal_spec(&domain.forge, env.limits.tasks.spec_bytes, writes, explanation, repairs.attempts)?;
    let seed = match domain.engine.core_mut().repair_seed(project) {
        Some(seed) => seed,
        None => domain.engine.core_mut().procedure_seed(project).ok_or(FindingRefusal::NoGoal)?,
    };
    let goal = seed.number();
    let root = domain.engine.core().tasks.root(triager).expect("live triager tree");
    let mut names = List::with_capacity(env.limits.tasks.holdings);
    let Ok(holdings) = forge_route::holdings_scope(domain, env).prepare(
        project,
        root,
        goal,
        tasks::Executor::Agent { charter: repairs.goal.charter },
        &spec,
        Some(triager),
        true,
        &[forge_route::GOAL_BRANCH_SYMBOL],
        &mut names,
    ) else {
        return Err(refused(tasks::Refusal::Holds));
    };
    let made = match seed.goal(
        triager,
        &context,
        SeededGoal {
            charter: repairs.goal.charter,
            spec,
            contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
            authority: goal_authority(domain.config.forge_connector, &context, false, repairs.goal.budget),
            llm: repairs.goal.llm,
            budget: repairs.goal.budget,
            tracked: Some(repairs.goal.priority),
            label,
            flags,
            holdings,
        },
    ) {
        Ok(made) => made,
        Err(why) => return Err(refused(why)),
    };
    report_holdings(domain, env, decision, project, &names)?;
    if let Err(why) = domain.engine.core().check_make(&super::super::environment_core(env).limits, &made) {
        return Err(refused(why));
    }
    Ok((goal, domain.engine.core_mut().bind_seeded_goal(made)))
}

fn report_holdings(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    project: u32,
    names: &List<forge::Name>,
) -> Result<(), FindingRefusal> {
    // Preflight sees the same reports as Make. These names are the new task's
    // private branches; creation itself still goes through Work::Tasks.
    for name in names {
        let Some((role, hold)) = domain.forge.resource_facts(name) else {
            return Err(refused(tasks::Refusal::Unreported));
        };
        let Some(event) = resource_report(domain.config.forge_connector, &env.limits.tasks, project, name, role, hold)
        else {
            return Err(refused(tasks::Refusal::Holds));
        };
        let routed = forge_route::step_core(domain, env, event);
        super::super::route_engine_requests(domain, env, decision, routed);
    }
    Ok(())
}

fn permission(domain: &Domain, project: u32, number: u32, by: Filer) -> Result<(), FindingRefusal> {
    match super::permitted(domain, project, number, by) {
        Ok(()) => Ok(()),
        Err(why) => Err(FindingRefusal::Words(why.words(project, number, b""))),
    }
}
fn words(parts: &[&[u8]]) -> FindingRefusal {
    FindingRefusal::Words(skein_lib::bytes::concat(parts))
}

fn refused(why: tasks::Refusal) -> FindingRefusal {
    FindingRefusal::Words(why.words())
}
