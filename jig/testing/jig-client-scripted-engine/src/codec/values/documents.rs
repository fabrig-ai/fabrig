//! Wire projections of documents facts (client/domain/testing.md, 4).
use super::super::{bytes, c, empty_pieces, facts, list, pieces, problem, turn_bytes, w};
use super::{
    wire_active_run, wire_activity_line, wire_authority_document, wire_bounds_create, wire_budget, wire_chip_executor,
    wire_chip_hold, wire_chip_phase, wire_closing_stage, wire_dependency, wire_document_text, wire_effect,
    wire_executor, wire_funder, wire_grant, wire_kind_implication, wire_llm_choice, wire_new_deadline, wire_note_scope,
    wire_part, wire_party, wire_phase, wire_proposal_key, wire_recurring_overlap, wire_refusal, wire_requester,
    wire_resource_line, wire_result_details, wire_revision, wire_specification_document, wire_subscriptions,
    wire_task_contract_document, wire_task_flags, wire_task_line, wire_task_link, wire_task_result, wire_task_run_wait,
    wire_turn_name, wire_waits_on, wire_wake_policy,
};
pub(in crate::codec) fn wire_field_rule(value: &c::FieldRule, limits: &w::Limits) -> Result<w::FieldRule, w::Problem> {
    w::FieldRule::new(limits, w::FieldRuleParts { name: bytes::copy_of(&value.name), max: value.max })
}

