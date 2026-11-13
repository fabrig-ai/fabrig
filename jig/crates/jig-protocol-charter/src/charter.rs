//! Translate the core charter into Smith's without adding policy
//! (domain/hosts/charters.md; smith's `run.md`, section 3).
//! A Finite report takes delivery only with the policy's modify grant and
//! the root's workspace delivery fact; its outcome keeps the contract's forms.

use alloc::boxed::Box;
use skein_lib::List;
use smith_domain_run as run;

use crate::boundary::{self, Charter};
use crate::{CONTRACT_CHECKS_MUST_PASS, CONTRACT_TEXT_MAX};

fn count(len: usize) -> Option<u32> {
    u32::try_from(len).ok()
}

fn model(llm: boundary::Llm) -> run::charter::Llm {
    let model = llm.model;
    run::charter::Llm {
        prices: run::Prices {
            input: model.prices.input,
            cached: model.prices.cached,
            cache_write: model.prices.cache_write,
            output: model.prices.output,
            unit: model.prices.unit,
        },
        dialect: model.dialect,
        account: llm.grant,
        endpoint: run::charter::Endpoint(model.endpoint),
        model: model.name,
        effort: model.effort,
        window: model.window,
        output: model.output,
    }
}

fn fields(fields: Box<[boundary::FieldRule]>) -> Option<Box<[run::outcome::FieldRule]>> {
    let mut mapped = List::with_capacity(count(fields.len())?);
    for field in fields {
        mapped.push(run::outcome::FieldRule { name: field.name, max: field.max }).ok()?;
    }
    Some(mapped.into_boxed())
}

fn text(rule: boundary::TextRule) -> Option<run::outcome::TextSpec> {
    Some(run::outcome::TextSpec { max: rule.max, fields: fields(rule.fields)? })
}

fn items(items: boundary::Items) -> Option<run::outcome::ItemSpec> {
    let mut kinds = List::with_capacity(count(items.kinds.len())?);
    for kind in items.kinds {
        kinds.push(run::outcome::ItemRule { kind: kind.kind, fields: fields(kind.fields)? }).ok()?;
    }
    Some(run::outcome::ItemSpec { min: items.min, max: items.max, kinds: kinds.into_boxed() })
}

fn outcome(contract: boundary::Contract) -> Option<run::outcome::OutcomeSpec> {
    let mut verdicts = List::with_capacity(count(contract.verdicts.len())?);
    for verdict in contract.verdicts {
        verdicts
            .push(run::outcome::VerdictRule {
                name: verdict.name,
                text_max: verdict.text_max,
                fields: fields(verdict.fields)?,
                items: items(verdict.items)?,
            })
            .ok()?;
    }
    Some(run::outcome::OutcomeSpec {
        change: match contract.change {
            Some(rule) => {
                Some(run::outcome::ChangeSpec { checks_must_pass: rule.checks_must_pass, fields: fields(rule.fields)? })
            }
            None => None,
        },
        verdicts: verdicts.into_boxed(),
        report: match contract.report {
            Some(rule) => Some(text(rule)?),
            None => None,
        },
        failure: match contract.failure {
            Some(rule) => Some(text(rule)?),
            None => None,
        },
    })
}

/// Move the core's bounded charter and ordered brief into Smith's start.
/// Resume requires both the run policy and its validated transcript.
#[must_use]
pub fn charter(charter: Charter) -> Option<run::Charter> {
    let mut sections = List::with_capacity(count(charter.brief.len())?);
    for section in charter.brief {
        sections.push(run::Section { title: section.title, text: section.text }).ok()?;
    }
    let mut tools = List::with_capacity(count(charter.tools.len())?);
    for tool in charter.tools {
        tools
            .push(run::charter::HostTool {
                reserve: false,
                name: tool.name,
                description: tool.description,
                schema: tool.schema,
                effect: match tool.effect {
                    boundary::ToolEffect::Read => run::HostEffect::Read,
                    boundary::ToolEffect::Write => run::HostEffect::Write,
                },
                timeout: tool.timeout,
            })
            .ok()?;
    }
    let mut models = List::with_capacity(count(charter.models.len())?);
    for candidate in charter.models {
        models.push(model(candidate)).ok()?;
    }
    let outcome = outcome(charter.contract)?;
    let deliver = match charter.mode {
        run::Mode::Finite => match &outcome.change {
            Some(change) => Some(change.clone()),
            None => {
                if outcome.report.is_some() && charter.workspace.modify && charter.delivers {
                    Some(run::outcome::ChangeSpec {
                        checks_must_pass: CONTRACT_CHECKS_MUST_PASS,
                        fields: Box::new([
                            run::outcome::FieldRule { name: Box::from(b"title".as_slice()), max: CONTRACT_TEXT_MAX },
                            run::outcome::FieldRule {
                                name: Box::from(b"body".as_slice()),
                                max: charter.words.max(CONTRACT_TEXT_MAX),
                            },
                        ]),
                    })
                } else {
                    None
                }
            }
        },
        run::Mode::Open => None,
    };
    Some(run::Charter {
        check_timeout: None,
        mode: charter.mode,
        resume: charter.resumes,
        waiting: charter.waiting,
        instructions: charter.instructions,
        brief: run::Brief { sections: sections.into_boxed() },
        conventions: match charter.conventions {
            Some(paths) => Some(run::Conventions { guide: paths.guide, checks: paths.checks }),
            None => None,
        },
        grants: run::charter::Grants {
            contract: 1,
            wait: charter.wait,
            tools: run::charter::Tools {
                inspect: charter.workspace.inspect,
                modify: charter.workspace.modify && deliver.is_some(),
                shell: charter.workspace.shell,
            },
            save: deliver.is_some() && outcome.change.is_some(),
            deliver,
            agents: charter.agents,
            host_tools: tools.into_boxed(),
        },
        outcome,
        budget: run::Budget { turns: charter.budget.turns, spend: charter.budget.spend, time: charter.budget.time },
        reserve: run::charter::Reserve { turns: charter.budget.reserve.turns, time: charter.budget.reserve.time },
        llm: model(charter.model),
        models: models.into_boxed(),
    })
}
