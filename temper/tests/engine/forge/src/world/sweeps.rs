//! Finite forge faults and other users' writes (testing/fakes.md, 2).
use super::{World, forge, raw, top};
use skein_lib::{Duration, Time};
use skein_world::faults::{Injection, Kind, Struck};
use temper_engine_domain_forge_landing::{Hold, Phase};
use temper_faults::forge::BUSY;

#[derive(PartialEq, Eq)]
enum Restored {
    ClosePermission(u64),
    Reopened(u64),
}

#[derive(Default)]
pub(super) struct Faults {
    calls: Option<(Time, forge::Config)>,
    busy_read_fell: bool,
    merge: Option<(Time, forge::Config)>,
    fell: Vec<(Kind, forge::Called)>,
    refusal: Option<Duration>,
    refused: Option<(u64, Time)>,
    closed: Option<(u64, Time, Injection)>,
    close_span: Option<Duration>,
    restored: Vec<Restored>,
    released: u32,
    others: Vec<(u64, Kind, bool)>,
    excluded_merge: Option<(Kind, &'static str)>,
    open_change: bool,
    changed: Vec<u64>,
    other_merges: Vec<(u64, u64, u64)>,
    step_seconds: Option<u64>,
    crowd: Option<Duration>,
    neighbours: Vec<skein_lib::Token>,
}

impl World {
    #[must_use]
    pub fn forge_config(&self) -> &forge::Config {
        &self.forge_env.limits
    }

    /// Fill the fake's small call table with other users' reads before the
    /// next landing read. These calls release their slots after the named span.
    pub fn crowd_next_read_for(&mut self, span: Duration) {
        self.sweep_faults.crowd = Some(span);
    }

    pub(super) fn neighbour_reply(&mut self, token: skein_lib::Token) -> bool {
        if let Some(index) = self.sweep_faults.neighbours.iter().position(|pending| *pending == token) {
            self.sweep_faults.neighbours.remove(index);
            true
        } else {
            false
        }
    }

    /// The simulated horizon of one workload step, not the test's wall budget.
    pub fn landing_step_horizon(&mut self, seconds: u64) {
        self.sweep_faults.step_seconds = Some(seconds);
    }

    pub(crate) fn landing_step_seconds(&self) -> u64 {
        self.sweep_faults.step_seconds.unwrap_or(20)
    }

    /// Change only connector calls after a landing exists, until this clock;
    /// fixture writes stay reliable.
    pub fn fault_calls_until(&mut self, until: Time, config: forge::Config) {
        self.sweep_faults.calls = Some((until, config));
    }

    /// Override the first merge before this clock, then return to ordinary calls.
    pub fn fault_next_merge_until(&mut self, until: Time, config: forge::Config) {
        self.sweep_faults.merge = Some((until, config));
    }

    #[must_use]
    pub fn fell_on(&self, kind: Kind, call: forge::Called) -> bool {
        self.sweep_faults.fell.contains(&(kind, call))
    }

    /// Refuse the next close by policy, restoring permission after this span.
    pub fn refuse_next_close_for(&mut self, span: Duration) {
        self.sweep_faults.refusal = Some(span);
    }

    /// Act on distinct members after their open pull has been read. False closes,
    /// true merges; the queued reply remains the connector's preceding snapshot.
    pub fn other_users_after_member_reads(&mut self, actions: Vec<(u64, Kind, bool)>, closed_for: Duration) {
        self.sweep_faults.others = actions;
        self.sweep_faults.close_span = Some(closed_for);
    }

    /// Draw the other user's merge on an open, unqueued change's next read.
    pub fn merge_open_change_after_read(&mut self, task: u64, kind: Kind) {
        self.sweep_faults.others = vec![(task, kind, true)];
        self.sweep_faults.open_change = true;
    }

    /// Leave out a merge from readiness for a batch through its member record.
    pub fn exclude_batch_member_merges(&mut self, exclusion: &skein_world::sweep::Exclusion) {
        self.sweep_faults.excluded_merge = Some((exclusion.fault, exclusion.scenario));
    }

