//! Owned representation bounds, independent of admission judgments (jig/docs/design/domain/engine.md, 6).
use crate::{Act, Ask, Identity, Key, Limits, Outcome, Refusal, RestoreBound, RestoreCause};
use alloc::boxed::Box;

fn owned_size(key: &Key, value: Option<usize>, invariant: &str) -> usize {
    // Allocated sizes fit usize, so overflow is a bug rather than a configuration refusal.
    assert!(value.is_some(), "{key:?}: {invariant}");
    value.expect("asserted representable owned size")
}

fn owned_bytes(key: &Key, value: Option<u64>, invariant: &str) -> u64 {
    // Allocated sizes and bounded counts fit u64, so failure is a bug rather than a configuration refusal.
    assert!(value.is_some(), "{key:?}: {invariant}");
    value.expect("asserted representable owned bytes or count")
}

pub(super) fn within(key: &Key, which: RestoreBound, len: usize, limit: u32) -> Result<(), RestoreCause> {
    let held = owned_bytes(key, u64::try_from(len).ok(), "allocated payload length fits u64");
    measured(which, held, limit)
}

fn measured(which: RestoreBound, held: u64, limit: u32) -> Result<(), RestoreCause> {
    let limit = u64::from(limit);
    if held > limit {
        return Err(RestoreCause::Bound { which, held, limit });
    }
    Ok(())
}

pub(super) fn occupancy(
    key: &Key,
    which: RestoreBound,
    first: u32,
    second: u32,
    limit: u32,
) -> Result<(), RestoreCause> {
    let held = owned_bytes(key, u64::from(first).checked_add(u64::from(second)), "held occupancy fits u64");
    let held = owned_bytes(key, held.checked_add(1), "incoming occupancy fits u64");
    measured(which, held, limit)
}

pub(super) fn identity(key: &Key, limits: &Limits, identity: &Identity) -> Result<(), RestoreCause> {
    let held =
        owned_size(key, identity.key.subject.len().checked_add(identity.login.len()), "owned identity bytes fit usize");
    let held = owned_size(key, held.checked_add(identity.name.len()), "owned identity bytes fit usize");
    within(key, RestoreBound::IdentityBytes, held, limits.identity_bytes)
}

pub(super) fn models(key: &Key, names: &[Box<[u8]>]) -> Result<(), RestoreCause> {
    within(key, RestoreBound::ModelNames, names.len(), crate::MODELS)?;
    for name in names {
        within(key, RestoreBound::ModelNameBytes, name.len(), crate::MODEL_NAME_BYTES)?;
    }
    Ok(())
}

fn bytes(key: &Key, which: RestoreBound, value: Option<u64>, limit: u32, invariant: &str) -> Result<(), RestoreCause> {
    measured(which, owned_bytes(key, value, invariant), limit)
}

fn choice(key: &Key, value: Option<&crate::LlmChoice>) -> Result<(), RestoreCause> {
    if let Some(value) = value {
        within(key, RestoreBound::Subagents, value.subagents.len(), crate::MODELS)?;
    }
    Ok(())
}

fn batch(key: &Key, limits: &Limits, members: &[crate::Member]) -> Result<(), RestoreCause> {
    within(key, RestoreBound::Batch, members.len(), limits.batch)?;
    for member in members {
        within(key, RestoreBound::Words, member.words.len(), limits.words)?;
        choice(key, member.llm.as_ref())?;
        bytes(
            key,
            RestoreBound::AmendmentBytes,
            crate::member_bytes(member),
            limits.amendment_bytes,
            "owned member bytes fit u64",
        )?;
    }
    Ok(())
}

fn amendment(key: &Key, limits: &Limits, value: &crate::Amendment) -> Result<(), RestoreCause> {
    within(key, RestoreBound::Words, value.reason.len(), limits.words)?;
    choice(key, value.llm.as_ref())?;
    bytes(
        key,
        RestoreBound::AmendmentBytes,
        crate::amendment_bytes(value),
        limits.amendment_bytes,
        "owned amendment bytes fit u64",
    )
}

fn adoption(key: &Key, limits: &Limits, value: &crate::Adoption) -> Result<(), RestoreCause> {
    let items = owned_size(
        key,
        value.resource.path.len().checked_add(value.options.len()),
        "owned adoption slot count fits usize",
    );
    let mut bytes = owned_size(key, items.checked_mul(size_of::<Box<[u8]>>()), "owned adoption slot bytes fit usize");
    for part in &value.resource.path {
        bytes = owned_size(key, bytes.checked_add(part.len()), "owned adoption bytes fit usize");
    }
    for part in &value.options {
        bytes = owned_size(key, bytes.checked_add(part.len()), "owned adoption bytes fit usize");
    }
    within(key, RestoreBound::AmendmentBytes, bytes, limits.amendment_bytes)
}

