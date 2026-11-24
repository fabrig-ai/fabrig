//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::translate::too_large;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Writer, bytes};
pub(crate) fn wire_revision(value: c::Revision) -> w::Revision {
    w::Revision { value: value.value }
}

pub(crate) fn wire_person_answer(value: &c::PersonAnswer) -> w::PersonAnswer {
    match value {
        c::PersonAnswer::Choice { option, reason } => {
            w::PersonAnswer::Choice(w::PersonAnswerChoice { option: *option, reason: bytes::copy_of(reason.bytes()) })
        }
        c::PersonAnswer::Approved { state, words } => {
            let opt0 = match words {
                Some(item) => Some(bytes::copy_of(item.bytes())),
                None => None,
            };
            w::PersonAnswer::Approved(w::PersonAnswerApproved { state: bytes::copy_of(state), words: opt0 })
        }
        c::PersonAnswer::Words(value) => {
            w::PersonAnswer::Words(w::PersonAnswerWords { value: bytes::copy_of(value.bytes()) })
        }
        c::PersonAnswer::Cannot { reason } => {
            w::PersonAnswer::Cannot(w::PersonAnswerCannot { reason: bytes::copy_of(reason.bytes()) })
        }
    }
}

pub(crate) fn wire_proposer(value: &c::Proposer) -> w::Proposer {
    match value {
        c::Proposer::Task(value) => w::Proposer::Task(w::ProposerTask { value: *value }),
        c::Proposer::Party(value) => w::Proposer::Party(w::ProposerParty { value: *value }),
    }
}

pub(crate) fn wire_proposal_key(value: &c::ProposalKey) -> w::ProposalKey {
    w::ProposalKey { proposer: wire_proposer(&value.proposer), number: value.number }
}

pub(crate) fn wire_decision(value: &c::Decision) -> w::Decision {
    match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => {
            w::Decision::Rejected(w::DecisionRejected { reason: bytes::copy_of(reason.bytes()) })
        }
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => {
            let opt0 = match again {
                Some(item) => Some(*item),
                None => None,
            };
            w::Decision::Released(w::DecisionReleased { again: opt0 })
        }
        c::Decision::LeftHeld { reason } => {
            w::Decision::LeftHeld(w::DecisionLeftHeld { reason: bytes::copy_of(reason.bytes()) })
        }
    }
}

pub(crate) fn wire_person_address(value: &c::PersonAddress) -> w::PersonAddress {
    match value {
        c::PersonAddress::Party(value) => w::PersonAddress::Party(w::PersonAddressParty { value: *value }),
        c::PersonAddress::Role(value) => w::PersonAddress::Role(w::PersonAddressRole { value: *value }),
    }
}

pub(crate) fn wire_address_change(value: &c::AddressChange) -> w::AddressChange {
    w::AddressChange { revision: wire_revision(value.revision), address: wire_person_address(&value.address) }
}

pub(crate) fn wire_parameter(value: &c::Parameter) -> w::Parameter {
    match value {
        c::Parameter::Number { name, value } => w::Parameter::Number(w::ParameterNumber { name: *name, value: *value }),
        c::Parameter::Bytes { name, value } => {
            w::Parameter::Bytes(w::ParameterBytes { name: *name, value: bytes::copy_of(value.bytes()) })
        }
        c::Parameter::Resource { name, connector, resource } => {
            w::Parameter::Resource(w::ParameterResource { name: *name, connector: *connector, resource: *resource })
        }
    }
}

pub(crate) fn wire_specification(value: &c::Specification) -> w::Specification {
    let mut list0 = List::with_capacity(value.parameters.len());
    for item in &value.parameters {
        list0.push(wire_parameter(item)).expect("reserved supplied list");
    }
    let mut list1 = List::with_capacity(value.inputs.len());
    for item in &value.inputs {
        list1.push(*item).expect("reserved supplied list");
    }
    w::Specification { asks: value.asks, words: bytes::copy_of(value.words.bytes()), parameters: list0, inputs: list1 }
}

pub(crate) fn wire_wake_rule(value: &c::WakeRule) -> w::WakeRule {
    match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => w::WakeRule::Batch(w::WakeRuleBatch { count: *count, age: *age }),
        c::WakeRule::Paced { span } => w::WakeRule::Paced(w::WakeRulePaced { span: *span }),
    }
}

