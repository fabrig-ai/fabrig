//! The core admits each typed application before its forge connector decides it.
//! Trials: temper/docs/design/domain/trial.md, 6.
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
            | forge::Ask::Connect { .. }
            | forge::Ask::Replace { .. }
            | forge::Ask::Install { .. }
            | forge::Ask::Accept { .. }
            | forge::Ask::Reachable { .. },
        ) => people::ApplicationScope::Deployment,
        Application::Forge(forge::Ask::MakeRepository { into, .. }) => match into {
            Some(into) => people::ApplicationScope::Project(into.project),
            None => people::ApplicationScope::Deployment,
        },
        Application::Forge(forge::Ask::DismissMaking { into, .. }) => match into {
            Some(project) => people::ApplicationScope::Project(*project),
            None => people::ApplicationScope::Deployment,
        },
        Application::Forge(forge::Ask::SetDraft(call)) => people::ApplicationScope::Project(call.project),
        Application::Forge(forge::Ask::Adopt(_)) => scope,
        Application::Try { .. } | Application::AddDevice(_) | Application::RemoveDevice { .. } => {
            people::ApplicationScope::Own
        }
        Application::Dismiss {
            what:
                forge::PersonFact::ToBegin { project }
                | forge::PersonFact::GoalFrom { project, .. }
                | forge::PersonFact::KeepAsking { project, .. },
        }
        | Application::Follow { project, .. }
        | Application::Looked { project }
        | Application::SetAway { project, .. }
        | Application::Overturn { project, .. }
        | Application::FileFinding(forge::FileFinding { project, .. })
        | Application::HandFinding(forge::HandFinding { project, .. })
        | Application::TriageFinding(forge::TriageFinding { project, .. }) => {
            people::ApplicationScope::Project(*project)
        }
    };
    let carried = match &request {
        Application::Try { .. } => {
            if let Some(carried) = carried {
                assert!(
                    super::now(
                        domain,
                        crate::Request::Party(crate::PartyRequest::Deliver(crate::Delivery::CarriedUnused {
                            number: carried.number
                        },))
                    ),
                    "carried value output door reserved"
                );
            }
            None
        }
        Application::Follow { .. }
        | Application::Forge(_)
        | Application::Dismiss { .. }
        | Application::Looked { .. }
        | Application::SetAway { .. }
        | Application::Overturn { .. }
        | Application::AddDevice(_)
        | Application::RemoveDevice { .. }
        | Application::FileFinding(_)
        | Application::HandFinding(_)
        | Application::TriageFinding(_) => carried,
    };
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

#[expect(clippy::too_many_arguments, reason = "the core admission carries the party beside its request metadata")]
pub(super) fn admitted(
    domain: &mut Domain,
    request: Token,
    token: Token,
    owner: Token,
    retention: Duration,
    role: Option<people::Role>,
    person: u64,
    env: &skein_lib::Env<crate::Limits>,
    decision: &mut crate::Decision,
) {
    let (held, ask) = domain.application.take().expect("one admitted application value");
    assert_eq!(held, owner, "the core admits the application's owner");
    let carried = domain.application_carried.take();
    match ask {
        Application::FileFinding(call) => {
            let answer = super::findings::file_finding(domain, env, decision, forge::Filer::Party(person), call);
            let mut out = skein_lib::Queue::uncapped();
            domain.forge.answer_finding(request, token, forge::Answered::FileFinding(answer), &mut out);
            forge_route::outputs(domain, env, decision, &mut out);
        }
        Application::HandFinding(call) => {
            let answer = super::findings::hand_finding(domain, env, decision, forge::Filer::Party(person), call);
            let mut out = skein_lib::Queue::uncapped();
            domain.forge.answer_finding(request, token, forge::Answered::HandFinding(answer), &mut out);
            forge_route::outputs(domain, env, decision, &mut out);
        }
        Application::TriageFinding(call) => {
            let answer = super::findings::triage_finding(domain, env, decision, forge::Filer::Party(person), call);
            let mut out = skein_lib::Queue::uncapped();
            domain.forge.answer_finding(request, token, forge::Answered::TriageFinding(answer), &mut out);
            forge_route::outputs(domain, env, decision, &mut out);
        }
        Application::Follow { project, follow, act } => {
            let authorized = match domain.forge.follow(project, follow) {
                Some(setting) => match setting.owner {
                    forge::settings::FollowOwner::Person(owner) => owner == person,
                    forge::settings::FollowOwner::Task(_) => false,
                },
                None => false,
            } || domain.core.people.role(person, project) == Some(people::Role::Owner);
            domain.work.push(Work::Forge(forge::Event::FollowAct {
                request,
                token,
                retention,
                project,
                follow,
                act,
                authorized,
            }));
        }
        Application::Try { account, again } => {
            sweep_answers(domain, env);
            let answer = super::trials::asked(domain, env, person, account, again);
            keep_trial_answer(domain, token, retention, answer, env);
            finished(domain, request, Ok(token));
        }
        Application::AddDevice(call) => add_device(domain, request, token, person, call),
        Application::RemoveDevice { number } => {
            domain.work.push(Work::Forge(forge::Event::RemoveDevice { request, token, party: person, number }));
        }
        Application::SetAway { project, on, until } => {
            domain.work.push(Work::Forge(forge::Event::SetAway { request, token, party: person, project, on, until }));
        }
        Application::Overturn { project, number } => {
            domain.work.push(Work::Forge(forge::Event::Overturn { request, token, party: person, project, number }));
        }
        Application::Looked { project } => {
            domain.work.push(Work::Forge(forge::Event::Looked { request, token, party: person, project }));
        }
        Application::Dismiss { what } => {
            domain.work.push(Work::Forge(forge::Event::Dismiss { request, token, party: person, what }));
        }
        Application::Forge(forge::Ask::SetDraft(call)) => {
            if let Some(carried) = carried {
                assert!(
                    super::now(
                        domain,
                        crate::Request::Party(crate::PartyRequest::Deliver(crate::Delivery::CarriedUnused {
                            number: carried.number
                        }))
                    ),
                    "carried value output door reserved"
                );
            }
            domain.work.push(Work::Forge(forge::Event::SetDraft { request, token, party: person, call }));
        }
        Application::Forge(ask) => {
            if let Some(context) = domain.application_playground.take() {
                if domain.playground_contexts.len() == domain.playground_contexts.capacity() {
                    finished(domain, request, Err(people::Again::Busy));
                    return;
                }
                domain.playground_contexts.insert(request, context).expect("bounded playground starts");
            }
            domain.work.push(Work::Forge(forge::Event::Application { request, token, retention, ask, role, carried }));
        }
    }
}

