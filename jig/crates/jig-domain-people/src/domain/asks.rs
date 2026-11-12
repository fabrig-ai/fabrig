//! Request shape and payload validation (domain/people/acts.md, 1).
use crate::{Ask, LimitKind, Limits, Refusal, Seen};
use alloc::boxed::Box;

pub(super) fn project(ask: &Ask) -> u32 {
    ask.family().named_project().unwrap_or(crate::DEPLOYMENT)
}

pub(super) fn valid_adoption(limits: &Limits, adoption: &crate::Adoption) -> bool {
    let Some(items) = adoption.resource.path.len().checked_add(adoption.options.len()) else { return false };
    let Some(mut bytes) = items.checked_mul(size_of::<Box<[u8]>>()) else { return false };
    for segment in &adoption.resource.path {
        let Some(next) = bytes.checked_add(segment.len()) else { return false };
        bytes = next;
    }
    for option in &adoption.options {
        let Some(next) = bytes.checked_add(option.len()) else { return false };
        bytes = next;
    }
    !adoption.resource.path.is_empty() && bytes <= skein_lib::length::usize_of(limits.amendment_bytes)
}

pub(super) fn note_pattern_bytes(pattern: &crate::Pattern) -> Option<usize> {
    let mut bytes = pattern.segments.len().checked_mul(size_of::<Box<[u8]>>())?;
    for segment in &pattern.segments {
        bytes = bytes.checked_add(segment.len())?;
    }
    let last = match &pattern.last {
        crate::Last::Exact(bytes) | crate::Last::Open(bytes) => bytes.len(),
    };
    bytes.checked_add(last)
}

fn valid_note(limits: &Limits, name: u64, scope: &crate::NoteScope, change: &crate::NoteChange) -> bool {
    if name == 0 {
        return false;
    }
    let scope_bytes = match scope {
        crate::NoteScope::Deployment | crate::NoteScope::Project | crate::NoteScope::Party { .. } => 0,
        crate::NoteScope::Goal { goal } => {
            if *goal == 0 {
                return false;
            }
            0
        }
        crate::NoteScope::Resources { pattern, .. } => {
            let Some(bytes) = note_pattern_bytes(pattern) else { return false };
            bytes
        }
    };
    let change_bytes = match change {
        crate::NoteChange::Correct { description, body, references, recalled } => {
            if *recalled == 0 {
                return false;
            }
            for number in references {
                if *number == 0 {
                    return false;
                }
            }
            let Some(bytes) = description.len().checked_add(body.len()) else { return false };
            let Some(ref_bytes) = references.len().checked_mul(size_of::<u64>()) else { return false };
            let Some(total) = bytes.checked_add(ref_bytes) else { return false };
            total
        }
        crate::NoteChange::Delete { recalled } => {
            if *recalled == 0 {
                return false;
            }
            0
        }
    };
    match scope_bytes.checked_add(change_bytes) {
        Some(bytes) => bytes <= skein_lib::length::usize_of(limits.amendment_bytes),
        None => false,
    }
}

fn valid_person_result(limits: &Limits, result: &crate::PersonResult) -> bool {
    let bound = skein_lib::length::usize_of(limits.words);
    match result {
        crate::PersonResult::Report { words } | crate::PersonResult::Verdict { words, .. } => words.len() <= bound,
        crate::PersonResult::Failure { reason } => reason.len() <= bound,
        crate::PersonResult::Approved { state, words } => {
            let words = match words {
                Some(words) => words.len(),
                None => 0,
            };
            match state.len().checked_add(words) {
                Some(bytes) => bytes <= bound,
                None => false,
            }
        }
    }
}

