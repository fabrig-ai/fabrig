//! Raw file retention is an instruction on the dialect's exchange.
use super::{Bounds, Plan, Read, Request, Response, Stage, Step, api};
use crate::api::Excess;
use crate::github::plans::plan::{Exchange, Retention};
use alloc::boxed::Box;
impl Plan {
    pub(super) fn directory_done(&mut self, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        if self.stage != Stage::Directory {
            return Err(api::Error::InvalidAnswer);
        }
        let Read::File { head, path, .. } = &self.read else { return Err(api::Error::InvalidAnswer) };
        let answer = self.directory.as_mut().ok_or(api::Error::InvalidAnswer)?.answer(*head, path.clone());
        self.terminal(answer, bounds)
    }

    pub(super) fn send_raw(&mut self, request: Request, retention: Retention) -> Step {
        self.response = Response::Head;
        self.count = 0;
        Step::Send(Exchange {
            request,
            retention: Some(retention),
            comparison: crate::github::plans::plan::Comparison::Full,
        })
    }
    pub(super) fn file_initial(&mut self) -> Result<Step, api::Error> {
        let Read::File { head, path, .. } = &self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let request =
            Request::Raw { owner: row.owner.clone(), name: row.name.clone(), path: path.clone(), commit: *head };
        self.stage = Stage::Raw;
        Ok(self.send_raw(request, Retention::Head { bytes: self.raw_cap }))
    }
    pub(super) fn raw_done(
        &mut self,
        bytes: Box<[u8]>,
        more: bool,
        bounds: Bounds,
    ) -> Result<Option<Step>, api::Error> {
        if let Read::Job { .. } = self.read {
            return self.job_raw(bytes, more, bounds);
        }
        if self.stage == Stage::Before || self.stage == Stage::After {
            if more {
                return Err(api::Error::TooLarge(Excess {
                    bound: api::Bound::AnswerBytes,
                    limit: u64::from(bounds.answer_bytes),
                    size: self.size.saturating_add(skein_lib::len_u64(bytes.len())).saturating_add(1),
                    exact: false,
                }));
            }
            if skein_lib::len_u64(bytes.len()) > u64::from(self.raw_cap) {
                return Err(api::Error::TooLarge(Excess {
                    bound: api::Bound::AnswerBytes,
                    limit: u64::from(bounds.answer_bytes),
                    size: self.size.saturating_add(skein_lib::len_u64(bytes.len())),
                    exact: false,
                }));
            }
            self.keep_bytes(0, &[&bytes], bounds)?;
            return self.side_done(Some(bytes), bounds);
        }
        if self.stage != Stage::Raw {
            return Err(api::Error::InvalidAnswer);
        }
        let Read::File { head, path, .. } = &self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        if skein_lib::len_u64(bytes.len()) > u64::from(self.raw_cap) {
            let size = self.size.saturating_add(skein_lib::len_u64(bytes.len()));
            let excess = if size > u64::from(bounds.answer_bytes) {
                Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size, exact: !more }
            } else {
                Excess {
                    bound: api::Bound::Forge,
                    limit: u64::from(self.raw_cap),
                    size: skein_lib::len_u64(bytes.len()),
                    exact: !more,
                }
            };
            return Err(api::Error::TooLarge(excess));
        }
        if path.is_empty() {
            return Err(api::Error::InvalidAnswer);
        }
        let answer = api::Answer::File { head: *head, path: path.clone(), bytes, truncated: more };
        self.terminal(answer, bounds)
    }
}

