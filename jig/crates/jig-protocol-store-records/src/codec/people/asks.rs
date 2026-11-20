//! Asks values retained by people (protocol/store/records.md, 1).
mod account_changes;
use super::{
    Box, decode_amendment, decode_holdings, decode_numbers, decode_pattern, decode_policy_change, decode_role,
    decode_segments, domain, encode_amendment, encode_holdings, encode_numbers, encode_pattern, encode_policy_change,
    encode_role, encode_segments, wire,
};
pub(super) fn encode_seen(value: &domain::Seen, _limits: &wire::Limits) -> wire::Seen {
    match value {
        domain::Seen::Results { through } => wire::Seen::Results(wire::SeenResults { through: *through }),
        domain::Seen::Conversation { task, through } => {
            wire::Seen::Conversation(wire::SeenConversation { task: *task, through: *through })
        }
    }
}
pub(super) fn decode_seen(value: wire::Seen) -> domain::Seen {
    match value {
        wire::Seen::Results(value) => {
            let wire::SeenResults { through } = value;
            domain::Seen::Results { through }
        }
        wire::Seen::Conversation(value) => {
            let wire::SeenConversation { task, through } = value;
            domain::Seen::Conversation { task, through }
        }
    }
}

pub(super) fn encode_note_scope(
    value: &domain::NoteScope,
    limits: &wire::Limits,
) -> Result<wire::NoteScope, wire::Problem> {
    Ok(match value {
        domain::NoteScope::Party { party } => wire::NoteScope::Party(wire::NoteScopeParty { party: *party }),
        domain::NoteScope::Deployment => wire::NoteScope::Deployment,
        domain::NoteScope::Project => wire::NoteScope::Project,
        domain::NoteScope::Goal { goal } => wire::NoteScope::Task(wire::NoteScopeTask { task: *goal }),
        domain::NoteScope::Resources { connector, pattern } => wire::NoteScope::Resources(wire::NoteScopeResources {
            connector: *connector,
            pattern: encode_pattern(pattern, limits)?,
        }),
    })
}
pub(super) fn decode_note_scope(
    value: wire::NoteScope,
    limits: &wire::Limits,
) -> Result<domain::NoteScope, wire::Problem> {
    Ok(match value {
        wire::NoteScope::Party(value) => domain::NoteScope::Party { party: value.party },
        wire::NoteScope::Deployment => domain::NoteScope::Deployment,
        wire::NoteScope::Project => domain::NoteScope::Project,
        wire::NoteScope::Task(value) => {
            let wire::NoteScopeTask { task } = value;
            domain::NoteScope::Goal { goal: task }
        }
        wire::NoteScope::Resources(value) => {
            let wire::NoteScopeResources { connector, pattern } = value;
            domain::NoteScope::Resources { connector, pattern: decode_pattern(pattern, limits)? }
        }
    })
}

pub(super) fn encode_note_change(
    value: &domain::NoteChange,
    limits: &wire::Limits,
) -> Result<wire::NoteChange, wire::Problem> {
    Ok(match value {
        domain::NoteChange::Correct { description, body, references, recalled } => {
            wire::NoteChange::Correct(wire::NoteChangeCorrect {
                description: description.clone(),
                body: body.clone(),
                references: encode_numbers(
                    references,
                    limits,
                    limits.parameter_bytes_value,
                    limits.note_change_correct_references,
                )?,
                recalled: *recalled,
            })
        }
        domain::NoteChange::Delete { recalled } => {
            wire::NoteChange::Delete(wire::NoteChangeDelete { recalled: *recalled })
        }
    })
}
pub(super) fn decode_note_change(
    value: wire::NoteChange,
    limits: &wire::Limits,
) -> Result<domain::NoteChange, wire::Problem> {
    Ok(match value {
        wire::NoteChange::Correct(value) => {
            let wire::NoteChangeCorrect { description, body, references, recalled } = value;
            domain::NoteChange::Correct {
                description,
                body,
                references: decode_numbers(references, limits, limits.parameter_bytes_value)?,
                recalled,
            }
        }
        wire::NoteChange::Delete(value) => {
            let wire::NoteChangeDelete { recalled } = value;
            domain::NoteChange::Delete { recalled }
        }
    })
}

