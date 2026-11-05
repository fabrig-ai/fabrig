//! Actions runs and nonrecursive trees (forge.md, 5.2).
use super::{
    Body, Heard,
    plan::{Bounds, Step},
    reading,
    tables::{Repository, Tables},
};
use crate::api;
use crate::forgejo::{document as d, request as r};
use alloc::boxed::Box;
use skein_lib::{List, Time};

#[derive(Debug)]
pub(super) enum Mode {
    Runs { workflow: Box<[u8]>, commit: api::Commit, page: u32 },
    Run(u64),
    Tree { object: api::Commit, page: u32 },
}
#[derive(Debug)]
enum State {
    Runs { workflow: Box<[u8]>, commit: api::Commit, page: u32, runs: List<api::Run>, count: u32 },
    Tree { object: api::Commit, page: u32, entries: List<api::TreeEntry>, count: u32 },
    Run { expected: u64, jobs: List<api::RunJob> },
    Jobs { run: api::Run, jobs: List<api::RunJob> },
    Steps { run: api::Run, jobs: List<api::RunJob>, position: u32 },
    Ended,
}
#[derive(Debug)]
pub(super) struct Plan {
    state: State,
    row: Repository,
    size: u64,
}
impl Plan {
    pub(super) fn new(mode: Mode, row: Repository, bounds: Bounds) -> Self {
        let state = match mode {
            Mode::Runs { workflow, commit, page } => State::Runs {
                workflow,
                commit,
                page,
                runs: List::with_capacity(capacity(bounds, crate::measure::RUN_BYTES).min(bounds.page_size)),
                count: 0,
            },
            Mode::Tree { object, page } => State::Tree {
                object,
                page,
                entries: List::with_capacity(capacity(bounds, crate::measure::TREE_ENTRY_BYTES).min(bounds.page_size)),
                count: 0,
            },
            Mode::Run(expected) => {
                State::Run { expected, jobs: List::with_capacity(capacity(bounds, crate::measure::RUN_JOB_BYTES)) }
            }
        };
        Self { state, row, size: 0 }
    }
    pub(super) fn begin(&self, bounds: Bounds) -> Step {
        let at = self.row.at();
        Step::send(match &self.state {
            State::Runs { workflow, commit, page, .. } => r::Request::WorkflowRuns {
                at,
                workflow: workflow.clone(),
                head_sha: self.row.id(*commit),
                page: r::Page { page: Some(*page), limit: Some(bounds.page_size) },
            },
            State::Run { expected, .. } => r::Request::Run { at, run: *expected },
            State::Tree { object, page, .. } => r::Request::Tree {
                at,
                object: self.row.id(*object),
                page: r::Page { page: Some(*page), limit: Some(bounds.page_size) },
            },
            State::Jobs { .. } | State::Steps { .. } | State::Ended => unreachable!("begin before an exchange"),
        })
    }
    fn add(&mut self, size: u64, bounds: Bounds) -> Result<(), api::Error> {
        self.size = self.size.checked_add(size).ok_or(api::Error::TooLarge)?;
        if self.size > u64::from(bounds.answer_bytes) {
            return Err(api::Error::TooLarge);
        }
        Ok(())
    }
    pub(super) fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        repository: api::Repository,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        match heard {
            Heard::Head { status, fields } => {
                if status != 200 {
                    let failure =
                        super::failure::classify(super::failure::Asked::Leaf, status, None, fields.retry_after);
                    match failure.drop {
                        super::failure::Drop::Nothing => {}
                        super::failure::Drop::Path | super::failure::Drop::Row => tables.drop_repository(repository),
                    }
                    return Err(failure.error);
                }
                Ok(None)
            }
            Heard::Failed(fault) => Err(super::failure::transport(fault)),
            Heard::Row(row) => {
                self.row(row, bounds)?;
                Ok(None)
            }
            Heard::Done(body) => match body {
                Body::Document(document) => self.done(Some(document), bounds),
                Body::Malformed(
                    crate::forgejo::decode::Malformed::Bound { .. } | crate::forgejo::decode::Malformed::TooLong { .. },
                ) => Err(api::Error::TooLarge),
                Body::Empty | Body::Raw { .. } | Body::Malformed(_) => Err(api::Error::InvalidAnswer),
            },
        }
    }
    fn row(&mut self, row: d::Row, bounds: Bounds) -> Result<(), api::Error> {
        let source = core::mem::replace(&mut self.state, State::Ended);
        self.state = match row {
            d::Row::Run(run) => match source {
                State::Runs { workflow, commit, page, mut runs, count } => {
                    let value = run_value(run, self.row.object_format)?;
                    // Forgejo filters head_sha but ignores workflow (forge.md, 9).
                    if value.workflow.as_ref() == workflow.as_ref() && value.commit == commit {
                        let size = crate::measure::run(&value)
                            .ok_or(api::Error::TooLarge)?
                            .checked_add(u64::try_from(crate::measure::RUN_BYTES).expect("run width"))
                            .ok_or(api::Error::TooLarge)?;
                        self.add(size, bounds)?;
                        runs.push(value).or(Err(api::Error::TooLarge))?;
                    }
                    State::Runs {
                        workflow,
                        commit,
                        page,
                        runs,
                        count: count.checked_add(1).ok_or(api::Error::TooLarge)?,
                    }
                }
                State::Tree { .. } | State::Run { .. } | State::Jobs { .. } | State::Steps { .. } => {
                    return Err(api::Error::InvalidAnswer);
                }
                State::Ended => unreachable!("the owner delivers no rows after Done"),
            },
            d::Row::TreeEntry(entry) => match source {
                State::Tree { object, page, mut entries, count } => {
                    let value = api::TreeEntry {
                        name: entry.path,
                        mode: entry.mode,
                        kind: entry.kind,
                        object: reading::commit(entry.sha, self.row.object_format).ok_or(api::Error::InvalidAnswer)?,
                    };
                    self.add(
                        skein_lib::len_u64(crate::measure::TREE_ENTRY_BYTES.saturating_add(value.name.len())),
                        bounds,
                    )?;
                    entries.push(value).or(Err(api::Error::TooLarge))?;
                    State::Tree { object, page, entries, count: count.checked_add(1).ok_or(api::Error::TooLarge)? }
                }
                State::Runs { .. } | State::Run { .. } | State::Jobs { .. } | State::Steps { .. } => {
                    return Err(api::Error::InvalidAnswer);
                }
                State::Ended => unreachable!("the owner delivers no rows after Done"),
            },
            d::Row::Job(job) => match source {
                State::Jobs { run, jobs } => self.job_row(run, jobs, job, bounds)?,
                State::Runs { .. } | State::Tree { .. } | State::Run { .. } | State::Steps { .. } => {
                    return Err(api::Error::InvalidAnswer);
                }
                State::Ended => unreachable!("the owner delivers no rows after Done"),
            },
            d::Row::Step(step) => match source {
                State::Steps { run, mut jobs, position } => {
                    let job = jobs.get_mut(position).expect("failed job position");
                    job.steps = job.steps.checked_add(1).ok_or(api::Error::TooLarge)?;
                    if step.status == d::StepStatus::Failure && job.step.is_none() {
                        let size = skein_lib::len_u64(step.name.len());
                        job.step = Some(api::Step { number: step.number, name: step.name });
                        self.add(size, bounds)?;
                    }
                    State::Steps { run, jobs, position }
                }
                State::Runs { .. } | State::Tree { .. } | State::Run { .. } | State::Jobs { .. } => {
                    return Err(api::Error::InvalidAnswer);
                }
                State::Ended => unreachable!("the owner delivers no rows after Done"),
            },
            d::Row::Entry { .. }
            | d::Row::Found(_)
            | d::Row::Reachable(_)
            | d::Row::Token(_)
            | d::Row::User(_)
            | d::Row::Issue(_)
            | d::Row::Comment(_)
            | d::Row::Status(_)
            | d::Row::Review(_)
            | d::Row::ReviewComment(_)
            | d::Row::Branch(_)
            | d::Row::Protection(_)
            | d::Row::Pull(_)
            | d::Row::File(_)
            | d::Row::Commit(_)
            | d::Row::Artifact(_)
            | d::Row::CommitMessage(_)
            | d::Row::Hook(_)
            | d::Row::Team(_) => return Err(api::Error::InvalidAnswer),
        };
        Ok(())
    }
    fn job_row(
        &mut self,
        run: api::Run,
        mut jobs: List<api::RunJob>,
        job: d::Job,
        bounds: Bounds,
    ) -> Result<State, api::Error> {
        if job.run_id != run.id {
            return Err(api::Error::InvalidAnswer);
        }
        let (state, conclusion) = state(job.status);
        self.add(skein_lib::len_u64(crate::measure::RUN_JOB_BYTES.saturating_add(job.name.len())), bounds)?;
        jobs.push(api::RunJob {
            steps: 0,
            id: job.id,
            name: job.name,
            attempt: job.attempt,
            state,
            conclusion,
            step: None,
        })
        .or(Err(api::Error::TooLarge))?;
        Ok(State::Jobs { run, jobs })
    }
    fn done(&mut self, document: Option<d::Document>, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        let source = core::mem::replace(&mut self.state, State::Ended);
        let step;
        self.state = match source {
            State::Runs { runs, count, .. } => match document {
                Some(d::Document::Runs { .. }) => {
                    step =
                        Step::Done(Ok(api::Answer::Runs { runs: runs.into_boxed(), more: count == bounds.page_size }));
                    State::Ended
                }
                Some(_) | None => return Err(api::Error::InvalidAnswer),
            },
            State::Tree { entries, count, .. } => match document {
                Some(d::Document::Tree { truncated }) => {
                    step = Step::Done(Ok(api::Answer::Tree {
                        entries: entries.into_boxed(),
                        more: truncated || count == bounds.page_size,
                    }));
                    State::Ended
                }
                Some(_) | None => return Err(api::Error::InvalidAnswer),
            },
            State::Run { expected, jobs } => match document {
                Some(d::Document::Run(run)) => {
                    if run.id != expected {
                        return Err(api::Error::InvalidAnswer);
                    }
                    let run = run_value(run, self.row.object_format)?;
                    self.add(crate::measure::run(&run).ok_or(api::Error::TooLarge)?, bounds)?;
                    step = Step::send(r::Request::Jobs { at: self.row.at(), run: expected });
                    State::Jobs { run, jobs }
                }
                Some(_) | None => return Err(api::Error::InvalidAnswer),
            },
            State::Jobs { run, jobs } => match document {
                Some(d::Document::Listed { .. }) => {
                    let (state, next) = self.next_job(run, jobs, 0);
                    step = next;
                    state
                }
                Some(_) | None => return Err(api::Error::InvalidAnswer),
            },
            State::Steps { run, jobs, position } => match document {
                Some(d::Document::Job(job)) => {
                    let expected = jobs.get(position).expect("failed job position");
                    if job.id != expected.id || job.attempt != expected.attempt || job.run_id != run.id {
                        return Err(api::Error::InvalidAnswer);
                    }
                    let (state, next) = self.next_job(run, jobs, position.saturating_add(1));
                    step = next;
                    state
                }
                Some(_) | None => return Err(api::Error::InvalidAnswer),
            },
            State::Ended => unreachable!("the owner delivers no document after Done"),
        };
        Ok(Some(step))
    }
    fn next_job(&self, mut run: api::Run, jobs: List<api::RunJob>, position: u32) -> (State, Step) {
        for index in position..jobs.len() {
            let job = jobs.get(index).expect("listed job");
            if job.conclusion == Some(api::Conclusion::Failure) {
                let step = Step::send(r::Request::Job { at: self.row.at(), job: job.id });
                return (State::Steps { run, jobs, position: index }, step);
            }
        }
        run.jobs = jobs.into_boxed();
        (State::Ended, Step::Done(Ok(api::Answer::Run(run))))
    }
}

