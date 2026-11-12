use alloc::boxed::Box;
use jig_domain_link::{Form, Grant};
use skein_lib::{Duration, Reader};
use smith_charter::v4 as wire;

use crate::{
    Budget, Charter, Contract, EndpointName, Model, Prices, Reserve, Section, TextRule, Tool, ToolEffect, charter,
    encode,
};

fn source() -> Charter {
    let model = Model {
        prices: Prices { input: 2, cached: 1, cache_write: Some(4), output: 3, unit: 1000 },
        dialect: 3,
        endpoint: 7,
        name: Box::from(&b"small"[..]),
        window: 4096,
        output: 128,
        effort: Some(Box::from(&b"low"[..])),
    };
    Charter {
        mode: smith_domain_run::Mode::Finite,
        instructions: Box::from(&b"help"[..]),
        brief: Box::new([]),
        tools: Box::new([Tool {
            name: Box::from(&b"read"[..]),
            description: Box::from(&b"Read a value"[..]),
            schema: Box::from(&b"{}"[..]),
            effect: ToolEffect::Read,
            timeout: Duration::ZERO,
        }]),
        wait: true,
        agents: false,
        workspace: crate::WorkspaceTools { inspect: false, modify: false, shell: false },
        delivers: false,
        conventions: None,
        words: 512,
        contract: Contract {
            report: Some(TextRule { max: 512, fields: Box::new([]) }),
            failure: None,
            verdicts: Box::new([]),
            change: None,
        },
        budget: Budget {
            turns: 3,
            spend: 9000,
            time: Duration::from_secs(60),
            reserve: Reserve { turns: 1, time: Duration::from_secs(10) },
        },
        model: crate::Llm { model, grant: Some(5) },
        models: Box::new([]),
        waiting: Duration::ZERO,
        resumes: false,
    }
}

#[test]
fn core_charter_encodes_with_the_same_sections_tools_contract_and_model() {
    let mut source = source();
    let sections = Box::new([Section { title: Box::from(&b"Task"[..]), text: Box::from(&b"Do it"[..]) }]);
    source.brief = sections;
    let charter = charter(source).expect("bounded charter");
    let names = [EndpointName { number: 7, dialect: 3, name: Box::from(&b"primary"[..]) }];
    let bytes = encode(charter, &names, &wire::CEILINGS).expect("bounded encoding");
    let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("Smith codec accepts it");
    assert_eq!(decoded.instructions(), b"help");
    assert_eq!(decoded.brief().get(0).expect("one section").title(), b"Task");
    assert_eq!(decoded.tools().host().get(0).expect("one tool").name(), b"read");
    assert_eq!(decoded.main().endpoint(), b"primary");
    assert_eq!(decoded.budget().spend(), 9000);
    assert_eq!(decoded.budget().reserve().turns(), 1);
    assert_eq!(decoded.budget().reserve().time(), Duration::from_secs(10));
    assert_eq!(decoded.main().window(), 4096);
    assert_eq!(decoded.main().output(), 128);
    assert_eq!(*decoded.main().prices().cache_write(), Some(4));
    assert_eq!(decoded.main().effort().as_deref(), Some(b"low".as_slice()), "the choice's effort reaches smith");
    assert!(!decoded.resume(), "a resume requires a validated transcript");
}

fn call_name() -> Box<[u8]> {
    let name = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation: 7, completion: 3, position: 2 },
    )
    .expect("a call name within its ceilings");
    let mut writer = skein_lib::Writer::new(16);
    name.encode(&mut writer).expect("sixteen bytes");
    writer.finish()
}

#[test]
fn a_call_key_reads_completion_and_position() {
    assert_eq!(crate::call_key(&call_name(), 7), Some(crate::CallKey { activation: 7, completion: 3, position: 2 }));
}

#[test]
fn settled_call_keys_round_trip_at_the_integer_bounds() {
    for key in [
        crate::CallKey { activation: 0, completion: 0, position: 0 },
        crate::CallKey { activation: u64::MAX, completion: u32::MAX, position: u32::MAX },
        crate::CallKey { activation: u64::MAX, completion: 0, position: u32::MAX },
        crate::CallKey { activation: 0, completion: u32::MAX, position: 0 },
    ] {
        assert_eq!(crate::call_key(&crate::call_name(&key), key.activation), Some(key));
    }
}

#[test]
fn a_call_name_for_another_attempt_has_no_key() {
    assert_eq!(crate::call_key(&call_name(), 8), None);
}

#[test]
fn a_call_name_with_trailing_bytes_has_no_key() {
    let bytes = call_name();
    let mut writer = skein_lib::Writer::new(17);
    writer.put(&bytes).expect("room for the name");
    writer.put(&[0]).expect("room for a trailing byte");
    assert_eq!(crate::call_key(&writer.finish(), 7), None);
}

fn result_bytes(form: wire::Form, label: Option<Box<[u8]>>, fields: skein_lib::List<wire::Field>) -> Box<[u8]> {
    let value = wire::RunResult::new(
        &wire::CEILINGS,
        wire::RunResultParts {
            form,
            label,
            text: Box::from(b"finished".as_slice()),
            fields,
            items: skein_lib::List::with_capacity(0),
        },
    )
    .expect("bounded result");
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured encoding");
    writer.finish()
}

