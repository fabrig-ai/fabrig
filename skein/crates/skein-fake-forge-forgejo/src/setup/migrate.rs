//! Catalogue copies and deletion (skein/docs/design/testing/fakes/forge.md, 4).
use super::{Store, absent, concat, encoded, full, refusal, remove};
use crate::{Config, Hosted, Limits, Repository, response::Reply};
use alloc::boxed::Box;
use skein_fake_forge_domain::api::{Answer, Error, Op, Write};
use skein_forge::forgejo::{document::CreatedRepository, request::Request, server};
use skein_lib::{Time, usize_of};

// Missing-source refusal: copy-facts' setup-migrate-missing-source transcript.
// The private-source fallback uses the same authentication refusal.
pub(crate) const MISSING_SOURCE: &[u8] =
    b"Authentication failed: Clone: exit status 128 - fatal: could not read Username for 'http://127.0.0.1:3001': terminal prompts disabled\n.";
pub(crate) const PRIVATE_SOURCE: &[u8] = MISSING_SOURCE;

pub(crate) fn valid_source(address: &[u8]) -> bool {
    let Some(tail) = address.strip_prefix(b"https://") else { return false };
    let Some(slash) = skein_lib::bytes::find(tail, b"/") else { return false };
    let authority = tail.get(..slash).expect("found separator");
    let host = match skein_lib::bytes::find(authority, b":") {
        None => authority,
        Some(colon) => {
            let port = authority.get(colon.saturating_add(1)..).expect("found colon");
            let Some(port) = skein_forge::forgejo::id::unsigned(port) else { return false };
            if port == 0 || port > u64::from(u16::MAX) {
                return false;
            }
            authority.get(..colon).expect("found colon")
        }
    };
    if !host.ends_with(b".invalid") || host.len() <= b".invalid".len() {
        return false;
    }
    let mut label_start = true;
    let mut hyphen = false;
    for byte in host {
        if *byte == b'.' {
            if label_start || hyphen {
                return false;
            }
            label_start = true;
        } else {
            if (!byte.is_ascii_alphanumeric() && *byte != b'-') || (label_start && *byte == b'-') {
                return false;
            }
            label_start = false;
        }
        hyphen = *byte == b'-';
    }
    !label_start && !hyphen
}

impl Store {
    pub(crate) fn migration(
        &mut self,
        request: &Request,
        user: u64,
        config: &mut Config,
        limits: &Limits,
        mut hosted: Option<&mut Hosted<'_>>,
    ) -> Result<(Box<[u8]>, Op), Reply> {
        if !self.resolve_user(config, user).is_admin {
            return Err(refusal(403, b"user should be the site admin", limits));
        }
        let (at, from) = parts(request).expect("only migration and deletion enter here");
        match from {
            Some(from) => {
                if self.organization(&at.owner).is_none() {
                    return Err(absent(limits));
                }
                if !super::creation::repository_name(&at.name) {
                    return Err(refusal(422, b"invalid repository name", limits));
                }
                let destination = concat(&[&at.owner, b"/", &at.name]);
                let domain_taken = match hosted.as_ref() {
                    Some(hosted) => hosted.domain.metadata(&destination).is_ok(),
                    None => false,
                };
                if super::collaborators::repository(config, at).is_some() || domain_taken {
                    return Err(refusal(409, b"repository already exists", limits));
                }
                if config.repositories.room() == 0
                    || self.private.room() == 0
                    || destination.len() > usize_of(limits.documents.target_bytes)
                {
                    return Err(full(limits));
                }
                let Some(id) = self.repository_high.checked_add(1) else { return Err(full(limits)) };
                let mut found = None;
                for source in &config.sources {
                    if source.address.as_ref() == from {
                        found = Some(source);
                    }
                }
                let message = match found {
                    None => Some(MISSING_SOURCE),
                    Some(source) => {
                        if source.private {
                            Some(PRIVATE_SOURCE)
                        } else {
                            None
                        }
                    }
                };
                if let Some(message) = message {
                    let Some(hosted) = hosted.as_mut() else { return Err(full(limits)) };
                    let organization = self.organization(&at.owner).expect("checked organization");
                    let Some(owner) = self.find(&organization.owner) else { return Err(absent(limits)) };
                    if let Err(error) = skein_fake_forge_domain::create_empty_repository(
                        hosted.domain,
                        hosted.config,
                        &destination,
                        owner.id,
                    ) {
                        return Err(crate::response::failure(
                            limits,
                            crate::errors::answer(error, crate::Route::Migrate, Time::ZERO),
                        ));
                    }
                    if hosted.domain.can_grant(&destination, user) == Ok(true) {
                        skein_fake_forge_domain::grant(
                            hosted.domain,
                            &destination,
                            user,
                            skein_fake_forge_domain::api::Permission::Admin,
                        );
                    }
                    config
                        .repositories
                        .push(Repository { id, full: destination, protected: None })
                        .expect("checked room");
                    self.private.push(u64::from(id)).expect("checked private room");
                    self.repository_high = id;
                    return Err(refusal(422, message, limits));
                }
                let source = found.expect("public source");
                let repository = source.repository.clone();
                let op = Op::Write(Write::Copy { owner: at.owner.clone(), name: at.name.clone(), fill: source.fill });
                config.repositories.push(Repository { id, full: destination, protected: None }).expect("checked room");
                self.private.push(u64::from(id)).expect("checked private room");
                self.repository_high = id;
                Ok((repository, op))
            }
            None => {
                let Some(id) = super::collaborators::repository(config, at) else { return Err(absent(limits)) };
                for repository in &config.repositories {
                    if repository.id == id {
                        return Ok((repository.full.clone(), Op::Write(Write::DeleteRepository)));
                    }
                }
                unreachable!("resolved repository row")
            }
        }
    }

