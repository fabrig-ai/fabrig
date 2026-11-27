//! Forge faults on the composed world's forge calls (temper/docs/design/testing/composed-catalogue.md, 2).
use super::{Faults, World, forge_faults as kinds, stale};
use crate::composed::{Key, engine, people, tasks};
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api;
use skein_lib::{Duration, Env, Time, Token, Wall};
use skein_world::faults::{Injection, Kind, Struck};
use std::collections::BTreeMap;
use temper_blame::{Observed, Reaction};
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{open, translate::commit};
use temper_worker_checkout_world::forge::Move;

pub(super) const REMOTE: &[u8] = b"org/repo";
const OUTSIDE_USER: u64 = 2;
const UNAVAILABLE_SPAN: Duration = Duration::from_secs(2);
const LIMITED_SPAN: Duration = Duration::from_secs(2);
const LAG: fake::Lag = fake::Lag { chance: 1000, min: Duration::from_millis(100), max: Duration::from_secs(1) };
const NO_LAG: fake::Lag = fake::Lag { chance: 0, min: Duration::ZERO, max: Duration::ZERO };

const BASE_LAG_SPAN: Duration = Duration::from_secs(2);

pub(super) struct BaseLag {
    pulls: Vec<u64>,
    branch: Vec<u8>,
    before: u64,
    at: Time,
    until: Time,
}

pub(super) struct CloseCall {
    call: Token,
    repository: api::Repository,
    number: u64,
    ahead: usize,
}

pub(super) struct CloseRefusal {
    number: u64,
    goal: String,
    until: Time,
    entry: Option<Injection>,
}

#[derive(Default)]
pub(super) struct MergeLag {
    armed: bool,
    active: BTreeMap<u64, Time>,
}

fn hex(commit: u64) -> String {
    String::from_utf8(
        temper_engine_domain_forge::commit_text(&temper_engine_forge_world::translate::commit(commit)).into(),
    )
    .expect("hex")
}

impl World {
    pub(super) fn task_goal(&self, mut task: u64) -> Option<&str> {
        for _ in 0..32 {
            for goal in &self.workload().goals {
                if self.workload_names().task(&goal.name) == Some(task)
                    || goal.changes.iter().any(|change| self.workload_names().task(&change.name) == Some(task))
                {
                    return Some(&goal.name);
                }
            }
            let row = self.root.tasks().task(task)?;
            let tasks::Party::Task(parent) = row.requester else { return None };
            task = parent;
        }
        None
    }

    fn pull_goal(&self, number: u64) -> Option<&str> {
        for (task, row) in self.root.forge().inspection_changes() {
            if row.pull == Some(number) {
                return self.task_goal(*task);
            }
        }
        for (_, row) in self.root.forge().inspection_landings() {
            if row.landing.round.pull == Some(number) {
                return row.landing.members.first().and_then(|member| self.task_goal(member.change));
            }
        }
        None
    }

    fn branch_goal(&self, branch: &[u8]) -> Option<&str> {
        for (_, row) in self.root.forge().inspection_landings() {
            if row.landing.round.branch.as_ref() == branch
                || row.landing.members.iter().any(|m| m.branch.as_ref() == branch)
            {
                return row.landing.members.first().and_then(|member| self.task_goal(member.change));
            }
        }
        None
    }