#[derive(Debug)]
pub(super) struct State {
    descriptors: skein_lib::Queue<crate::github::document::File>,
    files: skein_lib::List<api::File>,
    commits: skein_lib::List<api::Commit>,
    current: Option<api::File>,
    before: api::Commit,
    after: api::Commit,
    contains: bool,
    previous: Option<Box<[u8]>>,
    status: Option<crate::github::document::FileStatus>,
    more: bool,
    descriptor_size: u64,
}
impl State {
    pub(super) fn new() -> Self {
        Self {
            descriptors: skein_lib::Queue::with_capacity(0),
            files: skein_lib::List::with_capacity(0),
            commits: skein_lib::List::with_capacity(0),
            current: None,
            before: [0; 32],
            after: [0; 32],
            contains: false,
            previous: None,
            status: None,
            more: false,
            descriptor_size: 0,
        }
    }
}
impl Plan {
    pub(super) fn files_admission(&mut self, bounds: Bounds) -> Result<(), api::Error> {
        let commits = super::size::capacity(bounds, crate::measure::COMMIT_BYTES)?;
        let files = super::size::capacity(bounds, crate::measure::FILE_BYTES)?;
        let (capacity, commits) = if let Read::PullFiles { .. } = self.read { (files, 0) } else { (commits, commits) };
        self.limit = super::size::page_size(bounds, capacity)?;
        self.files.descriptors = skein_lib::Queue::with_capacity(files);
        self.files.files = skein_lib::List::with_capacity(files);
        self.files.commits = skein_lib::List::with_capacity(commits);
        Ok(())
    }
    pub(super) fn compare_initial(&mut self) -> Result<Step, api::Error> {
        let Read::Compare { before, after } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        self.files.before = before;
        self.files.after = after;
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let request =
            Request::Compare { owner: row.owner.clone(), name: row.name.clone(), before, after, per_page: self.limit };
        self.stage = Stage::Compare;
        Ok(self.send(request))
    }
    pub(super) fn file_row(&mut self, row: crate::github::document::Row, bounds: Bounds) -> Result<(), api::Error> {
        if self.stage != Stage::Compare && self.stage != Stage::FileListing {
            return Err(api::Error::InvalidAnswer);
        }
        match row {
            crate::github::document::Row::Commit(commit) => {
                if self.stage != Stage::Compare {
                    return Err(api::Error::InvalidAnswer);
                }
                if self.files.commits.len() >= self.limit || self.files.commits.len() >= bounds.rows {
                    return Err(api::Error::TooLarge(crate::measure::capacity_excess(
                        self.files.commits.len(),
                        bounds.rows,
                        bounds.answer_bytes,
                        crate::measure::COMMIT_BYTES,
                    )));
                }
                self.keep_bytes(32, &[], bounds)?;
                if self.files.commits.push(commit).is_err() {
                    return Err(api::Error::TooLarge(crate::measure::capacity_excess(
                        self.files.commits.capacity(),
                        bounds.rows,
                        bounds.answer_bytes,
                        crate::measure::COMMIT_BYTES,
                    )));
                }
            }
            crate::github::document::Row::File(file) => {
                if self.files.descriptors.len() >= bounds.rows
                    || (self.stage == Stage::Compare && self.files.descriptors.len() >= 299)
                {
                    return Err(api::Error::TooLarge(Excess::at_least(
                        if self.files.descriptors.len() >= bounds.rows { api::Bound::Rows } else { api::Bound::Forge },
                        u64::from(if self.files.descriptors.len() >= bounds.rows { bounds.rows } else { 299 }),
                        u64::from(self.files.descriptors.len()).saturating_add(1),
                    )));
                }
                if self.stage == Stage::FileListing {
                    self.page_row(bounds)?;
                    if file.status == crate::github::document::FileStatus::Renamed && file.previous.is_none() {
                        return Err(api::Error::InvalidAnswer);
                    }
                }
                let previous = match &file.previous {
                    Some(path) => path.as_ref(),
                    None => &[],
                };
                let path_bytes = skein_lib::len_u64(file.path.len());
                let previous_bytes = skein_lib::len_u64(previous.len());
                let width = path_bytes
                    .checked_add(64)
                    .ok_or(api::Error::TooLarge(Excess::unnamed()))?
                    .checked_add(previous_bytes)
                    .ok_or(api::Error::TooLarge(Excess::unnamed()))?;
                self.files.descriptor_size =
                    self.files.descriptor_size.checked_add(width).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
                if self.files.descriptor_size > u64::from(bounds.answer_bytes) {
                    return Err(api::Error::TooLarge(Excess {
                        bound: api::Bound::AnswerBytes,
                        limit: u64::from(bounds.answer_bytes),
                        size: self.files.descriptor_size,
                        exact: false,
                    }));
                }
                self.keep_bytes(64, &[&file.path], bounds)?;
                if self.files.descriptors.try_push(file).is_err() {
                    return Err(api::Error::TooLarge(crate::measure::capacity_excess(
                        self.files.descriptors.capacity(),
                        bounds.rows,
                        bounds.answer_bytes,
                        crate::measure::FILE_BYTES,
                    )));
                }
            }
            crate::github::document::Row::CommitMessage(_)
            | crate::github::document::Row::Entry { .. }
            | crate::github::document::Row::Installation(_)
            | crate::github::document::Row::Rule(_)
            | crate::github::document::Row::Log(_)
            | crate::github::document::Row::Reachable(_)
            | crate::github::document::Row::Branch(_)
            | crate::github::document::Row::SkippedBranch
            | crate::github::document::Row::Issue(_)
            | crate::github::document::Row::Comment(_)
            | crate::github::document::Row::Pull(_)
            | crate::github::document::Row::Context(_)
            | crate::github::document::Row::Run(_)
            | crate::github::document::Row::Collaborator(_)
            | crate::github::document::Row::Remark(_)
            | crate::github::document::Row::Review(_) => return Err(api::Error::InvalidAnswer),
        }
        Ok(())
    }
    pub(super) fn compare_done(
        &mut self,
        comparison: crate::github::document::Compare,
        bounds: Bounds,
    ) -> Result<Option<Step>, api::Error> {
        if self.stage == Stage::MergeBase {
            self.files.before = comparison.merge_base.ok_or(api::Error::InvalidAnswer)?;
            return Ok(Some(self.file_listing()?));
        }
        if self.stage != Stage::Compare {
            return Err(api::Error::InvalidAnswer);
        }
        if comparison.total_commits > u64::from(self.files.commits.len()) {
            return Err(api::Error::TooLarge(Excess::exact(
                if comparison.total_commits > u64::from(bounds.rows) { api::Bound::Rows } else { api::Bound::Forge },
                if comparison.total_commits > u64::from(bounds.rows) {
                    u64::from(bounds.rows)
                } else {
                    u64::from(self.files.commits.len())
                },
                comparison.total_commits,
            )));
        }
        self.files.contains = comparison.behind_by == 0;
        self.next_file(bounds)
    }
    fn next_file(&mut self, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        if let Some(file) = self.files.descriptors.pop() {
            self.files.status = None;
            self.files.previous = None;
            if let Read::PullFiles { .. } = self.read {
                self.files.status = Some(file.status);
                if file.status == crate::github::document::FileStatus::Renamed {
                    self.files.previous = file.previous;
                }
            }
            self.files.current = Some(api::File { path: file.path, before: None, after: None });
            return self.side_request(self.files.status != Some(crate::github::document::FileStatus::Added), bounds);
        }
        let files = core::mem::replace(&mut self.files.files, skein_lib::List::with_capacity(0)).into_boxed();
        let commits = core::mem::replace(&mut self.files.commits, skein_lib::List::with_capacity(0)).into_boxed();
        if let Read::Compare { before, after } = self.read {
            self.terminal(
                api::Answer::Compare { before, after, contains_before: self.files.contains, files, commits },
                bounds,
            )
        } else if let Read::PullFiles { head, .. } = self.read {
            self.terminal(api::Answer::PullFiles { head, files, more: self.files.more }, bounds)
        } else {
            Err(api::Error::InvalidAnswer)
        }
    }
    fn side_request(&mut self, before: bool, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        let file = self.files.current.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let request = Request::Raw {
            owner: row.owner.clone(),
            name: row.name.clone(),
            path: if before {
                match &self.files.previous {
                    Some(path) => path.clone(),
                    None => file.path.clone(),
                }
            } else {
                file.path.clone()
            },
            commit: if before { self.files.before } else { self.files.after },
        };
        self.stage = if before { Stage::Before } else { Stage::After };
        self.raw_cap = u32::try_from(
            u64::from(bounds.answer_bytes).checked_sub(self.size).ok_or(api::Error::TooLarge(Excess::unnamed()))?,
        )
        .ok()
        .ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        Ok(Some(self.send_raw(request, Retention::Whole { bytes: self.raw_cap })))
    }
    pub(super) fn side_done(&mut self, bytes: Option<Box<[u8]>>, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        let file = self.files.current.as_mut().ok_or(api::Error::InvalidAnswer)?;
        if self.stage == Stage::Before {
            file.before = bytes;
            if self.files.status == Some(crate::github::document::FileStatus::Removed) {
                return self.finish_file(bounds);
            }
            return self.side_request(false, bounds);
        }
        if self.stage != Stage::After {
            return Err(api::Error::InvalidAnswer);
        }
        file.after = bytes;
        self.finish_file(bounds)
    }
    fn finish_file(&mut self, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        let file = self.files.current.take().ok_or(api::Error::InvalidAnswer)?;
        if self.files.files.push(file).is_err() {
            return Err(api::Error::TooLarge(crate::measure::capacity_excess(
                self.files.files.capacity(),
                bounds.rows,
                bounds.answer_bytes,
                crate::measure::FILE_BYTES,
            )));
        }
        self.next_file(bounds)
    }
}