pub(super) fn encode_resource_role(value: domain::ResourceRole) -> wire::ResourceRole {
    match value {
        domain::ResourceRole::Owned => wire::ResourceRole::Owned,
        domain::ResourceRole::Fork => wire::ResourceRole::Fork,
        domain::ResourceRole::Context => wire::ResourceRole::Context,
    }
}
pub(super) fn decode_resource_role(value: wire::ResourceRole) -> domain::ResourceRole {
    match value {
        wire::ResourceRole::Owned => domain::ResourceRole::Owned,
        wire::ResourceRole::Fork => domain::ResourceRole::Fork,
        wire::ResourceRole::Context => domain::ResourceRole::Context,
    }
}

pub(super) fn encode_resource_name(
    value: &domain::ResourceName,
    limits: &wire::Limits,
) -> Result<wire::ResourceName, wire::Problem> {
    let domain::ResourceName { connector, path } = value;
    Ok(wire::ResourceName {
        connector: *connector,
        path: encode_segments(path, limits, limits.parameter_bytes_value, limits.resource_name_path)?,
    })
}
pub(super) fn decode_resource_name(
    value: wire::ResourceName,
    limits: &wire::Limits,
) -> Result<domain::ResourceName, wire::Problem> {
    let wire::ResourceName { connector, path } = value;
    Ok(domain::ResourceName { connector, path: decode_segments(path, limits, limits.parameter_bytes_value)? })
}

pub(super) fn encode_adoption(
    value: &domain::Adoption,
    limits: &wire::Limits,
) -> Result<wire::Adoption, wire::Problem> {
    let domain::Adoption { resource, role, options } = value;
    Ok(wire::Adoption {
        resource: encode_resource_name(resource, limits)?,
        role: encode_resource_role(*role),
        options: encode_segments(options, limits, limits.parameter_bytes_value, limits.adoption_options)?,
    })
}
pub(super) fn decode_adoption(value: wire::Adoption, limits: &wire::Limits) -> Result<domain::Adoption, wire::Problem> {
    let wire::Adoption { resource, role, options } = value;
    Ok(domain::Adoption {
        resource: decode_resource_name(resource, limits)?,
        role: decode_resource_role(role),
        options: decode_segments(options, limits, limits.parameter_bytes_value)?,
    })
}

pub(super) fn encode_person_result(value: &domain::PersonResult, _limits: &wire::Limits) -> wire::PersonResult {
    match value {
        domain::PersonResult::Approved { state, words } => {
            wire::PersonResult::Approved(wire::PersonResultApproved { state: state.clone(), words: words.clone() })
        }
        domain::PersonResult::Report { words } => {
            wire::PersonResult::Report(wire::PersonResultReport { words: words.clone() })
        }
        domain::PersonResult::Verdict { code, words } => {
            wire::PersonResult::Verdict(wire::PersonResultVerdict { code: *code, words: words.clone() })
        }
        domain::PersonResult::Failure { reason } => {
            wire::PersonResult::Failure(wire::PersonResultFailure { reason: reason.clone() })
        }
    }
}
pub(super) fn decode_person_result(value: wire::PersonResult) -> domain::PersonResult {
    match value {
        wire::PersonResult::Approved(value) => {
            let wire::PersonResultApproved { state, words } = value;
            domain::PersonResult::Approved { state, words }
        }
        wire::PersonResult::Report(value) => {
            let wire::PersonResultReport { words } = value;
            domain::PersonResult::Report { words }
        }
        wire::PersonResult::Verdict(value) => {
            let wire::PersonResultVerdict { code, words } = value;
            domain::PersonResult::Verdict { code, words }
        }
        wire::PersonResult::Failure(value) => {
            let wire::PersonResultFailure { reason } = value;
            domain::PersonResult::Failure { reason }
        }
    }
}

pub(super) fn encode_proposer(value: &domain::Proposer, _limits: &wire::Limits) -> wire::Proposer {
    match value {
        domain::Proposer::Task(value) => wire::Proposer::Task(wire::ProposerTask { value: *value }),
        domain::Proposer::Party(value) => wire::Proposer::Party(wire::ProposerParty { value: *value }),
    }
}
pub(super) fn decode_proposer(value: wire::Proposer) -> domain::Proposer {
    match value {
        wire::Proposer::Task(value) => {
            let wire::ProposerTask { value } = value;
            domain::Proposer::Task(value)
        }
        wire::Proposer::Party(value) => {
            let wire::ProposerParty { value } = value;
            domain::Proposer::Party(value)
        }
    }
}

