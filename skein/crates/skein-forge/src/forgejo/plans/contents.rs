//! Contents plans: list changed paths, then read each side within the answer bound.
//! See skein/docs/design/forge/forgejo.md, sections 1.2 and 1.4.
use crate::api;
use crate::forgejo::plans::{
    Body, Heard,
    plan::{Bounds, Exchange, Step},
    reading,
    tables::{Repository, Tables},
};
use crate::forgejo::{document as d, request as r};
use alloc::boxed::Box;
use skein_lib::{List, bytes, usize_of};
#[derive(Debug)]
struct Sides {
    before: api::Commit,
    after: api::Commit,
    changed: List<d::ChangedFile>,
    files: List<api::File>,
    position: u32,
    head: api::Commit,
    more: Option<bool>,
    cut: bool,
    count: u32,
}
#[derive(Debug)]
struct Compared {
    before: api::Commit,
    after: api::Commit,
    files: List<Box<[u8]>>,
    changed: u64,
    ahead: u64,
    full: bool,
}
#[derive(Debug)]
enum State {
    Pull { number: u64, head: api::Commit, page: u32, changed: List<d::ChangedFile>, files: List<api::File> },
    Files(Sides),
    Compare(Compared),
    Reverse(Compared),
    Before { sides: Sides, missing: bool },
    After { sides: Sides, missing: bool },
    File { head: api::Commit, path: Box<[u8]>, max: u32, missing: bool },
    Directory { head: api::Commit, path: Box<[u8]>, listing: Box<crate::directory::Listing> },
    Ended,
}
#[derive(Debug)]
pub struct Plan {
    state: State,
    row: Repository,
    size: u64,
    limit: u32,
    skip: u32,
    sides: bool,
}
impl Plan {
    pub fn new(read: api::Read, row: Repository, bounds: Bounds) -> Result<Self, api::Error> {
        let (skip, sides) =
            if let api::Read::PullFiles { skip, sides, .. } = &read { (*skip, *sides) } else { (0, false) };
        let (state, limit) = match read {
            api::Read::PullFiles { number, head, page, .. } => (
                State::Pull {
                    number,
                    head,
                    page,
                    changed: List::with_capacity(bounds.page_size),
                    files: List::with_capacity(bounds.page_size),
                },
                bounds.page_size,
            ),
            api::Read::Compare { before, after } => (
                State::Compare(Compared {
                    before,
                    after,
                    files: List::with_capacity(bounds.rows),
                    changed: 0,
                    ahead: 0,
                    full: false,
                }),
                bounds.rows,
            ),
            api::Read::File { head, path, max_bytes } => {
                (State::File { head, path, max: max_bytes, missing: false }, 0)
            }
            api::Read::Locate { .. }
            | api::Read::Items { .. }
            | api::Read::Item { .. }
            | api::Read::Pull { .. }
            | api::Read::PullFor { .. }
            | api::Read::Reviews { .. }
            | api::Read::Statuses { .. }
            | api::Read::Remarks { .. }
            | api::Read::Branch { .. }
            | api::Read::Branches { .. }
            | api::Read::Protection { .. }
            | api::Read::Settings
            | api::Read::Collaborators { .. }
            | api::Read::Permission { .. }
            | api::Read::Checks { .. }
            | api::Read::Runs { .. }
            | api::Read::Run { .. }
            | api::Read::Tree { .. }
            | api::Read::Artifact { .. }
            | api::Read::Commits { .. }
            | api::Read::Job { .. } => return Err(api::Error::InvalidAnswer),
        };
        Ok(Self { state, row, size: 0, limit, skip, sides })
    }
    pub fn begin(&mut self, bounds: Bounds) -> Result<Step, api::Error> {
        let state = core::mem::replace(&mut self.state, State::Ended);
        let (state, step) = match state {
            state @ State::Pull { number, .. } => (state, Step::send(r::Request::Pull { at: self.row.at(), number })),
            State::Compare(sides) => {
                let step = Step::send(r::Request::Compare {
                    at: self.row.at(),
                    before: self.row.id(sides.before),
                    after: self.row.id(sides.after),
                    files: true,
                });
                (State::Compare(sides), step)
            }
            State::File { head, path, max, missing } => {
                self.size = skein_lib::len_u64(path.len());
                self.check_size(bounds)?;
                if path.is_empty() {
                    self.directory(head, path, max, bounds)
                } else {
                    let step = Step::Send(Exchange {
                        request: r::Request::Raw {
                            at: self.row.at(),
                            path: path.clone(),
                            reference: self.row.id(head),
                        },
                        range: Some(crate::forgejo::plans::plan::Range::Head(max.saturating_add(1))),
                    });
                    (State::File { head, path, max, missing }, step)
                }
            }
            State::Files(_)
            | State::Reverse(_)
            | State::Before { .. }
            | State::After { .. }
            | State::Directory { .. } => return Err(api::Error::InvalidAnswer),
            State::Ended => unreachable!("contents plan begins before it ends"),
        };
        self.state = state;
        Ok(step)
    }
    #[must_use]
    pub fn raw_room(&self, answer_bytes: u32) -> u32 {
        answer_bytes.saturating_sub(u32::try_from(self.size).expect("checked answer size"))
    }
    fn check_size(&self, bounds: Bounds) -> Result<(), api::Error> {
        if self.size > u64::from(bounds.answer_bytes) {
            Err(api::Error::TooLarge(api::Excess {
                bound: api::Bound::AnswerBytes,
                limit: u64::from(bounds.answer_bytes),
                size: self.size,
                exact: false,
            }))
        } else {
            Ok(())
        }
    }
    fn add_size(&mut self, size: u64, bounds: Bounds) -> Result<(), api::Error> {
        self.size = self.size.checked_add(size).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
        self.check_size(bounds)
    }
    pub fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        repository: api::Repository,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        match heard {
            Heard::Head { status, fields } => {
                let (raw, asked) = match &mut self.state {
                    State::Before { missing, .. } | State::After { missing, .. } => {
                        *missing = status == 404;
                        (true, super::failure::Asked::Read)
                    }
                    State::File { missing, .. } => {
                        *missing = status == 404;
                        (true, super::failure::Asked::Leaf)
                    }
                    State::Directory { .. } => (false, super::failure::Asked::Leaf),
                    State::Files(sides) => {
                        sides.more = fields.has_more;
                        (false, super::failure::Asked::Read)
                    }
                    State::Pull { .. } | State::Compare(_) | State::Reverse(_) => (false, super::failure::Asked::Read),
                    State::Ended => unreachable!("contents owner stops after Done"),
                };
                if raw && status == 404 {
                    return Ok(None);
                }
                if status != 200 && !(raw && status == 206) {
                    return Err(super::failure::refused(asked, status, None, fields.retry_after, repository, tables));
                }
                Ok(None)
            }
            Heard::Row(row) => {
                self.row(row, bounds)?;
                Ok(None)
            }
            Heard::Done(body) => {
                let state = core::mem::replace(&mut self.state, State::Ended);
                let missing = match &state {
                    State::Before { missing, .. } | State::After { missing, .. } | State::File { missing, .. } => {
                        *missing
                    }
                    State::Pull { .. }
                    | State::Files(_)
                    | State::Compare(_)
                    | State::Reverse(_)
                    | State::Directory { .. } => false,
                    State::Ended => unreachable!("contents owner stops after Done"),
                };
                let body = if missing {
                    None
                } else {
                    match &body {
                        Body::Document(document) => {
                            crate::forgejo::plans::reads::keep::keep_document(document, repository.forge, tables);
                        }
                        Body::Malformed(error) => return Err(super::failure::malformed(*error)),
                        Body::Empty => return Err(api::Error::InvalidAnswer),
                        Body::Raw { .. } => {}
                    }
                    Some(body)
                };
                let (state, step) = self.done(state, body, bounds)?;
                self.state = state;
                Ok(Some(step))
            }
            Heard::Failed(failure) => Err(crate::forgejo::plans::failure::transport(failure)),
        }
    }
    fn row(&mut self, row: d::Row, bounds: Bounds) -> Result<(), api::Error> {
        let state = core::mem::replace(&mut self.state, State::Ended);
        self.state = match row {
            d::Row::Entry { name, kind } => match state {
                State::Directory { head, path, mut listing } => {
                    listing.keep(name, kind);
                    State::Directory { head, path, listing }
                }
                State::Pull { .. }
                | State::Files(_)
                | State::Compare(_)
                | State::Reverse(_)
                | State::Before { .. }
                | State::After { .. }
                | State::File { .. } => return Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("contents owner stops after Done"),
            },
            d::Row::File(file) => match state {
                State::Files(sides) => State::Files(self.keep_file(sides, file, bounds)?),
                State::Compare(mut compared) => {
                    compared.changed =
                        compared.changed.checked_add(1).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
                    let width = u64::try_from(crate::measure::STRING_BYTES)
                        .expect("path width")
                        .saturating_add(skein_lib::len_u64(file.filename.len()));
                    if !compared.full
                        && compared.files.len() < bounds.rows
                        && self.size.saturating_add(width) <= u64::from(bounds.answer_bytes)
                    {
                        self.size = self.size.saturating_add(width);
                        compared.files.push(file.filename).expect("rows checked");
                    } else {
                        compared.full = true;
                    }
                    State::Compare(compared)
                }
                State::Reverse(sides) => State::Reverse(sides),
                State::Pull { .. }
                | State::Before { .. }
                | State::After { .. }
                | State::File { .. }
                | State::Directory { .. } => return Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("contents owner stops after Done"),
            },
            d::Row::Commit(_) => match state {
                State::Compare(compared) => State::Compare(compared),
                State::Reverse(sides) => State::Reverse(sides),
                State::Pull { .. }
                | State::Files(_)
                | State::Before { .. }
                | State::After { .. }
                | State::File { .. }
                | State::Directory { .. } => return Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("contents owner stops after Done"),
            },
            d::Row::Artifact(_)
            | d::Row::CommitMessage(_)
            | d::Row::Found(_)
            | d::Row::Reachable(_)
            | d::Row::Token(_)
            | d::Row::User(_)
            | d::Row::Issue(_)
            | d::Row::Comment(_)
            | d::Row::Review(_)
            | d::Row::Status(_)
            | d::Row::ReviewComment(_)
            | d::Row::Pull(_)
            | d::Row::Branch(_)
            | d::Row::Protection(_)
            | d::Row::TreeEntry(_)
            | d::Row::Run(_)
            | d::Row::Hook(_)
            | d::Row::Team(_)
            | d::Row::Step(_)
            | d::Row::Job(_) => return Err(api::Error::InvalidAnswer),
        };
        Ok(())
    }
    fn keep_file(&mut self, mut sides: Sides, file: d::ChangedFile, bounds: Bounds) -> Result<Sides, api::Error> {
        sides.count = sides.count.saturating_add(1);
        if sides.count > self.limit {
            return Err(api::Error::TooLarge(api::Excess::at_least(
                api::Bound::Forge,
                u64::from(self.limit),
                u64::from(sides.count),
            )));
        }
        if sides.count <= self.skip || sides.cut {
            return Ok(sides);
        }
        let added = skein_lib::len_u64(crate::measure::FILE_BYTES.saturating_add(file.filename.len()));
        if let Err(error) = self.add_size(added, bounds) {
            if sides.files.is_empty() {
                return Err(error);
            }
            sides.cut = true;
            return Ok(sides);
        }
        let value = api::File { path: file.filename.clone(), before: None, after: None };
        sides.files.push(value).or(Err(api::Error::TooLarge(crate::measure::capacity_excess(
            sides.files.capacity(),
            bounds.rows,
            bounds.answer_bytes,
            crate::measure::FILE_BYTES,
        ))))?;
        sides.changed.push(file).or(Err(api::Error::TooLarge(crate::measure::capacity_excess(
            sides.changed.capacity(),
            bounds.rows,
            bounds.answer_bytes,
            crate::measure::FILE_BYTES,
        ))))?;
        Ok(sides)
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        clippy::manual_let_else,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "each contents state completes one exchange or cuts before an incomplete file"
    )]
    fn done(&mut self, state: State, body: Option<Body>, bounds: Bounds) -> Result<(State, Step), api::Error> {
        match state {
            State::Pull { number, head, page, changed, files } => {
                let pull = match document(body)? {
                    d::Document::Pull(pull) => pull,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                if reading::commit(pull.head.sha.ok_or(api::Error::InvalidAnswer)?, self.row.object_format)
                    .ok_or(api::Error::InvalidAnswer)?
                    != head
                {
                    return Err(api::Error::Stale);
                }
                let before = if self.sides {
                    reading::commit(pull.merge_base.ok_or(api::Error::InvalidAnswer)?, self.row.object_format)
                        .ok_or(api::Error::InvalidAnswer)?
                } else {
                    head
                };
                Ok((
                    State::Files(Sides {
                        before,
                        after: head,
                        changed,
                        files,
                        position: 0,
                        head,
                        more: None,
                        cut: false,
                        count: 0,
                    }),
                    Step::send(r::Request::PullFiles {
                        at: self.row.at(),
                        number,
                        page: r::Page { page: Some(page), limit: Some(self.limit) },
                    }),
                ))
            }
            State::Files(sides) => {
                listed(body)?;
                if !self.sides {
                    return Ok(self.terminal(sides));
                }
                self.size = 0;
                self.next_side(sides, bounds)
            }
            State::Compare(mut sides) => {
                sides.ahead = comparison(body)?.total_commits;
                let step = Step::send(r::Request::Compare {
                    at: self.row.at(),
                    before: self.row.id(sides.after),
                    after: self.row.id(sides.before),
                    files: false,
                });
                Ok((State::Reverse(sides), step))
            }
            State::Reverse(compared) => {
                let behind = comparison(body)?.total_commits;
                let answer = api::Answer::Compare {
                    before: compared.before,
                    after: compared.after,
                    ahead: compared.ahead,
                    behind,
                    files: compared.files.into_boxed(),
                    changed: compared.changed,
                };
                Ok((State::Ended, Step::Done(Ok(answer))))
            }
            State::Before { mut sides, .. } => {
                let value = match self.raw(body, bounds) {
                    Ok(value) => value,
                    Err(api::Error::TooLarge(_)) if sides.position > 0 => return Ok(self.cut_files(sides)),
                    Err(error) => return Err(error),
                };
                sides.files.get_mut(sides.position).ok_or(api::Error::InvalidAnswer)?.before = value;
                let file = sides.changed.get(sides.position).expect("listed file");
                let deleted = file.status == d::FileStatus::Deleted;
                if deleted {
                    sides.position = sides.position.saturating_add(1);
                    self.next_side(sides, bounds)
                } else {
                    let step = self.read_side(file.filename.clone(), sides.after);
                    Ok((State::After { sides, missing: false }, step))
                }
            }
            State::After { mut sides, .. } => {
                let value = match self.raw(body, bounds) {
                    Ok(value) => value,
                    Err(api::Error::TooLarge(_)) if sides.position > 0 => return Ok(self.cut_files(sides)),
                    Err(error) => return Err(error),
                };
                sides.files.get_mut(sides.position).ok_or(api::Error::InvalidAnswer)?.after = value;
                sides.position = sides.position.saturating_add(1);
                self.next_side(sides, bounds)
            }
            State::File { head, path, max, .. } => match body {
                None => Ok(self.directory(head, path, max, bounds)),
                Some(Body::Raw { bytes: value, more }) => {
                    let room = max.min(self.raw_room(bounds.answer_bytes));
                    let length = value.len().min(usize_of(room));
                    self.add_size(u64::try_from(length).expect("bounded bytes"), bounds)?;
                    let truncated = more || value.len() > length;
                    let bytes = if value.len() == length {
                        value
                    } else {
                        bytes::copy_of(value.get(..length).expect("bounded length"))
                    };
                    Ok((State::Ended, Step::Done(Ok(api::Answer::File { head, path, bytes, truncated }))))
                }
                Some(Body::Document(_) | Body::Malformed(_) | Body::Empty) => Err(api::Error::InvalidAnswer),
            },
            State::Directory { head, path, mut listing } => {
                listed(body)?;
                Ok((State::Ended, Step::Done(Ok(listing.answer(head, path)))))
            }
            State::Ended => unreachable!("contents owner stops after Done"),
        }
    }
    fn directory(&self, head: api::Commit, path: Box<[u8]>, max: u32, bounds: Bounds) -> (State, Step) {
        let listing = Box::new(crate::directory::Listing::new(max.min(self.raw_room(bounds.answer_bytes))));
        let step =
            Step::send(r::Request::Contents { at: self.row.at(), path: path.clone(), reference: self.row.id(head) });
        (State::Directory { head, path, listing }, step)
    }
    fn next_side(&mut self, sides: Sides, bounds: Bounds) -> Result<(State, Step), api::Error> {
        if sides.position == sides.changed.len() {
            return Ok(self.terminal(sides));
        }
        let file = sides.changed.get(sides.position).expect("listed file");
        let added = skein_lib::len_u64(crate::measure::FILE_BYTES.saturating_add(file.filename.len()));
        if let Err(error) = self.add_size(added, bounds) {
            if sides.position == 0 {
                return Err(error);
            }
            return Ok(self.cut_files(sides));
        }
        if file.status == d::FileStatus::Added {
            let step = self.read_side(file.filename.clone(), sides.after);
            return Ok((State::After { sides, missing: false }, step));
        }
        let path = if file.status == d::FileStatus::Renamed {
            file.previous_filename.as_ref().ok_or(api::Error::InvalidAnswer)?.clone()
        } else {
            file.filename.clone()
        };
        self.check_size(bounds)?;
        let step = self.read_side(path, sides.before);
        Ok((State::Before { sides, missing: false }, step))
    }
    fn read_side(&self, path: Box<[u8]>, head: api::Commit) -> Step {
        Step::send(r::Request::Raw { at: self.row.at(), path, reference: self.row.id(head) })
    }
    fn raw(&mut self, body: Option<Body>, bounds: Bounds) -> Result<Option<Box<[u8]>>, api::Error> {
        let value = match body {
            None => None,
            Some(Body::Raw { bytes, more }) => {
                if more {
                    return Err(api::Error::TooLarge(api::Excess {
                        bound: api::Bound::AnswerBytes,
                        limit: u64::from(bounds.answer_bytes),
                        size: self.size.saturating_add(skein_lib::len_u64(bytes.len())).saturating_add(1),
                        exact: false,
                    }));
                }
                Some(bytes)
            }
            Some(Body::Document(_) | Body::Malformed(_) | Body::Empty) => return Err(api::Error::InvalidAnswer),
        };
        if let Some(value) = &value {
            self.add_size(skein_lib::len_u64(value.len()), bounds)?;
        }
        Ok(value)
    }
    fn cut_files(&self, mut sides: Sides) -> (State, Step) {
        sides.cut = true;
        sides.files.truncate(sides.position);
        self.terminal(sides)
    }
    fn terminal(&self, sides: Sides) -> (State, Step) {
        let files = sides.files.into_boxed();
        let answer = api::Answer::PullFiles {
            head: sides.head,
            more: sides.cut || sides.more.unwrap_or(sides.count == self.limit),
            files,
            cut: sides.cut,
        };
        (State::Ended, Step::Done(Ok(answer)))
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn listed(body: Option<Body>) -> Result<(), api::Error> {
    match document(body)? {
        d::Document::Listed { .. } => Ok(()),
        _ => Err(api::Error::InvalidAnswer),
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn comparison(body: Option<Body>) -> Result<d::Compare, api::Error> {
    match document(body)? {
        d::Document::Compare(value) => Ok(value),
        _ => Err(api::Error::InvalidAnswer),
    }
}

fn document(body: Option<Body>) -> Result<d::Document, api::Error> {
    match body {
        Some(Body::Document(document)) => Ok(document),
        Some(Body::Empty | Body::Raw { .. } | Body::Malformed(_)) | None => Err(api::Error::InvalidAnswer),
    }
}
