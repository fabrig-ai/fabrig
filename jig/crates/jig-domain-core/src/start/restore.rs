//! Store rows enter through the start that owns their child routing (jig/docs/design/domain/engine.md, 6).
use super::Phase;
use crate::{Core, Limits, Queue, Record, Restored, accounts, people, tasks};
use skein_lib::Env;

/// The information a root needs after routing one restored row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Restore {
    Kept,
    Refused,
    Deployment {
        commits: u64,
    },
    /// A people row queued for the root's next sweep decision.
    Swept,
}

impl Core {
    /// Restore a start row and step its owner before the next row arrives.
    pub fn restore(&mut self, env: &Env<Limits>, record: Record) -> Restore {
        self.start_input();
        if self.start_failure().is_some() {
            return Restore::Refused;
        }
        let kept = match self.start_phase() {
            Some(Phase::LoadInbox) => self.restore_inbox_row(&env.limits, &record),
            Some(Phase::Sweep) => {
                let Record::People(row) = record else { unreachable!("people sweep range") };
                self.start.sweep.push(row);
                return Restore::Swept;
            }
            Some(Phase::LoadCore | Phase::RestoreConnector { .. }) => match record {
                Record::Core(row) => {
                    return match self.restore_core(row, &env.limits) {
                        Restored::Deployment { commits } => Restore::Deployment { commits },
                        Restored::Live | Restored::Archive => Restore::Kept,
                        Restored::Rejected => Restore::Refused,
                    };
                }
                Record::Tasks(row) => {
                    if !self.restore_task_row(&row) {
                        return Restore::Refused;
                    }
                    let mut out = Queue::uncapped();
                    tasks::step(
                        &mut self.tasks,
                        &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
                        tasks::Event::Restore { record: row },
                        &mut out,
                    );
                    out.is_empty()
                }
                Record::People(people::Stored::Policy { project, name, value }) => {
                    self.restore_policy(&env.limits, project, name, value)
                }
                Record::People(row) => {
                    let mut out = Queue::uncapped();
                    people::step(
                        &mut self.people,
                        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
                        people::Event::Restore { record: row },
                        &mut out,
                    );
                    out.is_empty()
                }
                Record::Notes(_) => true,
                Record::Accounts(record) => {
                    let mut out = Queue::uncapped();
                    accounts::step(
                        &mut self.accounts,
                        &Env { now: env.now, wall: env.wall, limits: env.limits.accounts },
                        accounts::Event::Restore { record },
                        &mut out,
                    );
                    out.is_empty()
                }
            },
            Some(
                Phase::Link
                | Phase::Switch
                | Phase::Wires
                | Phase::AdoptRuns
                | Phase::ReadAfresh { .. }
                | Phase::SettleOutbox { .. }
                | Phase::Open,
            )
            | None => unreachable!("restore rows belong to the start's load phases"),
        };
        self.start_collect_failure();
        if kept && self.start_failure().is_none() { Restore::Kept } else { Restore::Refused }
    }
}
