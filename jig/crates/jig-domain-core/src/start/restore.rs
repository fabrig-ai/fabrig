//! Store rows enter through the start that owns their child routing (jig/docs/design/domain/engine/restart.md).
pub use jig_domain_core_vocabulary::start::restore::*;

use crate::{Core, Limits, Queue, Record, Restored, accounts, people, start::Phase, tasks};
use skein_lib::Env;

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
                    crate::routing::step::step_tasks(self, env, tasks::Event::Restore { record: row }, &mut out);
                    out.is_empty()
                }
                Record::People(people::Stored::Policy { project, name, value }) => {
                    self.restore_policy(&env.limits, project, name, value)
                }
                Record::People(people::Stored::Public { project }) => {
                    // Public keys follow policy keys in the people restore walk (protocol/store/README.md, 4.4).
                    if project == 0
                        || self.authority.policy(project).is_none()
                        || self.visibility.public.insert(project, ()).is_err()
                    {
                        self.restore_failure = Some(crate::start::Failure::ProjectPolicy { project });
                        false
                    } else {
                        crate::public::restored(self, project);
                        true
                    }
                }
                Record::People(row) => {
                    let mut out = Queue::uncapped();
                    crate::routing::step::step_people(self, env, people::Event::Restore { record: row }, &mut out);
                    out.is_empty()
                }
                Record::Notes(_) => true,
                Record::Accounts(record) => {
                    let mut out = Queue::uncapped();
                    crate::routing::step::step_accounts(self, env, accounts::Event::Restore { record }, &mut out);
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