    /// Script a person's retry after a busy read, a member's pull reopens,
    /// or permission to close the round's pull is restored. Only matching
    /// scripted faults merit a retry; other holds remain findings of the workload.
    pub fn release_restored_fault(&mut self, task: u64) -> bool {
        // Each landing step must encounter a fresh busy read fault to merit a retry.
        let busy_read_fell = std::mem::take(&mut self.sweep_faults.busy_read_fell);
        let row = self.landing(task);
        if busy_read_fell && row.room.is_none() && matches!(row.landing.phase, Phase::Held { why: Hold::Read, .. }) {
            return true;
        }
        let fault =
            match row.landing.phase {
                Phase::Held { why: Hold::Record { member }, .. } => {
                    Restored::Reopened(row.landing.members[usize::from(member)].pull)
                }
                Phase::Held { why: Hold::Cleanup, .. }
                    if row.landing.round.cleanup.as_deref().is_some_and(|account| {
                        account.starts_with(b"close item: forge protection refused the write;")
                    }) =>
                {
                    let Some(pull) = row.landing.round.pull else { return false };
                    Restored::ClosePermission(pull)
                }
                _ => return false,
            };
        let Some(index) = self.sweep_faults.restored.iter().position(|restored| *restored == fault) else {
            return false;
        };
        self.sweep_faults.restored.remove(index);
        self.sweep_faults.released += 1;
        true
    }

    #[must_use]
    pub fn restored_fault_releases(&self) -> u32 {
        self.sweep_faults.released
    }

    pub(super) fn sweep_deadline(&self) -> Option<Time> {
        self.sweep_faults
            .refused
            .map(|(_, until)| until)
            .into_iter()
            .chain(self.sweep_faults.closed.map(|(_, until, _)| until))
            .min()
    }

    /// Pull, base before, and base after each other user's main-branch merge.
    pub fn take_other_merges(&mut self) -> Vec<(u64, u64, u64)> {
        std::mem::take(&mut self.sweep_faults.other_merges)
    }

    pub(super) fn track_sweep_call(&mut self, fell: &forge::Fell) {
        if let Some(call) = fell.call {
            let kind = Kind { family: fell.fault.family(), name: fell.fault.name() };
            if call == forge::Called::Read
                && BUSY.contains(&kind)
                && self.sweep_faults.calls.as_ref().is_some_and(|(until, _)| fell.at < *until)
            {
                self.sweep_faults.busy_read_fell = true;
            }
            let struck = (kind, call);
            if !self.sweep_faults.fell.contains(&struck) {
                self.sweep_faults.fell.push(struck);
            }
        }
    }

    pub(super) fn sweep_call(&mut self, event: &forge::Event, config: &mut forge::Config) {
        if matches!(event, forge::Event::Call { op: raw::Op::Read(_), .. })
            && self.stored.iter().any(|(_, row)| matches!(row, top::Stored::Landing(_)))
            && let Some(span) = self.sweep_faults.crowd.take()
        {
            let mut env =
                skein_lib::Env { now: self.env.now, wall: self.env.wall, limits: self.forge_env.limits.clone() };
            env.limits.latency_min = span;
            env.limits.latency_max = span;
            for _ in 0..env.limits.limits.calls {
                self.serial += 1;
                let token = skein_lib::Token::new(self.serial);
                self.sweep_faults.neighbours.push(token);
                forge::step(
                    &mut self.forge,
                    &env,
                    forge::Event::Call {
                        reply_to: skein_lib::ReplyTo::new(token),
                        user: 2,
                        repository: Box::from(b"org/repo".as_slice()),
                        op: raw::Op::Read(raw::Read::Branch { branch: Box::from(b"main".as_slice()) }),
                    },
                    &mut self.forge_out,
                );
            }
        }
        if let Some((until, faults)) = &self.sweep_faults.calls
            && self.env.now < *until
            && self.stored.iter().any(|(_, row)| matches!(row, top::Stored::Landing(_)))
        {
            *config = faults.clone();
        }
        if matches!(event, forge::Event::Call { op: raw::Op::Write(raw::Write::Merge { .. }), .. })
            && let Some((until, faults)) = self.sweep_faults.merge.take()
            && self.env.now < until
        {
            *config = faults;
        }
        if let forge::Event::Call { op: raw::Op::Write(raw::Write::Close { number }), .. } = event
            && let Some(span) = self.sweep_faults.refusal.take()
        {
            forge::refuse_close(&mut self.forge, b"org/repo", *number, true);
            self.sweep_faults.refused = Some((*number, self.env.now.saturating_add(span)));
        }
    }