#[test]
fn a_report_result_reads_its_words() {
    let bytes = result_bytes(wire::Form::Report, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Ok(crate::RunResult::Report { words: Box::from(b"finished".as_slice()) }));
}

#[test]
fn a_verdict_result_reads_its_label_and_fields() {
    let mut fields = skein_lib::List::with_capacity(1);
    fields
        .push(
            wire::Field::new(
                &wire::CEILINGS,
                wire::FieldParts { name: Box::from(b"status".as_slice()), text: Box::from(b"green".as_slice()) },
            )
            .expect("bounded field"),
        )
        .expect("one field");
    let bytes = result_bytes(wire::Form::Verdict, Some(Box::from(b"ready".as_slice())), fields);
    assert_eq!(
        crate::result(&bytes),
        Ok(crate::RunResult::Verdict {
            label: Box::from(b"ready".as_slice()),
            words: Box::from(b"finished".as_slice()),
            fields: Box::new([crate::Field {
                name: Box::from(b"status".as_slice()),
                value: Box::from(b"green".as_slice())
            }]),
        })
    );
}

#[test]
fn a_change_result_does_not_decode() {
    let bytes = result_bytes(wire::Form::Change, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Err(crate::Unreadable::Form));
}

#[test]
fn a_truncated_result_does_not_decode() {
    let bytes = result_bytes(wire::Form::Report, None, skein_lib::List::with_capacity(0));
    let end = bytes.len().checked_sub(1).expect("a result has bytes");
    let short = bytes.get(..end).expect("remove the final byte");
    assert_eq!(crate::result(short), Err(crate::Unreadable::Malformed));
}

#[test]
fn a_failure_result_reads_its_reason() {
    let bytes = result_bytes(wire::Form::Failure, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Ok(crate::RunResult::Failure { reason: Box::from(b"finished".as_slice()) }));
}

#[test]
fn a_result_with_trailing_bytes_does_not_decode() {
    let bytes = result_bytes(wire::Form::Report, None, skein_lib::List::with_capacity(0));
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(1).expect("one extra byte"));
    writer.put(&bytes).expect("room for the result");
    writer.put(&[0]).expect("room for a trailing byte");
    assert_eq!(crate::result(&writer.finish()), Err(crate::Unreadable::Malformed));
}

#[test]
fn a_verdict_without_a_label_does_not_decode() {
    let bytes = result_bytes(wire::Form::Verdict, None, skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Err(crate::Unreadable::Form));
}

#[test]
fn a_report_with_a_label_does_not_decode() {
    let bytes =
        result_bytes(wire::Form::Report, Some(Box::from(b"ready".as_slice())), skein_lib::List::with_capacity(0));
    assert_eq!(crate::result(&bytes), Err(crate::Unreadable::Form));
}

#[test]
fn a_verdict_with_items_does_not_decode() {
    let mut items = skein_lib::List::with_capacity(1);
    items
        .push(
            wire::Item::new(
                &wire::CEILINGS,
                wire::ItemParts { kind: Box::from(b"follow".as_slice()), fields: skein_lib::List::with_capacity(0) },
            )
            .expect("bounded item"),
        )
        .expect("one item");
    let value = wire::RunResult::new(
        &wire::CEILINGS,
        wire::RunResultParts {
            form: wire::Form::Verdict,
            label: Some(Box::from(b"ready".as_slice())),
            text: Box::new([]),
            fields: skein_lib::List::with_capacity(0),
            items,
        },
    )
    .expect("bounded result");
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured encoding");
    assert_eq!(crate::result(&writer.finish()), Err(crate::Unreadable::Form));
}

#[test]
fn a_conversations_attempts_share_one_identity() {
    let deployment = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let expected = [0x06, 0x8b, 0xfc, 0xde, 0x3e, 0x7d, 0xde, 0x1c, 0x53, 0xe8, 0x5c, 0x10, 0xf4, 0x7d, 0xef, 0x5d];
    for _attempt in [3_u64, 4, 9] {
        assert_eq!(crate::run_identity(&deployment, 7, 3), expected);
    }
}

#[test]
fn two_conversations_of_a_task_have_two_identities() {
    let deployment = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    assert_eq!(
        crate::run_identity(&deployment, 7, 3),
        [0x06, 0x8b, 0xfc, 0xde, 0x3e, 0x7d, 0xde, 0x1c, 0x53, 0xe8, 0x5c, 0x10, 0xf4, 0x7d, 0xef, 0x5d]
    );
    assert_eq!(
        crate::run_identity(&deployment, 7, 4),
        [0x92, 0xf3, 0x7d, 0x0f, 0x30, 0xe6, 0xf6, 0x6f, 0x8f, 0xee, 0x8a, 0x01, 0x68, 0x18, 0x8f, 0x3a]
    );
}

