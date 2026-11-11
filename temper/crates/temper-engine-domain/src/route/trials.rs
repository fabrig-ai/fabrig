//! Joins the trials child to the core; holds nothing (temper/docs/design/domain/trial.md, 4 and 5).
use crate::{Domain, Limits, TryAnswer, boundary::Work};
use alloc::boxed::Box;
use jig_domain_accounts::Holder;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};
use temper_engine_domain_trials as trials;

pub(crate) const TRIAL_CHARTER: u32 = 10;

enum Choice {
    First,
    Held(tasks::LlmChoice),
    Unserved,
}

/// The trial's choice (temper/docs/design/domain/trial.md, 2): none for the person's first usable
/// account; for a later one, `Core::account_llm` for it, or refused where it serves no model.
fn choice(core: &core::Core, person: u64, account: u32) -> Choice {
    // A person seed takes its models from the deployment ceiling without narrowing them.
    let ceiling = core.authority.rules().ceiling.models;
    let models = tasks::Models::from_parts(ceiling.bits(), ceiling.is_all());
    let payer = tasks::Payer::Party(person);
    for offered in &core.offer(payer, people::DEPLOYMENT, models) {
        for other in &offered.accounts {
            if *other != account {
                return match core.account_llm(TRIAL_CHARTER, models, payer, people::DEPLOYMENT, account) {
                    Some(llm) => Choice::Held(llm),
                    None => Choice::Unserved,
                };
            }
        }
    }
    Choice::First
}

pub(crate) fn asked(domain: &mut Domain, env: &Env<Limits>, person: u64, account: u32, again: bool) -> TryAnswer {
    let Some(row) = domain.core.accounts.account(account) else { return TryAnswer::NotTheirs };
    if row.holder() != Holder::Party(person) {
        return TryAnswer::NotTheirs;
    }
    route(domain, env, trials::Event::Try { account, person, again });
    TryAnswer::Tried
}

pub(crate) fn route(domain: &mut Domain, env: &Env<Limits>, event: trials::Event) {
    let mut out = Queue::uncapped();
    trials::step(&mut domain.trials, &env.with(crate::limits::trial_limits(&env.limits)), event, &mut out);
    // A Make can answer Refused once; all other requests terminate here.
    while let Some(request) = out.pop() {
        match request {
            trials::Request::Make { account, person } => {
                let llm = match choice(&domain.core, person, account) {
                    Choice::First => None,
                    Choice::Held(llm) => Some(llm),
                    Choice::Unserved => {
                        trials::step(
                            &mut domain.trials,
                            &env.with(crate::limits::trial_limits(&env.limits)),
                            trials::Event::Refused { account },
                            &mut out,
                        );
                        continue;
                    }
                };
                let Some(seed) = domain.core.person_seed(person) else {
                    trials::step(
                        &mut domain.trials,
                        &env.with(crate::limits::trial_limits(&env.limits)),
                        trials::Event::Refused { account },
                        &mut out,
                    );
                    continue;
                };
                let task = seed.number();
                domain.work.push(Work::Core(core::Event::PersonTask {
                    seed: Box::new(seed),
                    executor: tasks::Executor::Agent { charter: TRIAL_CHARTER },
                    llm,
                    spec: tasks::Spec {
                        job: None,
                        asks: false,
                        words: Box::from(b"Say one word.".as_slice()),
                        parameters: Box::new([]),
                        inputs: Box::new([]),
                    },
                    contract: tasks::Contract::Report { words: 64 },
                }));
                trials::step(
                    &mut domain.trials,
                    &env.with(crate::limits::trial_limits(&env.limits)),
                    trials::Event::Made { account, task },
                    &mut out,
                );
            }
            trials::Request::Cancel { task } => cancel(domain, task),
            trials::Request::Changed { person: _ } => {
                let mut subjects = List::with_capacity(1);
                subjects.push(Token::new(trials::TRIALS_SUBJECT)).expect("one trials subject");
                domain
                    .work
                    .push(Work::Core(core::Event::Marked { connector: domain.config.forge_connector, subjects }));
            }
        }
    }
}