pub(crate) fn wire_results_wake(value: c::ResultsWake) -> w::ResultsWake {
    match value {
        c::ResultsWake::Never => w::ResultsWake::Never,
        c::ResultsWake::Each => w::ResultsWake::Each,
        c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
    }
}

pub(crate) fn wire_owner_wake_policy(value: &c::WakePolicy) -> w::WakePolicy {
    w::WakePolicy {
        words: wire_wake_rule(&value.words),
        notices: wire_wake_rule(&value.notices),
        news: wire_wake_rule(&value.news),
        results: wire_results_wake(value.results),
        questions: value.questions,
        answers: value.answers,
        timers: value.timers,
    }
}

pub(crate) fn wire_owner_pattern_end(value: &c::PatternEnd) -> w::PatternEnd {
    match value {
        c::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact { value: bytes::copy_of(value) }),
        c::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen { value: bytes::copy_of(value) }),
    }
}

pub(crate) fn wire_owner_pattern(value: &c::Pattern) -> w::Pattern {
    let mut list0 = List::with_capacity(value.segments.len());
    for item in &value.segments {
        list0.push(bytes::copy_of(item)).expect("reserved supplied list");
    }
    w::Pattern { segments: list0, last: wire_owner_pattern_end(&value.last) }
}

pub(crate) fn wire_owner_grant(value: &c::Grant) -> w::Grant {
    w::Grant { connector: value.connector, kind: value.kind, pattern: wire_owner_pattern(&value.pattern) }
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

pub(crate) fn wire_owner_authority_executor(value: c::AuthorityExecutor) -> w::AuthorityExecutor {
    match value {
        c::AuthorityExecutor::Job(value) => w::AuthorityExecutor::Job(w::AuthorityExecutorJob { value }),
        c::AuthorityExecutor::Charter(value) => w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter { value }),
        c::AuthorityExecutor::Procedure(value) => {
            w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure { value })
        }
        c::AuthorityExecutor::Role(value) => w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value }),
    }
}

pub(crate) fn wire_owner_delegation(value: &c::Delegation) -> w::Delegation {
    let mut list0 = List::with_capacity(value.kinds.len());
    for item in &value.kinds {
        list0.push(wire_owner_authority_executor(*item)).expect("reserved supplied list");
    }
    w::Delegation { kinds: list0, tasks: value.tasks, depth: value.depth }
}

pub(crate) fn wire_owner_resource_scope(value: &c::ResourceScope) -> w::ResourceScope {
    w::ResourceScope { connector: value.connector, pattern: wire_owner_pattern(&value.pattern) }
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
        list0.push(wire_owner_grant(item)).expect("reserved supplied list");
    }
    let opt1 = match value.deadline {
        Some(item) => Some(item.as_nanos()),
        None => None,
    };
    let mut list2 = List::with_capacity(value.note_resources.len());
    for item in &value.note_resources {
        list2.push(wire_owner_resource_scope(item)).expect("reserved supplied list");
    }
    Ok(w::Authority {
        models: super::model_sets::encode(value.models.as_ref())?,
        tools: value.tools,
        grants: wire_owner_grant_piece(&list0, limits)?,
        delegation: wire_owner_delegation(&value.delegation),
        spend: value.spend,
        deadline: opt1,
        notes: value.notes,
        note_resources: wire_owner_resource_scope_piece(&list2, limits)?,
    })
}

pub(crate) fn wire_amendment(value: &c::Amendment, limits: &w::Limits) -> Result<w::Amendment, w::Problem> {
    let opt0 = match &value.addressed {
        Some(item) => Some(wire_address_change(item)),
        None => None,
    };
    let opt1 = match &value.spec {
        Some(item) => Some(wire_specification(item)),
        None => None,
    };
    let opt2 = match &value.wake {
        Some(item) => Some(wire_owner_wake_policy(item)),
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
            Some(v) => Some(super::operations::tracking(v)),
            None => None,
        },
        llm: super::llm::wire_choice(value.llm.as_ref()),
    })
}

