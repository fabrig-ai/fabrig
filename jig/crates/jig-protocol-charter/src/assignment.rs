//! Assemble the core's run charter with jig's contract policy
//! (jig/docs/design/domain/hosts.md, 8 and 8.1; domain/engine.md, 7.1).

use crate::{
    Budget, ChangeRule, Charter, Contract, FieldRule, Items, Llm, Mode, Model, Prices, Reserve, Section, TextRule,
    Tool, VerdictRule, WorkspaceTools,
};
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, List};

/// Maximum contract text and field bytes (jig/docs/design/domain/hosts.md, 8).
pub const CONTRACT_TEXT_MAX: u32 = 1024;
/// Delivery requires passing checks (jig/docs/design/domain/hosts.md, 8).
pub const CONTRACT_CHECKS_MUST_PASS: bool = true;
/// Every run can wait (jig/docs/design/domain/hosts.md, 8.1).
pub const WAIT_GRANTED: bool = true;

/// Build the charter translated by [`crate::charter`]. Sources are the
/// core's retained main and alternative bindings in `run.sources`.
/// `brief` holds the root-rendered sections in order; `resumed` says a
/// validated transcript accompanies the start. Provider identity belongs to
/// the start envelope and is derived by [`crate::run_identity`].
/// Returns none for a person's approval or missing model source bindings.
#[must_use]
pub fn assignment(run: core::RunCharter, brief: Box<[Section]>, tools: Box<[Tool]>, resumed: bool) -> Option<Charter> {
    let policy = run.policy;
    let mut models = List::with_capacity(u32::try_from(policy.alternatives.len()).ok()?);
    for (index, candidate) in policy.alternatives.into_iter().enumerate() {
        models.push(Llm { model: model(candidate), grant: *run.sources.get(index.checked_add(1)?)? }).ok()?;
    }
    Some(Charter {
        mode: match run.ended_by {
            tasks::EndedBy::Result => Mode::Finite,
            tasks::EndedBy::Requester => Mode::Open,
        },
        instructions: policy.instructions,
        brief,
        tools,
        wait: WAIT_GRANTED,
        agents: policy.agents,
        workspace: WorkspaceTools { inspect: policy.inspect, modify: policy.modify, shell: policy.shell },
        conventions: None,
        contract: contract(run.contract)?,
        budget: Budget {
            turns: policy.turns,
            spend: run.budget,
            time: policy.time,
            reserve: Reserve { turns: policy.reserve_turns, time: policy.reserve_time },
        },
        model: Llm { model: model(policy.model), grant: *run.sources.first()? },
        models: models.into_boxed(),
        waiting: policy.waiting,
        resumes: policy.resume && resumed,
    })
}

/// The numeric party label carried by a host message (hosts.md, 9).
#[must_use]
pub fn sender(value: tasks::Party) -> Box<[u8]> {
    match value {
        tasks::Party::Task(number) | tasks::Party::Person(number) => Box::from(Decimal::of(number).as_bytes()),
        tasks::Party::Deployment { .. } => Box::from(&b"deployment"[..]),
    }
}

fn model(value: core::Model) -> Model {
    Model {
        prices: Prices {
            input: value.input_price,
            cached: value.cached_price,
            cache_write: value.cache_write_price,
            output: value.output_price,
            unit: value.price_unit,
        },
        dialect: value.dialect,
        endpoint: value.endpoint,
        name: value.name,
        window: value.window,
        output: value.output,
        effort: value.effort,
    }
}
fn contract(value: tasks::Contract) -> Option<Contract> {
    Some(match value {
        tasks::Contract::Approval { .. } => return None,
        tasks::Contract::Report { words } => Contract {
            report: Some(TextRule { max: words.min(CONTRACT_TEXT_MAX), fields: Box::new([]) }),
            failure: Some(TextRule { max: words.min(CONTRACT_TEXT_MAX), fields: Box::new([]) }),
            verdicts: Box::new([]),
            change: None,
        },
        tasks::Contract::Verdict { choices } => {
            let mut verdicts = List::with_capacity(u32::try_from(choices.len()).expect("verdict choices"));
            for choice in choices {
                let mut fields = List::with_capacity(u32::try_from(choice.fields.len()).ok()?);
                for field in choice.fields {
                    fields.push(FieldRule { name: field.name, max: field.max.min(CONTRACT_TEXT_MAX) }).ok()?;
                }
                verdicts
                    .push(VerdictRule {
                        name: Box::from(Decimal::of(u64::from(choice.code)).as_bytes()),
                        text_max: choice.words.min(CONTRACT_TEXT_MAX),
                        fields: fields.into_boxed(),
                        items: Items { min: 0, max: 0, kinds: Box::new([]) },
                    })
                    .expect("one verdict rule");
            }
            Contract {
                report: None,
                failure: Some(TextRule { max: CONTRACT_TEXT_MAX, fields: Box::new([]) }),
                verdicts: verdicts.into_boxed(),
                change: None,
            }
        }
        tasks::Contract::Change { words, .. } => Contract {
            report: None,
            failure: Some(TextRule { max: words.min(CONTRACT_TEXT_MAX), fields: Box::new([]) }),
            verdicts: Box::new([VerdictRule {
                name: Box::from(&b"no_change"[..]),
                text_max: words.min(CONTRACT_TEXT_MAX),
                fields: Box::new([]),
                items: Items { min: 0, max: 0, kinds: Box::new([]) },
            }]),
            change: Some(ChangeRule {
                checks_must_pass: CONTRACT_CHECKS_MUST_PASS,
                fields: Box::new([
                    FieldRule { name: Box::from(&b"title"[..]), max: words.min(CONTRACT_TEXT_MAX) },
                    FieldRule { name: Box::from(&b"body"[..]), max: words.min(CONTRACT_TEXT_MAX) },
                ]),
            }),
        },
    })
}
