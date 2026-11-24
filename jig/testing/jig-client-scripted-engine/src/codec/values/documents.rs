//! Wire projections of documents facts (client/domain/testing.md, 4).
use super::super::{bytes, c, empty_pieces, facts, list, pieces, problem, try_list, try_pieces, turn_bytes, w};
use super::{
    wire_active_run, wire_activity_line, wire_authority_document, wire_bounds_create, wire_budget, wire_chip_executor,
    wire_chip_hold, wire_chip_phase, wire_closing_stage, wire_dependency, wire_document_text, wire_effect,
    wire_executor, wire_funder, wire_grant, wire_kind_implication, wire_llm_choice, wire_new_deadline, wire_note_scope,
    wire_part, wire_party, wire_phase, wire_proposal_key, wire_recurring_overlap, wire_refusal, wire_requester,
    wire_resource_line, wire_result_details, wire_revision, wire_specification_document, wire_subscriptions,
    wire_task_contract_document, wire_task_flags, wire_task_line, wire_task_link, wire_task_result, wire_task_run_wait,
    wire_turn_name, wire_waits_on, wire_wake_policy,
};
pub(in crate::codec) fn wire_field_rule(value: &c::FieldRule) -> w::FieldRule {
    w::FieldRule { name: bytes::copy_of(&value.name), max: value.max }
}

pub(in crate::codec) fn wire_offered(value: &c::Offered, limits: &w::Limits) -> Result<w::Offered, w::Problem> {
    Ok(w::Offered {
        act: wire_act_kind(&value.act),
        admission: wire_admission(&value.admission, limits)?,
        effect: value.effect.as_ref().map(wire_effect),
    })
}

pub(in crate::codec) fn wire_act_kind(value: &c::ActKind) -> w::ActKind {
    match value {
        c::ActKind::Talk => w::ActKind::Talk,
        c::ActKind::Leave => w::ActKind::Leave,
        c::ActKind::Message => w::ActKind::Message,
        c::ActKind::Answer => w::ActKind::Answer,
        c::ActKind::Take => w::ActKind::Take,
        c::ActKind::HandBack => w::ActKind::HandBack,
        c::ActKind::Accept => w::ActKind::Accept,
        c::ActKind::Reject => w::ActKind::Reject,
        c::ActKind::PassUp => w::ActKind::PassUp,
        c::ActKind::Release => w::ActKind::Release,
        c::ActKind::LeaveHeld => w::ActKind::LeaveHeld,
        c::ActKind::Amend => w::ActKind::Amend,
        c::ActKind::Stop => w::ActKind::Stop,
        c::ActKind::Cancel => w::ActKind::Cancel,
        c::ActKind::Move => w::ActKind::MoveTask,
        c::ActKind::Reorder => w::ActKind::Reorder,
        c::ActKind::Seen => w::ActKind::Seen,
        c::ActKind::EditNote => w::ActKind::EditNote,
        c::ActKind::DeleteNote => w::ActKind::DeleteNote,
        c::ActKind::Adopt => w::ActKind::Adopt,
        c::ActKind::ChangePolicy => w::ActKind::ChangePolicy,
        c::ActKind::Rename => w::ActKind::Rename,
        c::ActKind::RemoveProject => w::ActKind::RemoveProject,
        c::ActKind::SetRoles => w::ActKind::SetRoles,
        c::ActKind::MakeService => w::ActKind::MakeService,
        c::ActKind::SetPool => w::ActKind::SetPool,
        c::ActKind::Withdraw => w::ActKind::Withdraw,
    }
}

pub(in crate::codec) fn wire_admission(value: &c::Admission, limits: &w::Limits) -> Result<w::Admission, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Admission::Allowed => w::Admission::Allowed,
        c::Admission::Proposed { to } => w::Admission::Proposed(w::AdmissionProposed { to: wire_recipient(to) }),
        c::Admission::Bounded { within, beyond } => w::Admission::Bounded(Box::new(w::AdmissionBounded {
            within: wire_bounds(within, limits)?,
            beyond: wire_beyond(beyond, limits)?,
        })),
        c::Admission::Refused(value) => {
            w::Admission::Refused(w::AdmissionRefused { value: wire_refusal(value, limits)? })
        }
    })
}