pub(crate) fn wire_seen(value: &c::SeenThrough) -> w::SeenThrough {
    match value {
        c::SeenThrough::Results { through } => w::SeenThrough::Results(w::SeenResults { through: *through }),
        c::SeenThrough::Conversation { task: chat, through } => {
            w::SeenThrough::Conversation(w::SeenConversation { task: *chat, through: *through })
        }
    }
}

fn writable_note_scope(value: &c::NoteScope, path: w::Path) -> Result<w::NoteScope, w::Problem> {
    match value {
        c::NoteScope::Tasks { .. } => Err(w::Problem { path, reason: skein_codec::Reason::Malformed }),
        c::NoteScope::Deployment
        | c::NoteScope::Project(_)
        | c::NoteScope::Resource { .. }
        | c::NoteScope::Task { .. }
        | c::NoteScope::Party { .. } => Ok(wire_note_scope(value)),
    }
}

pub(crate) fn wire_note_scope(value: &c::NoteScope) -> w::NoteScope {
    match value {
        c::NoteScope::Tasks { project } => w::NoteScope::Tasks(w::NoteScopeTasks { project: *project }),
        c::NoteScope::Party { party } => w::NoteScope::Party(w::NoteScopeParty { party: *party }),
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => w::NoteScope::Project(w::NoteScopeProject { value: *value }),
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource {
            project: *project,
            connector: *connector,
            pattern: wire_owner_pattern(pattern),
        }),
        c::NoteScope::Task { project, task: goal } => {
            w::NoteScope::Task(w::NoteScopeTask { project: *project, task: *goal })
        }
    }
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

pub(crate) fn wire_policy_judge(value: c::PolicyJudge) -> w::PolicyJudge {
    w::PolicyJudge { connector: value.connector, requirement: value.requirement, parameters: value.parameters }
}

pub(crate) fn wire_policy_guard(value: &c::PolicyGuard) -> w::PolicyGuard {
    match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => {
            w::PolicyGuard::Observed(w::PolicyGuardObserved { freshness: *freshness })
        }
    }
}

