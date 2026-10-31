//! Scripted documents in the generated client wire vocabulary (client/domain/testing.md, 4).
#![expect(clippy::unnecessary_wraps, reason = "field translations share a fallible signature for bounded constructors")]
#![expect(clippy::trivially_copy_pass_by_ref, reason = "field translations borrow all source facts uniformly")]
use super::{bytes, c, empty_pieces, facts, list, pieces, problem, turn_bytes, w};
pub(super) fn wire_read_value(value: &c::ReadValue, limits: &w::Limits) -> Result<w::ReadValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReadValue::Settings(value) => w::ReadValue::Settings(wire_settings_document(value, limits)?),
        c::ReadValue::Setting(value) => w::ReadValue::Setting(wire_setting_document(value, limits)?),
        c::ReadValue::Revision(value) => w::ReadValue::Revision(w::ReadRevisionAnswer::new(
            limits,
            w::ReadRevisionAnswerParts { value: wire_revision_answer(value, limits)? },
        )?),
        c::ReadValue::Revisions(value) => w::ReadValue::Revisions(wire_revisions_page(value, limits)?),
        c::ReadValue::Tasks(value) => w::ReadValue::Tasks(wire_task_page(value, limits)?),
        c::ReadValue::OwnerProject(value) => w::ReadValue::OwnerProject(wire_owner_project(value, limits)?),
        c::ReadValue::Historical(value) => w::ReadValue::Historical(Box::new(wire_owner_historical(value, limits)?)),
        c::ReadValue::Me(value) => w::ReadValue::Me(wire_me(value, limits)?),
        c::ReadValue::Turn(value) => w::ReadValue::Turn(wire_archived_turn(value, limits)?),
        c::ReadValue::Task(value) => w::ReadValue::Task(wire_task(value, limits)?),
        c::ReadValue::OwnerProposal(value) => w::ReadValue::OwnerProposal(wire_owner_proposal(value, limits)?),
        c::ReadValue::OwnerEscalation(value) => w::ReadValue::OwnerEscalation(wire_owner_escalation(value, limits)?),
        c::ReadValue::Question(value) => w::ReadValue::Question(wire_question(value, limits)?),
        c::ReadValue::OwnerPersonTask(value) => w::ReadValue::OwnerPersonTask(wire_owner_person_task(value, limits)?),
        c::ReadValue::Result(value) => w::ReadValue::Result(wire_result_document(value, limits)?),
        c::ReadValue::OwnerEnded(value) => w::ReadValue::OwnerEnded(wire_owner_ended_page(value, limits)?),
        c::ReadValue::History(value) => w::ReadValue::History(wire_history_page(value, limits)?),
        c::ReadValue::Activity(value) => w::ReadValue::Activity(wire_activity_page(value, limits)?),
        c::ReadValue::OwnerInbox(value) => w::ReadValue::OwnerInbox(wire_owner_inbox_page(value, limits)?),
        c::ReadValue::OwnerSpend(value) => w::ReadValue::OwnerSpend(wire_owner_spend_page(value, limits)?),
        c::ReadValue::Notes(value) => w::ReadValue::Notes(wire_note_page(value, limits)?),
        c::ReadValue::OwnerDeployment(value) => w::ReadValue::OwnerDeployment(wire_owner_deployment(value, limits)?),
    })
}

pub(super) fn wire_settings_document(
    value: &c::SettingsDocument,
    limits: &w::Limits,
) -> Result<w::SettingsDocument, w::Problem> {
    w::SettingsDocument::new(
        limits,
        w::SettingsDocumentParts {
            newest: value.newest,
            settings: list(value.settings.iter().map(|item| wire_setting_line(item, limits)))?,
            held: list(value.held.iter().map(|item| wire_held_revision(item, limits)))?,
            pending: list(value.pending.iter().map(|item| wire_settings_proposal(item, limits)))?,
        },
    )
}

pub(super) fn wire_setting_line(value: &c::SettingLine, limits: &w::Limits) -> Result<w::SettingLine, w::Problem> {
    w::SettingLine::new(
        limits,
        w::SettingLineParts {
            setting: wire_setting(&value.setting, limits)?,
            name: value.name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            follows: wire_setting_follows(&value.follows, limits)?,
            last: value.last,
        },
    )
}

pub(super) fn wire_setting(value: &c::Setting, limits: &w::Limits) -> Result<w::Setting, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Setting::Charter(value) => w::Setting::Charter(wire_setting_number(value, limits)?),
        c::Setting::Template(value) => w::Setting::Template(wire_setting_number(value, limits)?),
        c::Setting::Policy(value) => w::Setting::Policy(wire_setting_number(value, limits)?),
        c::Setting::Rules => w::Setting::Rules,
    })
}

pub(super) fn wire_setting_number(
    value: &c::SettingNumber,
    limits: &w::Limits,
) -> Result<w::SettingNumber, w::Problem> {
    w::SettingNumber::new(limits, w::SettingNumberParts { number: value.number })
}

pub(super) fn wire_document_text(value: &c::DocumentText, limits: &w::Limits) -> Result<w::DocumentText, w::Problem> {
    w::DocumentText::new(limits, w::DocumentTextParts { bytes: bytes::copy_of(value.bytes.bytes()), cut: value.cut })
}

pub(super) fn wire_setting_follows(
    value: &c::SettingFollows,
    _limits: &w::Limits,
) -> Result<w::SettingFollows, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingFollows::Default => w::SettingFollows::Default,
        c::SettingFollows::Change => w::SettingFollows::Change,
    })
}

pub(super) fn wire_held_revision(value: &c::HeldRevision, limits: &w::Limits) -> Result<w::HeldRevision, w::Problem> {
    w::HeldRevision::new(
        limits,
        w::HeldRevisionParts { head: wire_revision_head(&value.head, limits)?, holders: value.holders },
    )
}

pub(super) fn wire_revision_head(value: &c::RevisionHead, limits: &w::Limits) -> Result<w::RevisionHead, w::Problem> {
    w::RevisionHead::new(
        limits,
        w::RevisionHeadParts {
            number: value.number,
            against: value.against,
            at: value.at.as_nanos(),
            by: wire_revision_decider(&value.by, limits)?,
            proposal: value.proposal.as_ref().map(|item| wire_proposal_key(item, limits)).transpose()?,
            moved: value.moved,
        },
    )
}

pub(super) fn wire_revision_decider(
    value: &c::RevisionDecider,
    limits: &w::Limits,
) -> Result<w::RevisionDecider, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionDecider::FirstStart => w::RevisionDecider::FirstStart,
        c::RevisionDecider::Release => w::RevisionDecider::Release,
        c::RevisionDecider::Party(value) => w::RevisionDecider::Party(wire_revision_party(value, limits)?),
    })
}

pub(super) fn wire_revision_party(
    value: &c::RevisionParty,
    limits: &w::Limits,
) -> Result<w::RevisionParty, w::Problem> {
    w::RevisionParty::new(limits, w::RevisionPartyParts { number: value.number })
}

pub(super) fn wire_proposal_key(value: &c::ProposalKey, limits: &w::Limits) -> Result<w::ProposalKey, w::Problem> {
    w::ProposalKey::new(
        limits,
        w::ProposalKeyParts { proposer: wire_proposer(&value.proposer, limits)?, number: value.number },
    )
}

pub(super) fn wire_proposer(value: &c::Proposer, limits: &w::Limits) -> Result<w::Proposer, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Proposer::Task(value) => {
            w::Proposer::Task(w::ProposerTask::new(limits, w::ProposerTaskParts { value: *(value) })?)
        }
        c::Proposer::Party(value) => {
            w::Proposer::Party(w::ProposerParty::new(limits, w::ProposerPartyParts { value: *(value) })?)
        }
    })
}

pub(super) fn wire_settings_proposal(
    value: &c::SettingsProposal,
    limits: &w::Limits,
) -> Result<w::SettingsProposal, w::Problem> {
    w::SettingsProposal::new(
        limits,
        w::SettingsProposalParts { proposer: wire_proposer(&value.proposer, limits)?, number: value.number },
    )
}

pub(super) fn wire_setting_document(
    value: &c::SettingDocument,
    limits: &w::Limits,
) -> Result<w::SettingDocument, w::Problem> {
    w::SettingDocument::new(
        limits,
        w::SettingDocumentParts {
            setting: wire_setting(&value.setting, limits)?,
            revision: value.revision,
            value: wire_setting_value(&value.value, limits)?,
        },
    )
}

pub(super) fn wire_setting_value(value: &c::SettingValue, limits: &w::Limits) -> Result<w::SettingValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingValue::Charter(value) => w::SettingValue::Charter(Box::new(wire_charter(value, limits)?)),
        c::SettingValue::Template(value) => w::SettingValue::Template(Box::new(wire_template(value, limits)?)),
        c::SettingValue::Policy(value) => w::SettingValue::Policy(Box::new(wire_setting_policy(value, limits)?)),
        c::SettingValue::Rules(value) => w::SettingValue::Rules(Box::new(wire_rules(value, limits)?)),
    })
}

pub(super) fn wire_charter(value: &c::Charter, limits: &w::Limits) -> Result<w::Charter, w::Problem> {
    w::Charter::new(
        limits,
        w::CharterParts {
            skein_bools: [value.resume, value.inspect, value.modify, value.shell, value.agents],
            number: value.number,
            instructions: bytes::copy_of(value.instructions.bytes()),
            waiting: value.waiting,
            turns: value.turns,
            time: value.time,
            reserve_turns: value.reserve_turns,
            reserve_time: value.reserve_time,
            model: wire_setting_model(&value.model, limits)?,
            alternatives: list(value.alternatives.iter().map(|item| wire_setting_model(item, limits)))?,
            call_timeout: value.call_timeout,
            hosts: wire_setting_hosts(&value.hosts, limits)?,
            spend: value.spend,
        },
    )
}

pub(super) fn wire_setting_model(value: &c::SettingModel, limits: &w::Limits) -> Result<w::SettingModel, w::Problem> {
    w::SettingModel::new(
        limits,
        w::SettingModelParts {
            dialect: value.dialect,
            endpoint: value.endpoint,
            program: value
                .program
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
            name: bytes::copy_of(value.name.bytes()),
            window: value.window,
            output: value.output,
            input_price: value.input_price,
            cached_price: value.cached_price,
            cache_write_price: value.cache_write_price.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            output_price: value.output_price,
            price_unit: value.price_unit,
            efforts: list(value.efforts.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
            effort: value
                .effort
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
        },
    )
}

pub(super) fn wire_setting_hosts(value: &c::SettingHosts, _limits: &w::Limits) -> Result<w::SettingHosts, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingHosts::Workers => w::SettingHosts::Workers,
        c::SettingHosts::Engine => w::SettingHosts::Engine,
        c::SettingHosts::Both => w::SettingHosts::Both,
    })
}

pub(super) fn wire_template(value: &c::Template, limits: &w::Limits) -> Result<w::Template, w::Problem> {
    w::Template::new(
        limits,
        w::TemplateParts {
            number: value.number,
            name: wire_document_text(&value.name, limits)?,
            executor: wire_template_executor(&value.executor, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            shallower: value.shallower,
            budget: wire_template_budget(&value.budget, limits)?,
            deadline: wire_template_deadline(&value.deadline, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            flags: wire_task_flags(&value.flags, limits)?,
            label: value.label,
            grace: value.grace.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            llm: value.llm.as_ref().map(|item| wire_llm_choice(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_template_executor(
    value: &c::TemplateExecutor,
    limits: &w::Limits,
) -> Result<w::TemplateExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateExecutor::Charter(value) => {
            w::TemplateExecutor::Charter(w::TemplateCharter::new(limits, w::TemplateCharterParts { value: *(value) })?)
        }
        c::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    })
}

pub(super) fn wire_task_contract_document(
    value: &c::TaskContractDocument,
    limits: &w::Limits,
) -> Result<w::TaskContractDocument, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskContractDocument::Approval { state, part } => {
            w::TaskContractDocument::Approval(w::TaskContractDocumentApproval::new(
                limits,
                w::TaskContractDocumentApprovalParts {
                    state: bytes::copy_of(state),
                    part: wire_owner_part(part, limits)?,
                },
            )?)
        }
        c::TaskContractDocument::Report { words } => w::TaskContractDocument::Report(
            w::TaskContractDocumentReport::new(limits, w::TaskContractDocumentReportParts { words: *(words) })?,
        ),
        c::TaskContractDocument::Verdict { choices, cut } => {
            w::TaskContractDocument::Verdict(w::TaskContractDocumentVerdict::new(
                limits,
                w::TaskContractDocumentVerdictParts {
                    choices: pieces(
                        (choices).iter().map(|item| wire_verdict_choice(item, limits)),
                        w::VerdictChoice::measure,
                        w::VerdictChoice::encode,
                    )?,
                    cut: *(cut),
                },
            )?)
        }
        c::TaskContractDocument::Change { connector, kind, words } => {
            w::TaskContractDocument::Change(w::TaskContractDocumentChange::new(
                limits,
                w::TaskContractDocumentChangeParts { connector: *(connector), kind: *(kind), words: *(words) },
            )?)
        }
    })
}

pub(super) fn wire_owner_part(value: &c::OwnerPart, limits: &w::Limits) -> Result<w::OwnerPart, w::Problem> {
    w::OwnerPart::new(
        limits,
        w::OwnerPartParts {
            connector: value.connector,
            kind: wire_owner_part_kind(&value.kind, limits)?,
            key: wire_owner_part_key(&value.key, limits)?,
        },
    )
}

pub(super) fn wire_owner_part_kind(
    value: &c::OwnerPartKind,
    _limits: &w::Limits,
) -> Result<w::OwnerPartKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPartKind::Resource => w::OwnerPartKind::Resource,
        c::OwnerPartKind::Pool => w::OwnerPartKind::Pool,
        c::OwnerPartKind::Result => w::OwnerPartKind::Result,
        c::OwnerPartKind::Procedure => w::OwnerPartKind::Procedure,
        c::OwnerPartKind::Effect => w::OwnerPartKind::Effect,
        c::OwnerPartKind::Outbox => w::OwnerPartKind::Outbox,
        c::OwnerPartKind::Hold => w::OwnerPartKind::Hold,
    })
}

pub(super) fn wire_owner_part_key(value: &c::OwnerPartKey, limits: &w::Limits) -> Result<w::OwnerPartKey, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPartKey::Path(value) => w::OwnerPartKey::Path(w::OwnerPartKeyPath::new(
            limits,
            w::OwnerPartKeyPathParts {
                value: list((value).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            },
        )?),
        c::OwnerPartKey::Number(value) => {
            w::OwnerPartKey::Number(w::OwnerPartKeyNumber::new(limits, w::OwnerPartKeyNumberParts { value: *(value) })?)
        }
    })
}

pub(super) fn wire_verdict_choice(
    value: &c::VerdictChoice,
    limits: &w::Limits,
) -> Result<w::VerdictChoice, w::Problem> {
    w::VerdictChoice::new(
        limits,
        w::VerdictChoiceParts {
            code: value.code,
            words: value.words,
            followups: value.followups,
            fields: list(value.fields.iter().map(|item| wire_field_rule(item, limits)))?,
        },
    )
}

pub(super) fn wire_field_rule(value: &c::FieldRule, limits: &w::Limits) -> Result<w::FieldRule, w::Problem> {
    w::FieldRule::new(limits, w::FieldRuleParts { name: bytes::copy_of(&value.name), max: value.max })
}

pub(super) fn wire_template_budget(
    value: &c::TemplateBudget,
    limits: &w::Limits,
) -> Result<w::TemplateBudget, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateBudget::Amount(value) => {
            w::TemplateBudget::Amount(w::TemplateAmount::new(limits, w::TemplateAmountParts { value: *(value) })?)
        }
        c::TemplateBudget::AsAllowed { cap } => w::TemplateBudget::AsAllowed(w::TemplateAsAllowed::new(
            limits,
            w::TemplateAsAllowedParts { cap: (cap).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()? },
        )?),
        c::TemplateBudget::Given => w::TemplateBudget::Given,
    })
}

pub(super) fn wire_template_deadline(
    value: &c::TemplateDeadline,
    limits: &w::Limits,
) -> Result<w::TemplateDeadline, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateDeadline::None => w::TemplateDeadline::None,
        c::TemplateDeadline::Span(value) => {
            w::TemplateDeadline::Span(w::TemplateSpan::new(limits, w::TemplateSpanParts { value: *(value) })?)
        }
        c::TemplateDeadline::Given => w::TemplateDeadline::Given,
    })
}

pub(super) fn wire_task_flags(value: &c::TaskFlags, limits: &w::Limits) -> Result<w::TaskFlags, w::Problem> {
    w::TaskFlags::new(
        limits,
        w::TaskFlagsParts {
            conversation: value.conversation,
            listed: value.listed,
            retention: wire_retention(&value.retention, limits)?,
            ended_by: wire_ended_by(&value.ended_by, limits)?,
        },
    )
}

pub(super) fn wire_retention(value: &c::Retention, _limits: &w::Limits) -> Result<w::Retention, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Retention::Kept => w::Retention::Kept,
        c::Retention::ForgottenOnceRead => w::Retention::ForgottenOnceRead,
    })
}

pub(super) fn wire_ended_by(value: &c::EndedBy, _limits: &w::Limits) -> Result<w::EndedBy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedBy::Result => w::EndedBy::Result,
        c::EndedBy::Requester => w::EndedBy::Requester,
    })
}

pub(super) fn wire_llm_choice(value: &c::LlmChoice, limits: &w::Limits) -> Result<w::LlmChoice, w::Problem> {
    w::LlmChoice::new(
        limits,
        w::LlmChoiceParts {
            account: value.account.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            main: wire_model_effort(&value.main, limits)?,
            subagents: list(value.subagents.iter().map(|item| wire_model_effort(item, limits)))?,
        },
    )
}

pub(super) fn wire_model_effort(value: &c::ModelEffort, limits: &w::Limits) -> Result<w::ModelEffort, w::Problem> {
    w::ModelEffort::new(
        limits,
        w::ModelEffortParts {
            model: bytes::copy_of(value.model.bytes()),
            effort: value
                .effort
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
        },
    )
}

pub(super) fn wire_setting_policy(
    value: &c::SettingPolicy,
    limits: &w::Limits,
) -> Result<w::SettingPolicy, w::Problem> {
    w::SettingPolicy::new(
        limits,
        w::SettingPolicyParts {
            escalation_role: value.escalation_role.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            ceiling: wire_owner_authority(&value.ceiling, limits)?,
            projections: list(value.projections.iter().map(|item| wire_owner_grant(item, limits)))?,
            period_spend: value.period_spend,
            roles: list(value.roles.iter().map(|item| wire_policy_edit_role(item, limits)))?,
            requirements: list(value.requirements.iter().map(|item| wire_policy_requirement(item, limits)))?,
            permissions: list(value.permissions.iter().map(|item| wire_policy_permission(item, limits)))?,
            templates: list(value.templates.iter().map(|item| wire_template_override(item, limits)))?,
        },
    )
}

pub(super) fn wire_owner_authority(
    value: &c::OwnerAuthority,
    limits: &w::Limits,
) -> Result<w::OwnerAuthority, w::Problem> {
    w::OwnerAuthority::new(
        limits,
        w::OwnerAuthorityParts {
            models: value.models.as_ref().map(|item| wire_model_set(item, limits)).transpose()?,
            tools: value.tools,
            grants: pieces(
                value.grants.iter().map(|item| wire_owner_grant(item, limits)),
                w::OwnerGrant::measure,
                w::OwnerGrant::encode,
            )?,
            delegation: wire_owner_delegation(&value.delegation, limits)?,
            spend: value.spend,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            notes: value.notes,
            note_resources: pieces(
                value.note_resources.iter().map(|item| wire_owner_resource_scope(item, limits)),
                w::OwnerResourceScope::measure,
                w::OwnerResourceScope::encode,
            )?,
        },
    )
}

pub(super) fn wire_model_set(value: &c::AuthorityModels, limits: &w::Limits) -> Result<w::ModelSet, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AuthorityModels::All => w::ModelSet::All,
        c::AuthorityModels::Named(value) => w::ModelSet::Names(w::ModelSetNames::new(
            limits,
            w::ModelSetNamesParts {
                names: list((value).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            },
        )?),
    })
}

pub(super) fn wire_owner_grant(value: &c::OwnerGrant, limits: &w::Limits) -> Result<w::OwnerGrant, w::Problem> {
    w::OwnerGrant::new(
        limits,
        w::OwnerGrantParts {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
        },
    )
}

pub(super) fn wire_owner_pattern(value: &c::OwnerPattern, limits: &w::Limits) -> Result<w::OwnerPattern, w::Problem> {
    w::OwnerPattern::new(
        limits,
        w::OwnerPatternParts {
            segments: list(value.segments.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            last: wire_owner_pattern_end(&value.last, limits)?,
        },
    )
}

pub(super) fn wire_owner_pattern_end(
    value: &c::OwnerPatternEnd,
    limits: &w::Limits,
) -> Result<w::OwnerPatternEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPatternEnd::Exact(value) => w::OwnerPatternEnd::Exact(w::OwnerPatternEndExact::new(
            limits,
            w::OwnerPatternEndExactParts { value: bytes::copy_of(value) },
        )?),
        c::OwnerPatternEnd::Open(value) => w::OwnerPatternEnd::Open(w::OwnerPatternEndOpen::new(
            limits,
            w::OwnerPatternEndOpenParts { value: bytes::copy_of(value) },
        )?),
    })
}

pub(super) fn wire_owner_delegation(
    value: &c::OwnerDelegation,
    limits: &w::Limits,
) -> Result<w::OwnerDelegation, w::Problem> {
    w::OwnerDelegation::new(
        limits,
        w::OwnerDelegationParts {
            kinds: list(value.kinds.iter().map(|item| wire_owner_authority_executor(item, limits)))?,
            tasks: value.tasks,
            depth: value.depth,
        },
    )
}

pub(super) fn wire_owner_authority_executor(
    value: &c::OwnerAuthorityExecutor,
    limits: &w::Limits,
) -> Result<w::OwnerAuthorityExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAuthorityExecutor::Charter(value) => w::OwnerAuthorityExecutor::Charter(
            w::OwnerAuthorityExecutorCharter::new(limits, w::OwnerAuthorityExecutorCharterParts { value: *(value) })?,
        ),
        c::OwnerAuthorityExecutor::Procedure(value) => {
            w::OwnerAuthorityExecutor::Procedure(w::OwnerAuthorityExecutorProcedure::new(
                limits,
                w::OwnerAuthorityExecutorProcedureParts { value: *(value) },
            )?)
        }
        c::OwnerAuthorityExecutor::Role(value) => w::OwnerAuthorityExecutor::Role(w::OwnerAuthorityExecutorRole::new(
            limits,
            w::OwnerAuthorityExecutorRoleParts { value: *(value) },
        )?),
    })
}

pub(super) fn wire_owner_resource_scope(
    value: &c::OwnerResourceScope,
    limits: &w::Limits,
) -> Result<w::OwnerResourceScope, w::Problem> {
    w::OwnerResourceScope::new(
        limits,
        w::OwnerResourceScopeParts { connector: value.connector, pattern: wire_owner_pattern(&value.pattern, limits)? },
    )
}

pub(super) fn wire_policy_edit_role(
    value: &c::PolicyEditRole,
    limits: &w::Limits,
) -> Result<w::PolicyEditRole, w::Problem> {
    w::PolicyEditRole::new(
        limits,
        w::PolicyEditRoleParts {
            number: value.number,
            authority: wire_owner_authority(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        },
    )
}

pub(super) fn wire_policy_requirement(
    value: &c::PolicyRequirement,
    limits: &w::Limits,
) -> Result<w::PolicyRequirement, w::Problem> {
    w::PolicyRequirement::new(
        limits,
        w::PolicyRequirementParts {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
            judge: wire_policy_judge(&value.judge, limits)?,
            guard: wire_policy_guard(&value.guard, limits)?,
            must_be_guarded: value.must_be_guarded,
        },
    )
}

pub(super) fn wire_policy_judge(value: &c::PolicyJudge, limits: &w::Limits) -> Result<w::PolicyJudge, w::Problem> {
    w::PolicyJudge::new(
        limits,
        w::PolicyJudgeParts {
            connector: value.connector,
            requirement: value.requirement,
            parameters: value.parameters,
        },
    )
}

pub(super) fn wire_policy_guard(value: &c::PolicyGuard, limits: &w::Limits) -> Result<w::PolicyGuard, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => w::PolicyGuard::Observed(w::PolicyGuardObserved::new(
            limits,
            w::PolicyGuardObservedParts { freshness: *(freshness) },
        )?),
    })
}

pub(super) fn wire_policy_permission(
    value: &c::PolicyPermission,
    limits: &w::Limits,
) -> Result<w::PolicyPermission, w::Problem> {
    w::PolicyPermission::new(
        limits,
        w::PolicyPermissionParts { connector: value.connector, permission: value.permission, role: value.role },
    )
}

pub(super) fn wire_template_override(
    value: &c::TemplateOverride,
    limits: &w::Limits,
) -> Result<w::TemplateOverride, w::Problem> {
    w::TemplateOverride::new(
        limits,
        w::TemplateOverrideParts {
            template: value.template,
            executor: value.executor.as_ref().map(|item| wire_template_executor(item, limits)).transpose()?,
            budget: value.budget.as_ref().map(|item| wire_template_budget(item, limits)).transpose()?,
            deadline: value.deadline.as_ref().map(|item| wire_template_deadline(item, limits)).transpose()?,
            priority: value.priority.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(super) fn wire_rules(value: &c::Rules, limits: &w::Limits) -> Result<w::Rules, w::Problem> {
    w::Rules::new(
        limits,
        w::RulesParts {
            ceiling: wire_owner_authority(&value.ceiling, limits)?,
            period_spend: value.period_spend,
            minimum_run_spend: value.minimum_run_spend,
            maximum_run_spend: value.maximum_run_spend,
            implies: list(value.implies.iter().map(|item| wire_owner_kind_implication(item, limits)))?,
            requirements: list(value.requirements.iter().map(|item| wire_policy_requirement(item, limits)))?,
        },
    )
}

pub(super) fn wire_owner_kind_implication(
    value: &c::OwnerKindImplication,
    limits: &w::Limits,
) -> Result<w::OwnerKindImplication, w::Problem> {
    w::OwnerKindImplication::new(
        limits,
        w::OwnerKindImplicationParts { connector: value.connector, kind: value.kind, implies: value.implies },
    )
}

pub(super) fn wire_revision_answer(
    value: &c::RevisionAnswer,
    limits: &w::Limits,
) -> Result<w::RevisionAnswer, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionAnswer::Revision(value) => w::RevisionAnswer::Revision(wire_revision_document(value, limits)?),
        c::RevisionAnswer::Forgotten(value) => w::RevisionAnswer::Forgotten(wire_revision_forgotten(value, limits)?),
        c::RevisionAnswer::Unknown => w::RevisionAnswer::Unknown,
    })
}

pub(super) fn wire_revision_document(
    value: &c::RevisionDocument,
    limits: &w::Limits,
) -> Result<w::RevisionDocument, w::Problem> {
    w::RevisionDocument::new(
        limits,
        w::RevisionDocumentParts {
            head: wire_revision_head(&value.head, limits)?,
            items: pieces(
                value.items.iter().map(|item| wire_revision_item(item, limits)),
                w::RevisionItem::measure,
                w::RevisionItem::encode,
            )?,
        },
    )
}

pub(super) fn wire_revision_item(value: &c::RevisionItem, limits: &w::Limits) -> Result<w::RevisionItem, w::Problem> {
    w::RevisionItem::new(
        limits,
        w::RevisionItemParts {
            setting: wire_setting(&value.setting, limits)?,
            edit: wire_revision_edit(&value.edit, limits)?,
        },
    )
}

pub(super) fn wire_revision_edit(value: &c::RevisionEdit, limits: &w::Limits) -> Result<w::RevisionEdit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionEdit::Set(value) => w::RevisionEdit::Set(wire_setting_set(value, limits)?),
        c::RevisionEdit::TooLarge(value) => w::RevisionEdit::TooLarge(wire_size(value, limits)?),
        c::RevisionEdit::Removed => w::RevisionEdit::Removed,
        c::RevisionEdit::Reset => w::RevisionEdit::Reset,
    })
}

pub(super) fn wire_setting_set(value: &c::SettingSet, limits: &w::Limits) -> Result<w::SettingSet, w::Problem> {
    w::SettingSet::new(limits, w::SettingSetParts { value: wire_setting_value(&value.value, limits)? })
}

pub(super) fn wire_size(value: &c::Size, limits: &w::Limits) -> Result<w::Size, w::Problem> {
    w::Size::new(limits, w::SizeParts { bytes: value.bytes })
}

pub(super) fn wire_revision_forgotten(
    value: &c::RevisionForgotten,
    limits: &w::Limits,
) -> Result<w::RevisionForgotten, w::Problem> {
    w::RevisionForgotten::new(limits, w::RevisionForgottenParts { oldest: value.oldest })
}

pub(super) fn wire_revisions_page(
    value: &c::RevisionsPage,
    limits: &w::Limits,
) -> Result<w::RevisionsPage, w::Problem> {
    w::RevisionsPage::new(
        limits,
        w::RevisionsPageParts {
            rows: list(value.rows.iter().map(|item| wire_revision_row(item, limits)))?,
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            forgotten: value.forgotten,
        },
    )
}

pub(super) fn wire_revision_row(value: &c::RevisionRow, limits: &w::Limits) -> Result<w::RevisionRow, w::Problem> {
    w::RevisionRow::new(
        limits,
        w::RevisionRowParts {
            head: wire_revision_head(&value.head, limits)?,
            items: list(value.items.iter().map(|item| wire_revision_item_head(item, limits)))?,
            more: value.more,
        },
    )
}

pub(super) fn wire_revision_item_head(
    value: &c::RevisionItemHead,
    limits: &w::Limits,
) -> Result<w::RevisionItemHead, w::Problem> {
    w::RevisionItemHead::new(
        limits,
        w::RevisionItemHeadParts {
            setting: wire_setting(&value.setting, limits)?,
            edit: wire_revision_item_edit(&value.edit, limits)?,
        },
    )
}

