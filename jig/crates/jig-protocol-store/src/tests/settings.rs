//! Whole domain values cross the settings schema without an encoded-record envelope.
use super::fixtures;
use crate::codec;
use jig_domain_authority as a;
use jig_domain_core_vocabulary::{self as core, settings as s};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Wall};

fn model() -> core::Model {
    core::Model {
        provider: 1,
        dialect: 3,
        endpoint: 4,
        program: Some(Box::from(b"program".as_slice())),
        name: Box::from(b"model".as_slice()),
        cache_life_ms: 300_000,
        window: 8192,
        output: 1024,
        input_price: 1,
        cached_price: 2,
        cache_write_price: Some(3),
        output_price: 4,
        price_unit: 1000,
        efforts: Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]),
        effort: Some(Box::from(b"high".as_slice())),
        speeds: Box::default(),
        speed: None,
    }
}

pub(super) fn charter() -> core::Charter {
    core::Charter {
        name: None,
        parts: Box::new([
            core::CharterPart { on: tasks::Reasons::FIRST, instructions: Box::from(b"first part".as_slice()) },
            core::CharterPart {
                on: tasks::Reasons::QUESTION.union(tasks::Reasons::TIMER),
                instructions: Box::from(b"later part".as_slice()),
            },
        ]),
        rest: true,
        wake: jig_domain_tasks::WakePolicy::DEFAULT,
        documents: Box::new([]),
        number: 1,
        hosts: fleet::Kinds::Both,
        spend: 100,
        policy: core::RunPolicy {
            instructions: Box::from(b"instructions\nwhole".as_slice()),
            waiting: Duration::from_secs(2),
            resume: true,
            turns: 4,
            time: Duration::from_secs(30),
            reserve_turns: 1,
            reserve_time: Duration::from_secs(5),
            model: model(),
            alternatives: Box::new([model()]),
            inspect: true,
            modify: false,
            shell: true,
            agents: false,
            call_timeout: Duration::from_secs(1),
            tools: core::RunPolicy::EVERY_FAMILY,
        },
    }
}

fn pattern() -> a::Pattern {
    a::Pattern { segments: Box::new([Box::from(b"one".as_slice())]), last: a::Last::Open(Box::from(b"two".as_slice())) }
}

fn authority() -> a::Authority {
    a::Authority {
        tools: a::Tools(7),
        models: a::Models::from_parts(3, false),
        grants: Box::new([a::Grant { connector: 1, kind: 2, pattern: pattern() }]),
        delegation: a::Delegation {
            kinds: Box::new([a::Executor::Charter(1), a::Executor::Procedure(2), a::Executor::Role(0)]),
            tasks: 2,
            depth: 1,
        },
        budget: a::Budget { spend: 100, deadline: Some(Wall::from_nanos(50)) },
        notes: a::Scopes(7),
        note_resources: Box::new([a::ResourceScope { connector: 1, pattern: pattern() }]),
    }
}

fn requirement() -> a::Requirement {
    a::Requirement {
        connector: 1,
        kind: 2,
        pattern: pattern(),
        judge: a::Judge { connector: 1, requirement: 3, parameters: 4 },
        guard: a::Guard::Observed { freshness: Duration::from_secs(1) },
        must_be_guarded: true,
    }
}

fn rules() -> a::Rules {
    a::Rules {
        ceiling: authority(),
        period_spend: 500,
        minimum_run_spend: 1,
        maximum_run_spend: 100,
        implies: a::Implies::new(Box::new([]), 0).unwrap(),
        requirements: Box::new([requirement()]),
    }
}

pub(super) fn policy() -> core::ConfiguredPolicy {
    core::ConfiguredPolicy {
        name: None,
        policy: a::Policy {
            escalation_role: Some(0),
            ceiling: authority(),
            projections: Box::new([a::Grant { connector: 1, kind: 2, pattern: pattern() }]),
            period_spend: 400,
            roles: Box::new([a::Role {
                number: 0,
                authority: authority(),
                period_spend: 300,
                requests: a::Requests::ALL,
                decides: a::Proposals::ALL,
            }]),
            requirements: Box::new([requirement()]),
        },
        permissions: Box::new([people::PermissionRole { connector: 1, permission: 2, role: 0 }]),
        templates: Box::new([people::TemplateOverride {
            template: 1,
            executor: Some(people::TemplateExecutor::Chosen),
            budget: Some(people::TemplateBudget::Amount(20)),
            deadline: Some(people::TemplateDeadline::Span(Duration::from_secs(5))),
            priority: Some(3),
        }]),
    }
}

pub(super) fn template() -> core::Template {
    core::Template {
        name: None,
        resources: Box::new([]),
        number: 1,
        shown: Box::from(b"template".as_slice()),
        executor: people::TemplateExecutor::Charter(1),
        contract: tasks::Contract::Verdict {
            choices: Box::new([tasks::Verdict {
                code: 1,
                words: 3,
                followups: 0,
                fields: Box::new([tasks::FieldRule { name: Box::from(b"field".as_slice()), max: 8 }]),
            }]),
        },
        shallower: 1,
        budget: people::TemplateBudget::AsAllowed { cap: Some(99) },
        deadline: people::TemplateDeadline::Given,
        tracked: Some(4),
        flags: tasks::Flags {
            conversation: true,
            listed: false,
            retention: tasks::Retention::ForgottenOnceRead,
            ended_by: tasks::EndedBy::Requester,
        },
        label: 7,
        grace: Some(Duration::from_secs(1)),
        llm: Some(tasks::LlmChoice {
            later: Some(tasks::LlmLater {
                main: tasks::ModelEffort { model: 1, effort: Some(0), speed: None },
                subagents: Box::new([tasks::ModelEffort { model: 0, effort: None, speed: None }]),
            }),
            account: Some(3),
            main: tasks::ModelEffort { model: 0, effort: Some(1), speed: None },
            subagents: Box::new([tasks::ModelEffort { model: 1, effort: None, speed: None }]),
        }),
    }
}

pub(super) fn catalogue_name(name: &[u8]) -> s::ModelName {
    s::ModelName {
        name: Box::from(name),
        efforts: Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]),
    }
}

mod applications;
#[path = "models.rs"]
mod models;
mod replaced;
