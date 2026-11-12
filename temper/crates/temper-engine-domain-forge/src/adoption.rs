//! A core-admitted keyed adoption and its retained terminal (domain/forge/adoption.md).
use crate::domain::{child, emit};
use crate::{
    Adopted, AdoptedRepository, Adoption, AdoptionRefusal, ApplicationOutcome, Domain, Key, Kinds, Limits, Protection,
    Repository, Request, Role, Stored, reason,
};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_forge::api;
use skein_lib::{Duration, Env, List, Map, Queue, Token, Wall};

#[derive(Hash, Debug)]
pub(crate) struct Pending {
    pub(crate) token: Token,
    pub(crate) retention: Duration,
    pub(crate) observed: Option<ApplicationOutcome>,
}

#[derive(Hash, Debug)]
pub(crate) struct State {
    pending: Map<Token, Pending>,
    pub(crate) kept: Map<Token, (Wall, ApplicationOutcome)>,
}

impl State {
    pub(crate) fn restore(
        &mut self,
        token: u64,
        until: Wall,
        outcome: ApplicationOutcome,
    ) -> Result<(), crate::RestoreFit> {
        let held = self.kept.len();
        let limit = self.kept.capacity();
        match self.kept.insert(Token::new(token), (until, outcome)) {
            Ok(_) => Ok(()),
            Err(_) => Err(crate::RestoreFit {
                family: Some(crate::RecordKind::Application),
                set: crate::RestoreSet::Rows(crate::RecordKind::Application),
                held: u64::from(held),
                additional: 1,
                limit: u64::from(limit),
            }),
        }
    }
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { pending: Map::with_capacity(limits.adoptions), kept: Map::with_capacity(limits.applications) }
    }
}

impl Domain {
    /// The repository located for this core-admitted application.
    #[must_use]
    pub fn applying(&self, request: Token) -> Option<api::Repository> {
        let pending = self.adoptions.get(&request)?;
        if pending.repository.repository == 0 { None } else { Some(pending.repository) }
    }

    /// The forge's outcome under the people child's token, while retained.
    #[must_use]
    pub fn application_outcome(&self, token: Token) -> Option<&ApplicationOutcome> {
        match self.application.kept.get(&token) {
            Some((_, outcome)) => Some(outcome),
            None => None,
        }
    }
}

pub(crate) fn start(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    retention: Duration,
    adoption: Adoption,
    out: &mut Queue<Request>,
) {
    if !room(d, env, request, out) {
        return;
    }
    if d.application.pending.len() >= env.limits.adoptions || d.adoptions.len() >= env.limits.adoptions {
        return again(request, people::Again::Busy, out);
    }
    let pending = Pending { token, retention, observed: None };
    if !d.identities.contains_key(&adoption.forge) && !d.writers.contains_key(&adoption.forge) {
        return finish(d, env, request, pending, ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge), out);
    }
    let connected = match d.connection(adoption.forge) {
        crate::Connection::Connected { .. } => true,
        crate::Connection::NotSetUp
        | crate::Connection::SettingUp { .. }
        | crate::Connection::SetUpFailed { .. }
        | crate::Connection::Connecting
        | crate::Connection::Stopped { .. } => false,
        // Direct domain worlds may supply writers; the service configures none.
        crate::Connection::NotConnected => d.writers.contains_key(&adoption.forge),
    };
    if d.forge_ended(adoption.forge) || !connected {
        return finish(
            d,
            env,
            request,
            pending,
            ApplicationOutcome::ForgeRefused(reason(api::Error::NotConnected)),
            out,
        );
    }
    assert!(d.application.pending.insert(request, pending).is_ok(), "reserved application flight");
    if invalid_fields(&adoption, &env.limits) {
        let pending = d.application.pending.remove(&request).expect("admitted application");
        return finish(d, env, request, pending, ApplicationOutcome::ForgeRefused(AdoptionRefusal::TooLarge), out);
    }
    let repository = api::Repository { forge: adoption.forge, repository: 0 };
    let access = match d.configured.get(&adoption.forge) {
        Some(config) => match config.kind {
            crate::Kind::Forgejo => match config.mode {
                crate::Mode::Managed | crate::Mode::Owned => true,
                crate::Mode::Connected | crate::Mode::Fake => false,
            },
            crate::Kind::GitHub => false,
        },
        None => false,
    };
    let owner = adoption.owner.clone();
    let name = adoption.name.clone();
    let pending = PendingAdoption {
        request: adoption,
        repository,
        stage: if access { AdoptionStage::Access } else { AdoptionStage::Locate },
        permission: None,
        settings: None,
        protection: Protection::Unknown,
        collaborators: List::with_capacity(env.limits.collaborators),
    };
    assert!(d.adoptions.insert(request, pending).is_ok(), "reserved adoption flight");
    if access {
        out.push(Request::AdoptAccess { forge: repository.forge, request, owner, name });
    } else {
        let read = api::Read::Locate { owner, name };
        child(d, env, skein_forge_client::Event::Read { owner: request, repository, read }, out);
    }
}