pub(in crate::codec) fn wire_offered(value: &c::Offered, limits: &w::Limits) -> Result<w::Offered, w::Problem> {
    w::Offered::new(
        limits,
        w::OfferedParts {
            act: wire_act_kind(&value.act, limits)?,
            admission: wire_admission(&value.admission, limits)?,
            effect: value.effect.as_ref().map(|item| wire_effect(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_act_kind(value: &c::ActKind, _limits: &w::Limits) -> Result<w::ActKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
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
    })
}

pub(in crate::codec) fn wire_admission(value: &c::Admission, limits: &w::Limits) -> Result<w::Admission, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Admission::Allowed => w::Admission::Allowed,
        c::Admission::Proposed { to } => w::Admission::Proposed(w::AdmissionProposed::new(
            limits,
            w::AdmissionProposedParts { to: wire_recipient(to, limits)? },
        )?),
        c::Admission::Bounded { within, beyond } => w::Admission::Bounded(Box::new(w::AdmissionBounded::new(
            limits,
            w::AdmissionBoundedParts { within: wire_bounds(within, limits)?, beyond: wire_beyond(beyond, limits)? },
        )?)),
        c::Admission::Refused(value) => w::Admission::Refused(w::AdmissionRefused::new(
            limits,
            w::AdmissionRefusedParts { value: wire_refusal(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_recipient(value: &c::Recipient, limits: &w::Limits) -> Result<w::Recipient, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Recipient::Task(value) => {
            w::Recipient::Task(w::RecipientTask::new(limits, w::RecipientTaskParts { value: *(value) })?)
        }
        c::Recipient::Party(value) => {
            w::Recipient::Party(w::RecipientParty::new(limits, w::RecipientPartyParts { value: *(value) })?)
        }
        c::Recipient::Role { project, role } => w::Recipient::Role(w::RecipientRole::new(
            limits,
            w::RecipientRoleParts { project: *(project), role: *(role) },
        )?),
        c::Recipient::Roles { project, roles } => w::Recipient::Roles(w::RecipientRoles::new(
            limits,
            w::RecipientRolesParts {
                project: *(project),
                roles: list((roles).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_bounds(value: &c::Bounds, limits: &w::Limits) -> Result<w::Bounds, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Bounds::Create(value) => w::Bounds::Create(wire_bounds_create(value, limits)?),
        c::Bounds::Amend { budget_left, grants, implications, authority, ceiling } => {
            w::Bounds::Amend(w::BoundsAmend::new(
                limits,
                w::BoundsAmendParts {
                    budget_left: *(budget_left),
                    grants: pieces(
                        (grants).iter().map(|item| wire_grant(item, limits)),
                        w::Grant::measure,
                        w::Grant::encode,
                    )?,
                    implications: list((implications).iter().map(|item| wire_kind_implication(item, limits)))?,
                    authority: wire_authority_document(authority, limits)?,
                    ceiling: wire_authority_document(ceiling, limits)?,
                },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_beyond(value: &c::Beyond, limits: &w::Limits) -> Result<w::Beyond, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Beyond::Proposed { to } => w::Beyond::Proposed(w::BeyondProposed::new(
            limits,
            w::BeyondProposedParts { to: wire_recipient(to, limits)? },
        )?),
        c::Beyond::Refused(value) => w::Beyond::Refused(w::BeyondRefused::new(
            limits,
            w::BeyondRefusedParts { value: wire_refusal(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_impact(value: &c::Impact, limits: &w::Limits) -> Result<w::Impact, w::Problem> {
    w::Impact::new(
        limits,
        w::ImpactParts {
            tasks: wire_impact_counts(&value.tasks, limits)?,
            live_runs: value.live_runs,
            proposals: value.proposals,
        },
    )
}

pub(in crate::codec) fn wire_impact_counts(
    value: &c::ImpactCounts,
    limits: &w::Limits,
) -> Result<w::ImpactCounts, w::Problem> {
    w::ImpactCounts::new(
        limits,
        w::ImpactCountsParts {
            ended: value.ended,
            waiting: value.waiting,
            running: value.running,
            held: value.held,
            closing: value.closing,
            done: value.done,
            failed: value.failed,
            cancelled: value.cancelled,
        },
    )
}

pub(in crate::codec) fn wire_recurring(value: &c::Recurring, limits: &w::Limits) -> Result<w::Recurring, w::Problem> {
    w::Recurring::new(
        limits,
        w::RecurringParts {
            chip: facts::chip(&value.chip, limits)?,
            key: value.key,
            batch: pieces(
                value.batch.iter().map(|item| wire_proposed_task(item, limits)),
                w::ProposedTask::measure,
                w::ProposedTask::encode,
            )?,
            budget: value.budget,
            overlap: wire_recurring_overlap(&value.overlap, limits)?,
            current_period: value.current_period,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            last_period: value.last_period,
            pending_period: value.pending_period.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            last_batch: list(value.last_batch.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_proposed_task(
    value: &c::ProposedTask,
    limits: &w::Limits,
) -> Result<w::ProposedTask, w::Problem> {
    w::ProposedTask::new(
        limits,
        w::ProposedTaskParts {
            deadline: wire_new_deadline(&value.deadline, limits)?,
            number: value.number,
            spec: wire_specification_document(&value.spec, limits)?,
            executor: wire_executor(&value.executor, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            authority: wire_authority_document(&value.authority, limits)?,
            budget: value.budget,
            funder: wire_funder(&value.funder, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            resources: pieces(
                value.resources.iter().map(|item| wire_part(item, limits)),
                w::Part::measure,
                w::Part::encode,
            )?,
            wake: wire_wake_policy(&value.wake, limits)?,
            priority: value.priority.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_task(value: &c::Task, limits: &w::Limits) -> Result<w::Task, w::Problem> {
    w::Task::new(
        limits,
        w::TaskParts {
            talk: value
                .talk
                .map(|talk| {
                    w::Talk::new(
                        limits,
                        w::TalkParts { party: talk.party, opened: talk.opened.as_nanos(), last: talk.last.as_nanos() },
                    )
                })
                .transpose()?,
            session: value
                .session
                .map(|session| {
                    w::Session::new(limits, w::SessionParts { first: session.first, last: session.last.as_nanos() })
                })
                .transpose()?,
            settings: value.settings.as_ref().map(|item| wire_task_settings(item, limits)).transpose()?,
            label: value.label,
            flags: wire_task_flags(&value.flags, limits)?,
            last_activity: value.last_activity.as_nanos(),
            made: value.made.as_nanos(),
            waits_on: wire_waits_on(&value.waits_on, limits)?,
            chip: facts::chip(&value.chip, limits)?,
            goal: value.goal.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            priority: value.priority.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            lineage_chips: pieces(
                value.lineage_chips.iter().map(|item| facts::chip(item, limits)),
                w::Chip::measure,
                w::Chip::encode,
            )?,
            inputs: pieces(
                value.inputs.iter().map(|item| wire_task_link(item, limits)),
                w::TaskLink::measure,
                w::TaskLink::encode,
            )?,
            project: value.project,
            requester: wire_requester(&value.requester, limits)?,
            lineage: list(value.lineage.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            spec: wire_specification_document(&value.spec, limits)?,
            spec_revision: wire_revision(&value.spec_revision, limits)?,
            person_revision: value.person_revision.as_ref().map(|item| wire_revision(item, limits)).transpose()?,
            taken_by: value.taken_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            wake: wire_wake_policy(&value.wake, limits)?,
            subscriptions: wire_subscriptions(&value.subscriptions, limits)?,
            result_contract: wire_task_contract_document(&value.result_contract, limits)?,
            dependencies: pieces(
                value.dependencies.iter().map(|item| wire_task_link(item, limits)),
                w::TaskLink::measure,
                w::TaskLink::encode,
            )?,
            references: pieces(
                value.references.iter().map(|item| wire_task_link(item, limits)),
                w::TaskLink::measure,
                w::TaskLink::encode,
            )?,
            cut: value.cut,
            resources: pieces(
                value.resources.iter().map(|item| wire_resource_line(item, limits)),
                w::ResourceLine::measure,
                w::ResourceLine::encode,
            )?,
            budget: wire_budget(&value.budget, limits)?,
            authority: wire_authority_document(&value.authority, limits)?,
            hold: value.hold.as_ref().map(|item| wire_chip_hold(item, limits)).transpose()?,
            proposals: list(value.proposals.iter().map(|item| wire_proposal_key(item, limits)))?,
            activity: value.activity,
            changed: value.changed.as_nanos(),
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
            result: value.result.as_ref().map(|item| wire_task_result(item, limits)).transpose()?,
            delegates: pieces(
                value.delegates.iter().map(|item| wire_live_delegate(item, limits)),
                w::LiveDelegate::measure,
                w::LiveDelegate::encode,
            )?,
            context: value.context.as_ref().map(|item| wire_task_context(item, limits)).transpose()?,
            llm: value.llm.as_ref().map(|item| wire_llm_choice(item, limits)).transpose()?,
            kept_account: value.kept_account.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            run_wait: value.run_wait.as_ref().map(|item| wire_task_run_wait(item, limits)).transpose()?,
            dependency_facts: list(value.dependency_facts.iter().map(|item| wire_dependency(item, limits)))?,
        },
    )
}

pub(in crate::codec) fn wire_task_settings(
    value: &c::TaskSettings,
    limits: &w::Limits,
) -> Result<w::TaskSettings, w::Problem> {
    w::TaskSettings::new(
        limits,
        w::TaskSettingsParts {
            task: value.task,
            run: value.run.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            newest: value.newest,
        },
    )
}

pub(in crate::codec) fn wire_result_document(
    value: &c::Result,
    limits: &w::Limits,
) -> Result<w::ResultDocument, w::Problem> {
    w::ResultDocument::new(
        limits,
        w::ResultDocumentParts {
            task: facts::chip(&value.task, limits)?,
            ending: facts::ending(&value.ending, limits)?,
            value: value.value.as_ref().map(|item| facts::result_value(item, limits)).transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            details: value.details.as_ref().map(|item| wire_result_details(item, limits)).transpose()?,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_live_delegate(
    value: &c::LiveDelegate,
    limits: &w::Limits,
) -> Result<w::LiveDelegate, w::Problem> {
    w::LiveDelegate::new(
        limits,
        w::LiveDelegateParts { chip: facts::chip(&value.chip, limits)?, takeover_budget: value.takeover_budget },
    )
}

pub(in crate::codec) fn wire_task_context(
    value: &c::TaskContext,
    limits: &w::Limits,
) -> Result<w::TaskContext, w::Problem> {
    w::TaskContext::new(
        limits,
        w::TaskContextParts {
            parts: empty_pieces(&value.parts)?,
            closing: value.closing.as_ref().map(|item| facts::closing(item, limits)).transpose()?,
            spend: list(value.spend.iter().map(|item| wire_spend_line(item, limits)))?,
            runtime: wire_runtime(&value.runtime, limits)?,
            impact: wire_impact(&value.impact, limits)?,
            writer_slots: empty_pieces(&value.writer_slots)?,
        },
    )
}

pub(in crate::codec) fn wire_spend_line(value: &c::SpendLine, limits: &w::Limits) -> Result<w::SpendLine, w::Problem> {
    w::SpendLine::new(
        limits,
        w::SpendLineParts {
            task: value.task,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            spent: value.spent,
        },
    )
}

pub(in crate::codec) fn wire_runtime(value: &c::Runtime, limits: &w::Limits) -> Result<w::Runtime, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Runtime::Active(value) => w::Runtime::Active(w::RuntimeActive::new(
            limits,
            w::RuntimeActiveParts { value: wire_active_run(value, limits)? },
        )?),
        c::Runtime::Waiting { .. } | c::Runtime::Procedure { .. } => return Err(problem()),
        c::Runtime::Idle => w::Runtime::Idle,
        c::Runtime::Starting => w::Runtime::Starting,
        c::Runtime::Running { attempt, since, tool } => w::Runtime::Running(w::RuntimeRunning::new(
            limits,
            w::RuntimeRunningParts {
                attempt: *(attempt),
                since: (since).as_nanos(),
                tool: (tool).as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))).transpose()?,
            },
        )?),
        c::Runtime::WaitingForWords { attempt, since } => w::Runtime::WaitingForWords(w::RuntimeWaitingForWords::new(
            limits,
            w::RuntimeWaitingForWordsParts { attempt: *(attempt), since: (since).as_nanos() },
        )?),
        c::Runtime::BackingOff { at, attempt, tries, class, reason } => {
            w::Runtime::BackingOff(w::RuntimeBackingOff::new(
                limits,
                w::RuntimeBackingOffParts {
                    at: (at).as_nanos(),
                    attempt: *(attempt),
                    tries: *(tries),
                    class: wire_failure_class(class, limits)?,
                    reason: bytes::copy_of((reason).bytes()),
                },
            )?)
        }
        c::Runtime::Person { addressed, since } => w::Runtime::Person(w::RuntimePerson::new(
            limits,
            w::RuntimePersonParts {
                addressed: wire_recipient(addressed, limits)?,
                since: (since).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::Runtime::Closing(value) => w::Runtime::Closing(w::RuntimeClosing::new(
            limits,
            w::RuntimeClosingParts { value: facts::closing(value, limits)? },
        )?),
        c::Runtime::Held { was } => w::Runtime::Held(w::RuntimeHeld::new(
            limits,
            w::RuntimeHeldParts { was: (was).as_ref().map(|item| facts::held_runtime(item, limits)).transpose()? },
        )?),
        c::Runtime::Ended => w::Runtime::Ended,
    })
}

pub(in crate::codec) fn wire_failure_class(
    value: &c::FailureClass,
    _limits: &w::Limits,
) -> Result<w::FailureClass, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::FailureClass::HostLost => w::FailureClass::HostLost,
        c::FailureClass::Temporary => w::FailureClass::Temporary,
        c::FailureClass::Permanent => w::FailureClass::Permanent,
        c::FailureClass::Stall => w::FailureClass::Stall,
    })
}

pub(in crate::codec) fn wire_question(value: &c::Question, limits: &w::Limits) -> Result<w::Question, w::Problem> {
    w::Question::new(
        limits,
        w::QuestionParts {
            task: facts::chip(&value.task, limits)?,
            from: value.from.as_ref().map(|item| wire_requester(item, limits)).transpose()?,
            message: value.message,
            words: wire_document_text(&value.words, limits)?,
            since: value.since.as_nanos(),
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
            ending: value.ending.as_ref().map(|item| facts::ending(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_result_field(
    value: &c::ResultField,
    limits: &w::Limits,
) -> Result<w::ResultField, w::Problem> {
    w::ResultField::new(
        limits,
        w::ResultFieldParts { name: bytes::copy_of(&value.name), value: bytes::copy_of(&value.value) },
    )
}

pub(in crate::codec) fn wire_needs(value: &c::Needs, limits: &w::Limits) -> Result<w::Needs, w::Problem> {
    w::Needs::new(
        limits,
        w::NeedsParts {
            questions: value.questions,
            proposals: value.proposals,
            escalations: value.escalations,
            person_tasks: value.person_tasks,
        },
    )
}

pub(in crate::codec) fn wire_archived_turn(
    value: &c::ArchivedTurn,
    limits: &w::Limits,
) -> Result<w::ArchivedTurn, w::Problem> {
    w::ArchivedTurn::new(
        limits,
        w::ArchivedTurnParts {
            task: value.task,
            name: wire_turn_name(&value.name, limits)?,
            cumulative: value.cumulative,
            read: value.read.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            at: value.at.as_nanos(),
            turn: turn_bytes(&value.turn, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_decoded_activity_line(
    value: &c::DecodedActivityLine,
    limits: &w::Limits,
) -> Result<w::DecodedActivityLine, w::Problem> {
    w::DecodedActivityLine::new(
        limits,
        w::DecodedActivityLineParts {
            source: wire_activity_line(&value.source, limits)?,
            decoded: value.decoded.as_ref().map(|item| wire_archived_turn(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_note(value: &c::Note, limits: &w::Limits) -> Result<w::Note, w::Problem> {
    w::Note::new(
        limits,
        w::NoteParts {
            number: value.number,
            revision: wire_revision(&value.revision, limits)?,
            scope: wire_note_scope(&value.scope, limits)?,
            description: wire_document_text(&value.description, limits)?,
            body: wire_document_text(&value.body, limits)?,
            writer: wire_note_author(&value.writer, limits)?,
            tasks: list(value.tasks.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            changed: value.changed.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_note_author(
    value: &c::NoteAuthor,
    limits: &w::Limits,
) -> Result<w::NoteAuthor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NoteAuthor::Party { party } => {
            w::NoteAuthor::Party(w::NoteAuthorParty::new(limits, w::NoteAuthorPartyParts { party: *(party) })?)
        }
        c::NoteAuthor::Task { task, attempt } => w::NoteAuthor::Task(w::NoteAuthorTask::new(
            limits,
            w::NoteAuthorTaskParts { task: *(task), attempt: *(attempt) },
        )?),
    })
}

pub(in crate::codec) fn wire_tasks(value: &c::Tasks, limits: &w::Limits) -> Result<w::Tasks, w::Problem> {
    w::Tasks::new(
        limits,
        w::TasksParts {
            rows: pieces(
                value.rows.iter().map(|item| wire_task_line(item, limits)),
                w::TaskLine::measure,
                w::TaskLine::encode,
            )?,
            more: value.more,
        },
    )
}

pub(in crate::codec) fn wire_chip_fields(
    value: &c::Chip,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipFields, w::Problem> {
    w::ChipFields::new(
        limits,
        w::ChipFieldsParts {
            task: value.task,
            title: wire_document_text(&value.title, limits)?,
            executor: wire_chip_executor(&value.executor, limits)?,
            phase: wire_phase(&value.phase, limits)?,
            details: facts.chip_phase(&value.details, Some(parent), limits)?,
        },
    )
}

pub(in crate::codec) fn wire_chip_phase_fields(
    value: &c::ChipPhaseFacts,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipPhaseOwnerFields, w::Problem> {
    w::ChipPhaseOwnerFields::new(
        limits,
        w::ChipPhaseOwnerFieldsParts {
            phase: wire_chip_phase(&value.phase, limits)?,
            details: facts.chip_details(&value.details, Some(parent), limits)?,
        },
    )
}

pub(in crate::codec) fn wire_closing_fields(
    value: &c::Closing,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ClosingFields, w::Problem> {
    w::ClosingFields::new(
        limits,
        w::ClosingFieldsParts {
            delegates: value.delegates,
            connectors: value
                .connectors
                .as_ref()
                .map(|item| list((item).iter().map(|item| Ok::<_, w::Problem>(*(item)))))
                .transpose()?,
            stage: value.stage.as_ref().map(|item| wire_closing_stage(item, limits)).transpose()?,
            ending: value.ending.as_ref().map(|item| facts.ending(item, Some(parent), limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_attempt_failure(
    value: &c::AttemptFailure,
    limits: &w::Limits,
) -> Result<w::AttemptFailure, w::Problem> {
    w::AttemptFailure::new(
        limits,
        w::AttemptFailureParts {
            attempt: value.attempt,
            class: wire_failure_class(&value.class, limits)?,
            reason: bytes::copy_of(value.reason.bytes()),
            at: value.at.as_nanos(),
        },
    )
}

pub(in crate::codec) fn wire_hold_reason_fields(
    value: &c::HoldReason,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::HoldReasonFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldReason::Tries { failures } => w::HoldReasonFields::Tries(w::HoldReasonTriesFields::new(
            limits,
            w::HoldReasonTriesFieldsParts {
                failures: list((failures).iter().map(|item| wire_attempt_failure(item, limits)))?,
            },
        )?),
        c::HoldReason::Stalled { since, waiting } => w::HoldReasonFields::Stalled(w::HoldReasonStalledFields::new(
            limits,
            w::HoldReasonStalledFieldsParts { since: (since).as_nanos(), waiting: bytes::copy_of((waiting).bytes()) },
        )?),
        c::HoldReason::Dependency { task, how } => w::HoldReasonFields::Dependency(w::HoldReasonDependencyFields::new(
            limits,
            w::HoldReasonDependencyFieldsParts { task: *(task), how: facts.ending(how, Some(parent), limits)? },
        )?),
        c::HoldReason::Drift { .. }
        | c::HoldReason::EffectFailed { .. }
        | c::HoldReason::EffectUncertain { .. }
        | c::HoldReason::Resource { .. }
        | c::HoldReason::Pool { .. }
        | c::HoldReason::Procedure { .. } => return Err(problem()),
        c::HoldReason::Budget { spent, limit } => w::HoldReasonFields::Budget(w::HoldReasonBudgetFields::new(
            limits,
            w::HoldReasonBudgetFieldsParts { spent: *(spent), limit: *(limit) },
        )?),
        c::HoldReason::Deadline { at } => w::HoldReasonFields::Deadline(w::HoldReasonDeadlineFields::new(
            limits,
            w::HoldReasonDeadlineFieldsParts { at: (at).as_nanos() },
        )?),
        c::HoldReason::Stopped { by, at } => w::HoldReasonFields::Stopped(w::HoldReasonStoppedFields::new(
            limits,
            w::HoldReasonStoppedFieldsParts { by: wire_party(by, limits)?, at: (at).as_nanos() },
        )?),
    })
}

pub(in crate::codec) fn wire_result_value_fields(
    value: &c::ResultValue,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ResultValueFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ResultValue::Report(value) => w::ResultValueFields::Report(w::ResultValueReportFields::new(
            limits,
            w::ResultValueReportFieldsParts { value: wire_document_text(value, limits)? },
        )?),
        c::ResultValue::OwnerVerdict { verdict, words, fields } => {
            w::ResultValueFields::OwnerVerdict(w::ResultValueOwnerVerdictFields::new(
                limits,
                w::ResultValueOwnerVerdictFieldsParts {
                    verdict: *(verdict),
                    words: wire_document_text(words, limits)?,
                    fields: list((fields).iter().map(|item| wire_result_field(item, limits)))?,
                },
            )?)
        }
        c::ResultValue::Verdict { verdict, words, fields, tasks, proposal } => {
            w::ResultValueFields::Verdict(w::ResultValueVerdictFields::new(
                limits,
                w::ResultValueVerdictFieldsParts {
                    verdict: *(verdict),
                    words: wire_document_text(words, limits)?,
                    fields: list((fields).iter().map(|item| wire_result_field(item, limits)))?,
                    tasks: list((tasks).iter().map(|item| facts.chip(item, Some(parent), limits)))?,
                    proposal: (proposal).as_ref().map(|item| wire_proposal_key(item, limits)).transpose()?,
                },
            )?)
        }
        c::ResultValue::Connector { .. } => return Err(problem()),
        c::ResultValue::Change { kind, part, words } => w::ResultValueFields::Change(w::ResultValueChangeFields::new(
            limits,
            w::ResultValueChangeFieldsParts {
                kind: *(kind),
                part: wire_part(part, limits)?,
                words: wire_document_text(words, limits)?,
            },
        )?),
        c::ResultValue::Choice { option, by, reason } => w::ResultValueFields::Choice(w::ResultValueChoiceFields::new(
            limits,
            w::ResultValueChoiceFieldsParts {
                option: *(option),
                by: wire_party(by, limits)?,
                reason: wire_document_text(reason, limits)?,
            },
        )?),
        c::ResultValue::Approved { state, words } => w::ResultValueFields::Approved(w::ResultValueApprovedFields::new(
            limits,
            w::ResultValueApprovedFieldsParts {
                state: bytes::copy_of(state),
                words: (words).as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            },
        )?),
        c::ResultValue::Failure(value) => w::ResultValueFields::Failure(w::ResultValueFailureFields::new(
            limits,
            w::ResultValueFailureFieldsParts { value: wire_document_text(value, limits)? },
        )?),
        c::ResultValue::Words(value) => w::ResultValueFields::Words(w::ResultValueWordsFields::new(
            limits,
            w::ResultValueWordsFieldsParts { value: wire_document_text(value, limits)? },
        )?),
    })
}
