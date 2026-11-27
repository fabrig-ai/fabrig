//! Runner failures at a change's head (domain/forge/change/README.md, 2, step 3).
use alloc::boxed::Box;
use skein_forge::api::{self, Check, Commit, JobAttempt, Read};
use skein_lib::{Decimal, List, Map};
use temper_engine_domain_forge_change::Fault;
use temper_engine_domain_forge_landing::{Fault as RunFault, Judgement, Shown, job_fault, judge};

const HOLD_BYTES: u32 = jig_domain_tasks::HOLD_ACCOUNT_BYTES;
const RECOVERY: &[u8] = b"; the change wasn't judged. Release it to run CI again, or cancel it.";

#[derive(Hash, Debug)]
pub(super) struct Failed {
    checks: Box<[api::Status]>,
    runs: Map<u64, Option<api::Run>>,
}
impl Failed {
    pub(super) fn baseline(&self, head: Commit) -> Box<[JobAttempt]> {
        release_baseline(&self.checks, head)
    }

    pub(super) fn new(checks: Box<[api::Status]>) -> Self {
        // At most one run per check, plus the change's pinned recheck.
        let capacity = u32::try_from(checks.len()).expect("bounded checks").saturating_add(1);
        Self { runs: Map::with_capacity(capacity), checks }
    }

    pub(super) fn next(&mut self, head: Commit) -> Option<Read> {
        for check in &self.checks {
            if check.check == Check::Failed
                && let Some(job) = check.job
                && job.head == head
                && !self.runs.contains_key(&job.run)
            {
                self.runs.insert(job.run, None).expect("one run per check");
                return Some(Read::Run { run: job.run });
            }
        }
        None
    }

    pub(super) fn next_recheck(&mut self, job: JobAttempt) -> Option<Read> {
        if self.runs.contains_key(&job.run) {
            return None;
        }
        self.runs.insert(job.run, None).expect("one recheck run");
        Some(Read::Run { run: job.run })
    }

    pub(super) fn measurement(&self, recheck: crate::CiRecheck) -> Option<u32> {
        let expected = recheck.job;
        let run = self.runs.get(&expected.run)?.as_ref()?;
        if run.commit != expected.head || run.id != expected.run {
            return None;
        }
        for job in &run.jobs {
            if job.id == expected.job
                && job.attempt > expected.attempt
                && job.attempt >= recheck.taken.unwrap_or(0)
                && job.state == api::RunState::Completed
            {
                // A check that still links the old attempt is not a verdict of this run.
                for check in &self.checks {
                    if let Some(link) = check.job
                        && link.head == expected.head
                        && link.run == expected.run
                        && link.job == expected.job
                        && link.attempt != job.attempt
                    {
                        return None;
                    }
                }
                return Some(job.attempt);
            }
        }
        None
    }

    pub(super) fn red_stands(&self, head: Commit, repair: u64, measured: bool) -> Box<[u8]> {
        if !measured {
            return crate::recheck::stands_unchecked(
                head,
                repair,
                b"no failed check names a job the forge can run again",
            );
        }
        let mut keys = List::with_capacity(u32::try_from(self.checks.len()).expect("bounded checks"));
        for check in &self.checks {
            if check.check == Check::Failed {
                keys.push(check.context.clone()).expect("at most one context per check");
            }
        }
        crate::recheck::stands_again(head, repair, keys.as_slice())
    }

    pub(super) fn read(&mut self, id: u64, answer: Result<api::Answer, api::Error>) {
        if let Ok(api::Answer::Run(run)) = answer {
            self.runs.insert(id, Some(run)).expect("requested run");
        }
    }

