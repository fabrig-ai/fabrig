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
    }
}
fn charter() -> core::Charter {
    core::Charter {
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
            later: Some(tasks::LlmLater {
                main: tasks::ModelEffort { model: 1, effort: Some(0) },
                subagents: Box::new([tasks::ModelEffort { model: 0, effort: None }]),
            }),
            account: Some(3),
            main: tasks::ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::new([tasks::ModelEffort { model: 1, effort: None }]),
        }),
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::too_many_lines,
    reason = "one round trip covers current, removed, revision and proposed whole values"
)]
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
    let mut rows = skein_lib::List::with_capacity(34);
    for row in [
        s::Record::Extent { oldest: 1, keys: 1 },
        s::Record::Revision {
            history: false,
            revision: s::Revision {
                applications: Box::default(),
                head: s::Head { number: 1, against: 0, ..head.clone() },
                items: Box::new([]),
            },
        },
        s::Record::Charter {
            charter: 1,
            revision: 2,
            value: s::CharterVersion { overlay: None, charter: Some(Box::new(charter())) },
        },
        s::Record::Charter { charter: 1, revision: 3, value: s::CharterVersion { overlay: None, charter: None } },
    ] {
        rows.push(row).expect("settings fixture count");
    }
    for (setting, value) in [
        (s::Setting::Charter(1), s::Value::Charter(Box::new(charter()))),
        (s::Setting::Template(1), s::Value::Template(Box::new(template()))),
        (s::Setting::Policy(1), s::Value::Policy(Box::new(policy()))),
        (s::Setting::Rules, s::Value::Rules(Box::new(rules()))),
        (
            s::Setting::Model(9),
            s::Value::Model(Box::new(s::CatalogueModel {
                first_read: Some(Wall::from_nanos(u64::MAX)),
                number: 9,
                model: model(),
                shown: Some(b"Shown".as_slice().into()),
                description: Some(b"Description".as_slice().into()),
                picker: true,
                withdrawn: true,
                source: s::ModelSource::List,
                estimated: true,
            })),
        ),
    ] {
        let head = if setting == s::Setting::Model(9) {
            s::Head { by: s::Decider::List { provider: u32::MAX }, ..head.clone() }
        } else {
            head.clone()
        };
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
            revision: s::Revision {
                applications: Box::default(),
                head: head.clone(),
                items: items
                    .iter()
                    .map(|item| s::RevisionItem {
                        setting: item.setting,
                        edit: item.edit.clone(),
                        replaced: s::Replaced::NotOffered,
                    })
                    .collect(),
            },
        })
        .expect("settings fixture count");
        rows.push(s::Record::Revision {
            history: true,
            revision: s::Revision {
                applications: Box::default(),
                head: head.clone(),
                items: items
                    .iter()
                    .map(|item| s::RevisionItem {
                        setting: item.setting,
                        edit: item.edit.clone(),
                        replaced: s::Replaced::NotOffered,
                    })
                    .collect(),
            },
        })
        .expect("settings fixture count");
        rows.push(s::Record::Proposal {
            proposer: people::Proposer::Party(6),
            proposal: 7,
            value: s::Proposal { key: None, applications: Box::default(), against: 2, items },
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
            s::Value::Overlay(_)
            | s::Value::Policy(_)
            | s::Value::Charter(_)
            | s::Value::Model(_)
            | s::Value::Application(_) => {
                panic!("fixture setting")
            }
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
                key: None,
                applications: Box::default(),
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
                s::Record::Application { .. }
                | s::Record::Revision { .. }
                | s::Record::Extent { .. }
                | s::Record::Charter { .. } => {
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

/// Settings' policies retain job kinds (jig/docs/design/protocol/store/records.md, 1).
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
fn a_charters_tools_survive_the_store() {
    let limits = codec::limits(&fixtures::CORE_LIMITS).expect("store limits");
    for tools in [tasks::Tools(1_536), core::RunPolicy::EVERY_FAMILY] {
        let mut charter = charter();
        charter.policy.tools = tools;
        let row = s::Record::Charter {
            charter: 1,
            revision: 2,
            value: s::CharterVersion { overlay: None, charter: Some(Box::new(charter)) },
        };
        let bytes = codec::settings::encode(&row, &limits.settings, &[]).expect("charter encoding");
        assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits.settings, &[]), Ok(row));
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
    let row = s::Record::Charter {
        charter: 1,
        revision: 2,
        value: s::CharterVersion { overlay: None, charter: Some(Box::new(charter)) },
    };
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
    let value = crate::people_v16::Ask::decode(&limits, &mut skein_lib::Reader::new(&bytes)).expect("stored batch");
    assert_eq!(codec::people::decode_ask(value, &limits), Ok(ask));
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

#[expect(
    clippy::disallowed_methods,
    reason = "ordinary codec fixtures translate revision literals while retaining proposal items"
)]
fn document_records(charter: core::Charter) -> [s::Record; 6] {
    let setting = s::Setting::Charter(charter.number);
    let value = s::Value::Charter(Box::new(charter.clone()));
    let items = Box::new([s::Item { setting, edit: s::Edit::Set(value.clone()) }]);
    let revision = s::Revision {
        applications: Box::default(),
        head: s::Head {
            number: 2,
            against: 1,
            at: Wall::from_nanos(10),
            by: s::Decider::Party(3),
            proposal: None,
            moved: false,
        },
        items: items
            .iter()
            .map(|item| s::RevisionItem {
                setting: item.setting,
                edit: item.edit.clone(),
                replaced: s::Replaced::NotOffered,
            })
            .collect(),
    };
    [
        s::Record::Default { setting, default: s::Default { value: value.clone(), revision: 2 } },
        s::Record::Changed { setting, changed: s::Changed { value: Some(value), revision: 2 } },
        s::Record::Charter {
            charter: charter.number,
            revision: 2,
            value: s::CharterVersion { overlay: None, charter: Some(Box::new(charter)) },
        },
        s::Record::Revision { history: false, revision: revision.clone() },
        s::Record::Revision { history: true, revision },
        s::Record::Proposal {
            proposer: people::Proposer::Party(3),
            proposal: 4,
            value: s::Proposal { key: None, applications: Box::default(), against: 1, items },
        },
    ]
}

/// Documents stay with every stored charter value (jig/docs/design/domain/engine/settings.md, 9).
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
            overlay: None,
            charter: Some(Box::new(core::Charter { documents: Box::new([]), parts: Box::new([]), ..charter.clone() })),
        },
    };
    let mut bytes = codec::settings::encode(&empty, &limits, &[]).unwrap().into_vec();
    bytes.truncate(bytes.len().checked_sub(8).unwrap());
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
    bytes.extend_from_slice(&0_u32.to_be_bytes());
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
            crate::settings_v3::Limits { document_path: core::DOCUMENT_PATH_BYTES - 1, ..limits },
            crate::settings_v3::Limits { document_heading_some: core::DOCUMENT_HEADING_BYTES - 1, ..limits },
        ] {
            assert_eq!(codec::settings::encode(&row, &lowered, &[]).unwrap_err().reason, skein_codec::Reason::Bound);
            assert_eq!(
                codec::settings::decode(&row.key(), &bytes, &lowered, &[]).unwrap_err().reason,
                skein_codec::Reason::Bound
            );
        }
    }
}