fn note(key: &Key, limits: &Limits, scope: &crate::NoteScope, change: &crate::NoteChange) -> Result<(), RestoreCause> {
    let scope = match scope {
        crate::NoteScope::Deployment | crate::NoteScope::Project | crate::NoteScope::Goal { .. } => 0,
        crate::NoteScope::Resources { pattern, .. } => {
            owned_size(key, super::super::note_pattern_bytes(pattern), "owned note pattern bytes fit usize")
        }
    };
    let change = match change {
        crate::NoteChange::Delete { .. } => 0,
        crate::NoteChange::Correct { description, body, references, .. } => {
            let bytes = owned_size(key, description.len().checked_add(body.len()), "owned note text bytes fit usize");
            let refs =
                owned_size(key, references.len().checked_mul(size_of::<u64>()), "owned note reference bytes fit usize");
            owned_size(key, bytes.checked_add(refs), "owned note change bytes fit usize")
        }
    };
    let held = owned_size(key, scope.checked_add(change), "owned note bytes fit usize");
    within(key, RestoreBound::AmendmentBytes, held, limits.amendment_bytes)
}

fn person_result(key: &Key, limits: &Limits, result: &crate::PersonResult) -> Result<(), RestoreCause> {
    let held = match result {
        crate::PersonResult::Report { words } | crate::PersonResult::Verdict { words, .. } => words.len(),
        crate::PersonResult::Failure { reason } => reason.len(),
        crate::PersonResult::Approved { state, words } => {
            let words = match words {
                Some(words) => words.len(),
                None => 0,
            };
            owned_size(key, state.len().checked_add(words), "owned person result bytes fit usize")
        }
    };
    within(key, RestoreBound::Words, held, limits.words)
}