    pub(super) fn fault(&self, head: Commit, baseline: &[JobAttempt]) -> Fault {
        let counted = counts(baseline, head);
        let mut shown = Shown::Nothing;
        let mut first = None;
        let mut reruns = 0;
        for check in &self.checks {
            if check.check != Check::Failed {
                continue;
            }
            if let Some((attempt, fact)) = self.runner_check(check, head) {
                if first.is_none() {
                    first = Some(fact);
                }
                reruns = reruns.max(runs_again(&counted, attempt));
            } else {
                shown = Shown::Failures;
            }
        }
        // A failed aggregate with no readable red check also names no runner fact.
        if first.is_none() {
            shown = Shown::Failures;
        }
        match judge(shown, first) {
            Judgement::Refused => Fault::Change,
            Judgement::Runner(_) => Fault::Runner { reruns },
            Judgement::Passed | Judgement::Broke => unreachable!("a failed CI read shows red or a runner fact"),
        }
    }

    fn runner_check(&self, check: &api::Status, head: Commit) -> Option<(JobAttempt, RunFault)> {
        let attempt = check.job?;
        let run = self.runs.get(&attempt.run)?.as_ref()?;
        Some((attempt, runner_fault(run, attempt, head)?))
    }

    pub(super) fn hold_words(&self, head: Commit, baseline: &[JobAttempt]) -> Box<[u8]> {
        let counted = counts(baseline, head);
        let mut words = List::with_capacity(HOLD_BYTES);
        let commit = crate::commit_text(&head);
        append(&mut words, b"CI's runner failed at ");
        append(&mut words, commit.get(..12).expect("commit prefix"));
        let mut separator: &[u8] = b": ";
        let more = " …".as_bytes();
        for check in &self.checks {
            if check.check != Check::Failed {
                continue;
            }
            let Some((attempt, fact)) = self.runner_check(check, head) else { continue };
            let job = Decimal::of(attempt.job);
            let times = runs_again(&counted, attempt);
            let count = Decimal::of(u64::from(times));
            let unit: &[u8] = if times == 1 { b" time" } else { b" times" };
            let start = [separator, b"job ", job.as_bytes(), b" of check "];
            let end = [b": ".as_slice(), reason(fact), b", run again ", count.as_bytes(), unit];
            let mut room = skein_lib::usize_of(words.room()).saturating_sub(RECOVERY.len()).saturating_sub(more.len());
            for parts in [start.as_slice(), end.as_slice()] {
                for part in parts {
                    room = room.saturating_sub(part.len());
                }
            }
            if room < more.len() {
                append(&mut words, more);
                break;
            }
            for part in start {
                append(&mut words, part);
            }
            append(&mut words, &crate::text::fit(&check.context, room, "…".as_bytes()));
            for part in end {
                append(&mut words, part);
            }
            separator = b"; ";
        }
        append(&mut words, RECOVERY);
        words.into_boxed()
    }

    pub(super) fn job(&self, head: Commit) -> Option<JobAttempt> {
        for check in &self.checks {
            if check.check == Check::Failed
                && let Some((attempt, _)) = self.runner_check(check, head)
            {
                return Some(attempt);
            }
        }
        for check in &self.checks {
            if check.check == Check::Failed
                && let Some(attempt) = check.job
                && attempt.head == head
                && let Some(Some(run)) = self.runs.get(&attempt.run)
                && run.commit == head
                && run.id == attempt.run
            {
                for job in &run.jobs {
                    if job.id == attempt.job && job.attempt == attempt.attempt {
                        return Some(attempt);
                    }
                }
            }
        }
        None
    }