pub(super) fn encode_proposal_decision(
    value: &domain::ProposalDecision,
    _limits: &wire::Limits,
) -> wire::ProposalDecision {
    match value {
        domain::ProposalDecision::Accept => wire::ProposalDecision::Accept,
        domain::ProposalDecision::Reject { reason } => {
            wire::ProposalDecision::Reject(wire::ProposalDecisionReject { reason: reason.clone() })
        }
        domain::ProposalDecision::Pass => wire::ProposalDecision::Pass,
    }
}
pub(super) fn decode_proposal_decision(value: wire::ProposalDecision) -> domain::ProposalDecision {
    match value {
        wire::ProposalDecision::Accept => domain::ProposalDecision::Accept,
        wire::ProposalDecision::Reject(value) => {
            let wire::ProposalDecisionReject { reason } = value;
            domain::ProposalDecision::Reject { reason }
        }
        wire::ProposalDecision::Pass => domain::ProposalDecision::Pass,
    }
}

pub(super) fn encode_escalation_decision(
    value: &domain::EscalationDecision,
    _limits: &wire::Limits,
) -> wire::EscalationDecision {
    match value {
        domain::EscalationDecision::Release => wire::EscalationDecision::Release,
        domain::EscalationDecision::Reject { reason } => {
            wire::EscalationDecision::Reject(wire::EscalationDecisionReject { reason: reason.clone() })
        }
        domain::EscalationDecision::Pass => wire::EscalationDecision::Pass,
    }
}
pub(super) fn decode_escalation_decision(value: wire::EscalationDecision) -> domain::EscalationDecision {
    match value {
        wire::EscalationDecision::Release => domain::EscalationDecision::Release,
        wire::EscalationDecision::Reject(value) => {
            let wire::EscalationDecisionReject { reason } = value;
            domain::EscalationDecision::Reject { reason }
        }
        wire::EscalationDecision::Pass => domain::EscalationDecision::Pass,
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive arm translates each kind of keyed request")]
pub fn encode_ask(value: &domain::Ask, limits: &wire::Limits) -> Result<wire::Ask, wire::Problem> {
    Ok(match value {
        domain::Ask::SetVisibility { project, public } => {
            wire::Ask::SetVisibility(wire::AskSetVisibility { project: *project, public: *public })
        }
        domain::Ask::Restore { revision, reason } => {
            wire::Ask::Restore(wire::AskRestore { revision: *revision, reason: reason.clone() })
        }
        domain::Ask::RemoveProject { project, grace } => wire::Ask::RemoveProject(wire::AskRemoveProject {
            project: *project,
            grace: super::compound::encode_grace(*grace),
        }),
        domain::Ask::MoveToNewest { revision } => {
            wire::Ask::MoveToNewest(wire::AskMoveToNewest { revision: *revision })
        }
        domain::Ask::ChangeSettings { against, digest, projects } => {
            wire::Ask::ChangeSettings(wire::AskChangeSettings {
                against: *against,
                digest: *digest,
                projects: encode_projects(projects, limits)?,
            })
        }
        domain::Ask::Compound { project, operations } => wire::Ask::Compound(wire::AskCompound {
            project: *project,
            operations: super::compound::encode(operations, limits)?,
        }),
        domain::Ask::Propose { project, action, reason } => {
            let action = match action {
                domain::Action::Settings { against, digest } => {
                    wire::Action::Settings(wire::ActionSettings { against: *against, digest: *digest })
                }
                domain::Action::Batch(batch) => {
                    wire::Action::Batch(wire::ActionBatch { members: super::batch::encode(batch, limits)? })
                }
                domain::Action::Amend { task, revision, amendment } => {
                    wire::Action::Amend(Box::new(wire::ActionAmend {
                        task: *task,
                        revision: *revision,
                        amendment: encode_amendment(amendment, limits)?,
                    }))
                }
            };
            wire::Ask::Propose(wire::AskPropose { project: *project, action, reason: reason.clone() })
        }
        domain::Ask::Withdraw { project, proposal } => {
            wire::Ask::Withdraw(wire::AskWithdraw { project: *project, proposal: *proposal })
        }
        domain::Ask::Delegate { project, task, batch } => wire::Ask::Delegate(wire::AskDelegate {
            project: *project,
            task: *task,
            batch: super::batch::encode(batch, limits)?,
        }),
        domain::Ask::Create { project, batch } => {
            wire::Ask::Create(wire::AskCreate { project: *project, batch: super::batch::encode(batch, limits)? })
        }
        domain::Ask::MarkSeen { seen } => wire::Ask::MarkSeen(wire::AskMarkSeen { seen: encode_seen(seen, limits) }),
        domain::Ask::Application { scope, connector, digest } => wire::Ask::Application(wire::AskApplication {
            scope: encode_application_scope(*scope, limits),
            connector: *connector,
            digest: *digest,
        }),
        domain::Ask::EditNote { project, name, scope, change } => wire::Ask::EditNote(wire::AskEditNote {
            project: *project,
            name: *name,
            scope: encode_note_scope(scope, limits)?,
            change: encode_note_change(change, limits)?,
        }),
        domain::Ask::MakeService { project, name, role } => wire::Ask::MakeService(wire::AskMakeService {
            project: *project,
            name: name.clone(),
            role: encode_role(*role, limits),
        }),
        domain::Ask::Adopt { project, adoption } => {
            wire::Ask::Adopt(wire::AskAdopt { project: *project, adoption: encode_adoption(adoption, limits)? })
        }
        domain::Ask::Stop { project, task } => wire::Ask::Stop(wire::AskStop { project: *project, task: *task }),
        domain::Ask::Conclude { project, task } => {
            wire::Ask::Conclude(wire::AskConclude { project: *project, task: *task })
        }
        domain::Ask::Cancel { project, task, reason, grace } => wire::Ask::Cancel(wire::AskCancel {
            project: *project,
            task: *task,
            reason: reason.clone(),
            grace: match grace {
                Some(span) => Some(span.as_nanos()),
                None => None,
            },
        }),
        domain::Ask::Lift { hold, revision, with_tasks } => {
            wire::Ask::Lift(wire::AskLift { hold: *hold, revision: *revision, with_tasks: *with_tasks })
        }
        domain::Ask::Release { project, task, revision, again } => {
            wire::Ask::Release(wire::AskRelease { project: *project, task: *task, revision: *revision, again: *again })
        }
        domain::Ask::TakePerson { project, task } => {
            wire::Ask::TakePerson(wire::AskTakePerson { project: *project, task: *task })
        }
        domain::Ask::HandBackPerson { project, task } => {
            wire::Ask::HandBackPerson(wire::AskHandBackPerson { project: *project, task: *task })
        }
        domain::Ask::AnswerPerson { project, task, revision, result } => {
            wire::Ask::AnswerPerson(wire::AskAnswerPerson {
                project: *project,
                task: *task,
                revision: *revision,
                result: encode_person_result(result, limits),
            })
        }
        domain::Ask::Move { project, task, reason } => {
            wire::Ask::MoveTask(wire::AskMove { project: *project, task: *task, reason: reason.clone() })
        }
        domain::Ask::DecideProposal { project, proposer, proposal, decision, running } => {
            wire::Ask::DecideProposal(wire::AskDecideProposal {
                project: *project,
                proposer: encode_proposer(proposer, limits),
                proposal: *proposal,
                decision: encode_proposal_decision(decision, limits),
                running: match running {
                    Some(domain::Running::Stays) => Some(wire::Running::Stays),
                    Some(domain::Running::Moves) => Some(wire::Running::Moves),
                    None => None,
                },
            })
        }
        domain::Ask::Talk { project, task, words } => {
            wire::Ask::Talk(wire::AskSay { project: *project, task: *task, words: words.clone() })
        }
        domain::Ask::Leave { project, task } => wire::Ask::Leave(wire::AskStop { project: *project, task: *task }),
        domain::Ask::Say { project, task, words } => {
            wire::Ask::Say(wire::AskSay { project: *project, task: *task, words: words.clone() })
        }
        domain::Ask::AnswerQuestion { project, task, question, words } => {
            wire::Ask::AnswerQuestion(wire::AskAnswerQuestion {
                project: *project,
                task: *task,
                question: *question,
                words: words.clone(),
            })
        }
        domain::Ask::Amend { project, task, revision, amendment, release } => {
            wire::Ask::Amend(Box::new(wire::AskAmend {
                project: *project,
                task: *task,
                revision: *revision,
                amendment: encode_amendment(amendment, limits)?,
                release: *release,
            }))
        }
        domain::Ask::SetRoles { project, holdings } => {
            wire::Ask::SetRoles(wire::AskSetRoles { project: *project, holdings: encode_holdings(holdings, limits)? })
        }
        domain::Ask::Rename { project, name } => {
            wire::Ask::Rename(wire::AskRename { project: *project, name: name.as_bytes().into() })
        }
        domain::Ask::MakeProject { name, policy } => {
            wire::Ask::MakeProject(wire::AskMakeProject { name: name.as_bytes().into(), policy: *policy })
        }
        domain::Ask::ChangePolicy { project, change } => wire::Ask::ChangePolicy(wire::AskChangePolicy {
            project: *project,
            change: encode_policy_change(change, limits)?,
        }),
        domain::Ask::SetPool { project, person, budget } => {
            wire::Ask::SetPool(wire::AskSetPool { project: *project, person: *person, budget: *budget })
        }
        domain::Ask::DecideEscalation { project, task, revision, decision } => {
            wire::Ask::DecideEscalation(wire::AskDecideEscalation {
                project: *project,
                task: *task,
                revision: *revision,
                decision: encode_escalation_decision(decision, limits),
            })
        }
        domain::Ask::AccountChange { holder, change } => wire::Ask::AccountChange(wire::AskAccountChange {
            holder: encode_account_holder(*holder),
            change: account_changes::encode(change, limits)?,
        }),
        domain::Ask::AccountSignIn { holder, provider } => wire::Ask::AccountSignIn(wire::AskAccountSignIn {
            holder: encode_account_holder(*holder),
            provider: *provider,
        }),
        domain::Ask::AccountSignOut { holder, source } => wire::Ask::AccountSignOut(wire::AskAccountSignOut {
            holder: encode_account_holder(*holder),
            source: *source,
        }),
    })
}
#[expect(clippy::too_many_lines, reason = "one exhaustive arm translates each kind of keyed request")]
pub fn decode_ask(value: wire::Ask, limits: &wire::Limits) -> Result<domain::Ask, wire::Problem> {
    Ok(match value {
        wire::Ask::SetVisibility(value) => domain::Ask::SetVisibility { project: value.project, public: value.public },
        wire::Ask::RemoveProject(value) => {
            domain::Ask::RemoveProject { project: value.project, grace: super::compound::decode_grace(value.grace) }
        }
        wire::Ask::MoveToNewest(value) => domain::Ask::MoveToNewest { revision: value.revision },
        wire::Ask::Restore(value) => {
            let wire::AskRestore { revision, reason } = value;
            domain::Ask::Restore { revision, reason }
        }
        wire::Ask::ChangeSettings(value) => {
            let wire::AskChangeSettings { against, digest, projects } = value;
            domain::Ask::ChangeSettings { against, digest, projects: projects.into_boxed() }
        }
        wire::Ask::Compound(value) => {
            let wire::AskCompound { project, operations } = value;
            domain::Ask::Compound { project, operations: super::compound::decode(operations, limits)? }
        }
        wire::Ask::Propose(value) => {
            let wire::AskPropose { project, action, reason } = value;
            let action = match action {
                wire::Action::Settings(value) => {
                    domain::Action::Settings { against: value.against, digest: value.digest }
                }
                wire::Action::Batch(batch) => domain::Action::Batch(super::batch::decode(batch.members, limits)?),
                wire::Action::Amend(value) => {
                    let wire::ActionAmend { task, revision, amendment } = *value;
                    domain::Action::Amend { task, revision, amendment: Box::new(decode_amendment(amendment, limits)?) }
                }
            };
            domain::Ask::Propose { project, action, reason }
        }
        wire::Ask::Withdraw(value) => {
            let wire::AskWithdraw { project, proposal } = value;
            domain::Ask::Withdraw { project, proposal }
        }
        wire::Ask::Delegate(value) => {
            let wire::AskDelegate { project, task, batch } = value;
            domain::Ask::Delegate { project, task, batch: super::batch::decode(batch, limits)? }
        }
        wire::Ask::Create(value) => {
            let wire::AskCreate { project, batch } = value;
            domain::Ask::Create { project, batch: super::batch::decode(batch, limits)? }
        }
        wire::Ask::MarkSeen(value) => {
            let wire::AskMarkSeen { seen } = value;
            domain::Ask::MarkSeen { seen: decode_seen(seen) }
        }
        wire::Ask::Application(value) => {
            let wire::AskApplication { scope, connector, digest } = value;
            domain::Ask::Application { scope: decode_application_scope(scope), connector, digest }
        }
        wire::Ask::EditNote(value) => {
            let wire::AskEditNote { project, name, scope, change } = value;
            domain::Ask::EditNote {
                project,
                name,
                scope: Box::new(decode_note_scope(scope, limits)?),
                change: Box::new(decode_note_change(change, limits)?),
            }
        }
        wire::Ask::MakeService(value) => {
            let wire::AskMakeService { project, name, role } = value;
            domain::Ask::MakeService { project, name, role: decode_role(role) }
        }
        wire::Ask::Adopt(value) => {
            let wire::AskAdopt { project, adoption } = value;
            domain::Ask::Adopt { project, adoption: decode_adoption(adoption, limits)? }
        }
        wire::Ask::Stop(value) => {
            let wire::AskStop { project, task } = value;
            domain::Ask::Stop { project, task }
        }
        wire::Ask::Conclude(value) => domain::Ask::Conclude { project: value.project, task: value.task },
        wire::Ask::Cancel(value) => {
            let wire::AskCancel { project, task, reason, grace } = value;
            domain::Ask::Cancel {
                project,
                task,
                reason,
                grace: match grace {
                    Some(span) => Some(skein_lib::Duration::from_nanos(span)),
                    None => None,
                },
            }
        }
        wire::Ask::Lift(value) => {
            let wire::AskLift { hold, revision, with_tasks } = value;
            domain::Ask::Lift { hold, revision, with_tasks }
        }
        wire::Ask::Release(value) => {
            let wire::AskRelease { project, task, revision, again } = value;
            domain::Ask::Release { project, task, revision, again }
        }
        wire::Ask::TakePerson(value) => {
            let wire::AskTakePerson { project, task } = value;
            domain::Ask::TakePerson { project, task }
        }
        wire::Ask::HandBackPerson(value) => {
            let wire::AskHandBackPerson { project, task } = value;
            domain::Ask::HandBackPerson { project, task }
        }
        wire::Ask::AnswerPerson(value) => {
            let wire::AskAnswerPerson { project, task, revision, result } = value;
            domain::Ask::AnswerPerson { project, task, revision, result: decode_person_result(result) }
        }
        wire::Ask::MoveTask(value) => {
            let wire::AskMove { project, task, reason } = value;
            domain::Ask::Move { project, task, reason }
        }
        wire::Ask::DecideProposal(value) => {
            let wire::AskDecideProposal { project, proposer, proposal, decision, running } = value;
            domain::Ask::DecideProposal {
                project,
                proposer: decode_proposer(proposer),
                proposal,
                decision: decode_proposal_decision(decision),
                running: match running {
                    Some(wire::Running::Stays) => Some(domain::Running::Stays),
                    Some(wire::Running::Moves) => Some(domain::Running::Moves),
                    None => None,
                },
            }
        }
        wire::Ask::Talk(value) => {
            let v = value;
            domain::Ask::Talk { project: v.project, task: v.task, words: v.words }
        }
        wire::Ask::Leave(value) => domain::Ask::Leave { project: value.project, task: value.task },
        wire::Ask::Say(value) => {
            let wire::AskSay { project, task, words } = value;
            domain::Ask::Say { project, task, words }
        }
        wire::Ask::AnswerQuestion(value) => {
            let wire::AskAnswerQuestion { project, task, question, words } = value;
            domain::Ask::AnswerQuestion { project, task, question, words }
        }
        wire::Ask::Amend(value) => {
            let wire::AskAmend { project, task, revision, amendment, release } = *value;
            domain::Ask::Amend { project, task, revision, amendment: decode_amendment(amendment, limits)?, release }
        }
        wire::Ask::SetRoles(value) => {
            let wire::AskSetRoles { project, holdings } = value;
            domain::Ask::SetRoles { project, holdings: decode_holdings(holdings) }
        }
        wire::Ask::Rename(value) => {
            let wire::AskRename { project, name } = value;
            domain::Ask::Rename { project, name: project_name(&name, wire::paths::ASK_RENAME_NAME)? }
        }
        wire::Ask::MakeProject(value) => {
            let wire::AskMakeProject { name, policy } = value;
            domain::Ask::MakeProject { name: project_name(&name, wire::paths::ASK_MAKE_PROJECT_NAME)?, policy }
        }
        wire::Ask::ChangePolicy(value) => {
            let wire::AskChangePolicy { project, change } = value;
            domain::Ask::ChangePolicy { project, change: decode_policy_change(change, limits)? }
        }
        wire::Ask::SetPool(value) => {
            let wire::AskSetPool { project, person, budget } = value;
            domain::Ask::SetPool { project, person, budget }
        }
        wire::Ask::DecideEscalation(value) => {
            let wire::AskDecideEscalation { project, task, revision, decision } = value;
            domain::Ask::DecideEscalation { project, task, revision, decision: decode_escalation_decision(decision) }
        }
        wire::Ask::AccountChange(value) => domain::Ask::AccountChange {
            holder: decode_account_holder(value.holder),
            change: account_changes::decode(value.change)?,
        },
        wire::Ask::AccountSignIn(value) => {
            let wire::AskAccountSignIn { holder, provider } = value;
            domain::Ask::AccountSignIn { holder: decode_account_holder(holder), provider }
        }
        wire::Ask::AccountSignOut(value) => {
            let wire::AskAccountSignOut { holder, source } = value;
            domain::Ask::AccountSignOut { holder: decode_account_holder(holder), source }
        }
    })
}
const fn encode_account_holder(value: domain::AccountHolder) -> wire::AccountHolder {
    match value {
        domain::AccountHolder::Own => wire::AccountHolder::Own,
        domain::AccountHolder::Deployment => wire::AccountHolder::Deployment,
    }
}
const fn decode_account_holder(value: wire::AccountHolder) -> domain::AccountHolder {
    match value {
        wire::AccountHolder::Own => domain::AccountHolder::Own,
        wire::AccountHolder::Deployment => domain::AccountHolder::Deployment,
    }
}

fn encode_application_scope(scope: domain::ApplicationScope, _limits: &wire::Limits) -> wire::ApplicationScope {
    match scope {
        domain::ApplicationScope::Project(project) => wire::ApplicationScope::Project(wire::ScopeProject { project }),
        domain::ApplicationScope::Deployment => wire::ApplicationScope::Deployment,
        domain::ApplicationScope::Own => wire::ApplicationScope::Own,
    }
}
fn decode_application_scope(scope: wire::ApplicationScope) -> domain::ApplicationScope {
    match scope {
        wire::ApplicationScope::Project(project) => domain::ApplicationScope::Project(project.project),
        wire::ApplicationScope::Deployment => domain::ApplicationScope::Deployment,
        wire::ApplicationScope::Own => domain::ApplicationScope::Own,
    }
}

#[expect(clippy::disallowed_methods, reason = "stored project names are UTF-8 text")]
pub(super) fn project_name(bytes: &[u8], path: wire::Path) -> Result<Box<str>, wire::Problem> {
    match core::str::from_utf8(bytes) {
        Ok(name) => Ok(name.into()),
        Err(_) => Err(wire::Problem { path, reason: skein_codec::Reason::Utf8 }),
    }
}

fn encode_projects(projects: &[u32], limits: &wire::Limits) -> Result<skein_lib::List<u32>, wire::Problem> {
    let mut result = skein_lib::List::with_capacity(limits.ask_change_settings_projects);
    for &project in projects {
        if result.push(project).is_err() {
            return Err(wire::Problem {
                path: wire::paths::ASK_CHANGE_SETTINGS_PROJECTS,
                reason: skein_codec::Reason::Bound,
            });
        }
    }
    Ok(result)
}
