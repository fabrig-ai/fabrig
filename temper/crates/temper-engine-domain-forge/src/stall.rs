//! Diagnostic reads only after a CI bound (change/repairs.md, A stall on CI says why;
//! domain/landing.md, 4.2 and 7.1). Reports are volatile; restoration has only the base hold.
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{Decimal, List, Writer};

#[derive(Clone, Hash, Debug)]
pub(crate) struct Report {
    jobs: Box<[Job]>,
    more: u64,
    reading: Option<api::JobAttempt>,
}
#[derive(Clone, Hash, Debug)]
struct Job {
    name: Box<[u8]>,
    attempt: Option<api::JobAttempt>,
    answer: Option<Result<api::Answer, api::Error>>,
}
impl crate::Domain {
    /// Volatile clauses appended whole within the caller's hold bound.
    #[must_use]
    pub fn ci_stall_words(&self, task: u64, base: &[u8], limit: u32) -> Box<[u8]> {
        if base.len() > skein_lib::usize_of(limit) {
            return skein_lib::bytes::copy_of(base);
        }
        match self.stall_words.get(&task) {
            Some(report) => report.words(base, limit),
            None => skein_lib::bytes::copy_of(base),
        }
    }
}
impl Report {
    pub(crate) fn checks(checks: &[api::Status]) -> Self {
        let mut jobs = List::with_capacity(8);
        let mut more = 0u64;
        for check in checks {
            if check.check == api::Check::Pending {
                if jobs.room() == 0 {
                    more = more.saturating_add(1);
                } else {
                    jobs.push(Job { name: check.context.clone(), attempt: check.job, answer: None })
                        .expect("eight clauses");
                }
            }
        }
        Self { jobs: jobs.into_boxed(), more, reading: None }
    }
    pub(crate) fn run(run: &api::Run) -> Self {
        let mut jobs = List::with_capacity(8);
        let mut more = 0u64;
        for job in &run.jobs {
            if job.state != api::RunState::Completed {
                if jobs.room() == 0 {
                    more = more.saturating_add(1);
                } else {
                    jobs.push(Job {
                        name: job.name.clone(),
                        attempt: Some(api::JobAttempt {
                            head: run.commit,
                            run: run.id,
                            job: job.id,
                            attempt: job.attempt,
                        }),
                        answer: None,
                    })
                    .expect("eight clauses");
                }
            }
        }
        Self { jobs: jobs.into_boxed(), more, reading: None }
    }
    pub(crate) fn next(&mut self) -> Option<api::Read> {
        for job in &self.jobs {
            if job.answer.is_none()
                && let Some(attempt) = job.attempt
            {
                self.reading = Some(attempt);
                return Some(api::Read::JobState { attempt });
            }
        }
        None
    }
    pub(crate) fn answered(&mut self, answer: Result<api::Answer, api::Error>) {
        let Some(attempt) = self.reading.take() else { return };
        for job in &mut self.jobs {
            if job.attempt == Some(attempt) {
                job.answer = Some(answer.clone());
            }
        }
    }
    pub(crate) fn words(&self, base: &[u8], limit: u32) -> Box<[u8]> {
        let mut kept = List::with_capacity(8);
        let mut length = base.len();
        let mut more = self.more;
        for job in &self.jobs {
            let clause = clause(job);
            if length.saturating_add(clause.len()) <= skein_lib::usize_of(limit) {
                length = length.saturating_add(clause.len());
                kept.push(clause).expect("eight clauses");
            } else {
                more = more.saturating_add(1);
            }
        }
        while more > 0 && length.saturating_add(suffix(more).len()) > skein_lib::usize_of(limit) {
            let Some(last) = kept.last() else {
                break;
            };
            length = length.saturating_sub(last.len());
            kept.truncate(kept.len().saturating_sub(1));
            more = more.saturating_add(1);
        }
        let mut words = List::with_capacity(limit);
        put(&mut words, base);
        for clause in &kept {
            put(&mut words, clause);
        }
        if more > 0 {
            put(&mut words, &suffix(more));
        }
        words.into_boxed()
    }
}
fn put(words: &mut List<u8>, part: &[u8]) {
    for byte in part {
        if words.room() > 0 {
            words.push(*byte).expect("hold room");
        }
    }
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded answer strings");
    }
    let mut words = Writer::new(length);
    for part in parts {
        words.put(part).expect("measured words");
    }
    words.finish()
}
fn suffix(more: u64) -> Box<[u8]> {
    join(&[b"; and ", Decimal::of(more).as_bytes(), b" more pending"])
}
fn clause(job: &Job) -> Box<[u8]> {
    let why = if job.attempt.is_none() {
        Box::from(b"pending, no job linked".as_slice())
    } else if let Some(Ok(api::Answer::JobState { state, labels, step, .. })) = &job.answer {
        match state {
            api::JobState::Waiting => {
                let mut words = Box::from(b"waiting for a runner".as_slice());
                for (index, label) in labels.iter().enumerate() {
                    words = join(&[&words, if index == 0 { b" labelled " } else { b", " }, label]);
                }
                words
            }
            api::JobState::Blocked => Box::from(b"blocked by the jobs it needs".as_slice()),
            api::JobState::Running => match step {
                Some(step) => {
                    join(&[b"running at step ", Decimal::of(u64::from(step.number)).as_bytes(), b", ", &step.name])
                }
                None => Box::from(b"running".as_slice()),
            },
            api::JobState::Completed => Box::from(b"completed".as_slice()),
        }
    } else {
        Box::from(b"pending, its job unread".as_slice())
    };
    join(&[b"; ", &job.name, b": ", &why])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clauses_fit_whole_and_count_every_omitted_pending_job() {
        let mut checks = List::with_capacity(3);
        for _ in 0_u32..3 {
            checks
                .push(api::Status {
                    at: skein_lib::Time::ZERO,
                    context: Box::from(b"test".as_slice()),
                    check: api::Check::Pending,
                    description: Box::new([]),
                    url: Box::new([]),
                    job: None,
                })
                .expect("three checks");
        }
        let report = Report::checks(checks.as_slice());
        for (limit, expected) in [
            (
                95,
                b"stall; test: pending, no job linked; test: pending, no job linked; test: pending, no job linked"
                    .as_slice(),
            ),
            (65, b"stall; test: pending, no job linked; and 2 more pending".as_slice()),
            (25, b"stall; and 3 more pending".as_slice()),
        ] {
            assert_eq!(report.words(b"stall", limit).as_ref(), expected);
        }
    }
}
