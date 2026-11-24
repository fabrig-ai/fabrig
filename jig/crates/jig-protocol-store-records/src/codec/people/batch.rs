//! Party batch values (protocol/store/records.md, 1).
use crate::generated::people as mapped;
pub(crate) mod parts;
use super::pieces::{decode_spec_parameters, encode_spec_parameters, shared_sum, split};
use super::policy::{authority_to_domain, authority_to_record};
use super::{Box, List, Reader, count, domain, problem, wire};
use wire::Record;
pub(crate) fn member_to_record(member: &domain::Member, limits: &wire::Limits) -> Result<wire::Member, wire::Problem> {
    let mut symbols = List::with_capacity(limits.member_symbols);
    for symbol in &member.symbols {
        let value = mapped::symbol_to_record(symbol);
        if symbols.push(value).is_err() {
            return Err(wire::Problem { path: wire::paths::MEMBER_SYMBOLS, reason: skein_codec::Reason::Bound });
        }
    }
    Ok(wire::Member {
        llm: match &member.llm {
            Some(value) => Some(mapped::llm_choice_to_record(value)?),
            None => None,
        },
        executor: mapped::task_executor_to_record(&member.executor),
        words: member.words.clone(),
        parameters: encode_spec_parameters(&member.parameters, limits, shared_sum(limits), limits.member_parameters)?,
        inputs: super::pieces::encode_numbers(&member.inputs, limits, shared_sum(limits), limits.member_inputs)?,
        deadline: mapped::deadline_to_record(&member.deadline),
        budget: mapped::budget_to_record(&member.budget),
        named: super::pieces::encode_grant_piece(&member.named, limits, shared_sum(limits), limits.member_named)?,
        symbols,
        contract: encode_contract(&member.contract, limits)?,
        authority: authority_to_record(&member.authority, limits)?,
        wake: match &member.wake {
            Some(wake) => Some(mapped::wake_policy_to_record(wake)),
            None => None,
        },
        dependencies: parts::encode_dependencies(&member.dependencies, limits)?,
        tracked: member.tracked,
        label: member.label,
        flags: mapped::flags_to_record(&member.flags),
    })
}
pub(crate) fn member_to_domain(member: wire::Member, limits: &wire::Limits) -> Result<domain::Member, wire::Problem> {
    let wire::Member {
        executor,
        words,
        parameters,
        contract,
        authority,
        budget,
        deadline,
        inputs,
        named,
        symbols,
        wake,
        dependencies,
        tracked,
        label,
        flags,
        llm,
    } = member;
    let mut decoded_symbols = List::with_capacity(symbols.len());
    for symbol in symbols.into_boxed() {
        decoded_symbols.push(mapped::symbol_to_domain(symbol)).expect("decoded symbols bound");
    }

    Ok(domain::Member {
        symbols: decoded_symbols.into_boxed(),
        llm: match llm {
            Some(value) => Some(mapped::llm_choice_to_domain(value)),
            None => None,
        },
        deadline: mapped::deadline_to_domain(deadline),
        executor: mapped::task_executor_to_domain(executor),
        words,
        parameters: decode_spec_parameters(parameters, limits, shared_sum(limits))?,
        contract: decode_contract(contract, limits)?,
        authority: authority_to_domain(authority, limits)?,
        wake: match wake {
            Some(wake) => Some(mapped::wake_policy_to_domain(wake)),
            None => None,
        },
        dependencies: parts::decode_dependencies(dependencies, limits)?,
        inputs: super::pieces::decode_numbers(inputs, limits, shared_sum(limits))?,
        budget: mapped::budget_to_domain(budget),
        named: super::pieces::decode_grant_piece(named, limits, shared_sum(limits))?,
        tracked,
        label,
        flags: mapped::flags_to_domain(flags),
    })
}
fn encode_contract(value: &domain::Contract, limits: &wire::Limits) -> Result<wire::Contract, wire::Problem> {
    Ok(match value {
        domain::Contract::Change { connector, kind, words } => {
            wire::Contract::Change(wire::ContractChange { connector: *connector, kind: *kind, words: *words })
        }
        domain::Contract::Report { words } => wire::Contract::Report(wire::ContractReport { words: *words }),
        domain::Contract::Verdict { choices } => {
            let mut items = List::with_capacity(count(choices.len())?);
            for choice in choices {
                let value = wire::Verdict {
                    code: choice.code,
                    words: choice.words,
                    followups: choice.followups,
                    fields: encode_fields(&choice.fields, limits)?,
                };
                let writer = value.encode(limits)?;
                items.push(writer).expect("choice count");
            }
            wire::Contract::Verdict(wire::ContractVerdict { choices: joined(items, limits.contract_verdict_choices)? })
        }
    })
}
fn decode_contract(value: wire::Contract, limits: &wire::Limits) -> Result<domain::Contract, wire::Problem> {
    Ok(match value {
        wire::Contract::Change(value) => {
            let wire::ContractChange { connector, kind, words } = value;
            domain::Contract::Change { connector, kind, words }
        }
        wire::Contract::Report(value) => domain::Contract::Report { words: value.words },
        wire::Contract::Verdict(value) => {
            let bytes = value.choices;
            let mut reader = Reader::new(&bytes);
            let mut choices = List::with_capacity(count(bytes.len())?);
            for _ in 0..bytes.len() {
                let Some(bytes) = split(&mut reader)? else { break };
                let wire::Verdict { code, words, followups, fields } =
                    wire::Verdict::decode(limits, &mut Reader::new(&bytes))?;
                choices
                    .push(domain::Verdict { code, words, followups, fields: decode_fields(fields, limits)? })
                    .expect("framed choice count");
            }
            domain::Contract::Verdict { choices: choices.into_boxed() }
        }
    })
}
fn encode_fields(fields: &[domain::FieldRule], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    let mut items = List::with_capacity(count(fields.len())?);
    for field in fields {
        let value = mapped::field_rule_to_record(field);
        let writer = value.encode(limits)?;
        items.push(writer).expect("field count");
    }
    joined(items, limits.verdict_fields)
}
fn decode_fields(bytes: Box<[u8]>, limits: &wire::Limits) -> Result<Box<[domain::FieldRule]>, wire::Problem> {
    let mut reader = Reader::new(&bytes);
    let mut fields = List::with_capacity(count(bytes.len())?);
    for _ in 0..bytes.len() {
        let Some(bytes) = split(&mut reader)? else { break };
        let value = wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?;
        fields.push(mapped::field_rule_to_domain(value)).expect("framed field count");
    }
    Ok(fields.into_boxed())
}

