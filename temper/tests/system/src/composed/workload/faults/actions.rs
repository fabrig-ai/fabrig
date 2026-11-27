//! The composed party route and its moments (temper/docs/design/testing/workloads.md, 5).
use super::{Act, Faults, Struck, World};
use crate::composed::{Key, engine, people, tasks};
use temper_engine_domain_forge::LandingPhase;
use temper_engine_domain_world::acts;
use temper_faults::people::RELEASES_A_MEMBER;
use temper_workload::parties::Action;

impl World {
    pub(super) fn workload_actions_tick(&mut self, f: &mut Faults) -> Result<(), String> {
        for action in f.watch.due(&self.store, self.workload_names(), self.iteration) {
            if !f.actions.contains(&action) {
                continue;
            }
            let target = self.workload_names().task(&action.target).expect("drawn member");
            let live_landing = self.workload_landings().any(|row| {
                !matches!(row.landing.phase, LandingPhase::Landed | LandingPhase::Cancelled)
                    && self
                        .workload_live_task(row.landing.task)
                        .is_some_and(|task| !matches!(task.phase, tasks::Phase::Closing(_)))
                    && row
                        .landing
                        .members
                        .iter()
                        .any(|member| member.change == target && member.left.is_none() && member.landed.is_none())
            });
            if !live_landing
                || !self
                    .workload_live_task(target)
                    .is_some_and(|task| matches!(task.phase, tasks::Phase::Waiting | tasks::Phase::Active(_)))
            {
                f.skip_hold(&action);
                continue;
            }
            self.workload_act(
                f,
                &action,
                people::Ask::Stop { project: 1, task: target, reason: Box::from(b"Parking the task".as_slice()) },
            )?;
            return Ok(());
        }
        // A hold may never latch if the landing completes between observations.
        if let Some(hold) = f
            .actions
            .iter()
            .find(|action| {
                action.act == Act::HoldMember
                    && self.workload_names().task(&action.target).is_some_and(|task| self.workload_task_ended(task))
            })
            .cloned()
        {
            f.skip_hold(&hold);
        }
        let Some(action) = f.actions.iter().find(|action| action.act == Act::ReleaseMember).cloned() else {
            return Ok(());
        };
        if f.actions.iter().any(|action| action.act == Act::HoldMember) {
            return Ok(());
        }
        let target = self.workload_names().task(&action.target).expect("held member");
        let revision = self.workload_live_task(target).and_then(|row| {
            matches!(row.phase, tasks::Phase::Held { why: tasks::Hold::Placed { .. }, .. })
                .then_some(row.revisions.hold)
        });
        // Latch the hold itself, not membership in the former landing: a held
        // member may leave that landing before its release falls.
        if let Some(revision) = revision {
            f.release_from.get_or_insert((self.iteration, revision));
        }
        if let Some((first, held)) = f.release_from {
            if self.iteration.saturating_sub(first) < u64::from(action.after) {
                return Ok(());
            }
            if revision == Some(held) {
                self.workload_act(
                    f,
                    &action,
                    people::Ask::Release { project: 1, task: target, revision: held, again: None },
                )?;
                return Ok(());
            }
        } else if !self.workload_task_ended(target) {
            // The accepted stop's task write has not been seen yet.
            return Ok(());
        }
        f.actions.retain(|pending| pending != &action);
        if f.release_configured {
            f.ledger.ineligible(RELEASES_A_MEMBER);
        }
        Ok(())
    }

    fn workload_act(&mut self, f: &mut Faults, action: &Action, ask: people::Ask) -> Result<(), String> {
        eprintln!("act {action:?} at {:?}", self.env.now);
        let at = self.env.now;
        let sign_in = match action.party {
            temper_workload::draw::Party::First => self.party.sign_in,
            temper_workload::draw::Party::Second => self.workload.as_ref().expect("workload").other.sign_in,
        }
        .expect("drawn party signed in");
        let story = self.workload.as_mut().expect("workload");
        let mut key = [0; 16];
        key[..8].copy_from_slice(&story.next_reply.to_le_bytes());
        story.next_reply += 1;
        let reply = self.notes_act_as(sign_in, ask, key);
        if matches!(reply, people::Reply::Outcome(people::Outcome::Refused(_)) | people::Reply::Refused(_)) {
            return Err(format!("drawn act {action:?} refused: {reply:?}"));
        }
        if action.act == Act::ReleaseMember {
            f.released_member = Some((self.workload_names().task(&action.target).expect("released member"), at));
        }
        f.actions.retain(|pending| pending != action);
        if action.act != Act::ReleaseMember || f.release_configured {
            let injection = f.ledger.inject(
                acts::kind(action.act),
                at,
                Struck { request: Some(action.target.clone()), ..Struck::default() },
            );
            f.ledger.stop(injection, at).expect("an act is an instant");
        }
        f.stopped(at);
        Ok(())
    }

    pub(super) fn workload_live_task(&self, task: u64) -> Option<&tasks::TaskRecord> {
        let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)));
        match self.store.rows.get(&key) {
            Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => Some(row),
            _ => None,
        }
    }
}

impl Faults {
    fn skip_hold(&mut self, hold: &Action) {
        self.actions.retain(|action| action.party != hold.party || action.target != hold.target);
        self.ledger.ineligible(acts::kind(hold.act));
        if self.release_configured {
            self.ledger.ineligible(RELEASES_A_MEMBER);
        }
    }
}
