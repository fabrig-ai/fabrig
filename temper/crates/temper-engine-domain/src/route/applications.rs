//! The core admits each typed application before its forge connector decides it.
use super::{Domain, Work, forge, forge_route};
use crate::{Application, ApplicationAnswer};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};

/// The application's admission as the core decides it, before its owner is named.
pub(super) struct Admission {
    pub(super) reply_to: ReplyTo,
    pub(super) sign_in: u64,
    pub(super) key: [u8; 16],
    pub(super) digest: [u8; 32],
    pub(super) scope: people::ApplicationScope,
    pub(super) carried: Option<core::Carried>,
}

pub(super) fn start(domain: &mut Domain, admission: Admission, request: Application) {
    assert!(domain.application.is_none(), "one application admission value");
    let owner = Token::new(domain.next_application_owner);
    domain.next_application_owner = domain.next_application_owner.checked_add(1).expect("application owner counter");
    let Admission { reply_to, sign_in, key, digest, scope, carried } = admission;
    let scope = match &request {
        Application::Forge(
            forge::Ask::SetUp { .. }
            | forge::Ask::RemoveForge { .. }
            | forge::Ask::StartPlayground
            | forge::Ask::EndPlayground
            | forge::Ask::StartDemo
            | forge::Ask::Connect { .. }
            | forge::Ask::Install { .. }
            | forge::Ask::Accept { .. }
            | forge::Ask::Reachable { .. },
        ) => people::ApplicationScope::Deployment,
        Application::Forge(forge::Ask::Adopt(_)) => scope,
        Application::Return { .. } => people::ApplicationScope::Own,
    };
    if request == Application::Forge(forge::Ask::StartDemo) {
        domain.application_demo = Some(super::demo::Context { sign_in, key, carried });
    }
    if request == Application::Forge(forge::Ask::StartPlayground) {
        domain.application_playground = Some(super::playground::Context { sign_in, key, carried });
    }
    domain.application_carried = carried;
    domain.application = Some((owner, request));
    domain.work.push(Work::Core(core::Event::Application {
        reply_to,
        sign_in,
        key,
        scope,
        connector: domain.config.forge_connector,
        digest,
        owner,
        carried,
    }));
}

pub(super) fn admitted(
    domain: &mut Domain,
    request: Token,
    token: Token,
    owner: Token,
    retention: Duration,
    role: Option<people::Role>,
    env: &skein_lib::Env<crate::Limits>,
) {
    let (held, ask) = domain.application.take().expect("one admitted application value");
    assert_eq!(held, owner, "the core admits the application's owner");
    let carried = domain.application_carried.take();
    match ask {
        Application::Forge(ask) => {
            if let Some(context) = domain.application_demo.take() {
                if domain.demo_contexts.len() == domain.demo_contexts.capacity() {
                    finished(domain, request, Err(people::Again::Busy));
                    return;
                }
                assert!(domain.demo_contexts.insert(request, context).is_ok(), "bounded admitted starts");
            }
            if let Some(context) = domain.application_playground.take() {
                if domain.playground_contexts.len() == domain.playground_contexts.capacity() {
                    finished(domain, request, Err(people::Again::Busy));
                    return;
                }
                assert!(domain.playground_contexts.insert(request, context).is_ok(), "bounded admitted starts");
            }
            domain.work.push(Work::Forge(forge::Event::Application { request, token, retention, ask, role, carried }));
        }
        Application::Return { source, path } => {
            // Only live keyed answers need a volatile marker; the core bounds their room.
            loop {
                let mut expired = None;
                for (held, until) in &domain.returns {
                    if until.0 <= env.now || until.1 <= env.wall {
                        expired = Some(*held);
                        break;
                    }
                }
                let Some(expired) = expired else { break };
                domain.returns.remove(&expired);
            }
            domain
                .returns
                .insert(
                    token,
                    (
                        env.now.saturating_add(retention),
                        skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(retention.as_nanos())),
                    ),
                )
                .expect("one marker per retained key");
            assert!(super::now(domain, crate::Request::Return { source, path }), "return output door reserved");
            finished(domain, request, Ok(token));
        }
    }
}

pub(super) fn drop(domain: &mut Domain, owner: Token) {
    if let Some((held, _)) = &domain.application {
        assert_eq!(*held, owner, "drop names the application owner");
        domain.application = None;
        domain.application_demo = None;
        domain.application_playground = None;
        domain.application_carried = None;
    }
}

pub(super) fn part(
    domain: &mut Domain,
    request: Token,
    adoption: forge::Adoption,
    repository: skein_forge::api::Repository,
) {
    match forge_route::adopt_repository_ask(adoption, repository, domain.config.forge_connector) {
        Some(ask) => domain.work.push(Work::Core(core::Event::ApplicationPart { request, ask: Box::new(ask) })),
        None => domain.work.push(Work::Forge(forge::Event::ApplicationAnswered {
            request,
            outcome: people::Outcome::Refused(people::Refusal::Unknown),
        })),
    }
}

pub(super) fn finished(domain: &mut Domain, request: Token, outcome: Result<Token, people::Again>) {
    domain.work.push(Work::Core(core::Event::ApplicationFinished { request, outcome }));
}