#[test]
fn selected_source_grants_are_independent_of_endpoint_resolution() {
    let names = [EndpointName { number: 7, dialect: 3, name: b"primary".as_slice().into() }];
    for main in [Some(5), Some(19), None] {
        let mut source = source();
        source.model.grant = main;
        let mut other = source.model.clone();
        other.grant = Some(23);
        source.models = Box::new([source.model.clone(), other]);
        source.resumes = true;
        let translated = charter(source).expect("runtime model bindings");
        let bytes = encode(translated, &names, &wire::CEILINGS).expect("grants do not resolve endpoints");
        let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("charter");
        assert_eq!(*decoded.main().account(), main);
        assert_eq!(*decoded.models().get(0).unwrap().account(), main);
        assert_eq!(*decoded.models().get(1).unwrap().account(), Some(23));
    }
}

#[test]
fn endpoint_resolution_still_refuses_wrong_dialects_and_unknown_numbers() {
    let names = [EndpointName { number: 7, dialect: 3, name: b"primary".as_slice().into() }];
    for (number, dialect) in [(8, 3), (7, 2)] {
        let mut source = source();
        source.model.model.endpoint = number;
        source.model.model.dialect = dialect;
        source.resumes = true;
        let translated = charter(source).expect("bounded charter");
        assert!(encode(translated, &names, &wire::CEILINGS).is_none());
    }
}

#[test]
fn open_mode_is_encoded_and_never_grants_delivery() {
    let decoded = encoded_mode(smith_domain_run::Mode::Open);
    assert_eq!(*decoded.mode(), wire::Mode::Open);
    assert!(decoded.tools().deliver().is_none());
    assert!(decoded.tools().wait());
}

#[test]
fn finite_mode_is_encoded_with_delivery() {
    let decoded = encoded_mode(smith_domain_run::Mode::Finite);
    assert_eq!(*decoded.mode(), wire::Mode::Finite);
    assert!(decoded.tools().deliver().is_some());
}

#[test]
fn finite_report_is_encoded_with_delivery() {
    let mut value = assignment_fixture(b"Delivery", 16, 20, 4);
    value.policy.modify = true;
    let mut assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("report charter");
    assembled.delivers = true;
    let translated = charter(assembled).expect("bounded charter");
    let names = [EndpointName { number: 1, dialect: 3, name: b"primary".as_slice().into() }];
    let bytes = encode(translated, &names, &crate::limits()).expect("bounded encoding");
    let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("Smith codec accepts it");
    assert_eq!(*decoded.mode(), wire::Mode::Finite);
    assert!(decoded.tools().deliver().is_some());
    assert!(decoded.tools().families().modify());
    assert!(decoded.contract().change().is_none());
    assert!(decoded.contract().report().is_some());
    assert!(decoded.contract().failure().is_some());
    assert!(decoded.contract().verdicts().is_empty());
}

fn encoded_mode(mode: smith_domain_run::Mode) -> wire::Charter {
    let mut value = source();
    value.mode = mode;
    value.contract.change = Some(crate::ChangeRule { checks_must_pass: false, fields: Box::new([]) });
    let value = charter(value).unwrap();
    let names = [EndpointName { number: 7, dialect: 3, name: b"primary".as_slice().into() }];
    let bytes = encode(value, &names, &crate::limits()).expect("bounded encoding");
    wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("Smith codec accepts it")
}

