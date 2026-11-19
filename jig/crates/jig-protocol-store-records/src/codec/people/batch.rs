//! Party batch values (protocol/store/records.md, 1).
pub(crate) mod parts;
use super::pieces::{decode_spec_parameters, encode_spec_parameters, shared_sum, split};
use super::policy::{decode_authority, decode_wake_policy, encode_authority, encode_wake_policy};
use super::{Box, List, Reader, count, domain, problem, wire};
use wire::Record;
pub(super) fn encode(batch: &[domain::Member], limits: &wire::Limits) -> Result<List<wire::Member>, wire::Problem> {
    let mut members = List::with_capacity(count(batch.len())?);
    for member in batch {
        let mut symbols = List::with_capacity(limits.member_symbols);
        for symbol in &member.symbols {
            let value = wire::Symbol { connector: symbol.connector, symbol: symbol.symbol };
            if symbols.push(value).is_err() {
                return Err(wire::Problem { path: wire::paths::MEMBER_SYMBOLS, reason: skein_codec::Reason::Bound });
            }
        }
        members
            .push(wire::Member {
                llm: super::llm::encode_llm(member.llm.as_ref(), limits)?,
                executor: parts::encode_executor(member.executor, limits),
                words: member.words.clone(),
                parameters: encode_spec_parameters(
                    &member.parameters,
                    limits,
                    shared_sum(limits),
                    limits.member_parameters,
                )?,
                inputs: super::pieces::encode_numbers(
                    &member.inputs,
                    limits,
                    shared_sum(limits),
                    limits.member_inputs,
                )?,
                deadline: match member.deadline {
                    domain::Deadline::Named => wire::Deadline::Named,
                    domain::Deadline::Span(value) => wire::Deadline::Span(wire::DeadlineSpan { value }),
                },
                budget: match member.budget {
                    domain::Budget::Named => wire::Budget::Named,
                    domain::Budget::AsAllowed { cap } => wire::Budget::AsAllowed(wire::BudgetAsAllowed { cap }),
                },
                named: super::pieces::encode_grant_piece(
                    &member.named,
                    limits,
                    shared_sum(limits),
                    limits.member_named,
                )?,
                symbols,
                contract: encode_contract(&member.contract, limits)?,
                authority: encode_authority(&member.authority, limits)?,
                wake: match &member.wake {
                    Some(wake) => Some(encode_wake_policy(wake, limits)),
                    None => None,
                },
                dependencies: parts::encode_dependencies(&member.dependencies, limits)?,
                tracked: member.tracked,
                label: member.label,
                flags: wire::Flags {
                    ended_by: match member.flags.ended_by {
                        domain::EndedBy::Result => wire::EndedBy::Result,
                        domain::EndedBy::Requester => wire::EndedBy::Requester,
                        domain::EndedBy::Standing => wire::EndedBy::Standing,
                        domain::EndedBy::Closer(closer) => {
                            wire::EndedBy::Closer(wire::Closer { connector: closer.connector, code: closer.code })
                        }
                    },
                    conversation: member.flags.conversation,
                    listed: member.flags.listed,
                    retention: match member.flags.retention {
                        domain::Retention::Kept => wire::Retention::Kept,
                        domain::Retention::ForgottenOnceRead => wire::Retention::ForgottenOnceRead,
                    },
                },
            })
            .expect("member count");
    }
    Ok(members)
}
pub(super) fn decode(batch: List<wire::Member>, limits: &wire::Limits) -> Result<Box<[domain::Member]>, wire::Problem> {
    let mut members = List::with_capacity(batch.len());
    for member in batch.into_boxed() {
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
            let wire::Symbol { connector, symbol } = symbol;
            decoded_symbols.push(domain::Symbol { connector, symbol }).expect("decoded symbols bound");
        }
        let wire::Flags { conversation, listed, retention, ended_by } = flags;
        members
            .push(domain::Member {
                symbols: decoded_symbols.into_boxed(),
                llm: super::llm::decode_llm(llm),
                deadline: match deadline {
                    wire::Deadline::Named => domain::Deadline::Named,
                    wire::Deadline::Span(value) => domain::Deadline::Span(value.value),
                },
                executor: parts::decode_executor(executor),
                words,
                parameters: decode_spec_parameters(parameters, limits, shared_sum(limits))?,
                contract: decode_contract(contract, limits)?,
                authority: decode_authority(authority, limits)?,
                wake: match wake {
                    Some(wake) => Some(decode_wake_policy(wake)),
                    None => None,
                },
                dependencies: parts::decode_dependencies(dependencies, limits)?,
                inputs: super::pieces::decode_numbers(inputs, limits, shared_sum(limits))?,
                budget: match budget {
                    wire::Budget::Named => domain::Budget::Named,
                    wire::Budget::AsAllowed(value) => domain::Budget::AsAllowed { cap: value.cap },
                },
                named: super::pieces::decode_grant_piece(named, limits, shared_sum(limits))?,
                tracked,
                label,
                flags: domain::Flags {
                    ended_by: match ended_by {
                        wire::EndedBy::Result => domain::EndedBy::Result,
                        wire::EndedBy::Requester => domain::EndedBy::Requester,
                        wire::EndedBy::Standing => domain::EndedBy::Standing,
                        wire::EndedBy::Closer(closer) => {
                            domain::EndedBy::Closer(domain::Closer { connector: closer.connector, code: closer.code })
                        }
                    },
                    conversation,
                    listed,
                    retention: match retention {
                        wire::Retention::Kept => domain::Retention::Kept,
                        wire::Retention::ForgottenOnceRead => domain::Retention::ForgottenOnceRead,
                    },
                },
            })
            .expect("member count");
    }
    Ok(members.into_boxed())
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
        let value = wire::FieldRule { name: field.name.clone(), max: field.max };
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
        let wire::FieldRule { name, max } = wire::FieldRule::decode(limits, &mut Reader::new(&bytes))?;
        fields.push(domain::FieldRule { name, max }).expect("framed field count");
    }
    Ok(fields.into_boxed())
}

fn joined(items: List<Box<[u8]>>, bound: u32) -> Result<Box<[u8]>, wire::Problem> {
    match crate::codec::pieces::join(items, bound) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(problem()),
    }
}