pub(super) fn ask(key: &Key, limits: &Limits, value: &Ask) -> Result<(), RestoreCause> {
    match value {
        Ask::Compound { operations, .. } => {
            measured(RestoreBound::Operations, u64::from(operations.len()), limits.operations)?;
            for operation in operations {
                let fits = match operation {
                    crate::Operation::Cancel { reason, .. } | crate::Operation::Move { reason, .. } => {
                        within(key, RestoreBound::Words, reason.len(), limits.words)
                    }
                    crate::Operation::Message { words, .. } => {
                        within(key, RestoreBound::Words, words.len(), limits.words)
                    }
                    crate::Operation::Amend { amendment: value, .. } => amendment(key, limits, value),
                    crate::Operation::Release { .. } | crate::Operation::Stop { .. } => Ok(()),
                };
                fits?;
            }
            Ok(())
        }
        Ask::Create { batch: members, .. } => batch(key, limits, members),
        Ask::Propose { action, reason, .. } => {
            within(key, RestoreBound::Words, reason.len(), limits.words)?;
            match action {
                crate::Action::Batch(members) => batch(key, limits, members),
                crate::Action::Amend { amendment: value, .. } => amendment(key, limits, value),
                crate::Action::Settings { .. } => Ok(()),
            }
        }
        Ask::Amend { amendment: value, .. } => amendment(key, limits, value),
        Ask::Say { words, .. } | Ask::AnswerQuestion { words, .. } => {
            within(key, RestoreBound::Words, words.len(), limits.words)
        }
        Ask::Move { reason, .. } | Ask::Cancel { reason, .. } => {
            within(key, RestoreBound::Words, reason.len(), limits.words)
        }
        Ask::AnswerPerson { result, .. } => person_result(key, limits, result),
        Ask::Adopt { adoption: value, .. } => adoption(key, limits, value),
        Ask::EditNote { scope, change, .. } => note(key, limits, scope, change),
        Ask::MakeService { name, .. } => {
            within(key, RestoreBound::Words, name.len(), limits.words)?;
            let held =
                owned_size(key, name.len().checked_add(size_of::<u64>()), "owned service identity bytes fit usize");
            within(key, RestoreBound::IdentityBytes, held, limits.identity_bytes)
        }
        Ask::SetRoles { holdings, .. } => within(key, RestoreBound::Holdings, holdings.len(), limits.holdings),
        Ask::Rename { name, .. } | Ask::MakeProject { name, .. } => {
            within(key, RestoreBound::NameBytes, name.len(), u32::try_from(crate::NAME_BYTES).expect("name bound fits"))
        }
        Ask::ChangePolicy { change, .. } => bytes(
            key,
            RestoreBound::AmendmentBytes,
            crate::policy_change_bytes(change),
            limits.amendment_bytes,
            "owned policy change bytes fit u64",
        ),
        Ask::DecideEscalation { decision, .. } => match decision {
            crate::EscalationDecision::Reject { reason } => {
                within(key, RestoreBound::Words, reason.len(), limits.words)
            }
            crate::EscalationDecision::Release | crate::EscalationDecision::Pass => Ok(()),
        },
        Ask::DecideProposal { decision, .. } => match decision {
            crate::ProposalDecision::Reject { reason } => within(key, RestoreBound::Words, reason.len(), limits.words),
            crate::ProposalDecision::Accept | crate::ProposalDecision::Pass => Ok(()),
        },
        Ask::AccountChange { change, .. } => account_change(key, limits, change),
        Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::SetPool { .. }
        | Ask::Withdraw { .. }
        | Ask::Lift { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::Stop { .. }
        | Ask::Release { .. } => Ok(()),
    }
}

pub(super) fn act(key: &Key, limits: &Limits, value: &Act) -> Result<(), RestoreCause> {
    match value {
        Act::Close { reason, .. } => within(key, RestoreBound::Words, reason.len(), limits.words),
        Act::Adopt { adoption: value, .. } => adoption(key, limits, value),
        Act::MakeProject { name, .. } => {
            within(key, RestoreBound::NameBytes, name.len(), u32::try_from(crate::NAME_BYTES).expect("name bound fits"))
        }
        Act::AddKey { .. } | Act::RemoveAccount { .. } => Ok(()),
    }
}

pub(super) fn outcome(key: &Key, limits: &Limits, outcome: &Outcome) -> Result<(), RestoreCause> {
    match outcome {
        Outcome::Refused(why) | Outcome::CompoundRefused { why, .. } => refusal(key, why),
        Outcome::AccountSignedOut { failed_hosts, .. } | Outcome::AccountRemoved { failed_hosts, .. } => {
            within(key, RestoreBound::ProgramHosts, failed_hosts.len(), limits.program_hosts)?;
            for row in failed_hosts {
                within(key, RestoreBound::ProgramHostNameBytes, row.name.len(), limits.program_host_name_bytes)?;
            }
            Ok(())
        }
        Outcome::AccountProgramDeclared { name, .. } => within(key, RestoreBound::ProgramNameBytes, name.len(), 32),
        Outcome::AccountSignInStarted { address, .. } => {
            within(key, RestoreBound::AddressBytes, address.len(), crate::ADDRESS)
        }
        Outcome::Compound { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::Created { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Lifted { .. }
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
        | Outcome::Renamed { .. }
        | Outcome::ProjectMade { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::PoolSet { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountFinishing { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountChanged
        | Outcome::SignedOut
        | Outcome::Again(_) => Ok(()),
    }
}

fn refusal(key: &Key, why: &Refusal) -> Result<(), RestoreCause> {
    match why {
        Refusal::Circle { tasks, .. } | Refusal::Full { tasks, .. } => {
            within(key, RestoreBound::NamedTasks, tasks.len(), crate::NAMED_TASKS)
        }
        Refusal::Kept { pending } => within(key, RestoreBound::KeptPending, pending.len(), 64),
        Refusal::SignInRedirect { .. }
        | Refusal::Stale { .. }
        | Refusal::NameTaken
        | Refusal::AccountMove { .. }
        | Refusal::Dependency { .. }
        | Refusal::Account { .. }
        | Refusal::NotWellFormed { .. }
        | Refusal::Role { .. }
        | Refusal::Authority { .. }
        | Refusal::Limit { .. }
        | Refusal::Moved { .. }
        | Refusal::Ended { .. }
        | Refusal::Taken { .. }
        | Refusal::NotWaiting
        | Refusal::NoFurther
        | Refusal::Unknown
        | Refusal::NotOffered
        | Refusal::Provider
        | Refusal::KeyConflict
        | Refusal::NoEscalationHolder
        | Refusal::Llm { .. } => Ok(()),
    }
}

fn account_change(key: &Key, limits: &Limits, change: &crate::AccountChange) -> Result<(), RestoreCause> {
    match change {
        crate::AccountChange::DeclareProgram { name, .. } => {
            within(key, RestoreBound::ProgramNameBytes, name.len(), 32)
        }
        crate::AccountChange::SetUses { uses: Some(uses), .. } => {
            within(key, RestoreBound::Uses, uses.len(), crate::USES)?;
            let held =
                owned_size(key, uses.len().checked_mul(size_of::<crate::Use>()), "owned account use bytes fit usize");
            within(key, RestoreBound::Words, held, limits.words)
        }
        crate::AccountChange::SetUses { uses: None, .. }
        | crate::AccountChange::CheckProgram { .. }
        | crate::AccountChange::AddKey { .. }
        | crate::AccountChange::ReplaceKey { .. }
        | crate::AccountChange::RemoveKey { .. }
        | crate::AccountChange::Resume { .. }
        | crate::AccountChange::Remove { .. }
        | crate::AccountChange::MoveAccount { .. }
        | crate::AccountChange::MoveSource { .. }
        | crate::AccountChange::MoveOut { .. }
        | crate::AccountChange::Written { .. }
        | crate::AccountChange::MoveIn { .. }
        | crate::AccountChange::FinishSignIn { .. } => Ok(()),
    }
}
