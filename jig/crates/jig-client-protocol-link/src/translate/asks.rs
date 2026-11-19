//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use jig_client_protocol_wire::Record as _;
use crate::translate::too_large;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use skein_lib::{List, Writer, bytes};
#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_revision(value: c::Revision, _limits: &w::Limits) -> Result<w::Revision, w::Problem> {
    Ok(w::Revision { value: value.value })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_person_answer(value: &c::PersonAnswer, _limits: &w::Limits) -> Result<w::PersonAnswer, w::Problem> {
    Ok(match value {
        c::PersonAnswer::Choice { option, reason } => w::PersonAnswer::Choice(w::PersonAnswerChoice { option: *option, reason: bytes::copy_of(reason.bytes()) }),
        c::PersonAnswer::Approved { state, words } => {
            let opt0 = match words {
                Some(item) => Some(bytes::copy_of(item.bytes())),
                None => None,
            };
            w::PersonAnswer::Approved(w::PersonAnswerApproved { state: bytes::copy_of(state), words: opt0 })
        }
        c::PersonAnswer::Words(value) => w::PersonAnswer::Words(w::PersonAnswerWords { value: bytes::copy_of(value.bytes()) }),
        c::PersonAnswer::Cannot { reason } => w::PersonAnswer::Cannot(w::PersonAnswerCannot { reason: bytes::copy_of(reason.bytes()) }),
    })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_proposer(value: &c::Proposer, _limits: &w::Limits) -> Result<w::Proposer, w::Problem> {
    Ok(match value {
        c::Proposer::Task(value) => {
            w::Proposer::Task(w::ProposerTask { value: *value })
        }
        c::Proposer::Party(value) => {
            w::Proposer::Party(w::ProposerParty { value: *value })
        }
    })
}

pub(crate) fn wire_proposal_key(value: &c::ProposalKey, limits: &w::Limits) -> Result<w::ProposalKey, w::Problem> {
    Ok(w::ProposalKey { proposer: wire_proposer(&value.proposer, limits)?, number: value.number })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_decision(value: &c::Decision, _limits: &w::Limits) -> Result<w::Decision, w::Problem> {
    Ok(match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => w::Decision::Rejected(w::DecisionRejected { reason: bytes::copy_of(reason.bytes()) }),
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => {
            let opt0 = match again {
                Some(item) => Some(*item),
                None => None,
            };
            w::Decision::Released(w::DecisionReleased { again: opt0 })
        }
        c::Decision::LeftHeld { reason } => w::Decision::LeftHeld(w::DecisionLeftHeld { reason: bytes::copy_of(reason.bytes()) }),
    })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_person_address(
    value: &c::PersonAddress,
    _limits: &w::Limits,
) -> Result<w::PersonAddress, w::Problem> {
    Ok(match value {
        c::PersonAddress::Party(value) => {
            w::PersonAddress::Party(w::PersonAddressParty { value: *value })
        }
        c::PersonAddress::Role(value) => {
            w::PersonAddress::Role(w::PersonAddressRole { value: *value })
        }
    })
}

pub(crate) fn wire_address_change(
    value: &c::AddressChange,
    limits: &w::Limits,
) -> Result<w::AddressChange, w::Problem> {
    Ok(w::AddressChange {
            revision: wire_revision(value.revision, limits)?,
            address: wire_person_address(&value.address, limits)?,
        })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_parameter(value: &c::Parameter, _limits: &w::Limits) -> Result<w::Parameter, w::Problem> {
    Ok(match value {
        c::Parameter::Number { name, value } => w::Parameter::Number(w::ParameterNumber { name: *name, value: *value }),
        c::Parameter::Bytes { name, value } => w::Parameter::Bytes(w::ParameterBytes { name: *name, value: bytes::copy_of(value.bytes()) }),
        c::Parameter::Resource { name, connector, resource } => w::Parameter::Resource(w::ParameterResource { name: *name, connector: *connector, resource: *resource }),
    })
}

pub(crate) fn wire_specification(value: &c::Specification, limits: &w::Limits) -> Result<w::Specification, w::Problem> {
    let mut list0 = List::with_capacity(value.parameters.len());
    for item in &value.parameters {
        list0.push(wire_parameter(item, limits)?).expect("reserved supplied list");
    }
    let mut list1 = List::with_capacity(value.inputs.len());
    for item in &value.inputs {
        list1.push(*item).expect("reserved supplied list");
    }
    Ok(w::Specification {
            asks: value.asks,
            words: bytes::copy_of(value.words.bytes()),
            parameters: list0,
            inputs: list1,
        })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_wake_rule(value: &c::WakeRule, _limits: &w::Limits) -> Result<w::WakeRule, w::Problem> {
    Ok(match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => {
            w::WakeRule::Batch(w::WakeRuleBatch { count: *count, age: *age })
        }
        c::WakeRule::Paced { span } => {
            w::WakeRule::Paced(w::WakeRulePaced { span: *span })
        }
    })
}

pub(crate) fn wire_results_wake(value: c::ResultsWake) -> w::ResultsWake {
    match value {
        c::ResultsWake::Never => w::ResultsWake::Never,
        c::ResultsWake::Each => w::ResultsWake::Each,
        c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
    }
}

pub(crate) fn wire_owner_wake_policy(value: &c::WakePolicy, limits: &w::Limits) -> Result<w::WakePolicy, w::Problem> {
    Ok(w::WakePolicy {
            words: wire_wake_rule(&value.words, limits)?,
            notices: wire_wake_rule(&value.notices, limits)?,
            news: wire_wake_rule(&value.news, limits)?,
            results: wire_results_wake(value.results),
            questions: value.questions,
            answers: value.answers,
            timers: value.timers,
        })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_owner_pattern_end(value: &c::PatternEnd, _limits: &w::Limits) -> Result<w::PatternEnd, w::Problem> {
    Ok(match value {
        c::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact { value: bytes::copy_of(value) }),
        c::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen { value: bytes::copy_of(value) }),
    })
}

pub(crate) fn wire_owner_pattern(value: &c::Pattern, limits: &w::Limits) -> Result<w::Pattern, w::Problem> {
    let mut list0 = List::with_capacity(value.segments.len());
    for item in &value.segments {
        list0.push(bytes::copy_of(item)).expect("reserved supplied list");
    }
    Ok(w::Pattern { segments: list0, last: wire_owner_pattern_end(&value.last, limits)? })
}

pub(crate) fn wire_owner_grant(value: &c::Grant, limits: &w::Limits) -> Result<w::Grant, w::Problem> {
    Ok(w::Grant {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
        })
}

pub(super) fn wire_owner_grant_piece(rows: &List<w::Grant>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure(limits)?).ok_or(too_large())?;
        length = length.checked_add(4).ok_or(too_large())?;
    }
    if length > limits.owned_record {
        return Err(too_large());
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(length));
    for row in rows {
        let mut item = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_owner_authority_executor(
    value: c::AuthorityExecutor,
    _limits: &w::Limits,
) -> Result<w::AuthorityExecutor, w::Problem> {
    Ok(match value {
        c::AuthorityExecutor::Job(value) => {
            w::AuthorityExecutor::Job(w::AuthorityExecutorJob { value })
        }
        c::AuthorityExecutor::Charter(value) => w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter { value }),
        c::AuthorityExecutor::Procedure(value) => w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure { value }),
        c::AuthorityExecutor::Role(value) => {
            w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value })
        }
    })
}