pub(in crate::codec) fn wire_recipient(value: &c::Recipient) -> w::Recipient {
    match value {
        c::Recipient::Task(value) => w::Recipient::Task(w::RecipientTask { value: *(value) }),
        c::Recipient::Party(value) => w::Recipient::Party(w::RecipientParty { value: *(value) }),
        c::Recipient::Role { project, role } => {
            w::Recipient::Role(w::RecipientRole { project: *(project), role: *(role) })
        }
        c::Recipient::Roles { project, roles } => {
            w::Recipient::Roles(w::RecipientRoles { project: *(project), roles: list((roles).iter().copied()) })
        }
    }
}

pub(in crate::codec) fn wire_bounds(value: &c::Bounds, limits: &w::Limits) -> Result<w::Bounds, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Bounds::Create(value) => w::Bounds::Create(wire_bounds_create(value, limits)?),
        c::Bounds::Amend { budget_left, grants, implications, authority, ceiling } => {
            w::Bounds::Amend(w::BoundsAmend {
                budget_left: *(budget_left),
                grants: pieces((grants).iter().map(wire_grant), limits)?,
                implications: list((implications).iter().map(wire_kind_implication)),
                authority: wire_authority_document(authority, limits)?,
                ceiling: wire_authority_document(ceiling, limits)?,
            })
        }
    })
}

pub(in crate::codec) fn wire_beyond(value: &c::Beyond, limits: &w::Limits) -> Result<w::Beyond, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Beyond::Proposed { to } => w::Beyond::Proposed(w::BeyondProposed { to: wire_recipient(to) }),
        c::Beyond::Refused(value) => w::Beyond::Refused(w::BeyondRefused { value: wire_refusal(value, limits)? }),
    })
}

pub(in crate::codec) fn wire_impact(value: &c::Impact) -> w::Impact {
    w::Impact { tasks: wire_impact_counts(&value.tasks), live_runs: value.live_runs, proposals: value.proposals }
}

pub(in crate::codec) fn wire_impact_counts(value: &c::ImpactCounts) -> w::ImpactCounts {
    w::ImpactCounts {
        ended: value.ended,
        waiting: value.waiting,
        running: value.running,
        held: value.held,
        closing: value.closing,
        done: value.done,
        failed: value.failed,
        cancelled: value.cancelled,
    }
}