pub(crate) fn valid_ask(limits: &Limits, ask: &Ask) -> bool {
    if !crate::words::valid(ask) {
        return false;
    }
    match ask {
        Ask::Restore { reason, .. } => reason.len() <= skein_lib::length::usize_of(limits.words),
        Ask::Compound { operations, .. } => crate::compound::refusal(limits, operations).is_none(),
        Ask::MarkSeen { seen } => match seen {
            Seen::Results { .. } => true,
            Seen::Conversation { task, .. } => *task != 0,
        },
        Ask::AccountChange { change: crate::AccountChange::DeclareProgram { name, .. } | crate::AccountChange::LoginProgram { name, .. }, .. } => {
            jig_domain_link::SourceName::new(name).is_some()
        }
        Ask::AccountChange { change: crate::AccountChange::SetUses { uses: Some(uses), .. }, .. } => {
            uses.len() <= skein_lib::length::usize_of(crate::USES)
                && match uses.len().checked_mul(size_of::<crate::Use>()) {
                    Some(bytes) => bytes <= skein_lib::length::usize_of(limits.words),
                    None => false,
                }
        }
        Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::Application { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => true,
        Ask::EditNote { name, scope, change, .. } => valid_note(limits, *name, scope, change),
        Ask::MakeService { name, .. } => {
            let Some(bytes) = name.len().checked_add(size_of::<u64>()) else {
                return false;
            };
            !name.is_empty()
                && name.len() <= skein_lib::length::usize_of(limits.words)
                && bytes <= skein_lib::length::usize_of(limits.identity_bytes)
        }
        Ask::Move { task, reason, .. } | Ask::Cancel { task, reason, .. } => {
            *task != 0 && reason.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Lift { hold, .. } => *hold != 0,
        Ask::TakePerson { task, .. }
        | Ask::HandBackPerson { task, .. }
        | Ask::Stop { task, .. }
        | Ask::Leave { task, .. }
        | Ask::Release { task, .. } => *task != 0,
        Ask::AnswerPerson { task, result, .. } => *task != 0 && valid_person_result(limits, result),
        Ask::Adopt { adoption, .. } => valid_adoption(limits, adoption),
        Ask::SetRoles { holdings, .. } => holdings.len() <= skein_lib::length::usize_of(limits.holdings),
        Ask::Rename { name, .. } | Ask::MakeProject { name, .. } => name.len() <= crate::NAME_BYTES,
        Ask::ChangePolicy { change, .. } => match crate::policy_change_bytes(change) {
            Some(bytes) => bytes <= u64::from(limits.amendment_bytes),
            None => false,
        },
        Ask::SetPool { person, .. } => *person != 0,
        Ask::Create { batch, .. } => crate::batch::valid(limits, batch),
        Ask::Propose { action: crate::Action::Batch(batch), reason, .. } => {
            crate::batch::valid(limits, batch) && reason.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Propose { action: crate::Action::Amend { task, amendment, .. }, reason, .. } => {
            *task != 0
                && reason.len() <= skein_lib::length::usize_of(limits.words)
                && crate::amendment::valid(limits, amendment)
        }
        Ask::Propose { project, action: crate::Action::Settings { .. }, reason } => {
            *project == crate::DEPLOYMENT && reason.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Withdraw { proposal, .. } => *proposal != 0,
        Ask::Talk { task, words, .. } | Ask::Say { task, words, .. } => {
            *task != 0 && !words.is_empty() && words.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::AnswerQuestion { task, question, words, .. } => {
            *task != 0
                && *question != 0
                && !words.is_empty()
                && words.len() <= skein_lib::length::usize_of(limits.words)
        }
        Ask::Amend { task, amendment, .. } => *task != 0 && crate::amendment::valid(limits, amendment),
        Ask::DecideEscalation { task, revision, decision, .. } => {
            *task != 0
                && *revision != 0
                && match decision {
                    crate::EscalationDecision::Release | crate::EscalationDecision::Pass => true,
                    crate::EscalationDecision::Reject { reason } => {
                        reason.len() <= skein_lib::length::usize_of(limits.words)
                    }
                }
        }
        Ask::DecideProposal { proposer, proposal, decision, running, .. } => {
            (running.is_none() || *decision == crate::ProposalDecision::Accept)
                && proposer.number() != 0
                && *proposal != 0
                && match decision {
                    crate::ProposalDecision::Accept | crate::ProposalDecision::Pass => true,
                    crate::ProposalDecision::Reject { reason } => {
                        reason.len() <= skein_lib::length::usize_of(limits.words)
                    }
                }
        }
    }
}

// Admission identifies the bounded payload before any keyed flight is made.
pub(crate) fn invalid_ask(limits: &Limits, ask: &Ask) -> Refusal {
    let which = match ask {
        Ask::Compound { operations, .. } => {
            return crate::compound::refusal(limits, operations).expect("invalid compound");
        }
        Ask::EditNote { name, .. } => {
            if *name == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Notes
        }
        Ask::MakeService { name, .. } => {
            if name.is_empty() {
                LimitKind::Empty
            } else {
                LimitKind::Identity
            }
        }
        Ask::Move { task, .. } | Ask::Cancel { task, .. } | Ask::AnswerPerson { task, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Words
        }
        Ask::MarkSeen { .. }
        | Ask::Application { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::Stop { .. }
        | Ask::Lift { .. }
        | Ask::Leave { .. }
        | Ask::Release { .. }
        | Ask::SetPool { .. }
        | Ask::Withdraw { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. } => return Refusal::Unknown,
        Ask::Adopt { .. } => LimitKind::Resource,
        Ask::SetRoles { .. } => LimitKind::Roles,
        Ask::ChangePolicy { .. } => LimitKind::Policy,
        Ask::Create { batch, .. } | Ask::Propose { action: crate::Action::Batch(batch), .. } => {
            crate::batch::invalid(limits, batch)
        }
        Ask::RemoveProject { .. }
        | Ask::SetVisibility { .. }
        | Ask::Restore { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::AccountChange { .. }
        | Ask::Propose { action: crate::Action::Amend { .. } | crate::Action::Settings { .. }, .. } => LimitKind::Words,
        Ask::Talk { task, words, .. } | Ask::Say { task, words, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            if words.is_empty() { LimitKind::Empty } else { LimitKind::Words }
        }
        Ask::AnswerQuestion { task, question, words, .. } => {
            if *task == 0 || *question == 0 {
                return Refusal::Unknown;
            }
            if words.is_empty() { LimitKind::Empty } else { LimitKind::Words }
        }
        Ask::Amend { task, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Spec
        }
        Ask::DecideEscalation { task, revision, .. } => {
            if *task == 0 {
                return Refusal::Unknown;
            }
            if *revision == 0 { LimitKind::Revision } else { LimitKind::Words }
        }
        Ask::DecideProposal { proposer, proposal, decision, running, .. } => {
            if running.is_some() && *decision != crate::ProposalDecision::Accept {
                return Refusal::NotWellFormed { at: 0, why: crate::Malformed::Running };
            }
            if proposer.number() == 0 || *proposal == 0 {
                return Refusal::Unknown;
            }
            LimitKind::Words
        }
    };
    Refusal::Limit { which }
}

#[cfg(test)]
mod classifications {
    use super::*;
    use crate::tests::{LIMITS, ask_classes};
    use crate::{ApplicationScope, AskFamily, Domain, Holding, Identity, IdentityKey, Outcome, RequestKey, Role};
    use skein_lib::{Env, List, Queue, Time, Wall};

    #[test]
    fn ask_classes_keep_all_three_scopes_and_answer_reservations() {
        for (ask, family) in ask_classes() {
            assert_eq!(ask.family(), family, "{ask:?}");
            let (scope, capacity, deletion, reserved) = match family {
                AskFamily::Project(project) | AskFamily::Application(ApplicationScope::Project(project)) => {
                    (project, Some(project), Some(project), true)
                }
                AskFamily::Restore | AskFamily::Deployment | AskFamily::Application(ApplicationScope::Deployment) => {
                    (crate::DEPLOYMENT, Some(crate::DEPLOYMENT), None, true)
                }
                AskFamily::Application(ApplicationScope::Own) | AskFamily::Account | AskFamily::Lift => {
                    (0, None, None, true)
                }
                AskFamily::Seen => (0, None, None, false),
            };
            assert_eq!(project(&ask), scope, "{ask:?}");
            assert_eq!(crate::capacity::project(&ask), capacity, "{ask:?}");
            assert_eq!(super::super::deletion::ask_project(&ask), deletion, "{ask:?}");
            assert_eq!(super::super::requests::reserves_answer(&ask), reserved, "{ask:?}");
        }
    }

    #[test]
    #[expect(
        clippy::wildcard_enum_match_arm,
        clippy::disallowed_methods,
        reason = "step tests use ordinary Rust to read expected roster members from every table row"
    )]
    fn every_ask_only_applies_rosters_with_their_exact_members() {
        for (ask, _) in ask_classes() {
            let expected = match &ask {
                Ask::SetRoles { project, holdings } => Some((*project, holdings.clone())),
                _ => None,
            };
            let expected_roster = expected.as_ref().map(|(project, holdings)| (*project, holdings.as_ref()));
            assert_eq!(ask.roster(), expected_roster, "{ask:?}");
            let mut domain = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
            let env = Env { limits: LIMITS, now: Time::ZERO, wall: Wall::EPOCH };
            domain.start_restore(env.wall);
            assert!(domain.start_link());
            domain.start_sweep();
            assert!(domain.start_complete(&env));
            domain.roles.insert(7, Box::new([Holding { person: 9, role: Role::Owner }])).unwrap();
            for person in [8, 9] {
                domain
                    .people
                    .insert(
                        person,
                        Identity {
                            key: IdentityKey { provider: 0, subject: Box::new([u8::try_from(person).unwrap()]) },
                            login: Box::new([]),
                            name: Box::new([]),
                        },
                    )
                    .unwrap();
            }
            let id = domain
                .pending
                .insert(super::super::requests::Pending::Party(super::super::requests::PartyPending {
                    key: RequestKey { person: 9, key: [8; 16] },
                    ask: ask.clone(),
                    models: Box::new([]),
                    replies: List::with_capacity(0),
                    part: None,
                }))
                .unwrap();
            let mut out = Queue::uncapped();
            let result = super::super::parties::apply_roles(&mut domain, &env, id.token(), &mut out);
            match expected {
                Some((project, holdings)) => {
                    assert_eq!(result, Ok(()), "{ask:?}");
                    assert_eq!(domain.roles.get(&project), Some(&holdings));
                }
                None => {
                    assert_eq!(result, Err(Outcome::Refused(Refusal::Unknown)), "{ask:?}");
                    assert!(out.pop().is_none());
                }
            }
        }
    }
}
