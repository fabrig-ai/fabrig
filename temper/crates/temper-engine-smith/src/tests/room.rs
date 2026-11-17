//! The domain's table and smith's independent run measure
//! (temper/docs/design/domain/agent/room.md, 4.4).

use super::{assignment, engine, run, tasks};
use crate::{charter, room::host_part as measure_host_part};

fn endpoints() -> smith_protocol_translate::Endpoints {
    let mut entries = skein_lib::List::with_capacity(2);
    entries
        .push(smith_protocol_translate::Endpoint { name: b"short".as_slice().into(), number: 1, dialect: 1 })
        .unwrap();
    entries
        .push(smith_protocol_translate::Endpoint { name: b"longest-endpoint".as_slice().into(), number: 2, dialect: 3 })
        .unwrap();
    smith_protocol_translate::Endpoints::new(entries)
}
fn host_part(limits: &run::Limits) -> temper_engine_domain::room::HostPart {
    measure_host_part(Some(limits), 65_536, &endpoints())
}

fn non_brief(charter: &run::Charter, limits: &run::Limits) -> u64 {
    limits.host_part.checked_sub(u64::from(run::brief_room(charter, None, limits).unwrap())).unwrap()
}

fn measured_limits() -> run::Limits {
    // Nonzero element costs exercise the receiving profile without a protocol dependency.
    run::Limits {
        schema_costs: run::SchemaCosts {
            finish: 101,
            deliver: 103,
            finish_text: 107,
            deliver_text: 109,
            change: 113,
            report: 127,
            failure: 131,
            form_join: 1,
            verdict_first: 137,
            verdict_next: 139,
            kind_first: 149,
            kind_next: 151,
            field_first: 157,
            text_field_first: 163,
            field_next: 167,
            escaped: [1; 256],
        },
        ..smith_agent_world::LIMITS.run
    }
}

fn configured(policy: &engine::RunPolicy) -> engine::Charter {
    engine::Charter {
        number: 1,
        rest: false,
        policy: policy.clone(),
        hosts: jig_domain_fleet::Kinds::Workers,
        spend: 1,
        wake: tasks::WakePolicy::DEFAULT,
        documents: Box::new([]),
        parts: Box::new([]),
    }
}

fn largest() -> engine::Assignment {
    let mut assignment = assignment();
    assignment.run.authority.tools = engine::RunPolicy::EVERY_FAMILY;
    assignment.run.contract = tasks::Contract::Change { connector: 1, kind: 1, words: u32::MAX };
    assignment
}

#[test]
fn the_table_is_smiths_measure() {
    for limits in [smith_agent_world::LIMITS.run, measured_limits()] {
        let table = host_part(&limits);
        for families in 0_u8..16 {
            for (tools, exact) in [
                (engine::RunPolicy::EVERY_FAMILY, true),
                (tasks::Tools(512), true),
                (tasks::Tools(3_072), true),
                (tasks::Tools(2_048), true),
            ] {
                let texts: [Box<[u8]>; 2] = [Box::new([]), Box::new([b'x'; 3_000])];
                for instructions in texts {
                    let mut assignment = largest();
                    let policy = &mut assignment.run.policy;
                    policy.inspect = families & 1 != 0;
                    policy.modify = families & 2 != 0;
                    policy.shell = families & 4 != 0;
                    policy.agents = families & 8 != 0;
                    policy.tools = tools;
                    policy.instructions = instructions;
                    let expected = table.non_brief(&configured(policy));
                    let change = charter(&assignment);
                    assert!(change.grants.deliver.is_some());
                    let measured = non_brief(&change, &limits);
                    assert!(expected >= measured, "families {families}, tools {tools:?}");
                    if exact {
                        assert_eq!(expected, measured, "families {families}, tools {tools:?}");
                    }
                    assignment.run.contract =
                        tasks::Contract::Report { words: jig_protocol_charter::CONTRACT_TEXT_MAX };
                    assert!(expected >= non_brief(&charter(&assignment), &limits));
                    assignment.run.ended_by = tasks::EndedBy::Requester;
                    assert!(expected >= non_brief(&charter(&assignment), &limits));
                }
            }
        }
    }
}