pub(crate) fn saved(domain: &mut Domain, env: &Env<Limits>, record: &core::Record) {
    let core::Record::Tasks(tasks::Stored::Activity(row)) = record else { return };
    if !domain.trials.holds(row.task) {
        return;
    }
    let event = match row.line {
        tasks::Line::Turn { .. } => trials::Event::Turn { task: row.task },
        tasks::Line::RunEnded { end, .. } => match end {
            tasks::EndKind::Finished => trials::Event::Finished { task: row.task },
            tasks::EndKind::Failed { class: why, .. } => trials::Event::RunFailed { task: row.task, class: class(why) },
            tasks::EndKind::Refused => trials::Event::RunFailed { task: row.task, class: trials::Class::Refused },
            tasks::EndKind::Parked | tasks::EndKind::Cancelled => return,
        },
        tasks::Line::Words { .. }
        | tasks::Line::Heard(_)
        | tasks::Line::Notified { .. }
        | tasks::Line::RunBegan { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Milestone { .. }
        | tasks::Line::Stood { .. }
        | tasks::Line::SessionEnded { .. }
        | tasks::Line::Kept { .. } => return,
    };
    route(domain, env, event);
}

fn class(class: tasks::Class) -> trials::Class {
    match class {
        tasks::Class::Transient => trials::Class::Transient,
        tasks::Class::Permanent => trials::Class::Permanent,
        tasks::Class::Run => trials::Class::Run,
        tasks::Class::Agent => trials::Class::Agent,
        tasks::Class::Lost => trials::Class::Lost,
        tasks::Class::Invalid => trials::Class::Invalid,
    }
}

pub(crate) fn observe(domain: &mut Domain, env: &Env<Limits>) {
    let mut moved = Queue::uncapped();
    for task in domain.trials.running() {
        let past_starting = domain.core.run_past_starting(task);
        let Some(held) = domain.core.tasks.task(task) else { continue };
        let person = match held.funder {
            tasks::Funder::Pool { person, .. } => person,
            tasks::Funder::Period { .. } | tasks::Funder::Task(_) | tasks::Funder::Recurring { .. } => continue,
        };
        let took = held.kept_account;
        // Running rows alone participate, and each field moves only forward.
        for row in domain.trials.rows(person) {
            if row.task == Some(task)
                && ((row.step == trials::Step::Starts && past_starting) || (row.took.is_none() && took.is_some()))
            {
                moved.push(trials::Event::Observed { task, past_starting, took });
            }
        }
    }
    for _ in 0..moved.len() {
        route(domain, env, moved.pop().expect("observation count"));
    }
}

pub(crate) fn restarted(domain: &mut Domain) {
    let mut tasks = Queue::uncapped();
    for candidate in domain.core.tasks.view_tasks() {
        let Some(row) = domain.core.tasks.task(candidate.number) else { continue };
        if row.executor == (tasks::Executor::Agent { charter: TRIAL_CHARTER }) {
            tasks.push(row.number);
        }
    }
    for _ in 0..tasks.len() {
        cancel(domain, tasks.pop().expect("restored trial count"));
    }
}

fn cancel(domain: &mut Domain, task: u64) {
    domain.work.push(Work::Tasks(tasks::Event::Cancel {
        reply_to: super::internal(u64::MAX),
        by: tasks::Party::Deployment { project: 0 },
        operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
    }));
}

#[cfg(test)]
mod tests {
    #[test]
    fn rows_fit_the_accounts_wire_and_delivery_room() {
        let mut limits = crate::tests::resume_fixture::limits();
        limits.accounts.accounts = 100;
        limits.journal.result_bytes = 0;
        let row = u32::try_from(size_of::<temper_engine_domain_trials::Row>()).unwrap();
        limits.journal.transcript_bytes = row * 2 - 1;
        assert_eq!(crate::limits::trial_limits(&limits).rows, 1);
        limits.journal.transcript_bytes = row * 2;
        assert_eq!(crate::limits::trial_limits(&limits).rows, 2);
        limits.journal.transcript_bytes = row * 100;
        assert_eq!(crate::limits::trial_limits(&limits).rows, 64);
        limits.accounts.accounts = 3;
        assert_eq!(crate::limits::trial_limits(&limits).rows, 3);
    }
}