pub(in crate::codec) fn wire_recurring(value: &c::Recurring, limits: &w::Limits) -> Result<w::Recurring, w::Problem> {
    Ok(w::Recurring {
        chip: facts::chip(&value.chip, limits)?,
        key: value.key,
        batch: try_pieces(value.batch.iter().map(|item| wire_proposed_task(item, limits)), limits)?,
        budget: value.budget,
        overlap: wire_recurring_overlap(&value.overlap),
        current_period: value.current_period,
        began: value.began.as_ref().map(|item| (item).as_nanos()),
        next: value.next.as_ref().map(|item| (item).as_nanos()),
        last_period: value.last_period,
        pending_period: value.pending_period,
        last_batch: list(value.last_batch.iter().copied()),
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_proposed_task(
    value: &c::ProposedTask,
    limits: &w::Limits,
) -> Result<w::ProposedTask, w::Problem> {
    Ok(w::ProposedTask {
        deadline: wire_new_deadline(&value.deadline),
        number: value.number,
        spec: wire_specification_document(&value.spec),
        executor: wire_executor(&value.executor),
        contract: wire_task_contract_document(&value.contract, limits)?,
        authority: wire_authority_document(&value.authority, limits)?,
        budget: value.budget,
        funder: wire_funder(&value.funder),
        dependencies: list(value.dependencies.iter().copied()),
        resources: pieces(value.resources.iter().map(wire_part), limits)?,
        wake: wire_wake_policy(&value.wake),
        priority: value.priority,
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_task(value: &c::Task, limits: &w::Limits) -> Result<w::Task, w::Problem> {
    Ok(w::Task {
        tasks_left: value.tasks_left,
        concluded: value.concluded.map(|v| w::Concluded { by: v.by, at: v.at.as_nanos() }),
        talk: value.talk.map(|talk| w::Talk {
            party: talk.party,
            opened: talk.opened.as_nanos(),
            last: talk.last.as_nanos(),
        }),
        session: value.session.map(|session| w::Session { first: session.first, last: session.last.as_nanos() }),
        settings: value.settings.as_ref().map(wire_task_settings),
        label: value.label,
        flags: wire_task_flags(&value.flags),
        last_activity: value.last_activity.as_nanos(),
        made: value.made.as_nanos(),
        waits_on: wire_waits_on(&value.waits_on),
        chip: facts::chip(&value.chip, limits)?,
        goal: value.goal,
        priority: value.priority,
        lineage_chips: try_pieces(value.lineage_chips.iter().map(|item| facts::chip(item, limits)), limits)?,
        inputs: try_pieces(value.inputs.iter().map(|item| wire_task_link(item, limits)), limits)?,
        project: value.project,
        requester: wire_requester(&value.requester),
        lineage: list(value.lineage.iter().copied()),
        spec: wire_specification_document(&value.spec),
        spec_revision: wire_revision(&value.spec_revision),
        person_revision: value.person_revision.as_ref().map(wire_revision),
        taken_by: value.taken_by,
        instructions: value.instructions.as_ref().map(wire_document_text),
        wake: wire_wake_policy(&value.wake),
        subscriptions: wire_subscriptions(&value.subscriptions),
        result_contract: wire_task_contract_document(&value.result_contract, limits)?,
        dependencies: try_pieces(value.dependencies.iter().map(|item| wire_task_link(item, limits)), limits)?,
        references: try_pieces(value.references.iter().map(|item| wire_task_link(item, limits)), limits)?,
        cut: value.cut,
        resources: pieces(value.resources.iter().map(wire_resource_line), limits)?,
        budget: wire_budget(&value.budget),
        authority: wire_authority_document(&value.authority, limits)?,
        hold: value.hold.as_deref().map(wire_chip_hold),
        proposals: list(value.proposals.iter().map(wire_proposal_key)),
        activity: value.activity,
        changed: value.changed.as_nanos(),
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
        result: value.result.as_ref().map(|item| wire_task_result(item, limits)).transpose()?,
        delegates: try_pieces(value.delegates.iter().map(|item| wire_live_delegate(item, limits)), limits)?,
        context: value.context.as_ref().map(|item| wire_task_context(item, limits)).transpose()?,
        llm: value.llm.as_ref().map(wire_llm_choice),
        kept_account: value.kept_account,
        run_wait: value.run_wait.as_ref().map(wire_task_run_wait),
        dependency_facts: list(value.dependency_facts.iter().map(wire_dependency)),
        unread: list(value.unread.iter().map(wire_unread)),
    })
}

fn wire_unread(value: &c::UnreadWords) -> w::UnreadWords {
    w::UnreadWords {
        message: value.message,
        by: wire_requester(&value.by),
        at: value.at.as_nanos(),
        seen: value.seen,
        head: Box::from(value.head.bytes()),
        bytes: value.bytes,
    }
}

pub(in crate::codec) fn wire_task_settings(value: &c::TaskSettings) -> w::TaskSettings {
    w::TaskSettings { task: value.task, run: value.run, newest: value.newest }
}

pub(in crate::codec) fn wire_result_document(
    value: &c::Result,
    limits: &w::Limits,
) -> Result<w::ResultDocument, w::Problem> {
    Ok(w::ResultDocument {
        task: facts::chip(&value.task, limits)?,
        ending: facts::ending(&value.ending, limits)?,
        value: value.value.as_ref().map(|item| facts::result_value(item, limits)).transpose()?,
        at: value.at.as_ref().map(|item| (item).as_nanos()),
        details: value.details.as_ref().map(|item| wire_result_details(item, limits)).transpose()?,
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
    })
}

pub(in crate::codec) fn wire_live_delegate(
    value: &c::LiveDelegate,
    limits: &w::Limits,
) -> Result<w::LiveDelegate, w::Problem> {
    Ok(w::LiveDelegate { chip: facts::chip(&value.chip, limits)?, takeover_budget: value.takeover_budget })
}

pub(in crate::codec) fn wire_task_context(
    value: &c::TaskContext,
    limits: &w::Limits,
) -> Result<w::TaskContext, w::Problem> {
    Ok(w::TaskContext {
        parts: empty_pieces(&value.parts)?,
        closing: value.closing.as_ref().map(|item| facts::closing(item, limits)).transpose()?,
        spend: list(value.spend.iter().map(wire_spend_line)),
        runtime: wire_runtime(&value.runtime, limits)?,
        impact: wire_impact(&value.impact),
        writer_slots: empty_pieces(&value.writer_slots)?,
    })
}

pub(in crate::codec) fn wire_spend_line(value: &c::SpendLine) -> w::SpendLine {
    w::SpendLine { task: value.task, attempt: value.attempt, spent: value.spent }
}

pub(in crate::codec) fn wire_runtime(value: &c::Runtime, limits: &w::Limits) -> Result<w::Runtime, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Runtime::Active(value) => w::Runtime::Active(w::RuntimeActive { value: wire_active_run(value) }),
        c::Runtime::Waiting { .. } | c::Runtime::Procedure { .. } => return Err(problem()),
        c::Runtime::Idle => w::Runtime::Idle,
        c::Runtime::Starting => w::Runtime::Starting,
        c::Runtime::Running { attempt, since, tool } => w::Runtime::Running(w::RuntimeRunning {
            attempt: *(attempt),
            since: (since).as_nanos(),
            tool: (tool).as_ref().map(|item| bytes::copy_of((item).bytes())),
        }),
        c::Runtime::WaitingForWords { attempt, since } => {
            w::Runtime::WaitingForWords(w::RuntimeWaitingForWords { attempt: *(attempt), since: (since).as_nanos() })
        }
        c::Runtime::BackingOff { at, attempt, tries, class, reason } => w::Runtime::BackingOff(w::RuntimeBackingOff {
            at: (at).as_nanos(),
            attempt: *(attempt),
            tries: *(tries),
            class: wire_failure_class(class),
            reason: bytes::copy_of((reason).bytes()),
        }),
        c::Runtime::Person { addressed, since } => w::Runtime::Person(w::RuntimePerson {
            addressed: wire_recipient(addressed),
            since: (since).as_ref().map(|item| (item).as_nanos()),
        }),
        c::Runtime::Closing(value) => w::Runtime::Closing(w::RuntimeClosing { value: facts::closing(value, limits)? }),
        c::Runtime::Held { was } => w::Runtime::Held(w::RuntimeHeld {
            was: (was).as_ref().map(|item| facts::held_runtime(item, limits)).transpose()?,
        }),
        c::Runtime::Ended => w::Runtime::Ended,
    })
}

pub(in crate::codec) fn wire_failure_class(value: &c::FailureClass) -> w::FailureClass {
    match value {
        c::FailureClass::HostLost => w::FailureClass::HostLost,
        c::FailureClass::Temporary => w::FailureClass::Temporary,
        c::FailureClass::Permanent => w::FailureClass::Permanent,
        c::FailureClass::Stall => w::FailureClass::Stall,
    }
}

pub(in crate::codec) fn wire_question(value: &c::Question, limits: &w::Limits) -> Result<w::Question, w::Problem> {
    Ok(w::Question {
        task: facts::chip(&value.task, limits)?,
        from: value.from.as_ref().map(wire_requester),
        message: value.message,
        words: wire_document_text(&value.words),
        since: value.since.as_nanos(),
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
        ending: value.ending.as_ref().map(|item| facts::ending(item, limits)).transpose()?,
    })
}

pub(in crate::codec) fn wire_result_field(value: &c::ResultField) -> w::ResultField {
    w::ResultField { name: bytes::copy_of(&value.name), value: bytes::copy_of(&value.value) }
}

pub(in crate::codec) fn wire_needs(value: &c::Needs) -> w::Needs {
    w::Needs {
        questions: value.questions,
        proposals: value.proposals,
        escalations: value.escalations,
        person_tasks: value.person_tasks,
    }
}

pub(in crate::codec) fn wire_archived_turn(value: &c::ArchivedTurn) -> Result<w::ArchivedTurn, w::Problem> {
    Ok(w::ArchivedTurn {
        task: value.task,
        name: wire_turn_name(&value.name),
        cumulative: value.cumulative,
        read: value.read,
        at: value.at.as_nanos(),
        turn: turn_bytes(&value.turn)?,
    })
}

pub(in crate::codec) fn wire_decoded_activity_line(
    value: &c::DecodedActivityLine,
) -> Result<w::DecodedActivityLine, w::Problem> {
    Ok(w::DecodedActivityLine {
        source: wire_activity_line(&value.source),
        decoded: value.decoded.as_ref().map(wire_archived_turn).transpose()?,
    })
}

pub(in crate::codec) fn wire_note(value: &c::Note, limits: &w::Limits) -> Result<w::Note, w::Problem> {
    Ok(w::Note {
        number: value.number,
        revision: wire_revision(&value.revision),
        scope: wire_note_scope(&value.scope),
        description: wire_document_text(&value.description),
        body: wire_document_text(&value.body),
        writer: wire_note_author(&value.writer),
        tasks: list(value.tasks.iter().copied()),
        changed: value.changed.as_ref().map(|item| (item).as_nanos()),
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
        cut: value.cut,
    })
}

pub(in crate::codec) fn wire_note_author(value: &c::NoteAuthor) -> w::NoteAuthor {
    match value {
        c::NoteAuthor::Party { party } => w::NoteAuthor::Party(w::NoteAuthorParty { party: *(party) }),
        c::NoteAuthor::Task { task, attempt } => {
            w::NoteAuthor::Task(w::NoteAuthorTask { task: *(task), attempt: *(attempt) })
        }
    }
}

pub(in crate::codec) fn wire_tasks(value: &c::Tasks, limits: &w::Limits) -> Result<w::Tasks, w::Problem> {
    Ok(w::Tasks {
        ended_mark: 0,
        rows: try_pieces(value.rows.iter().map(|item| wire_task_line(item, limits)), limits)?,
        more: value.more,
    })
}

pub(in crate::codec) fn wire_chip_fields(
    value: &c::Chip,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipFields, w::Problem> {
    Ok(w::ChipFields {
        task: value.task,
        title: wire_document_text(&value.title),
        executor: wire_chip_executor(&value.executor),
        phase: wire_phase(&value.phase),
        details: facts.chip_phase(&value.details, Some(parent), limits)?,
    })
}

pub(in crate::codec) fn wire_chip_phase_fields(
    value: &c::ChipPhaseFacts,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipPhaseOwnerFields, w::Problem> {
    Ok(w::ChipPhaseOwnerFields {
        phase: wire_chip_phase(&value.phase),
        details: facts.chip_details(&value.details, Some(parent), limits)?,
    })
}

pub(in crate::codec) fn wire_closing_fields(
    value: &c::Closing,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ClosingFields, w::Problem> {
    Ok(w::ClosingFields {
        delegates: value.delegates,
        connectors: value.connectors.as_ref().map(|item| list((item).iter().copied())),
        stage: value.stage.as_ref().map(wire_closing_stage),
        ending: value.ending.as_ref().map(|item| facts.ending(item, Some(parent), limits)).transpose()?,
    })
}

pub(in crate::codec) fn wire_attempt_failure(value: &c::AttemptFailure) -> w::AttemptFailure {
    w::AttemptFailure {
        attempt: value.attempt,
        class: wire_failure_class(&value.class),
        reason: bytes::copy_of(value.reason.bytes()),
        at: value.at.as_nanos(),
    }
}

pub(in crate::codec) fn wire_hold_reason_fields(
    value: &c::HoldReason,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::HoldReasonFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldReason::Tries { failures } => w::HoldReasonFields::Tries(w::HoldReasonTriesFields {
            failures: list((failures).iter().map(wire_attempt_failure)),
        }),
        c::HoldReason::Stalled { since, waiting } => w::HoldReasonFields::Stalled(w::HoldReasonStalledFields {
            since: (since).as_nanos(),
            waiting: bytes::copy_of((waiting).bytes()),
        }),
        c::HoldReason::Dependency { task, how } => w::HoldReasonFields::Dependency(w::HoldReasonDependencyFields {
            task: *(task),
            how: facts.ending(how, Some(parent), limits)?,
        }),
        c::HoldReason::Drift { .. }
        | c::HoldReason::EffectFailed { .. }
        | c::HoldReason::EffectUncertain { .. }
        | c::HoldReason::Resource { .. }
        | c::HoldReason::Pool { .. }
        | c::HoldReason::Procedure { .. } => return Err(problem()),
        c::HoldReason::Budget { spent, limit } => {
            w::HoldReasonFields::Budget(w::HoldReasonBudgetFields { spent: *(spent), limit: *(limit) })
        }
        c::HoldReason::Deadline { at } => {
            w::HoldReasonFields::Deadline(w::HoldReasonDeadlineFields { at: (at).as_nanos() })
        }
        c::HoldReason::Stopped { by, at } => {
            w::HoldReasonFields::Stopped(w::HoldReasonStoppedFields { by: wire_party(by), at: (at).as_nanos() })
        }
    })
}

pub(in crate::codec) fn wire_result_value_fields(
    value: &c::ResultValue,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ResultValueFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ResultValue::Report(value) => {
            w::ResultValueFields::Report(w::ResultValueReportFields { value: wire_document_text(value) })
        }
        c::ResultValue::OwnerVerdict { verdict, words, fields } => {
            w::ResultValueFields::OwnerVerdict(w::ResultValueOwnerVerdictFields {
                verdict: *(verdict),
                words: wire_document_text(words),
                fields: list((fields).iter().map(wire_result_field)),
            })
        }
        c::ResultValue::Verdict { verdict, words, fields, tasks, proposal } => {
            w::ResultValueFields::Verdict(w::ResultValueVerdictFields {
                verdict: *(verdict),
                words: wire_document_text(words),
                fields: list((fields).iter().map(wire_result_field)),
                tasks: try_list((tasks).iter().map(|item| facts.chip(item, Some(parent), limits)))?,
                proposal: (proposal).as_ref().map(wire_proposal_key),
            })
        }
        c::ResultValue::Connector { .. } => return Err(problem()),
        c::ResultValue::Change { kind, part, words } => w::ResultValueFields::Change(w::ResultValueChangeFields {
            kind: *(kind),
            part: wire_part(part),
            words: wire_document_text(words),
        }),
        c::ResultValue::Choice { option, by, reason } => w::ResultValueFields::Choice(w::ResultValueChoiceFields {
            option: *(option),
            by: wire_party(by),
            reason: wire_document_text(reason),
        }),
        c::ResultValue::Approved { state, words } => w::ResultValueFields::Approved(w::ResultValueApprovedFields {
            state: bytes::copy_of(state),
            words: (words).as_ref().map(wire_document_text),
        }),
        c::ResultValue::Failure(value) => {
            w::ResultValueFields::Failure(w::ResultValueFailureFields { value: wire_document_text(value) })
        }
        c::ResultValue::Words(value) => {
            w::ResultValueFields::Words(w::ResultValueWordsFields { value: wire_document_text(value) })
        }
    })
}
