//! The made repository's connector act (temper/docs/design/domain/forge/adoption.md, 5;
//! jig/docs/design/domain/people/acts.md, 3).
use super::{Domain, Work, core, forge, forge_route, people};
use alloc::boxed::Box;
use skein_lib::{List, ReplyTo, Token};

pub(super) fn act(
    domain: &mut Domain,
    key: forge::MakingKey,
    act: [u8; 16],
    adoption: forge::Adoption,
    repository: skein_forge::api::Repository,
) {
    let Some((project, adoption)) = forge_route::adoption_parts(adoption, repository, domain.config.forge_connector)
    else {
        failed(domain, key, forge::MakeFailure::Other, b"invalid made repository adoption".as_slice().into());
        return;
    };
    // Share the root's internal reply sequence with playground acts.
    let token = Token::new(domain.next_playground_reply);
    domain.next_playground_reply = domain.next_playground_reply.checked_sub(1).expect("internal act reply sequence");
    domain.making_replies.insert(token, key).expect("one act per making row");
    domain.work.push(Work::Core(core::Event::Act {
        reply_to: ReplyTo::new(token),
        connector: domain.config.forge_connector,
        key: act,
        act: Box::new(people::Act::Adopt { project, adoption }),
        carried: None,
    }));
}

/// Only an adoption naming a kept Adopting row can be the connector's ask back.
pub(super) fn asked(
    domain: &Domain,
    project: u32,
    adoption: &people::Adoption,
) -> Option<(forge::Adoption, skein_forge::api::Repository)> {
    if adoption.resource.connector != domain.config.forge_connector {
        return None;
    }
    let [forge, repository] = adoption.resource.path.as_ref() else { return None };
    let repository = skein_forge::api::Repository {
        forge: u16::from_be_bytes(forge.as_ref().try_into().ok()?),
        repository: u32::from_be_bytes(repository.as_ref().try_into().ok()?),
    };
    let role = match adoption.role {
        people::ResourceRole::Owned => forge::Role::Owned,
        people::ResourceRole::Fork => forge::Role::Fork,
        people::ResourceRole::Context => forge::Role::Context,
    };
    let [home, host, owner, name, prefix, landing, ci, checks @ ..] = adoption.options.as_ref() else { return None };
    let home = boolean(home)?;
    let ci = boolean(ci)?;
    let mut parsed = List::with_capacity(u32::try_from(checks.len()).ok()?);
    for check in checks {
        parsed.push(u32::from_be_bytes(check.as_ref().try_into().ok()?)).ok()?;
    }
    let key = forge::MakingKey { forge: repository.forge, org: owner.clone(), name: name.clone() };
    let row = domain.forge.making(&key)?;
    let into = row.into.as_ref()?;
    let branch = match &row.state {
        forge::MakingState::Adopting { id, branch } if *id == u64::from(repository.repository) => branch,
        forge::MakingState::Adopting { .. }
        | forge::MakingState::Made { .. }
        | forge::MakingState::Waiting
        | forge::MakingState::Making
        | forge::MakingState::Failed(_) => return None,
    };
    if into.project != project
        || into.home != home
        || into.host != *host
        || branch != landing
        || role != forge::Role::Owned
        || !ci
        || prefix.as_ref() != b"temper/"
        || !checks.is_empty()
    {
        return None;
    }
    Some((
        forge::Adoption {
            project,
            home,
            forge: repository.forge,
            host: host.clone(),
            owner: owner.clone(),
            name: name.clone(),
            prefix: prefix.clone(),
            role,
            landing: landing.clone(),
            ci,
            checks: parsed.into_boxed(),
        },
        repository,
    ))
}
fn boolean(bytes: &[u8]) -> Option<bool> {
    match bytes {
        [0] => Some(false),
        [1] => Some(true),
        _ => None,
    }
}

