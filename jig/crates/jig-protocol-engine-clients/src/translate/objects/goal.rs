//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{owner, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_tasks as t;
use skein_lib::{List, Writer, bytes};
pub fn pattern(value: &t::Pattern) -> Result<w::Pattern, w::Problem> {
    match mapped::pattern_to_record(value) {
        Ok(record) => Ok(record),
        Err(problem) => Err(super::super::problem(problem.reason)),
    }
}
pub fn authority(
    models: &[crate::Model],
    value: &t::Authority,
    limits: &w::Limits,
) -> Result<w::Authority, w::Problem> {
    let mut grants = List::with_capacity(super::super::count(value.grants.len())?);
    for row in &value.grants {
        grants.push(crate::translate::client::grant(row)?).expect("supplied rows fit");
    }
    let grants = owner::grants_pieces(&grants, limits.authority_grants, w::paths::AUTHORITY_GRANTS, limits)?;
    let delegation = mapped::delegation_to_record(&value.delegation);
    let delegation = match delegation {
        Ok(value) => value,
        Err(problem) => return Err(super::super::problem(problem.reason)),
    };
    let mut scopes = List::with_capacity(super::super::count(value.note_resources.len())?);
    for row in &value.note_resources {
        scopes.push(crate::translate::client::resource_scope(row)?).expect("supplied rows fit");
    }
    let scopes =
        owner::scopes_pieces(&scopes, limits.authority_note_resources, w::paths::AUTHORITY_NOTE_RESOURCES, limits)?;

    Ok(w::Authority {
        models: super::super::llm::authority(
            jig_domain_authority::Models::from_parts(value.models.bits(), value.models.is_all()),
            models,
        ),
        tools: crate::translate::convert::tools_to_record(&value.tools),
        grants,
        delegation,
        spend: value.budget.spend,
        deadline: match value.budget.deadline {
            Some(item) => Some(skein_codec::convert::wall_to_record(&item)),
            None => None,
        },
        notes: crate::translate::convert::scopes_to_record(value.notes),
        note_resources: scopes,
    })
}
pub fn spec(value: &t::Spec) -> Result<w::Specification, w::Problem> {
    let mut parameters = List::with_capacity(super::super::count(value.parameters.len())?);
    for row in &value.parameters {
        parameters.push(mapped::parameter_to_record(row)).expect("supplied parameters fit");
    }
    let mut inputs = List::with_capacity(super::super::count(value.inputs.len())?);
    for row in &value.inputs {
        inputs.push(*row).expect("supplied rows fit");
    }

    Ok(w::Specification { asks: value.asks, words: bytes::copy_of(&value.words), parameters, inputs })
}
pub fn contract(value: &t::Contract, limits: &w::Limits) -> Result<w::TaskContractDocument, w::Problem> {
    Ok(match value {
        t::Contract::Job { bytes } => w::TaskContractDocument::Job(w::TaskContractDocumentJob { bytes: *bytes }),
        t::Contract::Approval { state, part } => w::TaskContractDocument::Approval(w::TaskContractDocumentApproval {
            state: bytes::copy_of(state),
            part: owner::part(part),
        }),
        t::Contract::Report { words } => {
            w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: *words })
        }
        t::Contract::Change { connector, kind, words, .. } => {
            w::TaskContractDocument::Change(w::TaskContractDocumentChange {
                connector: *connector,
                kind: *kind,
                words: *words,
            })
        }
        t::Contract::Verdict { choices } => {
            let mut copied = List::with_capacity(super::super::count(choices.len())?);
            for row in choices {
                copied.push(crate::translate::client::verdict_choice(row)?).expect("supplied choices");
            }
            w::TaskContractDocument::Verdict(w::TaskContractDocumentVerdict {
                choices: owner::choices_pieces(
                    &copied,
                    limits.task_contract_document_verdict_choices,
                    w::paths::TASK_CONTRACT_DOCUMENT_VERDICT_CHOICES,
                    limits,
                )?,
                cut: false,
            })
        }
    })
}
pub fn goal(models: &[crate::Model], value: &t::New, limits: &w::Limits) -> Result<w::PersonProposalGoal, w::Problem> {
    if value.recurring.is_some() {
        return Err(w::Problem { path: w::paths::PERSON_PROPOSAL_GOAL, reason: skein_codec::Reason::Tag });
    }
    let mut dependencies = List::with_capacity(super::super::count(value.dependencies.len())?);
    for row in &value.dependencies {
        dependencies.push(*row).expect("supplied rows fit");
    }
    let mut holdings = List::with_capacity(super::super::count(value.holdings.len())?);
    for row in &value.holdings {
        let (name, kind) = match row {
            t::Holding::Write { resource, kind } => (resource, *kind),
            t::Holding::Slot { pool, kind } => (pool, *kind),
        };
        let mut path = List::with_capacity(super::super::count(name.path.len())?);
        for segment in &name.path {
            path.push(bytes::copy_of(segment)).expect("supplied path fits");
        }
        holdings
            .push(match row {
                t::Holding::Write { .. } => {
                    w::GoalHolding::Write(w::GoalHoldingWrite { connector: name.connector, path, kind })
                }
                t::Holding::Slot { .. } => {
                    w::GoalHolding::Slot(w::GoalHoldingSlot { connector: name.connector, path, kind })
                }
            })
            .expect("supplied holdings fit");
    }
    let holdings = goal_holding_pieces(
        &holdings,
        limits.person_proposal_goal_holdings,
        w::paths::PERSON_PROPOSAL_GOAL_HOLDINGS,
        limits,
    )?;

    Ok(w::PersonProposalGoal {
        number: value.number,
        project: value.project,
        executor: owner::executor(value.executor),
        spec: spec(&value.spec)?,
        contract: contract(&value.contract, limits)?,
        authority: authority(models, &value.authority, limits)?,
        numbers: owner::numbers(value.numbers),
        funder: owner::funder(value.funder),
        dependencies,
        holdings,
        wake: owner::wake(value.wake),
        tracked: value.tracked,
    })
}
pub(super) fn goal_holding_pieces(
    rows: &List<w::GoalHolding>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