// Run inputs from the roots' report fixtures, with workspace tools configured
// off where their starts do not prepare a workspace (domain/hosts/charters.md).
fn assignment_fixture(
    instructions: &[u8],
    words: u32,
    spend: u64,
    turns: u32,
) -> jig_domain_core_vocabulary::RunCharter {
    use jig_domain_core_vocabulary as core;
    use jig_domain_tasks as tasks;
    core::RunCharter {
        revision: 1,
        ended_by: tasks::EndedBy::Result,
        sources: Box::new([Some(1)]),
        accounts: Box::new([]),
        placement: jig_domain_fleet::PlacementConstraint {
            channel: skein_lib::Token::new(1),
            kind: jig_domain_fleet::HostKind::Worker,
        },
        policy: core::RunPolicy {
            instructions: instructions.into(),
            waiting: Duration::from_secs(1),
            resume: true,
            turns,
            time: Duration::from_secs(60),
            reserve_turns: 1,
            reserve_time: Duration::from_secs(10),
            model: core::Model {
                provider: 1,
                program: None,
                dialect: 3,
                endpoint: 1,
                name: b"scripted".as_slice().into(),
                cache_life_ms: 0,
                window: 8192,
                output: 1024,
                input_price: 1,
                cached_price: 1,
                cache_write_price: None,
                output_price: 1,
                price_unit: 1000,
                efforts: Box::new([]),
                effort: None,
            },
            alternatives: Box::new([]),
            inspect: false,
            modify: false,
            shell: false,
            agents: false,
            call_timeout: Duration::from_secs(10),
            tools: core::RunPolicy::EVERY_FAMILY,
        },
        contract: tasks::Contract::Report { words },
        authority: tasks::Authority {
            tools: tasks::Tools(1),
            models: tasks::Models::NONE,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        budget: spend,
    }
}

fn report_fixture_charter(instructions: &[u8], words: u32, spend: u64, turns: u32) -> Charter {
    Charter {
        mode: smith_domain_run::Mode::Finite,
        instructions: instructions.into(),
        brief: Box::new([]),
        tools: Box::new([]),
        wait: true,
        agents: false,
        workspace: crate::WorkspaceTools { inspect: false, modify: false, shell: false },
        delivers: false,
        conventions: None,
        words,
        contract: Contract {
            report: Some(TextRule { max: words, fields: Box::new([]) }),
            failure: Some(TextRule { max: words, fields: Box::new([]) }),
            verdicts: Box::new([]),
            change: None,
        },
        budget: Budget {
            turns,
            spend,
            time: Duration::from_secs(60),
            reserve: Reserve { turns: 1, time: Duration::from_secs(10) },
        },
        model: crate::Llm {
            grant: Some(1),
            model: Model {
                prices: Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
                dialect: 3,
                endpoint: 1,
                name: b"scripted".as_slice().into(),
                window: 8192,
                output: 1024,
                effort: None,
            },
        },
        models: Box::new([]),
        waiting: Duration::from_secs(1),
        resumes: false,
    }
}

#[test]
fn ops_report_fixture_keeps_its_charter() {
    let instructions = b"Complete the task within its authority.";
    let mut value = assignment_fixture(instructions, 128, 100, 8);
    value.policy.model.price_unit = 1_000_000;
    value.policy.agents = true;
    let mut expected = report_fixture_charter(instructions, 128, 100, 8);
    expected.model.model.prices.unit = 1_000_000;
    expected.agents = true;
    assert_eq!(crate::assignment(value, Box::new([]), Box::new([]), false), Some(expected));
}

#[test]
fn service_report_fixture_keeps_its_charter() {
    let mut value = assignment_fixture(b"Answer the person", 64, 20, 4);
    value.policy.model.name = b"small".as_slice().into();
    value.policy.waiting = Duration::from_secs(10);
    value.policy.time = Duration::from_secs(30);
    value.policy.reserve_time = Duration::from_secs(5);
    let mut expected = report_fixture_charter(b"Answer the person", 64, 20, 4);
    expected.model.model.name = b"small".as_slice().into();
    expected.waiting = Duration::from_secs(10);
    expected.budget.time = Duration::from_secs(30);
    expected.budget.reserve.time = Duration::from_secs(5);
    assert_eq!(crate::assignment(value, Box::new([]), Box::new([]), false), Some(expected));
}

#[test]
fn report_fixture_keeps_its_charter_and_conventions() {
    let value = assignment_fixture(b"Do the task", 128, 40, 8);
    let mut assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("report charter");
    let conventions =
        crate::Conventions { guide: b"AGENTS.md".as_slice().into(), checks: b"scripts/check".as_slice().into() };
    assembled.conventions = Some(conventions.clone());
    let sections: Box<[Section]> =
        Box::new([Section { title: b"Task".as_slice().into(), text: b"Say hello".as_slice().into() }]);
    let mut expected = report_fixture_charter(b"Do the task", 128, 40, 8);
    expected.conventions = Some(conventions);
    assembled.brief = sections.clone();
    expected.brief = sections;
    assert_eq!(assembled, expected);
    assert_eq!(charter(assembled), charter(expected));
}

#[test]
fn workspace_tools_follow_every_combination_of_run_policy() {
    for inspect in [false, true] {
        for modify in [false, true] {
            for shell in [false, true] {
                let mut value = assignment_fixture(b"Tools", 64, 20, 4);
                value.policy.inspect = inspect;
                value.policy.modify = modify;
                value.policy.shell = shell;
                let assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("report charter");
                assert_eq!(assembled.workspace, crate::WorkspaceTools { inspect, modify, shell });
                let translated = charter(assembled).expect("bounded charter");
                assert_eq!(translated.grants.tools.inspect, inspect);
                assert!(!translated.grants.tools.modify);
                assert_eq!(translated.grants.tools.shell, shell);
            }
        }
    }
}

#[test]
fn modify_needs_a_delivery() {
    use jig_domain_tasks as tasks;
    // The task fixture's contract text bound.
    const WORDS: u32 = 64;
    let fixture = assignment_fixture(b"Modify", WORDS, 20, 4);
    let change = tasks::Contract::Change { connector: 1, kind: 1, words: WORDS };
    let verdict = tasks::Contract::Verdict {
        choices: Box::new([tasks::Verdict { code: 1, followups: 0, words: WORDS, fields: Box::new([]) }]),
    };
    for (row, contract, ended_by, modify, expected) in [
        ("Report / Result / modify", fixture.contract.clone(), tasks::EndedBy::Result, true, false),
        ("Verdict / Result / modify", verdict, tasks::EndedBy::Result, true, false),
        ("Change / Result / modify", change.clone(), tasks::EndedBy::Result, true, true),
        ("Change / Requester / modify", change.clone(), tasks::EndedBy::Requester, true, false),
        ("Change / Result / no modify", change, tasks::EndedBy::Result, false, false),
    ] {
        let mut value = fixture.clone();
        value.contract = contract;
        value.ended_by = ended_by;
        value.policy.modify = modify;
        let assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("agent charter");
        assert_eq!(assembled.workspace.modify, modify, "{row}: policy rights");
        let translated = charter(assembled).expect("bounded charter");
        assert_eq!(translated.grants.tools.modify, expected, "{row}");
    }
}

#[test]
fn delivery_follows_contract_policy_and_workspace() {
    use jig_domain_tasks::{Contract, EndedBy};
    use smith_domain_run::outcome::{ChangeSpec, FieldRule};
    const WORDS: u32 = 16;
    // A whole-spec report bound (jig/docs/design/domain/hosts/charters.md).
    const LARGE_WORDS: u32 = 131_070;
    let report = Contract::Report { words: WORDS };
    let change = Contract::Change { connector: 1, kind: 1, words: WORDS };
    let verdict = Contract::Verdict {
        choices: Box::new([jig_domain_tasks::Verdict { code: 1, followups: 0, words: WORDS, fields: Box::new([]) }]),
    };
    let report_delivery = ChangeSpec {
        checks_must_pass: crate::CONTRACT_CHECKS_MUST_PASS,
        fields: Box::new([
            FieldRule { name: b"title".as_slice().into(), max: crate::CONTRACT_TEXT_MAX },
            FieldRule { name: b"body".as_slice().into(), max: crate::CONTRACT_TEXT_MAX },
        ]),
    };
    let mut large_report_delivery = report_delivery.clone();
    large_report_delivery.fields.get_mut(1).expect("body rule").max = LARGE_WORDS;
    let change_delivery = ChangeSpec {
        checks_must_pass: crate::CONTRACT_CHECKS_MUST_PASS,
        fields: Box::new([
            FieldRule { name: b"title".as_slice().into(), max: WORDS },
            FieldRule { name: b"body".as_slice().into(), max: WORDS },
        ]),
    };
    for (contract, ended_by, modify, delivers, delivery, writable) in [
        (report.clone(), EndedBy::Result, true, true, Some(&report_delivery), true),
        (Contract::Report { words: LARGE_WORDS }, EndedBy::Result, true, true, Some(&large_report_delivery), true),
        (report.clone(), EndedBy::Result, true, false, None, false),
        (report.clone(), EndedBy::Result, false, true, None, false),
        (report, EndedBy::Requester, true, true, None, false),
        (verdict, EndedBy::Result, true, true, None, false),
        (change.clone(), EndedBy::Result, true, false, Some(&change_delivery), true),
        (change.clone(), EndedBy::Result, false, false, Some(&change_delivery), false),
        (change, EndedBy::Requester, true, true, None, false),
    ] {
        let mut value = assignment_fixture(b"Delivery", WORDS, 20, 4);
        value.contract = contract.clone();
        value.ended_by = ended_by;
        value.policy.modify = modify;
        let mut assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("agent charter");
        assembled.delivers = delivers;
        let translated = charter(assembled).expect("bounded charter");
        assert_eq!(
            translated.grants.save,
            delivery.is_some() && translated.outcome.change.is_some(),
            "save only for a finite producer: {contract:?} / {ended_by:?}"
        );
        assert_eq!(translated.grants.deliver.as_ref(), delivery, "{contract:?} / {ended_by:?} / {modify} / {delivers}");
        assert_eq!(translated.grants.tools.modify, writable, "{contract:?} / {ended_by:?} / {modify} / {delivers}");
    }
}

#[test]
fn delivering_report_keeps_its_outcome_and_independent_delivery_bounds() {
    // Small reports keep a useful delivery floor; large reports keep their contract bound.
    for (words, text_max, body_max) in [(16, 16, 1024), (131_070, 1024, 131_070)] {
        let mut value = assignment_fixture(b"Delivery", words, 20, 4);
        value.policy.modify = true;
        let mut assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("report charter");
        assembled.delivers = true;
        let mut without_delivery = assembled.clone();
        without_delivery.delivers = false;
        let expected = charter(without_delivery).expect("bounded charter").outcome;
        let translated = charter(assembled).expect("bounded charter");
        assert_eq!(translated.outcome, expected, "delivery preserves the contract's finish forms");
        assert!(translated.outcome.change.is_none());
        assert_eq!(translated.outcome.report.expect("report text").max, text_max);
        assert_eq!(translated.outcome.failure.expect("failure text").max, text_max);
        assert!(translated.outcome.verdicts.is_empty(), "a report offers no no_change verdict");
        let delivery = translated.grants.deliver.expect("report's workspace delivers");
        assert!(delivery.checks_must_pass);
        assert_eq!(delivery.fields.len(), 2);
        let title = delivery.fields.first().expect("title rule");
        let body = delivery.fields.get(1).expect("body rule");
        assert_eq!(title.name.as_ref(), b"title");
        assert_eq!(body.name.as_ref(), b"body");
        assert_eq!(title.max, crate::CONTRACT_TEXT_MAX);
        assert_eq!(body.max, body_max);
    }
}

#[test]
fn contract_text_is_bounded_and_change_checks_are_required() {
    use jig_domain_tasks as tasks;
    // The contract text boundary and a whole-spec body bound (domain/hosts/charters.md).
    for words in [1023, 1024, 1025, 131_070] {
        let mut value = assignment_fixture(b"Contract", words, 20, 4);
        let report = crate::assignment(value.clone(), Box::new([]), Box::new([]), false).expect("report");
        assert_eq!(report.contract.report.expect("report text").max, words.min(crate::CONTRACT_TEXT_MAX));
        value.contract = tasks::Contract::Change { connector: 1, kind: 1, words };
        let change = crate::assignment(value, Box::new([]), Box::new([]), false).expect("change");
        assert_eq!(change.contract.failure.expect("failure text").max, words.min(crate::CONTRACT_TEXT_MAX));
        let no_change = &change.contract.verdicts[0];
        assert_eq!(no_change.name.as_ref(), b"no_change");
        assert_eq!(no_change.text_max, words.min(crate::CONTRACT_TEXT_MAX));
        assert!(no_change.fields.is_empty());
        assert_eq!((no_change.items.min, no_change.items.max), (0, 0));
        let rule = change.contract.change.expect("change rule");
        assert!(rule.checks_must_pass);
        assert_eq!(rule.checks_must_pass, crate::CONTRACT_CHECKS_MUST_PASS);
        assert_eq!(rule.fields[0].name.as_ref(), b"title");
        assert_eq!(rule.fields[1].name.as_ref(), b"body");
        assert_eq!(rule.fields[0].max, words.min(crate::CONTRACT_TEXT_MAX));
        assert_eq!(rule.fields[1].max, words);
    }
}

#[test]
fn alternative_models_keep_their_source_bindings_and_efforts() {
    let mut value = assignment_fixture(b"Models", 64, 20, 4);
    let mut alternative = value.policy.model.clone();
    alternative.endpoint = 2;
    alternative.effort = Some(b"high".as_slice().into());
    value.policy.alternatives = Box::new([alternative]);
    value.sources = Box::new([None, Some(7)]);
    let assembled = crate::assignment(value.clone(), Box::new([]), Box::new([]), false).expect("bound sources");
    assert_eq!(assembled.model.grant, None);
    assert_eq!(assembled.models[0].grant, Some(7));
    assert_eq!(assembled.models[0].model.endpoint, 2);
    assert_eq!(assembled.models[0].model.effort.as_deref(), Some(b"high".as_slice()));
    value.sources = Box::new([None]);
    assert_eq!(crate::assignment(value, Box::new([]), Box::new([]), false), None);
}

#[test]
fn verdicts_keep_named_fields_and_the_failure_text_ceiling() {
    use jig_domain_tasks as tasks;
    let mut value = assignment_fixture(b"Verdict", 64, 20, 4);
    value.contract = tasks::Contract::Verdict {
        choices: Box::new([tasks::Verdict {
            code: 7,
            followups: 0,
            words: 1025,
            fields: Box::new([tasks::FieldRule { name: b"reason".as_slice().into(), max: 80 }]),
        }]),
    };
    let assembled = crate::assignment(value, Box::new([]), Box::new([]), false).expect("verdict charter");
    assert_eq!(assembled.contract.failure.expect("failure").max, crate::CONTRACT_TEXT_MAX);
    let choice = &assembled.contract.verdicts[0];
    assert_eq!(choice.name.as_ref(), b"7");
    assert_eq!(choice.text_max, crate::CONTRACT_TEXT_MAX);
    assert_eq!(choice.fields[0].name.as_ref(), b"reason");
    assert_eq!(choice.fields[0].max, 80);
}

#[test]
fn a_persons_approval_and_a_missing_main_source_are_not_agent_assignments() {
    let mut value = assignment_fixture(b"Binding", 64, 20, 4);
    value.sources = Box::new([]);
    assert_eq!(crate::assignment(value.clone(), Box::new([]), Box::new([]), false), None);
    value.sources = Box::new([None]);
    value.contract = jig_domain_tasks::Contract::Approval {
        state: Box::new([]),
        part: jig_domain_tasks::Part {
            connector: 1,
            kind: jig_domain_tasks::PartKind::Result,
            key: jig_domain_tasks::PartKey::Number(1),
        },
    };
    assert_eq!(crate::assignment(value, Box::new([]), Box::new([]), false), None);
}

#[test]
fn assignment_keeps_brief_order_tools_mode_and_resume_policy() {
    for ended_by in
        [jig_domain_tasks::EndedBy::Result, jig_domain_tasks::EndedBy::Requester, jig_domain_tasks::EndedBy::Standing]
    {
        for resume in [false, true] {
            for resumed in [false, true] {
                let mut value = assignment_fixture(b"Brief", 64, 20, 4);
                value.ended_by = ended_by;
                value.policy.resume = resume;
                let sections: Box<[Section]> = Box::new([
                    Section { title: b"Task".as_slice().into(), text: b"First".as_slice().into() },
                    Section { title: b"Context".as_slice().into(), text: b"Second".as_slice().into() },
                ]);
                let tools = source().tools;
                let assembled =
                    crate::assignment(value, sections.clone(), tools.clone(), resumed).expect("report charter");
                assert_eq!(assembled.brief, sections);
                assert_eq!(assembled.tools, tools);
                assert_eq!(assembled.resumes, resume && resumed);
                assert_eq!(assembled.wait, crate::WAIT_GRANTED);
                let translated = charter(assembled).expect("bounded charter");
                assert_eq!(translated.resume, resume && resumed);
                assert_eq!(
                    translated.mode,
                    match ended_by {
                        jig_domain_tasks::EndedBy::Result | jig_domain_tasks::EndedBy::Closer(_) =>
                            smith_domain_run::Mode::Finite,
                        jig_domain_tasks::EndedBy::Requester | jig_domain_tasks::EndedBy::Standing =>
                            smith_domain_run::Mode::Open,
                    }
                );
                assert_eq!(translated.brief.sections[0].text.as_ref(), b"First");
                assert_eq!(translated.brief.sections[1].text.as_ref(), b"Second");
                assert_eq!(translated.grants.host_tools[0].name.as_ref(), b"read");
            }
        }
    }
}

fn hosted_fixture() -> jig_domain_core_vocabulary::Assignment {
    use jig_domain_core_vocabulary as core;
    use jig_domain_tasks as tasks;
    use skein_lib::Wall;
    jig_domain_core_vocabulary::Assignment {
        live: core::LiveAsk { text: true, reasoning: false, input: true },
        brief: Box::new([]),
        conversation: 3,
        task: 7,
        attempt: 5,
        charter: 1,
        run: Box::new(assignment_fixture(b"Do the task", 128, 40, 8)),
        messages: Box::new([
            tasks::Word {
                number: 11,
                from: tasks::Party::Person(2),
                kind: tasks::MessageKind::FinishRequested { revision: 3 },
                words: b"Finish now".as_slice().into(),
                at: Wall::from_nanos(0),
                hits: 1,
                eligible: true,
            },
            tasks::Word {
                number: 12,
                from: tasks::Party::Task(4),
                kind: tasks::MessageKind::EffectNotMade { entry: 9 },
                words: b"No effect".as_slice().into(),
                at: Wall::from_nanos(0),
                hits: 1,
                eligible: true,
            },
        ]),
        transcript: Box::new([b"committed turn".as_slice().into()]),
        answered: Box::new([core::Settled {
            key: core::CallKey { task: 7, attempt: 3, completion: 2, position: 1 },
            call: core::SettledCall {
                serial: 1,
                tool: b"read".as_slice().into(),
                answer: core::SettledAnswer::Host { error: true, body: b"refused".as_slice().into() },
            },
        }]),
        grants: Box::new([Grant {
            form: Form::Application,
            account: 1,
            generation: 2,
            valid: Duration::from_secs(30),
        }]),
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one hand-over pins its ordered sections, messages and settled calls")]
fn hosted_keeps_sections_messages_and_settled_calls_in_order() {
    use jig_domain_brief::{Core, GatherMissing, GatherPlaced};
    use jig_domain_core_vocabulary as core;
    use jig_domain_host as host;
    use skein_lib::Token;
    let mut value = hosted_fixture();
    let mut sections = skein_lib::List::with_capacity(19);
    let titles = [
        (Core::Task, b"Task".as_slice()),
        (Core::Lineage, b"Requesters".as_slice()),
        (Core::Results, b"Results".as_slice()),
        (Core::Plan, b"Plan".as_slice()),
        (Core::Attempts, b"Attempts".as_slice()),
        (Core::Calls, b"Calls".as_slice()),
        (Core::Waiting, b"Waiting".as_slice()),
        (Core::NotesIndex, b"Notes".as_slice()),
        (Core::TranscriptTail, b"Conversation".as_slice()),
    ];
    for (kind, _) in titles {
        sections.push(GatherPlaced::Core { kind, text: b"section text".as_slice().into() }).unwrap();
        sections.push(GatherPlaced::CoreMissing { kind, why: GatherMissing::Budget }).unwrap();
    }
    sections.push(GatherPlaced::Missing { connector: 1, kind: 2, why: GatherMissing::Late }).unwrap();
    value.brief = sections.into_boxed();
    let outcomes = [
        (core::DeliveryOutcome::Delivered, host::DeliveryOutcome::Delivered),
        (core::DeliveryOutcome::Nothing, host::DeliveryOutcome::Nothing),
        (core::DeliveryOutcome::Stale, host::DeliveryOutcome::Stale),
        (core::DeliveryOutcome::Refused, host::DeliveryOutcome::Refused),
        (core::DeliveryOutcome::Invalid, host::DeliveryOutcome::Refused),
        (core::DeliveryOutcome::Failed, host::DeliveryOutcome::Failed),
    ];
    let mut calls = skein_lib::List::with_capacity(7);
    for call in value.answered {
        calls.push(call).unwrap();
    }
    for (position, (outcome, _)) in outcomes.iter().enumerate() {
        calls
            .push(core::Settled {
                key: core::CallKey { task: 7, attempt: 4, completion: 8, position: u32::try_from(position).unwrap() },
                call: core::SettledCall {
                    serial: 2,
                    tool: b"deliver".as_slice().into(),
                    answer: core::SettledAnswer::Delivery {
                        outcome: *outcome,
                        evidence: b"evidence".as_slice().into(),
                    },
                },
            })
            .unwrap();
    }
    value.answered = calls.into_boxed();
    let result = crate::hosted(
        &value,
        crate::RootParts {
            tools: Box::new([]),
            endpoints: &[EndpointName { number: 1, dialect: 3, name: b"primary".as_slice().into() }],
            deployment: &[3; 16],
            limits: &wire::CEILINGS,
        },
    )
    .expect("hosted assignment");
    let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&result.assignment.charter)).unwrap();
    assert_eq!(decoded.brief().len(), 19);
    for ((_, title), pair) in titles.iter().zip(decoded.brief().as_slice().chunks_exact(2)) {
        assert_eq!(pair[0].title(), *title);
        assert_eq!(pair[0].body(), b"section text");
        assert_eq!(pair[1].title(), *title);
        assert_eq!(pair[1].body(), b"[missing]");
    }
    assert_eq!(decoded.brief().last().unwrap().title(), b"Context");
    assert_eq!(decoded.brief().last().unwrap().body(), b"[missing]");
    assert_eq!(
        result.messages.as_ref(),
        &[
            host::StartMessage {
                name: Token::new(11),
                sender: b"2".as_slice().into(),
                words: b"Finish now".as_slice().into(),
                finish_requested: true
            },
            host::StartMessage {
                name: Token::new(12),
                sender: b"4".as_slice().into(),
                words: b"Effect not made 9:\nNo effect".as_slice().into(),
                finish_requested: false
            },
        ]
    );
    for (word, start) in value.messages.iter().zip(result.messages.iter()) {
        assert_eq!(
            crate::message(Token::new(7), Token::new(5), word),
            host::LinkDown::Message {
                run: Token::new(7),
                attempt: Token::new(5),
                name: start.name,
                sender: start.sender.clone(),
                words: start.words.clone(),
                finish_requested: start.finish_requested,
            }
        );
    }
    assert_eq!(result.answered.len(), 7);
    for (call, replay) in value.answered.iter().zip(result.answered.iter()) {
        assert_eq!(
            crate::call_key(&replay.name, call.key.attempt),
            Some(crate::CallKey {
                activation: call.key.attempt,
                completion: call.key.completion,
                position: call.key.position,
            })
        );
        assert_eq!(replay.tool, call.call.tool);
    }
    assert_eq!(
        result.answered[0].answer,
        host::SettledAnswer::Host { error: true, body: b"refused".as_slice().into() }
    );
    for ((_, expected), replay) in outcomes.iter().zip(result.answered.iter().skip(1)) {
        assert_eq!(
            replay.answer,
            host::SettledAnswer::Delivery { outcome: *expected, evidence: b"evidence".as_slice().into() }
        );
    }
    assert_eq!(result.assignment.identity, crate::run_identity(&[3; 16], 7, 3));
    assert_eq!(result.assignment.run, Token::new(7));
    assert_eq!(result.assignment.attempt, Token::new(5));
    assert_eq!(result.assignment.workspace, None);
    assert!(!result.assignment.save);
    assert_eq!(result.assignment.live, host::LiveAsk { text: true, reasoning: false, input: true });
    assert_eq!(result.assignment.grants, value.grants);
    assert_eq!(result.turns, value.transcript);
    assert!(decoded.resume());
}

#[test]
fn hosted_titles_a_now_with_its_note_revision_and_write_time() {
    let mut value = hosted_fixture();
    value.brief = Box::new([jig_domain_brief::GatherPlaced::Core {
        kind: jig_domain_brief::Core::Now,
        text: b"Note 12, revision 3, written at 123 ns since Unix epoch.\nWrite your Now before you park.\n\nWaiting"
            .as_slice()
            .into(),
    }]);
    let result = crate::hosted(
        &value,
        crate::RootParts {
            tools: Box::new([]),
            endpoints: &[EndpointName { number: 1, dialect: 3, name: b"primary".as_slice().into() }],
            deployment: &[3; 16],
            limits: &wire::CEILINGS,
        },
    )
    .expect("hosted Now");
    let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&result.assignment.charter)).unwrap();
    let [section] = decoded.brief().as_slice() else { panic!("one Now") };
    assert_eq!(section.title(), b"Now: Note 12, revision 3, written at 123 ns since Unix epoch.");
    assert!(section.body().ends_with(b"Waiting"));
}