fn keep_trial_answer(
    domain: &mut Domain,
    token: Token,
    retention: Duration,
    answer: crate::TryAnswer,
    env: &skein_lib::Env<crate::Limits>,
) {
    domain
        .try_answers
        .insert(
            token,
            (
                answer,
                env.now.saturating_add(retention),
                skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(retention.as_nanos())),
            ),
        )
        .expect("one marker per retained key");
}

fn add_device(domain: &mut Domain, request: Token, token: Token, person: u64, call: forge::AddDevice) {
    let person_kind = match domain.core.people.kind(person) {
        Some(people::Kind::Person) => true,
        Some(people::Kind::Service) | None => false,
    };
    domain.work.push(Work::Forge(forge::Event::AddDevice { request, token, party: person, person: person_kind, call }));
}

pub(super) fn drop(domain: &mut Domain, owner: Token) {
    if let Some((held, _)) = &domain.application {
        assert_eq!(*held, owner, "drop names the application owner");
        domain.application = None;
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
    if let Some((answer, _, _)) = domain.try_answers.get(&token) {
        return Some(ApplicationAnswer::Try(*answer));
    }
    match domain.forge.application_answer(token) {
        Some(forge::Answered::Follow(words)) => Some(ApplicationAnswer::Follow(words)),
        Some(forge::Answered::Make(answer)) => Some(ApplicationAnswer::Make(answer)),
        Some(forge::Answered::Adopt(answer)) => Some(ApplicationAnswer::Adopt(answer)),
        Some(forge::Answered::SetUp(answer)) => Some(ApplicationAnswer::SetUp(answer)),
        Some(forge::Answered::RemoveForge(answer)) => Some(ApplicationAnswer::RemoveForge(answer)),
        Some(forge::Answered::StartPlayground(answer)) => Some(ApplicationAnswer::StartPlayground(answer)),
        Some(forge::Answered::EndPlayground(answer)) => Some(ApplicationAnswer::EndPlayground(answer)),
        Some(forge::Answered::Connect(answer)) => Some(ApplicationAnswer::Connect(answer)),
        Some(forge::Answered::Reachable(answer)) => Some(ApplicationAnswer::Reachable(answer)),
        Some(forge::Answered::Accept(answer)) => Some(ApplicationAnswer::Accept(answer)),
        Some(forge::Answered::Install(answer)) => Some(ApplicationAnswer::Install(answer)),
        Some(forge::Answered::Dismiss(answer)) => Some(ApplicationAnswer::Dismiss(answer)),
        Some(forge::Answered::Looked(answer)) => Some(ApplicationAnswer::Looked(answer)),
        Some(forge::Answered::SetDraft(answer)) => Some(ApplicationAnswer::SetDraft(answer)),
        Some(forge::Answered::SetAway(answer)) => Some(ApplicationAnswer::SetAway(answer)),
        Some(forge::Answered::Overturn(answer)) => Some(ApplicationAnswer::Overturn(answer)),
        Some(forge::Answered::Device(answer)) => Some(ApplicationAnswer::Device(answer)),
        Some(forge::Answered::FileFinding(answer)) => Some(ApplicationAnswer::FileFinding(answer)),
        Some(forge::Answered::HandFinding(answer)) => Some(ApplicationAnswer::HandFinding(answer)),
        Some(forge::Answered::TriageFinding(answer)) => Some(ApplicationAnswer::TriageFinding(answer)),
        None => None,
    }
}

// Retained keys outlive these volatile markers (temper/docs/design/domain/trial.md, 4).
fn sweep_answers(domain: &mut Domain, env: &skein_lib::Env<crate::Limits>) {
    let mut expired = skein_lib::List::with_capacity(domain.try_answers.len());
    for (token, (_, now, wall)) in &domain.try_answers {
        if *now <= env.now || *wall <= env.wall {
            expired.push(*token).expect("at most one marker per key");
        }
    }
    for token in &expired {
        domain.try_answers.remove(token);
    }
}
