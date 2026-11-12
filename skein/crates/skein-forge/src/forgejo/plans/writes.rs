//! Forge writes, with validation before resolution and no retry
//! (forge/README.md, 3.2–3.4, and forge/forgejo.md, 1.2 and 1.4).
use crate::api;
use crate::forgejo::plans::{
    Body, Heard,
    failure::{self, Asked},
    plan::{Bounds, Exchange, Step},
    reading,
    tables::{Repository, Tables},
};
use crate::forgejo::{
    document as d,
    id::ObjectId,
    request::{self as r, Request},
};
use crate::marker;
use alloc::boxed::Box;
use skein_lib::{Duration, List};
#[derive(Debug)]
struct Open {
    title: Box<[u8]>,
    body: Box<[u8]>,
    head: Box<[u8]>,
    base: Box<[u8]>,
}
#[derive(Debug)]
struct Looking {
    missing: List<u64>,
    position: u32,
    add: List<Box<[u8]>>,
    remove: List<Box<[u8]>>,
}
#[derive(Clone, Copy, Debug)]
enum Written {
    Issue,
    Comment,
    Review,
    Edited,
    Status,
    Branch,
    Bare,
    Deleted,
    Update(api::UpdateStyle),
    Pull,
    Merge { number: u64 },
}
#[derive(Debug)]
enum State {
    New { write: api::Write },
    Resolving { write: api::Write },
    Writing { row: Repository, written: Written },
    Merged { row: Repository },
    OpenHead { row: Repository, open: Open },
    OpenBase { row: Repository, open: Open, head: ObjectId },
    OpenCompare { row: Repository, open: Open },
    Reviewers { row: Repository, number: u64, reviewers: Box<[u64]> },
    Searching { row: Repository, number: u64, looking: Looking },
    Adding { row: Repository, number: u64, remove: List<Box<[u8]>> },
    Removing { row: Repository },
    Ended,
}
#[derive(Clone, Copy, Debug)]
enum Errors {
    Asked(Asked),
    Merged,
}
impl Errors {
    fn error(self, error: api::Error) -> api::Error {
        match self {
            Self::Asked(asked) => failure::before_write(asked, error),
            Self::Merged => api::Error::Timeout,
        }
    }
}
#[derive(Debug)]
pub struct Plan {
    repository: api::Repository,
    state: State,
    status: u16,
    retry: Option<Duration>,
    request: Option<Exchange>,
}
impl Plan {
    pub fn pending(&mut self) -> Option<Exchange> {
        self.request.take()
    }
    pub fn set_pending(&mut self, request: Exchange) {
        self.request = Some(request);
    }
    #[must_use]
    pub fn new(repository: api::Repository, write: api::Write) -> Self {
        Self { repository, state: State::New { write }, status: 0, retry: None, request: None }
    }
    pub fn begin(&mut self, bounds: Bounds, word: &[u8], tables: &mut Tables) -> Result<Step, api::Error> {
        let state = core::mem::replace(&mut self.state, State::Ended);
        let step;
        (self.state, step) = match state {
            State::New { write } => {
                validate(&write)?;
                match tables.repository(self.repository).cloned() {
                    Some(row) => initial(row, write, bounds, word)?,
                    None => (
                        State::Resolving { write },
                        Step::send(Request::RepositoryById { id: u64::from(self.repository.repository) }),
                    ),
                }
            }
            State::Resolving { .. }
            | State::Writing { .. }
            | State::Merged { .. }
            | State::OpenHead { .. }
            | State::OpenBase { .. }
            | State::OpenCompare { .. }
            | State::Reviewers { .. }
            | State::Searching { .. }
            | State::Adding { .. }
            | State::Removing { .. }
            | State::Ended => unreachable!("write begins once"),
        };
        Ok(step)
    }
    fn asked(&self) -> Asked {
        match self.errors() {
            Errors::Asked(asked) => asked,
            Errors::Merged => Asked::Merge,
        }
    }
    fn errors(&self) -> Errors {
        match &self.state {
            State::New { write } => Errors::Asked(asked(write)),
            State::Writing { written, .. } => Errors::Asked(written.exchange().0),
            State::Resolving { write } => {
                let asked = asked(write);
                Errors::Asked(if asked == Asked::OpenPull { Asked::BeforeWrite } else { asked })
            }
            State::OpenHead { .. } | State::OpenBase { .. } | State::OpenCompare { .. } => {
                Errors::Asked(Asked::BeforeWrite)
            }
            State::Merged { .. } => Errors::Merged,
            State::Reviewers { .. } | State::Searching { .. } | State::Adding { .. } | State::Removing { .. } => {
                Errors::Asked(Asked::Write)
            }
            State::Ended => unreachable!("ended write has no exchange"),
        }
    }
    fn expected(&self) -> u16 {
        match &self.state {
            State::New { .. } | State::Ended => unreachable!("write has no exchange"),
            State::Resolving { .. }
            | State::OpenHead { .. }
            | State::OpenBase { .. }
            | State::OpenCompare { .. }
            | State::Reviewers { .. }
            | State::Searching { .. }
            | State::Merged { .. } => 200,
            State::Adding { .. } => 201,
            State::Removing { .. } => 204,
            State::Writing { written, .. } => written.exchange().1,
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    pub fn heard(
        &mut self,
        heard: Heard,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        match heard {
            Heard::Head { status, fields } => {
                self.status = status;
                self.retry = fields.retry_after;
                Ok(None)
            }
            Heard::Row(row) => match row {
                d::Row::User(user) => {
                    tables.keep_user(self.repository.forge, user.id, user.login);
                    Ok(None)
                }
                _ => Ok(None),
            },
            Heard::Failed(fault) => Err(self.error(failure::transport(fault))),
            Heard::Done(body) => self.done(body, bounds, word, tables),
        }
    }
    fn done(
        &mut self,
        body: Body,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        if self.status == 409
            && let Some(answer) = self.conflict()
        {
            return Ok(Some(Step::Done(answer)));
        }
        if self.status != self.expected() {
            let refusal = match &body {
                Body::Document(d::Document::Error(error)) => match &error.message {
                    d::Body::Kept(message) => crate::forgejo::failure::refusal(self.status, message),
                    d::Body::Long(_) => None,
                },
                Body::Document(_) | Body::Empty | Body::Raw { .. } | Body::Malformed(_) => None,
            };
            let error = failure::refused(self.asked(), self.status, refusal, self.retry, self.repository, tables);
            return Ok(Some(Step::Done(Err(self.error(error)))));
        }
        let document = match body {
            Body::Document(document) => Some(document),
            Body::Empty => None,
            Body::Malformed(error) => {
                return Err(self.error(if self.asked() == Asked::BeforeWrite {
                    failure::malformed(error)
                } else {
                    api::Error::Timeout
                }));
            }
            Body::Raw { .. } => return Err(self.error(api::Error::Timeout)),
        };
        self.check_pull_ids(document.as_ref())?;
        self.check_repository(document.as_ref())?;
        if let Some(document) = &document {
            crate::forgejo::plans::reads::keep::keep_document(document, self.repository.forge, tables);
        }
        let errors = self.errors();
        let state = core::mem::replace(&mut self.state, State::Ended);
        let step;
        (self.state, step) = match next(state, document, self.repository, bounds, word, tables) {
            Ok(next) => next,
            Err(error) => return Err(errors.error(error)),
        };
        Ok(Some(step))
    }
    fn conflict(&self) -> Option<Result<api::Answer, api::Error>> {
        match &self.state {
            State::Writing { written, .. } => match written {
                Written::Branch => Some(Ok(api::Answer::Branch(api::BranchCreation::Exists))),
                Written::Update(api::UpdateStyle::Rebase) => Some(Err(api::Error::Conflict)),
                Written::Issue
                | Written::Comment
                | Written::Review
                | Written::Edited
                | Written::Status
                | Written::Update(api::UpdateStyle::Merge)
                | Written::Bare
                | Written::Deleted
                | Written::Pull
                | Written::Merge { .. } => None,
            },
            State::New { .. }
            | State::Resolving { .. }
            | State::Merged { .. }
            | State::OpenHead { .. }
            | State::OpenBase { .. }
            | State::OpenCompare { .. }
            | State::Reviewers { .. }
            | State::Searching { .. }
            | State::Adding { .. }
            | State::Removing { .. } => None,
            State::Ended => unreachable!("ended write has no conflict"),
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn check_repository(&self, document: Option<&d::Document>) -> Result<(), api::Error> {
        match &self.state {
            State::Resolving { .. } => {}
            State::New { .. }
            | State::Writing { .. }
            | State::Merged { .. }
            | State::OpenHead { .. }
            | State::OpenBase { .. }
            | State::OpenCompare { .. }
            | State::Reviewers { .. }
            | State::Searching { .. }
            | State::Adding { .. }
            | State::Removing { .. } => return Ok(()),
            State::Ended => unreachable!("ended write has no repository answer"),
        }
        match document.ok_or(self.error(api::Error::Timeout))? {
            d::Document::Repository(repository) => {
                let Ok(id) = u32::try_from(repository.id) else {
                    return Err(api::Error::TooLarge(api::Excess::exact(
                        api::Bound::Forge,
                        u64::from(u32::MAX),
                        repository.id,
                    )));
                };
                if id != self.repository.repository {
                    return Err(self.error(api::Error::Timeout));
                }
                Ok(())
            }
            _ => Err(self.error(api::Error::Timeout)),
        }
    }
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn check_pull_ids(&self, document: Option<&d::Document>) -> Result<(), api::Error> {
        let Some(document) = document else { return Ok(()) };
        match document {
            d::Document::Pull(pull) => {
                let row = match &self.state {
                    State::Writing { row, .. }
                    | State::Merged { row }
                    | State::OpenHead { row, .. }
                    | State::OpenBase { row, .. }
                    | State::OpenCompare { row, .. }
                    | State::Reviewers { row, .. }
                    | State::Searching { row, .. }
                    | State::Adding { row, .. }
                    | State::Removing { row } => row,
                    State::New { .. } | State::Resolving { .. } | State::Ended => {
                        unreachable!("pull answer needs a resolved repository")
                    }
                };
                let format = row.object_format;
                for id in [pull.head.sha, pull.base.sha, pull.merge_base, pull.merge_commit_sha].into_iter().flatten() {
                    if reading::commit(id, format).is_none() {
                        return Err(self.error(api::Error::Timeout));
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    #[must_use]
    pub fn error(&self, error: api::Error) -> api::Error {
        self.errors().error(error)
    }
}
fn initial(row: Repository, write: api::Write, bounds: Bounds, word: &[u8]) -> Result<(State, Step), api::Error> {
    let at = row.at();
    let written = writing(&write);
    let request = match write {
        api::Write::Dispatch { workflow, branch, inputs } => Request::Dispatch { at, workflow, branch, inputs },
        api::Write::Rerun { attempt: job } => Request::Rerun { at, job: job.job },
        api::Write::CreateIssue { key, title, body } => {
            Request::CreateIssue { at, title, body: marked(word, &key, &body, bounds)? }
        }
        api::Write::Post { number, key, body } => {
            Request::PostComment { at, number, body: marked(word, &key, &body, bounds)? }
        }
        api::Write::Review { number, key, verdict, body } => {
            let event = match verdict {
                api::Verdict::Approve => r::ReviewEvent::Approved,
                api::Verdict::RequestChanges => r::ReviewEvent::RequestChanges,
                api::Verdict::Comment => r::ReviewEvent::Comment,
            };
            Request::PostReview { at, number, event, body: marked(word, &key, &body, bounds)? }
        }
        api::Write::Edit { number, title, body } => Request::EditIssue { at, number, title, body, state: None },
        api::Write::Close { number } => {
            Request::EditIssue { at, number, title: None, body: None, state: Some(d::IssueState::Closed) }
        }
        api::Write::Reopen { number } => {
            Request::EditIssue { at, number, title: None, body: None, state: Some(d::IssueState::Open) }
        }
        api::Write::Update { number, style } => Request::Update {
            at,
            number,
            style: Some(match style {
                api::UpdateStyle::Merge => r::UpdateStyle::Merge,
                api::UpdateStyle::Rebase => r::UpdateStyle::Rebase,
            }),
        },
        api::Write::Status { commit, context, check, description, link } => {
            let state = match check {
                api::Check::Pending => d::StatusState::Pending,
                api::Check::Passed => d::StatusState::Success,
                api::Check::Failed => d::StatusState::Failure,
            };
            Request::PostStatus { at, commit: row.id(commit), state, context, description, link }
        }
        api::Write::CreateBranch { branch, commit } => Request::CreateBranch { at, branch, from: row.id(commit) },
        api::Write::DeleteBranch { branch } => Request::DeleteBranch { at, branch },
        api::Write::OpenPull { title, body, head, base } => {
            let request = Request::Branch { at, name: head.clone() };
            return Ok((State::OpenHead { row, open: Open { title, body, head, base } }, Step::send(request)));
        }
        api::Write::SetReviewers { number, reviewers } => {
            return Ok((State::Reviewers { row, number, reviewers }, Step::send(Request::Pull { at, number })));
        }
        api::Write::Merge { number, head, how } => Request::Merge {
            at,
            number,
            style: match how {
                api::MergeHow::Style => row.merge_style,
                api::MergeHow::FastForwardOnly => d::MergeStyle::FastForwardOnly,
            },
            head: row.id(head),
        },
    };
    let Some(written) = written else { unreachable!("reviewer exchanges have their own stages") };
    Ok((State::Writing { row, written }, Step::send(request)))
}
impl Written {
    fn exchange(self) -> (Asked, u16) {
        match self {
            Self::Pull => (Asked::OpenPull, 201),
            Self::Merge { .. } => (Asked::Merge, 200),
            Self::Update(_) => (Asked::Update, 200),
            Self::Deleted => (Asked::DeleteBranch, 204),
            Self::Review => (Asked::Write, 200),
            Self::Bare => (Asked::Write, 204),
            Self::Issue | Self::Comment | Self::Edited | Self::Status | Self::Branch => (Asked::Write, 201),
        }
    }
}
fn asked(write: &api::Write) -> Asked {
    match writing(write) {
        Some(written) => written.exchange().0,
        None => Asked::Write,
    }
}
fn writing(write: &api::Write) -> Option<Written> {
    Some(match write {
        api::Write::CreateIssue { .. } => Written::Issue,
        api::Write::Post { .. } => Written::Comment,
        api::Write::Review { .. } => Written::Review,
        api::Write::Edit { .. } | api::Write::Close { .. } | api::Write::Reopen { .. } => Written::Edited,
        api::Write::Status { .. } => Written::Status,
        api::Write::CreateBranch { .. } => Written::Branch,
        api::Write::Dispatch { .. } | api::Write::Rerun { .. } => Written::Bare,
        api::Write::DeleteBranch { .. } => Written::Deleted,
        api::Write::Update { style, .. } => Written::Update(*style),
        api::Write::OpenPull { .. } => Written::Pull,
        api::Write::Merge { number, .. } => Written::Merge { number: *number },
        api::Write::SetReviewers { .. } => return None,
    })
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn next(
    state: State,
    document: Option<d::Document>,
    repository: api::Repository,
    bounds: Bounds,
    word: &[u8],
    tables: &mut Tables,
) -> Result<(State, Step), api::Error> {
    match state {
        State::Resolving { write } => match document.ok_or(api::Error::Timeout)? {
            d::Document::Repository(document) => {
                let row = reading::repository(document);
                tables.keep_repository(repository, row.clone());
                initial(row, write, bounds, word)
            }
            _ => Err(api::Error::Timeout),
        },
        State::Writing { row, written: kind } => written(row, kind, document),
        State::OpenHead { row, open } => {
            let head = branch(&row, document)?;
            let request = Request::Branch { at: row.at(), name: open.base.clone() };
            Ok((State::OpenBase { row, open, head }, Step::send(request)))
        }
        State::OpenBase { row, open, head } => {
            let before = branch(&row, document)?;
            let request = Request::Compare { at: row.at(), before, after: head, files: false };
            Ok((State::OpenCompare { row, open }, Step::send(request)))
        }
        State::OpenCompare { row, open } => compared(row, open, document),
        State::Reviewers { row, number, reviewers: desired } => match document.ok_or(api::Error::Timeout)? {
            d::Document::Pull(pull) => Ok(reviewers(row, number, &desired, pull, repository.forge, tables)),
            _ => Err(api::Error::Timeout),
        },
        State::Searching { row, number, mut looking } => {
            match document.ok_or(api::Error::Missing)? {
                d::Document::Search { ok: true } => {}
                _ => return Err(api::Error::Missing),
            }
            let id = *looking.missing.get(looking.position).expect("looked-up reviewer");
            let Some(login) = tables.user(repository.forge, id) else {
                return Err(api::Error::Missing);
            };
            looking.add.push(Box::from(login)).expect("missing count");
            looking.position = looking.position.saturating_add(1);
            Ok(lookup(row, number, looking, repository.forge, tables))
        }
        State::Adding { row, number, remove } => Ok(remove_reviewers(row, number, remove)),
        State::Removing { .. } => Ok((State::Ended, Step::Done(Ok(api::Answer::Done)))),
        State::Merged { row } => match document.ok_or(api::Error::Timeout)? {
            d::Document::Pull(pull) => {
                let commit = pull.merge_commit_sha.ok_or(api::Error::Timeout)?;
                let commit = reading::commit(commit, row.object_format).ok_or(api::Error::Timeout)?;
                Ok((State::Ended, Step::Done(Ok(api::Answer::Merged(commit)))))
            }
            _ => Err(api::Error::Timeout),
        },
        State::New { .. } | State::Ended => unreachable!("write answer needs an exchange"),
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn compared(row: Repository, open: Open, document: Option<d::Document>) -> Result<(State, Step), api::Error> {
    match document.ok_or(api::Error::InvalidAnswer)? {
        d::Document::Compare(compare) => {
            if compare.total_commits == 0 {
                return Ok((State::Ended, Step::Done(Err(api::Error::NothingToMerge))));
            }
            let Open { title, body, head, base } = open;
            let request = Request::OpenPull { at: row.at(), title, body, head, base };
            Ok((State::Writing { row, written: Written::Pull }, Step::send(request)))
        }
        _ => Err(api::Error::InvalidAnswer),
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn branch(row: &Repository, document: Option<d::Document>) -> Result<ObjectId, api::Error> {
    match document.ok_or(api::Error::InvalidAnswer)? {
        d::Document::Branch(branch) => {
            if reading::commit(branch.commit, row.object_format).is_none() {
                return Err(api::Error::InvalidAnswer);
            }
            Ok(branch.commit)
        }
        _ => Err(api::Error::InvalidAnswer),
    }
}
fn reviewers(
    row: Repository,
    number: u64,
    desired: &[u64],
    pull: d::PullRequest,
    forge: u16,
    tables: &mut Tables,
) -> (State, Step) {
    let capacity = u32::try_from(desired.len()).expect("bounded op");
    let mut looking = Looking {
        missing: List::with_capacity(capacity),
        position: 0,
        add: List::with_capacity(capacity),
        remove: List::with_capacity(u32::try_from(pull.requested_reviewers.len()).expect("bounded inner")),
    };
    for id in desired {
        let mut found = false;
        for user in &pull.requested_reviewers {
            found |= user.id == *id;
        }
        if !found && !looking.missing.as_slice().contains(id) {
            looking.missing.push(*id).expect("desired count");
        }
    }
    for user in pull.requested_reviewers {
        if !desired.contains(&user.id) {
            looking.remove.push(user.login).expect("current count");
        }
    }
    lookup(row, number, looking, forge, tables)
}
fn lookup(row: Repository, number: u64, mut looking: Looking, forge: u16, tables: &mut Tables) -> (State, Step) {
    for _ in looking.position..looking.missing.len() {
        let id = *looking.missing.get(looking.position).expect("missing reviewer");
        let Some(login) = tables.user(forge, id) else {
            return (
                State::Searching { row, number, looking },
                Step::send(Request::UserSearch { uid: id, limit: Some(1) }),
            );
        };
        looking.add.push(Box::from(login)).expect("missing count");
        looking.position = looking.position.saturating_add(1);
    }
    if looking.add.is_empty() {
        return remove_reviewers(row, number, looking.remove);
    }
    let request = Request::RequestReviewers { at: row.at(), number, reviewers: looking.add.into_boxed() };
    (State::Adding { row, number, remove: looking.remove }, Step::send(request))
}
fn remove_reviewers(row: Repository, number: u64, remove: List<Box<[u8]>>) -> (State, Step) {
    if remove.is_empty() {
        return (State::Ended, Step::Done(Ok(api::Answer::Done)));
    }
    let request = Request::RemoveReviewers { at: row.at(), number, reviewers: remove.into_boxed() };
    (State::Removing { row }, Step::send(request))
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn written(row: Repository, written: Written, document: Option<d::Document>) -> Result<(State, Step), api::Error> {
    let answer = match written {
        Written::Issue => match document.ok_or(api::Error::Timeout)? {
            d::Document::Issue(issue) => Ok(api::Answer::Created(issue.number)),
            _ => Err(api::Error::Timeout),
        },
        Written::Comment => match document.ok_or(api::Error::Timeout)? {
            d::Document::Comment(comment) => Ok(api::Answer::Commented(comment.id)),
            _ => Err(api::Error::Timeout),
        },
        Written::Review => match document.ok_or(api::Error::Timeout)? {
            d::Document::Review(review) => {
                reading::commit(review.commit_id, row.object_format).ok_or(api::Error::Timeout)?;
                Ok(api::Answer::Reviewed(review.id))
            }
            _ => Err(api::Error::Timeout),
        },
        Written::Edited => match document.ok_or(api::Error::Timeout)? {
            d::Document::Issue(_) => Ok(api::Answer::Done),
            _ => Err(api::Error::Timeout),
        },
        Written::Status => match document.ok_or(api::Error::Timeout)? {
            d::Document::Status(_) => Ok(api::Answer::Done),
            _ => Err(api::Error::Timeout),
        },
        Written::Branch => match document.ok_or(api::Error::Timeout)? {
            d::Document::Branch(branch) => {
                reading::commit(branch.commit, row.object_format).ok_or(api::Error::Timeout)?;
                Ok(api::Answer::Branch(api::BranchCreation::Created))
            }
            _ => Err(api::Error::Timeout),
        },
        Written::Bare | Written::Update(_) | Written::Deleted => match document {
            None => Ok(api::Answer::Done),
            Some(_) => Err(api::Error::Timeout),
        },
        Written::Pull => match document.ok_or(api::Error::Timeout)? {
            d::Document::Pull(pull) => Ok(api::Answer::Created(pull.number)),
            _ => Err(api::Error::Timeout),
        },
        Written::Merge { number } => {
            if document.is_some() {
                return Err(api::Error::Timeout);
            }
            let request = Request::Pull { at: row.at(), number };
            return Ok((State::Merged { row }, Step::send(request)));
        }
    }?;
    Ok((State::Ended, Step::Done(Ok(answer))))
}

fn marked(word: &[u8], key: &[u8], body: &[u8], bounds: Bounds) -> Result<Box<[u8]>, api::Error> {
    marker::encode(word, key, body, bounds.body_bytes).ok_or(api::Error::Refused)
}
fn text(bytes: &[u8]) -> Result<(), api::Error> {
    let l = skein_json::writer::Limits { length: u32::MAX, depth: 1 };
    let mut json = skein_json::writer::Encoder::measure(&l);
    json.string(bytes);
    match json.measured() {
        Ok(_) => Ok(()),
        Err(_) => Err(api::Error::Refused),
    }
}
fn validate(write: &api::Write) -> Result<(), api::Error> {
    match write {
        api::Write::CreateIssue { title, body, .. } => {
            text(title)?;
            text(body)
        }
        api::Write::OpenPull { title, body, head, base } => {
            text(title)?;
            text(body)?;
            text(head)?;
            text(base)
        }
        api::Write::Post { body, .. } | api::Write::Review { body, .. } => text(body),
        api::Write::Edit { title, body, .. } => {
            if let Some(title) = title {
                text(title)?;
            }
            if let Some(body) = body {
                text(body)?;
            }
            Ok(())
        }
        api::Write::Status { context, description, link, .. } => {
            text(context)?;
            text(description)?;
            text(link)
        }
        api::Write::Dispatch { workflow, branch, inputs } => {
            text(workflow)?;
            text(branch)?;
            for input in inputs {
                text(&input.name)?;
                text(&input.value)?;
            }
            Ok(())
        }
        api::Write::CreateBranch { branch, .. } | api::Write::DeleteBranch { branch } => text(branch),
        api::Write::Rerun { .. }
        | api::Write::SetReviewers { .. }
        | api::Write::Close { .. }
        | api::Write::Reopen { .. }
        | api::Write::Merge { .. }
        | api::Write::Update { .. } => Ok(()),
    }
}
