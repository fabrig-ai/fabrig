//! Whole domain values cross the settings schema without an encoded-record envelope.
use super::fixtures;
use crate::codec;
use jig_domain_authority as a;
use jig_domain_core::{self as core, settings as s};
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Wall};

fn model() -> core::Model {
    core::Model {
        dialect: 3,
        endpoint: 4,
        program: Some(Box::from(b"program".as_slice())),
        name: Box::from(b"model".as_slice()),
        window: 8192,
        output: 1024,
        input_price: 1,
        cached_price: 2,
        cache_write_price: Some(3),
        output_price: 4,
        price_unit: 1000,
        efforts: Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]),
        effort: Some(Box::from(b"high".as_slice())),
    }
}
fn charter() -> core::Charter {
    core::Charter {
        wake: jig_domain_tasks::WakePolicy::DEFAULT,
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
fn policy() -> core::ConfiguredPolicy {
    core::ConfiguredPolicy {
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
fn template() -> core::Template {
    core::Template {
        resources: Box::new([]),
        number: 1,
        name: Box::from(b"template".as_slice()),
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
            account: Some(3),
            main: tasks::ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::new([tasks::ModelEffort { model: 1, effort: None }]),
        }),
    }
}
#[test]
fn each_settings_record_round_trips_its_whole_domain_value() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.run_bytes = 4096;
    domain.authority.executors = 3;
    let limits = codec::limits(&domain).unwrap();
    let head = s::Head {
        number: 3,
        against: 2,
        at: Wall::from_nanos(45),
        by: s::Decider::Party(7),
        proposal: Some((people::Proposer::Task(8), 9)),
        moved: true,
    };
    let mut rows = skein_lib::List::with_capacity(28);
    for row in [
        s::Record::Extent { oldest: 1 },
        s::Record::Revision {
            history: false,
            revision: s::Revision { head: s::Head { number: 1, against: 0, ..head.clone() }, items: Box::new([]) },
        },
        s::Record::Charter { charter: 1, revision: 2, value: s::CharterVersion { charter: Some(Box::new(charter())) } },
        s::Record::Charter { charter: 1, revision: 3, value: s::CharterVersion { charter: None } },
    ] {
        rows.push(row).expect("settings fixture count");
    }
    for (setting, value) in [
        (s::Setting::Charter(1), s::Value::Charter(Box::new(charter()))),
        (s::Setting::Template(1), s::Value::Template(Box::new(template()))),
        (s::Setting::Policy(1), s::Value::Policy(Box::new(policy()))),
        (s::Setting::Rules, s::Value::Rules(Box::new(rules()))),
    ] {
        rows.push(s::Record::Default { setting, default: s::Default { value: value.clone(), revision: 2 } })
            .expect("settings fixture count");
        rows.push(s::Record::Changed { setting, changed: s::Changed { value: Some(value.clone()), revision: 3 } })
            .expect("settings fixture count");
        rows.push(s::Record::Changed { setting, changed: s::Changed { value: None, revision: 3 } })
            .expect("settings fixture count");
        let items = Box::new([
            s::Item { setting, edit: s::Edit::Set(value) },
            s::Item { setting, edit: s::Edit::Removed },
            s::Item { setting, edit: s::Edit::Reset },
        ]);
        rows.push(s::Record::Revision {
            history: false,
            revision: s::Revision { head: head.clone(), items: items.clone() },
        })
        .expect("settings fixture count");
        rows.push(s::Record::Revision {
            history: true,
            revision: s::Revision { head: head.clone(), items: items.clone() },
        })
        .expect("settings fixture count");
        rows.push(s::Record::Proposal {
            proposer: people::Proposer::Party(6),
            proposal: 7,
            value: s::Proposal { against: 2, items },
        })
        .expect("settings fixture count");
    }
    let names = Box::new([catalogue_name(b"one"), catalogue_name(b"two")]);
    for row in rows.into_boxed() {
        let row = core::Record::Core(core::CoreRecord::Settings(row));
        let bytes = codec::encode(&row, &limits, &*names).expect("bounded settings row");
        assert_eq!(codec::decode(&row.key(), &bytes, &limits, &*names), Some(row.clone()), "{:?}", row.key());
        assert!(codec::decode(&row.key(), &bytes[..bytes.len() - 1], &limits, &*names).is_none());
    }
}

#[test]
fn frozen_model_masks_and_template_choices_follow_names_after_a_catalogue_reorder() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.authority.executors = 3;
    let limits = codec::limits(&domain).unwrap();
    let before = [catalogue_name(b"one"), catalogue_name(b"two")];
    let after = [before[1].clone(), before[0].clone()];
    let mut rules = rules();
    rules.ceiling.models = a::Models::Named(1);
    let rows = [
        s::Record::Changed {
            setting: s::Setting::Rules,
            changed: s::Changed { revision: 1, value: Some(s::Value::Rules(Box::new(rules))) },
        },
        s::Record::Changed {
            setting: s::Setting::Template(1),
            changed: s::Changed { revision: 1, value: Some(s::Value::Template(Box::new(template()))) },
        },
    ];
    for value in rows {
        let row = core::Record::Core(core::CoreRecord::Settings(value));
        let bytes = codec::encode(&row, &limits, &before).unwrap();
        let core::Record::Core(core::CoreRecord::Settings(s::Record::Changed { changed, .. })) =
            codec::decode(&row.key(), &bytes, &limits, &after).unwrap()
        else {
            panic!("changed setting")
        };
        match changed.value.unwrap() {
            s::Value::Rules(value) => assert_eq!(value.ceiling.models, a::Models::Named(2)),
            s::Value::Template(value) => {
                let choice = value.llm.unwrap();
                assert_eq!(choice.main.model, 1);
                assert_eq!(choice.subagents[0].model, 0);
            }
            s::Value::Policy(_) | s::Value::Charter(_) => panic!("fixture setting"),
        }
    }
}

