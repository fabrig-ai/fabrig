//! Terminal documents and the next exchange of a JSON read (skein/docs/design/forge/forgejo.md, 1.2).
use super::{
    Bounds, Plan, Request, Roster, State, Step, Tables, Whose, api, bytes, d, keep_document, reading, size, terminal,
};
use crate::api::Excess;
use alloc::boxed::Box;
use skein_lib::List;
impl Plan {
    pub(super) fn body(
        &mut self,
        state: State,
        body: super::Body,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<(State, Option<Step>), api::Error> {
        match state {
            State::Branches { missing: true, .. } => ended(api::Answer::Branches(Box::new([])), bounds),
            State::Ended => unreachable!("owner stops feeding an ended read plan"),
            state @ (State::New { .. }
            | State::Resolving { .. }
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
            | State::Searching { .. }) => {
                let document = match body {
                    super::Body::Document(document) => document,
                    super::Body::Malformed(crate::forgejo::decode::Malformed::Bound { field }) => {
                        return Err(if field == b"log" {
                            api::Error::InvalidAnswer
                        } else {
                            api::Error::TooLarge(Excess::unnamed())
                        });
                    }
                    super::Body::Malformed(_) | super::Body::Empty | super::Body::Raw { .. } => {
                        return Err(api::Error::InvalidAnswer);
                    }
                };
                self.done(state, document, bounds, word, tables)
            }
        }
    }
    #[expect(clippy::too_many_lines, reason = "each read state consumes its exchange's terminal document")]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
    )]
    fn done(
        &mut self,
        state: State,
        document: d::Document,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<(State, Option<Step>), api::Error> {
        check_repository(&state, &document, self.repository.repository)?;
        keep_document(&document, self.repository.forge, tables);
        let answer = match state {
            State::Resolving { read } => {
                let repository = repository(document)?;
                let row = reading::repository(repository);
                tables.keep_repository(self.repository, row.clone());
                let (state, step) = self.initial(read, row, bounds, tables)?;
                return Ok((state, Some(step)));
            }
            State::Locating => {
                let repository = self.keep_repository(document, tables)?;
                api::Answer::Located { repository: u32::try_from(repository.id).expect("repository checked") }
            }
            State::Settings => api::Answer::Settings(reading::settings(self.keep_repository(document, tables)?)),
            State::Issue { row, after, since } => {
                let item = match document {
                    d::Document::Issue(issue) => reading::summary(word, issue),
                    _ => return Err(api::Error::InvalidAnswer),
                };
                let size = size::summary(&item, bounds.rows)?;
                if size > u64::from(bounds.answer_bytes) {
                    return Err(api::Error::TooLarge(Excess {
                        bound: api::Bound::AnswerBytes,
                        limit: u64::from(bounds.answer_bytes),
                        size,
                        exact: false,
                    }));
                }
                let room = u64::from(bounds.answer_bytes)
                    .saturating_sub(size)
                    .checked_div(u64::try_from(crate::measure::COMMENT_BYTES).expect("comment width"))
                    .expect("nonzero width");
                let rows = List::with_capacity(bounds.rows.min(u32::try_from(room).expect("bounded room")));
                let step = self.send(Request::Comments { at: row.at(), number: item.number, since });
                return Ok((State::Comments { item, rows, more: false, size, after }, Some(step)));
            }
            State::Pull { row, ci } => return Self::pull_status(row, pull(document)?, bounds, ci),
            State::PullFor { row, head, base } => {
                let pull = pull(document)?;
                if pull.state == d::IssueState::Open {
                    return Self::pull_status(row, pull, bounds, false);
                }
                let step = self.send(Request::Pulls {
                    at: row.at(),
                    head: Some(head.clone()),
                    base: Some(base.clone()),
                    page: super::r::Page { page: Some(1), limit: Some(bounds.page_size) },
                });
                return Ok((State::Pulls { row, head, base, pull: Box::new(pull), found: false, page: 1 }, Some(step)));
            }
            State::Pulls { row, head, base, pull, found, page } => {
                let skipped = listed(document)?;
                if !found && self.count.saturating_add(skipped) >= bounds.page_size {
                    let page = page.checked_add(1).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
                    let step = self.send(Request::Pulls {
                        at: row.at(),
                        head: Some(head.clone()),
                        base: Some(base.clone()),
                        page: super::r::Page { page: Some(page), limit: Some(bounds.page_size) },
                    });
                    return Ok((State::Pulls { row, head, base, pull, found, page }, Some(step)));
                }
                return Self::pull_status(row, *pull, bounds, false);
            }
            State::Branch { format } => match document {
                d::Document::Branch(branch) => {
                    api::Answer::Commit(reading::commit(branch.commit, format).ok_or(api::Error::InvalidAnswer)?)
                }
                _ => return Err(api::Error::InvalidAnswer),
            },
            State::ProtectionBranch { row } => {
                let protection = match document {
                    d::Document::Branch(branch) => branch.effective_branch_protection_name,
                    _ => return Err(api::Error::InvalidAnswer),
                };
                let request = match protection {
                    Some(name) => Request::Protection { at: row.at(), name },
                    None => Request::Protections { at: row.at() },
                };
                return Ok((State::Protection, Some(self.send(request))));
            }
            State::Protection => match document {
                d::Document::Protection(protection) => api::Answer::Protection(Some(reading::protection(protection))),
                d::Document::Listed { .. } => api::Answer::Protection(None),
                _ => return Err(api::Error::InvalidAnswer),
            },
            State::Permission { row, whose } => {
                let permission = match document {
                    d::Document::Permission(permission) => api::UserPermission {
                        user: permission.user.id,
                        login: permission.user.login,
                        permission: reading::permission(permission.permission),
                    },
                    _ => return Err(api::Error::InvalidAnswer),
                };
                match whose {
                    Whose::One => api::Answer::Permission(permission),
                    Whose::Each(mut roster) => {
                        roster.rows.get_mut(roster.position).ok_or(api::Error::InvalidAnswer)?.permission =
                            permission.permission;
                        roster.position = roster.position.saturating_add(1);
                        return Ok(self.next_permission(row, roster, tables));
                    }
                }
            }
            State::Searching { row, user, whose } => {
                match document {
                    d::Document::Search { ok } => {
                        if !ok || self.count == 0 {
                            return Err(api::Error::Missing);
                        }
                    }
                    _ => return Err(api::Error::InvalidAnswer),
                }
                let login = tables.user(self.repository.forge, user).ok_or(api::Error::Missing)?;
                let step = self.send(Request::Permission { at: row.at(), login: bytes::copy_of(login) });
                return Ok((State::Permission { row, whose }, Some(step)));
            }
            State::Collaborators { row, rows, .. } => {
                listed(document)?;
                return Ok(self.next_permission(
                    row,
                    Roster { rows, position: 0, more: self.count >= bounds.page_size },
                    tables,
                ));
            }
            State::Items { rows, now, cut, .. } => api::Answer::Items {
                items: rows.into_boxed(),
                more: self.count.saturating_add(listed(document)?) >= self.bound.expect("items are explicitly paged")
                    || cut,
                cut,
                now,
            },
            State::Reviews { rows, cut, .. } => api::Answer::Reviews {
                reviews: rows.into_boxed(),
                more: self.count.saturating_add(listed(document)?) >= bounds.page_size || cut,
                cut,
            },
            State::Remarks { rows, more, cut, .. } => {
                listed(document)?;
                api::Answer::Remarks { remarks: rows.into_boxed(), more, cut }
            }
            State::Comments { item, rows, more, .. } => {
                listed(document)?;
                api::Answer::Item { item, comments: rows.into_boxed(), more }
            }
            State::Branches { rows, .. } => {
                listed(document)?;
                api::Answer::Branches(rows.into_boxed())
            }
            State::New { .. } => return Err(api::Error::InvalidAnswer),
            State::Delegated(_) | State::Status { .. } => {
                unreachable!("delegates consume their own terminal documents")
            }
            State::Ended => unreachable!("owner stops feeding an ended read plan"),
        };
        ended(answer, bounds)
    }
    fn keep_repository(&self, document: d::Document, tables: &mut Tables) -> Result<d::Repository, api::Error> {
        let repository = repository(document)?;
        let id = u32::try_from(repository.id).expect("repository checked");
        tables.keep_repository(
            api::Repository { forge: self.repository.forge, repository: id },
            reading::repository(repository.clone()),
        );
        Ok(repository)
    }
    fn pull_status(
        row: super::Repository,
        pull: d::PullRequest,
        bounds: Bounds,
        ci: bool,
    ) -> Result<(State, Option<Step>), api::Error> {
        if !ci {
            let pull = reading::pull(pull, row.object_format, api::Ci::None)?;
            return Ok((State::Ended, Some(terminal(api::Answer::Pull(pull), bounds)?)));
        }
        let commit = pull.head.sha.ok_or(api::Error::InvalidAnswer)?;
        if commit.format != row.object_format {
            return Err(api::Error::InvalidAnswer);
        }
        let mut actions = super::super::actions::Plan::statuses(commit.bytes, row.clone(), bounds)?;
        let step = actions.begin();
        Ok((State::Status { row, pull: Box::new(pull), actions: Box::new(actions) }, Some(step)))
    }
    fn next_permission(
        &mut self,
        row: super::Repository,
        roster: Roster,
        tables: &mut Tables,
    ) -> (State, Option<Step>) {
        if let Some(user) = roster.rows.get(roster.position) {
            let user = user.user;
            let (state, step) = self.permission(row, user, Whose::Each(roster), tables);
            return (state, Some(step));
        }
        (
            State::Ended,
            Some(Step::Done(Ok(api::Answer::Collaborators {
                collaborators: roster.rows.into_boxed(),
                more: roster.more,
            }))),
        )
    }
}
fn ended(answer: api::Answer, bounds: Bounds) -> Result<(State, Option<Step>), api::Error> {
    Ok((State::Ended, Some(terminal(answer, bounds)?)))
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn repository(document: d::Document) -> Result<d::Repository, api::Error> {
    match document {
        d::Document::Repository(repository) => Ok(repository),
        _ => Err(api::Error::InvalidAnswer),
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn pull(document: d::Document) -> Result<d::PullRequest, api::Error> {
    match document {
        d::Document::Pull(pull) => Ok(pull),
        _ => Err(api::Error::InvalidAnswer),
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn listed(document: d::Document) -> Result<u32, api::Error> {
    match document {
        d::Document::Listed { skipped } => Ok(skipped),
        _ => Err(api::Error::InvalidAnswer),
    }
}
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the exchange's decoder picks the kind: forge/README.status.md, 4.2"
)]
fn check_repository(state: &State, document: &d::Document, expected: u32) -> Result<(), api::Error> {
    let id = match document {
        d::Document::Repository(repository) => repository.id,
        _ => return Ok(()),
    };
    let Ok(id) = u32::try_from(id) else {
        return Err(api::Error::TooLarge(Excess::exact(api::Bound::Forge, u64::from(u32::MAX), id)));
    };
    let by_id = match state {
        State::Resolving { .. } | State::Settings => true,
        State::New { .. }
        | State::Locating
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
        | State::Searching { .. } => false,
        State::Ended => unreachable!("owner stops feeding an ended read plan"),
    };
    if id == 0 || (by_id && id != expected) {
        return Err(api::Error::InvalidAnswer);
    }
    Ok(())
}