pub(crate) fn wire_policy_requirement(value: &c::PolicyRequirement) -> w::PolicyRequirement {
    w::PolicyRequirement {
        connector: value.connector,
        kind: value.kind,
        pattern: wire_owner_pattern(&value.pattern),
        judge: wire_policy_judge(value.judge),
        guard: wire_policy_guard(&value.guard),
        must_be_guarded: value.must_be_guarded,
    }
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

pub(crate) fn wire_policy_permission(value: c::PolicyPermission) -> w::PolicyPermission {
    w::PolicyPermission { connector: value.connector, permission: value.permission, role: value.role }
}

pub(crate) fn wire_policy_edit(value: &c::PolicyEdit, limits: &w::Limits) -> Result<w::PolicyEdit, w::Problem> {
    Ok(match value {
        c::PolicyEdit::Templates { overrides } => super::operations::templates(overrides),
        c::PolicyEdit::RoleSpend { role, period_spend } => {
            w::PolicyEdit::RoleSpend(w::PolicyEditRoleSpend { role: *role, period_spend: *period_spend })
        }
        c::PolicyEdit::ProjectSpend { period_spend } => {
            w::PolicyEdit::ProjectSpend(w::PolicyEditProjectSpend { period_spend: *period_spend })
        }
        c::PolicyEdit::Role(value) => {
            w::PolicyEdit::Role(w::PolicyEditRoleValue { value: wire_policy_edit_role(value, limits)? })
        }
        c::PolicyEdit::Requirements { requirements } => {
            let mut list0 = List::with_capacity((requirements).len());
            for item in requirements {
                list0.push(wire_policy_requirement(item)).expect("reserved supplied list");
            }
            w::PolicyEdit::Requirements(w::PolicyEditRequirements {
                requirements: wire_policy_requirement_piece(&list0, limits)?,
            })
        }
        c::PolicyEdit::Projections { grants } => {
            let mut list0 = List::with_capacity((grants).len());
            for item in grants {
                list0.push(wire_owner_grant(item)).expect("reserved supplied list");
            }
            w::PolicyEdit::Projections(w::PolicyEditProjections { grants: wire_owner_grant_piece(&list0, limits)? })
        }
        c::PolicyEdit::Permissions { mappings } => {
            let mut list0 = List::with_capacity((mappings).len());
            for item in mappings {
                list0.push(wire_policy_permission(*item)).expect("reserved supplied list");
            }
            w::PolicyEdit::Permissions(w::PolicyEditPermissions { mappings: list0 })
        }
    })
}

pub(crate) fn wire_held_role(value: c::HeldRole) -> w::HeldRole {
    match value {
        c::HeldRole::Owner => w::HeldRole::Owner,
        c::HeldRole::Maintainer => w::HeldRole::Maintainer,
        c::HeldRole::Member => w::HeldRole::Member,
        c::HeldRole::Observer => w::HeldRole::Observer,
        c::HeldRole::Policy { role } => w::HeldRole::Policy(w::HeldRolePolicy { role }),
    }
}

pub(crate) fn wire_holding(value: &c::Holding) -> w::Holding {
    w::Holding { party: value.party, role: wire_held_role(value.role) }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive field translation arm per client ask")]
pub fn ask(value: &c::Ask, limits: &w::Limits) -> Result<w::Ask, w::Problem> {
    Ok(match value {
        c::Ask::PauseWorker { worker, mode } => w::Ask::PauseWorker(w::PauseWorker {
            worker: *worker,
            mode: match mode {
                c::WorkerPauseMode::Resume => w::WorkerPauseMode::Resume,
                c::WorkerPauseMode::Pause => w::WorkerPauseMode::Pause,
                c::WorkerPauseMode::Drain => w::WorkerPauseMode::Drain,
            },
        }),
        c::Ask::ReadLists { provider } => w::Ask::ReadLists(w::ReadLists { provider: *provider }),
        c::Ask::Restore { revision, reason } => w::Ask::Restore(wire_ask_restore(*revision, reason)),
        c::Ask::MoveToNewest(value) => w::Ask::MoveToNewest(w::MoveToNewest { revision: value.revision }),
        c::Ask::ChangeModel(value) => w::Ask::ChangeModel(super::settings::wire_change_model(value)),
        c::Ask::ChangeSettings(value) => w::Ask::ChangeSettings(super::settings::wire_change_settings(value, limits)?),
        c::Ask::Propose { project, action, reason } => super::operations::propose(*project, action, reason, limits)?,
        c::Ask::Withdraw { project, proposal } => {
            w::Ask::Withdraw(w::Withdraw { project: *project, proposal: *proposal })
        }
        c::Ask::Delegate { project, task, tasks } => super::operations::delegate(*project, *task, tasks, limits)?,
        c::Ask::Create { project, tasks } => super::operations::create(*project, tasks, limits)?,
        c::Ask::Compound { project, operations } => super::compound::encode(*project, operations, limits)?,
        c::Ask::Talk { project, task, words } => {
            w::Ask::Talk(w::Message { project: *project, task: *task, words: bytes::copy_of(words.bytes()) })
        }
        c::Ask::Leave { project, task } => w::Ask::Leave(w::Stop { project: *project, task: *task }),
        c::Ask::Message { project, task, words } => {
            w::Ask::Message(w::Message { project: *project, task: *task, words: bytes::copy_of(words.bytes()) })
        }
        c::Ask::Answer { project, on } => {
            w::Ask::Answer(w::Answer { project: *project, on: super::operations::answer(on) })
        }
        c::Ask::Take { project, task } => w::Ask::Take(w::Take { project: *project, task: *task }),
        c::Ask::HandBack { project, task } => w::Ask::HandBack(w::HandBack { project: *project, task: *task }),
        c::Ask::Decide { project, on } => {
            w::Ask::Decide(w::Decide { project: *project, on: super::operations::decide(on) })
        }
        c::Ask::Lift { hold, revision, with_tasks } => {
            w::Ask::Lift(w::Lift { hold: *hold, revision: revision.value, with_tasks: *with_tasks })
        }
        c::Ask::Release { project, task, revision, again } => {
            let opt0 = match again {
                Some(item) => Some(*item),
                None => None,
            };
            w::Ask::Release(w::Release {
                project: *project,
                task: *task,
                revision: wire_revision(*revision),
                again: opt0,
            })
        }
        c::Ask::Amend { project, task, revision, change, reason } => w::Ask::Amend(Box::new(w::Amend {
            project: *project,
            task: *task,
            revision: match revision {
                Some(value) => Some(wire_revision(*value)),
                None => None,
            },
            change: wire_amendment(change, limits)?,
            reason: bytes::copy_of(reason.bytes()),
        })),
        c::Ask::Stop { project, task } => w::Ask::Stop(w::Stop { project: *project, task: *task }),
        c::Ask::Conclude { project, task } => w::Ask::Conclude(w::Conclude { project: *project, task: *task }),
        c::Ask::Cancel { project, task, reason, grace } => w::Ask::Cancel(w::Cancel {
            project: *project,
            task: *task,
            reason: bytes::copy_of(reason.bytes()),
            grace: *grace,
        }),
        c::Ask::Move { project, task, reason } => {
            w::Ask::MoveTask(w::Move { project: *project, task: *task, reason: bytes::copy_of(reason.bytes()) })
        }
        c::Ask::Seen { through } => w::Ask::Seen(w::Seen { through: wire_seen(through) }),
        c::Ask::EditNote { project, note, scope, revision, description, body, references } => {
            let mut list0 = List::with_capacity((references).len());
            for item in references {
                list0.push(*item).expect("reserved supplied list");
            }
            w::Ask::EditNote(w::EditNote {
                project: *project,
                note: *note,
                scope: writable_note_scope(scope, w::paths::EDIT_NOTE_SCOPE)?,
                revision: wire_revision(*revision),
                description: bytes::copy_of(description.bytes()),
                body: bytes::copy_of(body.bytes()),
                references: list0,
            })
        }
        c::Ask::DeleteNote { project, note, scope, revision } => w::Ask::DeleteNote(w::DeleteNote {
            project: *project,
            note: *note,
            scope: writable_note_scope(scope, w::paths::DELETE_NOTE_SCOPE)?,
            revision: wire_revision(*revision),
        }),
        c::Ask::ChangePolicy { project, edit } => {
            w::Ask::ChangePolicy(w::ChangePolicy { project: *project, edit: wire_policy_edit(edit, limits)? })
        }
        c::Ask::SetRoles { project, holdings } => {
            let mut list0 = List::with_capacity((holdings).len());
            for item in holdings {
                list0.push(wire_holding(item)).expect("reserved supplied list");
            }
            w::Ask::SetRoles(w::SetRoles { project: *project, holdings: list0 })
        }
        c::Ask::ReissueService { party } => w::Ask::ReissueService(w::ReissueService { party: *party }),
        c::Ask::MakeService { project, name, role } => w::Ask::MakeService(w::MakeService {
            project: *project,
            name: bytes::copy_of(name.bytes()),
            role: wire_held_role(*role),
        }),
        c::Ask::AccountChange { holder, change } => w::Ask::AccountChange(w::AskAccountChange {
            holder: account_holder(*holder),
            change: account_change(change, limits)?,
        }),
        c::Ask::SignIn { holder, provider } => {
            w::Ask::SignIn(w::AskSignIn { holder: account_holder(*holder), provider: *provider })
        }
        c::Ask::SignOut { holder, source } => {
            w::Ask::SignOut(w::AskSignOut { holder: account_holder(*holder), source: *source })
        }
        c::Ask::SetVisibility { project, public } => {
            w::Ask::SetVisibility(w::SetVisibility { project: *project, public: *public })
        }
        c::Ask::RemoveProject { project, grace } => {
            w::Ask::RemoveProject(w::RemoveProject { project: *project, grace: *grace })
        }
        c::Ask::Rename { project, name } => {
            w::Ask::Rename(w::Rename { project: *project, name: bytes::copy_of(name.bytes()) })
        }
        c::Ask::MakeProject { name, policy } => {
            w::Ask::MakeProject(w::MakeProject { name: bytes::copy_of(name.bytes()), policy: *policy })
        }
        c::Ask::SetPool { project, party, budget } => {
            w::Ask::SetPool(w::SetPool { project: *project, party: *party, budget: *budget })
        }
    })
}

fn account_holder(value: c::AskAccountHolder) -> w::AskAccountHolder {
    match value {
        c::AskAccountHolder::Own => w::AskAccountHolder::Own,
        c::AskAccountHolder::Deployment => w::AskAccountHolder::Deployment,
    }
}

fn login_exit(value: c::LoginExit) -> w::LoginExit {
    match value {
        c::LoginExit::Exited { code } => w::LoginExit::Exited(w::LoginExitExited { code }),
        c::LoginExit::Killed { signal } => w::LoginExit::Killed(w::LoginExitKilled { signal }),
        c::LoginExit::NotStarted => w::LoginExit::NotStarted,
    }
}
fn account_change(value: &c::AccountChange, limits: &w::Limits) -> Result<w::AccountChange, w::Problem> {
    Ok(match value {
        c::AccountChange::LoginProgram { provider, name } => {
            if !c::valid_program_name(name) {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            w::AccountChange::LoginProgram(w::AccountChangeLoginProgram { provider: *provider, name: name.clone() })
        }
        c::AccountChange::CancelLogin { number } => {
            w::AccountChange::CancelLogin(w::AccountChangeCancelLogin { number: *number })
        }
        c::AccountChange::LoginEnded { number, exit } => {
            w::AccountChange::LoginEnded(w::AccountChangeLoginEnded { number: *number, exit: login_exit(*exit) })
        }
        c::AccountChange::MoveOut { account } => {
            w::AccountChange::MoveOut(w::AccountChangeMoveOut { account: *account })
        }
        c::AccountChange::Written { number } => w::AccountChange::Written(w::AccountChangeWritten { number: *number }),
        c::AccountChange::FinishSignIn { source, digest } => {
            w::AccountChange::FinishSignIn(w::AccountChangeFinishSignIn { source: *source, digest: *digest })
        }
        c::AccountChange::MoveIn { digest } => w::AccountChange::MoveIn(w::AccountChangeMoveIn { digest: *digest }),
        c::AccountChange::DeclareProgram { provider, join, name } => {
            if !c::valid_program_name(name) {
                return Err(super::piece_problem(skein_codec::Reason::Malformed));
            }
            w::AccountChange::DeclareProgram(w::AccountChangeDeclareProgram {
                provider: *provider,
                join: *join,
                name: name.clone(),
            })
        }
        c::AccountChange::CheckProgram { source } => {
            w::AccountChange::CheckProgram(w::AccountChangeCheckProgram { source: *source })
        }
        c::AccountChange::AddKey { provider, join, digest } => {
            w::AccountChange::AddKey(w::AccountChangeAddKey { provider: *provider, join: *join, digest: *digest })
        }
        c::AccountChange::ReplaceKey { source, digest } => {
            w::AccountChange::ReplaceKey(w::AccountChangeReplaceKey { source: *source, digest: *digest })
        }
        c::AccountChange::RemoveKey { source } => {
            w::AccountChange::RemoveKey(w::AccountChangeRemoveKey { source: *source })
        }
        c::AccountChange::Resume { source } => w::AccountChange::Resume(w::AccountChangeResume { source: *source }),
        c::AccountChange::Remove { account } => w::AccountChange::Remove(w::AccountChangeRemove { account: *account }),
        c::AccountChange::MoveAccount { account, place } => {
            w::AccountChange::MoveAccount(w::AccountChangeMoveAccount { account: *account, place: *place })
        }
        c::AccountChange::MoveSource { account, source, place } => {
            w::AccountChange::MoveSource(w::AccountChangeMoveSource {
                account: *account,
                source: *source,
                place: *place,
            })
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
                        c::AccountUse::Project(value) => {
                            w::AccountUse::Project(w::AccountUseProject { project: *value })
                        }
                    })
                    .expect("reserved uses");
                }
                Some(rows)
            }
        },
    }))
}

#[must_use]
pub fn wire_ask_restore(revision: u32, reason: &c::Text) -> w::Restore {
    w::Restore { revision, reason: bytes::copy_of(reason.bytes()) }
}
pub fn restore_ask_restore(value: &w::Restore, limits: &crate::Limits) -> Result<c::Ask, w::Problem> {
    let reason = super::store::text(&value.reason, limits)?;
    Ok(c::Ask::Restore { revision: value.revision, reason })
}
