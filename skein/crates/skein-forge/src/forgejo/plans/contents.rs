//! Contents plans: list changed paths, then read each side within the answer bound.
//! See skein/docs/design/forge.md, sections 5.2 and 5.4.
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
    ask: Ask,
}
#[derive(Debug)]
enum Ask {
    Pull { head: api::Commit, more: Option<bool> },
    Compare { commits: List<api::Commit>, contains: bool },
}
#[derive(Debug)]
enum State {
    Pull { number: u64, head: api::Commit, page: u32, changed: List<d::ChangedFile>, files: List<api::File> },
    Files(Sides),
    Compare(Sides),
    Reverse(Sides),
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
}
impl Plan {
    pub fn new(read: api::Read, row: Repository, bounds: Bounds) -> Result<Self, api::Error> {
        let (state, limit) = match read {
            api::Read::PullFiles { number, head, page } => (
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
                State::Compare(Sides {
                    before,
                    after,
                    changed: List::with_capacity(bounds.rows),
                    files: List::with_capacity(bounds.rows),
                    position: 0,
                    ask: Ask::Compare { commits: List::with_capacity(bounds.rows), contains: false },
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
        Ok(Self { state, row, size: 0, limit })
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
                        range: Some(crate::forgejo::plans::plan::Range::Head(max)),
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
        if self.size > u64::from(bounds.answer_bytes) { Err(api::Error::TooLarge) } else { Ok(()) }
    }
    fn add_size(&mut self, size: u64, bounds: Bounds) -> Result<(), api::Error> {
        self.size = self.size.checked_add(size).ok_or(api::Error::TooLarge)?;
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
                        match &mut sides.ask {
                            Ask::Pull { more, .. } => *more = fields.has_more,
                            Ask::Compare { .. } => return Err(api::Error::InvalidAnswer),
                        }
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
                        Body::Malformed(crate::forgejo::decode::Malformed::Bound { field }) => {
                            return Err(if *field == b"log" {
                                api::Error::InvalidAnswer
                            } else {
                                api::Error::TooLarge
                            });
                        }
                        Body::Empty | Body::Malformed(_) => return Err(api::Error::InvalidAnswer),
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
                State::Compare(sides) => State::Compare(self.keep_file(sides, file, bounds)?),
                State::Reverse(sides) => State::Reverse(sides),
                State::Pull { .. }
                | State::Before { .. }
                | State::After { .. }
                | State::File { .. }
                | State::Directory { .. } => return Err(api::Error::InvalidAnswer),
                State::Ended => unreachable!("contents owner stops after Done"),
            },
            d::Row::Commit(commit) => match state {
                State::Compare(mut sides) => {
                    let commit = reading::commit(commit, self.row.object_format).ok_or(api::Error::InvalidAnswer)?;
                    self.add_size(u64::try_from(crate::measure::COMMIT_BYTES).expect("commit width"), bounds)?;
                    match &mut sides.ask {
                        Ask::Compare { commits, .. } => commits.push(commit).or(Err(api::Error::TooLarge))?,
                        Ask::Pull { .. } => return Err(api::Error::InvalidAnswer),
                    }
                    State::Compare(sides)
                }
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
        match &sides.ask {
            Ask::Compare { .. } => {
                for kept in &sides.changed {
                    if kept.filename == file.filename {
                        return Ok(sides);
                    }
                }
            }
            Ask::Pull { .. } => {}
        }
        self.add_size(skein_lib::len_u64(crate::measure::FILE_BYTES.saturating_add(file.filename.len())), bounds)?;
        let value = api::File { path: file.filename.clone(), before: None, after: None };
        sides.files.push(value).or(Err(api::Error::TooLarge))?;
        sides.changed.push(file).or(Err(api::Error::TooLarge))?;
        Ok(sides)
    }
    #[expect(clippy::manual_let_else, reason = "document kinds are matched exhaustively at the exchange boundary")]
    fn done(&mut self, state: State, body: Option<Body>, bounds: Bounds) -> Result<(State, Step), api::Error> {
        match state {
            State::Pull { number, head, page, changed, files } => {
                let pull = match body {
                    Some(Body::Document(d::Document::Pull(pull))) => pull,
                    Some(_) | None => return Err(api::Error::InvalidAnswer),
                };
                if reading::commit(pull.head.sha.ok_or(api::Error::InvalidAnswer)?, self.row.object_format)
                    .ok_or(api::Error::InvalidAnswer)?
                    != head
                {
                    return Err(api::Error::Stale);
                }
                let before = reading::commit(pull.merge_base.ok_or(api::Error::InvalidAnswer)?, self.row.object_format)
                    .ok_or(api::Error::InvalidAnswer)?;
                Ok((
                    State::Files(Sides {
                        before,
                        after: head,
                        changed,
                        files,
                        position: 0,
                        ask: Ask::Pull { head, more: None },
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
                self.next_side(sides, bounds)
            }
            State::Compare(sides) => {
                comparison(body)?;
                let step = Step::send(r::Request::Compare {
                    at: self.row.at(),
                    before: self.row.id(sides.after),
                    after: self.row.id(sides.before),
                    files: false,
                });
                Ok((State::Reverse(sides), step))
            }
            State::Reverse(mut sides) => {
                let comparison = comparison(body)?;
                match &mut sides.ask {
                    Ask::Compare { contains, .. } => *contains = comparison.total_commits == 0,
                    Ask::Pull { .. } => return Err(api::Error::InvalidAnswer),
                }
                self.next_side(sides, bounds)
            }
            State::Before { mut sides, .. } => {
                let value = self.raw(body, bounds)?;
                sides.files.get_mut(sides.position).ok_or(api::Error::InvalidAnswer)?.before = value;
                let file = sides.changed.get(sides.position).expect("listed file");
                let deleted = match sides.ask {
                    Ask::Pull { .. } => file.status == d::FileStatus::Deleted,
                    Ask::Compare { .. } => false,
                };
                if deleted {
                    sides.position = sides.position.saturating_add(1);
                    self.next_side(sides, bounds)
                } else {
                    let step = self.read_side(file.filename.clone(), sides.after);
                    Ok((State::After { sides, missing: false }, step))
                }
            }
            State::After { mut sides, .. } => {
                let value = self.raw(body, bounds)?;
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
    fn next_side(&self, sides: Sides, bounds: Bounds) -> Result<(State, Step), api::Error> {
        if sides.position == sides.changed.len() {
            return Ok(self.terminal(sides));
        }
        let file = sides.changed.get(sides.position).expect("listed file");
        let compare = match sides.ask {
            Ask::Compare { .. } => true,
            Ask::Pull { .. } => false,
        };
        if !compare && file.status == d::FileStatus::Added {
            let step = self.read_side(file.filename.clone(), sides.after);
            return Ok((State::After { sides, missing: false }, step));
        }
        let path = if !compare && file.status == d::FileStatus::Renamed {
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
                    return Err(api::Error::TooLarge);
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
    fn terminal(&self, sides: Sides) -> (State, Step) {
        let files = sides.files.into_boxed();
        let answer = match sides.ask {
            Ask::Pull { head, more } => {
                api::Answer::PullFiles { head, more: more.unwrap_or(files.len() == usize_of(self.limit)), files }
            }
            Ask::Compare { commits, contains } => api::Answer::Compare {
                before: sides.before,
                after: sides.after,
                contains_before: contains,
                files,
                commits: commits.into_boxed(),
            },
        };
        (State::Ended, Step::Done(Ok(answer)))
    }
}
fn listed(body: Option<Body>) -> Result<(), api::Error> {
    match body {
        Some(Body::Document(d::Document::Listed { .. })) => Ok(()),
        Some(_) | None => Err(api::Error::InvalidAnswer),
    }
}
fn comparison(body: Option<Body>) -> Result<d::Compare, api::Error> {
    match body {
        Some(Body::Document(d::Document::Compare(value))) => Ok(value),
        Some(_) | None => Err(api::Error::InvalidAnswer),
    }
}