pub(super) fn wire_revision_item_edit(
    value: &c::RevisionItemEdit,
    _limits: &w::Limits,
) -> Result<w::RevisionItemEdit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RevisionItemEdit::Set => w::RevisionItemEdit::Set,
        c::RevisionItemEdit::Removed => w::RevisionItemEdit::Removed,
        c::RevisionItemEdit::Reset => w::RevisionItemEdit::Reset,
    })
}

pub(super) fn wire_task_page(value: &c::TaskPage, limits: &w::Limits) -> Result<w::TaskPage, w::Problem> {
    w::TaskPage::new(
        limits,
        w::TaskPageParts {
            live: pieces(
                value.live.iter().map(|item| wire_task_line(item, limits)),
                w::TaskLine::measure,
                w::TaskLine::encode,
            )?,
            live_after: value.live_after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            ended: wire_ended_group(&value.ended, limits)?,
        },
    )
}

pub(super) fn wire_task_line(value: &c::TaskLine, limits: &w::Limits) -> Result<w::TaskLine, w::Problem> {
    w::TaskLine::new(
        limits,
        w::TaskLineParts {
            chip: facts::chip(&value.chip, limits)?,
            project: value.project,
            requester: wire_owner_requester(&value.requester, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            label: value.label,
            flags: wire_task_flags(&value.flags, limits)?,
            changed: value.changed.as_nanos(),
            last_activity: value.last_activity.as_nanos(),
            ended_at: value.ended_at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            budget: wire_owner_budget(&value.budget, limits)?,
            waits_on: wire_waits_on(&value.waits_on, limits)?,
            phases: wire_owner_phase_counts(&value.phases, limits)?,
            held_below: value.held_below,
            conversation: value.conversation.as_ref().map(|item| wire_conversation_read(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_owner_requester(
    value: &c::OwnerRequester,
    limits: &w::Limits,
) -> Result<w::OwnerRequester, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRequester::Task(value) => {
            w::OwnerRequester::Task(w::OwnerRequesterTask::new(limits, w::OwnerRequesterTaskParts { value: *(value) })?)
        }
        c::OwnerRequester::Person(value) => w::OwnerRequester::Person(w::OwnerRequesterPerson::new(
            limits,
            w::OwnerRequesterPersonParts { value: *(value) },
        )?),
        c::OwnerRequester::Deployment { project } => w::OwnerRequester::Deployment(w::OwnerRequesterDeployment::new(
            limits,
            w::OwnerRequesterDeploymentParts { project: *(project) },
        )?),
    })
}

pub(super) fn wire_owner_budget(value: &c::OwnerBudget, limits: &w::Limits) -> Result<w::OwnerBudget, w::Problem> {
    w::OwnerBudget::new(
        limits,
        w::OwnerBudgetParts {
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            left: value.left,
            funder: wire_funder(&value.funder, limits)?,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_numbers(value: &c::OwnerNumbers, limits: &w::Limits) -> Result<w::OwnerNumbers, w::Problem> {
    w::OwnerNumbers::new(
        limits,
        w::OwnerNumbersParts {
            budget: value.budget,
            spent: value.spent,
            spent_below: value.spent_below,
            reserved: value.reserved,
        },
    )
}

pub(super) fn wire_funder(value: &c::Funder, limits: &w::Limits) -> Result<w::Funder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Funder::Task(value) => w::Funder::Task(w::FunderTask::new(limits, w::FunderTaskParts { value: *(value) })?),
        c::Funder::Pool { project, person, period } => w::Funder::Pool(w::FunderPool::new(
            limits,
            w::FunderPoolParts { project: *(project), person: *(person), period: *(period) },
        )?),
        c::Funder::Recurring { project, task, period } => w::Funder::Recurring(w::FunderRecurring::new(
            limits,
            w::FunderRecurringParts { project: *(project), task: *(task), period: *(period) },
        )?),
        c::Funder::Period { project, period } => w::Funder::Period(w::FunderPeriod::new(
            limits,
            w::FunderPeriodParts { project: *(project), period: *(period) },
        )?),
    })
}

pub(super) fn wire_waits_on(value: &c::WaitsOn, limits: &w::Limits) -> Result<w::WaitsOn, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WaitsOn::Dependencies { pending, count } => w::WaitsOn::Dependencies(w::WaitsOnDependencies::new(
            limits,
            w::WaitsOnDependenciesParts {
                pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                count: *(count),
            },
        )?),
        c::WaitsOn::Party { whom, entries, task, kind } => w::WaitsOn::Party(w::WaitsOnParty::new(
            limits,
            w::WaitsOnPartyParts {
                whom: wire_whom(whom, limits)?,
                entries: *(entries),
                task: *(task),
                kind: wire_waited_for(kind, limits)?,
            },
        )?),
        c::WaitsOn::Delegates => w::WaitsOn::Delegates,
        c::WaitsOn::News => w::WaitsOn::News,
        c::WaitsOn::Nothing => w::WaitsOn::Nothing,
    })
}

pub(super) fn wire_whom(value: &c::Whom, limits: &w::Limits) -> Result<w::Whom, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Whom::Person(value) => w::Whom::Person(w::WhomPerson::new(limits, w::WhomPersonParts { value: *(value) })?),
        c::Whom::Role { project, role } => {
            w::Whom::Role(w::WhomRole::new(limits, w::WhomRoleParts { project: *(project), role: *(role) })?)
        }
    })
}

pub(super) fn wire_waited_for(value: &c::WaitedFor, _limits: &w::Limits) -> Result<w::WaitedFor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WaitedFor::Question => w::WaitedFor::Question,
        c::WaitedFor::Proposal => w::WaitedFor::Proposal,
        c::WaitedFor::Escalation => w::WaitedFor::Escalation,
        c::WaitedFor::PersonTask => w::WaitedFor::PersonTask,
    })
}

pub(super) fn wire_owner_phase_counts(
    value: &c::OwnerPhaseCounts,
    limits: &w::Limits,
) -> Result<w::OwnerPhaseCounts, w::Problem> {
    w::OwnerPhaseCounts::new(
        limits,
        w::OwnerPhaseCountsParts {
            waiting: value.waiting,
            idle: value.idle,
            starting: value.starting,
            running: value.running,
            backing_off: value.backing_off,
            closing: value.closing,
            held: value.held,
            done: value.done,
            failed: value.failed,
            cancelled: value.cancelled,
        },
    )
}

pub(super) fn wire_conversation_read(
    value: &c::ConversationRead,
    limits: &w::Limits,
) -> Result<w::ConversationRead, w::Problem> {
    w::ConversationRead::new(
        limits,
        w::ConversationReadParts { seen: value.seen, unread: wire_count(&value.unread, limits)? },
    )
}

pub(super) fn wire_count(value: &c::Count, limits: &w::Limits) -> Result<w::Count, w::Problem> {
    w::Count::new(limits, w::CountParts { value: value.value, limited: value.limited })
}

pub(super) fn wire_cursor(value: &c::Cursor, limits: &w::Limits) -> Result<w::Cursor, w::Problem> {
    w::Cursor::new(limits, w::CursorParts { value: bytes::copy_of(value.value.bytes()) })
}

pub(super) fn wire_ended_group(value: &c::EndedGroup, limits: &w::Limits) -> Result<w::EndedGroup, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedGroup::Rows { rows, after, cut } => w::EndedGroup::Rows(w::EndedRows::new(
            limits,
            w::EndedRowsParts {
                rows: pieces(
                    (rows).iter().map(|item| wire_task_line(item, limits)),
                    w::TaskLine::measure,
                    w::TaskLine::encode,
                )?,
                after: (after).as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
                cut: *(cut),
            },
        )?),
        c::EndedGroup::None => w::EndedGroup::None,
    })
}

pub(super) fn wire_owner_project(value: &c::OwnerProject, limits: &w::Limits) -> Result<w::OwnerProject, w::Problem> {
    w::OwnerProject::new(
        limits,
        w::OwnerProjectParts {
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
            number: value.number,
            name: wire_document_text(&value.name, limits)?,
            resources: pieces(
                value.resources.iter().map(|item| wire_owner_project_resource(item, limits)),
                w::OwnerProjectResource::measure,
                w::OwnerProjectResource::encode,
            )?,
            policy: wire_owner_project_policy(&value.policy, limits)?,
            members: list(value.members.iter().map(|item| wire_owner_project_member(item, limits)))?,
            current_period: value.current_period.as_ref().map(|item| wire_pool_ledger(item, limits)).transpose()?,
            pools: list(value.pools.iter().map(|item| wire_pool_ledger(item, limits)))?,
            configured_period: value.configured_period,
            configured_pools: list(value.configured_pools.iter().map(|item| wire_pool_ledger(item, limits)))?,
            configured_pools_cut: value.configured_pools_cut,
            spend: pieces(
                value.spend.iter().map(|item| wire_owner_project_spend(item, limits)),
                w::OwnerProjectSpend::measure,
                w::OwnerProjectSpend::encode,
            )?,
            recurring: pieces(
                value.recurring.iter().map(|item| wire_recurring(item, limits)),
                w::Recurring::measure,
                w::Recurring::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_offered(value: &c::Offered, limits: &w::Limits) -> Result<w::Offered, w::Problem> {
    w::Offered::new(
        limits,
        w::OfferedParts {
            act: wire_act_kind(&value.act, limits)?,
            admission: wire_admission(&value.admission, limits)?,
            effect: value.effect.as_ref().map(|item| wire_effect(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_act_kind(value: &c::ActKind, _limits: &w::Limits) -> Result<w::ActKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
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

pub(super) fn wire_admission(value: &c::Admission, limits: &w::Limits) -> Result<w::Admission, w::Problem> {
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

pub(super) fn wire_recipient(value: &c::Recipient, limits: &w::Limits) -> Result<w::Recipient, w::Problem> {
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

pub(super) fn wire_bounds(value: &c::Bounds, limits: &w::Limits) -> Result<w::Bounds, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Bounds::Create(value) => w::Bounds::Create(wire_bounds_create(value, limits)?),
        c::Bounds::Amend { budget_left, grants, implications, authority, ceiling } => {
            w::Bounds::Amend(w::BoundsAmend::new(
                limits,
                w::BoundsAmendParts {
                    budget_left: *(budget_left),
                    grants: pieces(
                        (grants).iter().map(|item| wire_owner_grant(item, limits)),
                        w::OwnerGrant::measure,
                        w::OwnerGrant::encode,
                    )?,
                    implications: list((implications).iter().map(|item| wire_owner_kind_implication(item, limits)))?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    ceiling: wire_owner_authority_document(ceiling, limits)?,
                },
            )?)
        }
    })
}

pub(super) fn wire_bounds_create(value: &c::CreateBounds, limits: &w::Limits) -> Result<w::BoundsCreate, w::Problem> {
    w::BoundsCreate::new(
        limits,
        w::BoundsCreateParts {
            authority: wire_owner_authority_document(&value.authority, limits)?,
            pool_left: value.pool_left,
            charters: list(value.charters.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            procedures: list(value.procedures.iter().map(|item| wire_create_procedure(item, limits)))?,
            latest: value.latest.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            budget: value.budget,
            maximum_budget: value.maximum_budget,
            maximum_deadline: value
                .maximum_deadline
                .as_ref()
                .map(|item| Ok::<_, w::Problem>((item).as_nanos()))
                .transpose()?,
        },
    )
}

pub(super) fn wire_owner_authority_document(
    value: &c::OwnerAuthorityDocument,
    limits: &w::Limits,
) -> Result<w::OwnerAuthorityDocument, w::Problem> {
    w::OwnerAuthorityDocument::new(
        limits,
        w::OwnerAuthorityDocumentParts { authority: wire_owner_authority(&value.authority, limits)?, cut: value.cut },
    )
}

pub(super) fn wire_create_procedure(
    value: &c::CreateProcedure,
    limits: &w::Limits,
) -> Result<w::CreateProcedure, w::Problem> {
    w::CreateProcedure::new(limits, w::CreateProcedureParts { connector: value.connector, code: value.code })
}

pub(super) fn wire_beyond(value: &c::Beyond, limits: &w::Limits) -> Result<w::Beyond, w::Problem> {
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

pub(super) fn wire_refusal(value: &c::Refusal, limits: &w::Limits) -> Result<w::Refusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Refusal::SignInRedirect { why } => w::Refusal::SignInRedirect(w::RefusalSignInRedirect::new(
            limits,
            w::RefusalSignInRedirectParts { why: wire_sign_in_redirect_why(why, limits)? },
        )?),
        c::Refusal::Stale(value) => w::Refusal::Stale(wire_refusal_stale(value, limits)?),
        c::Refusal::Full(value) => w::Refusal::Full(wire_refusal_full(value, limits)?),
        c::Refusal::AccountMove { why } => w::Refusal::AccountMove(w::RefusalAccountMove::new(
            limits,
            w::RefusalAccountMoveParts { why: wire_account_move_why(why, limits)? },
        )?),
        c::Refusal::Circle { tasks, cut } => w::Refusal::Circle(w::RefusalCircle::new(
            limits,
            w::RefusalCircleParts {
                tasks: list((tasks).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                cut: *(cut),
            },
        )?),
        c::Refusal::Dependency { dependency, how } => w::Refusal::Dependency(w::RefusalDependency::new(
            limits,
            w::RefusalDependencyParts { dependency: *(dependency), how: wire_owner_task_status(how, limits)? },
        )?),
        c::Refusal::Kept { pending } => w::Refusal::Kept(w::RefusalKept::new(
            limits,
            w::RefusalKeptParts { pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))? },
        )?),
        c::Refusal::Account { account, why } => w::Refusal::Account(w::RefusalAccount::new(
            limits,
            w::RefusalAccountParts { account: *(account), why: wire_account_why(why, limits)? },
        )?),
        c::Refusal::NoEscalationHolder => w::Refusal::NoEscalationHolder,
        c::Refusal::Provider => w::Refusal::Provider,
        c::Refusal::Malformed { at, why } => w::Refusal::Malformed(w::RefusalMalformed::new(
            limits,
            w::RefusalMalformedParts { at: *(at), why: wire_malformed(why, limits)? },
        )?),
        c::Refusal::Role { held } => w::Refusal::Role(w::RefusalRole::new(
            limits,
            w::RefusalRoleParts { held: (held).as_ref().map(|item| wire_held_role(item, limits)).transpose()? },
        )?),
        c::Refusal::Taken { by } => {
            w::Refusal::Taken(w::RefusalTaken::new(limits, w::RefusalTakenParts { by: *(by) })?)
        }
        c::Refusal::NotWaiting => w::Refusal::NotWaiting,
        c::Refusal::NoFurther => w::Refusal::NoFurther,
        c::Refusal::Authority { lacking, proposable } => w::Refusal::Authority(w::RefusalAuthority::new(
            limits,
            w::RefusalAuthorityParts {
                lacking: wire_lacking(lacking, limits)?,
                proposable: (proposable).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::Refusal::Limit { which } => {
            w::Refusal::Limit(w::RefusalLimit::new(limits, w::RefusalLimitParts { which: wire_limit(which, limits)? })?)
        }
        c::Refusal::Moved { now, what } => w::Refusal::Moved(w::RefusalMoved::new(
            limits,
            w::RefusalMovedParts { now: wire_revision(now, limits)?, what: wire_moved(what, limits)? },
        )?),
        c::Refusal::Ended { how } => {
            w::Refusal::Ended(w::RefusalEnded::new(limits, w::RefusalEndedParts { how: facts::ending(how, limits)? })?)
        }
        c::Refusal::Unknown => w::Refusal::Unknown,
        c::Refusal::NameTaken => w::Refusal::NameTaken,
        c::Refusal::NotOffered => w::Refusal::NotOffered,
        c::Refusal::KeyConflict => w::Refusal::KeyConflict,
        c::Refusal::Llm { model, why } => w::Refusal::Llm(w::RefusalLlm::new(
            limits,
            w::RefusalLlmParts { model: bytes::copy_of((model).bytes()), why: wire_llm_why(why, limits)? },
        )?),
    })
}

pub(super) fn wire_sign_in_redirect_why(
    value: &c::SignInRedirectWhy,
    _limits: &w::Limits,
) -> Result<w::SignInRedirectWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SignInRedirectWhy::NotWaiting => w::SignInRedirectWhy::NotWaiting,
        c::SignInRedirectWhy::Address => w::SignInRedirectWhy::Address,
        c::SignInRedirectWhy::State => w::SignInRedirectWhy::State,
        c::SignInRedirectWhy::TooMany => w::SignInRedirectWhy::TooMany,
    })
}

pub(super) fn wire_refusal_stale(value: &c::RefusalStale, limits: &w::Limits) -> Result<w::RefusalStale, w::Problem> {
    w::RefusalStale::new(limits, w::RefusalStaleParts { newest: value.newest })
}

pub(super) fn wire_refusal_full(value: &c::RefusalFull, limits: &w::Limits) -> Result<w::RefusalFull, w::Problem> {
    w::RefusalFull::new(
        limits,
        w::RefusalFullParts {
            revision: value.revision,
            tasks: list(value.tasks.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            more: value.more,
        },
    )
}

pub(super) fn wire_account_move_why(
    value: &c::AccountMoveWhy,
    _limits: &w::Limits,
) -> Result<w::AccountMoveWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountMoveWhy::NotSignedIn => w::AccountMoveWhy::NotSignedIn,
        c::AccountMoveWhy::NotOurs => w::AccountMoveWhy::NotOurs,
        c::AccountMoveWhy::Held => w::AccountMoveWhy::Held,
        c::AccountMoveWhy::Cancelled => w::AccountMoveWhy::Cancelled,
        c::AccountMoveWhy::Moving => w::AccountMoveWhy::Moving,
    })
}

pub(super) fn wire_owner_task_status(
    value: &c::OwnerTaskStatus,
    _limits: &w::Limits,
) -> Result<w::OwnerTaskStatus, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskStatus::Done => w::OwnerTaskStatus::Done,
        c::OwnerTaskStatus::Failed => w::OwnerTaskStatus::Failed,
        c::OwnerTaskStatus::Cancelled => w::OwnerTaskStatus::Cancelled,
    })
}

pub(super) fn wire_account_why(value: &c::AccountWhy, _limits: &w::Limits) -> Result<w::AccountWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountWhy::Unknown => w::AccountWhy::Unknown,
        c::AccountWhy::Provider => w::AccountWhy::Provider,
        c::AccountWhy::Uses => w::AccountWhy::Uses,
        c::AccountWhy::Limited => w::AccountWhy::Limited,
    })
}

pub(super) fn wire_malformed(value: &c::Malformed, limits: &w::Limits) -> Result<w::Malformed, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Malformed::Running => w::Malformed::Running,
        c::Malformed::Settings(value) => w::Malformed::Settings(wire_malformed_settings(value, limits)?),
        c::Malformed::SettingsUnchanged => w::Malformed::SettingsUnchanged,
        c::Malformed::ProjectMismatch => w::Malformed::ProjectMismatch,
        c::Malformed::TwoAsAllowed => w::Malformed::TwoAsAllowed,
        c::Malformed::ConversationNotAgent => w::Malformed::ConversationNotAgent,
        c::Malformed::Cycle => w::Malformed::Cycle,
        c::Malformed::ProposedAsAllowed => w::Malformed::ProposedAsAllowed,
        c::Malformed::EndedBy => w::Malformed::EndedBy,
        c::Malformed::Name(value) => w::Malformed::Name(w::MalformedName::new(
            limits,
            w::MalformedNameParts { rule: wire_name_rule(value, limits)? },
        )?),
    })
}

pub(super) fn wire_malformed_settings(
    value: &c::MalformedSettings,
    limits: &w::Limits,
) -> Result<w::MalformedSettings, w::Problem> {
    w::MalformedSettings::new(
        limits,
        w::MalformedSettingsParts {
            setting: wire_setting(&value.setting, limits)?,
            fault: wire_setting_fault(&value.fault, limits)?,
        },
    )
}

pub(super) fn wire_setting_fault(value: &c::SettingFault, _limits: &w::Limits) -> Result<w::SettingFault, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingFault::Shape => w::SettingFault::Shape,
        c::SettingFault::Count => w::SettingFault::Count,
        c::SettingFault::Reference => w::SettingFault::Reference,
        c::SettingFault::Rules => w::SettingFault::Rules,
        c::SettingFault::Budget => w::SettingFault::Budget,
        c::SettingFault::Grace => w::SettingFault::Grace,
        c::SettingFault::Owner => w::SettingFault::Owner,
    })
}

pub(super) fn wire_name_rule(value: &c::NameRule, _limits: &w::Limits) -> Result<w::NameRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NameRule::Empty => w::NameRule::Empty,
        c::NameRule::Long => w::NameRule::Long,
        c::NameRule::Control => w::NameRule::Control,
        c::NameRule::Space => w::NameRule::Space,
        c::NameRule::Digits => w::NameRule::Digits,
    })
}

pub(super) fn wire_held_role(value: &c::HeldRole, limits: &w::Limits) -> Result<w::HeldRole, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HeldRole::Owner => w::HeldRole::Owner,
        c::HeldRole::Maintainer => w::HeldRole::Maintainer,
        c::HeldRole::Member => w::HeldRole::Member,
        c::HeldRole::Observer => w::HeldRole::Observer,
        c::HeldRole::Policy { role } => {
            w::HeldRole::Policy(w::HeldRolePolicy::new(limits, w::HeldRolePolicyParts { role: *(role) })?)
        }
    })
}

pub(super) fn wire_lacking(value: &c::Lacking, limits: &w::Limits) -> Result<w::Lacking, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Lacking::HoldOwner { hold, owner } => w::Lacking::HoldOwner(w::LackingHoldOwner::new(
            limits,
            w::LackingHoldOwnerParts { hold: *(hold), owner: wire_hold_owner(owner, limits)? },
        )?),
        c::Lacking::Approver => w::Lacking::Approver,
        c::Lacking::Budget { needed, left } => w::Lacking::Budget(w::LackingBudget::new(
            limits,
            w::LackingBudgetParts { needed: *(needed), left: *(left) },
        )?),
        c::Lacking::GrantKind { kind } => {
            w::Lacking::GrantKind(w::LackingGrantKind::new(limits, w::LackingGrantKindParts { kind: *(kind) })?)
        }
        c::Lacking::Funding => w::Lacking::Funding,
        c::Lacking::Tool => w::Lacking::Tool,
        c::Lacking::Scope => w::Lacking::Scope,
        c::Lacking::Models => w::Lacking::Models,
        c::Lacking::Reference => w::Lacking::Reference,
        c::Lacking::ResourceAccess => w::Lacking::ResourceAccess,
        c::Lacking::ResourceUnavailable => w::Lacking::ResourceUnavailable,
        c::Lacking::Shape => w::Lacking::Shape,
        c::Lacking::Again { required } => {
            w::Lacking::Again(w::LackingAgain::new(limits, w::LackingAgainParts { required: *(required) })?)
        }
        c::Lacking::Account => w::Lacking::Account,
        c::Lacking::Writer => w::Lacking::Writer,
        c::Lacking::Required { connector, kind } => w::Lacking::Required(w::LackingRequired::new(
            limits,
            w::LackingRequiredParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Lacking::Failed { connector, kind } => w::Lacking::Failed(w::LackingFailed::new(
            limits,
            w::LackingFailedParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Lacking::Unguarded { connector, kind } => w::Lacking::Unguarded(w::LackingUnguarded::new(
            limits,
            w::LackingUnguardedParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Lacking::TaskKind { executor } => w::Lacking::TaskKind(w::LackingTaskKind::new(
            limits,
            w::LackingTaskKindParts {
                executor: (executor).as_ref().map(|item| wire_executor(item, limits)).transpose()?,
            },
        )?),
        c::Lacking::Depth { needed, allowed } => w::Lacking::Depth(w::LackingDepth::new(
            limits,
            w::LackingDepthParts {
                needed: (needed).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                allowed: *(allowed),
            },
        )?),
        c::Lacking::Deadline { asked, latest } => w::Lacking::Deadline(w::LackingDeadline::new(
            limits,
            w::LackingDeadlineParts {
                asked: (asked).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                latest: (latest).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::Lacking::Tasks { needed, left } => w::Lacking::Tasks(w::LackingTasks::new(
            limits,
            w::LackingTasksParts {
                needed: (needed).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                left: *(left),
            },
        )?),
    })
}

pub(super) fn wire_hold_owner(value: &c::HoldOwner, limits: &w::Limits) -> Result<w::HoldOwner, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldOwner::Party { person } => {
            w::HoldOwner::Party(w::HoldOwnerParty::new(limits, w::HoldOwnerPartyParts { person: *(person) })?)
        }
        c::HoldOwner::Task { task } => {
            w::HoldOwner::Task(w::HoldOwnerTask::new(limits, w::HoldOwnerTaskParts { task: *(task) })?)
        }
    })
}

pub(super) fn wire_executor(value: &c::Executor, limits: &w::Limits) -> Result<w::Executor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Executor::Agent { charter } => {
            w::Executor::Agent(w::ExecutorAgent::new(limits, w::ExecutorAgentParts { charter: *(charter) })?)
        }
        c::Executor::Procedure { connector, kind } => w::Executor::Procedure(w::ExecutorProcedure::new(
            limits,
            w::ExecutorProcedureParts { connector: *(connector), kind: *(kind) },
        )?),
        c::Executor::Party { party } => {
            w::Executor::Party(w::ExecutorParty::new(limits, w::ExecutorPartyParts { party: *(party) })?)
        }
        c::Executor::Role { project, role } => w::Executor::Role(w::ExecutorRole::new(
            limits,
            w::ExecutorRoleParts { project: *(project), role: *(role) },
        )?),
    })
}

pub(super) fn wire_limit(value: &c::Limit, limits: &w::Limits) -> Result<w::Limit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Limit::Admission { which } => w::Limit::Admission(w::LimitAdmission::new(
            limits,
            w::LimitAdmissionParts { which: wire_admission_limit(which, limits)? },
        )?),
        c::Limit::LiveTasks { project, limit } => w::Limit::LiveTasks(w::LimitLiveTasks::new(
            limits,
            w::LimitLiveTasksParts { project: *(project), limit: *(limit) },
        )?),
        c::Limit::Inbox { task, limit } => {
            w::Limit::Inbox(w::LimitInbox::new(limits, w::LimitInboxParts { task: *(task), limit: *(limit) })?)
        }
        c::Limit::Words { limit } => {
            w::Limit::Words(w::LimitWords::new(limits, w::LimitWordsParts { limit: *(limit) })?)
        }
        c::Limit::Fields => w::Limit::Fields,
        c::Limit::Records => w::Limit::Records,
        c::Limit::Requests => w::Limit::Requests,
        c::Limit::Reads => w::Limit::Reads,
        c::Limit::Watches => w::Limit::Watches,
        c::Limit::Objects => w::Limit::Objects,
    })
}

pub(super) fn wire_admission_limit(
    value: &c::AdmissionLimit,
    _limits: &w::Limits,
) -> Result<w::AdmissionLimit, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AdmissionLimit::Goals => w::AdmissionLimit::Goals,
        c::AdmissionLimit::Operations => w::AdmissionLimit::Operations,
        c::AdmissionLimit::Grace => w::AdmissionLimit::Grace,
        c::AdmissionLimit::Words => w::AdmissionLimit::Words,
        c::AdmissionLimit::Identity => w::AdmissionLimit::Identity,
        c::AdmissionLimit::Roles => w::AdmissionLimit::Roles,
        c::AdmissionLimit::Arithmetic => w::AdmissionLimit::Arithmetic,
        c::AdmissionLimit::Duplicate => w::AdmissionLimit::Duplicate,
        c::AdmissionLimit::Empty => w::AdmissionLimit::Empty,
        c::AdmissionLimit::Batch => w::AdmissionLimit::Batch,
        c::AdmissionLimit::ProjectLiveTasks => w::AdmissionLimit::ProjectLiveTasks,
        c::AdmissionLimit::LiveTasks => w::AdmissionLimit::LiveTasks,
        c::AdmissionLimit::Tree => w::AdmissionLimit::Tree,
        c::AdmissionLimit::Depth => w::AdmissionLimit::Depth,
        c::AdmissionLimit::Delegates => w::AdmissionLimit::Delegates,
        c::AdmissionLimit::Subscription => w::AdmissionLimit::Subscription,
        c::AdmissionLimit::Dependencies => w::AdmissionLimit::Dependencies,
        c::AdmissionLimit::Cycle => w::AdmissionLimit::Cycle,
        c::AdmissionLimit::Spec => w::AdmissionLimit::Spec,
        c::AdmissionLimit::Contract => w::AdmissionLimit::Contract,
        c::AdmissionLimit::Authority => w::AdmissionLimit::Authority,
        c::AdmissionLimit::Inputs => w::AdmissionLimit::Inputs,
        c::AdmissionLimit::Attempts => w::AdmissionLimit::Attempts,
        c::AdmissionLimit::LiveDelegates => w::AdmissionLimit::LiveDelegates,
        c::AdmissionLimit::Restore => w::AdmissionLimit::Restore,
        c::AdmissionLimit::Read => w::AdmissionLimit::Read,
        c::AdmissionLimit::Turn => w::AdmissionLimit::Turn,
        c::AdmissionLimit::HoldKind => w::AdmissionLimit::HoldKind,
        c::AdmissionLimit::HoldTaken => w::AdmissionLimit::HoldTaken,
        c::AdmissionLimit::Holds => w::AdmissionLimit::Holds,
        c::AdmissionLimit::InboxFull => w::AdmissionLimit::InboxFull,
        c::AdmissionLimit::Notes => w::AdmissionLimit::Notes,
        c::AdmissionLimit::Revision => w::AdmissionLimit::Revision,
        c::AdmissionLimit::Question => w::AdmissionLimit::Question,
        c::AdmissionLimit::Priority => w::AdmissionLimit::Priority,
        c::AdmissionLimit::Policy => w::AdmissionLimit::Policy,
        c::AdmissionLimit::Resource => w::AdmissionLimit::Resource,
        c::AdmissionLimit::Service => w::AdmissionLimit::Service,
        c::AdmissionLimit::Accounts => w::AdmissionLimit::Accounts,
        c::AdmissionLimit::HolderAccounts => w::AdmissionLimit::HolderAccounts,
        c::AdmissionLimit::Sources => w::AdmissionLimit::Sources,
        c::AdmissionLimit::SignIns => w::AdmissionLimit::SignIns,
        c::AdmissionLimit::SettingsProposals => w::AdmissionLimit::SettingsProposals,
        c::AdmissionLimit::Projects => w::AdmissionLimit::Projects,
        c::AdmissionLimit::PlacedHolds => w::AdmissionLimit::PlacedHolds,
        c::AdmissionLimit::ProjectPlacedHolds => w::AdmissionLimit::ProjectPlacedHolds,
        c::AdmissionLimit::PartyPlacedHolds => w::AdmissionLimit::PartyPlacedHolds,
        c::AdmissionLimit::Exemptions => w::AdmissionLimit::Exemptions,
    })
}

