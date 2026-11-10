//! A single oldest-first commit range page (forge-github.md, 4.5).
use super::{Bounds, Plan, Read, Request, Response, Stage, Step, api, document, size};
use crate::api::Excess;
use crate::github::plans::plan::{Comparison, Exchange};
use skein_lib::{List, len_u64, usize_of};

impl Plan {
    pub(super) fn commits_admission(&mut self, bounds: Bounds) -> Result<(), api::Error> {
        let Read::Commits { page, .. } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        if page == 0 {
            return Err(api::Error::Empty);
        }
        let capacity = size::capacity(bounds, size_of::<api::CommitMessage>())?;
        self.limit = size::page_size(bounds, capacity)?;
        self.commits = List::with_capacity(self.limit);
        Ok(())
    }
    pub(super) fn commits_initial(&mut self) -> Result<Step, api::Error> {
        let Read::Commits { before, after, page } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let row = self.row.as_ref().ok_or(api::Error::InvalidAnswer)?;
        self.stage = Stage::Compare;
        self.response = Response::Head;
        Ok(Step::Send(Exchange {
            request: Request::Commits {
                owner: row.owner.clone(),
                name: row.name.clone(),
                before,
                after,
                page,
                per_page: self.limit,
            },
            retention: None,
            comparison: Comparison::Commits,
        }))
    }
    pub(super) fn commit_row(&mut self, row: document::Row, bounds: Bounds) -> Result<(), api::Error> {
        if self.stage != Stage::Compare {
            return Err(api::Error::InvalidAnswer);
        }
        let document::Row::CommitMessage(commit) = row else {
            return Err(api::Error::InvalidAnswer);
        };
        if commit.parents.len() > usize_of(bounds.rows) {
            return Err(api::Error::TooLarge(Excess::exact(
                api::Bound::Rows,
                u64::from(bounds.rows),
                len_u64(commit.parents.len()),
            )));
        }
        let parents = len_u64(commit.parents.len()).checked_mul(32).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        let width = u64::try_from(size_of::<api::CommitMessage>())
            .expect("commit width")
            .checked_add(parents)
            .ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        self.keep_bytes(width, &[&commit.message], bounds)?;
        self.commits.push(commit).or(Err(api::Error::TooLarge(crate::measure::capacity_excess(
            self.commits.capacity(),
            bounds.rows,
            bounds.answer_bytes,
            crate::measure::COMMIT_BYTES,
        ))))
    }
    pub(super) fn commits_done(
        &mut self,
        comparison: document::Compare,
        bounds: Bounds,
    ) -> Result<Option<Step>, api::Error> {
        if self.stage != Stage::Compare {
            return Err(api::Error::InvalidAnswer);
        }
        let Read::Commits { page, .. } = self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let Response::Body { next } = self.response else {
            return Err(api::Error::InvalidAnswer);
        };
        let skip = u64::from(page.checked_sub(1).ok_or(api::Error::Empty)?)
            .checked_mul(u64::from(self.limit))
            .ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        let end = skip.checked_add(u64::from(self.commits.len())).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        if (!next && comparison.total_commits != end) || (next && comparison.total_commits <= end) {
            return Err(api::Error::InvalidAnswer);
        }
        let commits = core::mem::replace(&mut self.commits, List::with_capacity(0)).into_boxed();
        self.terminal(api::Answer::Commits { commits, more: next }, bounds)
    }
}
