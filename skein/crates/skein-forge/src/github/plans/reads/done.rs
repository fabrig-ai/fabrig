//! Repository resolution and terminal documents for GitHub's first reads.
use super::{Bounds, Plan, Read, Response, Stage, Step, Tables, api, document, tables};
use crate::api::Excess;
use crate::{github::id, measure};
use skein_lib::List;

impl Plan {
    pub(super) fn done(
        &mut self,
        document: document::Document,
        bounds: Bounds,
        word: &[u8],
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        match document {
            document::Document::Ruleset(bypass) => self.ruleset_done(bypass),
            document::Document::ClassicProtection(value) => self.classic_done(value, bounds),
            document::Document::PullGraphql(pull) => self.pull_graphql_done(pull, bounds),
            document::Document::Repository(repository) => self.repository_done(repository, bounds, tables),
            document::Document::Branch(branch) => self.branch_document(branch, bounds),
            document::Document::Compare(comparison) => {
                if let Read::Commits { .. } = self.read {
                    self.commits_done(comparison, bounds)
                } else {
                    self.compare_done(comparison, bounds)
                }
            }
            document::Document::User(user) => self.user_done(user, tables),
            document::Document::Permission(value) => self.permission_done(value, bounds),
            document::Document::Pull(pull) => {
                if let Read::PullFiles { .. } = self.read {
                    self.file_pull_done(pull)
                } else {
                    self.pull_done(pull, bounds)
                }
            }
            document::Document::Job(job) => self.job_document(job, bounds),
            document::Document::Combined(combined) => match self.read {
                Read::Checks { .. } => self.checks_combined_done(),
                Read::Locate { .. }
                | Read::Settings
                | Read::Branch { .. }
                | Read::Branches { .. }
                | Read::Items { .. }
                | Read::Item { .. }
                | Read::Pull { .. }
                | Read::PullFor { .. }
                | Read::Statuses { .. }
                | Read::Permission { .. }
                | Read::Collaborators { .. }
                | Read::Remarks { .. }
                | Read::Reviews { .. }
                | Read::File { .. }
                | Read::Compare { .. }
                | Read::Commits { .. }
                | Read::PullFiles { .. }
                | Read::Protection { .. }
                | Read::Job { .. } => self.combined_done(combined, bounds),
            },
            document::Document::Issue(issue) => self.issue_done(issue, bounds, word),
            document::Document::Rows => self.rows_document(bounds),
            document::Document::Error(_)
            | document::Document::Delivery(_)
            | document::Document::Installation(_)
            | document::Document::Minted(_)
            | document::Document::Viewer(_)
            | document::Document::Graphql(_)
            | document::Document::App(_)
            | document::Document::HookConfig(_)
            | document::Document::Conversion(_)
            | document::Document::Id(_)
            | document::Document::Number(_)
            | document::Document::Done
            | document::Document::Merge(_)
            | document::Document::Enqueued(_)
            | document::Document::BotUser(_) => Err(api::Error::InvalidAnswer),
        }
    }
    fn branch_document(&mut self, branch: document::Branch, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        if self.stage != Stage::Read {
            return Err(api::Error::InvalidAnswer);
        }
        match self.read {
            Read::Branch { .. } => self.terminal(api::Answer::Commit(branch.commit), bounds),
            Read::Items { .. }
            | Read::Item { .. }
            | Read::Locate { .. }
            | Read::Settings
            | Read::Branches { .. }
            | Read::Pull { .. }
            | Read::PullFor { .. }
            | Read::Checks { .. }
            | Read::Statuses { .. }
            | Read::Permission { .. }
            | Read::Collaborators { .. }
            | Read::Remarks { .. }
            | Read::Reviews { .. }
            | Read::File { .. }
            | Read::Compare { .. }
            | Read::Commits { .. }
            | Read::PullFiles { .. }
            | Read::Protection { .. }
            | Read::Job { .. } => Err(api::Error::InvalidAnswer),
        }
    }
    fn rows_document(&mut self, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        match self.read {
            Read::File { .. } => self.directory_done(bounds),
            Read::Protection { .. } => self.protection_rows_done(),
            Read::Checks { .. } => self.checks_runs_done(bounds),
            Read::PullFiles { .. } => self.file_listing_done(bounds),
            Read::Reviews { .. } => self.reviews_done(bounds),
            Read::Remarks { .. } => self.remarks_done(bounds),
            Read::Collaborators { .. } => self.collaborators_done(bounds),
            Read::Items { .. } | Read::Item { .. } => self.issue_rows_done(bounds),
            Read::Pull { .. } | Read::PullFor { .. } | Read::Statuses { .. } => self.pull_rows_done(bounds),
            Read::Locate { .. }
            | Read::Settings
            | Read::Branch { .. }
            | Read::Branches { .. }
            | Read::Permission { .. }
            | Read::Compare { .. }
            | Read::Commits { .. }
            | Read::Job { .. } => self.branches_done(bounds),
        }
    }
    fn repository_done(
        &mut self,
        repository: document::Repository,
        bounds: Bounds,
        tables: &mut Tables,
    ) -> Result<Option<Step>, api::Error> {
        let id = id::repository(repository.id)?;
        if id == 0
            || ((self.stage == Stage::Resolve || self.read == Read::Settings) && id != self.repository.repository)
        {
            return Err(api::Error::InvalidAnswer);
        }
        let answer = match &self.read {
            Read::Locate { .. } => Some(api::Answer::Located { repository: id }),
            Read::Settings => Some(api::Answer::Settings(api::Settings {
                default_branch: repository.default_branch.clone(),
                merge: repository.merge,
                squash: repository.squash,
                rebase: repository.rebase,
            })),
            Read::Items { .. }
            | Read::Item { .. }
            | Read::Branch { .. }
            | Read::Branches { .. }
            | Read::Pull { .. }
            | Read::PullFor { .. }
            | Read::Checks { .. }
            | Read::Statuses { .. }
            | Read::Permission { .. }
            | Read::Collaborators { .. }
            | Read::Remarks { .. }
            | Read::Reviews { .. }
            | Read::File { .. }
            | Read::Compare { .. }
            | Read::Commits { .. }
            | Read::PullFiles { .. }
            | Read::Protection { .. }
            | Read::Job { .. }
                if self.stage == Stage::Resolve =>
            {
                None
            }
            Read::Items { .. }
            | Read::Item { .. }
            | Read::Branch { .. }
            | Read::Branches { .. }
            | Read::Pull { .. }
            | Read::PullFor { .. }
            | Read::Checks { .. }
            | Read::Statuses { .. }
            | Read::Permission { .. }
            | Read::Collaborators { .. }
            | Read::Remarks { .. }
            | Read::Reviews { .. }
            | Read::File { .. }
            | Read::Compare { .. }
            | Read::Commits { .. }
            | Read::PullFiles { .. }
            | Read::Protection { .. }
            | Read::Job { .. } => {
                return Err(api::Error::InvalidAnswer);
            }
        };
        let row = tables::repository(&repository);
        let key = api::Repository { forge: self.repository.forge, repository: id };
        tables.keep_repository(key, row.clone());
        // Repository documents carry the owner's id and login, as Forgejo's do.
        tables.keep_user(self.repository.forge, repository.owner_id, repository.owner);
        self.row = Some(row);
        match answer {
            Some(answer) => self.terminal(answer, bounds),
            None => Ok(Some(self.initial()?)),
        }
    }
    fn branches_done(&mut self, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        if self.stage != Stage::Read {
            return Err(api::Error::InvalidAnswer);
        }
        let Read::Branches { .. } = &self.read else {
            return Err(api::Error::InvalidAnswer);
        };
        let next = match self.response {
            Response::Body { next } => next,
            Response::None | Response::Head | Response::Refusal { .. } => return Err(api::Error::InvalidAnswer),
        };
        if next {
            self.page = self.page.checked_add(1).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
            return Ok(Some(self.initial()?));
        }
        let rows = core::mem::replace(&mut self.rows, List::with_capacity(0));
        self.terminal(api::Answer::Branches(rows.into_boxed()), bounds)
    }
    pub(super) fn terminal(&mut self, answer: api::Answer, bounds: Bounds) -> Result<Option<Step>, api::Error> {
        let size = measure::answer(&answer).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
        if size > u64::from(bounds.answer_bytes) {
            return Err(api::Error::TooLarge(Excess {
                bound: api::Bound::AnswerBytes,
                limit: u64::from(bounds.answer_bytes),
                size,
                exact: true,
            }));
        }
        self.stage = Stage::Done;
        self.response = Response::None;
        Ok(Some(Step::Done(Ok(answer))))
    }
}