#[test]
fn a_charter_naming_settings_pays_for_its_tools() {
    let limits = measured_limits();
    let table = host_part(&limits);
    let mut assignment = largest();
    assignment.run.policy.tools = tasks::Tools(513);
    let without = table.non_brief(&configured(&assignment.run.policy));
    let without_smith = non_brief(&charter(&assignment), &limits);
    assignment.run.policy.tools = tasks::Tools(1_537);
    let with = table.non_brief(&configured(&assignment.run.policy));
    let with_smith = non_brief(&charter(&assignment), &limits);
    assert!(table.families[10].rendered > 0);
    assert_eq!(with - without, table.families[10].rendered);
    assert_eq!(with_smith - without_smith, table.families[10].rendered);
    assert_eq!(with, with_smith);
    assignment.run.authority.tools = tasks::Tools(512);
    assert!(with >= non_brief(&charter(&assignment), &limits));
}

#[test]
fn policy_text_is_counted_as_smith_renders_it() {
    let limits = measured_limits();
    let table = host_part(&limits);
    for instructions in [b"".as_slice(), b"x", b"x\n", b"x\n\n", "λ\n".as_bytes()] {
        for agents in [false, true] {
            for alternatives in 0..=3 {
                let mut assignment = largest();
                assignment.run.policy.instructions = instructions.into();
                assignment.run.policy.agents = agents;
                let model = assignment.run.policy.model.clone();
                let mut models = skein_lib::List::with_capacity(alternatives);
                let mut sources = skein_lib::List::with_capacity(alternatives + 1);
                sources.push(Some(1)).unwrap();
                for name in [b"one".as_slice(), "λ-model".as_bytes(), b"third model"]
                    .into_iter()
                    .take(usize::try_from(alternatives).unwrap())
                {
                    let mut alternative = model.clone();
                    alternative.name = name.into();
                    models.push(alternative).unwrap();
                    sources.push(Some(1)).unwrap();
                }
                assignment.run.policy.alternatives = models.into_boxed();
                assignment.run.sources = sources.into_boxed();
                assert_eq!(
                    table.non_brief(&configured(&assignment.run.policy)),
                    non_brief(&charter(&assignment), &limits)
                );
            }
        }
    }
}

#[test]
fn all_parts_are_counted_as_smith_renders_them() {
    let limits = measured_limits();
    let table = host_part(&limits);
    for (instructions, first, last) in [
        (b"".as_slice(), b"".as_slice(), b"".as_slice()),
        (b"base", b"one", b"two"),
        (b"base\n", b"one\n", b"two\n"),
        (b"base", "λ".as_bytes(), b""),
    ] {
        let mut assignment = largest();
        assignment.run.policy.instructions = instructions.into();
        let mut source = configured(&assignment.run.policy);
        // Distinct reasons still count together: admission covers the maximal selection.
        source.parts = Box::new([
            jig_domain_core::CharterPart { on: tasks::Reasons::FIRST, instructions: first.into() },
            jig_domain_core::CharterPart { on: tasks::Reasons::WORDS, instructions: last.into() },
        ]);
        let mut joined = skein_lib::Writer::new(instructions.len() + first.len() + last.len() + 4);
        for text in [instructions, b"\n\n", first, b"\n\n", last] {
            joined.put(text).unwrap();
        }
        assignment.run.policy.instructions = joined.finish();
        assert_eq!(table.non_brief(&source), non_brief(&charter(&assignment), &limits));
    }
}

#[test]
fn a_small_receiving_bound_does_not_change_the_measured_costs() {
    let limits = measured_limits();
    let table = host_part(&limits);
    let small = host_part(&run::Limits { host_part: 1, ..limits });
    assert_eq!(small, temper_engine_domain::room::HostPart { bound: Some(1), ..table });
    assert_eq!(small.room(&configured(&assignment().run.policy)), 0);
}