/// Access precedes the bot's first read (temper/docs/design/domain/forge/adoption.md, 2).
pub(crate) fn accessed(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    answer: Result<(), AdoptionRefusal>,
    out: &mut Queue<Request>,
) {
    let Some(pending) = d.adoptions.get_mut(&request) else { return };
    if pending.stage != AdoptionStage::Access {
        return;
    }
    match answer {
        Ok(()) => {
            pending.stage = AdoptionStage::Locate;
            let repository = pending.repository;
            let read = api::Read::Locate { owner: pending.request.owner.clone(), name: pending.request.name.clone() };
            child(d, env, skein_forge_client::Event::Read { owner: request, repository, read }, out);
        }
        Err(why) => {
            d.adoptions.remove(&request);
            locate_refused(d, env, request, why, out);
        }
    }
}

/// Record exactly what the adoption reads decided before forwarding to the core.
pub(crate) fn adopted(d: &mut Domain, out: &mut Queue<Request>, reply_to: Token, result: Result<Adopted, api::Error>) {
    if let Some(pending) = d.application.pending.get_mut(&reply_to) {
        pending.observed = Some(match &result {
            Ok(adopted) => {
                let row = &adopted.repository;
                ApplicationOutcome::Adopted(AdoptedRepository {
                    repository: row.provider,
                    owner: row.owner.clone(),
                    name: row.name.clone(),
                    role: row.role,
                    home: row.home,
                    kinds: row.kinds,
                    protection: row.protection.clone(),
                })
            }
            Err(error) => ApplicationOutcome::ForgeRefused(reason(*error)),
        });
    }
    // An act has no pending party application. Its caller hears this result
    // under jig's request token and later tells MakingActed (domain/forge/adoption.md, 5).
    out.push(Request::Adopted { reply_to, result });
}

// The core's part still sees the provider-shaped refusal; the application
// retains the admission decision, which the provider's Error cannot express.
fn refused(d: &mut Domain, out: &mut Queue<Request>, request: Token, why: AdoptionRefusal) {
    adopted(d, out, request, Err(api::Error::Refused));
    if let Some(pending) = d.application.pending.get_mut(&request) {
        pending.observed = Some(ApplicationOutcome::ForgeRefused(why));
    }
}

pub(crate) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    outcome: people::Outcome,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.application.pending.remove(&request) else { return };
    d.adoptions.remove(&request);
    let outcome = if let Some(ApplicationOutcome::ForgeRefused(why)) = &pending.observed {
        ApplicationOutcome::ForgeRefused(*why)
    } else if let people::Outcome::Refused(why) = outcome {
        ApplicationOutcome::CoreRefused(why)
    } else if let people::Outcome::Again(why) = outcome {
        return again(request, why, out);
    } else if let people::Outcome::Adopted { .. } = outcome {
        pending.observed.take().expect("adoption reads recorded the repository")
    } else {
        unreachable!("the core's adoption part ends with an adoption outcome");
    };
    finish(d, env, request, pending, outcome, out);
}

pub(crate) fn finish(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    pending: Pending,
    outcome: ApplicationOutcome,
    out: &mut Queue<Request>,
) {
    let until = Wall::from_nanos(env.wall.as_nanos().saturating_add(pending.retention.as_nanos()));
    let row = Stored::Application { token: pending.token.raw(), until, outcome: outcome.clone() };
    assert!(d.application.kept.insert(pending.token, (until, outcome)).is_ok(), "reserved application outcome");
    out.push(Request::Save { record: row, release: None, read_afresh: None });
    out.push(Request::ApplicationDone { request, token: pending.token });
}