#[test]
fn hosted_refuses_a_connector_section_without_its_bytes() {
    let mut value = hosted_fixture();
    value.brief = Box::new([jig_domain_brief::GatherPlaced::Connector {
        connector: 1,
        kind: 2,
        token: skein_lib::Token::new(3),
        size: 4,
    }]);
    assert!(
        crate::hosted(
            &value,
            crate::RootParts {
                tools: Box::new([]),
                endpoints: &[EndpointName { number: 1, dialect: 3, name: b"primary".as_slice().into() }],
                deployment: &[3; 16],
                limits: &wire::CEILINGS,
            },
        )
        .is_none()
    );
}

#[test]
fn a_producers_save_grant_is_encoded_in_charter_version_four() {
    let mut source = source();
    source.contract.change = Some(crate::ChangeRule { checks_must_pass: true, fields: Box::new([]) });
    let value = charter(source).expect("producer charter");
    assert!(value.grants.save);
    let names = [EndpointName { number: 7, dialect: 3, name: b"primary".as_slice().into() }];
    let bytes = encode(value, &names, &wire::CEILINGS).expect("encoded charter");
    let decoded = wire::Charter::decode(&wire::CEILINGS, &mut Reader::new(&bytes)).expect("version four");
    assert!(decoded.tools().save());
    assert!(decoded.tools().deliver().is_some());
}
