//! Each playground start act is the person's request, under their sign-in
//! (temper/docs/design/draft/playground-reset.md, 6.1;
//! temper/docs/design/protocol/setup.md, 8.4).
use super::{Decision, Delivery, Domain, Env, Limits, Work, applications, core, emit, forge, internal, people};
use alloc::boxed::Box;
use skein_lib::{ReplyTo, Time, Token};

// Distinct from the demo's zero and the forge route's u64::MAX reply.
const REPLY: u64 = u64::MAX - 1;
#[derive(Debug)]
pub(crate) struct Context {
    pub sign_in: u64,
    pub key: [u8; 16],
    pub carried: Option<core::Carried>,
}
#[derive(Debug)]
pub(crate) struct Start {
    pub request: Token,
    pub phase: Phase,
    pub until: Time,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Phase {
    Waiting,
    Project,
    Account,
    Uses { account: u32 },
    Connecting,
    Adoption,
}
pub(crate) fn owns(to: Token) -> bool {
    to == Token::new(REPLY)
}

fn key(original: [u8; 16], purpose: &[u8]) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper playground start\0");
    hash.update(&original);
    hash.update(purpose);
    hash.finish().get(..16).expect("digest prefix").try_into().expect("16-byte derived key")
}

pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token) {
    assert!(domain.playground_start.is_none(), "one playground start at a time");
    let until = env.now.saturating_add(domain.forge.demo().expect("offered playground").deadline);
    domain.playground_start = Some(Start { request, phase: Phase::Waiting, until });
    progress(domain, env, decision);
}

fn ask(domain: &mut Domain, phase: Phase, purpose: &[u8], ask: people::Ask) {
    let start = domain.playground_start.as_mut().expect("active start");
    start.phase = phase.clone();
    domain.playground_reply = Some((start.request, phase));
    let context = domain.playground_contexts.get(&start.request).expect("admitted start envelope");
    domain.work.push(Work::People(people::Event::Ask {
        reply_to: internal(REPLY),
        sign_in: context.sign_in,
        key: key(context.key, purpose),
        ask,
        models: Box::new([]),
    }));
}

fn deployment_account(domain: &Domain) -> Option<u32> {
    let provider = domain.forge.demo().expect("offered playground").provider;
    for fact in &domain.core.accounts.account_views() {
        if let Some(account) = domain.core.accounts.account(fact.account)
            && account.provider() == provider
            && account.holder() == jig_domain_accounts::Holder::Deployment
        {
            return Some(fact.account);
        }
    }
    None
}

fn uses(domain: &mut Domain, account: u32) {
    let project = domain.forge.playground().expect("running playground").project.expect("made project");
    ask(
        domain,
        Phase::Uses { account },
        b"uses",
        people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::SetUses { account, uses: Some(Box::new([people::Use::Project(project)])) },
        },
    );
}

fn begin(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let row = *domain.forge.playground().expect("running playground");
    if row.project.is_none() {
        ask(domain, Phase::Project, b"project", people::Ask::MakeProject { name: "Playground".into(), policy: 1 });
        return;
    }
    let request = domain.playground_start.as_ref().expect("active start").request;
    let existing = deployment_account(domain);
    let context = domain.playground_contexts.get_mut(&request).expect("admitted start envelope");
    let carried = context.carried.take();
    if row.account.is_some() || existing.is_some() {
        if let Some(carried) = carried {
            emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
        }
        if row.account.is_none() {
            uses(domain, existing.expect("existing deployment account"));
        } else {
            domain.playground_start.as_mut().expect("active start").phase = Phase::Connecting;
        }
    } else if let Some(carried) = carried {
        domain.playground_start.as_mut().expect("active start").phase = Phase::Account;
        domain.playground_reply = Some((request, Phase::Account));
        domain.work.push(Work::Core(core::Event::Key {
            reply_to: internal(REPLY),
            sign_in: context.sign_in,
            key: key(context.key, b"account"),
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Deployment,
                change: people::AccountChange::AddKey {
                    provider: domain.forge.demo().expect("offered playground").provider,
                    join: None,
                    digest: carried.digest,
                },
            }),
            carried,
        }));
    } else {
        finish(domain, forge::StartAnswer::Failed(forge::StartFailure::Account));
    }
}

