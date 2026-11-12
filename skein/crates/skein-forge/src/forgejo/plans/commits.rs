//! Keeps the oldest commits within its capacity: Forgejo sends them newest
//! first (skein/docs/design/forge/forgejo.md, 1.2).
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
    skip: u32,
    commits: Queue<api::CommitMessage>,
    held: u64,
    given: u32,
}
impl Plan {
    pub(super) fn new(
        before: api::Commit,
        after: api::Commit,
        page: u32,
        row: Repository,
        bounds: Bounds,
    ) -> Result<Self, api::Error> {
        let skip = page
            .checked_sub(1)
            .ok_or(api::Error::Empty)?
            .checked_mul(bounds.page_size)
            .ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        if skip >= bounds.rows {
            return Err(api::Error::TooLarge(Excess::at_least(
                api::Bound::Rows,
                u64::from(bounds.rows),
                u64::from(skip).saturating_add(1),
            )));
        }
        let capacity = bounds.rows.min(
            bounds
                .answer_bytes
                .checked_div(u32::try_from(size_of::<api::CommitMessage>()).expect("commit width"))
                .expect("nonzero width"),
        );
        Ok(Self { row, before, after, skip, commits: Queue::with_capacity(capacity), held: 0, given: 0 })
    }
    pub(super) fn begin(&mut self) -> Step {
        Step::send(Request::Commits {
            at: self.row.at(),
            before: self.row.id(self.before),
            after: self.row.id(self.after),
        })
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
                self.received(row, bounds)?;
                Ok(None)
            }
            Heard::Done(body) => self.done(body, bounds),
        }
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        clippy::manual_let_else,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn received(&mut self, row: d::Row, bounds: Bounds) -> Result<(), api::Error> {
        let value = match row {
            d::Row::CommitMessage(value) => value,
            _ => return Err(api::Error::InvalidAnswer),
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
        if size > u64::from(bounds.answer_bytes) {
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

    fn done(&mut self, body: Body, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        match body {
            Body::Document(d::Document::Compare(compare)) => {
                if compare.total_commits != u64::from(self.given) {
                    return Err(api::Error::InvalidAnswer);
                }
            }
            Body::Malformed(error) => return Err(super::failure::malformed(error)),
            Body::Document(_) | Body::Empty | Body::Raw { .. } => {
                return Err(api::Error::InvalidAnswer);
            }
        }
        let end = self.skip.checked_add(bounds.page_size).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        let needed = self.given.min(end);
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
        let more = self.given > end;
        let count = needed.saturating_sub(self.skip);
        let mut page = List::with_capacity(count);
        let kept = self.commits.len();
        for position in 0..kept {
            let value = self.commits.pop().expect("the kept length bounds removals");
            if position >= kept.saturating_sub(needed) && page.len() < count {
                page.push(value).expect("page capacity");
            }
        }
        self.held = 0;
        let mut commits = page.into_boxed();
        commits.reverse();
        Ok(Some(Step::Done(Ok(api::Answer::Commits { commits, more }))))
    }
}

fn commit_bytes(value: &api::CommitMessage) -> Option<u64> {
    u64::try_from(size_of::<api::CommitMessage>())
        .expect("commit width")
        .checked_add(len_u64(value.parents.len()).checked_mul(32)?)?
        .checked_add(len_u64(value.message.len()))
}
