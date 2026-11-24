//! People's logical moments (`temper/docs/design/testing/workloads.md`, 4.1 and 6).
//! Only committed store rows are observed; the composed world uses this watch unchanged.
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_world::faults::Kind;
use temper_engine_domain::{Key, Record};
use temper_engine_domain_forge as forge;
use temper_workload::{
    names::Names,
    parties::{Act, Action, When},
};

use crate::store::Store;

#[must_use]
pub fn kind(act: Act) -> Kind {
    use temper_faults::people;
    match act {
        Act::CancelGoal => people::CANCELS_A_GOAL,
        Act::CancelChange => people::CANCELS_A_CHANGE,
        Act::CancelLanding => people::CANCELS_A_LANDING,
        Act::HoldMember => people::HOLDS_A_MEMBER,
        Act::ReleaseMember => people::RELEASES_A_MEMBER,
        Act::AmendPassedDeadline => jig_core_kit::open::AMENDS_A_PASSED_DEADLINE,
        Act::AnswerLate => people::ANSWERS_LATE,
        Act::NeverAnswer => people::NEVER_ANSWERS,
        Act::AnswerTwice => people::ANSWERS_TWICE,
        Act::AnswerEndedRun => people::ANSWERS_AN_ENDED_RUN,
        Act::WordsToEndedProducer => people::WORDS_TO_AN_ENDED_PRODUCER,
    }
}

struct Pending {
    action: Action,
    first: Option<u64>,
    fired: bool,
}

pub struct Watch {
    pending: Vec<Pending>,
}

impl Watch {
    #[must_use]
    pub fn new(actions: Vec<Action>) -> Self {
        Self { pending: actions.into_iter().map(|action| Pending { action, first: None, fired: false }).collect() }
    }

    pub(crate) fn delaying(&self) -> bool {
        self.pending.iter().any(|pending| pending.first.is_some() && !pending.fired)
    }

    /// A moment is latched when first observed, even if the task advances before `after` expires.
    pub fn due(&mut self, store: &Store, names: &Names, step: u64) -> Vec<Action> {
        let mut due = Vec::new();
        for pending in &mut self.pending {
            if pending.fired {
                continue;
            }
            let Some(target) = names.task(&pending.action.target) else { continue };
            if pending.first.is_none() && holds(store, target, pending.action.when) {
                pending.first = Some(step);
            }
            if pending.first.is_some_and(|first| step.saturating_sub(first) >= u64::from(pending.action.after)) {
                pending.fired = true;
                due.push(pending.action.clone());
            }
        }
        due
    }
}

fn holds(store: &Store, target: u64, when: When) -> bool {
    match when {
        When::Started | When::Opened => task(store, target).is_some(),
        When::Asked => producers(store, target).any(|row| !row.questions.is_empty()),
        When::Landing => landing(store, target).is_some(),
        When::Held => {
            landing(store, target).is_some()
                && task(store, target).is_some_and(|row| matches!(row.phase, tasks::Phase::Held { .. }))
        }
        When::DeadlinePassed => task(store, target)
            .is_some_and(|row| matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Deadline, .. })),
        When::ProducerEnded => producers(store, target).any(|row| matches!(row.phase, tasks::Phase::Ended(_))),
    }
}

/// The cancellation exclusion observes only this goal's branch-making read or write.
#[must_use]
pub fn goal_branch_making(world: &crate::forge_routes::World, goal: u64) -> bool {
    world.root.forge().inspection_making_read_waiters().contains(&goal)
        || world.store.rows.values().any(|record| {
            let Record::Forge { row } = record else { return false };
            matches!(row.as_ref(), forge::Stored::Entry { entry, held: false }
                if entry.task == goal && matches!(entry.effect.write, skein_forge::api::Write::CreateBranch { .. }))
        })
}

pub(crate) fn task(store: &Store, number: u64) -> Option<&tasks::TaskRecord> {
    [tasks::Key::Live(number), tasks::Key::Ended(number)].into_iter().find_map(|key| {
        match store.rows.get(&Key::Core(core::Key::Tasks(key))) {
            Some(Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) => {
                Some(row.as_ref())
            }
            _ => None,
        }
    })
}

pub(crate) fn producers(store: &Store, change: u64) -> impl Iterator<Item = &tasks::TaskRecord> {
    // The key ranges retain the store's live-then-ended order without walking
    // every activity, attempt and connector record between observations.
    let key = |key| temper_engine_domain::Key::Core(core::Key::Tasks(key));
    store
        .rows
        .range(key(tasks::Key::Live(0))..=key(tasks::Key::Live(u64::MAX)))
        .chain(store.rows.range(key(tasks::Key::Ended(0))..=key(tasks::Key::Ended(u64::MAX))))
        .filter_map(move |(_, record)| match record {
            Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.requester == tasks::Party::Task(change)
                    && matches!(row.executor, tasks::Executor::Agent { .. })
                    && matches!(row.contract, tasks::Contract::Change { .. }) =>
            {
                Some(row.as_ref())
            }
            _ => None,
        })
}

pub(crate) fn landing(store: &Store, change: u64) -> Option<&forge::LandingRow> {
    store
        .rows
        .range(
            temper_engine_domain::Key::Forge(forge::Key::Landing(0))
                ..=temper_engine_domain::Key::Forge(forge::Key::Landing(u64::MAX)),
        )
        .find_map(|(_, record)| match record {
            Record::Forge { row } => match row.as_ref() {
                forge::Stored::Landing(row)
                    if row.landing.members.iter().any(|member| member.change == change)
                        && task(store, row.landing.task)
                            .is_some_and(|row| !matches!(row.phase, tasks::Phase::Ended(_))) =>
                {
                    Some(row)
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
}
