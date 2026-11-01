//! The playground's project, scripted account and adoption
//! (`temper/docs/design/draft/playground-reset.md`, 6.1;
//! `jig/docs/design/domain/people.md`, 5.3).
use super::{Decision, Delivery, Domain, Env, Limits, Work, applications, core, emit, forge, kept, people};
use alloc::boxed::Box;
use skein_lib::{ReplyTo, Time, Token};

// Task numbers start at one. The connector runs either start flow, whose
// internal adoption reply uses zero, as the demo's does.
const ADOPTION: u64 = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Phase {
    Project,
    Account,
    Connecting,
    Adoption,
}

#[derive(Debug)]
pub(crate) struct Start {
    request: Token,
    maker: Option<u64>,
    pub(crate) row: forge::Playground,
    pub(crate) phase: Phase,
    pub(crate) until: Time,
    pub(crate) retry: bool,
    waiting: bool,
    unused: bool,
    failed: Option<forge::StartFailure>,
}

#[derive(Debug)]
pub(crate) struct ActReply {
    request: Token,
    phase: Phase,
}

#[derive(PartialEq, Eq)]
enum ActAnswer {
    Again,
    Project(u32),
    Account,
    Refused { grant: bool },
}

fn act_answer(outcome: people::Outcome) -> ActAnswer {
    use people::Outcome;
    match outcome {
        Outcome::Again(_) => ActAnswer::Again,
        Outcome::ProjectMade { project } => ActAnswer::Project(project),
        Outcome::AccountKeyKept { .. } => ActAnswer::Account,
        Outcome::Refused(why) => ActAnswer::Refused { grant: why == people::Refusal::NotOffered },
        Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::Compound { .. }
        | Outcome::CompoundRefused { .. }
        | Outcome::ProjectRemoved { .. }
        | Outcome::AccountRemoved { .. }
        | Outcome::Renamed { .. }
        | Outcome::Created { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::SignedOut
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Released { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::Moved { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Amended { .. }
        | Outcome::RolesSet { .. }
        | Outcome::PolicyChanged { .. }
        | Outcome::PoolSet { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::AccountSignInStarted { .. }
        | Outcome::AccountSignedOut { .. }
        | Outcome::AccountFinishing { .. }
        | Outcome::AccountProgramDeclared { .. }
        | Outcome::AccountChanged
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::Lifted { .. } => ActAnswer::Refused { grant: false },
    }
}

pub(crate) fn owns(domain: &Domain, token: Token) -> bool {
    if token != Token::new(ADOPTION) {
        return false;
    }
    match &domain.playground_start {
        Some(start) => start.phase == Phase::Adoption,
        None => false,
    }
}

pub(crate) fn start_key(forge: u16, purpose: &[u8]) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper playground start\0");
    hash.update(&forge.to_be_bytes());
    hash.update(purpose);
    hash.finish().get(..16).expect("digest prefix").try_into().expect("16-byte key")
}

pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token) {
    // The connector joins subsequent starts to this start's answer.
    if domain.playground_start.is_some() {
        return;
    }
    let row = *domain.forge.playground().expect("the connector kept the playground before starting it");
    begin(domain, env, decision, request, row);
}

pub(crate) fn begin(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    request: Token,
    row: forge::Playground,
) {
    let until = env.now.saturating_add(domain.forge.demo().expect("offered playground").deadline);
    let maker = if row.project.is_none() {
        let context = domain.demo_contexts.get(&request).expect("admitted start context");
        Some(domain.core.people.sign_in_person(context.sign_in).expect("admitted starting person"))
    } else {
        None
    };
    domain.playground_start = Some(Start {
        request,
        maker,
        row,
        phase: Phase::Project,
        until,
        retry: false,
        waiting: false,
        unused: false,
        failed: None,
    });
    progress(domain, env, decision);
}

pub(crate) fn retrying(domain: &Domain) -> bool {
    match &domain.playground_start {
        Some(start) => start.retry,
        None => false,
    }
}

pub(crate) fn resume(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    if retrying(domain)
        && let Some(start) = &domain.playground_start
        && env.now >= start.until
    {
        finish(
            domain,
            env,
            decision,
            forge::StartAnswer::Failed(forge::StartFailure::Group { why: b"busy".as_slice().into() }),
        );
        return;
    }
    if let Some(start) = &mut domain.playground_start {
        start.retry = false;
    }
    progress(domain, env, decision);
}

pub(crate) fn progress(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let Some(start) = &domain.playground_start else { return };
    if start.waiting {
        return;
    }
    if let Some(why) = &start.failed {
        finish(domain, env, decision, forge::StartAnswer::Failed(why.clone()));
        return;
    }
    if start.retry {
        if env.now >= start.until {
            finish(
                domain,
                env,
                decision,
                forge::StartAnswer::Failed(forge::StartFailure::Group { why: b"busy".as_slice().into() }),
            );
        }
        return;
    }
    match start.phase {
        Phase::Project | Phase::Account => prepare(domain, env, decision),
        Phase::Connecting | Phase::Adoption => {}
    }
    if let Some(start) = &domain.playground_start
        && start.phase == Phase::Connecting
    {
        connect(domain, env, decision);
    }
}

fn prepare(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let start = domain.playground_start.as_ref().expect("active start");
    let demo = domain.forge.demo().expect("offered playground");
    let context = domain.demo_contexts.get(&start.request).expect("admitted start context");
    let (phase, purpose, act, carried) = if let Some(project) = start.row.project {
        if start.row.account.is_some() {
            if let Some(carried) = domain.demo_contexts.get_mut(&start.request).expect("start context").carried.take() {
                emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
            }
            domain.playground_start.as_mut().expect("active start").phase = Phase::Connecting;
            return;
        }
        let Some(carried) = context.carried else {
            finish(domain, env, decision, forge::StartAnswer::Failed(forge::StartFailure::Account));
            return;
        };
        (
            Phase::Account,
            b"account".as_slice(),
            people::Act::AddKey {
                provider: demo.provider,
                join: None,
                digest: carried.digest,
                uses: Some(Box::new([people::Use::Project(project)])),
            },
            Some(carried),
        )
    } else {
        let maker = start.maker.expect("starting person kept before the act");
        (
            Phase::Project,
            b"project".as_slice(),
            people::Act::MakeProject { maker, name: demo.project_name.clone(), policy: demo.project_policy },
            None,
        )
    };
    let reply = ActReply { request: start.request, phase };
    let key = start_key(start.row.forge, purpose);
    let token = kept::reply_token(domain);
    domain.kept_replies.start.insert(token, reply).expect("one reply per active start");
    let start = domain.playground_start.as_mut().expect("active start");
    start.phase = phase;
    start.waiting = true;
    start.unused = false;
    domain.work.push(Work::Core(core::Event::Act {
        reply_to: ReplyTo::new(token),
        connector: domain.config.forge_connector,
        key,
        act: Box::new(act),
        carried,
    }));
}

fn connect(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let start = domain.playground_start.as_ref().expect("active start");
    if !super::demo::connected(&domain.forge.connection(start.row.forge)) {
        if env.now >= start.until {
            finish(domain, env, decision, forge::StartAnswer::Failed(forge::StartFailure::NotConnected));
        }
        return;
    }
    let project = start.row.project.expect("project before connecting");
    let demo = domain.forge.demo().expect("offered playground");
    for repository in &domain.forge.readable_repositories(project) {
        if repository.provider.forge == start.row.forge
            && repository.owner == demo.adoption.owner
            && repository.name == demo.adoption.name
        {
            finish(domain, env, decision, forge::StartAnswer::Running);
            return;
        }
    }
    let mut adoption = demo.adoption.clone();
    adoption.forge = start.row.forge;
    adoption.project = project;
    let context = domain.demo_contexts.get(&start.request).expect("active start context");
    let admission = applications::Admission {
        reply_to: ReplyTo::new(Token::new(ADOPTION)),
        sign_in: context.sign_in,
        key: start_key(start.row.forge, b"adoption"),
        digest: skein_lib::sha256::digest(b"temper playground adoption"),
        scope: people::ApplicationScope::Project(project),
        carried: None,
    };
    domain.playground_start.as_mut().expect("active start").phase = Phase::Adoption;
    applications::start(domain, admission, crate::Application::Forge(forge::Ask::Adopt(adoption)));
}

pub(crate) fn acted(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    reply: ActReply,
    outcome: people::Outcome,
) {
    let start = domain.playground_start.as_mut().expect("start waits for its act");
    assert_eq!(reply.request, start.request, "act belongs to this start");
    start.waiting = false;
    let answer = act_answer(outcome);
    if answer == ActAnswer::Again {
        start.retry = true;
        progress(domain, env, decision);
        return;
    }
    if reply.phase == Phase::Account {
        let carried = domain.demo_contexts.get_mut(&reply.request).expect("active context").carried.take();
        if start.unused
            && let Some(carried) = carried
        {
            emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
        }
    }
    match answer {
        ActAnswer::Project(project) if reply.phase == Phase::Project => {
            start.row.project = Some(project);
            start.phase = Phase::Account;
            domain.work.push(Work::Forge(forge::Event::PlaygroundProject { project }));
            progress(domain, env, decision);
        }
        ActAnswer::Account if reply.phase == Phase::Account => {
            let provider = domain.forge.demo().expect("offered playground").provider;
            let mut newest = None;
            for (number, account) in domain.core.accounts.accounts_by_number() {
                if account.provider() == provider && account.holder() == jig_domain_accounts::Holder::Deployment {
                    newest = Some(*number);
                }
            }
            if let Some(account) = newest {
                start.row.account = Some(account);
                start.phase = Phase::Connecting;
                domain.work.push(Work::Forge(forge::Event::PlaygroundAccount { account }));
                progress(domain, env, decision);
            } else {
                finish(domain, env, decision, forge::StartAnswer::Failed(forge::StartFailure::Account));
            }
        }
        ActAnswer::Project(_) | ActAnswer::Account | ActAnswer::Refused { .. } => {
            if answer == (ActAnswer::Refused { grant: true }) {
                let line = match reply.phase {
                    Phase::Project => {
                        b"playground start: the forge connector is not granted make_project (engine.acts)".as_slice()
                    }
                    Phase::Account => {
                        b"playground start: the forge connector is not granted add_key (engine.acts)".as_slice()
                    }
                    Phase::Connecting | Phase::Adoption => unreachable!("start act phase"),
                };
                emit(decision, &env.limits, Delivery::DemoNote { line: line.into() });
            }
            let why = match reply.phase {
                Phase::Project => forge::StartFailure::Group { why: b"project".as_slice().into() },
                Phase::Account => forge::StartFailure::Account,
                Phase::Connecting | Phase::Adoption => unreachable!("start act phase"),
            };
            finish(domain, env, decision, forge::StartAnswer::Failed(why));
        }
        ActAnswer::Again => unreachable!("retry returned before the terminal answer"),
    }
}

// Jig reports unused before Acted. Hold that report until a terminal answer:
// Again must leave the carried value available for the next send.
pub(crate) fn carried_unused(domain: &mut Domain, number: u64) -> bool {
    let Some(start) = &mut domain.playground_start else { return false };
    if start.phase == Phase::Account
        && let Some(context) = domain.demo_contexts.get(&start.request)
        && let Some(carried) = context.carried
        && carried.number == number
    {
        start.unused = true;
        return true;
    }
    false
}

pub(crate) fn adopted(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    outcome: Option<forge::Answered>,
) {
    let Some(start) = &domain.playground_start else { return };
    if start.phase != Phase::Adoption {
        return;
    }
    let answer = match outcome {
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::Adopted(_))) => forge::StartAnswer::Running,
        Some(
            forge::Answered::Adopt(
                forge::ApplicationOutcome::CoreRefused(_) | forge::ApplicationOutcome::ForgeRefused(_),
            )
            | forge::Answered::SetUp(_)
            | forge::Answered::MakeForge(_)
            | forge::Answered::RemoveForge(_)
            | forge::Answered::EndPlayground(_)
            | forge::Answered::StartPlayground(_)
            | forge::Answered::Connect(_)
            | forge::Answered::Install(_)
            | forge::Answered::Accept(_)
            | forge::Answered::Reachable(_),
        )
        | None => forge::StartAnswer::Failed(forge::StartFailure::Adoption),
    };
    finish(domain, env, decision, answer);
}

pub(crate) fn finish(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, answer: forge::StartAnswer) {
    let Some(start) = domain.playground_start.take() else { return };
    if let Some(context) = domain.demo_contexts.get_mut(&start.request)
        && let Some(carried) = context.carried.take()
    {
        emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
    }
    domain.work.push(Work::Forge(forge::Event::PlaygroundAnswered { answer }));
}

pub(crate) fn connection(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, forge: u16, why: &[u8]) {
    let Some(start) = &mut domain.playground_start else { return };
    if start.row.forge == forge {
        start.failed =
            Some(forge::StartFailure::Group { why: why.get(..why.len().min(256)).expect("bounded reason").into() });
        progress(domain, env, decision);
    }
}