pub(crate) fn progress(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let Some(start) = &domain.playground_start else { return };
    let row = match domain.forge.playground() {
        Some(row) if row.state == forge::PlaygroundState::Running => *row,
        _ => {
            domain.playground_start = None;
            return;
        }
    };
    let connected = connected(&domain.forge.connection(row.forge.expect("made forge")));
    if !connected && env.now >= start.until {
        finish(domain, forge::StartAnswer::Failed(forge::StartFailure::NotConnected));
        return;
    }
    if start.phase == Phase::Waiting && domain.playground_reply.is_none() {
        begin(domain, env, decision);
    }
    let Some(start) = &domain.playground_start else { return };
    if start.phase != Phase::Connecting || !connected {
        return;
    }
    let mut adoption = domain.forge.demo().expect("offered playground").adoption.clone();
    adoption.forge = row.forge.expect("made forge");
    adoption.project = row.project.expect("made project");
    for repository in domain.forge.adopted_ids(adoption.forge) {
        let repository = domain
            .forge
            .repository(skein_forge::api::Repository {
                forge: adoption.forge,
                repository: u32::try_from(repository).expect("provider repository number"),
            })
            .expect("adopted repository");
        if repository.project == adoption.project
            && repository.owner == adoption.owner
            && repository.name == adoption.name
        {
            finish(domain, forge::StartAnswer::Running);
            return;
        }
    }
    let request = start.request;
    let context = domain.playground_contexts.get(&request).expect("active start");
    let admission = applications::Admission {
        reply_to: internal(REPLY),
        sign_in: context.sign_in,
        key: key(context.key, b"adoption"),
        digest: skein_lib::sha256::digest(b"temper playground adoption"),
        scope: people::ApplicationScope::Project(adoption.project),
        carried: None,
    };
    domain.playground_start.as_mut().expect("active start").phase = Phase::Adoption;
    domain.playground_reply = Some((request, Phase::Adoption));
    applications::start(domain, admission, crate::Application::Forge(forge::Ask::Adopt(adoption)));
}

fn finish(domain: &mut Domain, answer: forge::StartAnswer) {
    domain.playground_start = None;
    domain.work.push(Work::Forge(forge::Event::PlaygroundAnswered { answer }));
}
fn active(domain: &Domain, request: Token) -> bool {
    match &domain.playground_start {
        Some(start) => start.request == request,
        None => false,
    }
}

pub(crate) fn reply(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    _to: ReplyTo,
    reply: people::Reply,
) {
    let Some((request, phase)) = domain.playground_reply.take() else { return };
    if !active(domain, request) {
        progress(domain, env, decision);
        return;
    }
    match (phase, reply) {
        (Phase::Project, people::Reply::Outcome(people::Outcome::ProjectMade { project })) => {
            domain.work.push(Work::Forge(forge::Event::PlaygroundMade { project: Some(project), account: None }));
            domain.playground_start.as_mut().expect("active start").phase = Phase::Waiting;
        }
        (Phase::Account, people::Reply::Outcome(people::Outcome::AccountKeyKept { .. })) => {
            if let Some(account) = deployment_account(domain) {
                uses(domain, account);
            } else {
                finish(domain, forge::StartAnswer::Failed(forge::StartFailure::Account));
            }
        }
        (Phase::Uses { account }, people::Reply::Outcome(people::Outcome::AccountChanged)) => {
            domain.work.push(Work::Forge(forge::Event::PlaygroundMade { project: None, account: Some(account) }));
            domain.playground_start.as_mut().expect("active start").phase = Phase::Waiting;
        }
        (phase, _) => {
            let why = match phase {
                Phase::Project => forge::StartFailure::Project,
                Phase::Account | Phase::Uses { .. } => forge::StartFailure::Account,
                Phase::Adoption => forge::StartFailure::Adoption,
                Phase::Waiting | Phase::Connecting => unreachable!("reply belongs to a requested act"),
            };
            finish(domain, forge::StartAnswer::Failed(why));
        }
    }
}

pub(crate) fn adopted(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    outcome: Option<forge::Answered>,
) {
    let Some((request, _)) = domain.playground_reply.take() else { return };
    if !active(domain, request) {
        progress(domain, env, decision);
        return;
    }
    let answer = match outcome {
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::Adopted(_))) => forge::StartAnswer::Running,
        Some(
            forge::Answered::Adopt(
                forge::ApplicationOutcome::CoreRefused(_) | forge::ApplicationOutcome::ForgeRefused(_),
            )
            | forge::Answered::SetUp(_)
            | forge::Answered::StartPlayground(_)
            | forge::Answered::EndPlayground(_)
            | forge::Answered::RemoveForge(_)
            | forge::Answered::Connect(_)
            | forge::Answered::Install(_)
            | forge::Answered::Accept(_)
            | forge::Answered::Reachable(_),
        )
        | None => forge::StartAnswer::Failed(forge::StartFailure::Adoption),
    };
    finish(domain, answer);
}

pub(crate) fn connection(domain: &mut Domain, forge: u16, why: &[u8]) {
    let number = match domain.forge.playground() {
        Some(row) => row.forge,
        None => None,
    };
    if domain.playground_start.is_some() && number == Some(forge) {
        finish(
            domain,
            forge::StartAnswer::Failed(forge::StartFailure::Group {
                why: why.get(..why.len().min(256)).expect("bounded reason").into(),
            }),
        );
    }
}

pub(crate) fn done(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token, success: bool) {
    if let Some(context) = domain.playground_contexts.remove(&request)
        && success
        && let Some(carried) = context.carried
    {
        emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
    }
    if active(domain, request) {
        domain.playground_start = None;
    }
}

pub(crate) fn connected(connection: &forge::Connection) -> bool {
    match connection {
        forge::Connection::Connected { .. } => true,
        forge::Connection::Connecting
        | forge::Connection::NotConnected
        | forge::Connection::NotSetUp
        | forge::Connection::SettingUp { .. }
        | forge::Connection::SetUpFailed { .. }
        | forge::Connection::Stopped { .. } => false,
    }
}
