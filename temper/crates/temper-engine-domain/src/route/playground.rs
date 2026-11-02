//! The person's playground acts and the deployment's reset (temper/docs/design/protocol/setup.md, 8.4).
use super::{Decision, Delivery, Domain, Env, Limits, Work, applications, core, emit, forge, people};
use alloc::boxed::Box;
use skein_lib::{ReplyTo, Token};

#[derive(Debug)]
pub(crate) struct Context {
    pub sign_in: u64,
    pub key: [u8; 16],
    pub carried: Option<core::Carried>,
}
#[derive(Debug)]
pub(crate) struct Start {
    request: Token,
    forge: u16,
    account: Option<u32>,
    project: Option<u32>,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Reply {
    Account,
    Project,
    Uses,
    Adoption,
    Reset { key: [u8; 16], act: forge::ResetAct },
}
fn reply_to(domain: &mut Domain, phase: Reply) -> ReplyTo {
    let token = Token::new(domain.next_playground_reply);
    domain.next_playground_reply = domain.next_playground_reply.checked_sub(1).expect("playground reply sequence");
    let request = match phase {
        Reply::Account | Reply::Project | Reply::Uses | Reply::Adoption => {
            Some(domain.playground_start.as_ref().expect("active start").request)
        }
        Reply::Reset { .. } => None,
    };
    domain.playground_replies.insert(token, (request, phase)).expect("bounded playground replies");
    ReplyTo::new(token)
}
fn key(original: [u8; 16], purpose: &[u8]) -> [u8; 16] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(b"temper playground start\0");
    hash.update(&original);
    hash.update(purpose);
    hash.finish().get(..16).expect("digest prefix").try_into().expect("16-byte key")
}
fn held_account(domain: &Domain) -> Option<u32> {
    let provider = domain.playground_provider?;
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
pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token, forge: u16) {
    if domain.playground_start.is_some() {
        return;
    }
    let Some(context) = domain.playground_contexts.get_mut(&request) else { return };
    let carried = context.carried.take();
    let sign_in = context.sign_in;
    let original = context.key;
    let account = held_account(domain);
    domain.playground_start = Some(Start { request, forge, account, project: None });
    if account.is_some() {
        if let Some(carried) = carried {
            emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
        }
        project(domain, env, decision);
    } else if let Some(carried) = carried {
        let reply_to = reply_to(domain, Reply::Account);
        domain.work.push(Work::Core(core::Event::Key {
            reply_to,
            sign_in,
            key: key(original, b"account"),
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Deployment,
                change: people::AccountChange::AddKey {
                    provider: domain.playground_provider.expect("configured playground"),
                    join: None,
                    digest: carried.digest,
                },
            }),
            carried,
        }));
    } else {
        fail(domain, forge::StartFailure::Account);
    }
}
fn progress(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    let start = domain.playground_start.as_ref().expect("active start");
    if let Some(row) = domain.forge.playground_progress(start.project, start.account) {
        super::save(decision, &env.limits, crate::Write::Save(crate::Record::Forge { row: Box::new(row) }));
    }
}
fn ask(domain: &mut Domain, phase: Reply, purpose: &[u8], ask: people::Ask) {
    let start = domain.playground_start.as_ref().expect("active start");
    let context = domain.playground_contexts.get(&start.request).expect("start envelope");
    let sign_in = context.sign_in;
    let key = key(context.key, purpose);
    let reply_to = reply_to(domain, phase);
    domain.work.push(Work::People(people::Event::Ask { reply_to, sign_in, key, ask, models: Box::new([]) }));
}
fn project(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    progress(domain, env, decision);
    let Some(policy) = domain.playground_policy else {
        fail(domain, forge::StartFailure::Adoption);
        return;
    };
    ask(domain, Reply::Project, b"project", people::Ask::MakeProject { name: "Playground".into(), policy });
}
fn fail(domain: &mut Domain, why: forge::StartFailure) {
    domain.playground_start = None;
    domain.work.push(Work::Forge(forge::Event::PlaygroundFailed { why }));
}
pub(crate) fn reply(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    token: Token,
    reply: people::Reply,
) {
    let (request, phase) = domain.playground_replies.remove(&token).expect("playground reply");
    let Some(start) = &domain.playground_start else { return };
    if request != Some(start.request) {
        return;
    }
    match phase {
        Reply::Account => {
            let kept = match reply {
                people::Reply::Outcome(people::Outcome::AccountKeyKept { .. }) => true,
                people::Reply::Outcome(_)
                | people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Again(_)
                | people::Reply::Full { .. }
                | people::Reply::Refused(_)
                | people::Reply::Signer(_) => false,
            };
            if kept {
                let account = held_account(domain);
                domain.playground_start.as_mut().expect("start").account = account;
                project(domain, env, decision);
            } else {
                fail(domain, forge::StartFailure::Account);
            }
        }
        Reply::Project => {
            let project = match reply {
                people::Reply::Outcome(people::Outcome::ProjectMade { project }) => Some(project),
                people::Reply::Outcome(_)
                | people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Again(_)
                | people::Reply::Full { .. }
                | people::Reply::Refused(_)
                | people::Reply::Signer(_) => None,
            };
            if let Some(project) = project {
                let start = domain.playground_start.as_mut().expect("start");
                start.project = Some(project);
                let account = start.account.expect("scripted account");
                progress(domain, env, decision);
                ask(
                    domain,
                    Reply::Uses,
                    b"uses",
                    people::Ask::AccountChange {
                        holder: people::AccountHolder::Deployment,
                        change: people::AccountChange::SetUses {
                            account,
                            uses: Some(Box::new([people::Use::Project(project)])),
                        },
                    },
                );
            } else {
                fail(domain, forge::StartFailure::Adoption);
            }
        }
        Reply::Uses => {
            let changed = match reply {
                people::Reply::Outcome(people::Outcome::AccountChanged) => true,
                people::Reply::Outcome(_)
                | people::Reply::SignedIn { .. }
                | people::Reply::SignedOut
                | people::Reply::Again(_)
                | people::Reply::Full { .. }
                | people::Reply::Refused(_)
                | people::Reply::Signer(_) => false,
            };
            if changed {
                adopt(domain);
            } else {
                fail(domain, forge::StartFailure::Account);
            }
        }
        Reply::Adoption | Reply::Reset { .. } => unreachable!("people start reply"),
    }
}
fn adopt(domain: &mut Domain) {
    let start = domain.playground_start.as_ref().expect("start");
    let project = start.project.expect("playground project");
    let forge = start.forge;
    let host = domain.forge.record(forge).expect("playground forge").place.api.host.clone();
    let context = domain.playground_contexts.get(&start.request).expect("start envelope");
    let sign_in = context.sign_in;
    let key = key(context.key, b"adoption");
    let reply_to = reply_to(domain, Reply::Adoption);
    applications::start(
        domain,
        applications::Admission {
            reply_to,
            sign_in,
            key,
            digest: skein_lib::sha256::digest(b"temper playground adoption"),
            scope: people::ApplicationScope::Project(project),
            carried: None,
        },
        crate::Application::Forge(forge::Ask::Adopt(forge::Adoption {
            project,
            home: true,
            forge,
            host,
            owner: b"playground".as_slice().into(),
            name: b"widget".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        })),
    );
}
pub(crate) fn adopted(domain: &mut Domain, token: Token, outcome: Option<forge::Answered>) {
    let (request, phase) = domain.playground_replies.remove(&token).expect("adoption reply");
    let Some(start) = &domain.playground_start else { return };
    if request != Some(start.request) {
        return;
    }
    match phase {
        Reply::Adoption => {}
        Reply::Account | Reply::Project | Reply::Uses | Reply::Reset { .. } => unreachable!("adoption reply"),
    }
    let success = match outcome {
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::Adopted(_))) => true,
        Some(
            forge::Answered::Adopt(
                forge::ApplicationOutcome::CoreRefused(_) | forge::ApplicationOutcome::ForgeRefused(_),
            )
            | forge::Answered::SetUp(_)
            | forge::Answered::RemoveForge(_)
            | forge::Answered::StartPlayground(_)
            | forge::Answered::EndPlayground(_)
            | forge::Answered::Connect(_)
            | forge::Answered::Install(_)
            | forge::Answered::Accept(_)
            | forge::Answered::Reachable(_),
        )
        | None => false,
    };
    if success {
        if let Some(start) = domain.playground_start.take() {
            domain.work.push(Work::Forge(forge::Event::PlaygroundMade {
                project: start.project.expect("project"),
                account: start.account.expect("account"),
            }));
        }
    } else {
        fail(domain, forge::StartFailure::Adoption);
    }
}
pub(crate) fn done(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, request: Token, success: bool) {
    if let Some(context) = domain.playground_contexts.remove(&request)
        && success
        && let Some(carried) = context.carried
    {
        emit(decision, &env.limits, Delivery::CarriedUnused { number: carried.number });
    }
    if let Some(start) = &domain.playground_start
        && start.request == request
    {
        domain.playground_start = None;
    }
}
pub(crate) fn reset(domain: &mut Domain, key: [u8; 16], act: forge::ResetAct) {
    let reply_to = reply_to(domain, Reply::Reset { key, act });
    let act = match act {
        forge::ResetAct::RemoveProject { project } => people::Act::RemoveProject { project, grace: None },
        forge::ResetAct::RemoveAccount { account } => people::Act::RemoveAccount { account },
    };
    domain.work.push(Work::Core(core::Event::Act {
        reply_to,
        connector: domain.config.forge_connector,
        key,
        act: Box::new(act),
        carried: None,
    }));
}
pub(crate) fn acted(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    token: Token,
    outcome: people::Outcome,
) {
    let (_, phase) = domain.playground_replies.remove(&token).expect("reset reply");
    let (key, act) = match phase {
        Reply::Reset { key, act } => (key, act),
        Reply::Account | Reply::Project | Reply::Uses | Reply::Adoption => unreachable!("reset act reply"),
    };
    let (keep, why) = match outcome {
        people::Outcome::Again(_) => (true, None),
        people::Outcome::Refused(people::Refusal::NotOffered) => (
            true,
            Some(match act {
                forge::ResetAct::RemoveProject { .. } => {
                    b"the forge connector is not granted remove_project (engine.acts)".as_slice()
                }
                forge::ResetAct::RemoveAccount { .. } => {
                    b"the forge connector is not granted remove_account (engine.acts)".as_slice()
                }
            }),
        ),
        people::Outcome::Refused(people::Refusal::Role { held: None }) => {
            (true, Some(b"no first owner configured".as_slice()))
        }
        people::Outcome::Refused(_)
        | people::Outcome::CompoundRefused { .. }
        | people::Outcome::Compound { .. }
        | people::Outcome::AccountRemoved { .. }
        | people::Outcome::SettingsChanged { .. }
        | people::Outcome::MovedToNewest { .. }
        | people::Outcome::Renamed { .. }
        | people::Outcome::ProjectMade { .. }
        | people::Outcome::ProjectRemoved { .. }
        | people::Outcome::Created { .. }
        | people::Outcome::MarkedSeen { .. }
        | people::Outcome::Applied { .. }
        | people::Outcome::SignedOut
        | people::Outcome::NoteEdited { .. }
        | people::Outcome::ServiceMade { .. }
        | people::Outcome::Adopted { .. }
        | people::Outcome::Proposed { .. }
        | people::Outcome::Withdrawn { .. }
        | people::Outcome::Stopped { .. }
        | people::Outcome::Cancelled { .. }
        | people::Outcome::Released { .. }
        | people::Outcome::PersonTaken { .. }
        | people::Outcome::PersonHandedBack { .. }
        | people::Outcome::PersonAnswered { .. }
        | people::Outcome::Moved { .. }
        | people::Outcome::ProposalDecided { .. }
        | people::Outcome::Said { .. }
        | people::Outcome::QuestionAnswered { .. }
        | people::Outcome::Amended { .. }
        | people::Outcome::RolesSet { .. }
        | people::Outcome::PolicyChanged { .. }
        | people::Outcome::PoolSet { .. }
        | people::Outcome::EscalationDecided { .. }
        | people::Outcome::AccountSignInStarted { .. }
        | people::Outcome::AccountFinishing { .. }
        | people::Outcome::AccountSignedOut { .. }
        | people::Outcome::AccountKeyKept { .. }
        | people::Outcome::AccountProgramDeclared { .. }
        | people::Outcome::AccountChanged
        | people::Outcome::AccountMovingOut { .. }
        | people::Outcome::Lifted { .. }
        | people::Outcome::AccountMovedIn { .. } => (false, None),
    };
    if let Some(why) = why {
        emit(decision, &env.limits, Delivery::PlaygroundNote { line: why.into() });
    }
    domain.work.push(Work::Forge(forge::Event::ResetActAnswered { key, keep }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::resume_fixture as fixture;

    #[test]
    fn late_start_answers_do_not_advance_a_later_start() {
        for phase in [Reply::Account, Reply::Project, Reply::Uses, Reply::Adoption] {
            let limits = fixture::limits();
            let mut domain = Domain::new(fixture::config(71), &limits).expect("root fixture");
            let request = Token::new(2);
            domain.playground_start = Some(Start { request, forge: 1, account: Some(1), project: Some(2) });
            let token = Token::new(100);
            domain.playground_replies.insert(token, (Some(Token::new(1)), phase)).expect("old start reply");
            let before = domain.work.len();
            match phase {
                Reply::Adoption => adopted(&mut domain, token, None),
                Reply::Account | Reply::Project | Reply::Uses => {
                    let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
                    let mut decision = Decision::decide(&mut domain.journal, &limits.journal);
                    reply(&mut domain, &env, &mut decision, token, people::Reply::Refused(people::Refusal::NotOffered));
                }
                Reply::Reset { .. } => unreachable!("start reply cases"),
            }
            assert_eq!(domain.playground_start.as_ref().expect("later start remains active").request, request);
            assert_eq!(domain.work.len(), before);
            assert!(!domain.playground_replies.contains_key(&token));
        }
    }
}
