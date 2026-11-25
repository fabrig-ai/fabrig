//! Pages a newest-first Forgejo comparison in oldest-first order
//! (skein/docs/design/forge/forgejo-operations.md, 1).
use super::{
    Body, Heard,
    plan::{Bounds, Step},
    reading,
    tables::{Repository, Tables},
};
use crate::api::Excess;
use crate::{
    api,
    forgejo::{document as d, request::Request},
};
use skein_lib::{List, Queue, len_u64, usize_of};

#[derive(Debug)]
pub(super) struct Plan {
    row: Repository,
    before: api::Commit,
    after: api::Commit,
    phase: Phase,
    commits: Queue<api::CommitMessage>,
    held: u64,
    given: u64,
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Oldest,
    Count { offset: u64 },
    Page { total: u64, start: u64, len: u32, more: bool },
}

impl Plan {
    pub(super) fn new(
        before: api::Commit,
        after: api::Commit,
        page: u32,
        row: Repository,
        bounds: Bounds,
    ) -> Result<Self, api::Error> {
        let index = page.checked_sub(1).ok_or(api::Error::Empty)?;
        let offset =
            u64::from(index).checked_mul(u64::from(bounds.page_size)).expect("product of two u32 values fits in u64");
        let phase = if index == 0 { Phase::Oldest } else { Phase::Count { offset } };
        let slots = bounds
            .answer_bytes
            .checked_div(u32::try_from(size_of::<api::CommitMessage>()).expect("commit width"))
            .expect("nonzero width");
        let capacity = bounds.page_size.min(bounds.rows).min(slots);
        Ok(Self { row, before, after, phase, commits: Queue::with_capacity(capacity), held: 0, given: 0 })
    }
    pub(super) fn begin(&self) -> Step {
        let at = self.row.at();
        let before = self.row.id(self.before);
        let after = self.row.id(self.after);
        let request = match self.phase {
            Phase::Count { .. } => Request::Compare { at, before, after, files: false },
            Phase::Oldest | Phase::Page { .. } => Request::Commits { at, before, after },
        };
        Step::send(request)
    }

    pub(super) fn after(&self) -> u64 {
        match self.phase {
            Phase::Page { start, .. } => start,
            Phase::Oldest | Phase::Count { .. } => 0,
        }
    }