    pub(super) fn candidate_goal(&self, commit: u64) -> Option<&str> {
        let commit = temper_engine_forge_world::translate::commit(commit);
        for (_, row) in self.root.forge().inspection_landings() {
            if let Some(member) = row.landing.members.iter().find(|m| m.candidate == Some(commit)) {
                return self.task_goal(member.change);
            }
        }
        None
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the configured forge families share one call's configuration and ledger boundary"
    )]
    pub(in crate::composed) fn workload_forge_call(&mut self, op: &api::Op) -> fake::Config {
        let mut config = self.forge.config();
        let Some(mut f) = self.workload_faults.take() else { return config };
        if f.due(kinds::TIMED_OUT)
            && let api::Op::Read(api::Read::Pull { number, .. }) = op
            && self.pull_goal(*number) == f.target(kinds::TIMED_OUT)
        {
            f.fire(kinds::TIMED_OUT);
            config.timeouts = 1000;
        } else if let api::Op::Read(api::Read::Pull { number, .. }) = op
            && (f.unavailable_until.is_some_and(|until| self.env.now < until) || f.due(kinds::UNAVAILABLE))
            && self.pull_goal(*number) == f.target(kinds::UNAVAILABLE)
        {
            // A call takes one busy reply; give the timeout above its own call.
            if f.unavailable_until.is_none() {
                f.fire(kinds::UNAVAILABLE);
                f.unavailable_until = Some(self.env.now.saturating_add(UNAVAILABLE_SPAN));
            }
            config.unavailable = 1000;
        } else if let api::Op::Read(api::Read::Pull { number, .. }) = op
            && (f.limited_until.is_some_and(|until| self.env.now < until) || f.due(kinds::LIMITED))
            && self.pull_goal(*number) == f.target(kinds::LIMITED)
        {
            config.rate_limit = 1;
            config.rate_window = LIMITED_SPAN;
            if f.limited_until.is_none() {
                // Spend the quota so the first struck read gets the fake's rate-limit reply.
                let env = Env { now: self.env.now, wall: Wall::EPOCH, limits: config.clone() };
                fake::rate(
                    self.forge.domain_mut(),
                    &env,
                    fake::RateKey::User(self.forge_binding.user),
                    env.limits.rate_limit,
                    env.limits.rate_window,
                )
                .expect("the limit starts with a fresh quota");
                f.fire(kinds::LIMITED);
                f.limited_until = Some(self.env.now.saturating_add(LIMITED_SPAN));
            }
        }
        if f.due(temper_blame::LOST)
            && let api::Op::Write(api::Write::Dispatch { branch, .. }) = op
            && self.branch_goal(branch) == f.target(temper_blame::LOST)
        {
            f.fire(temper_blame::LOST);
            // Dispatch schedules the job; the fake chooses its fault when it completes.
            config.lost = 1000;
        } else if f.plans.iter().any(|p| p.kind == temper_blame::LOST && p.fired)
            && !f.ledger.fell().contains(&temper_blame::LOST)
        {
            config.lost = 1000;
        }
        // The fake reports LOST in preference to SET_UP. Leave set-up due
        // for the next matching dispatch if the lost-job fault is still armed.
        if config.lost == 0
            && let Some(plan) = f.plans.iter().find(|p| p.kind == temper_blame::SET_UP)
        {
            if plan.fired {
                if !f.ledger.entries().iter().any(|entry| entry.kind == temper_blame::SET_UP) {
                    // The dispatch's job takes the fault when it completes.
                    config.setup_faults = 1000;
                }
            } else if f.due(temper_blame::SET_UP)
                && let api::Op::Write(api::Write::Dispatch { branch, .. }) = op
                && self.branch_goal(branch) == Some(plan.goal.as_str())
            {
                f.fire(temper_blame::SET_UP);
                config.setup_faults = 1000;
            }
        }
        if config.lost == 0
            && config.setup_faults == 0
            && f.due(temper_blame::SILENT)
            && let api::Op::Write(api::Write::Dispatch { branch, workflow, .. }) = op
            && self.branch_goal(branch) == f.target(temper_blame::SILENT)
        {
            // This workflow's job decides whether to report as it starts.
            fake::set_workflow_silent(self.forge.domain_mut(), REMOTE, workflow, 1000);
            f.silent_workflow = Some(workflow.clone());
        }
        if f.due(stale::MOVED_BASE)
            && let api::Op::Write(api::Write::Merge { number, .. }) = op
            && self.pull_goal(*number) == f.target(stale::MOVED_BASE)
            && let Ok(raw::Answer::Pull(pull)) = self.forge.inspect(REMOTE, &raw::Read::Pull { number: *number })
            && pull.base.as_ref() == b"main"
        {
            let goal = f.fire(stale::MOVED_BASE);
            let entry = f.ledger.inject(
                stale::MOVED_BASE,
                self.env.now,
                Struck { candidate: Some(hex(pull.commit)), request: Some(goal), ..Struck::default() },
            );
            // A legitimate, non-conflicting push after the engine read the base.
            self.forge.advance(REMOTE, &pull.base, b"outside/base-moved", b"a concurrent base write\n");
            f.ledger.stop(entry, self.env.now).expect("outside push");
            f.stopped(self.env.now);
            f.stale_release = self.store.rows.values().find_map(|row| match row {
                engine::Record::Forge { row } => match row.as_ref() {
                    temper_engine_domain_forge::Stored::Landing(landing)
                        if landing.landing.round.pull == Some(*number) =>
                    {
                        Some(landing.landing.task)
                    }
                    _ => None,
                },
                engine::Record::Core(_) => None,
            });
        }
        if f.due(kinds::CLOSED_BY_ANOTHER)
            && let api::Op::Read(api::Read::Pull { number, .. }) = op
            && self.pull_goal(*number) == f.target(kinds::CLOSED_BY_ANOTHER)
            && let Ok(raw::Answer::Pull(pull)) = self.forge.inspect(REMOTE, &raw::Read::Pull { number: *number })
            && pull.state == raw::State::Open
        {
            f.fire(kinds::CLOSED_BY_ANOTHER);
            let entry = f.ledger.inject(
                kinds::CLOSED_BY_ANOTHER,
                self.env.now,
                Struck { candidate: Some(hex(pull.commit)), request: Some(number.to_string()), ..Struck::default() },
            );
            assert_eq!(
                self.forge.api(REMOTE, 2, raw::Op::Write(raw::Write::Close { number: *number })),
                Ok(raw::Answer::Done)
            );
            f.closed = Some((*number, entry, self.env.now.saturating_add(Duration::from_secs(2))));
        }
        if let api::Op::Read(api::Read::Pull { number, .. }) = op
            && f.due(kinds::MERGED_BY_ANOTHER)
        {
            self.workload_other_merge(&mut f, *number);
        }
        if f.due(kinds::JOB_LOG)
            && let api::Op::Read(api::Read::Job { attempt, max_bytes, .. }) = op
            && let Ok(raw::Answer::Run(run)) = self.forge.inspect(REMOTE, &raw::Read::Run { run: attempt.run })
            && self.candidate_goal(run.commit) == f.target(kinds::JOB_LOG)
            && let Ok(raw::Answer::Job { steps, .. }) = self.forge.inspect(REMOTE, &raw::Read::Job { job: attempt.job })
            && let Some(step) = steps.iter().find(|step| step.status == raw::StepStatus::Failure)
        {
            // The fake's selected job log exceeds the client's bounded tail read.
            let mut log = vec![b'x'; usize::try_from(*max_bytes).expect("log cap") + 1];
            log.extend_from_slice(&step.log);
            fake::set_job_log(self.forge.domain_mut(), REMOTE, attempt.job, step.number, &log)
                .expect("bounded log prefix");
            let Ok(raw::Answer::Job { steps: enlarged, .. }) =
                self.forge.inspect(REMOTE, &raw::Read::Job { job: attempt.job })
            else {
                panic!("enlarged job")
            };
            let enlarged = &enlarged.iter().find(|found| found.number == step.number).expect("selected step").log;
            assert!(
                enlarged.len() > usize::try_from(*max_bytes).expect("log cap"),
                "the selected log exceeds the request, not another job"
            );
            assert!(enlarged.ends_with(&step.log), "the failed step's evidence stays in the tail");
            let goal = f.fire(kinds::JOB_LOG);
            let entry = f.ledger.inject(
                kinds::JOB_LOG,
                self.env.now,
                Struck {
                    request: Some(goal),
                    gate_run: Some(attempt.run.to_string()),
                    candidate: Some(hex(run.commit)),
                    ..Struck::default()
                },
            );
            f.log_restore = Some((attempt.job, step.number, step.log.clone(), entry));
        }
        if let api::Op::Write(api::Write::Merge { number, how, .. }) = op {
            // Head lag's switch also delays merge records. Arm it for this call
            // only if the campaign planned a late merge on this goal.
            let armed = self.workload_merge_lag(&mut f, *number, *how);
            if let Some(lag) = &mut f.merge_lag {
                lag.armed = armed;
            }
            fake::lag(self.forge.domain_mut(), if armed { LAG } else { NO_LAG });
        }
        if matches!(op, api::Op::Write(api::Write::Dispatch { .. } | api::Write::Rerun { .. })) {
            self.observe_gate_call(&mut f, op);
        }
        self.workload_faults = Some(f);
        config
    }

    pub(in crate::composed) fn workload_reopen_point(&self, repository: &[u8], op: &raw::Op) -> Option<(u64, usize)> {
        let raw::Op::Write(raw::Write::Close { number }) = *op else { return None };
        let f = self.workload_faults.as_ref()?;
        (repository == REMOTE && f.due(stale::REOPENED) && self.pull_goal(number) == f.target(stale::REOPENED))
            .then(|| (number, self.forge.observations().len()))
    }

    pub(in crate::composed) fn workload_reopen_after_close(&mut self, number: u64, before: usize) {
        // A successful close of an already closed pull is a no-op. Require the
        // engine's write to have closed it, then interleave before its next step.
        if !self.forge.observations()[before..].iter().any(|observation| {
            matches!(
                observation,
                fake::Observation::Closed { repository, number: closed, by }
                    if repository.as_ref() == REMOTE && *closed == number && *by == self.forge_binding.user
            )
        }) {
            return;
        }
        let pull = match self.forge.inspect(REMOTE, &raw::Read::Pull { number }) {
            Ok(raw::Answer::Pull(pull)) if pull.state == raw::State::Closed && pull.merged.is_none() => Some(pull),
            Ok(raw::Answer::Pull(_)) | Err(raw::Error::Missing(raw::What::Item | raw::What::Pull)) => None,
            other => panic!("read of pull {number} before outside reopen: {other:?}"),
        };
        let f = self.workload_faults.as_mut().expect("faulted workload");
        f.fire(stale::REOPENED);
        if let Some(pull) = pull {
            assert_ne!(OUTSIDE_USER, self.forge_binding.user);
            match self.forge.api(REMOTE, OUTSIDE_USER, raw::Op::Write(raw::Write::Reopen { number })) {
                Ok(raw::Answer::Done) => {
                    let entry = f.ledger.inject(
                        stale::REOPENED,
                        self.env.now,
                        Struck {
                            candidate: Some(hex(pull.commit)),
                            request: Some(number.to_string()),
                            ..Struck::default()
                        },
                    );
                    f.ledger.stop(entry, self.env.now).expect("outside reopen");
                    f.stopped(self.env.now);
                    return;
                }
                // The head vanished or a replacement owns this branch pair.
                Err(
                    raw::Error::Exists | raw::Error::Missing(raw::What::Item | raw::What::Pull | raw::What::Branch),
                ) => {}
                other => panic!("outside reopen of pull {number}: {other:?}"),
            }
        }
        f.ledger.ineligible(stale::REOPENED);
    }

    fn workload_other_merge(&mut self, f: &mut Faults, number: u64) {
        let kind = kinds::MERGED_BY_ANOTHER;
        let forge = self.root.forge();
        let Some((task, row)) = forge.inspection_changes().iter().find(|(_, row)| row.pull == Some(number)) else {
            return;
        };
        if self.task_goal(*task) != f.target(kind) {
            return;
        }
        let Ok(raw::Answer::Pull(pull)) = self.forge.inspect(REMOTE, &raw::Read::Pull { number }) else { return };
        // Strike the goal's pull into main, not a child's pull into the goal branch.
        if pull.state != raw::State::Open || pull.base.as_ref() != b"main" {
            return;
        }
        if let Some(scenario) = f.other_merge_exclusion {
            use temper_engine_domain_forge_change::State;
            let mut members = forge.inspection_landings().iter().flat_map(|(_, row)| row.landing.members.iter());
            if members.clone().any(|member| member.pull == number && !member.recorded)
                || (matches!(row.change.state, State::Ready { .. } | State::Queued { .. })
                    && !members.any(|member| member.pull == number && member.recorded))
            {
                f.ledger.leave_out(scenario, kind, self.env.now);
                return;
            }
        }
        let outside = raw::Write::Merge { number, head: pull.commit, how: raw::MergeHow::Style };
        let result = self.forge.api(REMOTE, 2, raw::Op::Write(outside));
        assert!(matches!(result, Ok(raw::Answer::Merged(_))), "outside merge of pull {number}: {result:?}");
        let goal = f.fire(kind);
        let entry = f.ledger.inject(
            kind,
            self.env.now,
            Struck { candidate: Some(hex(pull.commit)), request: Some(goal), ..Struck::default() },
        );
        f.ledger.stop(entry, self.env.now).expect("outside merge");
        f.stopped(self.env.now);
    }

    pub(in crate::composed) fn hold_workload_close(
        &mut self,
        call: Token,
        repository: api::Repository,
        op: &api::Op,
    ) -> bool {
        let api::Op::Write(api::Write::Close { number }) = op else { return false };
        let Some(f) = &self.workload_faults else { return false };
        if !(self.close_is_refused(f, *number) || self.close_refusal_due(f, *number)) {
            return false;
        }
        // Leave this close in reply order, but read the exclusion where the
        // terminal enters the engine: more reads can be admitted after send.
        let ahead = self.root_events.len();
        self.workload_faults.as_mut().expect("faulted workload").close_calls.push_back(CloseCall {
            call,
            repository,
            number: *number,
            ahead,
        });
        true
    }

    pub(in crate::composed) fn workload_close_waiting(&self) -> bool {
        self.workload_faults.as_ref().is_some_and(|f| !f.close_calls.is_empty())
    }

    pub(in crate::composed) fn workload_close_reply(&mut self) -> Option<engine::Event> {
        let f = self.workload_faults.as_mut()?;
        if f.close_calls.front()?.ahead != 0 {
            for close in &mut f.close_calls {
                close.ahead -= 1;
            }
            return None;
        }
        let CloseCall { call, repository, number, .. } = f.close_calls.pop_front().expect("due close reply");
        let mut f = self.workload_faults.take().expect("faulted workload");
        let pull_present = matches!(self.forge.inspect(REMOTE, &raw::Read::Pull { number }), Ok(raw::Answer::Pull(_)));
        if pull_present {
            self.begin_close_refusal(&mut f, number);
        }
        let refused = pull_present && self.close_is_refused(&f, number);
        self.workload_faults = Some(f);
        if refused {
            fake::refuse_close(self.forge.domain_mut(), REMOTE, number, true);
        }
        let op = api::Op::Write(api::Write::Close { number });
        let reply = self.forge_event(call, repository, &op);
        if refused {
            // Refuse only the engine's close, not another user's outside write.
            fake::refuse_close(self.forge.domain_mut(), REMOTE, number, false);
        }
        self.workload_forge_answer(&op);
        Some(reply)
    }

    fn close_refusal_due(&self, f: &Faults, number: u64) -> bool {
        f.due(kinds::CLOSE_REFUSED) && self.pull_goal(number) == f.target(kinds::CLOSE_REFUSED)
    }

    fn begin_close_refusal(&self, f: &mut Faults, number: u64) {
        if self.close_refusal_due(f, number) {
            let goal = f.fire(kinds::CLOSE_REFUSED);
            // The close is still in the client's table while it handles the
            // refusal and tries to admit the capability refresh's first read.
            if let Some(scenario) = f.close_refusal_exclusion
                && !self.root.forge().inspection_client().has_room()
            {
                f.ledger.leave_out(scenario, kinds::CLOSE_REFUSED, self.env.now);
                f.ledger.ineligible(kinds::CLOSE_REFUSED);
                eprintln!("close refusal: pull {number} left out by {scenario}");
                return;
            }
            f.close_refusal = Some(CloseRefusal {
                number,
                goal,
                until: self.env.now.saturating_add(Duration::from_secs(2)),
                entry: None,
            });
        }
    }

    fn close_is_refused(&self, f: &Faults, number: u64) -> bool {
        f.close_refusal.as_ref().is_some_and(|refusal| refusal.number == number && self.env.now < refusal.until)
    }

    fn stop_close_refusal(&mut self, f: &mut Faults) {
        if f.close_refusal.as_ref().is_some_and(|refusal| self.env.now >= refusal.until) {
            let CloseRefusal { number, until, entry, .. } = f.close_refusal.take().expect("close refusal due to stop");
            if let Some(entry) = entry {
                f.ledger.stop(entry, until).expect("close permission restored");
                f.stopped(until);
                f.close_permission_restored = Some(number);
            } else {
                f.ledger.ineligible(kinds::CLOSE_REFUSED);
            }
        }
    }

    fn release_restored_close(&mut self, f: &mut Faults) {
        let Some(number) = f.close_permission_restored else { return };
        let close_protected = b"close item: forge protection refused the write;";
        let cleanup = self
            .workload_landings()
            .find(|row| {
                row.landing.round.pull == Some(number)
                    && matches!(row.landing.phase, landing::Phase::Held { why: landing::Hold::Cleanup, .. })
                    && row.landing.round.cleanup.as_deref().is_some_and(|account| account.starts_with(close_protected))
            })
            .map(|row| row.landing.task);
        let Some(task) = cleanup else { return };
        let Some(row) = self.root.tasks().task(task) else { return };
        let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } = &row.phase else { return };
        let tasks::HoldSource::Account { words, .. } = source.as_ref() else { return };
        if !words.strip_prefix(b"cleanup: ").is_some_and(|account| account.starts_with(close_protected)) {
            return;
        }
        let revision = row.revisions.hold;
        let target = f.target(kinds::CLOSE_REFUSED).expect("refused close target");
        let party = self.workload().goal(target).expect("refused close goal").party;
        eprintln!("close refusal: {party:?} releases task {task} for pull {number} at {:?}", self.env.now);
        self.workload_enqueue(party, people::Ask::Release { project: 1, task, revision, again: None });
        f.close_permission_restored = None;
    }

    fn workload_merge_lag(&self, f: &mut Faults, number: u64, how: api::MergeHow) -> bool {
        let kind = kinds::MERGE_LATE;
        let Some(lag) = &f.merge_lag else { return false };
        if !f.due(kind)
            || self.pull_goal(number) != f.target(kind)
            || lag.active.get(&number).is_some_and(|&until| self.env.now < until)
        {
            return false;
        }
        if let api::MergeHow::Recorded { commit } = how
            && self.workload_landings().any(|row| open::recording_landed_squash(row, number, commit))
        {
            f.ledger.leave_out(open::MERGE_LATE_RECORD.scenario, kind, self.env.now);
            return false;
        }
        true
    }

    pub(in crate::composed) fn workload_base_advance(&mut self, moves: &[Move], to: api::Commit) {
        let Some(f) = &self.workload_faults else { return };
        if !f.base_behind_configured
            || !f.due(kinds::BASE_BEHIND)
            || f.base_lag.as_ref().is_some_and(|lag| self.env.now < lag.until)
        {
            return;
        }
        let goal = f.target(kinds::BASE_BEHIND);
        for movement in moves.iter().rev() {
            let Some(before) = movement.from else { continue };
            if movement.remote != REMOTE || before == movement.to || commit(movement.to) != to {
                continue;
            }
            let mut pulls = Vec::new();
            for page in 1..=self.forge.config().limits.items {
                let read =
                    raw::Read::OpenPulls { head: None, base: Some(movement.branch.as_slice().into()), page, limit: 1 };
                let Ok(raw::Answer::Pulls(found)) = self.forge.inspect(REMOTE, &read) else { break };
                let Some(pull) = found.first() else { break };
                if self.pull_goal(pull.number) == goal {
                    pulls.push(pull.number);
                }
            }
            if !pulls.is_empty() {
                self.workload_faults.as_mut().expect("faulted workload").base_lag = Some(BaseLag {
                    pulls,
                    branch: movement.branch.clone(),
                    before,
                    at: self.env.now,
                    until: self.env.now.saturating_add(BASE_LAG_SPAN),
                });
                return;
            }
        }
    }

    pub(in crate::composed) fn workload_base_behind_reply(&mut self, op: &api::Op, pull: &mut api::Pull) {
        let Some(f) = &mut self.workload_faults else { return };
        let Some(lag) = &f.base_lag else { return };
        if !matches!(op, api::Op::Read(api::Read::Pull { .. }))
            || self.env.now >= lag.until
            || !lag.pulls.contains(&pull.number)
            || pull.base.as_ref() != lag.branch
            || pull.state != api::State::Open
        {
            return;
        }
        // Change only the base field, leaving head lag and merge records alone.
        let before = commit(lag.before);
        if pull.base_commit == Some(before) {
            return;
        }
        pull.base_commit = Some(before);
        let kind = kinds::BASE_BEHIND;
        if f.due(kind) {
            let (at, until, base) = (lag.at, lag.until, hex(lag.before));
            let goal = f.fire(kind);
            let entry =
                f.ledger.inject(kind, at, Struck { request: Some(goal), base: Some(base), ..Struck::default() });
            f.ledger.stop(entry, until).expect("base lag interval");
            f.stopped(until);
        }
    }

    pub(in crate::composed) fn workload_forge_answer(&mut self, _op: &api::Op) {
        let Some(mut f) = self.workload_faults.take() else { return };
        if let Some(workflow) = f.silent_workflow.take() {
            fake::set_workflow_silent(self.forge.domain_mut(), REMOTE, &workflow, 0);
        }
        self.copy_workload_forge_faults(&mut f);
        self.record_workload_reds(&mut f);
        if let Some((job, step, log, entry)) = f.log_restore.take() {
            fake::set_job_log(self.forge.domain_mut(), REMOTE, job, step, &log).expect("restore the bounded log");
            f.ledger.stop(entry, self.env.now).expect("bounded read answered");
            f.stopped(self.env.now);
        }
        let armed = f.merge_lag.as_mut().is_some_and(|lag| std::mem::take(&mut lag.armed));
        if f.lag_until.is_some() {
            fake::lag(self.forge.domain_mut(), LAG);
        } else if armed {
            fake::lag(self.forge.domain_mut(), NO_LAG);
        }
        self.workload_faults = Some(f);
    }

    pub(super) fn copy_workload_forge_faults(&mut self, f: &mut Faults) {
        assert_eq!(self.forge.domain_mut().fell_lost(), 0, "fake forge lost fault records");
        while let Some(fell) = self.forge.domain_mut().pop_fell() {
            let kind = Kind { family: fell.fault.family(), name: fell.fault.name() };
            if kind == kinds::CLOSE_REFUSED
                && let Some(refusal) = &mut f.close_refusal
                && fell.pull == Some(refusal.number)
            {
                // Count one interval, including any retries of this close.
                refusal.entry.get_or_insert_with(|| {
                    eprintln!("close refusal: pull {} at {:?} until {:?}", refusal.number, fell.at, refusal.until);
                    f.ledger.inject(kind, fell.at, Struck { request: Some(refusal.goal.clone()), ..Struck::default() })
                });
                continue;
            }
            let until = fell.until.or(match kind {
                kinds::UNAVAILABLE => f.unavailable_until,
                kinds::LIMITED => f.limited_until,
                _ => None,
            });
            let until = until.unwrap_or(fell.at);
            if matches!(kind, kinds::HEAD_BEHIND | kinds::BASE_BEHIND)
                && let Some(lag) = &mut f.merge_lag
                && let Some(pull) = fell.pull
            {
                lag.active.insert(pull, until);
            }
            if f.base_behind_configured && kind == kinds::BASE_BEHIND {
                // HEAD_BEHIND can lag other base pushes while its switch is on.
                // Keep their time bound, but count only a scoped, observed view.
                f.stopped(until);
                continue;
            }
            if kind == temper_blame::SET_UP {
                assert_eq!(
                    fell.commit.and_then(|commit| self.candidate_goal(commit)),
                    f.target(kind),
                    "ci/set_up must strike its planned goal"
                );
            }
            let request = if kind == kinds::MERGE_LATE {
                // A refused merge or one that cannot lag leaves the plan due.
                Some(f.fire(kind))
            } else {
                fell.reply_to.map(|token| token.raw().to_string())
            };
            let entry = f.ledger.inject(
                kind,
                fell.at,
                Struck {
                    request,
                    gate_run: fell.run.map(|run| run.to_string()),
                    runner: fell.runner.map(|runner| runner.to_string()),
                    candidate: fell.commit.map(hex),
                    ..Struck::default()
                },
            );
            if kind == temper_blame::SILENT {
                assert_eq!(
                    fell.commit.and_then(|commit| self.candidate_goal(commit)),
                    f.target(kind),
                    "ci/silent must strike its planned goal"
                );
                f.fire(kind);
                f.silent_run = Some((fell.run.expect("silent workflow run"), entry));
                continue;
            }
            f.ledger.stop(entry, until).expect("fake forge interval");
            f.stopped(until);
        }
    }

    fn release_silent_gate(&mut self, f: &mut Faults) {
        let Some((run, injection)) = f.silent_run else { return };
        let held = self.workload_landings().find_map(|row| {
            use temper_engine_domain_forge_landing::{Hold, Phase};
            let Phase::Held { why: Hold::Timeout { run: timed_out, member, .. }, .. } = row.landing.phase else {
                return None;
            };
            if timed_out != run {
                return None;
            }
            let task = self.workload_live_task(row.landing.task)?;
            if !matches!(task.phase, tasks::Phase::Held { why, .. } if !matches!(why, tasks::Hold::Placed { .. })) {
                return None;
            }
            let goal = self.task_goal(row.landing.members[usize::from(member)].change)?;
            let party = self.workload().goals.iter().find(|drawn| drawn.name == goal)?.party;
            Some((task.number, task.revisions.hold, party))
        });
        if let Some((task, revision, party)) = held {
            // A release follows the old run until it ends (landing-proof.md, 2.1).
            // Only now that the real timeout held the landing, end its silent run.
            self.forge.at(self.env.now);
            assert_ne!(OUTSIDE_USER, self.forge_binding.user);
            assert_eq!(
                self.forge.api(REMOTE, OUTSIDE_USER, raw::Op::Write(raw::Write::Cancel { run })),
                Ok(raw::Answer::Done),
                "cancel the still-running silent gate"
            );
            f.ledger.stop(injection, self.env.now).expect("silent gate reached its timeout");
            f.stopped(self.env.now);
            f.silent_run = None;
            self.workload_enqueue(party, people::Ask::Release { project: 1, task, revision, again: None });
        }
    }

    pub(super) fn workload_forge_tick(&mut self, f: &mut Faults) {
        self.release_silent_gate(f);
        if f.due(kinds::HEAD_BEHIND) {
            f.fire(kinds::HEAD_BEHIND);
            fake::lag(self.forge.domain_mut(), LAG);
            f.lag_until = Some(self.env.now.saturating_add(Duration::from_secs(1200)));
        }
        if f.lag_until.is_some_and(|until| self.env.now >= until || f.ledger.fell().contains(&kinds::HEAD_BEHIND)) {
            fake::lag(self.forge.domain_mut(), NO_LAG);
            f.lag_until = None;
            f.stopped(self.env.now);
        }
        if let Some((number, entry, until)) = f.closed
            && self.env.now >= until
        {
            let restored = match self.forge.inspect(REMOTE, &raw::Read::Pull { number }) {
                Ok(raw::Answer::Pull(pull)) if pull.state == raw::State::Open => true,
                Ok(raw::Answer::Pull(pull))
                    if pull.merged.is_none() && self.forge.branch(REMOTE, &pull.head).is_some() =>
                {
                    match self.forge.api(REMOTE, 2, raw::Op::Write(raw::Write::Reopen { number })) {
                        Ok(raw::Answer::Done) => true,
                        // Another pull now owns this branch pair.
                        Err(raw::Error::Exists) => false,
                        other => panic!("reopen of live pull {number}: {other:?}"),
                    }
                }
                Ok(raw::Answer::Pull(_)) | Err(raw::Error::Missing(raw::What::Item | raw::What::Pull)) => false,
                other => panic!("read of pull {number} before reopening: {other:?}"),
            };
            f.ledger.stop(entry, self.env.now).expect("other-user close ends");
            f.stopped(self.env.now);
            f.closed = None;
            if restored {
                f.restored = Some(number);
            } else {
                f.ledger.ineligible(kinds::CLOSED_BY_ANOTHER);
            }
        }
        self.stop_close_refusal(f);
        self.release_restored_close(f);
        if let Some(task) = f.stale_release
            && let Some(row) = self.root.tasks().task(task)
            && matches!(row.phase, tasks::Phase::Held { .. })
        {
            let revision = row.revisions.hold;
            self.workload_enqueue(
                temper_workload::draw::Party::First,
                people::Ask::Release { project: 1, task, revision, again: None },
            );
            f.stale_release = None;
        }
        if let Some(number) = f.restored {
            use jig_domain_core::{Key as CoreKey, Record};
            use temper_engine_domain_forge::{Key as ForgeKey, Stored};
            let held = self
                .store
                .rows
                .range(Key::Core(CoreKey::Tasks(tasks::Key::Live(0)))..=Key::Core(CoreKey::Tasks(tasks::Key::Live(u64::MAX))))
                .find_map(|(_, record)| {
                    let engine::Record::Core(Record::Tasks(tasks::Stored::Live(row))) = record else { return None };
                    if !matches!(row.phase, tasks::Phase::Held { why, .. } if !matches!(why, tasks::Hold::Placed { .. })) {
                        return None;
                    }
                    let affected = self.store.change_hot(row.number).is_some_and(|change| change.pull == Some(number))
                        || matches!(self.store.rows.get(&Key::Forge(ForgeKey::Landing(row.number))),
                            Some(engine::Record::Forge { row: forge })
                                if matches!(forge.as_ref(), Stored::Landing(landing)
                                    if landing.landing.members.iter().any(|m| m.pull == number)));
                    affected.then_some((row.number, row.revisions.hold))
                });
            if let Some((task, revision)) = held {
                // Only release the hold whose other-user close this run restored.
                self.workload_enqueue(
                    temper_workload::draw::Party::First,
                    people::Ask::Release { project: 1, task, revision, again: None },
                );
                f.restored = None;
            }
        }
    }

    pub(super) fn gate_runs(&self) -> Vec<raw::Run> {
        let mut result = Vec::new();
        for page in 1.. {
            let Ok(raw::Answer::Runs(runs)) = self
                .forge
                .inspect(REMOTE, &raw::Read::Runs { workflow: None, commit: None, number: None, page, limit: 64 })
            else {
                panic!("runs")
            };
            let done = runs.len() < 64;
            result.extend(runs);
            if done {
                return result;
            }
        }
        unreachable!()
    }

    fn observe_gate_call(&self, f: &mut Faults, op: &api::Op) {
        let retry = match op {
            api::Op::Write(api::Write::Rerun { attempt }) => {
                if !f.retries.insert((attempt.run, attempt.job, attempt.attempt)) {
                    return;
                }
                self.gate_runs().into_iter().find(|run| run.id == attempt.run)
            }
            api::Op::Write(api::Write::Dispatch { inputs, .. }) => {
                let faults = f
                    .ledger
                    .entries()
                    .iter()
                    .filter(|fault| matches!(fault.kind, temper_blame::LOST | temper_blame::SET_UP));
                if faults.clone().next().is_none() {
                    return;
                }
                self.gate_runs()
                    .into_iter()
                    .filter(|run| {
                        run.inputs.iter().any(|old| {
                            old.0.as_ref() == b"temper_key"
                                && inputs.iter().any(|input| input.name == old.0 && input.value == old.1)
                        })
                    })
                    .filter(|run| {
                        let id = run.id.to_string();
                        faults.clone().any(|fault| fault.struck.gate_run.as_ref() == Some(&id))
                    })
                    .max_by_key(|run| run.id)
            }
            _ => None,
        };
        if let Some(run) = retry {
            let key = run.inputs.iter().find(|input| input.0.as_ref() == b"temper_key").expect("gate key");
            f.reactions.push(Observed {
                at: self.env.now,
                reaction: Reaction::GateRetry {
                    gate: String::from_utf8_lossy(&key.1).into_owned(),
                    run: run.id.to_string(),
                    bound: u64::from(self.root.forge().landing_settings(1).expect("project settings").gate_retries),
                },
            });
        }
    }
}