mod applications;
mod replaced;

#[test]
fn proposal_keys_and_the_extent_counter_round_trip_without_narrowing() {
    let limits = codec::settings::limits(&fixtures::CORE_LIMITS).unwrap();
    for key in [None, Some(1), Some(u64::MAX)] {
        let row = s::Record::Proposal {
            proposer: people::Proposer::Task(8),
            proposal: 7,
            value: s::Proposal { key, against: 1, items: Box::default(), applications: Box::default() },
        };
        let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
        assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits, &[]), Ok(row));
    }
    for keys in [1, u64::MAX] {
        let row = s::Record::Extent { oldest: 2, keys };
        let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
        assert_eq!(codec::settings::decode(&row.key(), &bytes, &limits, &[]), Ok(row));
    }
}

#[path = "models.rs"]
mod models;

/// A part selects one or more known run reasons (jig/docs/design/protocol/store/records.md).
#[test]
fn charter_parts_refuse_empty_and_unknown_reason_bits() {
    // Bit 14 is the first bit outside the fourteen stored reasons.
    const UNKNOWN: tasks::Reasons = tasks::Reasons(1 << 14);
    let limits = codec::settings::limits(&fixtures::CORE_LIMITS).unwrap();
    for on in [UNKNOWN, tasks::Reasons::EMPTY] {
        let mut charter = charter();
        charter.parts = Box::new([core::CharterPart { on, instructions: Box::from(b"part".as_slice()) }]);
        let row = s::Record::Charter {
            charter: charter.number,
            revision: 2,
            value: s::CharterVersion { overlay: None, charter: Some(Box::new(charter)) },
        };
        let bytes = codec::settings::encode(&row, &limits, &[]).unwrap();
        assert_eq!(
            codec::settings::decode(&row.key(), &bytes, &limits, &[]).unwrap_err().reason,
            skein_codec::Reason::Bound,
            "{on:?}"
        );
    }
}

