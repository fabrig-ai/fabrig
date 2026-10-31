//! Wire projections of proposed task facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
use super::{wire_field_rule, wire_part};
pub(in crate::codec) fn wire_task_contract_document(
    value: &c::TaskContractDocument,
    limits: &w::Limits,
) -> Result<w::TaskContractDocument, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskContractDocument::Approval { state, part } => {
            w::TaskContractDocument::Approval(w::TaskContractDocumentApproval::new(
                limits,
                w::TaskContractDocumentApprovalParts { state: bytes::copy_of(state), part: wire_part(part, limits)? },
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

pub(in crate::codec) fn wire_verdict_choice(
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

pub(in crate::codec) fn wire_funder(value: &c::Funder, limits: &w::Limits) -> Result<w::Funder, w::Problem> {
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