pub(crate) fn wire_owner_delegation(value: &c::Delegation, limits: &w::Limits) -> Result<w::Delegation, w::Problem> {
    let mut list0 = List::with_capacity(value.kinds.len());
    for item in &value.kinds {
        list0.push(wire_owner_authority_executor(*item, limits)?).expect("reserved supplied list");
    }
    Ok(w::Delegation { kinds: list0, tasks: value.tasks, depth: value.depth })
}

pub(crate) fn wire_owner_resource_scope(
    value: &c::ResourceScope,
    limits: &w::Limits,
) -> Result<w::ResourceScope, w::Problem> {
    Ok(w::ResourceScope { connector: value.connector, pattern: wire_owner_pattern(&value.pattern, limits)? })
}

fn wire_owner_resource_scope_piece(rows: &List<w::ResourceScope>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure(limits)?).ok_or(too_large())?;
        length = length.checked_add(4).ok_or(too_large())?;
    }
    if length > limits.owned_record {
        return Err(too_large());
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(length));
    for row in rows {
        let mut item = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

pub(crate) fn wire_owner_authority(value: &c::Authority, limits: &w::Limits) -> Result<w::Authority, w::Problem> {
    let mut list0 = List::with_capacity(value.grants.len());
    for item in &value.grants {
        list0.push(wire_owner_grant(item, limits)?).expect("reserved supplied list");
    }
    let opt1 = match value.deadline {
        Some(item) => Some(item.as_nanos()),
        None => None,
    };
    let mut list2 = List::with_capacity(value.note_resources.len());
    for item in &value.note_resources {
        list2.push(wire_owner_resource_scope(item, limits)?).expect("reserved supplied list");
    }
    Ok(w::Authority {
            models: super::model_sets::encode(value.models.as_ref(), limits)?,
            tools: value.tools,
            grants: wire_owner_grant_piece(&list0, limits)?,
            delegation: wire_owner_delegation(&value.delegation, limits)?,
            spend: value.spend,
            deadline: opt1,
            notes: value.notes,
            note_resources: wire_owner_resource_scope_piece(&list2, limits)?,
        })
}

pub(crate) fn wire_amendment(value: &c::Amendment, limits: &w::Limits) -> Result<w::Amendment, w::Problem> {
    let opt0 = match &value.addressed {
        Some(item) => Some(wire_address_change(item, limits)?),
        None => None,
    };
    let opt1 = match &value.spec {
        Some(item) => Some(wire_specification(item, limits)?),
        None => None,
    };
    let opt2 = match &value.wake {
        Some(item) => Some(wire_owner_wake_policy(item, limits)?),
        None => None,
    };
    let opt3 = match &value.dependencies {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt4 = match &value.authority {
        Some(item) => Some(wire_owner_authority(item, limits)?),
        None => None,
    };
    let opt5 = match &value.instructions {
        Some(item) => Some(bytes::copy_of(item.bytes())),
        None => None,
    };
    Ok(w::Amendment {
            addressed: opt0,
            spec: opt1,
            wake: opt2,
            dependencies: opt3,
            authority: opt4,
            instructions: opt5,
            tracked: match value.tracked {
                Some(v) => Some(super::operations::tracking(v, limits)?),
                None => None,
            },
            llm: super::llm::wire_choice(value.llm.as_ref(), limits)?,
        })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_seen(value: &c::SeenThrough, _limits: &w::Limits) -> Result<w::SeenThrough, w::Problem> {
    Ok(match value {
        c::SeenThrough::Results { through } => {
            w::SeenThrough::Results(w::SeenResults { through: *through })
        }
        c::SeenThrough::Conversation { task: chat, through } => w::SeenThrough::Conversation(w::SeenConversation { task: *chat, through: *through }),
    })
}

fn writable_note_scope(value: &c::NoteScope, limits: &w::Limits, path: w::Path) -> Result<w::NoteScope, w::Problem> {
    match value {
        c::NoteScope::Tasks { .. } => Err(w::Problem { path, reason: skein_codec::Reason::Malformed }),
        c::NoteScope::Deployment
        | c::NoteScope::Project(_)
        | c::NoteScope::Resource { .. }
        | c::NoteScope::Task { .. }
        | c::NoteScope::Party { .. } => wire_note_scope(value, limits),
    }
}

pub(crate) fn wire_note_scope(value: &c::NoteScope, limits: &w::Limits) -> Result<w::NoteScope, w::Problem> {
    Ok(match value {
        c::NoteScope::Tasks { project } => {
            w::NoteScope::Tasks(w::NoteScopeTasks { project: *project })
        }
        c::NoteScope::Party { party } => {
            w::NoteScope::Party(w::NoteScopeParty { party: *party })
        }
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => {
            w::NoteScope::Project(w::NoteScopeProject { value: *value })
        }
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource {
                project: *project,
                connector: *connector,
                pattern: wire_owner_pattern(pattern, limits)?,
            }),
        c::NoteScope::Task { project, task: goal } => {
            w::NoteScope::Task(w::NoteScopeTask { project: *project, task: *goal })
        }
    })
}

pub(crate) fn wire_policy_edit_role(
    value: &c::PolicyEditRole,
    limits: &w::Limits,
) -> Result<w::PolicyEditRole, w::Problem> {
    Ok(w::PolicyEditRole {
            number: value.number,
            authority: wire_owner_authority(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_policy_judge(value: c::PolicyJudge, _limits: &w::Limits) -> Result<w::PolicyJudge, w::Problem> {
    Ok(w::PolicyJudge {
            connector: value.connector,
            requirement: value.requirement,
            parameters: value.parameters,
        })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_policy_guard(value: &c::PolicyGuard, _limits: &w::Limits) -> Result<w::PolicyGuard, w::Problem> {
    Ok(match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => w::PolicyGuard::Observed(w::PolicyGuardObserved { freshness: *freshness }),
    })
}

pub(crate) fn wire_policy_requirement(
    value: &c::PolicyRequirement,
    limits: &w::Limits,
) -> Result<w::PolicyRequirement, w::Problem> {
    Ok(w::PolicyRequirement {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
            judge: wire_policy_judge(value.judge, limits)?,
            guard: wire_policy_guard(&value.guard, limits)?,
            must_be_guarded: value.must_be_guarded,
        })
}

fn wire_policy_requirement_piece(
    rows: &List<w::PolicyRequirement>,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut length = 0_u32;
    for row in rows {
        length = length.checked_add(row.measure(limits)?).ok_or(too_large())?;
        length = length.checked_add(4).ok_or(too_large())?;
    }
    if length > limits.owned_record {
        return Err(too_large());
    }
    let mut writer = Writer::new(skein_lib::length::usize_of(length));
    for row in rows {
        let mut item = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut item).expect("measured row");
        let item = item.finish();
        skein_codec::piece::write(&mut writer, &item).expect("measured piece");
    }
    Ok(writer.finish())
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_policy_permission(
    value: c::PolicyPermission,
    _limits: &w::Limits,
) -> Result<w::PolicyPermission, w::Problem> {
    Ok(w::PolicyPermission { connector: value.connector, permission: value.permission, role: value.role })
}

pub(crate) fn wire_policy_edit(value: &c::PolicyEdit, limits: &w::Limits) -> Result<w::PolicyEdit, w::Problem> {
    Ok(match value {
        c::PolicyEdit::Templates { overrides } => super::operations::templates(overrides, limits)?,
        c::PolicyEdit::RoleSpend { role, period_spend } => w::PolicyEdit::RoleSpend(w::PolicyEditRoleSpend { role: *role, period_spend: *period_spend }),
        c::PolicyEdit::ProjectSpend { period_spend } => w::PolicyEdit::ProjectSpend(w::PolicyEditProjectSpend { period_spend: *period_spend }),
        c::PolicyEdit::Role(value) => w::PolicyEdit::Role(w::PolicyEditRoleValue { value: wire_policy_edit_role(value, limits)? }),
        c::PolicyEdit::Requirements { requirements } => {
            let mut list0 = List::with_capacity((requirements).len());
            for item in requirements {
                list0.push(wire_policy_requirement(item, limits)?).expect("reserved supplied list");
            }
            w::PolicyEdit::Requirements(w::PolicyEditRequirements { requirements: wire_policy_requirement_piece(&list0, limits)? })
        }
        c::PolicyEdit::Projections { grants } => {
            let mut list0 = List::with_capacity((grants).len());
            for item in grants {
                list0.push(wire_owner_grant(item, limits)?).expect("reserved supplied list");
            }
            w::PolicyEdit::Projections(w::PolicyEditProjections { grants: wire_owner_grant_piece(&list0, limits)? })
        }
        c::PolicyEdit::Permissions { mappings } => {
            let mut list0 = List::with_capacity((mappings).len());
            for item in mappings {
                list0.push(wire_policy_permission(*item, limits)?).expect("reserved supplied list");
            }
            w::PolicyEdit::Permissions(w::PolicyEditPermissions { mappings: list0 })
        }
    })
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
pub(crate) fn wire_held_role(value: c::HeldRole, _limits: &w::Limits) -> Result<w::HeldRole, w::Problem> {
    Ok(match value {
        c::HeldRole::Owner => w::HeldRole::Owner,
        c::HeldRole::Maintainer => w::HeldRole::Maintainer,
        c::HeldRole::Member => w::HeldRole::Member,
        c::HeldRole::Observer => w::HeldRole::Observer,
        c::HeldRole::Policy { role } => {
            w::HeldRole::Policy(w::HeldRolePolicy { role })
        }
    })
}

pub(crate) fn wire_holding(value: &c::Holding, limits: &w::Limits) -> Result<w::Holding, w::Problem> {
    Ok(w::Holding { party: value.party, role: wire_held_role(value.role, limits)? })
}

#[expect(clippy::too_many_lines, reason = "one exhaustive field translation arm per client ask")]
pub fn ask(value: &c::Ask, limits: &w::Limits) -> Result<w::Ask, w::Problem> {
    Ok(match value {
        c::Ask::ReadLists { provider } => {
            w::Ask::ReadLists(w::ReadLists { provider: *provider })
        }
        c::Ask::Restore { revision, reason } => w::Ask::Restore(wire_ask_restore(*revision, reason, limits)?),
        c::Ask::MoveToNewest(value) => {
            w::Ask::MoveToNewest(w::MoveToNewest { revision: value.revision })
        }
        c::Ask::ChangeModel(value) => w::Ask::ChangeModel(super::settings::wire_change_model(value, limits)?),
        c::Ask::ChangeSettings(value) => w::Ask::ChangeSettings(super::settings::wire_change_settings(value, limits)?),
        c::Ask::Propose { project, action, reason } => super::operations::propose(*project, action, reason, limits)?,
        c::Ask::Withdraw { project, proposal } => {
            w::Ask::Withdraw(w::Withdraw { project: *project, proposal: *proposal })
        }
        c::Ask::Delegate { project, task, tasks } => super::operations::delegate(*project, *task, tasks, limits)?,
        c::Ask::Create { project, tasks } => super::operations::create(*project, tasks, limits)?,
        c::Ask::Compound { project, operations } => super::compound::encode(*project, operations, limits)?,
        c::Ask::Talk { project, task, words } => w::Ask::Talk(w::Message { project: *project, task: *task, words: bytes::copy_of(words.bytes()) }),
        c::Ask::Leave { project, task } => {
            w::Ask::Leave(w::Stop { project: *project, task: *task })
        }
        c::Ask::Message { project, task, words } => w::Ask::Message(w::Message { project: *project, task: *task, words: bytes::copy_of(words.bytes()) }),
        c::Ask::Answer { project, on } => w::Ask::Answer(w::Answer { project: *project, on: super::operations::answer(on, limits)? }),
        c::Ask::Take { project, task } => {
            w::Ask::Take(w::Take { project: *project, task: *task })
        }
        c::Ask::HandBack { project, task } => {
            w::Ask::HandBack(w::HandBack { project: *project, task: *task })
        }
        c::Ask::Decide { project, on } => w::Ask::Decide(w::Decide { project: *project, on: super::operations::decide(on, limits)? }),
        c::Ask::Lift { hold, revision, with_tasks } => w::Ask::Lift(w::Lift { hold: *hold, revision: revision.value, with_tasks: *with_tasks }),
        c::Ask::Release { project, task, revision, again } => {
            let opt0 = match again {
                Some(item) => Some(*item),
                None => None,
            };
            w::Ask::Release(w::Release {
                    project: *project,
                    task: *task,
                    revision: wire_revision(*revision, limits)?,
                    again: opt0,
                })
        }
        c::Ask::Amend { project, task, revision, change, reason } => w::Ask::Amend(Box::new(w::Amend {
                project: *project,
                task: *task,
                revision: match revision {
                    Some(value) => Some(wire_revision(*value, limits)?),
                    None => None,
                },
                change: wire_amendment(change, limits)?,
                reason: bytes::copy_of(reason.bytes()),
            })),
        c::Ask::Stop { project, task } => {
            w::Ask::Stop(w::Stop { project: *project, task: *task })
        }
        c::Ask::Conclude { project, task } => {
            w::Ask::Conclude(w::Conclude { project: *project, task: *task })
        }
        c::Ask::Cancel { project, task, reason, grace } => w::Ask::Cancel(w::Cancel { project: *project, task: *task, reason: bytes::copy_of(reason.bytes()), grace: *grace }),
        c::Ask::Move { project, task, reason } => w::Ask::MoveTask(w::Move { project: *project, task: *task, reason: bytes::copy_of(reason.bytes()) }),
        c::Ask::Seen { through } => {
            w::Ask::Seen(w::Seen { through: wire_seen(through, limits)? })
        }
        c::Ask::EditNote { project, note, scope, revision, description, body, references } => {
            let mut list0 = List::with_capacity((references).len());
            for item in references {
                list0.push(*item).expect("reserved supplied list");
            }
            w::Ask::EditNote(w::EditNote {
                    project: *project,
                    note: *note,
                    scope: writable_note_scope(scope, limits, w::paths::EDIT_NOTE_SCOPE)?,
                    revision: wire_revision(*revision, limits)?,
                    description: bytes::copy_of(description.bytes()),
                    body: bytes::copy_of(body.bytes()),
                    references: list0,
                })
        }
        c::Ask::DeleteNote { project, note, scope, revision } => w::Ask::DeleteNote(w::DeleteNote {
                project: *project,
                note: *note,
                scope: writable_note_scope(scope, limits, w::paths::DELETE_NOTE_SCOPE)?,
                revision: wire_revision(*revision, limits)?,
            }),
        c::Ask::ChangePolicy { project, edit } => w::Ask::ChangePolicy(w::ChangePolicy { project: *project, edit: wire_policy_edit(edit, limits)? }),
        c::Ask::SetRoles { project, holdings } => {
            let mut list0 = List::with_capacity((holdings).len());
            for item in holdings {
                list0.push(wire_holding(item, limits)?).expect("reserved supplied list");
            }
            w::Ask::SetRoles(w::SetRoles { project: *project, holdings: list0 })
        }
        c::Ask::MakeService { project, name, role } => w::Ask::MakeService(w::MakeService {
                project: *project,
                name: bytes::copy_of(name.bytes()),
                role: wire_held_role(*role, limits)?,
            }),
        c::Ask::AccountChange { holder, change } => w::Ask::AccountChange(w::AskAccountChange { holder: account_holder(*holder), change: account_change(change, limits)? }),
        c::Ask::SignIn { holder, provider } => w::Ask::SignIn(w::AskSignIn { holder: account_holder(*holder), provider: *provider }),
        c::Ask::SignOut { holder, source } => w::Ask::SignOut(w::AskSignOut { holder: account_holder(*holder), source: *source }),
        c::Ask::SetVisibility { project, public } => w::Ask::SetVisibility(w::SetVisibility { project: *project, public: *public }),
        c::Ask::RemoveProject { project, grace } => w::Ask::RemoveProject(w::RemoveProject { project: *project, grace: *grace }),
        c::Ask::Rename { project, name } => w::Ask::Rename(w::Rename { project: *project, name: bytes::copy_of(name.bytes()) }),
        c::Ask::MakeProject { name, policy } => w::Ask::MakeProject(w::MakeProject { name: bytes::copy_of(name.bytes()), policy: *policy }),
        c::Ask::SetPool { project, party, budget } => w::Ask::SetPool(w::SetPool { project: *project, party: *party, budget: *budget }),
    })
}

fn account_holder(value: c::AskAccountHolder) -> w::AskAccountHolder {
    match value {
        c::AskAccountHolder::Own => w::AskAccountHolder::Own,
        c::AskAccountHolder::Deployment => w::AskAccountHolder::Deployment,
    }
}

#[expect(clippy::unnecessary_wraps, reason = "translation helpers keep the fallible codec boundary")]
fn login_exit(value: c::LoginExit, _limits: &w::Limits) -> Result<w::LoginExit, w::Problem> {
    Ok(match value {
        c::LoginExit::Exited { code } => {
            w::LoginExit::Exited(w::LoginExitExited { code })
        }
        c::LoginExit::Killed { signal } => {
            w::LoginExit::Killed(w::LoginExitKilled { signal })
        }
        c::LoginExit::NotStarted => w::LoginExit::NotStarted,
    })
}
fn account_change(value: &c::AccountChange, limits: &w::Limits) -> Result<w::AccountChange, w::Problem> {
    Ok(match value {
        c::AccountChange::LoginProgram { provider, name } => {
            if !c::valid_program_name(name) {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            w::AccountChange::LoginProgram(w::AccountChangeLoginProgram { provider: *provider, name: name.clone() })
        }
        c::AccountChange::CancelLogin { number } => w::AccountChange::CancelLogin(w::AccountChangeCancelLogin { number: *number }),
        c::AccountChange::LoginEnded { number, exit } => w::AccountChange::LoginEnded(w::AccountChangeLoginEnded { number: *number, exit: login_exit(*exit, limits)? }),
        c::AccountChange::MoveOut { account } => w::AccountChange::MoveOut(w::AccountChangeMoveOut { account: *account }),
        c::AccountChange::Written { number } => w::AccountChange::Written(w::AccountChangeWritten { number: *number }),
        c::AccountChange::FinishSignIn { source, digest } => {
            w::AccountChange::FinishSignIn(w::AccountChangeFinishSignIn { source: *source, digest: *digest })
        }
        c::AccountChange::MoveIn { digest } => w::AccountChange::MoveIn(w::AccountChangeMoveIn { digest: *digest }),
        c::AccountChange::DeclareProgram { provider, join, name } => {
            if !c::valid_program_name(name) {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            w::AccountChange::DeclareProgram(w::AccountChangeDeclareProgram { provider: *provider, join: *join, name: name.clone() })
        }
        c::AccountChange::CheckProgram { source } => w::AccountChange::CheckProgram(w::AccountChangeCheckProgram { source: *source }),
        c::AccountChange::AddKey { provider, join, digest } => w::AccountChange::AddKey(w::AccountChangeAddKey { provider: *provider, join: *join, digest: *digest }),
        c::AccountChange::ReplaceKey { source, digest } => {
            w::AccountChange::ReplaceKey(w::AccountChangeReplaceKey { source: *source, digest: *digest })
        }
        c::AccountChange::RemoveKey { source } => w::AccountChange::RemoveKey(w::AccountChangeRemoveKey { source: *source }),
        c::AccountChange::Resume { source } => w::AccountChange::Resume(w::AccountChangeResume { source: *source }),
        c::AccountChange::Remove { account } => w::AccountChange::Remove(w::AccountChangeRemove { account: *account }),
        c::AccountChange::MoveAccount { account, place } => {
            w::AccountChange::MoveAccount(w::AccountChangeMoveAccount { account: *account, place: *place })
        }
        c::AccountChange::MoveSource { account, source, place } => {
            w::AccountChange::MoveSource(w::AccountChangeMoveSource { account: *account, source: *source, place: *place })
        }
        c::AccountChange::SetUses { account, uses } => account_set_uses(*account, uses.as_deref(), limits)?,
    })
}

fn account_set_uses(
    account: u32,
    uses: Option<&[c::AccountUse]>,
    limits: &w::Limits,
) -> Result<w::AccountChange, w::Problem> {
    Ok(w::AccountChange::SetUses(w::AccountChangeSetUses {
            account,
            uses: match uses {
                None => None,
                Some(uses) => {
                    if uses.len()
                        > skein_lib::length::usize_of(limits.account_change_set_uses_uses_some.min(c::ACCOUNT_USES))
                    {
                        return Err(w::Problem {
                            path: w::paths::ACCOUNT_CHANGE_SET_USES_USES,
                            reason: skein_codec::Reason::Bound,
                        });
                    }
                    let mut rows =
                        List::with_capacity(u32::try_from(uses.len()).expect("account uses bounded by the wire"));
                    for use_ in uses {
                        rows.push(match use_ {
                            c::AccountUse::Party(value) => w::AccountUse::Party(w::AccountUseParty { party: *value }),
                            c::AccountUse::Project(value) => w::AccountUse::Project(w::AccountUseProject { project: *value }),
                        })
                        .expect("reserved uses");
                    }
                    Some(rows)
                }
            },
        }))
}

pub fn wire_ask_restore(revision: u32, reason: &c::Text, _limits: &w::Limits) -> Result<w::Restore, w::Problem> {
    Ok(w::Restore { revision, reason: bytes::copy_of(reason.bytes()) })
}
pub fn restore_ask_restore(value: &w::Restore, limits: &crate::Limits) -> Result<c::Ask, w::Problem> {
    let reason = super::store::text(&value.reason, limits)?;
    Ok(c::Ask::Restore { revision: value.revision, reason })
}