#[test]
fn a_templates_closer_round_trips_in_the_settings_family() {
    for ended_by in [tasks::EndedBy::Closer(tasks::Closer { connector: 1, code: 6 }), tasks::EndedBy::Standing] {
        let mut template = template();
        template.flags.conversation = false;
        template.tracked = None;
        template.flags.ended_by = ended_by;
        let row = core::Record::Core(core::CoreRecord::Settings(s::Record::Changed {
            setting: s::Setting::Template(1),
            changed: s::Changed { revision: 1, value: Some(s::Value::Template(Box::new(template))) },
        }));
        let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
        let names = [catalogue_name(b"one"), catalogue_name(b"two")];
        let bytes = codec::encode(&row, &limits, &names).unwrap();
        assert_eq!(codec::decode(&row.key(), &bytes, &limits, &names), Some(row));
    }
}

fn overlay() -> s::Overlay {
    let charter = charter();
    s::Overlay {
        rest: Some(charter.rest),
        model: Some(charter.policy.model),
        alternatives: Some(charter.policy.alternatives),
        inspect: Some(charter.policy.inspect),
        modify: Some(charter.policy.modify),
        shell: Some(charter.policy.shell),
        agents: Some(charter.policy.agents),
        tools: Some(charter.policy.tools),
        hosts: Some(charter.hosts),
        waiting: Some(charter.policy.waiting),
        resume: Some(charter.policy.resume),
        turns: Some(charter.policy.turns),
        time: Some(charter.policy.time),
        reserve_turns: Some(charter.policy.reserve_turns),
        reserve_time: Some(charter.policy.reserve_time),
        call_timeout: Some(charter.policy.call_timeout),
        spend: Some(charter.spend),
        documents: Some(charter.documents),
        instructions: charter.policy.instructions,
        parts: charter.parts,
    }
}
#[test]
fn overlays_round_trip_in_changes_history_proposals_and_held_versions() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.run_bytes = 4096;
    let limits = codec::limits(&domain).unwrap();
    let overlay = overlay();
    let value = s::Value::Overlay(Box::new(overlay.clone()));
    let setting = s::Setting::Charter(1);
    let head =
        s::Head { number: 2, against: 1, at: Wall::EPOCH, by: s::Decider::Party(1), proposal: None, moved: false };
    for row in [
        s::Record::Changed { setting, changed: s::Changed { value: Some(value.clone()), revision: 2 } },
        s::Record::Revision {
            history: true,
            revision: s::Revision {
                head,
                items: Box::new([s::RevisionItem {
                    setting,
                    edit: s::Edit::Set(value.clone()),
                    replaced: s::Replaced::Value(value.clone()),
                }]),
                applications: Box::new([]),
            },
        },
        s::Record::Proposal {
            proposer: people::Proposer::Party(1),
            proposal: 1,
            value: s::Proposal {
                key: None,
                against: 1,
                items: Box::new([s::Item { setting, edit: s::Edit::Set(value) }]),
                applications: Box::new([]),
            },
        },
        s::Record::Charter {
            charter: 1,
            revision: 1,
            value: s::CharterVersion { overlay: Some(Box::new(overlay)), charter: Some(Box::new(charter())) },
        },
    ] {
        let row = core::Record::Core(core::CoreRecord::Settings(row));
        let bytes = codec::encode(&row, &limits, &[]).unwrap();
        assert_eq!(codec::decode(&row.key(), &bytes, &limits, &[]), Some(row));
    }
}
#[test]
fn overlay_codec_bounds_instructions_parts_documents_and_alternatives() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.run_bytes = 4096;
    domain.tasks.charter_parts = 2;
    domain.tasks.charter_documents = 1;
    domain.settings.alternatives = 1;
    let limits = codec::limits(&domain).unwrap();
    let mut generous = domain;
    generous.run_bytes = 8192;
    generous.tasks.charter_parts = 3;
    generous.tasks.charter_documents = 2;
    generous.settings.alternatives = 2;
    let generous = codec::limits(&generous).unwrap();
    for past in [false, true] {
        for field in 0_u8..5 {
            let mut overlay = overlay();
            match field {
                0 => overlay.instructions = skein_lib::bytes::zeroed(4096 + usize::from(past)),
                1 => {
                    let part = overlay.parts[0].clone();
                    overlay.parts = if past {
                        Box::new([part.clone(), part.clone(), part])
                    } else {
                        Box::new([part.clone(), part])
                    };
                }
                2 => overlay.alternatives = Some(if past { Box::new([model(), model()]) } else { Box::new([model()]) }),
                3 => {
                    let document = core::Document {
                        connector: 1,
                        name: a::Name { segments: Box::new([Box::from(b"r".as_slice())]) },
                        path: b"p".as_slice().into(),
                        heading: None,
                    };
                    overlay.documents =
                        Some(if past { Box::new([document.clone(), document]) } else { Box::new([document]) });
                }
                _ => overlay.parts[0].instructions = skein_lib::bytes::zeroed(4096 + usize::from(past)),
            }
            let row = core::Record::Core(core::CoreRecord::Settings(s::Record::Changed {
                setting: s::Setting::Charter(1),
                changed: s::Changed { revision: 2, value: Some(s::Value::Overlay(Box::new(overlay))) },
            }));
            assert_eq!(codec::encode(&row, &limits, &[]).is_some(), !past, "encode field {field}");
            let bytes = codec::encode(&row, &generous, &[]).unwrap();
            assert_eq!(codec::decode(&row.key(), &bytes, &limits, &[]).is_some(), !past, "decode field {field}");
        }
    }
}

#[test]
fn charter_versions_without_overlays_keep_the_versioned_stored_bytes() {
    let limits = codec::limits(&fixtures::CORE_LIMITS).unwrap();
    let row = core::Record::Core(core::CoreRecord::Settings(s::Record::Charter {
        charter: 1,
        revision: 2,
        value: s::CharterVersion { overlay: None, charter: None },
    }));
    let bytes = [0_u8, 3, 0];
    assert_eq!(codec::decode(&row.key(), &bytes, &limits, &[]), Some(row.clone()));
    assert_eq!(codec::encode(&row, &limits, &[]).unwrap().as_ref(), bytes);
}
