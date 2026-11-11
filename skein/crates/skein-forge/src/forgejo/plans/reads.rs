//! JSON read states and their running answer bound (skein/docs/design/forge/README.md, 3.7, skein/docs/design/forge/forgejo.md, 1.2, 1.4).
use crate::forgejo::plans::{
    Body, Heard,
    plan::{Bounds, Exchange, Step},
    reading,
    tables::{Repository, Tables},
};
mod done;
pub mod keep;
use keep::{keep_document, keep_row, page_bound};
pub mod size;
use crate::api;
use crate::forgejo::{
    document as d,
    id::ObjectFormat,
    request::{self as r, Request},
};
use alloc::boxed::Box;
use skein_lib::{List, Time, Wall, bytes};
#[derive(Debug)]
enum Delegate {
    Artifact(super::artifact::Plan),
    Commits(super::commits::Plan),
    Turn(super::turn::Plan),
    Contents(super::contents::Plan),
    Actions(super::actions::Plan),
}
#[derive(Debug)]
struct Roster {
    rows: List<api::Collaborator>,
    position: u32,
    more: bool,
}
#[derive(Debug)]
enum Whose {
    One,
    Each(Roster),
}
#[derive(Debug)]
enum State {
    New { read: api::Read },
    Resolving { read: api::Read },
    Locating,
    Settings,
    Delegated(Box<Delegate>),
    Items { rows: List<api::Summary>, now: Time, size: u64, cut: bool },
    Reviews { format: ObjectFormat, rows: List<api::Review>, size: u64, skip: u32, cut: bool },
    Remarks { rows: List<api::Remark>, more: bool, page: u32, size: u64, skip: u32, cut: bool },
    Issue { row: Repository, after: u64, since: Option<Wall> },
    Comments { item: api::Summary, rows: List<api::Comment>, more: bool, size: u64, after: u64 },
    Pull { row: Repository, ci: bool },
    PullFor { row: Repository, head: Box<[u8]>, base: Box<[u8]> },
    Pulls { row: Repository, head: Box<[u8]>, base: Box<[u8]>, pull: Box<d::PullRequest>, found: bool, page: u32 },
    Status { row: Repository, pull: Box<d::PullRequest>, actions: Box<super::actions::Plan> },
    Branch { format: ObjectFormat },
    ProtectionBranch { row: Repository },
    Protection,
    Branches { prefix: Box<[u8]>, rows: List<Box<[u8]>>, size: u64, missing: bool },
    Collaborators { row: Repository, rows: List<api::Collaborator>, size: u64 },
    Permission { row: Repository, whose: Whose },
    Searching { row: Repository, user: u64, whose: Whose },
    Ended,
}
#[derive(Debug)]
pub struct Plan {
    repository: api::Repository,
    state: State,
    request: Option<Exchange>,
    cut: bool,
    count: u32,
    bound: Option<u32>,
}
impl Plan {
    #[must_use]
    pub fn new(repository: api::Repository, read: api::Read) -> Self {
        Self { repository, state: State::New { read }, request: None, cut: false, count: 0, bound: None }
    }
    pub fn begin(&mut self, bounds: Bounds, _word: &[u8], tables: &mut Tables) -> Result<Step, api::Error> {
        let state = core::mem::replace(&mut self.state, State::Ended);
        let (state, step) = match state {
            State::New { read: api::Read::Branches { ref prefix } } if prefix.is_empty() => {
                return Err(api::Error::Empty);
            }
            State::New { read: api::Read::Locate { owner, name } } => {
                let _row = tables.repository(self.repository);
                (State::Locating, self.send(Request::Repository { at: r::Repo { owner, name } }))
            }
            State::New { read: api::Read::Settings } => {
                if let Some(row) = tables.repository(self.repository) {
                    return terminal(api::Answer::Settings(row.settings.clone()), bounds);
                }
                (State::Settings, self.send(Request::RepositoryById { id: u64::from(self.repository.repository) }))
            }
            State::New { read } => match tables.repository(self.repository).cloned() {
                Some(row) => self.initial(read, row, bounds, tables)?,
                None => (
                    State::Resolving { read },
                    self.send(Request::RepositoryById { id: u64::from(self.repository.repository) }),
                ),
            },
            State::Resolving { .. }
            | State::Locating
            | State::Settings
            | State::Delegated(_)
            | State::Items { .. }
            | State::Reviews { .. }
            | State::Remarks { .. }
            | State::Issue { .. }
            | State::Comments { .. }
            | State::Pull { .. }
            | State::PullFor { .. }
            | State::Pulls { .. }
            | State::Status { .. }
            | State::Branch { .. }
            | State::ProtectionBranch { .. }
            | State::Protection
            | State::Branches { .. }
            | State::Collaborators { .. }
            | State::Permission { .. }
            | State::Searching { .. }
            | State::Ended => unreachable!("a read plan begins once"),
        };
        self.state = state;
        Ok(step)
    }
    pub fn pending(&mut self) -> Option<Exchange> {
        self.request.take()
    }
    pub fn set_pending(&mut self, request: Exchange) {
        self.request = Some(request);
    }
    fn send(&mut self, request: Request) -> Step {
        self.bound = page_bound(&request);
        self.count = 0;
        Step::send(request)
    }
    #[expect(clippy::too_many_lines, reason = "initial request dispatch exhaustively names the read vocabulary")]
    fn initial(
        &mut self,
        read: api::Read,
        row: Repository,
        bounds: Bounds,
        tables: &mut Tables,
    ) -> Result<(State, Step), api::Error> {
        let at = row.at();
        let page_at = r::Page { page: Some(1), limit: Some(bounds.page_size) };
        let (state, request) = match read {
            api::Read::Runs { workflow, commit, page } => {
                return Ok(turn(super::turn::Mode::Runs { workflow, commit, page }, row, bounds));
            }
            api::Read::Run { run } => return Ok(turn(super::turn::Mode::Run(run), row, bounds)),
            api::Read::Tree { object, page } => return Ok(turn(super::turn::Mode::Tree { object, page }, row, bounds)),
            api::Read::Artifact { run, name, max_bytes } => {
                if name.is_empty() || max_bytes == 0 {
                    return Err(api::Error::Empty);
                }
                if max_bytes > bounds.answer_bytes {
                    return Err(api::Error::TooLarge(api::Excess::exact(
                        api::Bound::AnswerBytes,
                        u64::from(bounds.answer_bytes),
                        u64::from(max_bytes),
                    )));
                }
                let plan = super::artifact::Plan::new(row, run, name, max_bytes);
                let step = plan.begin();
                return Ok((State::Delegated(Box::new(Delegate::Artifact(plan))), step));
            }
            api::Read::Commits { before, after, page } => {
                if page == 0 {
                    return Err(api::Error::Empty);
                }
                let mut plan = super::commits::Plan::new(before, after, page, row, bounds)?;
                let step = plan.begin();
                return Ok((State::Delegated(Box::new(Delegate::Commits(plan))), step));
            }
            api::Read::Items { since, page, kind, single } => {
                let kind = match kind {
                    Some(api::Kind::Issue) => Some(r::IssueKind::Issues),
                    Some(api::Kind::Pull) => Some(r::IssueKind::Pulls),
                    None => None,
                };
                (
                    State::Items { rows: List::with_capacity(bounds.page_size), now: Time::ZERO, size: 0, cut: false },
                    Request::Issues {
                        at,
                        state: Some(r::StateFilter::All),
                        kind,
                        since: Some(Wall::from_nanos(since.as_nanos())),
                        sort: Some(r::Sort::LeastUpdate),
                        page: r::Page { page: Some(page), limit: if single { Some(1) } else { page_at.limit } },
                    },
                )
            }
            api::Read::Item { number, after, since } => (
                State::Issue {
                    row,
                    after,
                    since: if since == Time::ZERO { None } else { Some(Wall::from_nanos(since.as_nanos())) },
                },
                Request::Issue { at, number },
            ),
            api::Read::Pull { number, ci } => (State::Pull { row, ci }, Request::Pull { at, number }),
            api::Read::PullFor { head, base } => {
                (State::PullFor { row, head: head.clone(), base: base.clone() }, Request::PullFor { at, head, base })
            }
            api::Read::Reviews { number, page, skip } => (
                State::Reviews {
                    format: row.object_format,
                    rows: List::with_capacity(bounds.page_size),
                    size: 0,
                    skip,
                    cut: false,
                },
                Request::Reviews { at, number, page: r::Page { page: Some(page), ..page_at } },
            ),
            api::Read::Remarks { number, review, page, skip } => (
                State::Remarks {
                    rows: List::with_capacity(bounds.page_size),
                    more: false,
                    page,
                    size: 0,
                    skip,
                    cut: false,
                },
                Request::ReviewComments { at, number, review },
            ),
            api::Read::Branch { branch: name } => {
                (State::Branch { format: row.object_format }, Request::Branch { at, name })
            }
            api::Read::Protection { branch: name } => (State::ProtectionBranch { row }, Request::Branch { at, name }),
            api::Read::Branches { prefix } => {
                let room = bounds
                    .answer_bytes
                    .checked_div(u32::try_from(crate::measure::STRING_BYTES).expect("box size"))
                    .expect("nonzero width");
                (
                    State::Branches {
                        prefix: prefix.clone(),
                        rows: List::with_capacity(room),
                        size: 0,
                        missing: false,
                    },
                    Request::Branches { at, prefix: Some(prefix), page: r::Page { page: None, limit: None } },
                )
            }
            api::Read::Collaborators { page } => (
                State::Collaborators { row, rows: List::with_capacity(bounds.page_size), size: 0 },
                Request::Collaborators { at, page: r::Page { page: Some(page), ..page_at } },
            ),
            api::Read::Permission { user } => return Ok(self.permission(row, user, Whose::One, tables)),
            read @ (api::Read::PullFiles { .. } | api::Read::Compare { .. } | api::Read::File { .. }) => {
                let mut plan = super::contents::Plan::new(read, row, bounds)?;
                let step = plan.begin(bounds)?;
                return Ok((State::Delegated(Box::new(Delegate::Contents(plan))), step));
            }
            read @ (api::Read::Checks { .. } | api::Read::Statuses { .. } | api::Read::Job { .. }) => {
                let mut plan = super::actions::Plan::new(read, row, bounds)?;
                let step = plan.begin();
                return Ok((State::Delegated(Box::new(Delegate::Actions(plan))), step));
            }
            api::Read::Locate { .. } | api::Read::Settings => return Err(api::Error::Unavailable),
        };
        Ok((state, self.send(request)))
    }
    fn permission(&mut self, row: Repository, user: u64, whose: Whose, tables: &mut Tables) -> (State, Step) {
        match tables.user(self.repository.forge, user) {
            Some(login) => {
                let request = Request::Permission { at: row.at(), login: bytes::copy_of(login) };
                (State::Permission { row, whose }, self.send(request))
            }
            None => {
                (State::Searching { row, user, whose }, self.send(Request::UserSearch { uid: user, limit: Some(1) }))
            }
        }
    }
    #[must_use]
    pub fn after(&self) -> u64 {
        match &self.state {
            State::Issue { after, .. } | State::Comments { after, .. } => *after,
            State::New { .. }
            | State::Resolving { .. }
            | State::Locating
            | State::Settings
            | State::Delegated(_)
            | State::Items { .. }
            | State::Reviews { .. }
            | State::Remarks { .. }
            | State::Pull { .. }
            | State::PullFor { .. }
            | State::Pulls { .. }
            | State::Status { .. }
            | State::Branch { .. }
            | State::ProtectionBranch { .. }
            | State::Protection
            | State::Branches { .. }
            | State::Collaborators { .. }
            | State::Permission { .. }
            | State::Searching { .. }
            | State::Ended => 0,
        }
    }
    #[must_use]
    pub fn raw_room(&self, answer_bytes: u32) -> u32 {
        match &self.state {
            State::Delegated(delegate) => match delegate.as_ref() {
                Delegate::Contents(plan) => plan.raw_room(answer_bytes),
                Delegate::Actions(plan) => plan.raw_room(answer_bytes),
                Delegate::Artifact(_) | Delegate::Commits(_) | Delegate::Turn(_) => answer_bytes,
            },
            State::Status { actions, .. } => actions.raw_room(answer_bytes),
            State::New { .. }
            | State::Resolving { .. }
            | State::Locating
            | State::Settings
            | State::Items { .. }
            | State::Reviews { .. }
            | State::Remarks { .. }
            | State::Issue { .. }
            | State::Comments { .. }
            | State::Pull { .. }
            | State::PullFor { .. }
            | State::Pulls { .. }
            | State::Branch { .. }
            | State::ProtectionBranch { .. }
            | State::Protection
            | State::Branches { .. }
            | State::Collaborators { .. }
            | State::Permission { .. }
            | State::Searching { .. }
            | State::Ended => answer_bytes,
        }
    }
    pub fn take_cut(&mut self) -> bool {
        core::mem::replace(&mut self.cut, false)
    }
    pub fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        let state = core::mem::replace(&mut self.state, State::Ended);
        let (state, step) = match state {
            State::Delegated(mut delegate) => {
                let step = match delegate.as_mut() {
                    Delegate::Artifact(plan) => plan.heard(heard, bounds, self.repository, tables),
                    Delegate::Commits(plan) => plan.heard(heard, bounds, self.repository, tables),
                    Delegate::Turn(plan) => plan.heard(heard, bounds, self.repository, tables),
                    Delegate::Contents(plan) => plan.heard(heard, bounds, self.repository, tables),
                    Delegate::Actions(plan) => plan.heard(heard, bounds, self.repository, tables),
                }?;
                let state = match &step {
                    Some(Step::Done(_)) => State::Ended,
                    Some(Step::Send(_)) | None => State::Delegated(delegate),
                };
                (state, step)
            }
            State::Status { row, pull, mut actions } => match actions.heard(heard, bounds, self.repository, tables)? {
                Some(Step::Done(answer)) => (
                    State::Ended,
                    Some(terminal(
                        api::Answer::Pull(reading::pull(*pull, row.object_format, checks_ci(answer?)?)?),
                        bounds,
                    )?),
                ),
                step => (State::Status { row, pull, actions }, step),
            },
            State::Ended => unreachable!("owner stops feeding an ended read plan"),
            state @ (State::New { .. }
            | State::Resolving { .. }
            | State::Locating
            | State::Settings
            | State::Items { .. }
            | State::Reviews { .. }
            | State::Remarks { .. }
            | State::Issue { .. }
            | State::Comments { .. }
            | State::Pull { .. }
            | State::PullFor { .. }
            | State::Pulls { .. }
            | State::Branch { .. }
            | State::ProtectionBranch { .. }
            | State::Protection
            | State::Branches { .. }
            | State::Collaborators { .. }
            | State::Permission { .. }
            | State::Searching { .. }) => match heard {
                Heard::Head { status, fields } => (head(state, status, fields, self.repository, tables)?, None),
                Heard::Row(row) => (self.row(state, row, bounds, word, tables)?, None),
                Heard::Done(body) => self.body(state, body, bounds, word, tables)?,
                Heard::Failed(failure) => return Err(super::failure::transport(failure)),
            },
        };
        self.state = state;
        Ok(step)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "streaming states validate their decoder-selected row kind and charge it before retaining it"
    )]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn row(
        &mut self,
        mut state: State,
        value: d::Row,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<State, api::Error> {
        self.count = self.count.checked_add(1).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
        if let Some(bound) = self.bound
            && self.count > bound
        {
            return Err(api::Error::TooLarge(api::Excess::at_least(
                if bound == bounds.rows { api::Bound::Rows } else { api::Bound::Forge },
                u64::from(bound),
                u64::from(self.count),
            )));
        }
        keep_row(&value, self.repository.forge, tables);
        match &mut state {
            State::Pulls { head, base, pull, found, .. } => match value {
                d::Row::Pull(value) => {
                    if !*found
                        && value.head.name == *head
                        && value.head.repo_id == u64::from(self.repository.repository)
                        && value.base.name == *base
                    {
                        **pull = value;
                        *found = true;
                    }
                }
                _ => return Err(api::Error::InvalidAnswer),
            },
            State::Searching { .. } => match value {
                d::Row::User(_) => {}
                _ => return Err(api::Error::InvalidAnswer),
            },
            State::Collaborators { rows, size, .. } => {
                let user = match value {
                    d::Row::User(user) => user.id,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                charge(
                    size,
                    u64::try_from(crate::measure::COLLABORATOR_BYTES).expect("collaborator width"),
                    rows.len(),
                    rows.capacity(),
                    bounds,
                )?;
                rows.push(api::Collaborator { user, permission: api::Permission::None }).expect("room checked");
            }
            State::Items { rows, size, cut, .. } => {
                let item = match value {
                    d::Row::Issue(issue) => reading::summary(word, issue),
                    _ => return Err(api::Error::InvalidAnswer),
                };
                let added = row_size(crate::measure::SUMMARY_BYTES, size::summary(&item, bounds.rows)?, bounds)?;
                match charge(size, added, rows.len(), rows.capacity(), bounds) {
                    Ok(()) => {
                        rows.push(item).expect("room checked");
                    }
                    Err(error) if rows.is_empty() => return Err(error),
                    Err(_) => {
                        *cut = true;
                        self.cut = true;
                    }
                }
            }
            State::Comments { rows, more, size, .. } => {
                let comment = match value {
                    d::Row::Comment(comment) => reading::comment(word, comment),
                    _ => return Err(api::Error::InvalidAnswer),
                };
                let added =
                    row_size(crate::measure::COMMENT_BYTES, body_size(&comment.body, comment.key.as_deref())?, bounds)?;
                if charge(size, added, rows.len(), rows.capacity(), bounds).is_ok() {
                    rows.push(comment).expect("room checked");
                } else {
                    *more = true;
                    self.cut = true;
                }
            }
            State::Reviews { format, rows, size, skip, cut } => {
                let review = match value {
                    d::Row::Review(review) => reading::review(word, review, *format)?,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                if let Some(review) = review {
                    if *skip > 0 {
                        *skip = skip.saturating_sub(1);
                        return Ok(state);
                    }
                    let added = row_size(
                        crate::measure::REVIEW_BYTES,
                        body_size(&review.body, review.key.as_deref())?,
                        bounds,
                    )?;
                    match charge(size, added, rows.len(), rows.capacity(), bounds) {
                        Ok(()) => {
                            rows.push(review).expect("room checked");
                        }
                        Err(error) if rows.is_empty() => return Err(error),
                        Err(_) => {
                            *cut = true;
                            self.cut = true;
                        }
                    }
                }
            }
            State::Remarks { rows, more, page, size, skip: passed, cut } => match value {
                d::Row::ReviewComment(remark) => {
                    let skip = page
                        .saturating_sub(1)
                        .checked_mul(bounds.page_size)
                        .ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
                    if self.count > skip.saturating_add(bounds.page_size) {
                        *more = true;
                        self.cut = true;
                    } else if self.count > skip.saturating_add(*passed) {
                        let remark = reading::remark(word, remark);
                        let added = row_size(
                            crate::measure::REMARK_BYTES,
                            body_size(&remark.body, Some(&remark.path))?,
                            bounds,
                        )?;
                        match charge(size, added, rows.len(), rows.capacity(), bounds) {
                            Ok(()) => {
                                rows.push(remark).expect("room checked");
                            }
                            Err(error) if rows.is_empty() => return Err(error),
                            Err(_) => {
                                *more = true;
                                *cut = true;
                                self.cut = true;
                            }
                        }
                    }
                }
                _ => return Err(api::Error::InvalidAnswer),
            },
            State::Branches { prefix, rows, size, .. } => {
                let branch = match value {
                    d::Row::Branch(branch) => branch.name,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                if !branch.starts_with(prefix) {
                    return Err(api::Error::InvalidAnswer);
                }
                let total = size
                    .checked_add(skein_lib::len_u64(branch.len()))
                    .ok_or(api::Error::TooLarge(api::Excess::unnamed()))?
                    .checked_add(skein_lib::len_u64(crate::measure::STRING_BYTES))
                    .ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
                if total > u64::from(bounds.answer_bytes) {
                    return Err(api::Error::TooLarge(api::Excess::at_least(
                        api::Bound::AnswerBytes,
                        u64::from(bounds.answer_bytes),
                        total,
                    )));
                }
                *size = total;
                rows.push(branch).expect("room checked");
            }
            State::Protection => match value {
                d::Row::Protection(_) => {}
                _ => return Err(api::Error::InvalidAnswer),
            },
            State::New { .. }
            | State::Resolving { .. }
            | State::Locating
            | State::Settings
            | State::Delegated(_)
            | State::Issue { .. }
            | State::Pull { .. }
            | State::PullFor { .. }
            | State::Status { .. }
            | State::Branch { .. }
            | State::ProtectionBranch { .. }
            | State::Permission { .. } => return Err(api::Error::InvalidAnswer),
            State::Ended => unreachable!("owner stops feeding an ended read plan"),
        }
        Ok(state)
    }
}
fn turn(mode: super::turn::Mode, row: Repository, bounds: Bounds) -> (State, Step) {
    let plan = super::turn::Plan::new(mode, row, bounds);
    let step = plan.begin(bounds);
    (State::Delegated(Box::new(Delegate::Turn(plan))), step)
}
fn charge(size: &mut u64, added: u64, count: u32, capacity: u32, bounds: Bounds) -> Result<(), api::Error> {
    if count >= capacity {
        let excess = if count >= bounds.rows {
            api::Excess {
                bound: api::Bound::Rows,
                limit: u64::from(bounds.rows),
                size: u64::from(count).saturating_add(1),
                exact: false,
            }
        } else {
            api::Excess {
                bound: api::Bound::AnswerBytes,
                limit: u64::from(bounds.answer_bytes),
                size: size.saturating_add(added),
                exact: false,
            }
        };
        return Err(api::Error::TooLarge(excess));
    }
    let total = size.checked_add(added).ok_or(api::Error::TooLarge(api::Excess::unnamed()))?;
    if total > u64::from(bounds.answer_bytes) {
        return Err(api::Error::TooLarge(api::Excess {
            bound: api::Bound::AnswerBytes,
            limit: u64::from(bounds.answer_bytes),
            size: total,
            exact: false,
        }));
    }
    *size = total;
    Ok(())
}
fn row_size(width: usize, bytes: u64, bounds: Bounds) -> Result<u64, api::Error> {
    if bounds.rows == 0 {
        return Err(api::Error::TooLarge(api::Excess::exact(api::Bound::Rows, 0, 1)));
    }
    bytes.checked_add(u64::try_from(width).expect("row width")).ok_or(api::Error::TooLarge(api::Excess::unnamed()))
}
fn body_size(body: &[u8], extra: Option<&[u8]>) -> Result<u64, api::Error> {
    let extra = match extra {
        Some(extra) => skein_lib::len_u64(extra.len()),
        None => 0,
    };
    skein_lib::len_u64(body.len()).checked_add(extra).ok_or(api::Error::TooLarge(api::Excess::unnamed()))
}
fn head(
    mut state: State,
    status: u16,
    fields: super::Fields,
    repository: api::Repository,
    tables: &mut Tables,
) -> Result<State, api::Error> {
    let asked = match &mut state {
        State::Branches { missing, .. } if status == 404 => {
            *missing = true;
            return Ok(state);
        }
        State::Branch { .. } | State::PullFor { .. } => super::failure::Asked::Leaf,
        State::Items { now, .. } => {
            if status == 200 {
                *now = reading::time(fields.date.ok_or(api::Error::InvalidAnswer)?);
            }
            super::failure::Asked::Read
        }
        State::New { .. }
        | State::Resolving { .. }
        | State::Locating
        | State::Settings
        | State::Delegated(_)
        | State::Reviews { .. }
        | State::Remarks { .. }
        | State::Issue { .. }
        | State::Comments { .. }
        | State::Pull { .. }
        | State::Pulls { .. }
        | State::Status { .. }
        | State::ProtectionBranch { .. }
        | State::Protection
        | State::Branches { .. }
        | State::Collaborators { .. }
        | State::Permission { .. }
        | State::Searching { .. } => super::failure::Asked::Read,
        State::Ended => unreachable!("owner stops feeding an ended read plan"),
    };
    if status != 200 {
        return Err(super::failure::refused(asked, status, None, fields.retry_after, repository, tables));
    }
    Ok(state)
}
fn terminal(answer: api::Answer, bounds: Bounds) -> Result<Step, api::Error> {
    let size = size::answer(&answer, bounds.rows)?;
    if size > u64::from(bounds.answer_bytes) {
        return Err(api::Error::TooLarge(api::Excess::exact(
            api::Bound::AnswerBytes,
            u64::from(bounds.answer_bytes),
            size,
        )));
    }
    Ok(Step::Done(Ok(answer)))
}
fn checks_ci(answer: api::Answer) -> Result<api::Ci, api::Error> {
    match answer {
        api::Answer::Statuses { ci, .. } => Ok(ci),
        api::Answer::Checks(_)
        | api::Answer::Runs { .. }
        | api::Answer::Run(_)
        | api::Answer::Tree { .. }
        | api::Answer::Commits { .. }
        | api::Answer::Items { .. }
        | api::Answer::Item { .. }
        | api::Answer::Pull(_)
        | api::Answer::Reviews { .. }
        | api::Answer::Remarks { .. }
        | api::Answer::Commit(_)
        | api::Answer::Branches(_)
        | api::Answer::Located { .. }
        | api::Answer::PullFiles { .. }
        | api::Answer::Compare { .. }
        | api::Answer::Job { .. }
        | api::Answer::Directory { .. }
        | api::Answer::File { .. }
        | api::Answer::Protection(_)
        | api::Answer::Settings(_)
        | api::Answer::Collaborators { .. }
        | api::Answer::Permission(_)
        | api::Answer::Created(_)
        | api::Answer::Commented(_)
        | api::Answer::Reviewed(_)
        | api::Answer::Merged(_)
        | api::Answer::Queued
        | api::Answer::Branch(_)
        | api::Answer::Done => Err(api::Error::InvalidAnswer),
    }
}
