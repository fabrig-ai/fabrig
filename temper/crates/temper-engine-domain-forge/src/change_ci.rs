//! Runner failures at a change's head (domain/forge/change/README.md, 2, step 3).
use alloc::boxed::Box;
use skein_forge::api::{self, Check, Commit, JobAttempt, Read};
use skein_lib::{Decimal, List, Map};
use temper_engine_domain_forge_change::Fault;
use temper_engine_domain_forge_landing::{Fault as RunFault, Verdict};

#[derive(Hash, Debug)]
pub(super) struct Failed {
    checks: Box<[api::Status]>,
    runs: Map<u64, Option<api::Run>>,
}
impl Failed {
    pub(super) fn new(checks: Box<[api::Status]>) -> Self {
        Self { runs: Map::with_capacity(u32::try_from(checks.len()).expect("bounded checks")), checks }
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

    pub(super) fn read(&mut self, id: u64, answer: Result<api::Answer, api::Error>) {
        if let Ok(api::Answer::Run(run)) = answer {
            self.runs.insert(id, Some(run)).expect("requested run");
        }
    }

    pub(super) fn fault(&self, head: Commit) -> Fault {
        let mut found = false;
        let mut reruns = 0;
        for check in &self.checks {
            if check.check != Check::Failed {
                continue;
            }
            let Some(attempt) = check.job else { return Fault::Change };
            let Some(Some(run)) = self.runs.get(&attempt.run) else { return Fault::Change };
            if runner(run, attempt, head).is_none() {
                return Fault::Change;
            }
            found = true;
            reruns = reruns.max(attempt.attempt.saturating_sub(1));
        }
        if found { Fault::Runner { reruns } } else { Fault::Change }
    }

    pub(super) fn job(&self, head: Commit) -> Option<JobAttempt> {
        for check in &self.checks {
            if check.check == Check::Failed
                && let Some(attempt) = check.job
                && let Some(Some(run)) = self.runs.get(&attempt.run)
                && runner(run, attempt, head).is_some()
            {
                return Some(attempt);
            }
        }
        None
    }

    pub(super) fn account(&self, head: Commit) -> Option<Box<[u8]>> {
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
        let reason = runner(run, attempt, head)?;
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
            for byte in bytes {
                if words.room() > 0 {
                    words.push(*byte).expect("account room");
                }
            }
        }
        Some(words.into_boxed())
    }
}

/// Use the landing's classifier on this check's job, not its run's other jobs.
pub(crate) fn runner(run: &api::Run, attempt: JobAttempt, head: Commit) -> Option<&'static [u8]> {
    if attempt.head != head || run.commit != head || run.id != attempt.run {
        return None;
    }
    for job in &run.jobs {
        if job.id != attempt.job {
            continue;
        }
        if job.attempt != attempt.attempt
            || job.state != api::RunState::Completed
            || job.conclusion != Some(api::Conclusion::Failure)
        {
            return None;
        }
        let single =
            api::Run { jobs: Box::new([job.clone()]), state: job.state, conclusion: job.conclusion, ..run.clone() };
        return match crate::runs::verdict(&single, head) {
            Verdict::Fault(RunFault::SetUp) => Some(b"set-up failed"),
            Verdict::Fault(RunFault::Completion) => Some(b"completion failed"),
            Verdict::Fault(RunFault::NoStep) => Some(b"no step failed"),
            Verdict::Running
            | Verdict::Passed
            | Verdict::Failed
            | Verdict::Fault(
                RunFault::Cancelled | RunFault::OtherCommit | RunFault::Lost | RunFault::MissingWorkflow,
            ) => None,
        };
    }
    None
}