fn catalogue_name(name: &[u8]) -> s::ModelName {
    s::ModelName {
        name: Box::from(name),
        efforts: Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]),
    }
}
#[test]
fn live_settings_remap_efforts_by_name_and_refuse_missing_efforts() {
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    let before = [catalogue_name(b"one"), catalogue_name(b"two")];
    let mut after = before.clone();
    after[0].efforts = Box::new([Box::from(b"high".as_slice()), Box::from(b"low".as_slice())]);
    let value = s::Value::Template(Box::new(template()));
    for original in [
        s::Record::Default {
            setting: s::Setting::Template(1),
            default: s::Default { value: value.clone(), revision: 2 },
        },
        s::Record::Changed {
            setting: s::Setting::Template(1),
            changed: s::Changed { value: Some(value.clone()), revision: 2 },
        },
        s::Record::Proposal {
            proposer: people::Proposer::Party(1),
            proposal: 1,
            value: s::Proposal {
                against: 1,
                items: Box::new([s::Item { setting: s::Setting::Template(1), edit: s::Edit::Set(value.clone()) }]),
            },
        },
    ] {
        let original = core::Record::Core(core::CoreRecord::Settings(original));
        let bytes = codec::encode(&original, &limits, &before).unwrap();
        for (names, expected_model, expected_effort) in
            [(&after, 0, 0), (&[catalogue_name(b"gone"), catalogue_name(b"two")], u8::MAX, 1)]
        {
            let decoded = codec::decode(&original.key(), &bytes, &limits, names).unwrap();
            let core::Record::Core(core::CoreRecord::Settings(record)) = decoded else { panic!("settings") };
            let value = match record {
                s::Record::Default { default, .. } => default.value,
                s::Record::Changed { changed, .. } => changed.value.unwrap(),
                s::Record::Proposal { value, .. } => match value.items[0].edit.clone() {
                    s::Edit::Set(value) => value,
                    s::Edit::Removed | s::Edit::Reset => panic!("set"),
                },
                s::Record::Revision { .. } | s::Record::Extent { .. } | s::Record::Charter { .. } => {
                    panic!("live fixture")
                }
            };
            let s::Value::Template(value) = value else { panic!("template") };
            assert_eq!(
                value.llm.unwrap().main,
                tasks::ModelEffort { model: expected_model, effort: Some(expected_effort) }
            );
        }
    }
    let mut missing = before.clone();
    missing[0].efforts = Box::new([Box::from(b"low".as_slice())]);
    let row = core::Record::Core(core::CoreRecord::Settings(s::Record::Changed {
        setting: s::Setting::Template(1),
        changed: s::Changed { value: Some(value), revision: 2 },
    }));
    let bytes = codec::encode(&row, &limits, &before).unwrap();
    let Some(core::Record::Core(core::CoreRecord::Settings(s::Record::Changed { changed, .. }))) =
        codec::decode(&row.key(), &bytes, &limits, &missing)
    else {
        panic!("changed")
    };
    let s::Value::Template(value) = changed.value.unwrap() else { panic!("template") };
    assert_eq!(value.llm.unwrap().main.model, u8::MAX);
}