fn state(status: d::StepStatus) -> (api::RunState, Option<api::Conclusion>) {
    match status {
        d::StepStatus::Unknown | d::StepStatus::Waiting | d::StepStatus::Blocked => (api::RunState::Waiting, None),
        d::StepStatus::Running => (api::RunState::Running, None),
        d::StepStatus::Success => (api::RunState::Completed, Some(api::Conclusion::Success)),
        d::StepStatus::Failure => (api::RunState::Completed, Some(api::Conclusion::Failure)),
        d::StepStatus::Cancelled => (api::RunState::Completed, Some(api::Conclusion::Cancelled)),
        d::StepStatus::Skipped => (api::RunState::Completed, Some(api::Conclusion::Skipped)),
    }
}
fn run_value(value: d::Run, format: crate::forgejo::id::ObjectFormat) -> Result<api::Run, api::Error> {
    if value.workflow.is_empty() || value.trigger_actor == 0 || value.html_url.is_empty() {
        return Err(api::Error::InvalidAnswer);
    }
    let (state, conclusion) = state(value.status);
    Ok(api::Run {
        id: value.id,
        number: value.index_in_repo,
        workflow: value.workflow,
        commit: reading::commit(value.commit_sha, format).ok_or(api::Error::InvalidAnswer)?,
        event: if value.event.as_ref() == b"workflow_dispatch" { api::Event::Dispatch } else { api::Event::Other },
        actor: value.trigger_actor,
        state,
        conclusion,
        url: value.html_url,
        created: Time::from_nanos(value.created_at.as_nanos()),
        jobs: Box::new([]),
    })
}

fn capacity(bounds: Bounds, width: usize) -> u32 {
    bounds
        .rows
        .min(bounds.answer_bytes.checked_div(u32::try_from(width).expect("list width fits")).expect("nonzero width"))
}