    pub(crate) fn migration_reply(
        &mut self,
        request: &Request,
        result: &Result<Answer, Error>,
        config: &mut Config,
        limits: &Limits,
        now: Time,
    ) -> Option<Reply> {
        let (at, from) = parts(request)?;
        let id = super::collaborators::repository(config, at);
        let answer = match result {
            Err(error) => {
                let remove = if from.is_some() { *error != Error::Timeout } else { *error == Error::Timeout };
                if remove && let Some(id) = id {
                    self.drop_repository(config, id);
                }
                return Some(crate::response::failure(limits, crate::errors::for_request(*error, request, now)));
            }
            Ok(answer) => answer,
        };
        if from.is_some() {
            let Some(default_branch) = copied_default(answer) else { return Some(full(limits)) };
            let Some(id) = id else { return Some(absent(limits)) };
            let name = concat(&[&at.owner, b"/", &at.name]);
            Some(encoded(
                201,
                server::Answer::CreatedRepository(CreatedRepository {
                    id: u64::from(id),
                    full_name: name.clone(),
                    default_branch: default_branch.into(),
                    clone_url: concat(&[&config.base, b"/", &name, b".git"]),
                    ssh_url: concat(&[b"ssh://git@localhost/", &name, b".git"]),
                }),
                limits,
            ))
        } else if *answer == Answer::Done {
            if let Some(id) = id {
                self.drop_repository(config, id);
            }
            Some(super::empty())
        } else {
            Some(full(limits))
        }
    }

    fn drop_repository(&mut self, config: &mut Config, id: u32) {
        for index in (0..config.repositories.len()).rev() {
            if config.repositories.get(index).expect("repository index").id == id {
                remove(&mut config.repositories, index);
            }
        }
        for index in (0..self.private.len()).rev() {
            if self.private.get(index) == Some(&u64::from(id)) {
                remove(&mut self.private, index);
            }
        }
        for index in 0..self.accounts.len() {
            let login = self.accounts.get(index).expect("account index").login.clone();
            self.revoke_collaborator(id, &login);
        }
        for (_, login) in &config.users {
            self.revoke_collaborator(id, login);
        }
        for index in (0..self.repo_hooks.len()).rev() {
            if self.repo_hooks.get(index).expect("hook index").repository == id {
                remove(&mut self.repo_hooks, index);
            }
        }
    }
}