    pub(super) fn account(&self, head: Commit, reason: Option<&[u8]>) -> Option<Box<[u8]>> {
        let attempt = self.job(head)?;
        let mut context = None;
        for check in &self.checks {
            if check.check == Check::Failed && check.job == Some(attempt) {
                context = Some(check.context.as_ref());
                break;
            }
        }
        let context = context?;
        let run = self.runs.get(&attempt.run)?.as_ref()?;
        let reason = match reason {
            Some(reason) => reason,
            None => runner(run, attempt, head)?,
        };
        let job = Decimal::of(attempt.job);
        let commit = crate::commit_text(&head);
        let head = commit.get(..12).expect("commit prefix");
        let fixed = b"ran job "
            .len()
            .saturating_add(job.as_bytes().len())
            .saturating_add(b" of ".len())
            .saturating_add(b" again at ".len())
            .saturating_add(head.len())
            .saturating_add(b": ".len())
            .saturating_add(reason.len());
        let context = crate::text::fit(
            context,
            skein_lib::usize_of(crate::EFFECT_ACCOUNT_BYTES).saturating_sub(fixed),
            "…".as_bytes(),
        );
        let mut words = List::with_capacity(crate::EFFECT_ACCOUNT_BYTES);
        for bytes in [b"ran job ".as_slice(), job.as_bytes(), b" of ", &context, b" again at ", head, b": ", reason] {
            append(&mut words, bytes);
        }
        Some(words.into_boxed())
    }
}

fn append(words: &mut List<u8>, part: &[u8]) {
    for byte in part {
        words.push(*byte).expect("fitted CI account");
    }
}

// Every admitted check can contribute one attempt to the stored release baseline.
pub(super) fn release_baseline(checks: &[api::Status], head: Commit) -> Box<[JobAttempt]> {
    let mut attempts = List::with_capacity(u32::try_from(checks.len()).expect("bounded checks"));
    for check in checks {
        if let Some(attempt) = check.job
            && attempt.head == head
            && attempt.attempt > 1
        {
            attempts.push(attempt).expect("at most one attempt per check");
        }
    }
    attempts.into_boxed()
}

pub(super) fn baseline(row: &crate::ChangeRow) -> &[JobAttempt] {
    match &row.runner_ci {
        Some(crate::RunnerCi::Counting { attempts }) => attempts,
        Some(crate::RunnerCi::Released | crate::RunnerCi::Held { .. }) | None => &[],
    }
}

fn counts(baseline: &[JobAttempt], head: Commit) -> Map<(u64, u64), u32> {
    let mut counted = Map::with_capacity(u32::try_from(baseline.len()).expect("bounded baseline"));
    for attempt in baseline {
        if attempt.head == head {
            let key = (attempt.run, attempt.job);
            let latest = counted.get(&key).copied().unwrap_or(1).max(attempt.attempt);
            counted.insert(key, latest).expect("at most one entry per baseline attempt");
        }
    }
    counted
}

fn runs_again(counted: &Map<(u64, u64), u32>, attempt: JobAttempt) -> u32 {
    attempt.attempt.saturating_sub(counted.get(&(attempt.run, attempt.job)).copied().unwrap_or(1))
}

pub(crate) fn runner(run: &api::Run, attempt: JobAttempt, head: Commit) -> Option<&'static [u8]> {
    Some(reason(runner_fault(run, attempt, head)?))
}

fn reason(fault: RunFault) -> &'static [u8] {
    match fault {
        RunFault::SetUp => b"set-up failed",
        RunFault::Completion => b"completion failed",
        RunFault::NoStep => b"no step failed",
        RunFault::Cancelled => b"cancelled",
        RunFault::OtherCommit | RunFault::Lost | RunFault::MissingWorkflow | RunFault::Unreported => {
            unreachable!("not a fact from a CI job at the head")
        }
    }
}

/// Use the shared classifier on this check's job, not its run's other jobs.
fn runner_fault(run: &api::Run, attempt: JobAttempt, head: Commit) -> Option<RunFault> {
    if attempt.head != head || run.commit != head || run.id != attempt.run {
        return None;
    }
    for job in &run.jobs {
        if job.id != attempt.job {
            continue;
        }
        if job.attempt != attempt.attempt || job.state != api::RunState::Completed {
            return None;
        }
        // Temper does not cancel a change's head CI.
        return job_fault(crate::runs::run_job(job), false);
    }
    None
}

#[cfg(test)]
mod tests;
