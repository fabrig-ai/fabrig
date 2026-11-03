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

fn document_domain() -> core::Limits {
    let mut domain = fixtures::CORE_LIMITS;
    domain.authority.segments = 8;
    domain.authority.segment_bytes = 4096;
    domain.tasks.hold_bytes = 4096;
    domain
}

fn charter_with_documents() -> core::Charter {
    let domain = document_domain();
    let mut segments = skein_lib::List::with_capacity(domain.authority.segments);
    for _ in 0..domain.authority.segments {
        segments.push(Box::<[u8]>::from([b'n'; 4096])).unwrap();
    }
    let document = core::Document {
        connector: 7,
        name: a::Name { segments: segments.into_boxed() },
        path: Box::new([b'p'; 256]),
        heading: Some(Box::new([b'h'; 80])),
    };
    let mut without_heading = document.clone();
    without_heading.connector = 9;
    without_heading.heading = None;
    core::Charter { documents: Box::new([document, without_heading]), ..charter() }
}

fn document_records(charter: core::Charter) -> [s::Record; 6] {
    let setting = s::Setting::Charter(charter.number);
    let value = s::Value::Charter(Box::new(charter.clone()));
    let items = Box::new([s::Item { setting, edit: s::Edit::Set(value.clone()) }]);
    let revision = s::Revision {
        head: s::Head {
            number: 2,
            against: 1,
            at: Wall::from_nanos(10),
            by: s::Decider::Party(3),
            proposal: None,
            moved: false,
        },
        items: items.clone(),
    };
    [
        s::Record::Default { setting, default: s::Default { value: value.clone(), revision: 2 } },
        s::Record::Changed { setting, changed: s::Changed { value: Some(value), revision: 2 } },
        s::Record::Charter {
            charter: charter.number,
            revision: 2,
            value: s::CharterVersion { charter: Some(Box::new(charter)) },
        },
        s::Record::Revision { history: false, revision: revision.clone() },
        s::Record::Revision { history: true, revision },
        s::Record::Proposal {
            proposer: people::Proposer::Party(3),
            proposal: 4,
            value: s::Proposal { against: 1, items },
        },
    ]
}

/// Documents stay with every stored charter value (jig/docs/design/domain/engine.md, 17.9).
#[test]
fn charter_documents_at_their_bounds_round_trip_in_every_settings_record() {
    let limits = codec::settings::limits(&document_domain()).unwrap();
    for row in document_records(charter_with_documents()) {
        let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
        assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits, &[]), Ok(row));
    }
}

// The generated encoder refuses these rows; write their document fields directly
// to check that stored bytes past the bounds are refused at decode too.
#[expect(clippy::disallowed_methods, reason = "ordinary tests construct stored bytes that the bounded encoder refuses")]
fn assert_documents_refused(charter: core::Charter) {
    let limits = codec::settings::limits(&document_domain()).unwrap();
    for row in document_records(charter.clone()) {
        assert_eq!(codec::settings::encode(&row, &limits, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
    }
    let empty = s::Record::Charter {
        charter: charter.number,
        revision: 2,
        value: s::CharterVersion {
            charter: Some(Box::new(core::Charter { documents: Box::new([]), ..charter.clone() })),
        },
    };
    let mut bytes = codec::settings::encode(&empty, &limits, &[]).unwrap().into_vec();
    bytes.truncate(bytes.len().checked_sub(4).unwrap());
    bytes.extend_from_slice(&u32::try_from(charter.documents.len()).unwrap().to_be_bytes());
    for document in &charter.documents {
        bytes.extend_from_slice(&document.connector.to_be_bytes());
        bytes.extend_from_slice(&u32::try_from(document.name.segments.len()).unwrap().to_be_bytes());
        for segment in &document.name.segments {
            bytes.extend_from_slice(&u32::try_from(segment.len()).unwrap().to_be_bytes());
            bytes.extend_from_slice(segment);
        }
        bytes.extend_from_slice(&u32::try_from(document.path.len()).unwrap().to_be_bytes());
        bytes.extend_from_slice(&document.path);
        match &document.heading {
            None => bytes.push(0),
            Some(heading) => {
                bytes.push(1);
                bytes.extend_from_slice(&u32::try_from(heading.len()).unwrap().to_be_bytes());
                bytes.extend_from_slice(heading);
            }
        }
    }
    assert_eq!(
        codec::settings::decode(&empty.key(), &bytes, &limits, &[]).unwrap_err().reason,
        skein_codec::Reason::Bound
    );
}

#[test]
fn charter_documents_one_past_the_count_are_refused() {
    let mut charter = charter_with_documents();
    charter.documents =
        Box::new([charter.documents[0].clone(), charter.documents[0].clone(), charter.documents[0].clone()]);
    assert_documents_refused(charter);
}

#[test]
fn charter_document_names_one_past_the_segments_are_refused() {
    let mut charter = charter_with_documents();
    let mut segments = skein_lib::List::with_capacity(9);
    for _ in 0_u32..9 {
        segments.push(Box::from(b"n".as_slice())).unwrap();
    }
    charter.documents[0].name.segments = segments.into_boxed();
    assert_documents_refused(charter);
}

#[test]
fn charter_document_segments_one_past_the_bytes_are_refused() {
    let mut charter = charter_with_documents();
    charter.documents[0].name.segments[0] = Box::new([b'n'; 4097]);
    assert_documents_refused(charter);
}

#[test]
fn charter_document_paths_one_past_the_bytes_are_refused() {
    let mut charter = charter_with_documents();
    charter.documents[0].path = Box::new([b'p'; 257]);
    assert_documents_refused(charter);
}

#[test]
fn charter_document_headings_one_past_the_bytes_are_refused() {
    let mut charter = charter_with_documents();
    charter.documents[0].heading = Some(Box::new([b'h'; 81]));
    assert_documents_refused(charter);
}

#[test]
fn charter_documents_obey_lowered_limits_in_every_settings_record() {
    let domain = document_domain();
    let limits = codec::settings::limits(&domain).unwrap();
    let mut fewer_documents = domain;
    fewer_documents.tasks.charter_documents -= 1;
    let mut fewer_segments = domain;
    fewer_segments.authority.segments -= 1;
    let mut shorter_segments = domain;
    shorter_segments.authority.segment_bytes -= 1;
    let shorter_segments = codec::settings::limits(&shorter_segments).unwrap();
    assert_eq!(shorter_segments.segment_bytes, limits.segment_bytes, "hold paths retain their byte bound");
    for row in document_records(charter_with_documents()) {
        let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
        for lowered in [
            codec::settings::limits(&fewer_documents).unwrap(),
            codec::settings::limits(&fewer_segments).unwrap(),
            shorter_segments,
            crate::settings_v1::Limits { document_path: core::DOCUMENT_PATH_BYTES - 1, ..limits },
            crate::settings_v1::Limits { document_heading_some: core::DOCUMENT_HEADING_BYTES - 1, ..limits },
        ] {
            assert_eq!(codec::settings::encode(&row, &lowered, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
            assert_eq!(
                codec::settings::decode(&row.key(), &bytes, &lowered, &[]).unwrap_err().reason,
                skein_codec::Reason::Bound
            );
        }
    }
}