pub(super) fn wire_revision(value: &c::Revision, limits: &w::Limits) -> Result<w::Revision, w::Problem> {
    w::Revision::new(limits, w::RevisionParts { value: value.value })
}

pub(super) fn wire_moved(value: &c::Moved, limits: &w::Limits) -> Result<w::Moved, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Moved::PlacedHold { revision } => w::Moved::PlacedHold(w::MovedPlacedHold::new(
            limits,
            w::MovedPlacedHoldParts { revision: (revision).value },
        )?),
        c::Moved::Hold { reason } => w::Moved::Hold(w::MovedHold::new(
            limits,
            w::MovedHoldParts { reason: (reason).as_ref().map(|item| wire_current_hold(item, limits)).transpose()? },
        )?),
        c::Moved::Spec => w::Moved::Spec,
        c::Moved::PersonTask => w::Moved::PersonTask,
        c::Moved::Note => w::Moved::Note,
        c::Moved::Waiting => w::Moved::Waiting,
    })
}

pub(super) fn wire_current_hold(value: &c::CurrentHold, limits: &w::Limits) -> Result<w::CurrentHold, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::CurrentHold::Document(value) => w::CurrentHold::Document(w::CurrentHoldDocument::new(
            limits,
            w::CurrentHoldDocumentParts { value: facts::hold_reason(value, limits)? },
        )?),
        c::CurrentHold::Reason(value) => w::CurrentHold::Reason(w::CurrentHoldReason::new(
            limits,
            w::CurrentHoldReasonParts { value: wire_hold_category(value, limits)? },
        )?),
    })
}

pub(super) fn wire_hold_category(value: &c::HoldCategory, limits: &w::Limits) -> Result<w::HoldCategory, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldCategory::Exceeded(value) => w::HoldCategory::Exceeded(wire_exceeded(value, limits)?),
        c::HoldCategory::Failures { class } => w::HoldCategory::Failures(w::HoldCategoryFailures::new(
            limits,
            w::HoldCategoryFailuresParts { class: wire_hold_failure(class, limits)? },
        )?),
        c::HoldCategory::Stalled => w::HoldCategory::Stalled,
        c::HoldCategory::Dependency { task } => w::HoldCategory::Dependency(w::HoldCategoryDependency::new(
            limits,
            w::HoldCategoryDependencyParts { task: *(task) },
        )?),
        c::HoldCategory::Stopped => w::HoldCategory::Stopped,
        c::HoldCategory::Placed { hold } => {
            w::HoldCategory::Placed(w::HoldCategoryPlaced::new(limits, w::HoldCategoryPlacedParts { hold: *(hold) })?)
        }
        c::HoldCategory::Drift => w::HoldCategory::Drift,
        c::HoldCategory::Effects => w::HoldCategory::Effects,
        c::HoldCategory::EffectFailed => w::HoldCategory::EffectFailed,
        c::HoldCategory::Uncertain { entry } => w::HoldCategory::Uncertain(w::HoldCategoryUncertain::new(
            limits,
            w::HoldCategoryUncertainParts { entry: *(entry) },
        )?),
        c::HoldCategory::Budget => w::HoldCategory::Budget,
        c::HoldCategory::Deadline => w::HoldCategory::Deadline,
        c::HoldCategory::HoldsWaited => w::HoldCategory::HoldsWaited,
        c::HoldCategory::Circle => w::HoldCategory::Circle,
        c::HoldCategory::ResourceUnavailable => w::HoldCategory::ResourceUnavailable,
        c::HoldCategory::ResourceAccess => w::HoldCategory::ResourceAccess,
        c::HoldCategory::PoolLost { pool } => w::HoldCategory::PoolLost(w::HoldCategoryPoolLost::new(
            limits,
            w::HoldCategoryPoolLostParts { pool: *(pool) },
        )?),
        c::HoldCategory::Procedure => w::HoldCategory::Procedure,
    })
}

pub(super) fn wire_exceeded(value: &c::Exceeded, limits: &w::Limits) -> Result<w::Exceeded, w::Problem> {
    w::Exceeded::new(limits, w::ExceededParts { bound: wire_bound(&value.bound, limits)?, value: value.value })
}

pub(super) fn wire_bound(value: &c::Bound, _limits: &w::Limits) -> Result<w::Bound, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Bound::Reasoning => w::Bound::Reasoning,
        c::Bound::Metadata => w::Bound::Metadata,
        c::Bound::Items => w::Bound::Items,
        c::Bound::Answer => w::Bound::Answer,
        c::Bound::Tools => w::Bound::Tools,
        c::Bound::HistoryItems => w::Bound::HistoryItems,
        c::Bound::History => w::Bound::History,
        c::Bound::Window => w::Bound::Window,
        c::Bound::Output => w::Bound::Output,
    })
}

pub(super) fn wire_hold_failure(value: &c::HoldFailure, _limits: &w::Limits) -> Result<w::HoldFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldFailure::Transient => w::HoldFailure::Transient,
        c::HoldFailure::Permanent => w::HoldFailure::Permanent,
        c::HoldFailure::Run => w::HoldFailure::Run,
        c::HoldFailure::Agent => w::HoldFailure::Agent,
        c::HoldFailure::Lost => w::HoldFailure::Lost,
        c::HoldFailure::Invalid => w::HoldFailure::Invalid,
    })
}

pub(super) fn wire_llm_why(value: &c::LlmWhy, _limits: &w::Limits) -> Result<w::LlmWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::LlmWhy::Unconfigured => w::LlmWhy::Unconfigured,
        c::LlmWhy::Effort => w::LlmWhy::Effort,
        c::LlmWhy::Charter => w::LlmWhy::Charter,
        c::LlmWhy::Authority => w::LlmWhy::Authority,
        c::LlmWhy::Procedure => w::LlmWhy::Procedure,
        c::LlmWhy::Repeated => w::LlmWhy::Repeated,
    })
}

pub(super) fn wire_effect(value: &c::Effect, limits: &w::Limits) -> Result<w::Effect, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Effect::Release(value) => w::Effect::Release(w::EffectRelease::new(
            limits,
            w::EffectReleaseParts { what: wire_release_effect(value, limits)? },
        )?),
        c::Effect::Cancel(value) => w::Effect::Cancel(wire_impact(value, limits)?),
        c::Effect::Move { reservation } => {
            w::Effect::MoveTask(w::EffectMove::new(limits, w::EffectMoveParts { reservation: *(reservation) })?)
        }
    })
}

pub(super) fn wire_release_effect(
    value: &c::ReleaseEffect,
    _limits: &w::Limits,
) -> Result<w::ReleaseEffect, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReleaseEffect::Judged => w::ReleaseEffect::Judged,
        c::ReleaseEffect::RunAgain => w::ReleaseEffect::RunAgain,
        c::ReleaseEffect::Uncertain => w::ReleaseEffect::Uncertain,
    })
}

pub(super) fn wire_impact(value: &c::Impact, limits: &w::Limits) -> Result<w::Impact, w::Problem> {
    w::Impact::new(
        limits,
        w::ImpactParts {
            tasks: wire_phase_counts(&value.tasks, limits)?,
            live_runs: value.live_runs,
            proposals: value.proposals,
        },
    )
}

pub(super) fn wire_phase_counts(value: &c::PhaseCounts, limits: &w::Limits) -> Result<w::PhaseCounts, w::Problem> {
    w::PhaseCounts::new(
        limits,
        w::PhaseCountsParts {
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

pub(super) fn wire_owner_project_resource(
    value: &c::OwnerProjectResource,
    limits: &w::Limits,
) -> Result<w::OwnerProjectResource, w::Problem> {
    w::OwnerProjectResource::new(
        limits,
        w::OwnerProjectResourceParts {
            part: wire_owner_part(&value.part, limits)?,
            role: wire_owner_project_resource_role(&value.role, limits)?,
        },
    )
}

pub(super) fn wire_owner_project_resource_role(
    value: &c::OwnerProjectResourceRole,
    _limits: &w::Limits,
) -> Result<w::OwnerProjectResourceRole, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProjectResourceRole::Owned => w::OwnerProjectResourceRole::Owned,
        c::OwnerProjectResourceRole::Participant => w::OwnerProjectResourceRole::Participant,
        c::OwnerProjectResourceRole::Unavailable => w::OwnerProjectResourceRole::Unavailable,
        c::OwnerProjectResourceRole::Context => w::OwnerProjectResourceRole::Context,
    })
}

pub(super) fn wire_owner_project_policy(
    value: &c::OwnerProjectPolicy,
    limits: &w::Limits,
) -> Result<w::OwnerProjectPolicy, w::Problem> {
    w::OwnerProjectPolicy::new(
        limits,
        w::OwnerProjectPolicyParts {
            templates: list(value.templates.iter().map(|item| wire_template_override(item, limits)))?,
            ceiling: wire_owner_authority_document(&value.ceiling, limits)?,
            period_spend: value.period_spend,
            escalation_role: value.escalation_role.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            projections: pieces(
                value.projections.iter().map(|item| wire_owner_grant(item, limits)),
                w::OwnerGrant::measure,
                w::OwnerGrant::encode,
            )?,
            roles: pieces(
                value.roles.iter().map(|item| wire_owner_project_policy_role(item, limits)),
                w::OwnerProjectPolicyRole::measure,
                w::OwnerProjectPolicyRole::encode,
            )?,
            requirements: pieces(
                value.requirements.iter().map(|item| wire_policy_requirement(item, limits)),
                w::PolicyRequirement::measure,
                w::PolicyRequirement::encode,
            )?,
            permissions: value
                .permissions
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_policy_permission(item, limits))))
                .transpose()?,
            permissions_cut: value.permissions_cut,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_project_policy_role(
    value: &c::OwnerProjectPolicyRole,
    limits: &w::Limits,
) -> Result<w::OwnerProjectPolicyRole, w::Problem> {
    w::OwnerProjectPolicyRole::new(
        limits,
        w::OwnerProjectPolicyRoleParts {
            number: value.number,
            authority: wire_owner_authority_document(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        },
    )
}

pub(super) fn wire_owner_project_member(
    value: &c::OwnerProjectMember,
    limits: &w::Limits,
) -> Result<w::OwnerProjectMember, w::Problem> {
    w::OwnerProjectMember::new(
        limits,
        w::OwnerProjectMemberParts {
            party: value.party,
            kind: wire_party_kind(&value.kind, limits)?,
            role: wire_held_role(&value.role, limits)?,
            login: wire_document_text(&value.login, limits)?,
            name: wire_document_text(&value.name, limits)?,
        },
    )
}

pub(super) fn wire_party_kind(value: &c::PartyKind, _limits: &w::Limits) -> Result<w::PartyKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PartyKind::Person => w::PartyKind::Person,
        c::PartyKind::Service => w::PartyKind::Service,
    })
}

pub(super) fn wire_pool_ledger(value: &c::PoolLedger, limits: &w::Limits) -> Result<w::PoolLedger, w::Problem> {
    w::PoolLedger::new(
        limits,
        w::PoolLedgerParts {
            source: wire_funder(&value.source, limits)?,
            parent: value.parent.as_ref().map(|item| wire_funder(item, limits)).transpose()?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            left: value.left,
            closed: value.closed,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_project_spend(
    value: &c::OwnerProjectSpend,
    limits: &w::Limits,
) -> Result<w::OwnerProjectSpend, w::Problem> {
    w::OwnerProjectSpend::new(
        limits,
        w::OwnerProjectSpendParts {
            chip: facts::chip(&value.chip, limits)?,
            funder: wire_funder(&value.funder, limits)?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
        },
    )
}

pub(super) fn wire_recurring(value: &c::Recurring, limits: &w::Limits) -> Result<w::Recurring, w::Problem> {
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

pub(super) fn wire_proposed_task(value: &c::ProposedTask, limits: &w::Limits) -> Result<w::ProposedTask, w::Problem> {
    w::ProposedTask::new(
        limits,
        w::ProposedTaskParts {
            deadline: wire_new_deadline(&value.deadline, limits)?,
            number: value.number,
            spec: wire_specification_document(&value.spec, limits)?,
            executor: wire_owner_executor(&value.executor, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            authority: wire_owner_authority_document(&value.authority, limits)?,
            budget: value.budget,
            funder: wire_funder(&value.funder, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            resources: pieces(
                value.resources.iter().map(|item| wire_owner_part(item, limits)),
                w::OwnerPart::measure,
                w::OwnerPart::encode,
            )?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            priority: value.priority.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_new_deadline(value: &c::NewDeadline, limits: &w::Limits) -> Result<w::NewDeadline, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NewDeadline::Named => w::NewDeadline::Named,
        c::NewDeadline::Span(value) => {
            w::NewDeadline::Span(w::DeadlineSpan::new(limits, w::DeadlineSpanParts { value: *(value) })?)
        }
    })
}

pub(super) fn wire_specification_document(
    value: &c::SpecificationDocument,
    limits: &w::Limits,
) -> Result<w::SpecificationDocument, w::Problem> {
    w::SpecificationDocument::new(
        limits,
        w::SpecificationDocumentParts {
            specification: wire_document_specification(&value.specification, limits)?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_document_specification(
    value: &c::DocumentSpecification,
    limits: &w::Limits,
) -> Result<w::DocumentSpecification, w::Problem> {
    w::DocumentSpecification::new(
        limits,
        w::DocumentSpecificationParts {
            words: wire_document_text(&value.words, limits)?,
            parameters: list(value.parameters.iter().map(|item| wire_parameter_document(item, limits)))?,
            inputs: list(value.inputs.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            asks: value.asks,
        },
    )
}

pub(super) fn wire_parameter_document(
    value: &c::ParameterDocument,
    limits: &w::Limits,
) -> Result<w::ParameterDocument, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ParameterDocument::Number { name, value } => w::ParameterDocument::Number(w::ParameterDocumentNumber::new(
            limits,
            w::ParameterDocumentNumberParts { name: *(name), value: *(value) },
        )?),
        c::ParameterDocument::Bytes { name, value } => w::ParameterDocument::Bytes(w::ParameterDocumentBytes::new(
            limits,
            w::ParameterDocumentBytesParts { name: *(name), value: wire_document_text(value, limits)? },
        )?),
        c::ParameterDocument::Resource { name, connector, resource } => {
            w::ParameterDocument::Resource(w::ParameterDocumentResource::new(
                limits,
                w::ParameterDocumentResourceParts { name: *(name), connector: *(connector), resource: *(resource) },
            )?)
        }
    })
}

pub(super) fn wire_owner_executor(
    value: &c::OwnerExecutor,
    limits: &w::Limits,
) -> Result<w::OwnerExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerExecutor::Agent { charter } => w::OwnerExecutor::Agent(w::OwnerExecutorAgent::new(
            limits,
            w::OwnerExecutorAgentParts { charter: *(charter) },
        )?),
        c::OwnerExecutor::Procedure { connector, code } => w::OwnerExecutor::Procedure(w::OwnerExecutorProcedure::new(
            limits,
            w::OwnerExecutorProcedureParts { connector: *(connector), code: *(code) },
        )?),
        c::OwnerExecutor::Person(value) => w::OwnerExecutor::Person(w::OwnerExecutorPerson::new(
            limits,
            w::OwnerExecutorPersonParts { value: wire_person_address(value, limits)? },
        )?),
    })
}

pub(super) fn wire_person_address(
    value: &c::PersonAddress,
    limits: &w::Limits,
) -> Result<w::PersonAddress, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PersonAddress::Party(value) => {
            w::PersonAddress::Party(w::PersonAddressParty::new(limits, w::PersonAddressPartyParts { value: *(value) })?)
        }
        c::PersonAddress::Role(value) => {
            w::PersonAddress::Role(w::PersonAddressRole::new(limits, w::PersonAddressRoleParts { value: *(value) })?)
        }
    })
}

pub(super) fn wire_owner_wake_policy(
    value: &c::OwnerWakePolicy,
    limits: &w::Limits,
) -> Result<w::OwnerWakePolicy, w::Problem> {
    w::OwnerWakePolicy::new(
        limits,
        w::OwnerWakePolicyParts {
            words: wire_wake_rule(&value.words, limits)?,
            notices: wire_wake_rule(&value.notices, limits)?,
            news: wire_wake_rule(&value.news, limits)?,
            results: wire_results_wake(&value.results, limits)?,
            questions: value.questions,
            answers: value.answers,
            timers: value.timers,
        },
    )
}

pub(super) fn wire_wake_rule(value: &c::WakeRule, limits: &w::Limits) -> Result<w::WakeRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WakeRule::Never => w::WakeRule::Never,
        c::WakeRule::Immediate => w::WakeRule::Immediate,
        c::WakeRule::Batch { count, age } => {
            w::WakeRule::Batch(w::WakeRuleBatch::new(limits, w::WakeRuleBatchParts { count: *(count), age: *(age) })?)
        }
    })
}

pub(super) fn wire_results_wake(value: &c::ResultsWake, _limits: &w::Limits) -> Result<w::ResultsWake, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ResultsWake::Never => w::ResultsWake::Never,
        c::ResultsWake::Each => w::ResultsWake::Each,
        c::ResultsWake::LastOrFailure => w::ResultsWake::LastOrFailure,
    })
}

pub(super) fn wire_recurring_overlap(
    value: &c::RecurringOverlap,
    _limits: &w::Limits,
) -> Result<w::RecurringOverlap, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RecurringOverlap::Skip => w::RecurringOverlap::Skip,
        c::RecurringOverlap::Wait => w::RecurringOverlap::Wait,
    })
}

pub(super) fn wire_owner_historical(
    value: &c::OwnerHistorical,
    limits: &w::Limits,
) -> Result<w::OwnerHistorical, w::Problem> {
    w::OwnerHistorical::new(
        limits,
        w::OwnerHistoricalParts {
            key: wire_object_key(&value.key, limits)?,
            value: wire_owner_historical_value(&value.value, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            by: value.by.as_ref().map(|item| wire_owner_requester(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_object_key(value: &c::ObjectKey, limits: &w::Limits) -> Result<w::ObjectKey, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ObjectKey::Task(value) => {
            w::ObjectKey::Task(w::ObjectKeyTask::new(limits, w::ObjectKeyTaskParts { value: *(value) })?)
        }
        c::ObjectKey::Proposal(value) => w::ObjectKey::Proposal(w::ObjectKeyProposal::new(
            limits,
            w::ObjectKeyProposalParts { value: wire_proposal_key(value, limits)? },
        )?),
        c::ObjectKey::Escalation(value) => w::ObjectKey::Escalation(w::ObjectKeyEscalation::new(
            limits,
            w::ObjectKeyEscalationParts { value: *(value) },
        )?),
        c::ObjectKey::Question { task, message } => w::ObjectKey::Question(w::ObjectKeyQuestion::new(
            limits,
            w::ObjectKeyQuestionParts { task: *(task), message: *(message) },
        )?),
        c::ObjectKey::PersonTask(value) => w::ObjectKey::PersonTask(w::ObjectKeyPersonTask::new(
            limits,
            w::ObjectKeyPersonTaskParts { value: *(value) },
        )?),
        c::ObjectKey::Result(value) => {
            w::ObjectKey::Result(w::ObjectKeyResult::new(limits, w::ObjectKeyResultParts { value: *(value) })?)
        }
        c::ObjectKey::Note { project, note } => w::ObjectKey::Note(w::ObjectKeyNote::new(
            limits,
            w::ObjectKeyNoteParts { project: *(project), note: *(note) },
        )?),
    })
}

pub(super) fn wire_owner_historical_value(
    value: &c::OwnerHistoricalValue,
    limits: &w::Limits,
) -> Result<w::OwnerHistoricalValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHistoricalValue::Task { document, result } => {
            w::OwnerHistoricalValue::Task(w::OwnerHistoricalValueTask::new(
                limits,
                w::OwnerHistoricalValueTaskParts {
                    document: wire_task(document, limits)?,
                    result: wire_result_document(result, limits)?,
                },
            )?)
        }
        c::OwnerHistoricalValue::Proposal(value) => {
            w::OwnerHistoricalValue::Proposal(w::OwnerHistoricalValueProposal::new(
                limits,
                w::OwnerHistoricalValueProposalParts { value: wire_owner_proposal_history(value, limits)? },
            )?)
        }
        c::OwnerHistoricalValue::PersonProposal { proposal, decision } => {
            w::OwnerHistoricalValue::PersonProposal(w::OwnerHistoricalValuePersonProposal::new(
                limits,
                w::OwnerHistoricalValuePersonProposalParts {
                    proposal: wire_owner_person_proposal(proposal, limits)?,
                    decision: (decision).as_ref().map(|item| wire_owner_proposal_decision(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerHistoricalValue::Escalation { task, decision, hold } => {
            w::OwnerHistoricalValue::Escalation(w::OwnerHistoricalValueEscalation::new(
                limits,
                w::OwnerHistoricalValueEscalationParts {
                    task: wire_task(task, limits)?,
                    decision: wire_owner_escalation_decision_record(decision, limits)?,
                    hold: (hold).as_ref().map(|item| wire_owner_milestone_line(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerHistoricalValue::Question { question, answer, ending } => {
            w::OwnerHistoricalValue::Question(w::OwnerHistoricalValueQuestion::new(
                limits,
                w::OwnerHistoricalValueQuestionParts {
                    question: wire_question(question, limits)?,
                    answer: (answer).as_ref().map(|item| wire_owner_activity_line(item, limits)).transpose()?,
                    ending: (ending).as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerHistoricalValue::PersonTask { document, result } => {
            w::OwnerHistoricalValue::PersonTask(w::OwnerHistoricalValuePersonTask::new(
                limits,
                w::OwnerHistoricalValuePersonTaskParts {
                    document: wire_owner_person_task(document, limits)?,
                    result: wire_result_document(result, limits)?,
                },
            )?)
        }
    })
}

pub(super) fn wire_task(value: &c::Task, limits: &w::Limits) -> Result<w::Task, w::Problem> {
    w::Task::new(
        limits,
        w::TaskParts {
            settings: value.settings.as_ref().map(|item| wire_task_settings(item, limits)).transpose()?,
            label: value.label,
            flags: wire_task_flags(&value.flags, limits)?,
            last_activity: value.last_activity.as_nanos(),
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
                value.inputs.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            project: value.project,
            requester: wire_owner_requester(&value.requester, limits)?,
            lineage: list(value.lineage.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            spec: wire_specification_document(&value.spec, limits)?,
            spec_revision: wire_revision(&value.spec_revision, limits)?,
            person_revision: value.person_revision.as_ref().map(|item| wire_revision(item, limits)).transpose()?,
            taken_by: value.taken_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            subscriptions: wire_subscriptions(&value.subscriptions, limits)?,
            result_contract: wire_task_contract_document(&value.result_contract, limits)?,
            dependencies: pieces(
                value.dependencies.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            references: pieces(
                value.references.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            cut: value.cut,
            resources: pieces(
                value.resources.iter().map(|item| wire_owner_resource_line(item, limits)),
                w::OwnerResourceLine::measure,
                w::OwnerResourceLine::encode,
            )?,
            budget: wire_owner_budget(&value.budget, limits)?,
            authority: wire_owner_authority_document(&value.authority, limits)?,
            hold: value.hold.as_ref().map(|item| wire_owner_chip_hold(item, limits)).transpose()?,
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

pub(super) fn wire_task_settings(value: &c::TaskSettings, limits: &w::Limits) -> Result<w::TaskSettings, w::Problem> {
    w::TaskSettings::new(
        limits,
        w::TaskSettingsParts {
            task: value.task,
            run: value.run.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            newest: value.newest,
        },
    )
}

pub(super) fn wire_owner_task_link(
    value: &c::OwnerTaskLink,
    limits: &w::Limits,
) -> Result<w::OwnerTaskLink, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskLink::Live(value) => w::OwnerTaskLink::Live(w::OwnerTaskLinkLive::new(
            limits,
            w::OwnerTaskLinkLiveParts { value: facts::chip(value, limits)? },
        )?),
        c::OwnerTaskLink::Retained { chip, at } => w::OwnerTaskLink::Retained(w::OwnerTaskLinkRetained::new(
            limits,
            w::OwnerTaskLinkRetainedParts {
                chip: facts::chip(chip, limits)?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::OwnerTaskLink::Stored { number, status } => w::OwnerTaskLink::Stored(w::OwnerTaskLinkStored::new(
            limits,
            w::OwnerTaskLinkStoredParts {
                number: *(number),
                status: (status).as_ref().map(|item| wire_owner_task_status(item, limits)).transpose()?,
            },
        )?),
    })
}

pub(super) fn wire_subscriptions(value: &c::Subscriptions, limits: &w::Limits) -> Result<w::Subscriptions, w::Problem> {
    w::Subscriptions::new(
        limits,
        w::SubscriptionsParts { entries: list(value.entries.iter().map(|item| wire_subscription(item, limits)))? },
    )
}

pub(super) fn wire_subscription(value: &c::Subscription, limits: &w::Limits) -> Result<w::Subscription, w::Problem> {
    w::Subscription::new(
        limits,
        w::SubscriptionParts { number: value.number, kind: wire_subscription_kind(&value.kind, limits)? },
    )
}

pub(super) fn wire_subscription_kind(
    value: &c::SubscriptionKind,
    limits: &w::Limits,
) -> Result<w::SubscriptionKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SubscriptionKind::Task { target, held, result } => w::SubscriptionKind::Task(w::SubscriptionKindTask::new(
            limits,
            w::SubscriptionKindTaskParts { target: *(target), held: *(held), result: *(result) },
        )?),
        c::SubscriptionKind::Timer { at, period } => w::SubscriptionKind::Timer(w::SubscriptionKindTimer::new(
            limits,
            w::SubscriptionKindTimerParts {
                at: (at).as_nanos(),
                period: (period).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::SubscriptionKind::Topic { connector, topic } => w::SubscriptionKind::Topic(w::SubscriptionKindTopic::new(
            limits,
            w::SubscriptionKindTopicParts { connector: *(connector), topic: *(topic) },
        )?),
    })
}

pub(super) fn wire_owner_resource_line(
    value: &c::OwnerResourceLine,
    limits: &w::Limits,
) -> Result<w::OwnerResourceLine, w::Problem> {
    w::OwnerResourceLine::new(
        limits,
        w::OwnerResourceLineParts {
            part: wire_owner_part(&value.part, limits)?,
            kind: value.kind,
            taken: value.taken,
            holder: value.holder.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            writer: value.writer.as_ref().map(|item| wire_owner_resource_writer(item, limits)).transpose()?,
            writer_lost: value.writer_lost,
        },
    )
}

pub(super) fn wire_owner_resource_writer(
    value: &c::OwnerResourceWriter,
    limits: &w::Limits,
) -> Result<w::OwnerResourceWriter, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerResourceWriter::Run { task, attempt } => w::OwnerResourceWriter::Run(w::OwnerResourceWriterRun::new(
            limits,
            w::OwnerResourceWriterRunParts { task: *(task), attempt: *(attempt) },
        )?),
        c::OwnerResourceWriter::Effect { entry } => w::OwnerResourceWriter::Effect(w::OwnerResourceWriterEffect::new(
            limits,
            w::OwnerResourceWriterEffectParts { entry: *(entry) },
        )?),
    })
}

pub(super) fn wire_owner_chip_hold(
    value: &c::OwnerChipHold,
    limits: &w::Limits,
) -> Result<w::OwnerChipHold, w::Problem> {
    w::OwnerChipHold::new(
        limits,
        w::OwnerChipHoldParts {
            account: value.account.as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
            source: value.source.as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
            reason: wire_hold_category(&value.reason, limits)?,
            revision: value.revision,
            before: wire_owner_prior_phase(&value.before, limits)?,
            escalation: value.escalation.as_ref().map(|item| wire_owner_hold_escalation(item, limits)).transpose()?,
            tries: wire_owner_failure_tries(&value.tries, limits)?,
            hold_wait_since: value
                .hold_wait_since
                .as_ref()
                .map(|item| Ok::<_, w::Problem>((item).as_nanos()))
                .transpose()?,
            circle: value.circle.as_ref().map(|item| wire_circle(item, limits)).transpose()?,
            placed: value.placed.as_ref().map(|item| wire_placed_line(item, limits)).transpose()?,
            words: value.words.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_owner_prior_phase(
    value: &c::OwnerPriorPhase,
    limits: &w::Limits,
) -> Result<w::OwnerPriorPhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPriorPhase::Waiting => w::OwnerPriorPhase::Waiting,
        c::OwnerPriorPhase::Active(value) => w::OwnerPriorPhase::Active(w::OwnerPriorPhaseActive::new(
            limits,
            w::OwnerPriorPhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::OwnerPriorPhase::Closing(value) => w::OwnerPriorPhase::Closing(w::OwnerPriorPhaseClosing::new(
            limits,
            w::OwnerPriorPhaseClosingParts { value: wire_closing_stage(value, limits)? },
        )?),
    })
}

pub(super) fn wire_run_stage(value: &c::RunStage, limits: &w::Limits) -> Result<w::RunStage, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunStage::Idle => w::RunStage::Idle,
        c::RunStage::Due => w::RunStage::Due,
        c::RunStage::Preparing => w::RunStage::Preparing,
        c::RunStage::Claimed { attempt } => {
            w::RunStage::Claimed(w::RunStageClaimed::new(limits, w::RunStageClaimedParts { attempt: *(attempt) })?)
        }
        c::RunStage::Running { attempt } => {
            w::RunStage::Running(w::RunStageRunning::new(limits, w::RunStageRunningParts { attempt: *(attempt) })?)
        }
        c::RunStage::BackingOff { until } => w::RunStage::BackingOff(w::RunStageBackingOff::new(
            limits,
            w::RunStageBackingOffParts { until: (until).as_nanos() },
        )?),
    })
}

pub(super) fn wire_closing_stage(value: &c::ClosingStage, limits: &w::Limits) -> Result<w::ClosingStage, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ClosingStage::Run { attempt } => {
            w::ClosingStage::Run(w::ClosingStageRun::new(limits, w::ClosingStageRunParts { attempt: *(attempt) })?)
        }
        c::ClosingStage::Delegates => w::ClosingStage::Delegates,
        c::ClosingStage::Effects => w::ClosingStage::Effects,
        c::ClosingStage::Releases => w::ClosingStage::Releases,
        c::ClosingStage::Settled => w::ClosingStage::Settled,
    })
}

pub(super) fn wire_owner_hold_escalation(
    value: &c::OwnerHoldEscalation,
    limits: &w::Limits,
) -> Result<w::OwnerHoldEscalation, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHoldEscalation::Routing => w::OwnerHoldEscalation::Routing,
        c::OwnerHoldEscalation::Waiting { holder, entry, since } => {
            w::OwnerHoldEscalation::Waiting(w::OwnerHoldEscalationWaiting::new(
                limits,
                w::OwnerHoldEscalationWaitingParts {
                    holder: wire_owner_escalation_holder(holder, limits)?,
                    entry: *(entry),
                    since: (since).as_nanos(),
                },
            )?)
        }
        c::OwnerHoldEscalation::Rejected { by, reason } => {
            w::OwnerHoldEscalation::Rejected(w::OwnerHoldEscalationRejected::new(
                limits,
                w::OwnerHoldEscalationRejectedParts { by: *(by), reason: wire_document_text(reason, limits)? },
            )?)
        }
    })
}

pub(super) fn wire_owner_escalation_holder(
    value: &c::OwnerEscalationHolder,
    limits: &w::Limits,
) -> Result<w::OwnerEscalationHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerEscalationHolder::Task(value) => w::OwnerEscalationHolder::Task(w::OwnerEscalationHolderTask::new(
            limits,
            w::OwnerEscalationHolderTaskParts { value: *(value) },
        )?),
        c::OwnerEscalationHolder::Person(value) => w::OwnerEscalationHolder::Person(
            w::OwnerEscalationHolderPerson::new(limits, w::OwnerEscalationHolderPersonParts { value: *(value) })?,
        ),
        c::OwnerEscalationHolder::Role { project, role } => {
            w::OwnerEscalationHolder::Role(w::OwnerEscalationHolderRole::new(
                limits,
                w::OwnerEscalationHolderRoleParts { project: *(project), role: *(role) },
            )?)
        }
    })
}

pub(super) fn wire_owner_failure_tries(
    value: &c::OwnerFailureTries,
    limits: &w::Limits,
) -> Result<w::OwnerFailureTries, w::Problem> {
    w::OwnerFailureTries::new(
        limits,
        w::OwnerFailureTriesParts {
            transient: value.transient,
            permanent: value.permanent,
            run: value.run,
            agent: value.agent,
            lost: value.lost,
            invalid: value.invalid,
        },
    )
}

pub(super) fn wire_circle(value: &c::Circle, limits: &w::Limits) -> Result<w::Circle, w::Problem> {
    w::Circle::new(
        limits,
        w::CircleParts { tasks: list(value.tasks.iter().map(|item| Ok::<_, w::Problem>(*(item))))?, cut: value.cut },
    )
}

pub(super) fn wire_placed_line(value: &c::PlacedHold, limits: &w::Limits) -> Result<w::PlacedLine, w::Problem> {
    w::PlacedLine::new(
        limits,
        w::PlacedLineParts {
            number: value.number,
            owner: wire_hold_owner(&value.owner, limits)?,
            mode: wire_hold_mode(&value.mode, limits)?,
            scope: wire_hold_scope(&value.scope, limits)?,
            reason: bytes::copy_of(&value.reason),
            placed_at: value.placed_at.as_nanos(),
            revision: value.revision.value,
            lifted: value.lifted,
            may_lift: value.may_lift,
        },
    )
}

pub(super) fn wire_hold_mode(value: &c::HoldMode, _limits: &w::Limits) -> Result<w::HoldMode, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldMode::Stop => w::HoldMode::Stop,
        c::HoldMode::Drain => w::HoldMode::Drain,
    })
}

pub(super) fn wire_hold_scope(value: &c::HoldScope, limits: &w::Limits) -> Result<w::HoldScope, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldScope::Subtree { task } => {
            w::HoldScope::Subtree(w::HoldScopeSubtree::new(limits, w::HoldScopeSubtreeParts { task: *(task) })?)
        }
        c::HoldScope::Project => w::HoldScope::Project,
    })
}

pub(super) fn wire_task_result(value: &c::TaskResult, limits: &w::Limits) -> Result<w::TaskResult, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskResult::Document(value) => w::TaskResult::Document(Box::new(w::TaskResultDocument::new(
            limits,
            w::TaskResultDocumentParts { value: wire_result_document(value, limits)? },
        )?)),
        c::TaskResult::Ending(value) => w::TaskResult::Ending(w::TaskResultEnding::new(
            limits,
            w::TaskResultEndingParts { value: facts::owner_chip_ending(value, limits)? },
        )?),
    })
}