fn failed(domain: &mut Domain, key: forge::MakingKey, reason: forge::MakeFailure, words: Box<[u8]>) {
    domain.work.push(Work::Forge(forge::Event::MakingActed {
        key,
        outcome: Some(forge::MakingFailed { step: forge::MakeStep::Adoption, reason, words }),
    }));
}
fn acted(domain: &mut Domain, key: forge::MakingKey, outcome: people::Outcome) {
    match outcome {
        people::Outcome::Adopted { .. } => {
            domain.work.push(Work::Forge(forge::Event::MakingActed { key, outcome: None }));
        }
        people::Outcome::Again(_) => domain.work.push(Work::Forge(forge::Event::MakingAgain { key })),
        people::Outcome::Refused(why) | people::Outcome::CompoundRefused { why, .. } => {
            failed(domain, key, forge::MakeFailure::NotAllowed, refusal_words(why));
        }
        people::Outcome::Restored { .. }
        | people::Outcome::ProjectRemoved { .. }
        | people::Outcome::AccountFinishing { .. }
        | people::Outcome::SettingsChanged { .. }
        | people::Outcome::MovedToNewest { .. }
        | people::Outcome::Compound { .. }
        | people::Outcome::Created { .. }
        | people::Outcome::MarkedSeen { .. }
        | people::Outcome::Applied { .. }
        | people::Outcome::SignedOut
        | people::Outcome::NoteEdited { .. }
        | people::Outcome::ServiceMade { .. }
        | people::Outcome::Proposed { .. }
        | people::Outcome::Withdrawn { .. }
        | people::Outcome::Left { .. }
        | people::Outcome::Stopped { .. }
        | people::Outcome::Cancelled { .. }
        | people::Outcome::Released { .. }
        | people::Outcome::Lifted { .. }
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
        | people::Outcome::Renamed { .. }
        | people::Outcome::ProjectMade { .. }
        | people::Outcome::PoolSet { .. }
        | people::Outcome::EscalationDecided { .. }
        | people::Outcome::AccountSignInStarted { .. }
        | people::Outcome::AccountSignedOut { .. }
        | people::Outcome::AccountKeyKept { .. }
        | people::Outcome::AccountProgramDeclared { .. }
        | people::Outcome::AccountReading { .. }
        | people::Outcome::AccountChanged
        | people::Outcome::AccountMovingOut { .. }
        | people::Outcome::AccountMovedIn { .. }
        | people::Outcome::AccountRemoved { .. }
        | people::Outcome::VisibilitySet { .. } => {
            failed(domain, key, forge::MakeFailure::Other, b"unexpected adoption act outcome".as_slice().into());
        }
    }
}
fn refusal_words(why: people::Refusal) -> Box<[u8]> {
    let words = match why {
        people::Refusal::NotOffered => b"the forge connector is not granted adopt (engine.acts)".as_slice(),
        people::Refusal::Role { held: None } => b"adopt has no configured first owner",
        people::Refusal::Role { held: Some(_) } => b"adopt lacks the required role",
        people::Refusal::Unknown => b"adopt project no longer exists",
        people::Refusal::Ended { .. } => b"adopt project has ended",
        people::Refusal::Authority { .. } => b"adopt lacks resource access or authority",
        people::Refusal::Limit { .. } => b"adopt exceeds a configured limit",
        people::Refusal::KeyConflict => b"adopt key names another act",
        people::Refusal::Judged { why, .. } => {
            return Box::from(why.get(..why.len().min(256)).expect("bounded adoption refusal"));
        }
        people::Refusal::Forgotten { .. } => b"adopt revision was forgotten",
        people::Refusal::Stale { .. } => b"adopt revision is stale",
        people::Refusal::Full { .. } => b"adopt has no room",
        people::Refusal::NameTaken => b"adopt name is taken",
        people::Refusal::NotWellFormed { .. } => b"adopt is not well formed",
        people::Refusal::Moved { .. } => b"adopt subject has moved",
        people::Refusal::Taken { .. } => b"adopt subject is taken",
        people::Refusal::NotWaiting => b"adopt subject is not waiting",
        people::Refusal::NoFurther => b"adopt has no further holder",
        people::Refusal::Provider => b"adopt provider is not allowed",
        people::Refusal::NoEscalationHolder => b"adopt has no escalation holder",
        people::Refusal::Llm { .. } => b"adopt model is not allowed",
        people::Refusal::Account { .. }
        | people::Refusal::AccountMove { .. }
        | people::Refusal::ProgramNameReserved => b"adopt account is not allowed",
        people::Refusal::SignInRedirect { .. } => b"adopt sign-in redirect was refused",
        people::Refusal::Circle { .. } => b"adopt has a dependency circle",
        people::Refusal::Dependency { .. } => b"adopt prerequisite failed",
        people::Refusal::Kept { .. } => b"adopt waits for prerequisites",
    };
    words.into()
}

/// Consume making replies; pass every other held output to the root's routes.
pub(super) fn held(domain: &mut Domain, held: core::Held) -> Option<core::Held> {
    match held {
        core::Held::Acted { to, outcome } => {
            let token = to.into_token();
            match domain.making_replies.remove(&token) {
                Some(key) => {
                    acted(domain, key, outcome);
                    None
                }
                None => Some(core::Held::Acted { to: ReplyTo::new(token), outcome }),
            }
        }
        held @ (core::Held::Drained
        | core::Held::Load(_)
        | core::Held::ApplicationDelivery { .. }
        | core::Held::ApplicationWatchEnded { .. }
        | core::Held::ReadLoad { .. }
        | core::Held::Read { .. }
        | core::Held::Applied { .. }
        | core::Held::CarriedUnused { .. }
        | core::Held::Delivery { .. }
        | core::Held::WatchEnded { .. }
        | core::Held::MakeEffect { .. }
        | core::Held::SettledCall { .. }
        | core::Held::Assign { .. }
        | core::Held::ReadList { .. }
        | core::Held::Sources { .. }
        | core::Held::CheckSource { .. }
        | core::Held::SignOutSource { .. }
        | core::Held::SourceSignedOut { .. }
        | core::Held::Grant { .. }
        | core::Held::Relayed { .. }
        | core::Held::Inbound { .. }
        | core::Held::Relay { .. }
        | core::Held::PeopleReply { .. }
        | core::Held::CallAnswer { .. }
        | core::Held::Result { .. }
        | core::Held::Acknowledge { .. }
        | core::Held::TaskTerminalAcknowledged { .. }
        | core::Held::TaskTurnKept { .. }
        | core::Held::AcknowledgeTurn { .. }
        | core::Held::Cancel { .. }
        | core::Held::Refuse { .. }
        | core::Held::TurnBusy { .. }
        | core::Held::StopRun { .. }
        | core::Held::NotesLoad { .. }
        | core::Held::NotesWritten { .. }
        | core::Held::NotesDeleted { .. }
        | core::Held::PublicRead { .. }
        | core::Held::PublicWait { .. }) => Some(held),
    }
}