    pub(super) fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        repository: api::Repository,
        tables: &mut Tables,
        cut: &mut bool,
    ) -> Result<Option<Step>, api::Error> {
        match heard {
            Heard::Head { status, fields } => {
                if status != 200 {
                    return Err(super::failure::refused(
                        super::failure::Asked::Leaf,
                        status,
                        None,
                        fields.retry_after,
                        repository,
                        tables,
                    ));
                }
                Ok(None)
            }
            Heard::Failed(fault) => Err(super::failure::transport(fault)),
            Heard::Row(row) => {
                match self.phase {
                    Phase::Count { .. } => {} // The count probe keeps no row.
                    Phase::Oldest => self.received(row, bounds)?,
                    Phase::Page { len, .. } => {
                        self.received(row, bounds)?;
                        *cut = self.given == u64::from(len);
                    }
                }
                Ok(None)
            }
            Heard::Done(body) => Ok(Some(self.done(body, bounds)?)),
        }
    }

    fn received(&mut self, row: d::Row, bounds: Bounds) -> Result<(), api::Error> {
        let value = match row {
            d::Row::CommitMessage(value) => value,
            d::Row::Entry { .. }
            | d::Row::Artifact(_)
            | d::Row::Reachable(_)
            | d::Row::Found(_)
            | d::Row::Hook(_)
            | d::Row::Token(_)
            | d::Row::Team(_)
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
            | d::Row::TreeEntry(_)
            | d::Row::Run(_)
            | d::Row::Job(_)
            | d::Row::Step(_) => return Err(api::Error::InvalidAnswer),
        };
        let format = self.row.object_format;
        let id = reading::commit(value.id, format).ok_or(api::Error::InvalidAnswer)?;
        if value.parents.len() > usize_of(bounds.rows) {
            return Err(api::Error::TooLarge(Excess::exact(
                api::Bound::Rows,
                u64::from(bounds.rows),
                len_u64(value.parents.len()),
            )));
        }
        let mut parents =
            List::with_capacity(u32::try_from(value.parents.len()).or(Err(api::Error::TooLarge(Excess::unnamed())))?);
        for parent in value.parents {
            parents.push(reading::commit(parent, format).ok_or(api::Error::InvalidAnswer)?).expect("one parent per id");
        }
        let value = api::CommitMessage { id, parents: parents.into_boxed(), message: value.message };
        let size = commit_bytes(&value).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        self.given = self.given.checked_add(1).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        if self.commits.capacity() == 0 || size > u64::from(bounds.answer_bytes) {
            self.commits.clear();
            self.held = 0;
            return Ok(());
        }
        // Each removal gives up a newer commit; the kept length bounds the drain.
        for _ in 0..self.commits.len() {
            if self.commits.room() > 0
                && self.held.checked_add(size).ok_or(api::Error::TooLarge(Excess::unnamed()))?
                    <= u64::from(bounds.answer_bytes)
            {
                break;
            }
            let newer = self.commits.pop().expect("the kept length bounds removals");
            self.held = self
                .held
                .checked_sub(commit_bytes(&newer).expect("kept commit bytes fit"))
                .expect("held counts every kept commit");
        }
        self.held = self.held.checked_add(size).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        self.commits.push(value);
        Ok(())
    }

    fn done(&mut self, body: Body, bounds: Bounds) -> Result<Step, api::Error> {
        let total = match body {
            Body::Document(d::Document::Compare(compare)) => compare.total_commits,
            Body::Malformed(error) => return Err(super::failure::malformed(error)),
            Body::Document(_) | Body::Empty | Body::Raw { .. } => return Err(api::Error::InvalidAnswer),
        };
        match self.phase {
            Phase::Count { offset } => self.counted(total, offset, bounds),
            Phase::Oldest => {
                if total != self.given {
                    return Err(api::Error::InvalidAnswer);
                }
                let len = page_len(total, bounds.page_size);
                self.finish(len, total > u64::from(len), bounds)
            }
            Phase::Page { total: counted, len, more, .. } => {
                if total != counted || self.given != u64::from(len) {
                    return Err(api::Error::InvalidAnswer);
                }
                self.finish(len, more, bounds)
            }
        }
    }

    fn counted(&mut self, total: u64, offset: u64, bounds: Bounds) -> Result<Step, api::Error> {
        if offset >= total {
            return self.finish(0, false, bounds);
        }
        // In newest-first order, everything newer than the page precedes it.
        let available = total.saturating_sub(offset);
        let len = page_len(available, bounds.page_size);
        let start = available.saturating_sub(u64::from(len));
        self.phase = Phase::Page { total, start, len, more: start > 0 };
        Ok(self.begin())
    }

    fn finish(&mut self, needed: u32, more: bool, bounds: Bounds) -> Result<Step, api::Error> {
        if self.commits.len() < needed {
            let excess = if needed > bounds.rows {
                Excess::at_least(api::Bound::Rows, u64::from(bounds.rows), u64::from(needed))
            } else {
                Excess::at_least(
                    api::Bound::AnswerBytes,
                    u64::from(bounds.answer_bytes),
                    u64::from(bounds.answer_bytes).saturating_add(1),
                )
            };
            return Err(api::Error::TooLarge(excess));
        }
        let mut page = List::with_capacity(needed);
        for _ in 0..needed {
            let value = self.commits.pop().expect("the full page is held");
            page.push(value).expect("page capacity");
        }
        let mut commits = page.into_boxed();
        commits.reverse();
        Ok(Step::Done(Ok(api::Answer::Commits { commits, more })))
    }
}

fn page_len(available: u64, size: u32) -> u32 {
    u32::try_from(available.min(u64::from(size))).expect("length at most the page size")
}

fn commit_bytes(value: &api::CommitMessage) -> Option<u64> {
    u64::try_from(size_of::<api::CommitMessage>())
        .expect("commit width")
        .checked_add(len_u64(value.parents.len()).checked_mul(32)?)?
        .checked_add(len_u64(value.message.len()))
}
