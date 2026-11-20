//! Start's repairs in Link (temper/docs/design/domain/landing-forming.md, 4).
use crate::{Domain, Key, Limits, Request, Stored, Writer};
use skein_lib::{Env, List, Queue};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;

impl Domain {
    /// End unmade landings and drop taskless holds that no live task writes under.
    pub fn link_landings(&mut self, tasks: &jig_domain_tasks::Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
        if self.restore_fit_failure().is_some() {
            return;
        }
        self.join_changes();
        let mut unmade = List::with_capacity(self.landing.rows.len());
        for (task, row) in &self.landing.rows {
            let live = match row.landing.phase {
                landing::Phase::Landed | landing::Phase::Cancelled => false,
                landing::Phase::Waiting
                | landing::Phase::Preparing
                | landing::Phase::Proving
                | landing::Phase::Proved { .. }
                | landing::Phase::Turn { .. }
                | landing::Phase::Held { .. } => true,
            };
            if live && row.entries.is_empty() && tasks.task(*task).is_none() {
                unmade.push(*task).expect("bounded restored landings");
            }
        }
        for task in unmade.into_boxed() {
            self.end_unmade(task, env, out);
        }
        let mut abandoned = List::with_capacity(self.holds.len());
        for (name, hold) in &self.holds {
            if tasks.task(hold.task).is_some() || self.forge_ended(name.forge) {
                continue;
            }
            let mut written = match hold.writer {
                Some(Writer::Run { task, .. } | Writer::Lost { task, .. }) => tasks.task(task).is_some(),
                Some(Writer::Entry(_)) | None => false,
            };
            for writer in &hold.design_writers {
                written |= tasks.task(writer.task).is_some();
            }
            if !written {
                abandoned.push(name.clone()).expect("bounded restored holds");
            }
        }
        for name in abandoned.into_boxed() {
            let hold = self.holds.remove(&name).expect("selected taskless hold");
            let task = hold.task;
            self.reconciliation.forget(&Stored::Hold(hold), &env.limits);
            out.push(Request::Erase { key: Key::Hold(name.clone()) });
            out.push(Request::HoldDropped { name, task });
        }
    }

    fn end_unmade(&mut self, task: u64, env: &Env<Limits>, out: &mut Queue<Request>) {
        let row = self.landing.rows.remove(&task).expect("selected unmade landing");
        self.landing.due.remove(&task);
        crate::room::leave(self, crate::room::Waiter::Landing(task));
        let mut members = List::with_capacity(env.limits.landing_members);
        for member in &row.landing.members {
            members.push(member.change).expect("bounded landing members");
            if let Some(change) = self.changes.get_mut(&member.change)
                && change.change.landing == Some(task)
            {
                change.change.landing = None;
                if let Some(merge) = member.landed {
                    if crate::domain::landed_commit(&change.change.state).is_none() {
                        change.change.state = change::State::Landed { merge, since: env.wall };
                    }
                } else if let change::State::Judging { head, ready_since, .. } = change.change.state {
                    change.change.state = change::State::Ready { head, ready_since };
                }
                let changed = change.clone();
                crate::domain::save_change_parts(self, out, changed);
            }
        }
        let repository = match row.landing.members.first() {
            Some(member) => Some(skein_forge::api::Repository {
                forge: member.repository.forge,
                repository: member.repository.repository,
            }),
            None => None,
        };
        self.reconciliation.forget(&Stored::Landing(row.clone()), &env.limits);
        out.push(Request::Erase { key: Key::Landing(task) });
        if let Some(repository) = repository {
            out.push(Request::LandingUnmade {
                task,
                repository,
                branch: row.landing.branch,
                changes: members.into_boxed(),
            });
        }
    }
}