pub(super) fn wire_result_document(value: &c::Result, limits: &w::Limits) -> Result<w::ResultDocument, w::Problem> {
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

pub(super) fn wire_result_details(
    value: &c::ResultDetails,
    limits: &w::Limits,
) -> Result<w::ResultDetails, w::Problem> {
    w::ResultDetails::new(
        limits,
        w::ResultDetailsParts {
            contract: wire_task_contract_document(&value.contract, limits)?,
            requester: wire_owner_requester(&value.requester, limits)?,
            executor: wire_owner_executor(&value.executor, limits)?,
            chosen_by: value.chosen_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            delegates: pieces(
                value.delegates.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            followups: value
                .followups
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_owner_task_link(item, limits)),
                        w::OwnerTaskLink::measure,
                        w::OwnerTaskLink::encode,
                    )
                })
                .transpose()?,
            proposal: value.proposal.as_ref().map(|item| wire_proposal_key(item, limits)).transpose()?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_live_delegate(value: &c::LiveDelegate, limits: &w::Limits) -> Result<w::LiveDelegate, w::Problem> {
    w::LiveDelegate::new(
        limits,
        w::LiveDelegateParts { chip: facts::chip(&value.chip, limits)?, takeover_budget: value.takeover_budget },
    )
}

pub(super) fn wire_task_context(value: &c::TaskContext, limits: &w::Limits) -> Result<w::TaskContext, w::Problem> {
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

pub(super) fn wire_spend_line(value: &c::SpendLine, limits: &w::Limits) -> Result<w::SpendLine, w::Problem> {
    w::SpendLine::new(
        limits,
        w::SpendLineParts {
            task: value.task,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            spent: value.spent,
        },
    )
}

pub(super) fn wire_runtime(value: &c::Runtime, limits: &w::Limits) -> Result<w::Runtime, w::Problem> {
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

pub(super) fn wire_active_run(value: &c::ActiveRun, limits: &w::Limits) -> Result<w::ActiveRun, w::Problem> {
    w::ActiveRun::new(
        limits,
        w::ActiveRunParts {
            stage: wire_run_stage(&value.stage, limits)?,
            phase_since: value.phase_since.as_nanos(),
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            placement: value.placement.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            retry: value.retry.as_ref().map(|item| wire_retry_notice(item, limits)).transpose()?,
            observation: value.observation.as_ref().map(|item| wire_run_observation(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_retry_notice(value: &c::RetryNotice, limits: &w::Limits) -> Result<w::RetryNotice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RetryNotice::OwnerFailed { class, count, allowance } => {
            w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed::new(
                limits,
                w::RetryNoticeOwnerFailedParts {
                    class: wire_hold_failure(class, limits)?,
                    count: *(count),
                    allowance: *(allowance),
                },
            )?)
        }
        c::RetryNotice::Failed { class, count, allowance } => w::RetryNotice::Failed(w::RetryNoticeFailed::new(
            limits,
            w::RetryNoticeFailedParts {
                class: wire_failure_class(class, limits)?,
                count: *(count),
                allowance: *(allowance),
            },
        )?),
        c::RetryNotice::Refused { count } => {
            w::RetryNotice::Refused(w::RetryNoticeRefused::new(limits, w::RetryNoticeRefusedParts { count: *(count) })?)
        }
    })
}

pub(super) fn wire_failure_class(value: &c::FailureClass, _limits: &w::Limits) -> Result<w::FailureClass, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::FailureClass::HostLost => w::FailureClass::HostLost,
        c::FailureClass::Temporary => w::FailureClass::Temporary,
        c::FailureClass::Permanent => w::FailureClass::Permanent,
        c::FailureClass::Stall => w::FailureClass::Stall,
    })
}

pub(super) fn wire_run_observation(
    value: &c::RunObservation,
    limits: &w::Limits,
) -> Result<w::RunObservation, w::Problem> {
    w::RunObservation::new(
        limits,
        w::RunObservationParts {
            channel: value.channel.raw(),
            work: wire_run_work(&value.work, limits)?,
            since: value.since.as_nanos(),
            reported_at: value.reported_at.as_nanos(),
            started: value.started.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_run_work(value: &c::RunWork, limits: &w::Limits) -> Result<w::RunWork, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::RunWork::Preparing => w::RunWork::Preparing,
        c::RunWork::Starting => w::RunWork::Starting,
        c::RunWork::Active => w::RunWork::Active,
        c::RunWork::Writing => w::RunWork::Writing,
        c::RunWork::Tool { call, tool } => w::RunWork::Tool(w::RunWorkTool::new(
            limits,
            w::RunWorkToolParts { call: wire_call_identity(call, limits)?, tool: bytes::copy_of((tool).bytes()) },
        )?),
        c::RunWork::Waiting => w::RunWork::Waiting,
        c::RunWork::Ending => w::RunWork::Ending,
    })
}

pub(super) fn wire_call_identity(value: &c::CallIdentity, limits: &w::Limits) -> Result<w::CallIdentity, w::Problem> {
    w::CallIdentity::new(
        limits,
        w::CallIdentityParts {
            owner: value.owner.raw(),
            sequence: value.sequence,
            retry: value.retry,
            id: bytes::copy_of(value.id.bytes()),
        },
    )
}

pub(super) fn wire_task_run_wait(value: &c::TaskRunWait, limits: &w::Limits) -> Result<w::TaskRunWait, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskRunWait::Account { account, source, why } => w::TaskRunWait::Account(w::TaskWaitAccount::new(
            limits,
            w::TaskWaitAccountParts {
                account: (account).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                source: (source).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                why: wire_task_wait_why(why, limits)?,
            },
        )?),
        c::TaskRunWait::Credential => w::TaskRunWait::Credential,
        c::TaskRunWait::Host => w::TaskRunWait::Host,
        c::TaskRunWait::Refused(value) => w::TaskRunWait::Refused(w::TaskRunWaitRefused::new(
            limits,
            w::TaskRunWaitRefusedParts { value: wire_task_wait_refused(value, limits)? },
        )?),
    })
}

pub(super) fn wire_task_wait_why(value: &c::TaskWaitWhy, limits: &w::Limits) -> Result<w::TaskWaitWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskWaitWhy::Source(value) => w::TaskWaitWhy::Source(w::TaskWaitSource::new(
            limits,
            w::TaskWaitSourceParts { state: wire_task_source_state(value, limits)? },
        )?),
        c::TaskWaitWhy::Refreshing => w::TaskWaitWhy::Refreshing,
        c::TaskWaitWhy::Host => w::TaskWaitWhy::Host,
        c::TaskWaitWhy::Empty => w::TaskWaitWhy::Empty,
        c::TaskWaitWhy::Invalid(value) => w::TaskWaitWhy::Invalid(w::TaskWaitInvalid::new(
            limits,
            w::TaskWaitInvalidParts { why: wire_account_why(value, limits)? },
        )?),
    })
}

pub(super) fn wire_task_source_state(
    value: &c::TaskSourceState,
    limits: &w::Limits,
) -> Result<w::TaskSourceState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskSourceState::Asking => w::TaskSourceState::Asking,
        c::TaskSourceState::Granted => w::TaskSourceState::Granted,
        c::TaskSourceState::Later { until } => w::TaskSourceState::Later(w::TaskSourceStateUntil::new(
            limits,
            w::TaskSourceStateUntilParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::TaskSourceState::Limited { until } => w::TaskSourceState::Limited(w::TaskSourceStateUntil::new(
            limits,
            w::TaskSourceStateUntilParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::TaskSourceState::Refused => w::TaskSourceState::Refused,
        c::TaskSourceState::NeedsSignIn => w::TaskSourceState::NeedsSignIn,
        c::TaskSourceState::Misconfigured => w::TaskSourceState::Misconfigured,
        c::TaskSourceState::Unloaded { at, why } => w::TaskSourceState::Unloaded(w::TaskSourceStateUnloaded::new(
            limits,
            w::TaskSourceStateUnloadedParts {
                at: wire_task_source_place(at, limits)?,
                why: wire_task_source_unread(why, limits)?,
            },
        )?),
        c::TaskSourceState::Cancelled => w::TaskSourceState::Cancelled,
        c::TaskSourceState::Closing => w::TaskSourceState::Closing,
        c::TaskSourceState::Moving => w::TaskSourceState::Moving,
    })
}

pub(super) fn wire_task_source_place(
    value: &c::OwnerAccountPlace,
    _limits: &w::Limits,
) -> Result<w::TaskSourcePlace, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountPlace::Directory => w::TaskSourcePlace::Directory,
        c::OwnerAccountPlace::File => w::TaskSourcePlace::File,
    })
}

pub(super) fn wire_task_source_unread(
    value: &c::OwnerAccountUnread,
    _limits: &w::Limits,
) -> Result<w::TaskSourceUnread, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountUnread::Unreadable => w::TaskSourceUnread::Unreadable,
        c::OwnerAccountUnread::Refused => w::TaskSourceUnread::Refused,
        c::OwnerAccountUnread::Failed => w::TaskSourceUnread::Failed,
        c::OwnerAccountUnread::Stalled => w::TaskSourceUnread::Stalled,
        c::OwnerAccountUnread::TooLarge => w::TaskSourceUnread::TooLarge,
    })
}

pub(super) fn wire_task_wait_refused(
    value: &c::TaskWaitRefused,
    limits: &w::Limits,
) -> Result<w::TaskWaitRefused, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskWaitRefused::Hello { kind, at, why } => w::TaskWaitRefused::Hello(w::TaskWaitHello::new(
            limits,
            w::TaskWaitHelloParts {
                kind: wire_owner_host_kind(kind, limits)?,
                at: (at).as_nanos(),
                why: wire_owner_hello_refusal(why, limits)?,
            },
        )?),
        c::TaskWaitRefused::Program(value) => w::TaskWaitRefused::Program(w::TaskWaitRefusedProgram::new(
            limits,
            w::TaskWaitRefusedProgramParts { value: wire_task_wait_program(value, limits)? },
        )?),
    })
}

pub(super) fn wire_owner_host_kind(
    value: &c::OwnerHostKind,
    _limits: &w::Limits,
) -> Result<w::OwnerHostKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHostKind::Worker => w::OwnerHostKind::Worker,
        c::OwnerHostKind::Engine => w::OwnerHostKind::Engine,
    })
}

pub(super) fn wire_owner_hello_refusal(
    value: &c::OwnerHelloRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerHelloRefusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHelloRefusal::Agent { why } => w::OwnerHelloRefusal::Agent(w::OwnerHelloRefusalAgent::new(
            limits,
            w::OwnerHelloRefusalAgentParts { why: wire_owner_agent_shortfall(why, limits)? },
        )?),
        c::OwnerHelloRefusal::ReservedChannel => w::OwnerHelloRefusal::ReservedChannel,
        c::OwnerHelloRefusal::Position { worker } => w::OwnerHelloRefusal::Position(w::OwnerHelloRefusalPosition::new(
            limits,
            w::OwnerHelloRefusalPositionParts { worker: *(worker) },
        )?),
        c::OwnerHelloRefusal::Workers { limit } => w::OwnerHelloRefusal::Workers(w::OwnerHelloRefusalWorkers::new(
            limits,
            w::OwnerHelloRefusalWorkersParts { limit: *(limit) },
        )?),
        c::OwnerHelloRefusal::StopBound { grace } => w::OwnerHelloRefusal::StopBound(
            w::OwnerHelloRefusalStopBound::new(limits, w::OwnerHelloRefusalStopBoundParts { grace: *(grace) })?,
        ),
    })
}

pub(super) fn wire_owner_agent_shortfall(
    value: &c::OwnerAgentShortfall,
    limits: &w::Limits,
) -> Result<w::OwnerAgentShortfall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAgentShortfall::Model { charter, model } => {
            w::OwnerAgentShortfall::Model(w::OwnerAgentShortfallModel::new(
                limits,
                w::OwnerAgentShortfallModelParts { charter: *(charter), model: *(model) },
            )?)
        }
        c::OwnerAgentShortfall::Window { charter, model } => {
            w::OwnerAgentShortfall::Window(w::OwnerAgentShortfallWindow::new(
                limits,
                w::OwnerAgentShortfallWindowParts { charter: *(charter), model: *(model) },
            )?)
        }
        c::OwnerAgentShortfall::Output { charter, model } => {
            w::OwnerAgentShortfall::Output(w::OwnerAgentShortfallOutput::new(
                limits,
                w::OwnerAgentShortfallOutputParts { charter: *(charter), model: *(model) },
            )?)
        }
        c::OwnerAgentShortfall::Turns => w::OwnerAgentShortfall::Turns,
        c::OwnerAgentShortfall::Spend => w::OwnerAgentShortfall::Spend,
        c::OwnerAgentShortfall::Time => w::OwnerAgentShortfall::Time,
        c::OwnerAgentShortfall::Waiting => w::OwnerAgentShortfall::Waiting,
        c::OwnerAgentShortfall::Checks => w::OwnerAgentShortfall::Checks,
        c::OwnerAgentShortfall::Message => w::OwnerAgentShortfall::Message,
        c::OwnerAgentShortfall::Inbox => w::OwnerAgentShortfall::Inbox,
        c::OwnerAgentShortfall::Unacknowledged => w::OwnerAgentShortfall::Unacknowledged,
    })
}

pub(super) fn wire_task_wait_program(
    value: &c::OpenProgram,
    limits: &w::Limits,
) -> Result<w::TaskWaitProgram, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OpenProgram::Model => w::TaskWaitProgram::Model,
        c::OpenProgram::Source { account, source } => w::TaskWaitProgram::Source(w::TaskWaitProgramSource::new(
            limits,
            w::TaskWaitProgramSourceParts { account: *(account), source: *(source) },
        )?),
    })
}

pub(super) fn wire_dependency(value: &c::Dependency, limits: &w::Limits) -> Result<w::Dependency, w::Problem> {
    w::Dependency::new(
        limits,
        w::DependencyParts {
            number: value.number,
            pending: value.pending,
            added: value.added.as_ref().map(|item| wire_dependency_added(item, limits)).transpose()?,
            keeps: value.keeps,
            status: value.status.as_ref().map(|item| wire_owner_task_status(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_dependency_added(
    value: &c::DependencyAdded,
    limits: &w::Limits,
) -> Result<w::DependencyAdded, w::Problem> {
    w::DependencyAdded::new(
        limits,
        w::DependencyAddedParts { at: value.at.as_nanos(), by: wire_owner_requester(&value.by, limits)? },
    )
}

pub(super) fn wire_owner_proposal_history(
    value: &c::OwnerProposalHistory,
    limits: &w::Limits,
) -> Result<w::OwnerProposalHistory, w::Problem> {
    w::OwnerProposalHistory::new(
        limits,
        w::OwnerProposalHistoryParts {
            action: wire_owner_retained_action(&value.action, limits)?,
            number: value.number,
            proposer: value.proposer,
            as_holder: value.as_holder,
            state: wire_owner_proposal_history_state(&value.state, limits)?,
        },
    )
}

pub(super) fn wire_owner_retained_action(
    value: &c::OwnerRetainedAction,
    limits: &w::Limits,
) -> Result<w::OwnerRetainedAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRetainedAction::Settings(value) => {
            w::OwnerRetainedAction::Settings(wire_owner_retained_action_settings(value, limits)?)
        }
        c::OwnerRetainedAction::Batch(value) => w::OwnerRetainedAction::Batch(w::OwnerRetainedActionBatch::new(
            limits,
            w::OwnerRetainedActionBatchParts {
                value: pieces(
                    (value).iter().map(|item| wire_proposed_task(item, limits)),
                    w::ProposedTask::measure,
                    w::ProposedTask::encode,
                )?,
            },
        )?),
        c::OwnerRetainedAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::OwnerRetainedAction::Effect(w::OwnerRetainedActionEffect::new(
                limits,
                w::OwnerRetainedActionEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    attempt: *(attempt),
                    completion: *(completion),
                    position: *(position),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerRetainedAction::ProcedureEffect { part, authority, step, purpose } => {
            w::OwnerRetainedAction::ProcedureEffect(w::OwnerRetainedActionProcedureEffect::new(
                limits,
                w::OwnerRetainedActionProcedureEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    step: *(step),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerRetainedAction::Amend { task, change } => {
            w::OwnerRetainedAction::Amend(Box::new(w::OwnerRetainedActionAmend::new(
                limits,
                w::OwnerRetainedActionAmendParts {
                    task: *(task),
                    change: wire_owner_retained_amendment(change, limits)?,
                },
            )?))
        }
        c::OwnerRetainedAction::Widen { task, authority } => {
            w::OwnerRetainedAction::Widen(w::OwnerRetainedActionWiden::new(
                limits,
                w::OwnerRetainedActionWidenParts {
                    task: *(task),
                    authority: wire_owner_authority_document(authority, limits)?,
                },
            )?)
        }
        c::OwnerRetainedAction::Release { task } => w::OwnerRetainedAction::Release(
            w::OwnerRetainedActionRelease::new(limits, w::OwnerRetainedActionReleaseParts { task: *(task) })?,
        ),
    })
}

pub(super) fn wire_owner_retained_action_settings(
    value: &c::OwnerRetainedActionSettings,
    limits: &w::Limits,
) -> Result<w::OwnerRetainedActionSettings, w::Problem> {
    w::OwnerRetainedActionSettings::new(
        limits,
        w::OwnerRetainedActionSettingsParts {
            against: value.against,
            items: list(value.items.iter().map(|item| wire_revision_item_head(item, limits)))?,
            made: value.made.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(super) fn wire_owner_retained_amendment(
    value: &c::OwnerRetainedAmendment,
    limits: &w::Limits,
) -> Result<w::OwnerRetainedAmendment, w::Problem> {
    w::OwnerRetainedAmendment::new(
        limits,
        w::OwnerRetainedAmendmentParts {
            addressed: value.addressed.as_ref().map(|item| wire_address_change(item, limits)).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            release: value.release.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            spec: value.spec.as_ref().map(|item| wire_specification_document(item, limits)).transpose()?,
            authority: value.authority.as_ref().map(|item| wire_owner_authority_document(item, limits)).transpose()?,
            wake: value.wake.as_ref().map(|item| wire_owner_wake_policy(item, limits)).transpose()?,
            dependencies: value
                .dependencies
                .as_ref()
                .map(|item| list((item).iter().map(|item| Ok::<_, w::Problem>(*(item)))))
                .transpose()?,
            reason: wire_document_text(&value.reason, limits)?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_address_change(
    value: &c::AddressChange,
    limits: &w::Limits,
) -> Result<w::AddressChange, w::Problem> {
    w::AddressChange::new(
        limits,
        w::AddressChangeParts {
            revision: wire_revision(&value.revision, limits)?,
            address: wire_person_address(&value.address, limits)?,
        },
    )
}

pub(super) fn wire_owner_proposal_history_state(
    value: &c::OwnerProposalHistoryState,
    limits: &w::Limits,
) -> Result<w::OwnerProposalHistoryState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProposalHistoryState::Pending { holder, since } => {
            w::OwnerProposalHistoryState::Pending(w::OwnerProposalHistoryStatePending::new(
                limits,
                w::OwnerProposalHistoryStatePendingParts {
                    holder: wire_owner_history_holder(holder, limits)?,
                    since: (since).as_nanos(),
                },
            )?)
        }
        c::OwnerProposalHistoryState::Accepted { by } => {
            w::OwnerProposalHistoryState::Accepted(w::OwnerProposalHistoryStateAccepted::new(
                limits,
                w::OwnerProposalHistoryStateAcceptedParts { by: wire_owner_requester(by, limits)? },
            )?)
        }
        c::OwnerProposalHistoryState::Rejected { by, reason } => {
            w::OwnerProposalHistoryState::Rejected(w::OwnerProposalHistoryStateRejected::new(
                limits,
                w::OwnerProposalHistoryStateRejectedParts {
                    by: wire_owner_requester(by, limits)?,
                    reason: wire_document_text(reason, limits)?,
                },
            )?)
        }
        c::OwnerProposalHistoryState::Withdrawn => w::OwnerProposalHistoryState::Withdrawn,
    })
}

pub(super) fn wire_owner_history_holder(
    value: &c::OwnerHistoryHolder,
    limits: &w::Limits,
) -> Result<w::OwnerHistoryHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHistoryHolder::Task(value) => w::OwnerHistoryHolder::Task(w::OwnerHistoryHolderTask::new(
            limits,
            w::OwnerHistoryHolderTaskParts { value: *(value) },
        )?),
        c::OwnerHistoryHolder::Person(value) => w::OwnerHistoryHolder::Person(w::OwnerHistoryHolderPerson::new(
            limits,
            w::OwnerHistoryHolderPersonParts { value: *(value) },
        )?),
        c::OwnerHistoryHolder::Policy { project, kind } => {
            w::OwnerHistoryHolder::Policy(w::OwnerHistoryHolderPolicy::new(
                limits,
                w::OwnerHistoryHolderPolicyParts {
                    project: *(project),
                    kind: wire_owner_message_proposal_kind(kind, limits)?,
                },
            )?)
        }
    })
}

pub(super) fn wire_owner_message_proposal_kind(
    value: &c::OwnerMessageProposalKind,
    _limits: &w::Limits,
) -> Result<w::OwnerMessageProposalKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerMessageProposalKind::Settings => w::OwnerMessageProposalKind::Settings,
        c::OwnerMessageProposalKind::Batch => w::OwnerMessageProposalKind::Batch,
        c::OwnerMessageProposalKind::Effect => w::OwnerMessageProposalKind::Effect,
        c::OwnerMessageProposalKind::Amend => w::OwnerMessageProposalKind::Amend,
        c::OwnerMessageProposalKind::Widen => w::OwnerMessageProposalKind::Widen,
        c::OwnerMessageProposalKind::Release => w::OwnerMessageProposalKind::Release,
    })
}

pub(super) fn wire_owner_person_proposal(
    value: &c::OwnerPersonProposal,
    limits: &w::Limits,
) -> Result<w::OwnerPersonProposal, w::Problem> {
    w::OwnerPersonProposal::new(
        limits,
        w::OwnerPersonProposalParts {
            number: value.number,
            proposer: value.proposer,
            project: value.project,
            goal: wire_owner_person_proposal_goal(&value.goal, limits)?,
            state: wire_owner_person_proposal_state(&value.state, limits)?,
        },
    )
}

pub(super) fn wire_owner_person_proposal_goal(
    value: &c::OwnerPersonProposalGoal,
    limits: &w::Limits,
) -> Result<w::OwnerPersonProposalGoal, w::Problem> {
    w::OwnerPersonProposalGoal::new(
        limits,
        w::OwnerPersonProposalGoalParts {
            number: value.number,
            project: value.project,
            executor: wire_owner_executor(&value.executor, limits)?,
            spec: wire_specification(&value.spec, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            authority: wire_owner_authority(&value.authority, limits)?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            funder: wire_funder(&value.funder, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            holdings: pieces(
                value.holdings.iter().map(|item| wire_owner_goal_holding(item, limits)),
                w::OwnerGoalHolding::measure,
                w::OwnerGoalHolding::encode,
            )?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(super) fn wire_specification(value: &c::Specification, limits: &w::Limits) -> Result<w::Specification, w::Problem> {
    w::Specification::new(
        limits,
        w::SpecificationParts {
            words: bytes::copy_of(value.words.bytes()),
            parameters: list(value.parameters.iter().map(|item| wire_parameter(item, limits)))?,
            inputs: list(value.inputs.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            asks: value.asks,
        },
    )
}

pub(super) fn wire_parameter(value: &c::Parameter, limits: &w::Limits) -> Result<w::Parameter, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Parameter::Number { name, value } => w::Parameter::Number(w::ParameterNumber::new(
            limits,
            w::ParameterNumberParts { name: *(name), value: *(value) },
        )?),
        c::Parameter::Bytes { name, value } => w::Parameter::Bytes(w::ParameterBytes::new(
            limits,
            w::ParameterBytesParts { name: *(name), value: bytes::copy_of((value).bytes()) },
        )?),
        c::Parameter::Resource { name, connector, resource } => w::Parameter::Resource(w::ParameterResource::new(
            limits,
            w::ParameterResourceParts { name: *(name), connector: *(connector), resource: *(resource) },
        )?),
    })
}

pub(super) fn wire_owner_goal_holding(
    value: &c::OwnerGoalHolding,
    limits: &w::Limits,
) -> Result<w::OwnerGoalHolding, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerGoalHolding::Write { connector, path, kind } => {
            w::OwnerGoalHolding::Write(w::OwnerGoalHoldingWrite::new(
                limits,
                w::OwnerGoalHoldingWriteParts {
                    connector: *(connector),
                    path: list((path).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
                    kind: *(kind),
                },
            )?)
        }
        c::OwnerGoalHolding::Slot { connector, path, kind } => w::OwnerGoalHolding::Slot(w::OwnerGoalHoldingSlot::new(
            limits,
            w::OwnerGoalHoldingSlotParts {
                connector: *(connector),
                path: list((path).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
                kind: *(kind),
            },
        )?),
    })
}

pub(super) fn wire_owner_person_proposal_state(
    value: &c::OwnerPersonProposalState,
    limits: &w::Limits,
) -> Result<w::OwnerPersonProposalState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPersonProposalState::Pending { since } => {
            w::OwnerPersonProposalState::Pending(w::OwnerPersonProposalStatePending::new(
                limits,
                w::OwnerPersonProposalStatePendingParts { since: (since).as_nanos() },
            )?)
        }
        c::OwnerPersonProposalState::Accepted { by } => {
            w::OwnerPersonProposalState::Accepted(w::OwnerPersonProposalStateAccepted::new(
                limits,
                w::OwnerPersonProposalStateAcceptedParts { by: wire_owner_requester(by, limits)? },
            )?)
        }
        c::OwnerPersonProposalState::Rejected { by, reason, message, at } => {
            w::OwnerPersonProposalState::Rejected(w::OwnerPersonProposalStateRejected::new(
                limits,
                w::OwnerPersonProposalStateRejectedParts {
                    by: wire_owner_requester(by, limits)?,
                    reason: bytes::copy_of((reason).bytes()),
                    message: *(message),
                    at: (at).as_nanos(),
                },
            )?)
        }
    })
}

pub(super) fn wire_owner_proposal_decision(
    value: &c::OwnerProposalDecision,
    limits: &w::Limits,
) -> Result<w::OwnerProposalDecision, w::Problem> {
    w::OwnerProposalDecision::new(
        limits,
        w::OwnerProposalDecisionParts {
            project: value.project,
            proposer: wire_proposer(&value.proposer, limits)?,
            proposal: value.proposal,
            kind: wire_owner_message_proposal_kind(&value.kind, limits)?,
            by: value.by,
            at: value.at.as_nanos(),
            choice: wire_owner_proposal_choice(&value.choice, limits)?,
            created: value
                .created
                .as_ref()
                .map(|item| {
                    w::OwnerProposalDecisionCreated::new(
                        limits,
                        w::OwnerProposalDecisionCreatedParts { first: item.0, last: item.1 },
                    )
                })
                .transpose()?,
        },
    )
}

pub(super) fn wire_owner_proposal_choice(
    value: &c::OwnerProposalChoice,
    _limits: &w::Limits,
) -> Result<w::OwnerProposalChoice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProposalChoice::Accepted => w::OwnerProposalChoice::Accepted,
        c::OwnerProposalChoice::Rejected => w::OwnerProposalChoice::Rejected,
        c::OwnerProposalChoice::Passed => w::OwnerProposalChoice::Passed,
        c::OwnerProposalChoice::Withdrawn => w::OwnerProposalChoice::Withdrawn,
        c::OwnerProposalChoice::Stale => w::OwnerProposalChoice::Stale,
    })
}

