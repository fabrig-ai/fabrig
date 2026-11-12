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
                return self.cut_files(
                    api::Error::TooLarge(Excess {
                        bound: api::Bound::AnswerBytes,
                        limit: u64::from(bounds.answer_bytes),
                        size: self.size.saturating_add(skein_lib::len_u64(bytes.len())).saturating_add(1),
                        exact: false,
                    }),
                    bounds,
                );
            }
            if skein_lib::len_u64(bytes.len()) > u64::from(self.raw_cap) {
                return self.cut_files(
                    api::Error::TooLarge(Excess {
                        bound: api::Bound::AnswerBytes,
                        limit: u64::from(bounds.answer_bytes),
                        size: self.size.saturating_add(skein_lib::len_u64(bytes.len())),
                        exact: false,
                    }),
                    bounds,
                );
            }
            if let Err(error) = self.keep_bytes(0, &[&bytes], bounds) {
                return self.cut_files(error, bounds);
            }
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
    paths: skein_lib::List<Box<[u8]>>,
    changed: u64,
    full: bool,
    current: Option<api::File>,
    before: api::Commit,
    after: api::Commit,
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
            paths: skein_lib::List::with_capacity(0),
            changed: 0,
            full: false,
            current: None,
            before: [0; 32],
            after: [0; 32],
            previous: None,
            status: None,
            more: false,
            descriptor_size: 0,
        }
    }
}
impl Plan {
    pub(super) fn files_admission(&mut self, bounds: Bounds) -> Result<(), api::Error> {
        if let Read::Compare { .. } = self.read {
            self.limit = 1;
            self.files.paths = skein_lib::List::with_capacity(bounds.rows);
        } else {
            self.limit = super::size::page_size(bounds, bounds.page_size)?;
            let files = self.limit;
            self.files.descriptors = skein_lib::Queue::with_capacity(files);
            self.files.files = skein_lib::List::with_capacity(files);
        }
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
            crate::github::document::Row::Commit(_) => {
                if self.stage != Stage::Compare {
                    return Err(api::Error::InvalidAnswer);
                }
            }
            crate::github::document::Row::File(file) => {
                if self.stage == Stage::Compare {
                    self.files.changed =
                        self.files.changed.checked_add(1).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
                    if self.files.changed >= 300 {
                        return Err(api::Error::TooLarge(Excess::at_least(api::Bound::Forge, 299, self.files.changed)));
                    }
                    let width = u64::try_from(crate::measure::STRING_BYTES)
                        .expect("path width")
                        .saturating_add(skein_lib::len_u64(file.path.len()));
                    if !self.files.full
                        && self.files.paths.len() < bounds.rows
                        && self.size.saturating_add(width) <= u64::from(bounds.answer_bytes)
                    {
                        self.size = self.size.saturating_add(width);
                        self.files.paths.push(file.path).expect("rows checked");
                    } else {
                        self.files.full = true;
                    }
                    return Ok(());
                }
                if self.files.descriptors.len() >= bounds.rows {
                    return Err(api::Error::TooLarge(Excess::at_least(
                        api::Bound::Rows,
                        u64::from(bounds.rows),
                        u64::from(self.files.descriptors.len()).saturating_add(1),
                    )));
                }
                if self.stage == Stage::FileListing {
                    self.page_row(bounds)?;
                    if let Read::PullFiles { skip, .. } = self.read
                        && self.count <= skip
                    {
                        return Ok(());
                    }
                    if file.status == crate::github::document::FileStatus::Renamed && file.previous.is_none() {
                        return Err(api::Error::InvalidAnswer);
                    }
                }
                if self.files.full {
                    return Ok(());
                }
                let path_bytes = skein_lib::len_u64(file.path.len());
                let width = path_bytes
                    .checked_add(u64::try_from(crate::measure::FILE_BYTES).expect("file width"))
                    .ok_or(api::Error::TooLarge(Excess::unnamed()))?;
                self.files.descriptor_size =
                    self.files.descriptor_size.checked_add(width).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
                if self.files.descriptor_size > u64::from(bounds.answer_bytes) {
                    if self.files.descriptors.is_empty() {
                        return Err(api::Error::TooLarge(Excess {
                            bound: api::Bound::AnswerBytes,
                            limit: u64::from(bounds.answer_bytes),
                            size: self.files.descriptor_size,
                            exact: false,
                        }));
                    }
                    self.files.full = true;
                    return Ok(());
                }
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
        let Read::Compare { before, after } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let files = core::mem::replace(&mut self.files.paths, skein_lib::List::with_capacity(0)).into_boxed();
        self.terminal(
            api::Answer::Compare {
                before,
                after,
                ahead: comparison.ahead_by,
                behind: comparison.behind_by,
                files,
                changed: self.files.changed,
            },
            bounds,
        )
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
            if let Err(error) =
                self.keep_bytes(u64::try_from(crate::measure::FILE_BYTES).expect("file width"), &[&file.path], bounds)
            {
                return self.cut_files(error, bounds);
            }
            self.files.current = Some(api::File { path: file.path, before: None, after: None });
            if let Read::PullFiles { sides: false, .. } = self.read {
                return self.finish_file(bounds);
            }
            return self.side_request(self.files.status != Some(crate::github::document::FileStatus::Added), bounds);
        }
        let files = core::mem::replace(&mut self.files.files, skein_lib::List::with_capacity(0)).into_boxed();
        if let Read::PullFiles { head, .. } = self.read {
            self.terminal(
                api::Answer::PullFiles { head, files, more: self.files.more || self.files.full, cut: self.files.full },
                bounds,
            )
        } else {
            Err(api::Error::InvalidAnswer)
        }
    }
    fn cut_files(&mut self, error: api::Error, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        if self.files.files.is_empty() {
            return Err(error);
        }
        let Read::PullFiles { head, .. } = self.read else { return Err(api::Error::InvalidAnswer) };
        self.files.current = None;
        let files = core::mem::replace(&mut self.files.files, skein_lib::List::with_capacity(0)).into_boxed();
        self.terminal(api::Answer::PullFiles { head, files, more: true, cut: true }, bounds)
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
        if let Read::PullFiles { sides: false, .. } = self.read {
            return Ok(Some(self.file_listing()?));
        }
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