fn joined(items: List<Box<[u8]>>, bound: u32) -> Result<Box<[u8]>, wire::Problem> {
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}

// Action's batch is a tuple containing a boxed slice, not a named record.
pub(crate) fn action_to_record(value: &domain::Action, limits: &wire::Limits) -> Result<wire::Action, wire::Problem> {
    Ok(match value {
        domain::Action::Batch(batch) => {
            let mut members = List::with_capacity(count(batch.len())?);
            for member in batch {
                members.push(member_to_record(member, limits)?).expect("member count");
            }
            wire::Action::Batch(wire::ActionBatch { members })
        }
        domain::Action::Amend { task, revision, amendment } => wire::Action::Amend(Box::new(wire::ActionAmend {
            task: *task,
            revision: *revision,
            amendment: super::amendment_to_record(amendment, limits)?,
        })),
        domain::Action::Settings { against, digest } => {
            wire::Action::Settings(wire::ActionSettings { against: *against, digest: *digest })
        }
    })
}
pub(crate) fn action_to_domain(value: wire::Action, limits: &wire::Limits) -> Result<domain::Action, wire::Problem> {
    Ok(match value {
        wire::Action::Batch(value) => {
            let mut members = List::with_capacity(value.members.len());
            for member in value.members.into_boxed() {
                members.push(member_to_domain(member, limits)?).expect("member count");
            }
            domain::Action::Batch(members.into_boxed())
        }
        wire::Action::Amend(value) => {
            let wire::ActionAmend { task, revision, amendment } = *value;
            domain::Action::Amend {
                task,
                revision,
                amendment: Box::new(super::amendment_to_domain(amendment, limits)?),
            }
        }
        wire::Action::Settings(value) => domain::Action::Settings { against: value.against, digest: value.digest },
    })
}