impl Plan {
    pub(super) fn pull_files_initial(&mut self) -> Result<Step, api::Error> {
        let Read::PullFiles { number, .. } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let request = Request::Pull { owner: row.owner.clone(), name: row.name.clone(), number };
        self.stage = Stage::FilePull;
        Ok(self.send(request))
    }
    pub(super) fn file_pull_done(&mut self, pull: crate::github::document::Pull) -> Result<Option<Step>, api::Error> {
        let Read::PullFiles { number, head, .. } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        if self.stage != Stage::FilePull || pull.number != number {
            return Err(api::Error::InvalidAnswer);
        }
        if pull.head_commit != head {
            return Err(api::Error::Stale);
        }
        self.files.after = head;
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let request = Request::Compare {
            owner: row.owner.clone(),
            name: row.name.clone(),
            before: pull.base_commit,
            after: head,
            per_page: 1,
        };
        self.stage = Stage::MergeBase;
        let Step::Send(mut exchange) = self.send(request) else {
            return Err(api::Error::InvalidAnswer);
        };
        exchange.comparison = crate::github::plans::plan::Comparison::MergeBase;
        Ok(Some(Step::Send(exchange)))
    }
    fn file_listing(&mut self) -> Result<Step, api::Error> {
        let Read::PullFiles { number, page, .. } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        let request =
            Request::PullFiles { owner: row.owner.clone(), name: row.name.clone(), number, page, per_page: self.limit };
        self.stage = Stage::FileListing;
        Ok(self.send(request))
    }
    pub(super) fn file_listing_done(&mut self, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        if self.stage != Stage::FileListing {
            return Err(api::Error::InvalidAnswer);
        }
        self.files.more = self.listing_next()?;
        self.next_file(bounds)
    }
}
