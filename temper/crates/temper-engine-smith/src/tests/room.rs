//! The domain's table and smith's independent run measure
//! (temper/docs/design/domain/agent/tools.md).

use super::{assignment, engine, run, tasks};
use crate::{charter, room::host_part};

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
        number: 1, rest: false, policy: policy.clone(),
        hosts: jig_domain_fleet::Kinds::Workers, spend: 1,
        wake: tasks::WakePolicy::DEFAULT, documents: Box::new([]), parts: Box::new([]),
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
                (tasks::Tools(512), false),
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
    assert!(table.settings > 0);
    assert_eq!(with - without, table.settings);
    assert_eq!(with_smith - without_smith, table.settings);
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
                assert_eq!(table.non_brief(&configured(&assignment.run.policy)), non_brief(&charter(&assignment), &limits));
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
        assignment.run.policy.instructions = [instructions, b"\n\n", first, b"\n\n", last].concat().into();
        assert_eq!(table.non_brief(&source), non_brief(&charter(&assignment), &limits));
    }
}

#[test]
fn a_small_receiving_bound_does_not_change_the_measured_costs() {
    let limits = measured_limits();
    let table = host_part(&limits);
    let small = host_part(&run::Limits { host_part: 1, ..limits });
    assert_eq!(small, temper_engine_domain::room::HostPart { bound: 1, ..table });
    assert_eq!(small.room(&configured(&assignment().run.policy)), 0);
}