pub(super) fn wire_owner_escalation_decision_record(
    value: &c::OwnerEscalationDecisionRecord,
    limits: &w::Limits,
) -> Result<w::OwnerEscalationDecisionRecord, w::Problem> {
    w::OwnerEscalationDecisionRecord::new(
        limits,
        w::OwnerEscalationDecisionRecordParts {
            project: value.project,
            requester: value.requester,
            task: value.task,
            revision: value.revision,
            by: value.by,
            at: value.at.as_nanos(),
            decision: wire_owner_escalation_decision(&value.decision, limits)?,
        },
    )
}

pub(super) fn wire_owner_escalation_decision(
    value: &c::OwnerEscalationDecision,
    limits: &w::Limits,
) -> Result<w::OwnerEscalationDecision, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerEscalationDecision::Release => w::OwnerEscalationDecision::Release,
        c::OwnerEscalationDecision::Reject { reason } => {
            w::OwnerEscalationDecision::Reject(w::OwnerEscalationDecisionReject::new(
                limits,
                w::OwnerEscalationDecisionRejectParts { reason: bytes::copy_of((reason).bytes()) },
            )?)
        }
        c::OwnerEscalationDecision::Pass => w::OwnerEscalationDecision::Pass,
    })
}

pub(super) fn wire_owner_milestone_line(
    value: &c::OwnerMilestoneLine,
    limits: &w::Limits,
) -> Result<w::OwnerMilestoneLine, w::Problem> {
    w::OwnerMilestoneLine::new(
        limits,
        w::OwnerMilestoneLineParts {
            task: value.task,
            position: value.position,
            phase: wire_owner_milestone_phase(&value.phase, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_milestone_phase(
    value: &c::OwnerMilestonePhase,
    limits: &w::Limits,
) -> Result<w::OwnerMilestonePhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerMilestonePhase::Waiting => w::OwnerMilestonePhase::Waiting,
        c::OwnerMilestonePhase::Active(value) => w::OwnerMilestonePhase::Active(w::OwnerMilestonePhaseActive::new(
            limits,
            w::OwnerMilestonePhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::OwnerMilestonePhase::Closing { stage, ending } => {
            w::OwnerMilestonePhase::Closing(w::OwnerMilestonePhaseClosing::new(
                limits,
                w::OwnerMilestonePhaseClosingParts {
                    stage: wire_closing_stage(stage, limits)?,
                    ending: facts::owner_chip_ending(ending, limits)?,
                },
            )?)
        }
        c::OwnerMilestonePhase::Held { before, why, source, ending } => {
            w::OwnerMilestonePhase::Held(w::OwnerMilestonePhaseHeld::new(
                limits,
                w::OwnerMilestonePhaseHeldParts {
                    before: wire_owner_prior_phase(before, limits)?,
                    why: wire_hold_category(why, limits)?,
                    source: (source).as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
                    ending: (ending).as_ref().map(|item| facts::owner_chip_ending(item, limits)).transpose()?,
                },
            )?)
        }
        c::OwnerMilestonePhase::Ended(value) => w::OwnerMilestonePhase::Ended(w::OwnerMilestonePhaseEnded::new(
            limits,
            w::OwnerMilestonePhaseEndedParts { value: facts::owner_chip_ending(value, limits)? },
        )?),
    })
}

pub(super) fn wire_question(value: &c::Question, limits: &w::Limits) -> Result<w::Question, w::Problem> {
    w::Question::new(
        limits,
        w::QuestionParts {
            task: facts::chip(&value.task, limits)?,
            from: value.from.as_ref().map(|item| wire_owner_requester(item, limits)).transpose()?,
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

pub(super) fn wire_owner_activity_line(
    value: &c::OwnerActivityLine,
    limits: &w::Limits,
) -> Result<w::OwnerActivityLine, w::Problem> {
    w::OwnerActivityLine::new(
        limits,
        w::OwnerActivityLineParts {
            task: value.task,
            position: value.position,
            at: value.at.as_nanos(),
            line: wire_owner_activity(&value.line, limits)?,
        },
    )
}

pub(super) fn wire_owner_activity(
    value: &c::OwnerActivity,
    limits: &w::Limits,
) -> Result<w::OwnerActivity, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerActivity::Words { message, by, kind, words } => w::OwnerActivity::Words(w::OwnerActivityWords::new(
            limits,
            w::OwnerActivityWordsParts {
                message: *(message),
                by: wire_owner_requester(by, limits)?,
                kind: wire_owner_message_kind(kind, limits)?,
                words: bytes::copy_of((words).bytes()),
            },
        )?),
        c::OwnerActivity::Heard(value) => w::OwnerActivity::Heard(w::OwnerActivityHeard::new(
            limits,
            w::OwnerActivityHeardParts { value: wire_owner_word(value, limits)? },
        )?),
        c::OwnerActivity::Notified { recipient, word } => w::OwnerActivity::Notified(w::OwnerActivityNotified::new(
            limits,
            w::OwnerActivityNotifiedParts {
                recipient: wire_owner_requester(recipient, limits)?,
                word: wire_owner_word(word, limits)?,
            },
        )?),
        c::OwnerActivity::RunBegan { attempt, why, resumed } => {
            w::OwnerActivity::RunBegan(w::OwnerActivityRunBegan::new(
                limits,
                w::OwnerActivityRunBeganParts {
                    attempt: *(attempt),
                    why: wire_owner_run_why(why, limits)?,
                    resumed: *(resumed),
                },
            )?)
        }
        c::OwnerActivity::RunEnded { attempt, end, spent } => {
            w::OwnerActivity::RunEnded(w::OwnerActivityRunEnded::new(
                limits,
                w::OwnerActivityRunEndedParts {
                    attempt: *(attempt),
                    end: wire_owner_run_end(end, limits)?,
                    spent: *(spent),
                },
            )?)
        }
        c::OwnerActivity::Turn { reply, attempt, turn, read } => w::OwnerActivity::Turn(w::OwnerActivityTurn::new(
            limits,
            w::OwnerActivityTurnParts {
                reply: *(reply),
                attempt: *(attempt),
                turn: *(turn),
                read: (read).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::OwnerActivity::Decided { revision } => w::OwnerActivity::Decided(w::OwnerActivityDecided::new(
            limits,
            w::OwnerActivityDecidedParts { revision: *(revision) },
        )?),
        c::OwnerActivity::Milestone { position } => w::OwnerActivity::Milestone(w::OwnerActivityMilestone::new(
            limits,
            w::OwnerActivityMilestoneParts { position: *(position) },
        )?),
        c::OwnerActivity::Kept { attempt, pending, question } => w::OwnerActivity::Kept(w::OwnerActivityKept::new(
            limits,
            w::OwnerActivityKeptParts {
                attempt: *(attempt),
                pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                question: (question).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
    })
}

pub(super) fn wire_owner_message_kind(
    value: &c::OwnerMessageKind,
    limits: &w::Limits,
) -> Result<w::OwnerMessageKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerMessageKind::QuestionOwed => w::OwnerMessageKind::QuestionOwed,
        c::OwnerMessageKind::SettingsDecision { proposal, outcome } => {
            w::OwnerMessageKind::SettingsDecision(w::OwnerMessageKindSettingsDecision::new(
                limits,
                w::OwnerMessageKindSettingsDecisionParts {
                    proposal: *(proposal),
                    outcome: wire_settings_outcome(outcome, limits)?,
                },
            )?)
        }
        c::OwnerMessageKind::Escalation { task, revision } => {
            w::OwnerMessageKind::Escalation(w::OwnerMessageKindEscalation::new(
                limits,
                w::OwnerMessageKindEscalationParts { task: *(task), revision: *(revision) },
            )?)
        }
        c::OwnerMessageKind::Proposal { proposer, proposal, kind } => {
            w::OwnerMessageKind::Proposal(w::OwnerMessageKindProposal::new(
                limits,
                w::OwnerMessageKindProposalParts {
                    proposer: *(proposer),
                    proposal: *(proposal),
                    kind: wire_owner_message_proposal_kind(kind, limits)?,
                },
            )?)
        }
        c::OwnerMessageKind::PersonStalled { task } => w::OwnerMessageKind::PersonStalled(
            w::OwnerMessageKindPersonStalled::new(limits, w::OwnerMessageKindPersonStalledParts { task: *(task) })?,
        ),
        c::OwnerMessageKind::ProposalDecision { proposal, accepted } => {
            w::OwnerMessageKind::ProposalDecision(w::OwnerMessageKindProposalDecision::new(
                limits,
                w::OwnerMessageKindProposalDecisionParts { proposal: *(proposal), accepted: *(accepted) },
            )?)
        }
        c::OwnerMessageKind::QuestionAnswered { question } => w::OwnerMessageKind::QuestionAnswered(
            w::OwnerMessageKindAnswer::new(limits, w::OwnerMessageKindAnswerParts { question: *(question) })?,
        ),
        c::OwnerMessageKind::QuestionEnded { question } => w::OwnerMessageKind::QuestionEnded(
            w::OwnerMessageKindAnswer::new(limits, w::OwnerMessageKindAnswerParts { question: *(question) })?,
        ),
        c::OwnerMessageKind::Words => w::OwnerMessageKind::Words,
        c::OwnerMessageKind::EffectNotMade { entry } => w::OwnerMessageKind::EffectNotMade(
            w::OwnerMessageKindEffectNotMade::new(limits, w::OwnerMessageKindEffectNotMadeParts { entry: *(entry) })?,
        ),
        c::OwnerMessageKind::Amendment { revision } => w::OwnerMessageKind::Amendment(
            w::OwnerMessageKindAmendment::new(limits, w::OwnerMessageKindAmendmentParts { revision: *(revision) })?,
        ),
        c::OwnerMessageKind::Question => w::OwnerMessageKind::Question,
        c::OwnerMessageKind::Answer { question } => w::OwnerMessageKind::Answer(w::OwnerMessageKindAnswer::new(
            limits,
            w::OwnerMessageKindAnswerParts { question: *(question) },
        )?),
        c::OwnerMessageKind::Notice { subscription, target, state } => {
            w::OwnerMessageKind::Notice(w::OwnerMessageKindNotice::new(
                limits,
                w::OwnerMessageKindNoticeParts {
                    subscription: *(subscription),
                    target: *(target),
                    state: wire_owner_notice_state(state, limits)?,
                },
            )?)
        }
        c::OwnerMessageKind::Timer { subscription } => w::OwnerMessageKind::Timer(w::OwnerMessageKindTimer::new(
            limits,
            w::OwnerMessageKindTimerParts { subscription: *(subscription) },
        )?),
        c::OwnerMessageKind::News { subscription, class } => w::OwnerMessageKind::News(w::OwnerMessageKindNews::new(
            limits,
            w::OwnerMessageKindNewsParts {
                subscription: *(subscription),
                class: wire_owner_news_class(class, limits)?,
            },
        )?),
        c::OwnerMessageKind::Result(value) => w::OwnerMessageKind::Result(w::OwnerMessageKindResult::new(
            limits,
            w::OwnerMessageKindResultParts { value: wire_owner_word_result(value, limits)? },
        )?),
        c::OwnerMessageKind::Met { dependency, removed } => w::OwnerMessageKind::Met(w::OwnerMessageKindMet::new(
            limits,
            w::OwnerMessageKindMetParts { dependency: *(dependency), removed: *(removed) },
        )?),
    })
}

pub(super) fn wire_settings_outcome(
    value: &c::SettingsOutcome,
    limits: &w::Limits,
) -> Result<w::SettingsOutcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingsOutcome::Accepted { revision } => w::SettingsOutcome::Accepted(w::SettingsAccepted::new(
            limits,
            w::SettingsAcceptedParts { revision: *(revision) },
        )?),
        c::SettingsOutcome::Rejected => w::SettingsOutcome::Rejected,
        c::SettingsOutcome::Stale { newest } => {
            w::SettingsOutcome::Stale(w::SettingsStale::new(limits, w::SettingsStaleParts { newest: *(newest) })?)
        }
    })
}

pub(super) fn wire_owner_notice_state(
    value: &c::OwnerNoticeState,
    _limits: &w::Limits,
) -> Result<w::OwnerNoticeState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerNoticeState::Held => w::OwnerNoticeState::Held,
        c::OwnerNoticeState::Done => w::OwnerNoticeState::Done,
        c::OwnerNoticeState::Failed => w::OwnerNoticeState::Failed,
        c::OwnerNoticeState::Cancelled => w::OwnerNoticeState::Cancelled,
    })
}

pub(super) fn wire_owner_news_class(
    value: &c::OwnerNewsClass,
    _limits: &w::Limits,
) -> Result<w::OwnerNewsClass, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerNewsClass::Wakes => w::OwnerNewsClass::Wakes,
        c::OwnerNewsClass::Kept => w::OwnerNewsClass::Kept,
        c::OwnerNewsClass::Dropped => w::OwnerNewsClass::Dropped,
    })
}

pub(super) fn wire_owner_word_result(
    value: &c::OwnerWordResult,
    limits: &w::Limits,
) -> Result<w::OwnerWordResult, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerWordResult::Approved { state, words_present } => {
            w::OwnerWordResult::Approved(w::OwnerWordResultApproved::new(
                limits,
                w::OwnerWordResultApprovedParts {
                    state: bytes::copy_of((state).bytes()),
                    words_present: *(words_present),
                },
            )?)
        }
        c::OwnerWordResult::Report => w::OwnerWordResult::Report,
        c::OwnerWordResult::Verdict { code, fields } => w::OwnerWordResult::Verdict(w::OwnerWordResultVerdict::new(
            limits,
            w::OwnerWordResultVerdictParts {
                code: *(code),
                fields: list((fields).iter().map(|item| wire_result_field(item, limits)))?,
            },
        )?),
        c::OwnerWordResult::Change { connector, kind, resource } => {
            w::OwnerWordResult::Change(w::OwnerWordResultChange::new(
                limits,
                w::OwnerWordResultChangeParts { connector: *(connector), kind: *(kind), resource: *(resource) },
            )?)
        }
        c::OwnerWordResult::Failed => w::OwnerWordResult::Failed,
        c::OwnerWordResult::Cancelled => w::OwnerWordResult::Cancelled,
    })
}

pub(super) fn wire_result_field(value: &c::ResultField, limits: &w::Limits) -> Result<w::ResultField, w::Problem> {
    w::ResultField::new(
        limits,
        w::ResultFieldParts { name: bytes::copy_of(&value.name), value: bytes::copy_of(&value.value) },
    )
}

pub(super) fn wire_owner_word(value: &c::OwnerWord, limits: &w::Limits) -> Result<w::OwnerWord, w::Problem> {
    w::OwnerWord::new(
        limits,
        w::OwnerWordParts {
            number: value.number,
            from: wire_owner_requester(&value.from, limits)?,
            kind: wire_owner_message_kind(&value.kind, limits)?,
            words: bytes::copy_of(value.words.bytes()),
            at: value.at.as_nanos(),
            hits: value.hits,
            eligible: value.eligible,
        },
    )
}

pub(super) fn wire_owner_run_why(value: &c::OwnerRunWhy, limits: &w::Limits) -> Result<w::OwnerRunWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRunWhy::Made => w::OwnerRunWhy::Made,
        c::OwnerRunWhy::Ready => w::OwnerRunWhy::Ready,
        c::OwnerRunWhy::Message { message } => w::OwnerRunWhy::Message(w::OwnerRunWhyMessage::new(
            limits,
            w::OwnerRunWhyMessageParts { message: *(message) },
        )?),
        c::OwnerRunWhy::Released => w::OwnerRunWhy::Released,
        c::OwnerRunWhy::Retry { tries } => w::OwnerRunWhy::Retry(w::OwnerRunWhyRetry::new(
            limits,
            w::OwnerRunWhyRetryParts { tries: wire_owner_failure_tries(tries, limits)? },
        )?),
        c::OwnerRunWhy::Narrowed => w::OwnerRunWhy::Narrowed,
        c::OwnerRunWhy::Changed => w::OwnerRunWhy::Changed,
    })
}

pub(super) fn wire_owner_run_end(value: &c::OwnerRunEnd, limits: &w::Limits) -> Result<w::OwnerRunEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRunEnd::Parked => w::OwnerRunEnd::Parked,
        c::OwnerRunEnd::Finished => w::OwnerRunEnd::Finished,
        c::OwnerRunEnd::Cancelled => w::OwnerRunEnd::Cancelled,
        c::OwnerRunEnd::Failed { class } => w::OwnerRunEnd::Failed(w::OwnerRunEndFailed::new(
            limits,
            w::OwnerRunEndFailedParts { class: wire_hold_failure(class, limits)? },
        )?),
        c::OwnerRunEnd::Refused => w::OwnerRunEnd::Refused,
    })
}

pub(super) fn wire_owner_person_task(
    value: &c::OwnerPersonTask,
    limits: &w::Limits,
) -> Result<w::OwnerPersonTask, w::Problem> {
    w::OwnerPersonTask::new(
        limits,
        w::OwnerPersonTaskParts {
            task: facts::chip(&value.task, limits)?,
            revision: wire_revision(&value.revision, limits)?,
            requester: wire_owner_requester(&value.requester, limits)?,
            spec: wire_specification_document(&value.spec, limits)?,
            address: wire_person_address(&value.address, limits)?,
            taken_by: value.taken_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            since: value.since.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            stall: value.stall.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            results: pieces(
                value.results.iter().map(|item| wire_owner_shown_result(item, limits)),
                w::OwnerShownResult::measure,
                w::OwnerShownResult::encode,
            )?,
            cut: value.cut,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
        },
    )
}

pub(super) fn wire_owner_shown_result(
    value: &c::OwnerShownResult,
    limits: &w::Limits,
) -> Result<w::OwnerShownResult, w::Problem> {
    w::OwnerShownResult::new(
        limits,
        w::OwnerShownResultParts {
            task: wire_owner_task_link(&value.task, limits)?,
            result: value.result.as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_me(value: &c::Me, limits: &w::Limits) -> Result<w::Me, w::Problem> {
    w::Me::new(
        limits,
        w::MeParts {
            party: wire_me_party(&value.party, limits)?,
            projects: pieces(
                value.projects.iter().map(|item| wire_me_project(item, limits)),
                w::MeProject::measure,
                w::MeProject::encode,
            )?,
            needs: wire_needs(&value.needs, limits)?,
            needs_total: value.needs_total,
            updates: wire_count(&value.updates, limits)?,
            results_seen: value.results_seen,
            cut: value.cut,
            retention: value.retention,
            deployment_role: value.deployment_role.as_ref().map(|item| wire_held_role(item, limits)).transpose()?,
            providers: pieces(
                value.providers.iter().map(|item| wire_provider_offer(item, limits)),
                w::ProviderOffer::measure,
                w::ProviderOffer::encode,
            )?,
            providers_cut: value.providers_cut,
            key_bytes: value.key_bytes,
            redirect_bytes: value.redirect_bytes,
        },
    )
}

pub(super) fn wire_me_party(value: &c::MeParty, limits: &w::Limits) -> Result<w::MeParty, w::Problem> {
    w::MeParty::new(
        limits,
        w::MePartyParts {
            number: value.number,
            kind: wire_party_kind(&value.kind, limits)?,
            login: wire_document_text(&value.login, limits)?,
            name: wire_document_text(&value.name, limits)?,
        },
    )
}

pub(super) fn wire_me_project(value: &c::MeProject, limits: &w::Limits) -> Result<w::MeProject, w::Problem> {
    w::MeProject::new(
        limits,
        w::MeProjectParts {
            project: value.project,
            name: wire_document_text(&value.name, limits)?,
            role: wire_held_role(&value.role, limits)?,
            pool: value.pool.as_ref().map(|item| wire_pool_ledger(item, limits)).transpose()?,
            templates: pieces(
                value.templates.iter().map(|item| wire_template(item, limits)),
                w::Template::measure,
                w::Template::encode,
            )?,
            creating: value.creating.as_ref().map(|item| wire_creating_bounds(item, limits)).transpose()?,
            models: list(value.models.iter().map(|item| wire_me_model(item, limits)))?,
        },
    )
}

pub(super) fn wire_creating_bounds(value: &c::Creating, limits: &w::Limits) -> Result<w::CreatingBounds, w::Problem> {
    w::CreatingBounds::new(
        limits,
        w::CreatingBoundsParts {
            within: wire_bounds_create(&value.within, limits)?,
            beyond: wire_beyond(&value.beyond, limits)?,
        },
    )
}

pub(super) fn wire_me_model(value: &c::MeModel, limits: &w::Limits) -> Result<w::MeModel, w::Problem> {
    w::MeModel::new(
        limits,
        w::MeModelParts {
            name: bytes::copy_of(value.name.bytes()),
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            window: value.window,
            output: value.output,
            efforts: list(value.efforts.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
            prices: wire_me_prices(&value.prices, limits)?,
            description: value
                .description
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
        },
    )
}

pub(super) fn wire_me_prices(value: &c::ModelPrices, limits: &w::Limits) -> Result<w::MePrices, w::Problem> {
    w::MePrices::new(
        limits,
        w::MePricesParts {
            input: value.input,
            cached: value.cached,
            cache_write: value.cache_write.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            output: value.output,
            unit: value.unit,
        },
    )
}

pub(super) fn wire_needs(value: &c::Needs, limits: &w::Limits) -> Result<w::Needs, w::Problem> {
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

pub(super) fn wire_provider_offer(
    value: &c::ProviderOffer,
    limits: &w::Limits,
) -> Result<w::ProviderOffer, w::Problem> {
    w::ProviderOffer::new(
        limits,
        w::ProviderOfferParts {
            skein_bools: [value.sign_in, value.key, value.programs_cut, value.none, value.signed_in_plan],
            provider: value.provider,
            name: wire_document_text(&value.name, limits)?,
            programs: list(value.programs.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes()))))?,
            manage_usage: value
                .manage_usage
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of(item)))
                .transpose()?,
            key_bytes: value.key_bytes,
        },
    )
}

