//! Joins the trials child to the core; holds each running row's last wait (temper/docs/design/domain/trial.md, 4 and 5).
use crate::{Domain, Limits, TryAnswer, boundary::Work};
use alloc::boxed::Box;
use jig_domain_accounts::{Holder, State as AccountState};
use jig_domain_core as core;
use jig_domain_core::documents::{TaskRunWait, TaskSourceState, TaskWaitWhy};
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
    let Some(row) = domain.engine.core().accounts.account(account) else { return TryAnswer::NotTheirs };
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
                let llm = match choice(domain.engine.core(), person, account) {
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
                let Some(seed) = domain.engine.core_mut().person_seed(person) else {
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
    let Some(trial) = domain.trials.running_row(row.task) else { return };
    let event = match row.line {
        tasks::Line::Turn { .. } => trials::Event::Turn { task: row.task },
        tasks::Line::RunEnded { end, .. } => {
            let core = domain.engine.core();
            let took = match task_row(core, row.task) {
                Some(task) => task.kept_account,
                None => None,
            };
            let state = if end == tasks::EndKind::Refused {
                core.accounts.account_state(took.unwrap_or(trial.account))
            } else {
                None
            };
            let Some(event) = run_ended(row.task, end, state) else { return };
            if let trials::Event::RunLimited { .. } = event
                && trial.took.is_none()
                && took.is_some()
            {
                route(domain, env, trials::Event::Observed { task: row.task, past_starting: true, took });
            }
            event
        }
        tasks::Line::Words { .. }
        | tasks::Line::Heard(_)
        | tasks::Line::Notified { .. }
        | tasks::Line::RunBegan { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Milestone { .. }
        | tasks::Line::NowWritten { .. }
        | tasks::Line::Stood { .. }
        | tasks::Line::TalkOpened { .. }
        | tasks::Line::TalkEnded { .. }
        | tasks::Line::SessionEnded { .. }
        | tasks::Line::Kept { .. } => return,
    };
    route(domain, env, event);
}

fn task_row(core: &core::Core, task: u64) -> Option<&tasks::TaskRecord> {
    core.tasks.task(task)
}

fn run_ended(task: u64, end: tasks::EndKind, state: Option<AccountState>) -> Option<trials::Event> {
    Some(match end {
        tasks::EndKind::Finished => trials::Event::Finished { task },
        tasks::EndKind::Failed { class: why, .. } => trials::Event::RunFailed { task, class: class(why) },
        tasks::EndKind::Refused => match state {
            Some(AccountState::Limited { until }) => trials::Event::RunLimited {
                task,
                reset: match until {
                    Some(until) => Some(until.wall),
                    None => None,
                },
            },
            Some(
                AccountState::Asking
                | AccountState::Granted
                | AccountState::Later { .. }
                | AccountState::Refused
                | AccountState::NeedsSignIn
                | AccountState::Misconfigured
                | AccountState::Unloaded { .. }
                | AccountState::Cancelled
                | AccountState::Closing
                | AccountState::Moving,
            )
            | None => trials::Event::RunFailed { task, class: trials::Class::Refused },
        },
        tasks::EndKind::Parked | tasks::EndKind::Cancelled | tasks::EndKind::Suspended { .. } => return None,
    })
}

fn wait_limited(task: u64, wait: Option<TaskRunWait>) -> Option<trials::Event> {
    if let Some(TaskRunWait::Account { why: TaskWaitWhy::Source(TaskSourceState::Limited { until }), .. }) = wait {
        Some(trials::Event::RunLimited { task, reset: until })
    } else {
        None
    }
}

fn class(class: tasks::Class) -> trials::Class {
    match class {
        tasks::Class::Transient => trials::Class::Transient,
        tasks::Class::Permanent => trials::Class::Permanent,
        tasks::Class::Run => trials::Class::Run,
        tasks::Class::Agent => trials::Class::Agent,
        tasks::Class::Lost => trials::Class::Lost,
        tasks::Class::Invalid => trials::Class::Invalid,
        tasks::Class::Infrastructure => trials::Class::Infrastructure,
    }
}

pub(crate) fn observe(domain: &mut Domain, env: &Env<Limits>) {
    let mut moved = Queue::uncapped();
    let mut waits = skein_lib::Map::with_capacity(crate::limits::trial_limits(&env.limits).rows);
    let mut changed = false;
    for task in domain.trials.running() {
        let past_starting = domain.engine.core().run_past_starting(task);
        let Some(held) = task_row(domain.engine.core(), task) else { continue };
        let wait = core::documents::run_wait(domain.engine.core(), held);
        let limited = wait_limited(task, wait);
        if limited.is_none() && domain.trial_waits.get(&task).copied().flatten() != wait {
            changed = true;
        }
        let took = held.kept_account;
        let row = domain.trials.running_row(task).expect("observing a running row");
        if (row.step == trials::Step::Starts && past_starting) || (row.took.is_none() && took.is_some()) {
            moved.push(trials::Event::Observed { task, past_starting, took });
        }
        if let Some(event) = limited {
            moved.push(event);
        } else {
            waits.insert(task, wait).expect("one wait per running trial");
        }
    }
    domain.trial_waits = waits;
    if changed {
        let mut subjects = List::with_capacity(1);
        subjects.push(Token::new(trials::TRIALS_SUBJECT)).expect("one trials subject");
        domain.work.push(Work::Core(core::Event::Marked { connector: domain.config.forge_connector, subjects }));
    }
    for _ in 0..moved.len() {
        route(domain, env, moved.pop().expect("observation count"));
    }
}

pub(crate) fn restarted(domain: &mut Domain) {
    let mut tasks = Queue::uncapped();
    for candidate in domain.engine.core().tasks.view_tasks() {
        let Some(row) = task_row(domain.engine.core(), candidate.number) else { continue };
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
    use super::*;
    use jig_domain_accounts::Deadline;
    use skein_lib::{Time, Wall};

    #[test]
    fn a_refused_run_is_limited_only_when_the_account_is_held_by_its_limit() {
        let task = 7;
        let wall = Wall::from_nanos(300);
        let at = Time::from_nanos(14);
        for until in [None, Some(Deadline { at, wall })] {
            let state = Some(AccountState::Limited { until });
            let reset = match until {
                Some(until) => Some(until.wall),
                None => None,
            };
            assert_eq!(
                run_ended(task, tasks::EndKind::Refused, state),
                Some(trials::Event::RunLimited { task, reset })
            );
            let end = tasks::EndKind::Failed { class: tasks::Class::Run, invalid_result: None };
            assert_eq!(run_ended(task, end, state), Some(trials::Event::RunFailed { task, class: trials::Class::Run }));
        }
        for state in [None, Some(AccountState::Granted), Some(AccountState::Later { until: Deadline { at, wall } })] {
            assert_eq!(
                run_ended(task, tasks::EndKind::Refused, state),
                Some(trials::Event::RunFailed { task, class: trials::Class::Refused }),
            );
        }
    }

    #[test]
    fn a_wait_on_a_limited_source_ends_the_trial_without_waiting_for_reset() {
        let task = 7;
        for until in [None, Some(Wall::from_nanos(300))] {
            let wait = |state| {
                Some(TaskRunWait::Account { account: Some(11), source: Some(12), why: TaskWaitWhy::Source(state) })
            };
            assert_eq!(
                wait_limited(task, wait(TaskSourceState::Limited { until })),
                Some(trials::Event::RunLimited { task, reset: until }),
            );
            assert_eq!(wait_limited(task, wait(TaskSourceState::Later { until })), None);
        }
        assert_eq!(wait_limited(task, Some(TaskRunWait::Credential)), None);
        assert_eq!(wait_limited(task, None), None);
    }

    #[test]
    fn rows_fit_the_accounts_wire_and_delivery_room() {
        let mut limits = crate::tests::resume_fixture::limits();
        limits.accounts.accounts = 100;
        limits.journal.result_bytes = 0;
        let row = u32::try_from(size_of::<crate::TrialRow>()).unwrap();
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