fn again(request: Token, why: people::Again, out: &mut Queue<Request>) {
    out.push(Request::ApplicationAgain { request, why });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum AdoptionStage {
    Access,
    Locate,
    Part,
    Permission,
    Branches,
    Settings,
    Protection,
    Collaborators(u32),
}

/// One admission read chain; only rows proven from current provider facts
/// can become durable adopted repositories.
#[derive(Hash, Debug)]
pub(crate) struct PendingAdoption {
    request: Adoption,
    repository: api::Repository,
    stage: AdoptionStage,
    permission: Option<api::Permission>,
    settings: Option<api::Settings>,
    protection: Protection,
    collaborators: List<api::Collaborator>,
}

pub(crate) fn adopt(d: &mut Domain, env: &Env<Limits>, reply_to: Token, out: &mut Queue<Request>) {
    let Some(mut pending) = d.adoptions.remove(&reply_to) else { return };
    let request = &pending.request;
    let repository = pending.repository;
    let invalid = !d.repositories.contains_key(&repository) && d.repositories.len() == env.limits.repositories;
    if invalid {
        adopted(
            d,
            out,
            reply_to,
            Err(api::Error::TooLarge(api::Excess::exact(
                api::Bound::Forge,
                u64::from(env.limits.repositories),
                u64::from(d.repositories.len()).saturating_add(1),
            ))),
        );
        return;
    }
    if let Some(existing) = d.repositories.get(&repository)
        && existing.project != request.project
        && existing.role != Role::Context
        && request.role != Role::Context
    {
        refused(d, out, reply_to, AdoptionRefusal::AnotherProject);
        return;
    }
    if request.home && request.role == Role::Context {
        adopted(d, out, reply_to, Err(api::Error::Refused));
        return;
    }
    if !request.ci && request.checks.is_empty() {
        adopted(d, out, reply_to, Err(api::Error::Refused));
        return;
    }
    if request.checks.len() > skein_lib::usize_of(env.limits.change_policy.gates) || request.checks.contains(&0) {
        let excess = if request.checks.len() > skein_lib::usize_of(env.limits.change_policy.gates) {
            api::Excess::exact(
                api::Bound::Forge,
                u64::from(env.limits.change_policy.gates),
                skein_lib::len_u64(request.checks.len()),
            )
        } else {
            api::Excess::unnamed()
        };
        adopted(d, out, reply_to, Err(api::Error::TooLarge(excess)));
        return;
    }
    for (index, check) in request.checks.iter().enumerate() {
        if request.checks.get(..index).expect("index in checks").contains(check) {
            adopted(d, out, reply_to, Err(api::Error::Refused));
            return;
        }
    }
    if request.home {
        for (_, existing) in &d.repositories {
            if existing.project == request.project
                && existing.home
                && existing.provider != repository
                && !d.forge_ended(existing.provider.forge)
            {
                refused(d, out, reply_to, AdoptionRefusal::HomeHeld);
                return;
            }
        }
    }
    for (_, pending) in &d.adoptions {
        if pending.repository == repository {
            adopted(d, out, reply_to, Err(api::Error::Busy));
            return;
        }
    }
    let author = *d.writers.get(&request.forge).expect("application checked configured forge");
    pending.stage = AdoptionStage::Permission;
    d.adoptions.insert(reply_to, pending).expect("adoption capacity checked");
    child(
        d,
        env,
        skein_forge_client::Event::Read { owner: reply_to, repository, read: api::Read::Permission { user: author } },
        out,
    );
}

fn owns_branch(d: &Domain, repository: api::Repository, branch: &[u8]) -> bool {
    for (name, head) in &d.heads {
        if head.owned
            && name.forge == repository.forge
            && name.repository == repository.repository
            && crate::items::branch_matches(name, branch)
        {
            return true;
        }
    }
    false
}

pub(crate) fn adoption_read(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let Some(mut pending) = d.adoptions.remove(&owner) else { return };
    let repository = pending.repository;
    match pending.stage {
        AdoptionStage::Locate => match result {
            Ok(api::Answer::Located { repository: id }) if id != 0 => {
                pending.repository.repository = id;
                pending.stage = AdoptionStage::Part;
                let repository = pending.repository;
                let adoption = pending.request.clone();
                d.adoptions.insert(owner, pending).expect("replaces locate flight");
                out.push(Request::ApplicationPart { request: owner, adoption, repository });
            }
            Err(error) => locate_refused(d, env, owner, reason(error), out),
            Ok(_) => locate_refused(d, env, owner, AdoptionRefusal::InvalidAnswer, out),
        },
        AdoptionStage::Access | AdoptionStage::Part => {
            unreachable!("no read while the adoption waits for access or the core")
        }
        AdoptionStage::Permission => match result {
            Ok(api::Answer::Permission(permission)) => {
                if permission.permission == api::Permission::None {
                    adopted(d, out, owner, Err(api::Error::Forbidden));
                    return;
                }
                pending.permission = Some(permission.permission);
                let check_prefix = pending.request.role != Role::Context;
                pending.stage = if check_prefix { AdoptionStage::Branches } else { AdoptionStage::Settings };
                let read = if check_prefix {
                    api::Read::Branches { prefix: pending.request.prefix.clone() }
                } else {
                    api::Read::Settings
                };
                d.adoptions.insert(owner, pending).expect("replaces read chain");
                child(d, env, skein_forge_client::Event::Read { owner, repository, read }, out);
            }
            Ok(_) => adopted(d, out, owner, Err(api::Error::InvalidAnswer)),
            Err(error) => adopted(d, out, owner, Err(error)),
        },
        AdoptionStage::Branches => match result {
            Ok(api::Answer::Branches(branches)) => {
                for branch in branches {
                    if branch.starts_with(&pending.request.prefix) && !owns_branch(d, repository, &branch) {
                        refused(d, out, owner, AdoptionRefusal::PrefixOccupied);
                        return;
                    }
                }
                pending.stage = AdoptionStage::Settings;
                d.adoptions.insert(owner, pending).expect("replaces read chain");
                child(d, env, skein_forge_client::Event::Read { owner, repository, read: api::Read::Settings }, out);
            }
            Ok(_) => adopted(d, out, owner, Err(api::Error::InvalidAnswer)),
            Err(error) => adopted(d, out, owner, Err(error)),
        },
        AdoptionStage::Settings => match result {
            Ok(api::Answer::Settings(settings)) => {
                pending.settings = Some(settings);
                pending.stage = AdoptionStage::Protection;
                let branch = pending.request.landing.clone();
                d.adoptions.insert(owner, pending).expect("replaces read chain");
                child(
                    d,
                    env,
                    skein_forge_client::Event::Read { owner, repository, read: api::Read::Protection { branch } },
                    out,
                );
            }
            Ok(_) => adopted(d, out, owner, Err(api::Error::InvalidAnswer)),
            Err(error) => adopted(d, out, owner, Err(error)),
        },
        AdoptionStage::Protection => {
            match result {
                Ok(api::Answer::Protection(Some(protection))) => {
                    pending.protection = Protection::Rule(protection);
                }
                Ok(api::Answer::Protection(None)) => pending.protection = Protection::Absent,
                Err(api::Error::Forbidden) => pending.protection = Protection::Unknown,
                Err(error) => {
                    adopted(d, out, owner, Err(error));
                    return;
                }
                Ok(_) => {
                    adopted(d, out, owner, Err(api::Error::InvalidAnswer));
                    return;
                }
            }
            pending.stage = AdoptionStage::Collaborators(1);
            d.adoptions.insert(owner, pending).expect("replaces read chain");
            child(
                d,
                env,
                skein_forge_client::Event::Read { owner, repository, read: api::Read::Collaborators { page: 1 } },
                out,
            );
        }
        AdoptionStage::Collaborators(page) => adoption_collaborators(d, env, owner, pending, page, result, out),
    }
}

fn adoption_collaborators(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut pending: PendingAdoption,
    page: u32,
    result: Result<api::Answer, api::Error>,
    out: &mut Queue<Request>,
) {
    let repository = pending.repository;
    match result {
        Ok(api::Answer::Collaborators { collaborators, more }) => {
            for person in collaborators {
                if pending.collaborators.push(person).is_err() {
                    adopted(
                        d,
                        out,
                        owner,
                        Err(api::Error::TooLarge(api::Excess::at_least(
                            api::Bound::Forge,
                            u64::from(pending.collaborators.capacity()),
                            u64::from(pending.collaborators.capacity()).saturating_add(1),
                        ))),
                    );
                    return;
                }
            }
            if more {
                let Some(next) = page.checked_add(1) else {
                    adopted(d, out, owner, Err(api::Error::TooLarge(api::Excess::unnamed())));
                    return;
                };
                pending.stage = AdoptionStage::Collaborators(next);
                d.adoptions.insert(owner, pending).expect("replaces read chain");
                child(
                    d,
                    env,
                    skein_forge_client::Event::Read {
                        owner,
                        repository,
                        read: api::Read::Collaborators { page: next },
                    },
                    out,
                );
            } else {
                finish_adoption(d, env, owner, pending, out);
            }
        }
        Ok(_) => adopted(d, out, owner, Err(api::Error::InvalidAnswer)),
        Err(error) => adopted(d, out, owner, Err(error)),
    }
}

fn finish_adoption(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    pending: PendingAdoption,
    out: &mut Queue<Request>,
) {
    let Some(permission) = pending.permission else { unreachable!("permission read before collaborators") };
    let Some(settings) = pending.settings else { unreachable!("settings read before collaborators") };
    let writable = match permission {
        api::Permission::Write | api::Permission::Admin => true,
        api::Permission::None | api::Permission::Read => false,
    } && pending.request.role != Role::Context;
    let kinds = Kinds {
        read: true,
        push: writable,
        open: writable,
        land: writable && (settings.merge || settings.squash || settings.rebase),
        review: writable,
        status: writable,
        comment: writable,
        issue: writable,
        branch: writable,
    };
    let request = pending.request;
    let row = Repository {
        project: request.project,
        home: request.home,
        provider: pending.repository,
        host: request.host,
        owner: request.owner,
        name: request.name,
        prefix: request.prefix,
        role: request.role,
        ci: request.ci,
        checks: request.checks,
        kinds,
        protection: pending.protection,
        settings,
        workflows: match d.repositories.get(&pending.repository) {
            Some(row) => row.workflows.clone(),
            None => Box::default(),
        },
    };
    // The core keeps each project's read grant. This row keeps the writer:
    // another project's context adoption must leave its home and effects alone.
    let writer_held = row.role == Role::Context
        && match d.repositories.get(&row.provider) {
            Some(existing) => existing.project != row.project && existing.role != Role::Context,
            None => false,
        };
    if !writer_held {
        d.repositories.insert(row.provider, row.clone()).expect("adoption capacity checked");
        d.tier_ops.dirty |= !crate::tiers::configured_all(d, row.provider).is_empty();
        emit(out, Request::Save { read_afresh: None, release: None, record: Stored::Repository(row.clone()) });
        crate::workflows::read(d, env, row.provider, None, out);
    }
    // The forges row lists its adoptions (client/forge/deployment.md).
    emit(out, Request::MarkedForges);
    let collaborators = human_collaborators(d, row.provider.forge, pending.collaborators);
    adopted(d, out, owner, Ok(Adopted { repository: row, collaborators }));
}

fn invalid_fields(request: &Adoption, limits: &Limits) -> bool {
    let bound = skein_lib::usize_of(limits.name_bytes);
    for field in [&request.host, &request.owner, &request.name, &request.landing, &request.prefix] {
        if field.is_empty() || field.len() > bound {
            return true;
        }
    }
    // resource_name counts forge/host/owner/repository/branch first (five).
    // tree_branch appends <root>/<c|r|s><task>[-<attempt>] to the prefix:
    // two branch parts, plus one for every slash already in the prefix.
    let mut segments = 7_u64;
    for byte in &request.prefix {
        if *byte == b'/' {
            segments = segments.checked_add(1).expect("prefix length is bounded by u32 name_bytes");
        }
    }
    if segments > u64::from(limits.authority_segments) {
        return true;
    }
    request.checks.len() > skein_lib::usize_of(limits.change_policy.gates)
}

fn locate_refused(d: &mut Domain, env: &Env<Limits>, request: Token, why: AdoptionRefusal, out: &mut Queue<Request>) {
    let pending = d.application.pending.remove(&request).expect("locate belongs to an application");
    finish(d, env, request, pending, ApplicationOutcome::ForgeRefused(why), out);
}

/// Release adoptions after the core has erased the project's tasks
/// (jig/docs/design/domain/people/acts.md, 2; temper/docs/design/domain/forge/adoption.md, 1).
pub(crate) fn release(d: &mut Domain, project: u32, request: Token, out: &mut Queue<Request>) {
    let mut repositories = List::with_capacity(d.repositories.capacity());
    for (provider, row) in &d.repositories {
        if row.project == project {
            repositories.push(*provider).expect("bounded adopted repositories");
        }
    }
    let changed = !repositories.is_empty();
    for provider in &repositories {
        d.repositories.remove(provider);
        emit(out, Request::Erase { key: Key::Repository(*provider) });
        emit(out, Request::Erase { key: Key::Client(skein_forge_client::Key::Repository(*provider)) });
    }
    if changed {
        emit(out, Request::MarkedForges);
    }
    let mut changed_rows = List::with_capacity(d.kept.capacity());
    for (task, names) in &d.kept {
        let mut remaining = List::with_capacity(u32::try_from(names.len()).expect("bounded kept names"));
        for name in names {
            let provider = api::Repository { forge: name.forge, repository: name.repository };
            if !repositories.as_slice().contains(&provider) {
                remaining.push(name.clone()).expect("bounded kept names");
            }
        }
        if remaining.as_slice() != names.as_ref() {
            changed_rows.push((*task, remaining.into_boxed())).expect("bounded kept rows");
        }
    }
    for (task, resources) in changed_rows.into_boxed() {
        if resources.is_empty() {
            d.kept.remove(&task);
            emit(out, Request::Erase { key: Key::Names(task) });
        } else {
            d.kept.insert(task, resources.clone()).expect("replaces kept row");
            emit(out, Request::Save { record: Stored::Names { task, resources }, release: None, read_afresh: None });
        }
    }
    crate::removal::unnamed(d, out);
    d.erase_project_settings(project, out);
    crate::dismissed::erase_project(d, project, out);
    crate::status::erase_project(d, project, out);
    crate::away::erase_project(d, project, out);
    crate::findings::erase_project(d, project, out);
    emit(out, Request::ProjectReleased { request });
}

/// An act already names the made repository (domain/forge/adoption.md, 5).
pub(crate) fn making_asked(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    adoption: Adoption,
    repository: api::Repository,
    out: &mut Queue<Request>,
) {
    if d.adoptions.len() >= env.limits.adoptions {
        again(request, people::Again::Busy, out);
        return;
    }
    if invalid_fields(&adoption, &env.limits) {
        adopted(d, out, request, Err(api::Error::TooLarge(api::Excess::unnamed())));
        return;
    }
    if d.forge_ended(repository.forge) || !d.writers.contains_key(&repository.forge) {
        adopted(d, out, request, Err(api::Error::NotConnected));
        return;
    }
    let pending = PendingAdoption {
        request: adoption,
        repository,
        stage: AdoptionStage::Part,
        permission: None,
        settings: None,
        protection: Protection::Unknown,
        collaborators: List::with_capacity(env.limits.collaborators),
    };
    d.adoptions.insert(request, pending).expect("adoption capacity checked");
    adopt(d, env, request, out);
}

pub(crate) fn room(d: &mut Domain, env: &Env<Limits>, request: Token, out: &mut Queue<Request>) -> bool {
    let mut expired = None;
    for (token, (until, _)) in &d.application.kept {
        if *until <= env.wall {
            expired = Some(*token);
            break;
        }
    }
    if let Some(token) = expired {
        d.application.kept.remove(&token);
        out.push(Request::Erase { key: Key::Application(token.raw()) });
    }
    if d.application.kept.len().saturating_add(d.application.pending.len()) >= env.limits.applications {
        again(request, people::Again::Busy, out);
        return false;
    }
    true
}

impl Domain {
    /// Whether this account is temper's retained writer on the forge, including while disconnected.
    #[must_use]
    pub fn is_writer(&self, forge: u16, account: u64) -> bool {
        self.writers.get(&forge) == Some(&account)
    }
}

fn human_collaborators(d: &Domain, forge: u16, collaborators: List<api::Collaborator>) -> Box<[api::Collaborator]> {
    let mut humans = List::with_capacity(collaborators.len());
    let linked = match d.link(forge) {
        Some(link) => Some(link.user),
        None => None,
    };
    for collaborator in &collaborators {
        if !d.is_writer(forge, collaborator.user) && linked != Some(collaborator.user) {
            humans.push(collaborator.clone()).expect("bounded adoption collaborators");
        }
    }
    humans.into_boxed()
}
