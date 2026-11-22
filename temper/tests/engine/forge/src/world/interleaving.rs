//! Real forge actions between connector calls (testing/fakes.md, 2).
use super::{World, forge, raw, top};
use skein_lib::{Env, Time};
use skein_world::{faults::Kind, sweep::Exclusion};
use std::collections::BTreeMap;

pub(super) struct Lagging {
    lag: forge::Lag,
    fault: Kind,
    scenario: &'static str,
    active: BTreeMap<u64, Time>,
}

pub(super) struct AfterWrite {
    op: raw::Op,
    kind: Kind,
    then: raw::Op,
    user: u64,
}

impl World {
    /// Exclude only a manual record of a squash already present on the base.
    /// A normal fast-forward's late merge and all head/base views remain enabled.
    pub fn exclude_lagging_records(&mut self, lag: forge::Lag, exclusion: &Exclusion) {
        assert_eq!(lag.chance, 1000, "each eligible write draws this sweep's lag");
        self.lagging =
            Some(Lagging { lag, fault: exclusion.fault, scenario: exclusion.scenario, active: BTreeMap::new() });
    }

    pub(super) fn track_lag(&mut self, fell: &forge::Fell) {
        if let Some(lagging) = &mut self.lagging
            && fell.fault.family() == "lagging"
            && let (Some(pull), Some(until)) = (fell.pull, fell.until)
        {
            lagging.active.insert(pull, until);
        }
    }

    pub(super) fn exclude_record_lag(&mut self, event: &forge::Event) -> Option<forge::Lag> {
        let forge::Event::Call {
            op: raw::Op::Write(raw::Write::Merge { number, how: raw::MergeHow::Recorded { commit }, .. }),
            ..
        } = event
        else {
            return None;
        };
        let lagging = self.lagging.as_ref()?;
        let landed_squash = self.stored.iter().any(|(_, record)| {
            let top::Stored::Landing(row) = record else { return false };
            row.landing.members.iter().any(|member| {
                member.pull == *number
                    && member.combined == Some(temper_engine_domain_forge_landing::Combine::Squash)
                    && member.landed == Some(crate::translate::commit(*commit))
                    && !member.recorded
            })
        });
        if !landed_squash || lagging.active.get(number).is_some_and(|until| self.env.now < *until) {
            return None;
        }
        let lag = lagging.lag;
        self.ledger.leave_out(lagging.scenario, lagging.fault, self.env.now);
        forge::lag(&mut self.forge, forge::Lag { chance: 0, ..lag });
        Some(lag)
    }

    /// Apply a legitimate external action only after the named engine write succeeds.
    pub fn after_write(&mut self, op: raw::Op, kind: Kind, then: raw::Op, user: u64) {
        assert!(self.after_write.is_none(), "one pending interleaving");
        self.after_write = Some(AfterWrite { op, kind, then, user });
    }

    pub(super) fn interleave_after(&mut self, op: &skein_forge::api::Op, succeeded: bool) {
        if !succeeded
            || !self
                .after_write
                .as_ref()
                .is_some_and(|after| after.op == crate::translate::op(op, &self.env.limits.client))
        {
            return;
        }
        let after = self.after_write.take().expect("matched interleaving");
        let number = match &after.then {
            raw::Op::Write(raw::Write::Reopen { number }) => *number,
            _ => panic!("this interleaving reopens an engine-closed pull"),
        };
        let injection = self.inject(
            after.kind,
            skein_world::faults::Struck { request: Some(number.to_string()), ..Default::default() },
        );
        let pending_outputs = std::mem::replace(&mut self.forge_out, skein_lib::Queue::uncapped());
        assert_eq!(self.external_now(after.then, after.user), raw::Answer::Done);
        self.forge_out = pending_outputs;
        self.stop(injection);
    }

    /// External calls at the world's current clock, so their lagging views last
    /// into subsequent engine reads rather than expiring at the fixture's epoch.
    pub fn external_now(&mut self, op: raw::Op, user: u64) -> raw::Answer {
        self.external_repository_at(b"org/repo", op, user, self.env.now, self.env.wall)
    }

    pub fn push_now_as(&mut self, branch: &[u8], path: &[u8], content: &[u8], user: u64) {
        let env = Env { now: self.env.now, wall: self.env.wall, limits: self.forge_env.limits.clone() };
        forge::advance(&mut self.forge, &env, b"org/repo", branch, path, content, user).expect("external push");
        self.copy_fell();
    }
}