fn parts(request: &Request) -> Option<(&skein_forge::forgejo::request::Repo, Option<&[u8]>)> {
    match request {
        Request::Migrate { at, from } => Some((at, Some(from))),
        Request::DeleteRepository { at } => Some((at, None)),
        Request::Artifacts { .. }
        | Request::ArtifactZip { .. }
        | Request::Commits { .. }
        | Request::RepoSearch { .. }
        | Request::AddCollaborator { .. }
        | Request::RepoHooks { .. }
        | Request::CreateRepoHook { .. }
        | Request::EditRepoHook { .. }
        | Request::DeleteRepoHook { .. }
        | Request::OrgHooks { .. }
        | Request::CreateOrgHook { .. }
        | Request::EditOrgHook { .. }
        | Request::DeleteOrgHook { .. }
        | Request::Org { .. }
        | Request::CreateOrg { .. }
        | Request::CreateRepository { .. }
        | Request::CreateFile { .. }
        | Request::Teams { .. }
        | Request::CreateTeam { .. }
        | Request::EditTeam { .. }
        | Request::TeamMembers { .. }
        | Request::AddMember { .. }
        | Request::User
        | Request::ApiSettings
        | Request::RepositoryById { .. }
        | Request::Repository { .. }
        | Request::UserRepos { .. }
        | Request::UserSearch { .. }
        | Request::Issues { .. }
        | Request::Issue { .. }
        | Request::Comments { .. }
        | Request::Pull { .. }
        | Request::PullFor { .. }
        | Request::CombinedStatus { .. }
        | Request::Reviews { .. }
        | Request::ReviewComments { .. }
        | Request::Branch { .. }
        | Request::Branches { .. }
        | Request::Protection { .. }
        | Request::Protections { .. }
        | Request::Collaborators { .. }
        | Request::Permission { .. }
        | Request::Pulls { .. }
        | Request::PullFiles { .. }
        | Request::Compare { .. }
        | Request::Contents { .. }
        | Request::Raw { .. }
        | Request::Tree { .. }
        | Request::WorkflowRuns { .. }
        | Request::Dispatch { .. }
        | Request::Rerun { .. }
        | Request::Runs { .. }
        | Request::Run { .. }
        | Request::Jobs { .. }
        | Request::Job { .. }
        | Request::JobLog { .. }
        | Request::CreateIssue { .. }
        | Request::OpenPull { .. }
        | Request::PostComment { .. }
        | Request::PostReview { .. }
        | Request::EditIssue { .. }
        | Request::RequestReviewers { .. }
        | Request::RemoveReviewers { .. }
        | Request::Merge { .. }
        | Request::Update { .. }
        | Request::PostStatus { .. }
        | Request::CreateBranch { .. }
        | Request::DeleteBranch { .. }
        | Request::Tokens { .. }
        | Request::CreateToken { .. }
        | Request::DeleteToken { .. }
        | Request::UserOf { .. }
        | Request::CreateUser { .. } => None,
    }
}

fn copied_default(answer: &Answer) -> Option<&[u8]> {
    match answer {
        Answer::Copied { default_branch, .. } => Some(default_branch),
        Answer::Artifacts(_)
        | Answer::Artifact { .. }
        | Answer::Commits { .. }
        | Answer::Located { .. }
        | Answer::Items { .. }
        | Answer::Item { .. }
        | Answer::Pull(_)
        | Answer::Pulls(_)
        | Answer::Comments(_)
        | Answer::Runs(_)
        | Answer::Run(_)
        | Answer::Jobs(_)
        | Answer::Job { .. }
        | Answer::PullFiles { .. }
        | Answer::Comparison { .. }
        | Answer::Checks(_)
        | Answer::Protection(_)
        | Answer::Settings(_)
        | Answer::Collaborators(_)
        | Answer::Statuses(_)
        | Answer::Permission(_)
        | Answer::Comment { .. }
        | Answer::Dependencies(_)
        | Answer::Labels(_)
        | Answer::Commit(_)
        | Answer::Tree(_)
        | Answer::GitTree { .. }
        | Answer::File(_)
        | Answer::Directory(_)
        | Answer::Pages { .. }
        | Answer::Page(_)
        | Answer::Created(_)
        | Answer::Commented(_)
        | Answer::Reviewed(_)
        | Answer::Merged(_)
        | Answer::Revision(_)
        | Answer::Done
        | Answer::Cloned { .. }
        | Answer::Pushed(_)
        | Answer::PushNew { .. }
        | Answer::Branch(_) => None,
    }
}
