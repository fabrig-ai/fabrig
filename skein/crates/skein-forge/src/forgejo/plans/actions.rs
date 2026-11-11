//! Actions status job attempts and verified logs (skein/docs/design/forge/forgejo.md, 1.2).
use crate::api;
use crate::forgejo::plans::{
    Body, Heard,
    plan::{Bounds, Exchange, Step},
    reading,
    tables::{Repository, Tables},
};
use crate::forgejo::{document as d, header, request as r};
use alloc::boxed::Box;
use skein_lib::List;
#[derive(Debug)]
struct Checks {
    head: api::Commit,
    shape: Shape,
    statuses: List<api::Status>,
    position: u32,
    size: u64,
    resolve_jobs: bool,
}
#[derive(Debug)]
enum Shape {
    Checks,
    Verdict(api::Ci),
    Statuses { ci: api::Ci, more: bool },
}
#[derive(Debug)]
enum State {
    Listing { checks: Checks, page: u32 },
    RunLookup { checks: Checks, run_number: u64, run: Option<u64>, missing: bool },
    JobsLookup { checks: Checks, run_number: u64, run: u64, missing: bool },
    Run { attempt: api::JobAttempt, max: u32, part: api::LogPart },
    Job { attempt: api::JobAttempt, max: u32, part: api::LogPart, step: Option<api::Step> },
    Log { attempt: api::JobAttempt, max: u32, part: api::LogPart, step: Option<api::Step>, truncated: bool },
    Ended,
}
#[derive(Debug)]
pub struct Plan {
    state: State,
    row: Repository,
    limit: u32,
    count: u32,
}
impl Plan {
    pub fn new(read: api::Read, row: Repository, bounds: Bounds) -> Result<Self, api::Error> {
        let state = match read {
            api::Read::Checks { commit } => Self::listing(commit, Shape::Checks, 1, bounds, true),
            api::Read::Statuses { commit, page } => {
                Self::listing(commit, Shape::Statuses { ci: api::Ci::None, more: false }, page, bounds, true)
            }
            api::Read::Job { attempt, max_bytes, part } => State::Run { attempt, max: max_bytes, part },
            api::Read::Runs { .. }
            | api::Read::Run { .. }
            | api::Read::Tree { .. }
            | api::Read::Artifact { .. }
            | api::Read::Commits { .. }
            | api::Read::Locate { .. }
            | api::Read::Items { .. }
            | api::Read::Item { .. }
            | api::Read::Pull { .. }
            | api::Read::PullFor { .. }
            | api::Read::Reviews { .. }
            | api::Read::Remarks { .. }
            | api::Read::Branch { .. }
            | api::Read::Branches { .. }
            | api::Read::Protection { .. }
            | api::Read::Settings
            | api::Read::Collaborators { .. }
            | api::Read::Permission { .. }
            | api::Read::PullFiles { .. }
            | api::Read::Compare { .. }
            | api::Read::File { .. } => return Err(api::Error::InvalidAnswer),
        };
        Ok(Self { state, row, limit: bounds.page_size, count: 0 })
    }
    fn listing(head: api::Commit, shape: Shape, page: u32, bounds: Bounds, resolve_jobs: bool) -> State {
        let room = bounds
            .answer_bytes
            .checked_div(u32::try_from(crate::measure::STATUS_BYTES).expect("status width"))
            .expect("nonzero width");
        let capacity = match shape {
            Shape::Checks => bounds.rows.min(room),
            Shape::Verdict(_) => 0,
            Shape::Statuses { .. } => bounds.page_size.min(bounds.rows).min(room),
        };
        let checks =
            Checks { head, shape, statuses: List::with_capacity(capacity), position: 0, size: 0, resolve_jobs };
        State::Listing { checks, page }
    }
    pub fn statuses(commit: api::Commit, row: Repository, bounds: Bounds) -> Result<Self, api::Error> {
        Ok(Self {
            state: Self::listing(commit, Shape::Verdict(api::Ci::None), 1, bounds, false),
            row,
            limit: bounds.page_size,
            count: 0,
        })
    }
    pub fn begin(&mut self) -> Step {
        let request = match &self.state {
            State::Listing { checks, page } => r::Request::CombinedStatus {
                at: self.row.at(),
                commit: self.row.id(checks.head),
                page: r::Page { page: Some(*page), limit: Some(self.limit) },
            },
            State::Run { attempt, .. } => r::Request::Run { at: self.row.at(), run: attempt.run },
            State::RunLookup { .. }
            | State::JobsLookup { .. }
            | State::Job { .. }
            | State::Log { .. }
            | State::Ended => unreachable!("actions begins with statuses or a run"),
        };
        self.send(request)
    }
    fn send(&mut self, request: r::Request) -> Step {
        self.count = 0;
        Step::send(request)
    }
    pub fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        repository: api::Repository,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        let state = core::mem::replace(&mut self.state, State::Ended);
        let (state, step) = match heard {
            Heard::Head { status, fields } => (Self::head(state, status, fields, repository, tables)?, None),
            Heard::Row(row) => (self.row(state, row, bounds, repository.forge, tables)?, None),
            Heard::Done(body) => self.done(state, body)?,
            Heard::Failed(failure) => return Err(crate::forgejo::plans::failure::transport(failure)),
        };
        self.state = state;
        Ok(step)
    }
    fn head(
        mut state: State,
        status: u16,
        fields: super::Fields,
        repository: api::Repository,
        tables: &mut Tables,
    ) -> Result<State, api::Error> {
        match &mut state {
            State::RunLookup { missing, .. } | State::JobsLookup { missing, .. } => {
                *missing = status == 404;
                if *missing {
                    return Ok(state);
                }
            }
            State::Run { .. } | State::Job { .. } => {
                if status == 404 {
                    return Err(api::Error::MissingJob);
                }
            }
            State::Log { truncated, part, max, .. } => {
                if status == 404 {
                    return Err(api::Error::MissingJob);
                }
                if let Some(range) = fields.content_range {
                    *truncated = match part {
                        api::LogPart::End => range.first > 0,
                        api::LogPart::Start => match range.total {
                            Some(total) => total > range.end.min(u64::from(*max)),
                            None => false,
                        },
                    };
                }
                if status == 206 {
                    return Ok(state);
                }
            }
            State::Listing { .. } => {}
            State::Ended => unreachable!("owner stops feeding an ended actions plan"),
        }
        if status == 404 || (300..400).contains(&status) {
            tables.drop_repository(repository);
        }
        if status != 200 {
            return Err(crate::forgejo::plans::failure::read(status, &fields));
        }
        Ok(state)
    }
    #[expect(clippy::too_many_lines, reason = "row dispatch names every decoded row and receiving state")]
    fn row(
        &mut self,
        state: State,
        row: d::Row,
        bounds: Bounds,
        forge: u16,
        tables: &mut Tables,
    ) -> Result<State, api::Error> {
        self.count = self.count.checked_add(1).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
        match row {
            d::Row::Status(status) => match state {
                State::Listing { mut checks, page } => {
                    if self.count > self.limit {
                        return Err(api::Error::TooLarge(api::Excess::at_least(
                            api::Bound::Forge,
                            u64::from(self.limit),
                            u64::from(self.count),
                        )));
                    }
                    if let Some(user) = &status.creator {
                        tables.keep_user(forge, user.id, user.login.clone());
                    }
                    let value = reading::status(status, &self.row.html_url)?;
                    if let Shape::Verdict(ci) = &mut checks.shape {
                        *ci = reading::fold_ci(*ci, value.check);
                        return Ok(State::Listing { checks, page });
                    }
                    let answer = api::Answer::Checks(Box::new([value.clone()]));
                    checks.size = checks
                        .size
                        .checked_add(crate::forgejo::plans::reads::size::answer(&answer, bounds.rows)?)
                        .ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
                    if checks.size > u64::from(bounds.answer_bytes) {
                        return Err(api::Error::TooLarge(api::Excess {
                            bound: api::Bound::AnswerBytes,
                            limit: u64::from(bounds.answer_bytes),
                            size: checks.size,
                            exact: false,
                        }));
                    }
                    match checks.statuses.push(value) {
                        Ok(()) => Ok(State::Listing { checks, page }),
                        Err(_) => Err(api::Error::TooLarge(api::Excess::unnamed())),
                    }
                }
                State::RunLookup { .. }
                | State::JobsLookup { .. }
                | State::Run { .. }
                | State::Job { .. }
                | State::Log { .. } => Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("owner stops feeding an ended actions plan"),
            },
            d::Row::Run(value) => match state {
                State::RunLookup { checks, run_number, mut run, missing } => {
                    if self.count > 1 {
                        return Err(api::Error::TooLarge(api::Excess::at_least(
                            api::Bound::Forge,
                            u64::from(1_u32),
                            u64::from(self.count),
                        )));
                    }
                    let commit =
                        reading::commit(value.commit_sha, self.row.object_format).ok_or(api::Error::InvalidAnswer)?;
                    if commit == checks.head && value.index_in_repo == run_number {
                        run = Some(value.id);
                    }
                    Ok(State::RunLookup { checks, run_number, run, missing })
                }
                State::Listing { .. }
                | State::JobsLookup { .. }
                | State::Run { .. }
                | State::Job { .. }
                | State::Log { .. } => Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("owner stops feeding an ended actions plan"),
            },
            d::Row::Job(job) => match state {
                State::JobsLookup { mut checks, run_number, run, missing } => {
                    for position in 0..checks.statuses.len() {
                        let status = checks.statuses.get_mut(position).expect("status position");
                        if header::job_link(&status.url, &self.row.html_url)
                            == Some((run_number, self.count.saturating_sub(1)))
                            && job.run_id == run
                        {
                            status.job = Some(api::JobAttempt {
                                head: checks.head,
                                run: job.run_id,
                                job: job.id,
                                attempt: job.attempt,
                            });
                        }
                    }
                    Ok(State::JobsLookup { checks, run_number, run, missing })
                }
                State::Listing { .. }
                | State::RunLookup { .. }
                | State::Run { .. }
                | State::Job { .. }
                | State::Log { .. } => Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("owner stops feeding an ended actions plan"),
            },
            d::Row::Step(value) => match state {
                State::Job { attempt, max, part, mut step } => {
                    if step.is_none() && value.status == d::StepStatus::Failure {
                        step = Some(api::Step { number: value.number, name: value.name });
                    }
                    Ok(State::Job { attempt, max, part, step })
                }
                State::Listing { .. }
                | State::RunLookup { .. }
                | State::JobsLookup { .. }
                | State::Run { .. }
                | State::Log { .. } => Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("owner stops feeding an ended actions plan"),
            },
            d::Row::TreeEntry(_)
            | d::Row::Found(_)
            | d::Row::Reachable(_)
            | d::Row::Token(_)
            | d::Row::User(_)
            | d::Row::Issue(_)
            | d::Row::Comment(_)
            | d::Row::Review(_)
            | d::Row::ReviewComment(_)
            | d::Row::Pull(_)
            | d::Row::Branch(_)
            | d::Row::Protection(_)
            | d::Row::Entry { .. }
            | d::Row::File(_)
            | d::Row::Hook(_)
            | d::Row::Team(_)
            | d::Row::Commit(_)
            | d::Row::Artifact(_)
            | d::Row::CommitMessage(_) => Err(api::Error::InvalidAnswer),
        }
    }
    #[expect(
        clippy::too_many_lines,
        clippy::manual_map,
        reason = "each exchange consumes its terminal document and maps optional step numbers explicitly"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        clippy::manual_let_else,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn done(&mut self, state: State, body: Body) -> Result<(State, Option<Step>), api::Error> {
        let state = match state {
            State::RunLookup { checks, missing: true, .. } | State::JobsLookup { checks, missing: true, .. } => {
                return self.next_check(checks);
            }
            state @ (State::Listing { .. }
            | State::RunLookup { .. }
            | State::JobsLookup { .. }
            | State::Run { .. }
            | State::Job { .. }
            | State::Log { .. }) => state,
            State::Ended => unreachable!("owner stops feeding an ended actions plan"),
        };
        let document = match body {
            Body::Document(document) => document,
            Body::Raw { bytes, more } => {
                return match state {
                    State::Log { attempt, max, part, step, truncated } => {
                        let cap = skein_lib::usize_of(max);
                        let cut = bytes.len() > cap;
                        let log = match part {
                            api::LogPart::Start if cut => {
                                skein_lib::bytes::copy_of(bytes.get(..cap).expect("head within bytes"))
                            }
                            api::LogPart::End | api::LogPart::Start => bytes,
                        };
                        Ok((
                            State::Ended,
                            Some(Step::Done(Ok(api::Answer::Job {
                                attempt,
                                step,
                                log,
                                truncated: more || truncated || cut,
                            }))),
                        ))
                    }
                    State::Listing { .. }
                    | State::RunLookup { .. }
                    | State::JobsLookup { .. }
                    | State::Run { .. }
                    | State::Job { .. } => Err(api::Error::InvalidAnswer),
                    State::Ended => unreachable!("owner stops feeding an ended actions plan"),
                };
            }
            Body::Malformed(crate::forgejo::decode::Malformed::Bound { field }) => {
                return Err(if field == b"log" {
                    api::Error::InvalidAnswer
                } else {
                    api::Error::TooLarge(api::Excess::unnamed())
                });
            }
            Body::Empty | Body::Malformed(_) => return Err(api::Error::InvalidAnswer),
        };
        match state {
            State::Listing { mut checks, mut page } => {
                let combined = match document {
                    d::Document::CombinedStatus(combined) => combined,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                checks.shape = match checks.shape {
                    Shape::Statuses { .. } => Shape::Statuses {
                        ci: reading::ci(combined.state, combined.total_count),
                        more: self.count == self.limit,
                    },
                    Shape::Checks | Shape::Verdict(_) => {
                        if self.count == self.limit {
                            page = page.checked_add(1).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
                            let step = self.send(r::Request::CombinedStatus {
                                at: self.row.at(),
                                commit: self.row.id(checks.head),
                                page: r::Page { page: Some(page), limit: Some(self.limit) },
                            });
                            return Ok((State::Listing { checks, page }, Some(step)));
                        }
                        match checks.shape {
                            Shape::Verdict(_) if page == 1 && combined.state != Some(d::StatusState::Warning) => {
                                Shape::Verdict(reading::ci(combined.state, combined.total_count))
                            }
                            Shape::Checks | Shape::Verdict(_) => checks.shape,
                            Shape::Statuses { .. } => unreachable!("listing shape"),
                        }
                    }
                };
                self.next_check(checks)
            }
            State::RunLookup { checks, run_number, run, .. } => {
                match document {
                    d::Document::Tree { .. } | d::Document::Runs { .. } => {}
                    _ => return Err(api::Error::InvalidAnswer),
                }
                match run {
                    Some(run) => {
                        let step = self.send(r::Request::Jobs { at: self.row.at(), run });
                        Ok((State::JobsLookup { checks, run_number, run, missing: false }, Some(step)))
                    }
                    None => self.next_check(checks),
                }
            }
            State::JobsLookup { checks, .. } => {
                match document {
                    d::Document::Listed { .. } => {}
                    _ => return Err(api::Error::InvalidAnswer),
                }
                self.next_check(checks)
            }
            State::Run { attempt, max, part } => {
                let run = match document {
                    d::Document::Run(run) => run,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                let head = reading::commit(run.commit_sha, self.row.object_format).ok_or(api::Error::InvalidAnswer)?;
                if head != attempt.head || run.id != attempt.run {
                    return Err(api::Error::MissingJob);
                }
                let sent = self.send(r::Request::Job { at: self.row.at(), job: attempt.job });
                Ok((State::Job { attempt, max, part, step: None }, Some(sent)))
            }
            State::Job { attempt, max, part, step } => {
                let job = match document {
                    d::Document::Job(job) => job,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                if job.id != attempt.job || job.run_id != attempt.run || job.attempt != attempt.attempt {
                    return Err(api::Error::MissingJob);
                }
                let sent = Step::Send(Exchange {
                    request: r::Request::JobLog {
                        at: self.row.at(),
                        job: attempt.job,
                        attempt: attempt.attempt,
                        step: match part {
                            api::LogPart::End => match &step {
                                Some(step) => Some(step.number),
                                None => None,
                            },
                            api::LogPart::Start => None,
                        },
                    },
                    range: Some(match part {
                        api::LogPart::End => crate::forgejo::plans::plan::Range::Tail(max),
                        api::LogPart::Start => crate::forgejo::plans::plan::Range::Head(max),
                    }),
                });
                Ok((State::Log { attempt, max, part, step, truncated: false }, Some(sent)))
            }
            State::Log { .. } => Err(api::Error::InvalidAnswer),
            State::Ended => unreachable!("owner stops feeding an ended actions plan"),
        }
    }
    fn next_check(&mut self, mut checks: Checks) -> Result<(State, Option<Step>), api::Error> {
        for _ in 0..checks.statuses.len().saturating_add(1) {
            if !checks.resolve_jobs || checks.position == checks.statuses.len() {
                let statuses = checks.statuses.into_boxed();
                let answer = match checks.shape {
                    Shape::Checks => api::Answer::Checks(statuses),
                    Shape::Verdict(ci) => api::Answer::Statuses { ci, statuses, more: false },
                    Shape::Statuses { ci, more } => api::Answer::Statuses { ci, statuses, more },
                };
                return Ok((State::Ended, Some(Step::Done(Ok(answer)))));
            }
            let position = checks.position;
            let status = checks.statuses.get(position).expect("check position");
            checks.position = checks.position.saturating_add(1);
            if let Some((run_number, _)) = header::job_link(&status.url, &self.row.html_url) {
                let mut seen = false;
                for prior in checks.statuses.as_slice().get(..skein_lib::usize_of(position)).expect("prior statuses") {
                    if let Some((number, _)) = header::job_link(&prior.url, &self.row.html_url) {
                        seen |= number == run_number;
                    }
                }
                if seen {
                    continue;
                }
                let step = self.send(r::Request::Runs {
                    at: self.row.at(),
                    head_sha: self.row.id(checks.head),
                    run_number,
                    page: r::Page { page: Some(1), limit: Some(1) },
                });
                return Ok((State::RunLookup { checks, run_number, run: None, missing: false }, Some(step)));
            }
        }
        Err(api::Error::InvalidAnswer)
    }
    #[must_use]
    pub fn raw_room(&self, answer_bytes: u32) -> u32 {
        let name = match &self.state {
            State::Job { step, .. } | State::Log { step, .. } => match step {
                Some(step) => step.name.len(),
                None => 0,
            },
            State::Listing { .. }
            | State::RunLookup { .. }
            | State::JobsLookup { .. }
            | State::Run { .. }
            | State::Ended => 0,
        };
        answer_bytes.saturating_sub(u32::try_from(name).expect("Bounded step name"))
    }
}