/// Settings' policies retain job kinds (jig/docs/design/protocol/store.md, 5.1).
#[test]
fn job_delegation_kinds_round_trip_in_settings() {
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    let mut value = rules();
    value.ceiling.models = a::Models::ALL;
    value.ceiling.delegation.kinds = Box::new([a::Executor::Job(0), a::Executor::Job(u32::MAX)]);
    let row = s::Record::Default {
        setting: s::Setting::Rules,
        default: s::Default { revision: 1, value: s::Value::Rules(Box::new(value)) },
    };
    let bytes = codec::settings::encode(&row, &limits.settings, &[]).unwrap();
    assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits.settings, &[]), Ok(row));
}

#[test]
fn a_template_with_resources_round_trips_through_the_settings_store() {
    const BOUND: u16 = 32;
    let limits = codec::limits(&fixtures::CORE_LIMITS).expect("store limits");
    let names = Box::new([catalogue_name(b"one"), catalogue_name(b"two")]);
    assert_eq!(limits.settings.template_resources, u32::from(BOUND));
    for count in [0, 2, BOUND, BOUND + 1] {
        let mut template = template();
        let mut symbols = skein_lib::List::with_capacity(u32::from(count));
        for symbol in 0..count {
            symbols.push(people::Symbol { connector: u16::MAX - symbol, symbol }).expect("symbol fixture count");
        }
        template.resources = symbols.into_boxed();
        let row = core::Record::Core(core::CoreRecord::Settings(s::Record::Default {
            setting: s::Setting::Template(template.number),
            default: s::Default { value: s::Value::Template(Box::new(template)), revision: 1 },
        }));
        if count > BOUND {
            assert!(codec::encode(&row, &limits, &*names).is_none(), "one past the resource bound");
            continue;
        }
        let bytes = codec::encode(&row, &limits, &*names).expect("bounded resources");
        assert_eq!(codec::decode(&row.key(), &bytes, &limits, &*names), Some(row.clone()));
        if count == BOUND {
            let mut narrower = limits;
            narrower.settings.template_resources = u32::from(BOUND - 1);
            assert!(codec::decode(&row.key(), &bytes, &narrower, &*names).is_none(), "decoded resource bound");
        }
    }
}

#[test]
fn a_charters_paced_wake_policy_round_trips() {
    let limits = codec::limits(&fixtures::CORE_LIMITS).expect("store limits");
    let mut charter = charter();
    charter.wake = tasks::WakePolicy {
        words: tasks::WakeRule::Never,
        notices: tasks::WakeRule::Batch { count: 3, age: Duration::from_secs(2) },
        news: tasks::WakeRule::Paced { span: Duration::from_millis(1_800_000) },
        results: tasks::ResultsWake::Each,
        ..tasks::WakePolicy::DEFAULT
    };
    let row =
        s::Record::Charter { charter: 1, revision: 2, value: s::CharterVersion { charter: Some(Box::new(charter)) } };
    let bytes = codec::settings::encode(&row, &limits.settings, &[]).expect("charter encoding");
    assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits.settings, &[]), Ok(row));
}

#[test]
fn a_members_absent_wake_round_trips_without_a_default() {
    let limits = codec::people::limits(&fixtures::CORE_LIMITS).expect("people store limits");
    let mut member = fixtures::people_member();
    member.wake = None;
    let ask = people::Ask::Create { project: 1, batch: Box::new([member]) };
    let value = codec::people::encode_ask(&ask, &limits).expect("bounded batch");
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured batch");
    let bytes = writer.finish();
    let value = crate::people_v14::Ask::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("stored batch");
    assert_eq!(codec::people::decode_ask(value, &limits), Ok(ask));
}