fn stock_charters() -> Box<[engine::Charter]> {
    use jig_protocol_config as config;
    let text = include_str!("../../../temper-init/template/temper.json")
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$provider\"", "null");
    let mut document: serde_json::Value = serde_json::from_str(&text).unwrap();
    // jig's unfixed parser takes the members that temper fixes outside its documents.
    let fixed: serde_json::Value = serde_json::from_str(
        r#"{
        "clients":{"heartbeat_ms":10000,"link":true},
        "deployment":{"directory":"state","seed":7},
        "providers":[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"}],
        "connectors":[1],"projects":[]
    }"#,
    )
    .unwrap();
    for (name, value) in fixed.as_object().unwrap() {
        document["engine"][name] = value.clone();
    }
    let engine_bytes = serde_json::to_vec(&document["engine"]).unwrap();
    let document =
        config::document(&engine_bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 })
            .unwrap();
    let engine =
        config::engine(&document, 0, &config::Directories::working(), config::Defaults::Passed, &config::Fixed::NONE)
            .unwrap();
    temper_engine_service_stock::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes).charters
}

fn encoded(charter: &run::Charter, endpoints: &smith_protocol_translate::Endpoints) -> u64 {
    u64::try_from(smith_protocol_translate::encode_charter(charter, &smith_charter::CEILINGS, endpoints).unwrap().len())
        .unwrap()
}

#[test]
fn fixed_costs_are_smiths_owned_and_encoded_measure_on_every_stock_wake() {
    let endpoints = endpoints();
    let table = measure_host_part(Some(&measured_limits()), 65_536, &endpoints);
    for mut stock in stock_charters() {
        for model in core::iter::once(&mut stock.policy.model).chain(&mut stock.policy.alternatives) {
            model.endpoint = 2;
            model.dialect = 3;
        }
        for name in tasks::REASON_NAMES {
            let wake = tasks::Reasons::named(name).unwrap();
            let mut assignment = largest();
            // The frame includes the project's optional delivery-check bound at its largest.
            assignment.check_timeout = Some(skein_lib::Duration::from_secs(86_400));
            assignment.charter = stock.number;
            assignment.run.policy = stock.policy.clone();
            let mut length = stock.policy.instructions.len();
            for part in &stock.parts {
                if part.on.meets(wake) {
                    length = length.checked_add(2).unwrap().checked_add(part.instructions.len()).unwrap();
                }
            }
            let mut instructions = skein_lib::Writer::new(length);
            instructions.put(&stock.policy.instructions).unwrap();
            for part in &stock.parts {
                if part.on.meets(wake) {
                    instructions.put(b"\n\n").unwrap();
                    instructions.put(&part.instructions).unwrap();
                }
            }
            assignment.run.policy.instructions = instructions.finish();
            assignment.run.sources = vec![Some(1); 1 + stock.policy.alternatives.len()].into_boxed_slice();
            let mut measured = charter(&assignment);
            measured.brief.sections = Box::new([]);
            let cost = table.on_wake(&stock, wake);
            assert_eq!(cost.owned, measured.owned_bytes().unwrap(), "charter {} {name:?}", stock.number);
            assert!(cost.encoded >= encoded(&measured, &endpoints), "charter {} {name:?}", stock.number);
            // Match the table's largest optional framing, retaining the stock's payload bytes.
            for model in core::iter::once(&mut measured.llm).chain(&mut measured.models) {
                model.account = Some(1);
                model.prices.cache_write = Some(1);
                if model.effort.is_none() {
                    model.effort = Some(Box::new([]));
                }
            }
            assert_eq!(cost.encoded, encoded(&measured, &endpoints), "charter {} {name:?}", stock.number);
        }
    }
}

#[test]
fn no_local_agent_keeps_the_owned_and_encoded_costs_and_skips_rendering() {
    let with = measure_host_part(Some(&measured_limits()), 65_536, &endpoints());
    let without = measure_host_part(None, 65_536, &endpoints());
    assert_eq!(without.bound, None);
    for (with, without) in [
        (with.fixed, without.fixed),
        (with.model, without.model),
        (with.always, without.always),
        (with.asking, without.asking),
        (with.asking_settings, without.asking_settings),
    ]
    .into_iter()
    .chain(with.families.into_iter().zip(without.families))
    {
        assert_eq!((with.owned, with.encoded), (without.owned, without.encoded));
        assert_eq!(without.rendered, 0);
    }
}