pub(super) fn answer(domain: &Domain, token: Token) -> Option<ApplicationAnswer> {
    if domain.returns.contains_key(&token) {
        return Some(ApplicationAnswer::Returning);
    }
    match domain.forge.application_answer(token) {
        Some(forge::Answered::Adopt(answer)) => Some(ApplicationAnswer::Adopt(answer)),
        Some(forge::Answered::SetUp(answer)) => Some(ApplicationAnswer::SetUp(answer)),
        Some(forge::Answered::RemoveForge(answer)) => Some(ApplicationAnswer::RemoveForge(answer)),
        Some(forge::Answered::StartPlayground(answer)) => Some(ApplicationAnswer::StartPlayground(answer)),
        Some(forge::Answered::EndPlayground(answer)) => Some(ApplicationAnswer::EndPlayground(answer)),
        Some(forge::Answered::StartDemo(answer)) => Some(ApplicationAnswer::StartDemo(answer)),
        Some(forge::Answered::Connect(answer)) => Some(ApplicationAnswer::Connect(answer)),
        Some(forge::Answered::Reachable(answer)) => Some(ApplicationAnswer::Reachable(answer)),
        Some(forge::Answered::Accept(answer)) => Some(ApplicationAnswer::Accept(answer)),
        Some(forge::Answered::Install(answer)) => Some(ApplicationAnswer::Install(answer)),
        None => None,
    }
}

/// Reuse the welcome's empty named project before connector admission (contracts, D5).
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "only setup project admission is intercepted; every other event is forwarded"
)]
#[expect(
    clippy::disallowed_methods,
    reason = "the setup project is validated text carried as bytes; the core's name reader takes str"
)]
pub(super) fn set_up_project(
    domain: &mut Domain,
    env: &skein_lib::Env<crate::Limits>,
    event: forge::Event,
    out: &mut skein_lib::Queue<forge::Request>,
) -> Option<forge::Event> {
    match event {
        forge::Event::Application {
            request,
            token,
            retention,
            role,
            carried,
            ask: forge::Ask::SetUp { org, repository, project: forge::SetUpProject::New(name), playground },
        } => {
            let project = match ::core::str::from_utf8(&name) {
                Ok(name) => domain.core.project_named(name),
                Err(_) => None,
            };
            let project = match project {
                Some(project) if domain.forge.project_has_repository(project) => {
                    domain.forge.refuse_set_up(
                        &super::environment_forge(env),
                        request,
                        token,
                        retention,
                        forge::SetUpRefusal::NewProject,
                        out,
                    );
                    return None;
                }
                Some(project) => forge::SetUpProject::Existing(project),
                None => forge::SetUpProject::New(name),
            };
            Some(forge::Event::Application {
                request,
                token,
                retention,
                role,
                carried,
                ask: forge::Ask::SetUp { org, repository, project, playground },
            })
        }
        event => Some(event),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::resume_fixture;
    use skein_lib::Queue;

    fn application() -> forge::Event {
        forge::Event::Application {
            request: Token::new(91),
            token: Token::new(92),
            retention: Duration::from_secs(60),
            role: Some(people::Role::Owner),
            carried: None,
            ask: forge::Ask::SetUp {
                org: b"work".as_slice().into(),
                repository: b"hello".as_slice().into(),
                playground: false,
                project: forge::SetUpProject::New(b"dEfAuLt".as_slice().into()),
            },
        }
    }
    fn root() -> (Domain, skein_lib::Env<crate::Limits>) {
        let mut config = resume_fixture::config(7);
        config.core.seed_projects[0].name = "Default".into();
        resume_fixture::root_with_config(config)
    }
    #[test]
    fn a_default_project_made_before_is_reused() {
        let (mut domain, env) = root();
        let mut out = Queue::uncapped();
        let event = set_up_project(&mut domain, &env, application(), &mut out).expect("empty project reused");
        let forge::Event::Application { ask: forge::Ask::SetUp { project, .. }, .. } = event else {
            panic!("setup application")
        };
        assert_eq!(project, forge::SetUpProject::Existing(1));
        assert!(out.is_empty());
    }
    #[test]
    fn a_named_project_with_a_repository_is_refused() {
        let (mut domain, env) = root();
        let mut out = Queue::uncapped();
        let repository = forge::Repository {
            project: 1,
            home: true,
            provider: skein_forge::api::Repository { forge: 1, repository: 2 },
            host: b"forge.example".as_slice().into(),
            owner: b"work".as_slice().into(),
            name: b"hello".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            ci: false,
            checks: Box::new([]),
            kinds: forge::Kinds {
                read: true,
                push: true,
                open: true,
                land: true,
                review: true,
                status: true,
                comment: true,
                issue: true,
                branch: true,
            },
            protection: forge::Protection::Absent,
            settings: skein_forge::api::Settings {
                default_branch: b"main".as_slice().into(),
                merge: true,
                squash: true,
                rebase: false,
            },
        };
        forge::step(
            &mut domain.forge,
            &super::super::environment_forge(&env),
            forge::Event::Restore { record: forge::Stored::Repository(repository) },
            &mut out,
        );
        assert!(set_up_project(&mut domain, &env, application(), &mut out).is_none());
        assert_eq!(
            domain.forge.application_answer(Token::new(92)),
            Some(forge::Answered::SetUp(forge::SetUpAnswer::Refused(forge::SetUpRefusal::NewProject)))
        );
        assert_eq!(out.pop(), Some(forge::Request::ApplicationDone { request: Token::new(91), token: Token::new(92) }));
        assert!(out.is_empty());
    }
}