    pub(super) fn stop_sweep_faults(&mut self) {
        if let Some((number, until)) = self.sweep_faults.refused
            && self.env.now >= until
        {
            forge::refuse_close(&mut self.forge, b"org/repo", number, false);
            self.sweep_faults.refused = None;
            self.sweep_faults.restored.push(Restored::ClosePermission(number));
        }
        if let Some((number, until, entry)) = self.sweep_faults.closed
            && self.env.now >= until
        {
            self.sweep_faults.closed = None;
            let pending = std::mem::replace(&mut self.forge_out, skein_lib::Queue::uncapped());
            assert_eq!(self.external_now(raw::Op::Write(raw::Write::Reopen { number }), 2), raw::Answer::Done);
            self.forge_out = pending;
            self.stop(entry);
            self.sweep_faults.restored.push(Restored::Reopened(number));
        }
    }

    pub(super) fn sweep_after_read(&mut self, result: &Result<raw::Answer, raw::Error>) {
        let Ok(raw::Answer::Pull(pull)) = result else { return };
        if pull.state != raw::State::Open || self.sweep_faults.changed.contains(&pull.number) {
            return;
        }
        let Some(index) = self.sweep_faults.others.iter().position(|(task, _, _)| {
            self.stored.iter().any(|(_, record)| {
                matches!(record, top::Stored::ChangeHot(row) if row.task == *task && row.pull == Some(pull.number))
            })
        }) else {
            return;
        };
        let (_, kind, merge) = self.sweep_faults.others[index];
        let unrecorded = self.stored.iter().any(|(_, record)| {
            matches!(record, top::Stored::Landing(row) if row.landing.members.iter().any(|m| m.pull == pull.number && !m.recorded))
        });
        let recorded = self.stored.iter().any(|(_, record)| {
            matches!(record, top::Stored::Landing(row) if row.landing.members.iter().any(|m| m.pull == pull.number && m.recorded))
        });
        let queued = !recorded
            && self.stored.iter().any(|(_, record)| {
                matches!(record, top::Stored::ChangeHot(row) if row.pull == Some(pull.number)
                && matches!(row.change.state, temper_engine_domain_forge_change::State::Ready { .. }
                    | temper_engine_domain_forge_change::State::Queued { .. }))
            });
        if !unrecorded && !self.sweep_faults.open_change && !queued {
            return;
        }
        self.sweep_faults.others.remove(index);
        if merge
            && (unrecorded || queued)
            && let Some((fault, scenario)) = self.sweep_faults.excluded_merge
        {
            assert_eq!(kind, fault);
            self.ledger.leave_out(scenario, kind, self.env.now);
            return;
        }
        self.sweep_faults.changed.push(pull.number);
        let entry = self.inject(kind, Struck { request: Some(pull.number.to_string()), ..Struck::default() });
        let op = if merge {
            raw::Write::Merge { number: pull.number, head: pull.commit, how: raw::MergeHow::Style }
        } else {
            raw::Write::Close { number: pull.number }
        };
        let pending = std::mem::replace(&mut self.forge_out, skein_lib::Queue::uncapped());
        let before = self.branch(b"main");
        let answer = self.external_now(raw::Op::Write(op), 2);
        self.forge_out = pending;
        assert!(matches!(answer, raw::Answer::Done | raw::Answer::Merged(_)), "other user's write: {answer:?}");
        if merge {
            assert_eq!(pull.base.as_ref(), b"main", "the other user's merge targets the workload's base");
            self.sweep_faults.other_merges.push((pull.number, before, self.branch(b"main")));
            self.stop(entry);
        } else {
            assert!(self.sweep_faults.closed.is_none(), "one finite external close");
            self.sweep_faults.closed = Some((
                pull.number,
                self.env.now.saturating_add(self.sweep_faults.close_span.expect("configured close span")),
                entry,
            ));
        }
    }
}