pub(super) fn wire_archived_turn(value: &c::ArchivedTurn, limits: &w::Limits) -> Result<w::ArchivedTurn, w::Problem> {
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

pub(super) fn wire_turn_name(value: &c::TurnName, limits: &w::Limits) -> Result<w::TurnName, w::Problem> {
    w::TurnName::new(limits, w::TurnNameParts { attempt: value.attempt, number: value.number })
}

pub(super) fn wire_owner_proposal(
    value: &c::OwnerProposal,
    limits: &w::Limits,
) -> Result<w::OwnerProposal, w::Problem> {
    w::OwnerProposal::new(
        limits,
        w::OwnerProposalParts {
            key: wire_proposal_key(&value.key, limits)?,
            project: value.project,
            reason: wire_document_text(&value.reason, limits)?,
            action: wire_owner_proposal_action(&value.action, limits)?,
            as_holder: value.as_holder,
            waiting: wire_owner_proposal_waiting(&value.waiting, limits)?,
            funding: list(value.funding.iter().map(|item| wire_owner_funding_offer(item, limits)))?,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
        },
    )
}

pub(super) fn wire_owner_proposal_action(
    value: &c::OwnerProposalAction,
    limits: &w::Limits,
) -> Result<w::OwnerProposalAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProposalAction::Settings(value) => {
            w::OwnerProposalAction::Settings(Box::new(wire_owner_proposal_action_settings(value, limits)?))
        }
        c::OwnerProposalAction::Batch(value) => w::OwnerProposalAction::Batch(w::OwnerProposalActionBatch::new(
            limits,
            w::OwnerProposalActionBatchParts {
                value: pieces(
                    (value).iter().map(|item| wire_proposed_task(item, limits)),
                    w::ProposedTask::measure,
                    w::ProposedTask::encode,
                )?,
            },
        )?),
        c::OwnerProposalAction::Effect { part, authority, attempt, completion, position, purpose } => {
            w::OwnerProposalAction::Effect(w::OwnerProposalActionEffect::new(
                limits,
                w::OwnerProposalActionEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    attempt: *(attempt),
                    completion: *(completion),
                    position: *(position),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerProposalAction::ProcedureEffect { part, authority, step, purpose } => {
            w::OwnerProposalAction::ProcedureEffect(w::OwnerProposalActionProcedureEffect::new(
                limits,
                w::OwnerProposalActionProcedureEffectParts {
                    part: wire_owner_part(part, limits)?,
                    authority: wire_owner_authority_document(authority, limits)?,
                    step: *(step),
                    purpose: *(purpose),
                },
            )?)
        }
        c::OwnerProposalAction::Amend { task, addressed, before, after, reason, release } => {
            w::OwnerProposalAction::Amend(Box::new(w::OwnerProposalActionAmend::new(
                limits,
                w::OwnerProposalActionAmendParts {
                    task: wire_owner_task_link(task, limits)?,
                    addressed: (addressed).as_ref().map(|item| wire_address_change(item, limits)).transpose()?,
                    before: wire_owner_amendment_state(before, limits)?,
                    after: wire_owner_amendment_state(after, limits)?,
                    reason: wire_document_text(reason, limits)?,
                    release: (release).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                },
            )?))
        }
        c::OwnerProposalAction::Widen { task, before, after } => {
            w::OwnerProposalAction::Widen(w::OwnerProposalActionWiden::new(
                limits,
                w::OwnerProposalActionWidenParts {
                    task: wire_owner_task_link(task, limits)?,
                    before: wire_owner_authority_document(before, limits)?,
                    after: wire_owner_authority_document(after, limits)?,
                },
            )?)
        }
        c::OwnerProposalAction::Release { task, hold } => {
            w::OwnerProposalAction::Release(w::OwnerProposalActionRelease::new(
                limits,
                w::OwnerProposalActionReleaseParts {
                    task: wire_owner_task_link(task, limits)?,
                    hold: (hold).as_ref().map(|item| wire_owner_chip_hold(item, limits)).transpose()?,
                },
            )?)
        }
    })
}

pub(super) fn wire_owner_proposal_action_settings(
    value: &c::OwnerProposalActionSettings,
    limits: &w::Limits,
) -> Result<w::OwnerProposalActionSettings, w::Problem> {
    w::OwnerProposalActionSettings::new(
        limits,
        w::OwnerProposalActionSettingsParts {
            against: value.against,
            items: list(value.items.iter().map(|item| wire_revision_item_head(item, limits)))?,
            review: value.review.as_ref().map(|item| wire_settings_review(item, limits)).transpose()?,
            made: value.made.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(super) fn wire_settings_review(
    value: &c::SettingsReview,
    limits: &w::Limits,
) -> Result<w::SettingsReview, w::Problem> {
    w::SettingsReview::new(
        limits,
        w::SettingsReviewParts {
            newest: value.newest,
            items: pieces(
                value.items.iter().map(|item| wire_review_item(item, limits)),
                w::ReviewItem::measure,
                w::ReviewItem::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_review_item(value: &c::ReviewItem, limits: &w::Limits) -> Result<w::ReviewItem, w::Problem> {
    w::ReviewItem::new(
        limits,
        w::ReviewItemParts {
            setting: wire_setting(&value.setting, limits)?,
            edit: wire_revision_item_edit(&value.edit, limits)?,
            now: wire_review_value(&value.now, limits)?,
            origin: wire_setting_origin(&value.origin, limits)?,
            would_be: wire_review_value(&value.would_be, limits)?,
        },
    )
}

pub(super) fn wire_review_value(value: &c::ReviewValue, limits: &w::Limits) -> Result<w::ReviewValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ReviewValue::Value(value) => w::ReviewValue::Value(wire_setting_set(value, limits)?),
        c::ReviewValue::TooLarge(value) => w::ReviewValue::TooLarge(wire_size(value, limits)?),
        c::ReviewValue::None => w::ReviewValue::None,
    })
}

pub(super) fn wire_setting_origin(
    value: &c::SettingOrigin,
    limits: &w::Limits,
) -> Result<w::SettingOrigin, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SettingOrigin::Default(value) => w::SettingOrigin::Default(wire_origin_revision(value, limits)?),
        c::SettingOrigin::Change(value) => w::SettingOrigin::Change(wire_origin_revision(value, limits)?),
        c::SettingOrigin::NotOffered => w::SettingOrigin::NotOffered,
    })
}

pub(super) fn wire_origin_revision(
    value: &c::OriginRevision,
    limits: &w::Limits,
) -> Result<w::OriginRevision, w::Problem> {
    w::OriginRevision::new(limits, w::OriginRevisionParts { revision: value.revision })
}

pub(super) fn wire_owner_amendment_state(
    value: &c::OwnerAmendmentState,
    limits: &w::Limits,
) -> Result<w::OwnerAmendmentState, w::Problem> {
    w::OwnerAmendmentState::new(
        limits,
        w::OwnerAmendmentStateParts {
            addressed: value.addressed.as_ref().map(|item| wire_person_address(item, limits)).transpose()?,
            taken_by: value.taken_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            instructions: value.instructions.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            spec: wire_specification_document(&value.spec, limits)?,
            authority: wire_owner_authority_document(&value.authority, limits)?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            dependencies: list(value.dependencies.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_proposal_waiting(
    value: &c::OwnerProposalWaiting,
    limits: &w::Limits,
) -> Result<w::OwnerProposalWaiting, w::Problem> {
    w::OwnerProposalWaiting::new(
        limits,
        w::OwnerProposalWaitingParts {
            holder: wire_owner_history_holder(&value.holder, limits)?,
            since: value.since.as_nanos(),
        },
    )
}

pub(super) fn wire_owner_funding_offer(
    value: &c::OwnerFundingOffer,
    limits: &w::Limits,
) -> Result<w::OwnerFundingOffer, w::Problem> {
    w::OwnerFundingOffer::new(
        limits,
        w::OwnerFundingOfferParts {
            source: wire_funder(&value.source, limits)?,
            needed: value.needed,
            left: value.left.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            after: value.after.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(super) fn wire_owner_escalation(
    value: &c::OwnerEscalation,
    limits: &w::Limits,
) -> Result<w::OwnerEscalation, w::Problem> {
    w::OwnerEscalation::new(
        limits,
        w::OwnerEscalationParts {
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
            task: wire_task(&value.task, limits)?,
        },
    )
}

pub(super) fn wire_owner_ended_page(
    value: &c::OwnerEndedPage,
    limits: &w::Limits,
) -> Result<w::OwnerEndedPage, w::Problem> {
    w::OwnerEndedPage::new(
        limits,
        w::OwnerEndedPageParts {
            rows: pieces(
                value.rows.iter().map(|item| wire_owner_ended_delegate(item, limits)),
                w::OwnerEndedDelegate::measure,
                w::OwnerEndedDelegate::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_ended_delegate(
    value: &c::OwnerEndedDelegate,
    limits: &w::Limits,
) -> Result<w::OwnerEndedDelegate, w::Problem> {
    w::OwnerEndedDelegate::new(
        limits,
        w::OwnerEndedDelegateParts {
            chip: facts::chip(&value.chip, limits)?,
            ended: facts::owner_chip_ending(&value.ended, limits)?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_history_page(value: &c::HistoryPage, limits: &w::Limits) -> Result<w::HistoryPage, w::Problem> {
    w::HistoryPage::new(
        limits,
        w::HistoryPageParts {
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_history_row(item, limits)),
                w::HistoryRow::measure,
                w::HistoryRow::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_history_row(value: &c::HistoryRow, limits: &w::Limits) -> Result<w::HistoryRow, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HistoryRow::Change(value) => w::HistoryRow::Change(Box::new(w::HistoryRowChange::new(
            limits,
            w::HistoryRowChangeParts { value: wire_owner_history_line(value, limits)? },
        )?)),
        c::HistoryRow::Milestone(value) => w::HistoryRow::Milestone(w::HistoryRowMilestone::new(
            limits,
            w::HistoryRowMilestoneParts { value: wire_owner_milestone_line(value, limits)? },
        )?),
    })
}

pub(super) fn wire_owner_history_line(
    value: &c::OwnerHistoryLine,
    limits: &w::Limits,
) -> Result<w::OwnerHistoryLine, w::Problem> {
    w::OwnerHistoryLine::new(
        limits,
        w::OwnerHistoryLineParts {
            task: value.task,
            revision: value.revision,
            by: wire_owner_requester(&value.by, limits)?,
            change: wire_owner_history_change(&value.change, limits)?,
            reason: wire_document_text(&value.reason, limits)?,
            proposal: value.proposal.as_ref().map(|item| wire_owner_proposal_history(item, limits)).transpose()?,
            amendment: value
                .amendment
                .as_ref()
                .map(|item| {
                    w::OwnerHistoryAmendment::new(
                        limits,
                        w::OwnerHistoryAmendmentParts {
                            before: wire_owner_amendment_state(&(item).0, limits)?,
                            after: wire_owner_amendment_state(&(item).1, limits)?,
                        },
                    )
                })
                .transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_history_change(
    value: &c::OwnerHistoryChange,
    _limits: &w::Limits,
) -> Result<w::OwnerHistoryChange, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHistoryChange::Cancelled => w::OwnerHistoryChange::Cancelled,
        c::OwnerHistoryChange::Released => w::OwnerHistoryChange::Released,
        c::OwnerHistoryChange::Amended => w::OwnerHistoryChange::Amended,
        c::OwnerHistoryChange::Moved => w::OwnerHistoryChange::Moved,
        c::OwnerHistoryChange::Proposed => w::OwnerHistoryChange::Proposed,
        c::OwnerHistoryChange::ProposalPassed => w::OwnerHistoryChange::ProposalPassed,
        c::OwnerHistoryChange::ProposalAccepted => w::OwnerHistoryChange::ProposalAccepted,
        c::OwnerHistoryChange::ProposalRejected => w::OwnerHistoryChange::ProposalRejected,
        c::OwnerHistoryChange::ProposalWithdrawn => w::OwnerHistoryChange::ProposalWithdrawn,
    })
}

pub(super) fn wire_activity_page(value: &c::ActivityPage, limits: &w::Limits) -> Result<w::ActivityPage, w::Problem> {
    w::ActivityPage::new(
        limits,
        w::ActivityPageParts {
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_activity_line(item, limits)),
                w::ActivityLine::measure,
                w::ActivityLine::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_activity_line(value: &c::ActivityLine, limits: &w::Limits) -> Result<w::ActivityLine, w::Problem> {
    w::ActivityLine::new(
        limits,
        w::ActivityLineParts {
            source: wire_owner_activity_line(&value.source, limits)?,
            decoded: value.decoded.as_ref().map(|item| wire_archived_turn(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_owner_inbox_page(
    value: &c::OwnerInboxPage,
    limits: &w::Limits,
) -> Result<w::OwnerInboxPage, w::Problem> {
    w::OwnerInboxPage::new(
        limits,
        w::OwnerInboxPageParts {
            selection: value
                .selection
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_owner_inbox_selection(item, limits)),
                        w::OwnerInboxSelection::measure,
                        w::OwnerInboxSelection::encode,
                    )
                })
                .transpose()?,
            cut: value.cut,
            rows: pieces(
                value.rows.iter().map(|item| wire_owner_inbox_entry(item, limits)),
                w::OwnerInboxEntry::measure,
                w::OwnerInboxEntry::encode,
            )?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_owner_inbox_selection(
    value: &c::OwnerInboxSelection,
    limits: &w::Limits,
) -> Result<w::OwnerInboxSelection, w::Problem> {
    w::OwnerInboxSelection::new(
        limits,
        w::OwnerInboxSelectionParts {
            cursor: wire_owner_inbox_cursor(&value.cursor, limits)?,
            goals: list(value.goals.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
        },
    )
}

pub(super) fn wire_owner_inbox_cursor(
    value: &c::OwnerInboxCursor,
    limits: &w::Limits,
) -> Result<w::OwnerInboxCursor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerInboxCursor::Number { number } => w::OwnerInboxCursor::Number(w::OwnerInboxCursorNumber::new(
            limits,
            w::OwnerInboxCursorNumberParts { number: *(number) },
        )?),
        c::OwnerInboxCursor::Inbox { task, object, at } => w::OwnerInboxCursor::Inbox(w::OwnerInboxCursorInbox::new(
            limits,
            w::OwnerInboxCursorInboxParts {
                task: *(task),
                object: wire_owner_inbox_object(object, limits)?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
    })
}

pub(super) fn wire_owner_inbox_object(
    value: &c::OwnerInboxObject,
    limits: &w::Limits,
) -> Result<w::OwnerInboxObject, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerInboxObject::Question { message } => w::OwnerInboxObject::Question(w::OwnerInboxObjectQuestion::new(
            limits,
            w::OwnerInboxObjectQuestionParts { message: *(message) },
        )?),
        c::OwnerInboxObject::Proposal { proposer, number } => {
            w::OwnerInboxObject::Proposal(w::OwnerInboxObjectProposal::new(
                limits,
                w::OwnerInboxObjectProposalParts { proposer: wire_proposer(proposer, limits)?, number: *(number) },
            )?)
        }
        c::OwnerInboxObject::Escalation { revision } => w::OwnerInboxObject::Escalation(
            w::OwnerInboxObjectEscalation::new(limits, w::OwnerInboxObjectEscalationParts { revision: *(revision) })?,
        ),
        c::OwnerInboxObject::PersonTask => w::OwnerInboxObject::PersonTask,
        c::OwnerInboxObject::Reply { message } => w::OwnerInboxObject::Turn(w::OwnerInboxObjectTurn::new(
            limits,
            w::OwnerInboxObjectTurnParts { message: *(message) },
        )?),
        c::OwnerInboxObject::Result { position } => w::OwnerInboxObject::Result(w::OwnerInboxObjectResult::new(
            limits,
            w::OwnerInboxObjectResultParts { position: *(position) },
        )?),
    })
}

pub(super) fn wire_owner_inbox_entry(
    value: &c::OwnerInboxEntry,
    limits: &w::Limits,
) -> Result<w::OwnerInboxEntry, w::Problem> {
    w::OwnerInboxEntry::new(
        limits,
        w::OwnerInboxEntryParts {
            reply: value.reply.as_ref().map(|item| wire_task_line(item, limits)).transpose()?,
            task: wire_owner_task_link(&value.task, limits)?,
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            object: wire_owner_inbox_object(&value.object, limits)?,
            activity: value.activity.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            at: value.at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            unread: value.unread.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            cursor: wire_owner_inbox_cursor(&value.cursor, limits)?,
        },
    )
}

pub(super) fn wire_owner_spend_page(
    value: &c::OwnerSpendPage,
    limits: &w::Limits,
) -> Result<w::OwnerSpendPage, w::Problem> {
    w::OwnerSpendPage::new(
        limits,
        w::OwnerSpendPageParts {
            rows: list(value.rows.iter().map(|item| wire_owner_funding_record(item, limits)))?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_funding_record(
    value: &c::OwnerFundingRecord,
    limits: &w::Limits,
) -> Result<w::OwnerFundingRecord, w::Problem> {
    w::OwnerFundingRecord::new(
        limits,
        w::OwnerFundingRecordParts {
            period: value.period.as_ref().map(|item| wire_owner_period_times(item, limits)).transpose()?,
            funder: wire_funder(&value.funder, limits)?,
            parent: value.parent.as_ref().map(|item| wire_funder(item, limits)).transpose()?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
            made: value.made,
            closed: value.closed,
        },
    )
}

pub(super) fn wire_owner_period_times(
    value: &c::OwnerPeriodTimes,
    limits: &w::Limits,
) -> Result<w::OwnerPeriodTimes, w::Problem> {
    w::OwnerPeriodTimes::new(
        limits,
        w::OwnerPeriodTimesParts {
            began: value.began.as_nanos(),
            next: value.next.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_note_page(value: &c::NotePage, limits: &w::Limits) -> Result<w::NotePage, w::Problem> {
    w::NotePage::new(
        limits,
        w::NotePageParts {
            cut: value.cut,
            rows: pieces(value.rows.iter().map(|item| wire_note(item, limits)), w::Note::measure, w::Note::encode)?,
            before: value.before.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            after: value.after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_note(value: &c::Note, limits: &w::Limits) -> Result<w::Note, w::Problem> {
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

pub(super) fn wire_note_scope(value: &c::NoteScope, limits: &w::Limits) -> Result<w::NoteScope, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::NoteScope::Deployment => w::NoteScope::Deployment,
        c::NoteScope::Project(value) => {
            w::NoteScope::Project(w::NoteScopeProject::new(limits, w::NoteScopeProjectParts { value: *(value) })?)
        }
        c::NoteScope::Resource { project, connector, pattern } => w::NoteScope::Resource(w::NoteScopeResource::new(
            limits,
            w::NoteScopeResourceParts {
                project: *(project),
                connector: *(connector),
                pattern: wire_owner_pattern(pattern, limits)?,
            },
        )?),
        c::NoteScope::Task { project, task } => w::NoteScope::Task(w::NoteScopeTask::new(
            limits,
            w::NoteScopeTaskParts { project: *(project), task: *(task) },
        )?),
    })
}

pub(super) fn wire_note_author(value: &c::NoteAuthor, limits: &w::Limits) -> Result<w::NoteAuthor, w::Problem> {
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

pub(super) fn wire_owner_deployment(
    value: &c::OwnerDeployment,
    limits: &w::Limits,
) -> Result<w::OwnerDeployment, w::Problem> {
    w::OwnerDeployment::new(
        limits,
        w::OwnerDeploymentParts {
            skein_bools: [value.cut, value.connectors_cut, value.providers_cut, value.readiness_cut],
            store: wire_store_fill(&value.store, limits)?,
            hosts: pieces(
                value.hosts.iter().map(|item| wire_owner_host(item, limits)),
                w::OwnerHost::measure,
                w::OwnerHost::encode,
            )?,
            attempts: pieces(
                value.attempts.iter().map(|item| wire_owner_host_run(item, limits)),
                w::OwnerHostRun::measure,
                w::OwnerHostRun::encode,
            )?,
            accounts: pieces(
                value.accounts.iter().map(|item| wire_owner_account(item, limits)),
                w::OwnerAccount::measure,
                w::OwnerAccount::encode,
            )?,
            tasks: wire_owner_capacity(&value.tasks, limits)?,
            workers: wire_owner_capacity(&value.workers, limits)?,
            attempts_capacity: wire_owner_capacity(&value.attempts_capacity, limits)?,
            waiting_attempts: value.waiting_attempts,
            proposals: wire_owner_capacity(&value.proposals, limits)?,
            projects: list(value.projects.iter().map(|item| wire_owner_project_capacity(item, limits)))?,
            grace_bound: value.grace_bound,
            hello_refusals: list(value.hello_refusals.iter().map(|item| wire_owner_hello_refusal_line(item, limits)))?,
            hosts_forgotten: value.hosts_forgotten,
            hello_refusals_forgotten: value.hello_refusals_forgotten,
            person_room_refusals: list(
                value.person_room_refusals.iter().map(|item| wire_owner_person_capacity_refusal(item, limits)),
            )?,
            person_room_refusals_forgotten: value.person_room_refusals_forgotten,
            person_answer_retention: value.person_answer_retention,
            task_room_refusals: list(
                value.task_room_refusals.iter().map(|item| wire_owner_task_capacity_refusal(item, limits)),
            )?,
            task_room_refusals_forgotten: value.task_room_refusals_forgotten,
            connectors: list(value.connectors.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            providers: pieces(
                value.providers.iter().map(|item| wire_provider_offer(item, limits)),
                w::ProviderOffer::measure,
                w::ProviderOffer::encode,
            )?,
            key_bytes: value.key_bytes,
            readiness: pieces(
                value.readiness.iter().map(|item| wire_payer_readiness(item, limits)),
                w::PayerReadiness::measure,
                w::PayerReadiness::encode,
            )?,
        },
    )
}

pub(super) fn wire_store_fill(value: &c::StoreFill, limits: &w::Limits) -> Result<w::StoreFill, w::Problem> {
    w::StoreFill::new(limits, w::StoreFillParts { bytes: value.bytes, mark: value.mark })
}

pub(super) fn wire_owner_host(value: &c::OwnerHost, limits: &w::Limits) -> Result<w::OwnerHost, w::Problem> {
    w::OwnerHost::new(
        limits,
        w::OwnerHostParts {
            skein_bools: [value.awaiting_logins, value.draining, value.connected, value.cut],
            name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
            rule: wire_owner_host_rule(&value.rule, limits)?,
            programs: pieces(
                value.programs.iter().map(|item| wire_owner_program_login(item, limits)),
                w::OwnerProgramLogin::measure,
                w::OwnerProgramLogin::encode,
            )?,
            channel: value.channel,
            kind: wire_owner_host_kind(&value.kind, limits)?,
            slots: value.slots,
            used: value.used,
            free: value.free,
            grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            grace_remaining: value.grace_remaining.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            runs: pieces(
                value.runs.iter().map(|item| wire_owner_host_run(item, limits)),
                w::OwnerHostRun::measure,
                w::OwnerHostRun::encode,
            )?,
        },
    )
}

pub(super) fn wire_owner_host_rule(value: &c::HostRule, limits: &w::Limits) -> Result<w::OwnerHostRule, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HostRule::Open => w::OwnerHostRule::Open,
        c::HostRule::Person(value) => {
            w::OwnerHostRule::Person(w::OwnerHostPerson::new(limits, w::OwnerHostPersonParts { party: *(value) })?)
        }
        c::HostRule::Several => w::OwnerHostRule::Several,
        c::HostRule::Unknown => w::OwnerHostRule::Unknown,
    })
}

pub(super) fn wire_owner_program_login(
    value: &c::OwnerProgramLogin,
    limits: &w::Limits,
) -> Result<w::OwnerProgramLogin, w::Problem> {
    w::OwnerProgramLogin::new(
        limits,
        w::OwnerProgramLoginParts {
            name: bytes::copy_of(&value.name),
            state: value.state.as_ref().map(|item| wire_owner_program_state(item, limits)).transpose()?,
            sign_out: value.sign_out.as_ref().map(|item| wire_owner_program_sign_out(item, limits)).transpose()?,
            cleanup: value.cleanup,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_program_state(
    value: &c::OwnerProgramState,
    limits: &w::Limits,
) -> Result<w::OwnerProgramState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProgramState::Usable => w::OwnerProgramState::Usable,
        c::OwnerProgramState::NotSignedIn => w::OwnerProgramState::NotSignedIn,
        c::OwnerProgramState::SignedInOtherwise { method } => w::OwnerProgramState::SignedInOtherwise(
            w::OwnerProgramOtherwise::new(limits, w::OwnerProgramOtherwiseParts { method: bytes::copy_of(method) })?,
        ),
        c::OwnerProgramState::Failed { reason, detail } => w::OwnerProgramState::Failed(w::OwnerProgramFailed::new(
            limits,
            w::OwnerProgramFailedParts {
                reason: wire_owner_program_failure(reason, limits)?,
                detail: bytes::copy_of(detail),
            },
        )?),
    })
}

pub(super) fn wire_owner_program_failure(
    value: &c::OwnerProgramFailure,
    _limits: &w::Limits,
) -> Result<w::OwnerProgramFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProgramFailure::Missing => w::OwnerProgramFailure::Missing,
        c::OwnerProgramFailure::Deadline => w::OwnerProgramFailure::Deadline,
        c::OwnerProgramFailure::Unreadable => w::OwnerProgramFailure::Unreadable,
        c::OwnerProgramFailure::Exited => w::OwnerProgramFailure::Exited,
        c::OwnerProgramFailure::Unconfigured => w::OwnerProgramFailure::Unconfigured,
        c::OwnerProgramFailure::Unknown => w::OwnerProgramFailure::Unknown,
        c::OwnerProgramFailure::Closed => w::OwnerProgramFailure::Closed,
    })
}

pub(super) fn wire_owner_program_sign_out(
    value: &c::OwnerProgramSignOut,
    limits: &w::Limits,
) -> Result<w::OwnerProgramSignOut, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProgramSignOut::SignedOut => w::OwnerProgramSignOut::SignedOut,
        c::OwnerProgramSignOut::Failed { reason, detail } => {
            w::OwnerProgramSignOut::Failed(w::OwnerProgramFailed::new(
                limits,
                w::OwnerProgramFailedParts {
                    reason: wire_owner_program_failure(reason, limits)?,
                    detail: bytes::copy_of(detail),
                },
            )?)
        }
    })
}

pub(super) fn wire_owner_host_run(value: &c::OwnerHostRun, limits: &w::Limits) -> Result<w::OwnerHostRun, w::Problem> {
    w::OwnerHostRun::new(
        limits,
        w::OwnerHostRunParts {
            task: wire_owner_task_link(&value.task, limits)?,
            attempt: value.attempt,
            kept: value.kept,
            listed: value.listed,
            state: wire_owner_attempt_state(&value.state, limits)?,
            location: wire_owner_run_location(&value.location, limits)?,
            grace_until: value.grace_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_attempt_state(
    value: &c::OwnerAttemptState,
    _limits: &w::Limits,
) -> Result<w::OwnerAttemptState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAttemptState::Waiting => w::OwnerAttemptState::Waiting,
        c::OwnerAttemptState::Adopted => w::OwnerAttemptState::Adopted,
        c::OwnerAttemptState::Claimed => w::OwnerAttemptState::Claimed,
        c::OwnerAttemptState::Cancelled => w::OwnerAttemptState::Cancelled,
        c::OwnerAttemptState::Handed => w::OwnerAttemptState::Handed,
        c::OwnerAttemptState::Acknowledged => w::OwnerAttemptState::Acknowledged,
        c::OwnerAttemptState::Stray => w::OwnerAttemptState::Stray,
        c::OwnerAttemptState::Kept => w::OwnerAttemptState::Kept,
        c::OwnerAttemptState::Fenced => w::OwnerAttemptState::Fenced,
        c::OwnerAttemptState::Closed => w::OwnerAttemptState::Closed,
    })
}

pub(super) fn wire_owner_run_location(
    value: &c::OwnerRunLocation,
    limits: &w::Limits,
) -> Result<w::OwnerRunLocation, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRunLocation::None => w::OwnerRunLocation::None,
        c::OwnerRunLocation::Connected { channel, kind } => {
            w::OwnerRunLocation::Connected(w::OwnerRunLocationConnected::new(
                limits,
                w::OwnerRunLocationConnectedParts { channel: *(channel), kind: wire_owner_host_kind(kind, limits)? },
            )?)
        }
        c::OwnerRunLocation::Adrift { channel, until, remaining } => {
            w::OwnerRunLocation::Adrift(w::OwnerRunLocationAdrift::new(
                limits,
                w::OwnerRunLocationAdriftParts {
                    channel: (channel).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                    remaining: (remaining).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                },
            )?)
        }
    })
}

pub(super) fn wire_owner_account(value: &c::OwnerAccount, limits: &w::Limits) -> Result<w::OwnerAccount, w::Problem> {
    w::OwnerAccount::new(
        limits,
        w::OwnerAccountParts {
            skein_bools: [value.plan, value.usable, value.expiring, value.attention, value.cut],
            number: value.number,
            place: value.place,
            uses: value
                .uses
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_owner_account_use(item, limits))))
                .transpose()?,
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            display: wire_document_text(&value.display, limits)?,
            holder: wire_owner_account_holder(&value.holder, limits)?,
            sources: pieces(
                value.sources.iter().map(|item| wire_owner_account_source(item, limits)),
                w::OwnerAccountSource::measure,
                w::OwnerAccountSource::encode,
            )?,
            generation: value.generation.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            state: wire_owner_account_state(&value.state, limits)?,
            spent_until: value.spent_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            runs: pieces(value.runs.iter().map(|item| facts::chip(item, limits)), w::Chip::measure, w::Chip::encode)?,
            waiting: pieces(
                value.waiting.iter().map(|item| facts::chip(item, limits)),
                w::Chip::measure,
                w::Chip::encode,
            )?,
            provider_name: value.provider_name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            manage_usage: value
                .manage_usage
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of(item)))
                .transpose()?,
        },
    )
}

pub(super) fn wire_owner_account_use(
    value: &c::OwnerAccountUse,
    limits: &w::Limits,
) -> Result<w::OwnerAccountUse, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountUse::Party(value) => w::OwnerAccountUse::Party(w::OwnerAccountUseParty::new(
            limits,
            w::OwnerAccountUsePartyParts { party: *(value) },
        )?),
        c::OwnerAccountUse::Project(value) => w::OwnerAccountUse::Project(w::OwnerAccountUseProject::new(
            limits,
            w::OwnerAccountUseProjectParts { project: *(value) },
        )?),
    })
}

pub(super) fn wire_owner_account_holder(
    value: &c::OwnerAccountHolder,
    limits: &w::Limits,
) -> Result<w::OwnerAccountHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountHolder::Deployment => w::OwnerAccountHolder::Deployment,
        c::OwnerAccountHolder::Party(value) => w::OwnerAccountHolder::Party(w::OwnerAccountHolderParty::new(
            limits,
            w::OwnerAccountHolderPartyParts { party: *(value) },
        )?),
    })
}

pub(super) fn wire_owner_account_source(
    value: &c::OwnerAccountSource,
    limits: &w::Limits,
) -> Result<w::OwnerAccountSource, w::Problem> {
    w::OwnerAccountSource::new(
        limits,
        w::OwnerAccountSourceParts {
            name: value.name.as_ref().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))).transpose()?,
            place: value.place,
            number: value.number,
            kind: wire_owner_account_kind(&value.kind, limits)?,
            state: wire_owner_account_state(&value.state, limits)?,
            usable: value.usable,
            expiring: value.expiring,
            attention: value.attention,
            spent_until: value.spent_until.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_account_kind(
    value: &c::OwnerAccountKind,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountKind, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountKind::SignIn => w::OwnerAccountKind::SignIn,
        c::OwnerAccountKind::Key => w::OwnerAccountKind::Key,
        c::OwnerAccountKind::Program => w::OwnerAccountKind::Program,
        c::OwnerAccountKind::None => w::OwnerAccountKind::None,
    })
}

pub(super) fn wire_owner_account_state(
    value: &c::OwnerAccountState,
    limits: &w::Limits,
) -> Result<w::OwnerAccountState, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountState::Asking => w::OwnerAccountState::Asking,
        c::OwnerAccountState::Granted => w::OwnerAccountState::Granted,
        c::OwnerAccountState::Later { until } => w::OwnerAccountState::Later(w::OwnerAccountStateLater::new(
            limits,
            w::OwnerAccountStateLaterParts {
                until: (until).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::OwnerAccountState::SignInAgain => w::OwnerAccountState::SignInAgain,
        c::OwnerAccountState::Misconfigured => w::OwnerAccountState::Misconfigured,
        c::OwnerAccountState::Unloaded { at, why } => {
            w::OwnerAccountState::Unloaded(w::OwnerAccountStateUnloaded::new(
                limits,
                w::OwnerAccountStateUnloadedParts {
                    at: wire_owner_account_place(at, limits)?,
                    why: wire_owner_account_unread(why, limits)?,
                },
            )?)
        }
        c::OwnerAccountState::Cancelled => w::OwnerAccountState::Cancelled,
        c::OwnerAccountState::Closing => w::OwnerAccountState::Closing,
        c::OwnerAccountState::Moving => w::OwnerAccountState::Moving,
    })
}

pub(super) fn wire_owner_account_place(
    value: &c::OwnerAccountPlace,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountPlace, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountPlace::Directory => w::OwnerAccountPlace::Directory,
        c::OwnerAccountPlace::File => w::OwnerAccountPlace::File,
    })
}

pub(super) fn wire_owner_account_unread(
    value: &c::OwnerAccountUnread,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountUnread, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountUnread::Unreadable => w::OwnerAccountUnread::Unreadable,
        c::OwnerAccountUnread::Refused => w::OwnerAccountUnread::Refused,
        c::OwnerAccountUnread::Failed => w::OwnerAccountUnread::Failed,
        c::OwnerAccountUnread::Stalled => w::OwnerAccountUnread::Stalled,
        c::OwnerAccountUnread::TooLarge => w::OwnerAccountUnread::TooLarge,
    })
}

pub(super) fn wire_owner_capacity(
    value: &c::OwnerCapacity,
    limits: &w::Limits,
) -> Result<w::OwnerCapacity, w::Problem> {
    w::OwnerCapacity::new(limits, w::OwnerCapacityParts { used: value.used, limit: value.limit })
}

pub(super) fn wire_owner_project_capacity(
    value: &c::OwnerProjectCapacity,
    limits: &w::Limits,
) -> Result<w::OwnerProjectCapacity, w::Problem> {
    w::OwnerProjectCapacity::new(
        limits,
        w::OwnerProjectCapacityParts {
            project: value.project,
            tasks: wire_owner_capacity(&value.tasks, limits)?,
            proposals: value.proposals,
        },
    )
}

pub(super) fn wire_owner_hello_refusal_line(
    value: &c::OwnerHelloRefusalLine,
    limits: &w::Limits,
) -> Result<w::OwnerHelloRefusalLine, w::Problem> {
    w::OwnerHelloRefusalLine::new(
        limits,
        w::OwnerHelloRefusalLineParts {
            channel: value.channel,
            at: value.at.as_nanos(),
            slots: value.slots,
            stop_bound: value.stop_bound,
            why: wire_owner_hello_refusal(&value.why, limits)?,
        },
    )
}

