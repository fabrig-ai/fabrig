//! Periodic CI execution readings (temper/docs/design/domain/forge/change/stalls.md, 2.1).
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::Map;
use temper_engine_domain_forge_change::{Execution, Reading};

#[derive(Hash, Debug)]
pub(crate) struct Gather {
    checks: Box<[api::Status]>,
    runs: Map<u64, Option<api::Run>>,
}

impl Gather {
    pub(crate) fn new(checks: Box<[api::Status]>) -> Self {
        Self { checks, runs: Map::with_capacity(8) }
    }

    pub(crate) fn next(&mut self) -> Option<api::Read> {
        if self.runs.len() == 8 {
            return None;
        }
        let mut newest = None;
        for check in &self.checks {
            if check.check == api::Check::Pending
                && let Some(job) = check.job
                && !self.runs.contains_key(&job.run)
            {
                newest = Some(newest.unwrap_or(0_u64).max(job.run));
            }
        }
        let run = newest?;
        self.runs.insert(run, None).expect("at most eight distinct runs");
        Some(api::Read::Run { run })
    }

    pub(crate) fn answered(&mut self, id: u64, answer: Result<api::Answer, api::Error>) {
        if let Ok(api::Answer::Run(run)) = answer {
            self.runs.insert(id, Some(run)).expect("requested run");
        }
    }

    pub(crate) fn reading(&self) -> Reading {
        fold(&self.checks, &self.runs)
    }
}

pub(crate) fn fold(checks: &[api::Status], runs: &Map<u64, Option<api::Run>>) -> Reading {
    let mut reading = Reading { execution: Execution::Unknown, run: 0, attempt: 0 };
    let mut pending = false;
    let mut waiting = true;
    let mut running = false;
    for (id, run) in runs {
        if run.is_some() {
            reading.run = reading.run.max(*id);
        }
    }
    for check in checks {
        if check.check != api::Check::Pending {
            continue;
        }
        pending = true;
        let job = linked_job(check.job, runs);
        if let Some(job) = job {
            reading.attempt = reading.attempt.max(job.attempt);
            match job.state {
                api::RunState::Running => {
                    running = true;
                    waiting = false;
                }
                api::RunState::Waiting => {}
                api::RunState::Completed => waiting = false,
            }
        } else {
            waiting = false;
        }
    }
    reading.execution = if running {
        Execution::Running
    } else if pending && waiting {
        Execution::Waiting
    } else {
        Execution::Unknown
    };
    reading
}

#[expect(clippy::manual_find, reason = "the step subset uses explicit loops, not iterator closures")]
fn linked_job(link: Option<api::JobAttempt>, runs: &Map<u64, Option<api::Run>>) -> Option<&api::RunJob> {
    let link = link?;
    let run = runs.get(&link.run)?.as_ref()?;
    for job in &run.jobs {
        if job.id == link.job {
            return Some(job);
        }
    }
    None
}