pub(super) fn wire_owner_person_capacity_refusal(
    value: &c::OwnerPersonCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerPersonCapacityRefusal, w::Problem> {
    w::OwnerPersonCapacityRefusal::new(
        limits,
        w::OwnerPersonCapacityRefusalParts {
            key: wire_owner_request_key(&value.key, limits)?,
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            action: wire_owner_person_capacity_action(&value.action, limits)?,
            at: value.at.as_nanos(),
            cause: wire_owner_person_capacity_cause(&value.cause, limits)?,
        },
    )
}

pub(super) fn wire_owner_request_key(
    value: &c::OwnerRequestKey,
    limits: &w::Limits,
) -> Result<w::OwnerRequestKey, w::Problem> {
    w::OwnerRequestKey::new(limits, w::OwnerRequestKeyParts { person: value.person, key: value.key })
}

pub(super) fn wire_owner_person_capacity_action(
    value: &c::OwnerPersonCapacityAction,
    limits: &w::Limits,
) -> Result<w::OwnerPersonCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPersonCapacityAction::Lift { hold } => w::OwnerPersonCapacityAction::Lift(
            w::OwnerPersonCapacityActionLift::new(limits, w::OwnerPersonCapacityActionLiftParts { hold: *(hold) })?,
        ),
        c::OwnerPersonCapacityAction::Create => w::OwnerPersonCapacityAction::Create,
        c::OwnerPersonCapacityAction::Compound => w::OwnerPersonCapacityAction::Compound,
        c::OwnerPersonCapacityAction::Application { connector } => {
            w::OwnerPersonCapacityAction::Application(w::OwnerPersonCapacityActionApplication::new(
                limits,
                w::OwnerPersonCapacityActionApplicationParts { connector: *(connector) },
            )?)
        }
        c::OwnerPersonCapacityAction::Note { name } => w::OwnerPersonCapacityAction::Note(
            w::OwnerPersonCapacityActionNote::new(limits, w::OwnerPersonCapacityActionNoteParts { name: *(name) })?,
        ),
        c::OwnerPersonCapacityAction::Service => w::OwnerPersonCapacityAction::Service,
        c::OwnerPersonCapacityAction::Adoption => w::OwnerPersonCapacityAction::Adoption,
        c::OwnerPersonCapacityAction::Roles => w::OwnerPersonCapacityAction::Roles,
        c::OwnerPersonCapacityAction::Policy => w::OwnerPersonCapacityAction::Policy,
        c::OwnerPersonCapacityAction::Pool { person } => w::OwnerPersonCapacityAction::Pool(
            w::OwnerPersonCapacityActionPool::new(limits, w::OwnerPersonCapacityActionPoolParts { person: *(person) })?,
        ),
        c::OwnerPersonCapacityAction::Escalation { task } => {
            w::OwnerPersonCapacityAction::Escalation(w::OwnerPersonCapacityActionEscalation::new(
                limits,
                w::OwnerPersonCapacityActionEscalationParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::Proposal { proposer, number } => {
            w::OwnerPersonCapacityAction::Proposal(w::OwnerPersonCapacityActionProposal::new(
                limits,
                w::OwnerPersonCapacityActionProposalParts {
                    proposer: wire_proposer(proposer, limits)?,
                    number: *(number),
                },
            )?)
        }
        c::OwnerPersonCapacityAction::Words { task } => w::OwnerPersonCapacityAction::Words(
            w::OwnerPersonCapacityActionWords::new(limits, w::OwnerPersonCapacityActionWordsParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Question { task, question } => {
            w::OwnerPersonCapacityAction::Question(w::OwnerPersonCapacityActionQuestion::new(
                limits,
                w::OwnerPersonCapacityActionQuestionParts { task: *(task), question: *(question) },
            )?)
        }
        c::OwnerPersonCapacityAction::Amend { task } => w::OwnerPersonCapacityAction::Amend(
            w::OwnerPersonCapacityActionAmend::new(limits, w::OwnerPersonCapacityActionAmendParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Move { task } => w::OwnerPersonCapacityAction::MoveTask(
            w::OwnerPersonCapacityActionMove::new(limits, w::OwnerPersonCapacityActionMoveParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::TakePerson { task } => {
            w::OwnerPersonCapacityAction::TakePerson(w::OwnerPersonCapacityActionTakePerson::new(
                limits,
                w::OwnerPersonCapacityActionTakePersonParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::HandBackPerson { task } => {
            w::OwnerPersonCapacityAction::HandBackPerson(w::OwnerPersonCapacityActionHandBackPerson::new(
                limits,
                w::OwnerPersonCapacityActionHandBackPersonParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::AnswerPerson { task } => {
            w::OwnerPersonCapacityAction::AnswerPerson(w::OwnerPersonCapacityActionAnswerPerson::new(
                limits,
                w::OwnerPersonCapacityActionAnswerPersonParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::Stop { task } => w::OwnerPersonCapacityAction::Stop(
            w::OwnerPersonCapacityActionStop::new(limits, w::OwnerPersonCapacityActionStopParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Cancel { task } => w::OwnerPersonCapacityAction::Cancel(
            w::OwnerPersonCapacityActionCancel::new(limits, w::OwnerPersonCapacityActionCancelParts { task: *(task) })?,
        ),
        c::OwnerPersonCapacityAction::Release { task } => {
            w::OwnerPersonCapacityAction::Release(w::OwnerPersonCapacityActionRelease::new(
                limits,
                w::OwnerPersonCapacityActionReleaseParts { task: *(task) },
            )?)
        }
        c::OwnerPersonCapacityAction::MarkSeen { seen } => {
            w::OwnerPersonCapacityAction::MarkSeen(w::OwnerPersonCapacityActionMarkSeen::new(
                limits,
                w::OwnerPersonCapacityActionMarkSeenParts { seen: wire_seen_through(seen, limits)? },
            )?)
        }
        c::OwnerPersonCapacityAction::AccountSignIn => w::OwnerPersonCapacityAction::AccountSignIn,
        c::OwnerPersonCapacityAction::AccountSignOut => w::OwnerPersonCapacityAction::AccountSignOut,
        c::OwnerPersonCapacityAction::AccountChange => w::OwnerPersonCapacityAction::AccountChange,
    })
}

pub(super) fn wire_seen_through(value: &c::SeenThrough, limits: &w::Limits) -> Result<w::SeenThrough, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::SeenThrough::Results { through } => {
            w::SeenThrough::Results(w::SeenResults::new(limits, w::SeenResultsParts { through: *(through) })?)
        }
        c::SeenThrough::Conversation { task, through } => w::SeenThrough::Conversation(w::SeenConversation::new(
            limits,
            w::SeenConversationParts { task: *(task), through: *(through) },
        )?),
    })
}

pub(super) fn wire_owner_person_capacity_cause(
    value: &c::OwnerPersonCapacityCause,
    limits: &w::Limits,
) -> Result<w::OwnerPersonCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPersonCapacityCause::Requests { limit } => {
            w::OwnerPersonCapacityCause::Requests(w::OwnerPersonCapacityCauseRequests::new(
                limits,
                w::OwnerPersonCapacityCauseRequestsParts { limit: *(limit) },
            )?)
        }
        c::OwnerPersonCapacityCause::Pending { limit } => {
            w::OwnerPersonCapacityCause::Pending(w::OwnerPersonCapacityCausePending::new(
                limits,
                w::OwnerPersonCapacityCausePendingParts { limit: *(limit) },
            )?)
        }
        c::OwnerPersonCapacityCause::Waiters { limit } => {
            w::OwnerPersonCapacityCause::Waiters(w::OwnerPersonCapacityCauseWaiters::new(
                limits,
                w::OwnerPersonCapacityCauseWaitersParts { limit: *(limit) },
            )?)
        }
        c::OwnerPersonCapacityCause::Limit { which } => {
            w::OwnerPersonCapacityCause::Limit(w::OwnerPersonCapacityCauseLimit::new(
                limits,
                w::OwnerPersonCapacityCauseLimitParts { which: wire_admission_limit(which, limits)? },
            )?)
        }
    })
}

pub(super) fn wire_owner_task_capacity_refusal(
    value: &c::OwnerTaskCapacityRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityRefusal, w::Problem> {
    w::OwnerTaskCapacityRefusal::new(
        limits,
        w::OwnerTaskCapacityRefusalParts {
            request: wire_owner_task_capacity_request(&value.request, limits)?,
            at: value.at.as_nanos(),
        },
    )
}

pub(super) fn wire_owner_task_capacity_request(
    value: &c::OwnerTaskCapacityRequest,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityRequest, w::Problem> {
    w::OwnerTaskCapacityRequest::new(
        limits,
        w::OwnerTaskCapacityRequestParts {
            project: value.project.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            by: wire_owner_requester(&value.by, limits)?,
            action: wire_owner_task_capacity_action(&value.action, limits)?,
            cause: wire_owner_task_capacity_cause(&value.cause, limits)?,
        },
    )
}

pub(super) fn wire_owner_task_capacity_action(
    value: &c::OwnerTaskCapacityAction,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityAction, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskCapacityAction::Batch { first, members } => {
            w::OwnerTaskCapacityAction::Batch(w::OwnerTaskCapacityActionBatch::new(
                limits,
                w::OwnerTaskCapacityActionBatchParts {
                    first: (first).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    members: *(members),
                },
            )?)
        }
        c::OwnerTaskCapacityAction::Proposal { number } => {
            w::OwnerTaskCapacityAction::Proposal(w::OwnerTaskCapacityActionProposal::new(
                limits,
                w::OwnerTaskCapacityActionProposalParts { number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::PersonProposal { number } => {
            w::OwnerTaskCapacityAction::PersonProposal(w::OwnerTaskCapacityActionPersonProposal::new(
                limits,
                w::OwnerTaskCapacityActionPersonProposalParts { number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::Message { task, number } => {
            w::OwnerTaskCapacityAction::Message(w::OwnerTaskCapacityActionMessage::new(
                limits,
                w::OwnerTaskCapacityActionMessageParts { task: *(task), number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::Source { funder } => {
            w::OwnerTaskCapacityAction::Source(w::OwnerTaskCapacityActionSource::new(
                limits,
                w::OwnerTaskCapacityActionSourceParts { funder: wire_funder(funder, limits)? },
            )?)
        }
        c::OwnerTaskCapacityAction::Introduce { left, right } => {
            w::OwnerTaskCapacityAction::Introduce(w::OwnerTaskCapacityActionIntroduce::new(
                limits,
                w::OwnerTaskCapacityActionIntroduceParts { left: *(left), right: *(right) },
            )?)
        }
        c::OwnerTaskCapacityAction::Subscription { number } => {
            w::OwnerTaskCapacityAction::Subscription(w::OwnerTaskCapacityActionSubscription::new(
                limits,
                w::OwnerTaskCapacityActionSubscriptionParts { number: *(number) },
            )?)
        }
        c::OwnerTaskCapacityAction::Effect { entry } => w::OwnerTaskCapacityAction::Effect(
            w::OwnerTaskCapacityActionEffect::new(limits, w::OwnerTaskCapacityActionEffectParts { entry: *(entry) })?,
        ),
        c::OwnerTaskCapacityAction::Claim { attempt } => w::OwnerTaskCapacityAction::Claim(
            w::OwnerTaskCapacityActionClaim::new(limits, w::OwnerTaskCapacityActionClaimParts { attempt: *(attempt) })?,
        ),
        c::OwnerTaskCapacityAction::Release => w::OwnerTaskCapacityAction::Release,
        c::OwnerTaskCapacityAction::Move => w::OwnerTaskCapacityAction::MoveTask,
        c::OwnerTaskCapacityAction::Place { hold } => w::OwnerTaskCapacityAction::Place(
            w::OwnerTaskCapacityActionPlace::new(limits, w::OwnerTaskCapacityActionPlaceParts { hold: *(hold) })?,
        ),
    })
}

pub(super) fn wire_owner_task_capacity_cause(
    value: &c::OwnerTaskCapacityCause,
    limits: &w::Limits,
) -> Result<w::OwnerTaskCapacityCause, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskCapacityCause::Tasks { limit } => w::OwnerTaskCapacityCause::Tasks(
            w::OwnerTaskCapacityCauseTasks::new(limits, w::OwnerTaskCapacityCauseTasksParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::CreationFlights { limit } => {
            w::OwnerTaskCapacityCause::CreationFlights(w::OwnerTaskCapacityCauseCreationFlights::new(
                limits,
                w::OwnerTaskCapacityCauseCreationFlightsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::ResourceReports { limit } => {
            w::OwnerTaskCapacityCause::ResourceReports(w::OwnerTaskCapacityCauseResourceReports::new(
                limits,
                w::OwnerTaskCapacityCauseResourceReportsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::ProjectTasks { limit } => {
            w::OwnerTaskCapacityCause::ProjectTasks(w::OwnerTaskCapacityCauseProjectTasks::new(
                limits,
                w::OwnerTaskCapacityCauseProjectTasksParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Tree { limit } => w::OwnerTaskCapacityCause::Tree(
            w::OwnerTaskCapacityCauseTree::new(limits, w::OwnerTaskCapacityCauseTreeParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::Delegates { limit } => {
            w::OwnerTaskCapacityCause::Delegates(w::OwnerTaskCapacityCauseDelegates::new(
                limits,
                w::OwnerTaskCapacityCauseDelegatesParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Inbox { task, messages, bytes } => {
            w::OwnerTaskCapacityCause::Inbox(w::OwnerTaskCapacityCauseInbox::new(
                limits,
                w::OwnerTaskCapacityCauseInboxParts { task: *(task), messages: *(messages), bytes: *(bytes) },
            )?)
        }
        c::OwnerTaskCapacityCause::Holdings { limit } => w::OwnerTaskCapacityCause::Holdings(
            w::OwnerTaskCapacityCauseHoldings::new(limits, w::OwnerTaskCapacityCauseHoldingsParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::HoldWaiters { limit } => {
            w::OwnerTaskCapacityCause::HoldWaiters(w::OwnerTaskCapacityCauseHoldWaiters::new(
                limits,
                w::OwnerTaskCapacityCauseHoldWaitersParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Proposals { limit } => {
            w::OwnerTaskCapacityCause::Proposals(w::OwnerTaskCapacityCauseProposals::new(
                limits,
                w::OwnerTaskCapacityCauseProposalsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::References { limit } => {
            w::OwnerTaskCapacityCause::References(w::OwnerTaskCapacityCauseReferences::new(
                limits,
                w::OwnerTaskCapacityCauseReferencesParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Subscriptions { limit } => {
            w::OwnerTaskCapacityCause::Subscriptions(w::OwnerTaskCapacityCauseSubscriptions::new(
                limits,
                w::OwnerTaskCapacityCauseSubscriptionsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Funders { limit } => w::OwnerTaskCapacityCause::Funders(
            w::OwnerTaskCapacityCauseFunders::new(limits, w::OwnerTaskCapacityCauseFundersParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::Writers { limit } => w::OwnerTaskCapacityCause::Writers(
            w::OwnerTaskCapacityCauseWriters::new(limits, w::OwnerTaskCapacityCauseWritersParts { limit: *(limit) })?,
        ),
        c::OwnerTaskCapacityCause::PlacedHolds { limit } => {
            w::OwnerTaskCapacityCause::PlacedHolds(w::OwnerTaskCapacityCausePlacedHolds::new(
                limits,
                w::OwnerTaskCapacityCausePlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::ProjectPlacedHolds { limit } => {
            w::OwnerTaskCapacityCause::ProjectPlacedHolds(w::OwnerTaskCapacityCauseProjectPlacedHolds::new(
                limits,
                w::OwnerTaskCapacityCauseProjectPlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::PartyPlacedHolds { limit } => {
            w::OwnerTaskCapacityCause::PartyPlacedHolds(w::OwnerTaskCapacityCausePartyPlacedHolds::new(
                limits,
                w::OwnerTaskCapacityCausePartyPlacedHoldsParts { limit: *(limit) },
            )?)
        }
        c::OwnerTaskCapacityCause::Exemptions { limit } => {
            w::OwnerTaskCapacityCause::Exemptions(w::OwnerTaskCapacityCauseExemptions::new(
                limits,
                w::OwnerTaskCapacityCauseExemptionsParts { limit: *(limit) },
            )?)
        }
    })
}

pub(super) fn wire_payer_readiness(
    value: &c::PayerReadiness,
    limits: &w::Limits,
) -> Result<w::PayerReadiness, w::Problem> {
    w::PayerReadiness::new(
        limits,
        w::PayerReadinessParts {
            provider: value.provider,
            payer: wire_owner_account_use(&value.payer, limits)?,
            readiness: wire_readiness(&value.readiness, limits)?,
            settled: value.settled.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}

pub(super) fn wire_readiness(value: &c::Readiness, _limits: &w::Limits) -> Result<w::Readiness, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Readiness::Usable => w::Readiness::Usable,
        c::Readiness::Unavailable => w::Readiness::Unavailable,
        c::Readiness::Unknown => w::Readiness::Unknown,
    })
}

pub(super) fn wire_delivery_value(value: &c::Delivery, limits: &w::Limits) -> Result<w::DeliveryValue, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Delivery::Settings(value) => w::DeliveryValue::Settings(wire_settings_document(value, limits)?),
        c::Delivery::OwnerProject(value) => w::DeliveryValue::OwnerProject(wire_owner_project(value, limits)?),
        c::Delivery::Me(value) => w::DeliveryValue::Me(wire_me(value, limits)?),
        c::Delivery::Task(value) => w::DeliveryValue::Task(Box::new(wire_task(value, limits)?)),
        c::Delivery::OwnerProposal(value) => w::DeliveryValue::OwnerProposal(wire_owner_proposal(value, limits)?),
        c::Delivery::OwnerTree(value) => w::DeliveryValue::OwnerTree(wire_owner_tree(value, limits)?),
        c::Delivery::Chips(value) => w::DeliveryValue::Chips(w::Chips::new(
            limits,
            w::ChipsParts {
                rows: pieces((value).iter().map(|item| facts::chip(item, limits)), w::Chip::measure, w::Chip::encode)?,
            },
        )?),
        c::Delivery::OwnerInbox(value) => w::DeliveryValue::OwnerInbox(wire_owner_inbox(value, limits)?),
        c::Delivery::Tasks(value) => w::DeliveryValue::Tasks(wire_tasks(value, limits)?),
        c::Delivery::RunState(value) => w::DeliveryValue::RunState(wire_run_state(value, limits)?),
        c::Delivery::OwnerDeployment(value) => w::DeliveryValue::OwnerDeployment(wire_owner_deployment(value, limits)?),
        c::Delivery::OwnerAccounts(value) => w::DeliveryValue::OwnerAccounts(wire_owner_accounts(value, limits)?),
    })
}

pub(super) fn wire_owner_tree(value: &c::OwnerTree, limits: &w::Limits) -> Result<w::OwnerTree, w::Problem> {
    w::OwnerTree::new(
        limits,
        w::OwnerTreeParts {
            root: facts::chip(&value.root, limits)?,
            rows: pieces(
                value.rows.iter().map(|item| wire_owner_tree_row(item, limits)),
                w::OwnerTreeRow::measure,
                w::OwnerTreeRow::encode,
            )?,
            folded: list(value.folded.iter().map(|item| wire_owner_tree_fold(item, limits)))?,
            held_below: value.held_below,
            next: value.next.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_owner_tree_row(value: &c::OwnerTreeRow, limits: &w::Limits) -> Result<w::OwnerTreeRow, w::Problem> {
    w::OwnerTreeRow::new(
        limits,
        w::OwnerTreeRowParts {
            chip: facts::chip(&value.chip, limits)?,
            parent: value.parent,
            depth: value.depth,
            proposals: list(value.proposals.iter().map(|item| wire_proposal_key(item, limits)))?,
            spent: value.spent,
            dependencies: pieces(
                value.dependencies.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_tree_fold(
    value: &c::OwnerTreeFold,
    limits: &w::Limits,
) -> Result<w::OwnerTreeFold, w::Problem> {
    w::OwnerTreeFold::new(
        limits,
        w::OwnerTreeFoldParts {
            parent: value.parent,
            phases: wire_owner_phase_counts(&value.phases, limits)?,
            held_below: value.held_below,
        },
    )
}

pub(super) fn wire_owner_inbox(value: &c::OwnerInbox, limits: &w::Limits) -> Result<w::OwnerInbox, w::Problem> {
    w::OwnerInbox::new(
        limits,
        w::OwnerInboxParts {
            party: value.party,
            needs: pieces(
                value.needs.iter().map(|item| wire_owner_inbox_entry(item, limits)),
                w::OwnerInboxEntry::measure,
                w::OwnerInboxEntry::encode,
            )?,
            updates: pieces(
                value.updates.iter().map(|item| wire_owner_inbox_entry(item, limits)),
                w::OwnerInboxEntry::measure,
                w::OwnerInboxEntry::encode,
            )?,
            next: value.next.as_ref().map(|item| wire_owner_inbox_cursor(item, limits)).transpose()?,
            cached_needs: value.cached_needs,
            cached_updates: wire_count(&value.cached_updates, limits)?,
            results_seen: value.results_seen,
            updates_time_complete: value.updates_time_complete,
            selection: value
                .selection
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_owner_inbox_selection(item, limits)),
                        w::OwnerInboxSelection::measure,
                        w::OwnerInboxSelection::encode,
                    )
                })
                .transpose()?,
        },
    )
}

pub(super) fn wire_tasks(value: &c::Tasks, limits: &w::Limits) -> Result<w::Tasks, w::Problem> {
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

pub(super) fn wire_run_state(value: &c::RunState, limits: &w::Limits) -> Result<w::RunState, w::Problem> {
    w::RunState::new(
        limits,
        w::RunStateParts {
            task: value.task,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            committed_turn: value.committed_turn,
            writing: value.writing.as_ref().map(|item| wire_run_text(item, limits)).transpose()?,
            call: value.call.as_ref().map(|item| wire_current_call(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_run_text(value: &c::RunText, limits: &w::Limits) -> Result<w::RunText, w::Problem> {
    w::RunText::new(
        limits,
        w::RunTextParts {
            attempt: value.attempt,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            turn: wire_provisional_turn_name(&value.turn, limits)?,
            from: value.from,
            through: value.through,
            text: bytes::copy_of(value.text.bytes()),
            cut: value.cut,
            call: value.call.as_ref().map(|item| wire_live_call(item, limits)).transpose()?,
            writing: value.writing.as_ref().map(|item| wire_writing_call(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_provisional_turn_name(
    value: &c::ProvisionalTurnName,
    limits: &w::Limits,
) -> Result<w::ProvisionalTurnName, w::Problem> {
    w::ProvisionalTurnName::new(
        limits,
        w::ProvisionalTurnNameParts { attempt: value.attempt, number: value.number, retry: value.retry },
    )
}

pub(super) fn wire_live_call(value: &c::LiveCall, limits: &w::Limits) -> Result<w::LiveCall, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::LiveCall::Decoded(value) => w::LiveCall::Decoded(w::LiveCallDecoded::new(
            limits,
            w::LiveCallDecodedParts { value: wire_call(value, limits)? },
        )?),
        c::LiveCall::Raw(value) => w::LiveCall::Raw(w::LiveCallRaw::new(
            limits,
            w::LiveCallRawParts { value: wire_current_call(value, limits)? },
        )?),
    })
}

pub(super) fn wire_call(value: &c::Call, limits: &w::Limits) -> Result<w::Call, w::Problem> {
    w::Call::new(
        limits,
        w::CallParts {
            number: value.number,
            tool: bytes::copy_of(value.tool.bytes()),
            input: bytes::copy_of(value.input.bytes()),
            outcome: value
                .outcome
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                .transpose()?,
            failed: value.failed,
            objects: list(value.objects.iter().map(|item| wire_object_key(item, limits)))?,
            parts: empty_pieces(&value.parts)?,
        },
    )
}

pub(super) fn wire_current_call(value: &c::CurrentCall, limits: &w::Limits) -> Result<w::CurrentCall, w::Problem> {
    w::CurrentCall::new(
        limits,
        w::CurrentCallParts {
            identity: wire_call_identity(&value.identity, limits)?,
            tool: bytes::copy_of(value.tool.bytes()),
            input: bytes::copy_of(value.input.bytes()),
            cut: value.cut,
            began: value.began.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_writing_call(value: &c::WritingCall, limits: &w::Limits) -> Result<w::WritingCall, w::Problem> {
    w::WritingCall::new(
        limits,
        w::WritingCallParts {
            id: bytes::copy_of(value.id.bytes()),
            tool: bytes::copy_of(value.tool.bytes()),
            from: value.from,
            through: value.through,
            input: bytes::copy_of(value.input.bytes()),
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_accounts(
    value: &c::OwnerAccounts,
    limits: &w::Limits,
) -> Result<w::OwnerAccounts, w::Problem> {
    w::OwnerAccounts::new(
        limits,
        w::OwnerAccountsParts {
            skein_bools: [value.usable, value.settled, value.cut, value.readiness_cut],
            program_hosts: pieces(
                value.program_hosts.iter().map(|item| wire_owner_program_host(item, limits)),
                w::OwnerProgramHost::measure,
                w::OwnerProgramHost::encode,
            )?,
            accounts: pieces(
                value.accounts.iter().map(|item| wire_owner_account_line(item, limits)),
                w::OwnerAccountLine::measure,
                w::OwnerAccountLine::encode,
            )?,
            signing: pieces(
                value.signing.iter().map(|item| wire_owner_account_sign_in(item, limits)),
                w::OwnerAccountSignIn::measure,
                w::OwnerAccountSignIn::encode,
            )?,
            last_end: value.last_end.as_ref().map(|item| wire_owner_account_last_end(item, limits)).transpose()?,
            readiness: pieces(
                value.readiness.iter().map(|item| wire_payer_readiness(item, limits)),
                w::PayerReadiness::measure,
                w::PayerReadiness::encode,
            )?,
        },
    )
}

pub(super) fn wire_owner_program_host(
    value: &c::OwnerProgramHost,
    limits: &w::Limits,
) -> Result<w::OwnerProgramHost, w::Problem> {
    w::OwnerProgramHost::new(
        limits,
        w::OwnerProgramHostParts {
            name: bytes::copy_of(&value.name),
            channel: value.channel,
            connected: value.connected,
            awaiting_logins: value.awaiting_logins,
            programs: pieces(
                value.programs.iter().map(|item| wire_owner_program_login(item, limits)),
                w::OwnerProgramLogin::measure,
                w::OwnerProgramLogin::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(super) fn wire_owner_account_line(
    value: &c::OwnerAccountLine,
    limits: &w::Limits,
) -> Result<w::OwnerAccountLine, w::Problem> {
    w::OwnerAccountLine::new(
        limits,
        w::OwnerAccountLineParts {
            number: value.number,
            place: value.place,
            uses: value
                .uses
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_owner_account_use(item, limits))))
                .transpose()?,
            plan: value.plan,
            provider: value.provider.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            display: wire_document_text(&value.display, limits)?,
            holder: wire_owner_account_holder(&value.holder, limits)?,
            sources: pieces(
                value.sources.iter().map(|item| wire_owner_account_source(item, limits)),
                w::OwnerAccountSource::measure,
                w::OwnerAccountSource::encode,
            )?,
            cut: value.cut,
            provider_name: value.provider_name.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
            manage_usage: value
                .manage_usage
                .as_ref()
                .map(|item| Ok::<_, w::Problem>(bytes::copy_of(item)))
                .transpose()?,
        },
    )
}

pub(super) fn wire_owner_account_sign_in(
    value: &c::OwnerAccountSignIn,
    limits: &w::Limits,
) -> Result<w::OwnerAccountSignIn, w::Problem> {
    w::OwnerAccountSignIn::new(
        limits,
        w::OwnerAccountSignInParts {
            source: value.source,
            provider: value.provider,
            holder: wire_owner_account_holder(&value.holder, limits)?,
        },
    )
}

pub(super) fn wire_owner_account_last_end(
    value: &c::OwnerAccountLastEnd,
    limits: &w::Limits,
) -> Result<w::OwnerAccountLastEnd, w::Problem> {
    w::OwnerAccountLastEnd::new(
        limits,
        w::OwnerAccountLastEndParts { source: value.source, end: wire_owner_account_end(&value.end, limits)? },
    )
}

pub(super) fn wire_owner_account_end(
    value: &c::OwnerAccountEnd,
    limits: &w::Limits,
) -> Result<w::OwnerAccountEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountEnd::Made { account, declined } => w::OwnerAccountEnd::Made(w::OwnerAccountEndMade::new(
            limits,
            w::OwnerAccountEndMadeParts { account: *(account), declined: *(declined) },
        )?),
        c::OwnerAccountEnd::Renewed { account, declined } => {
            w::OwnerAccountEnd::Renewed(w::OwnerAccountEndRenewed::new(
                limits,
                w::OwnerAccountEndRenewedParts { account: *(account), declined: *(declined) },
            )?)
        }
        c::OwnerAccountEnd::Refused { refusal } => w::OwnerAccountEnd::Refused(w::OwnerAccountEndRefused::new(
            limits,
            w::OwnerAccountEndRefusedParts { refusal: wire_owner_account_refusal(refusal, limits)? },
        )?),
        c::OwnerAccountEnd::Failed { failure } => w::OwnerAccountEnd::Failed(w::OwnerAccountEndFailed::new(
            limits,
            w::OwnerAccountEndFailedParts { failure: wire_owner_account_failure(failure, limits)? },
        )?),
    })
}

pub(super) fn wire_owner_account_refusal(
    value: &c::OwnerAccountRefusal,
    limits: &w::Limits,
) -> Result<w::OwnerAccountRefusal, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountRefusal::NotLimited => w::OwnerAccountRefusal::NotLimited,
        c::OwnerAccountRefusal::Uses => w::OwnerAccountRefusal::Uses,
        c::OwnerAccountRefusal::NotKept { why } => w::OwnerAccountRefusal::NotKept(w::OwnerAccountRefusalNotKept::new(
            limits,
            w::OwnerAccountRefusalNotKeptParts { why: wire_owner_account_key_failure(why, limits)? },
        )?),
        c::OwnerAccountRefusal::Limit { bound } => w::OwnerAccountRefusal::Limit(w::OwnerAccountRefusalLimit::new(
            limits,
            w::OwnerAccountRefusalLimitParts { bound: wire_owner_account_bound(bound, limits)? },
        )?),
        c::OwnerAccountRefusal::Held { provider } => w::OwnerAccountRefusal::Held(w::OwnerAccountRefusalHeld::new(
            limits,
            w::OwnerAccountRefusalHeldParts { provider: *(provider) },
        )?),
        c::OwnerAccountRefusal::Provider { provider } => w::OwnerAccountRefusal::Provider(
            w::OwnerAccountRefusalProvider::new(limits, w::OwnerAccountRefusalProviderParts { provider: *(provider) })?,
        ),
        c::OwnerAccountRefusal::Subject => w::OwnerAccountRefusal::Subject,
        c::OwnerAccountRefusal::Unknown => w::OwnerAccountRefusal::Unknown,
        c::OwnerAccountRefusal::Numbers => w::OwnerAccountRefusal::Numbers,
        c::OwnerAccountRefusal::Kind { kind } => w::OwnerAccountRefusal::Kind(w::OwnerAccountRefusalKind::new(
            limits,
            w::OwnerAccountRefusalKindParts { kind: wire_owner_account_kind(kind, limits)? },
        )?),
        c::OwnerAccountRefusal::Closing => w::OwnerAccountRefusal::Closing,
        c::OwnerAccountRefusal::NotStarted => w::OwnerAccountRefusal::NotStarted,
        c::OwnerAccountRefusal::NotSignedIn => w::OwnerAccountRefusal::NotSignedIn,
        c::OwnerAccountRefusal::NotOurs => w::OwnerAccountRefusal::NotOurs,
        c::OwnerAccountRefusal::Cancelled => w::OwnerAccountRefusal::Cancelled,
        c::OwnerAccountRefusal::Moving => w::OwnerAccountRefusal::Moving,
    })
}

pub(super) fn wire_owner_account_key_failure(
    value: &c::OwnerAccountKeyFailure,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountKeyFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountKeyFailure::TooLong => w::OwnerAccountKeyFailure::TooLong,
        c::OwnerAccountKeyFailure::NotStored => w::OwnerAccountKeyFailure::NotStored,
    })
}

pub(super) fn wire_owner_account_bound(
    value: &c::OwnerAccountBound,
    _limits: &w::Limits,
) -> Result<w::OwnerAccountBound, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountBound::Accounts => w::OwnerAccountBound::Accounts,
        c::OwnerAccountBound::HolderAccounts => w::OwnerAccountBound::HolderAccounts,
        c::OwnerAccountBound::Sources => w::OwnerAccountBound::Sources,
        c::OwnerAccountBound::SignIns => w::OwnerAccountBound::SignIns,
    })
}

pub(super) fn wire_owner_account_failure(
    value: &c::OwnerAccountFailure,
    limits: &w::Limits,
) -> Result<w::OwnerAccountFailure, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAccountFailure::Later { after } => w::OwnerAccountFailure::Later(w::OwnerAccountFailureLater::new(
            limits,
            w::OwnerAccountFailureLaterParts {
                after: (after).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            },
        )?),
        c::OwnerAccountFailure::SignInAgain => w::OwnerAccountFailure::SignInAgain,
        c::OwnerAccountFailure::MissingScope { scope } => {
            w::OwnerAccountFailure::MissingScope(w::OwnerAccountFailureMissingScope::new(
                limits,
                w::OwnerAccountFailureMissingScopeParts { scope: *(scope) },
            )?)
        }
        c::OwnerAccountFailure::Misconfigured => w::OwnerAccountFailure::Misconfigured,
        c::OwnerAccountFailure::Unloaded { at, why } => {
            w::OwnerAccountFailure::Unloaded(w::OwnerAccountFailureUnloaded::new(
                limits,
                w::OwnerAccountFailureUnloadedParts {
                    at: wire_owner_account_place(at, limits)?,
                    why: wire_owner_account_unread(why, limits)?,
                },
            )?)
        }
        c::OwnerAccountFailure::Cancelled => w::OwnerAccountFailure::Cancelled,
    })
}

pub(super) fn wire_outcome(value: &c::Outcome, limits: &w::Limits) -> Result<w::Outcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Outcome::SettingsChanged(value) => {
            w::Outcome::SettingsChanged(wire_outcome_settings_changed(value, limits)?)
        }
        c::Outcome::MovedToNewest(value) => w::Outcome::MovedToNewest(wire_outcome_moved_to_newest(value, limits)?),
        c::Outcome::Owner(value) => w::Outcome::Owner(w::OutcomeOwner::new(
            limits,
            w::OutcomeOwnerParts { value: wire_owner_outcome(value, limits)? },
        )?),
        c::Outcome::Made { what } => {
            w::Outcome::Made(w::OutcomeMade::new(limits, w::OutcomeMadeParts { what: wire_made(what, limits)? })?)
        }
    })
}

pub(super) fn wire_outcome_settings_changed(
    value: &c::OutcomeSettingsChanged,
    limits: &w::Limits,
) -> Result<w::OutcomeSettingsChanged, w::Problem> {
    w::OutcomeSettingsChanged::new(limits, w::OutcomeSettingsChangedParts { revision: value.revision })
}

pub(super) fn wire_outcome_moved_to_newest(
    value: &c::OutcomeMovedToNewest,
    limits: &w::Limits,
) -> Result<w::OutcomeMovedToNewest, w::Problem> {
    w::OutcomeMovedToNewest::new(
        limits,
        w::OutcomeMovedToNewestParts { newest: value.newest, moved: value.moved, stayed: value.stayed },
    )
}

pub(super) fn wire_owner_outcome(value: &c::OwnerOutcome, limits: &w::Limits) -> Result<w::OwnerOutcome, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerOutcome::Lifted { hold, released } => w::OwnerOutcome::Lifted(w::OwnerOutcomeLifted::new(
            limits,
            w::OwnerOutcomeLiftedParts { hold: *(hold), released: *(released) },
        )?),
        c::OwnerOutcome::AccountFinishing { source } => w::OwnerOutcome::AccountFinishing(
            w::OwnerOutcomeAccountFinishing::new(limits, w::OwnerOutcomeAccountFinishingParts { source: *(source) })?,
        ),
        c::OwnerOutcome::Created { first, last, given } => w::OwnerOutcome::Created(w::OwnerOutcomeCreated::new(
            limits,
            w::OwnerOutcomeCreatedParts {
                first: *(first),
                last: *(last),
                given: (given).as_ref().map(|item| wire_created_given(item, limits)).transpose()?,
            },
        )?),
        c::OwnerOutcome::Compound { project, operations, first_message } => {
            w::OwnerOutcome::Compound(w::OwnerOutcomeCompound::new(
                limits,
                w::OwnerOutcomeCompoundParts {
                    project: *(project),
                    operations: *(operations),
                    first_message: *(first_message),
                },
            )?)
        }
        c::OwnerOutcome::MarkedSeen { seen } => w::OwnerOutcome::MarkedSeen(w::OwnerOutcomeMarkedSeen::new(
            limits,
            w::OwnerOutcomeMarkedSeenParts { seen: wire_seen_through(seen, limits)? },
        )?),
        c::OwnerOutcome::NoteEdited { name } => w::OwnerOutcome::NoteEdited(w::OwnerOutcomeNoteEdited::new(
            limits,
            w::OwnerOutcomeNoteEditedParts { name: *(name) },
        )?),
        c::OwnerOutcome::ServiceMade { person } => w::OwnerOutcome::ServiceMade(w::OwnerOutcomeServiceMade::new(
            limits,
            w::OwnerOutcomeServiceMadeParts { person: *(person) },
        )?),
        c::OwnerOutcome::Adopted { project } => w::OwnerOutcome::Adopted(w::OwnerOutcomeAdopted::new(
            limits,
            w::OwnerOutcomeAdoptedParts { project: *(project) },
        )?),
        c::OwnerOutcome::Stopped { task } => w::OwnerOutcome::Stopped(w::OwnerOutcomeStopped::new(
            limits,
            w::OwnerOutcomeStoppedParts { task: *(task) },
        )?),
        c::OwnerOutcome::Cancelled { task } => w::OwnerOutcome::Cancelled(w::OwnerOutcomeCancelled::new(
            limits,
            w::OwnerOutcomeCancelledParts { task: *(task) },
        )?),
        c::OwnerOutcome::Released { task } => w::OwnerOutcome::Released(w::OwnerOutcomeReleased::new(
            limits,
            w::OwnerOutcomeReleasedParts { task: *(task) },
        )?),
        c::OwnerOutcome::PersonTaken { task } => w::OwnerOutcome::PersonTaken(w::OwnerOutcomePersonTaken::new(
            limits,
            w::OwnerOutcomePersonTakenParts { task: *(task) },
        )?),
        c::OwnerOutcome::PersonHandedBack { task } => w::OwnerOutcome::PersonHandedBack(
            w::OwnerOutcomePersonHandedBack::new(limits, w::OwnerOutcomePersonHandedBackParts { task: *(task) })?,
        ),
        c::OwnerOutcome::PersonAnswered { task } => w::OwnerOutcome::PersonAnswered(
            w::OwnerOutcomePersonAnswered::new(limits, w::OwnerOutcomePersonAnsweredParts { task: *(task) })?,
        ),
        c::OwnerOutcome::Moved { task } => {
            w::OwnerOutcome::Moved(w::OwnerOutcomeMoved::new(limits, w::OwnerOutcomeMovedParts { task: *(task) })?)
        }
        c::OwnerOutcome::ProposalDecided { proposal, by, at, choice, repeated } => {
            w::OwnerOutcome::ProposalDecided(w::OwnerOutcomeProposalDecided::new(
                limits,
                w::OwnerOutcomeProposalDecidedParts {
                    proposal: wire_proposal_key(proposal, limits)?,
                    by: *(by),
                    at: (at).as_nanos(),
                    choice: wire_owner_proposal_choice(choice, limits)?,
                    repeated: *(repeated),
                },
            )?)
        }
        c::OwnerOutcome::Said { task, message } => w::OwnerOutcome::Said(w::OwnerOutcomeSaid::new(
            limits,
            w::OwnerOutcomeSaidParts { task: *(task), message: *(message) },
        )?),
        c::OwnerOutcome::QuestionAnswered { task, question, message } => {
            w::OwnerOutcome::QuestionAnswered(w::OwnerOutcomeQuestionAnswered::new(
                limits,
                w::OwnerOutcomeQuestionAnsweredParts { task: *(task), question: *(question), message: *(message) },
            )?)
        }
        c::OwnerOutcome::Amended { task } => w::OwnerOutcome::Amended(w::OwnerOutcomeAmended::new(
            limits,
            w::OwnerOutcomeAmendedParts { task: *(task) },
        )?),
        c::OwnerOutcome::Proposed { proposal, to } => w::OwnerOutcome::Proposed(w::OwnerOutcomeProposed::new(
            limits,
            w::OwnerOutcomeProposedParts { proposal: *(proposal), to: *(to) },
        )?),
        c::OwnerOutcome::Withdrawn { proposal } => w::OwnerOutcome::Withdrawn(w::OwnerOutcomeWithdrawn::new(
            limits,
            w::OwnerOutcomeWithdrawnParts { proposal: *(proposal) },
        )?),
        c::OwnerOutcome::RolesSet { project } => w::OwnerOutcome::RolesSet(w::OwnerOutcomeRolesSet::new(
            limits,
            w::OwnerOutcomeRolesSetParts { project: *(project) },
        )?),
        c::OwnerOutcome::PolicyChanged { project } => w::OwnerOutcome::PolicyChanged(
            w::OwnerOutcomePolicyChanged::new(limits, w::OwnerOutcomePolicyChangedParts { project: *(project) })?,
        ),
        c::OwnerOutcome::Renamed { project } => w::OwnerOutcome::Renamed(w::OwnerOutcomeRenamed::new(
            limits,
            w::OwnerOutcomeRenamedParts { project: *(project) },
        )?),
        c::OwnerOutcome::ProjectRemoved { project } => w::OwnerOutcome::ProjectRemoved(
            w::OwnerOutcomeProjectRemoved::new(limits, w::OwnerOutcomeProjectRemovedParts { project: *(project) })?,
        ),
        c::OwnerOutcome::ProjectMade { project } => w::OwnerOutcome::ProjectMade(w::OwnerOutcomeProjectMade::new(
            limits,
            w::OwnerOutcomeProjectMadeParts { project: *(project) },
        )?),
        c::OwnerOutcome::PoolSet { project, person } => w::OwnerOutcome::PoolSet(w::OwnerOutcomePoolSet::new(
            limits,
            w::OwnerOutcomePoolSetParts { project: *(project), person: *(person) },
        )?),
        c::OwnerOutcome::EscalationDecided { task, revision, by, at, choice } => {
            w::OwnerOutcome::EscalationDecided(w::OwnerOutcomeEscalationDecided::new(
                limits,
                w::OwnerOutcomeEscalationDecidedParts {
                    task: *(task),
                    revision: wire_revision(revision, limits)?,
                    by: *(by),
                    at: (at).as_nanos(),
                    choice: wire_owner_escalation_choice(choice, limits)?,
                },
            )?)
        }
        c::OwnerOutcome::AccountKeyKept { source } => w::OwnerOutcome::AccountKeyKept(
            w::OwnerOutcomeAccountKeyKept::new(limits, w::OwnerOutcomeAccountKeyKeptParts { source: *(source) })?,
        ),
        c::OwnerOutcome::AccountProgramDeclared { source, name } => {
            w::OwnerOutcome::AccountProgramDeclared(w::OwnerOutcomeAccountProgramDeclared::new(
                limits,
                w::OwnerOutcomeAccountProgramDeclaredParts { source: *(source), name: bytes::copy_of(name) },
            )?)
        }
        c::OwnerOutcome::AccountChanged => w::OwnerOutcome::AccountChanged,
        c::OwnerOutcome::AccountMovedIn { account, renewed } => {
            w::OwnerOutcome::AccountMovedIn(w::OwnerOutcomeAccountMovedIn::new(
                limits,
                w::OwnerOutcomeAccountMovedInParts { account: *(account), renewed: *(renewed) },
            )?)
        }
        c::OwnerOutcome::AccountRemoved { confirmed, failed_hosts } => {
            w::OwnerOutcome::AccountRemoved(w::OwnerOutcomeAccountRemoved::new(
                limits,
                w::OwnerOutcomeAccountRemovedParts {
                    confirmed: *(confirmed),
                    failed_hosts: list((failed_hosts).iter().map(|item| wire_account_host_failure(item, limits)))?,
                },
            )?)
        }
        c::OwnerOutcome::AccountSignInStarted { source, address } => {
            w::OwnerOutcome::AccountSignInStarted(w::OwnerOutcomeAccountSignInStarted::new(
                limits,
                w::OwnerOutcomeAccountSignInStartedParts { source: *(source), address: bytes::copy_of(address) },
            )?)
        }
        c::OwnerOutcome::AccountSignedOut { confirmed, failed_hosts } => {
            w::OwnerOutcome::AccountSignedOut(w::OwnerOutcomeAccountSignedOut::new(
                limits,
                w::OwnerOutcomeAccountSignedOutParts {
                    confirmed: *(confirmed),
                    failed_hosts: list((failed_hosts).iter().map(|item| wire_account_host_failure(item, limits)))?,
                },
            )?)
        }
    })
}

pub(super) fn wire_created_given(value: &c::CreatedGiven, limits: &w::Limits) -> Result<w::CreatedGiven, w::Problem> {
    w::CreatedGiven::new(limits, w::CreatedGivenParts { task: value.task, amount: value.amount })
}

pub(super) fn wire_owner_escalation_choice(
    value: &c::OwnerEscalationChoice,
    _limits: &w::Limits,
) -> Result<w::OwnerEscalationChoice, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerEscalationChoice::Released => w::OwnerEscalationChoice::Released,
        c::OwnerEscalationChoice::Rejected => w::OwnerEscalationChoice::Rejected,
        c::OwnerEscalationChoice::Passed => w::OwnerEscalationChoice::Passed,
    })
}

pub(super) fn wire_account_host_failure(
    value: &c::AccountHostFailure,
    limits: &w::Limits,
) -> Result<w::AccountHostFailure, w::Problem> {
    w::AccountHostFailure::new(
        limits,
        w::AccountHostFailureParts {
            name: bytes::copy_of(&value.name),
            why: wire_account_host_why(&value.why, limits)?,
        },
    )
}

pub(super) fn wire_account_host_why(
    value: &c::AccountHostWhy,
    _limits: &w::Limits,
) -> Result<w::AccountHostWhy, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AccountHostWhy::NotInContact => w::AccountHostWhy::NotInContact,
        c::AccountHostWhy::Lost => w::AccountHostWhy::Lost,
        c::AccountHostWhy::Failed => w::AccountHostWhy::Failed,
    })
}

pub(super) fn wire_made(value: &c::Made, limits: &w::Limits) -> Result<w::Made, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Made::Tasks { tasks } => w::Made::Tasks(w::MadeTasks::new(
            limits,
            w::MadeTasksParts { tasks: list((tasks).iter().map(|item| Ok::<_, w::Problem>(*(item))))? },
        )?),
        c::Made::Service { party } => w::Made::Service(w::MadeService::new(
            limits,
            w::MadeServiceParts { party: *(party), credential: skein_lib::Secret::new(Box::new([])) },
        )?),
        c::Made::PersonResult { task } => {
            w::Made::PersonResult(w::MadePersonResult::new(limits, w::MadePersonResultParts { task: *(task) })?)
        }
        c::Made::Note { note, scope } => w::Made::Note(w::MadeNote::new(
            limits,
            w::MadeNoteParts { note: *(note), scope: wire_note_scope(scope, limits)? },
        )?),
        c::Made::MovingOut { account, number, record } => w::Made::MovingOut(w::MadeMovingOut::new(
            limits,
            w::MadeMovingOutParts { account: *(account), number: *(number), record: (record).clone() },
        )?),
    })
}

pub(super) fn wire_phase(value: &c::Phase, _limits: &w::Limits) -> Result<w::Phase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Phase::Waiting => w::Phase::Waiting,
        c::Phase::Idle => w::Phase::Idle,
        c::Phase::Due => w::Phase::Due,
        c::Phase::Preparing => w::Phase::Preparing,
        c::Phase::Claimed => w::Phase::Claimed,
        c::Phase::Running => w::Phase::Running,
        c::Phase::BackingOff => w::Phase::BackingOff,
        c::Phase::Active => w::Phase::Active,
        c::Phase::Closing => w::Phase::Closing,
        c::Phase::Held => w::Phase::Held,
        c::Phase::Ended => w::Phase::Ended,
        c::Phase::Done => w::Phase::Done,
        c::Phase::Failed => w::Phase::Failed,
        c::Phase::Cancelled => w::Phase::Cancelled,
    })
}

pub(super) fn wire_chip_fields(
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
            executor: wire_executor(&value.executor, limits)?,
            phase: wire_phase(&value.phase, limits)?,
            details: facts.chip_phase(&value.details, Some(parent), limits)?,
        },
    )
}

pub(super) fn wire_owner_chip_phase(
    value: &c::OwnerChipPhase,
    limits: &w::Limits,
) -> Result<w::OwnerChipPhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerChipPhase::Waiting { dependencies, more, holds } => {
            w::OwnerChipPhase::Waiting(w::OwnerChipPhaseWaiting::new(
                limits,
                w::OwnerChipPhaseWaitingParts {
                    dependencies: list((dependencies).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                    more: *(more),
                    holds: *(holds),
                },
            )?)
        }
        c::OwnerChipPhase::Active(value) => w::OwnerChipPhase::Active(w::OwnerChipPhaseActive::new(
            limits,
            w::OwnerChipPhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::OwnerChipPhase::Person { address, taken_by, since } => {
            w::OwnerChipPhase::Person(w::OwnerChipPhasePerson::new(
                limits,
                w::OwnerChipPhasePersonParts {
                    address: wire_person_address(address, limits)?,
                    taken_by: (taken_by).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
                    since: (since).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
                },
            )?)
        }
        c::OwnerChipPhase::Closing(value) => w::OwnerChipPhase::Closing(w::OwnerChipPhaseClosing::new(
            limits,
            w::OwnerChipPhaseClosingParts { value: wire_closing_stage(value, limits)? },
        )?),
        c::OwnerChipPhase::Held { reason, revision } => w::OwnerChipPhase::Held(w::OwnerChipPhaseHeld::new(
            limits,
            w::OwnerChipPhaseHeldParts { reason: wire_hold_category(reason, limits)?, revision: *(revision) },
        )?),
        c::OwnerChipPhase::Ended => w::OwnerChipPhase::Ended,
    })
}

pub(super) fn wire_chip_phase_owner_fields(
    value: &c::ChipPhase,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipPhaseOwnerFields, w::Problem> {
    w::ChipPhaseOwnerFields::new(
        limits,
        w::ChipPhaseOwnerFieldsParts {
            phase: wire_owner_chip_phase(&value.phase, limits)?,
            details: facts.owner_chip_details(&value.details, Some(parent), limits)?,
        },
    )
}

pub(super) fn wire_closing_fields(
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

pub(super) fn wire_actor(value: &c::Actor, limits: &w::Limits) -> Result<w::Actor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Actor::Party(value) => w::Actor::Party(w::ActorParty::new(limits, w::ActorPartyParts { value: *(value) })?),
        c::Actor::Task(value) => w::Actor::Task(w::ActorTask::new(limits, w::ActorTaskParts { value: *(value) })?),
        c::Actor::Deployment => w::Actor::Deployment,
        c::Actor::ProjectDeployment { project } => w::Actor::ProjectDeployment(w::ActorProjectDeployment::new(
            limits,
            w::ActorProjectDeploymentParts { project: *(project) },
        )?),
    })
}

pub(super) fn wire_party(value: &c::Party, limits: &w::Limits) -> Result<w::Party, w::Problem> {
    w::Party::new(
        limits,
        w::PartyParts {
            number: value.number,
            kind: wire_party_kind(&value.kind, limits)?,
            name: bytes::copy_of(value.name.bytes()),
        },
    )
}

pub(super) fn wire_decision(value: &c::Decision, limits: &w::Limits) -> Result<w::Decision, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Decision::Accepted => w::Decision::Accepted,
        c::Decision::Rejected { reason } => w::Decision::Rejected(w::DecisionRejected::new(
            limits,
            w::DecisionRejectedParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
        c::Decision::PassedUp => w::Decision::PassedUp,
        c::Decision::Released { again } => w::Decision::Released(w::DecisionReleased::new(
            limits,
            w::DecisionReleasedParts { again: (again).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()? },
        )?),
        c::Decision::LeftHeld { reason } => w::Decision::LeftHeld(w::DecisionLeftHeld::new(
            limits,
            w::DecisionLeftHeldParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
    })
}

pub(super) fn wire_ending_fields(
    value: &c::Ending,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::EndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Ending::Task(value) => w::EndingFields::Task(w::EndingTaskFields::new(
            limits,
            w::EndingTaskFieldsParts { value: facts.owner_chip_ending(value, Some(parent), limits)? },
        )?),
        c::Ending::Done => w::EndingFields::Done,
        c::Ending::Failed { reason } => w::EndingFields::Failed(w::EndingFailedFields::new(
            limits,
            w::EndingFailedFieldsParts {
                reason: (reason)
                    .as_ref()
                    .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                    .transpose()?,
            },
        )?),
        c::Ending::Cancelled { by, reason, at } => w::EndingFields::Cancelled(w::EndingCancelledFields::new(
            limits,
            w::EndingCancelledFieldsParts {
                by: wire_actor(by, limits)?,
                reason: (reason)
                    .as_ref()
                    .map(|item| Ok::<_, w::Problem>(bytes::copy_of((item).bytes())))
                    .transpose()?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::Ending::Withdrawn { reason } => w::EndingFields::Withdrawn(w::EndingWithdrawnFields::new(
            limits,
            w::EndingWithdrawnFieldsParts { reason: bytes::copy_of((reason).bytes()) },
        )?),
        c::Ending::Decided { by, party, how, at } => w::EndingFields::Decided(w::EndingDecidedFields::new(
            limits,
            w::EndingDecidedFieldsParts {
                by: *(by),
                party: (party).as_ref().map(|item| wire_party(item, limits)).transpose()?,
                how: (how).as_ref().map(|item| wire_decision(item, limits)).transpose()?,
                at: (at).as_nanos(),
            },
        )?),
        c::Ending::Deleted => w::EndingFields::Deleted,
    })
}

pub(super) fn wire_held_runtime_fields(
    value: &c::HeldRuntime,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::HeldRuntimeFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HeldRuntime::Waiting => w::HeldRuntimeFields::Waiting,
        c::HeldRuntime::Active(value) => w::HeldRuntimeFields::Active(w::HeldRuntimeActiveFields::new(
            limits,
            w::HeldRuntimeActiveFieldsParts { value: wire_active_run(value, limits)? },
        )?),
        c::HeldRuntime::Closing(value) => w::HeldRuntimeFields::Closing(w::HeldRuntimeClosingFields::new(
            limits,
            w::HeldRuntimeClosingFieldsParts { value: facts.closing(value, Some(parent), limits)? },
        )?),
    })
}

pub(super) fn wire_attempt_failure(
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

pub(super) fn wire_hold_reason_fields(
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

pub(super) fn wire_owner_person_wait(
    value: &c::OwnerPersonWait,
    limits: &w::Limits,
) -> Result<w::OwnerPersonWait, w::Problem> {
    w::OwnerPersonWait::new(
        limits,
        w::OwnerPersonWaitParts {
            since: value.since.as_nanos(),
            stall: value.stall,
            deadline: value.deadline.as_nanos(),
            notice: value.notice.as_ref().map(|item| wire_owner_person_stall_notice(item, limits)).transpose()?,
        },
    )
}

pub(super) fn wire_owner_person_stall_notice(
    value: &c::OwnerPersonStallNotice,
    limits: &w::Limits,
) -> Result<w::OwnerPersonStallNotice, w::Problem> {
    w::OwnerPersonStallNotice::new(
        limits,
        w::OwnerPersonStallNoticeParts { message: value.message, at: value.at.as_nanos(), activity: value.activity },
    )
}

pub(super) fn wire_owner_chip_observation(
    value: &c::OwnerChipObservation,
    limits: &w::Limits,
) -> Result<w::OwnerChipObservation, w::Problem> {
    w::OwnerChipObservation::new(
        limits,
        w::OwnerChipObservationParts {
            channel: value.channel.raw(),
            work: wire_owner_chip_work(&value.work, limits)?,
            since: value.since.as_nanos(),
            reported_at: value.reported_at.as_nanos(),
            started: value.started.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(super) fn wire_owner_chip_work(
    value: &c::OwnerChipWork,
    limits: &w::Limits,
) -> Result<w::OwnerChipWork, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerChipWork::Preparing => w::OwnerChipWork::Preparing,
        c::OwnerChipWork::Starting => w::OwnerChipWork::Starting,
        c::OwnerChipWork::Active => w::OwnerChipWork::Active,
        c::OwnerChipWork::Writing => w::OwnerChipWork::Writing,
        c::OwnerChipWork::Tool { call, tool } => w::OwnerChipWork::Tool(w::OwnerChipWorkTool::new(
            limits,
            w::OwnerChipWorkToolParts { call: bytes::copy_of(call), tool: bytes::copy_of(tool) },
        )?),
        c::OwnerChipWork::Waiting => w::OwnerChipWork::Waiting,
        c::OwnerChipWork::Ending => w::OwnerChipWork::Ending,
    })
}

pub(super) fn wire_owner_chip_details_fields(
    value: &c::OwnerChipDetails,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::OwnerChipDetailsFields, w::Problem> {
    w::OwnerChipDetailsFields::new(
        limits,
        w::OwnerChipDetailsFieldsParts {
            project: value.project,
            executor: wire_owner_executor(&value.executor, limits)?,
            phase_since: value.phase_since.as_nanos(),
            run_since: value.run_since.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            retry: value.retry.as_ref().map(|item| wire_retry_notice(item, limits)).transpose()?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            person_wait: value.person_wait.as_ref().map(|item| wire_owner_person_wait(item, limits)).transpose()?,
            hold: value.hold.as_ref().map(|item| wire_owner_chip_hold(item, limits)).transpose()?,
            budget: wire_owner_budget(&value.budget, limits)?,
            activity: value.activity,
            resources: pieces(
                value.resources.iter().map(|item| wire_owner_resource_line(item, limits)),
                w::OwnerResourceLine::measure,
                w::OwnerResourceLine::encode,
            )?,
            resources_cut: value.resources_cut,
            resources_observed: value.resources_observed,
            procedure: value.procedure.as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
            live_delegates: value.live_delegates,
            closing_connectors: value
                .closing_connectors
                .as_ref()
                .map(|item| list((item).iter().map(|item| Ok::<_, w::Problem>(*(item)))))
                .transpose()?,
            ending: value
                .ending
                .as_ref()
                .map(|item| facts.owner_chip_ending(item, Some(parent), limits))
                .transpose()?,
            ended_at: value.ended_at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            observation: value
                .observation
                .as_ref()
                .map(|item| wire_owner_chip_observation(item, limits))
                .transpose()?,
            dependencies: list(value.dependencies.iter().map(|item| wire_dependency(item, limits)))?,
        },
    )
}

pub(super) fn wire_owner_chip_ending_fields(
    value: &c::OwnerChipEnding,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::OwnerChipEndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerChipEnding::Done(value) => w::OwnerChipEndingFields::Done(w::OwnerChipEndingDoneFields::new(
            limits,
            w::OwnerChipEndingDoneFieldsParts { value: facts.result_value(value, Some(parent), limits)? },
        )?),
        c::OwnerChipEnding::Failed(value) => w::OwnerChipEndingFields::Failed(w::OwnerChipEndingFailedFields::new(
            limits,
            w::OwnerChipEndingFailedFieldsParts { value: wire_document_text(value, limits)? },
        )?),
        c::OwnerChipEnding::Cancelled { reason, result, by, at } => {
            w::OwnerChipEndingFields::Cancelled(w::OwnerChipEndingCancelledFields::new(
                limits,
                w::OwnerChipEndingCancelledFieldsParts {
                    reason: wire_document_text(reason, limits)?,
                    result: (result).as_ref().map(|item| facts.result_value(item, Some(parent), limits)).transpose()?,
                    by: wire_owner_requester(by, limits)?,
                    at: (at).as_nanos(),
                },
            )?)
        }
    })
}

pub(super) fn wire_result_value_fields(
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
                part: wire_owner_part(part, limits)?,
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
