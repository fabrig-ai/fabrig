//! Native writes and later checks use the standard production derivation,
//! simulated time and the fake filesystem, without starting OS processes.

use skein_fake_llm_domain::api;
use skein_lib::List;
use smith_agent_process_world::{World, charter};

// The end-to-end regression repeats this twenty-byte line 160 times: 3,200
// content bytes across a 2,048-byte input threshold.
const CODING_LINES: usize = 160;
const INPUT_THRESHOLD: usize = 2048;
// Seven provider turns: large write, two reads, edit, replacement, shell, finish.
const CALLS: usize = 7;
const OUTPUT: u32 = 32_768;
// Each rounded fixture turn can cost one spend unit; twenty covers seven calls.
const SPEND: u64 = 20;

fn coding_charter() -> Box<[u8]> {
    let limits = &smith_charter::CEILINGS;
    smith_agent_process_world::wire::edit_charter(&charter(), |parts| {
        let mut budget = parts.budget.clone().into_parts();
        budget.turns = u32::try_from(CALLS).expect("coding turns");
        budget.spend = SPEND;
        // Standard checks receive five minutes; the charter reserves that inside
        // a ten-minute virtual run, leaving the model five minutes to work.
        budget.time = skein_lib::Duration::from_secs(600);
        budget.reserve = smith_charter::Reserve::new(
            limits,
            smith_charter::ReserveParts { turns: 0, time: skein_lib::Duration::from_secs(300) },
        )
        .expect("check reserve");
        parts.budget = smith_charter::Budget::new(limits, budget).expect("coding budget");
        let mut main = parts.main.clone().into_parts();
        main.output = OUTPUT;
        parts.main = smith_charter::Llm::new(limits, main).expect("coding model");
        let mut tools = parts.tools.clone().into_parts();
        tools.families = smith_charter::Families::new(
            limits,
            smith_charter::FamiliesParts { skein_bools: [true, true, true, false] },
        )
        .expect("workspace tools");
        parts.tools = smith_charter::Tools::new(limits, tools).expect("coding tools");
        let mut contract = parts.contract.clone().into_parts();
        contract.change = Some(
            smith_charter::ChangeRule::new(
                limits,
                smith_charter::ChangeRuleParts { checks_must_pass: true, fields: List::with_capacity(0) },
            )
            .expect("checked change"),
        );
        parts.contract = smith_charter::Contract::new(limits, contract).expect("coding contract");
        parts.conventions = Some(
            smith_charter::Conventions::new(
                limits,
                smith_charter::ConventionsParts {
                    guide: b"AGENTS.md".as_slice().into(),
                    checks: b".smith-test/check".as_slice().into(),
                },
            )
            .expect("check convention"),
        );
    })
}

#[test]
fn a_standard_coding_write_reaches_following_calls_and_checks() {
    let content = "// coding regression\n".repeat(CODING_LINES);
    let large = serde_json::to_vec(&serde_json::json!({"path":"result.txt", "content":content})).expect("write input");
    assert!(large.len() > INPUT_THRESHOLD, "the write crosses the input threshold");
    let mut calls = vec![(b"write".as_slice(), large)];
    calls.extend(
        [
            ("read", r#"{"path":"result.txt"}"#),
            ("read", r#"{"path":"original.txt"}"#),
            ("edit", r#"{"path":"original.txt","edits":[{"old":"original","new":"edited"}]}"#),
            ("write", r#"{"path":"result.txt","content":"new\n"}"#),
            ("shell", r#"{"command":"printf 'ran\n'"}"#),
            ("finish", r#"{"form":"change","text":"","fields":{}}"#),
        ]
        .into_iter()
        .map(|(tool, input)| (tool.as_bytes(), input.as_bytes().to_vec())),
    );
    let mut world = World::new(37, &coding_charter());
    world.standard_workspace();
    world.script(
        calls
            .into_iter()
            .map(|(name, arguments)| {
                smith_agent_world::fixture::calls(
                    Box::new([api::Line::Call { name: name.into(), arguments: arguments.into() }]),
                    1,
                )
            })
            .collect(),
    );
    assert!(world.settle());
    assert_eq!(world.peer_queries().len(), CALLS, "every subsequent call completed; answer={:?}", world.answer());
    for query in world.peer_queries().into_iter().skip(1) {
        let output = query
            .messages
            .iter()
            .flat_map(|message| &message.parts)
            .filter_map(|part| match part {
                api::Part::ToolOutput { output, is_error, .. } => Some((output, is_error)),
                api::Part::Text { .. } | api::Part::ToolCall { .. } | api::Part::Opaque { .. } => None,
            })
            .next_back()
            .expect("follow-up workspace answer");
        assert!(!output.1, "workspace call succeeded: {}", String::from_utf8_lossy(output.0));
    }
    let reads: Vec<_> =
        world.peer_queries()[2]
            .messages
            .iter()
            .flat_map(|message| &message.parts)
            .filter_map(|part| {
                if let api::Part::ToolOutput { output, is_error: false, .. } = part { Some(output) } else { None }
            })
            .collect();
    let read = reads.last().expect("large file read answer");
    assert_eq!(
        read.windows(b"// coding regression".len()).filter(|bytes| *bytes == b"// coding regression").count(),
        CODING_LINES,
        "the read returns every line: {}",
        String::from_utf8_lossy(read)
    );
    assert_eq!(world.checked_files(), [b"new\n".to_vec()], "the subsequent check sees the replacement");
    assert_eq!(world.workspace_file(b"result.txt"), b"new\n");
    assert_eq!(world.workspace_file(b"original.txt"), b"edited\n");
    assert!(matches!(world.answer().expect("host answer").result(), smith_channel::RunResult::Accepted(_)));
    assert_eq!(world.observed().iter().filter(|frame| frame.kind == 0x0107).count(), 1, "one checked delivery");
    world.assert_agent_clean();
}

// A short title and body each fit this delivery field bound.
const DELIVERY_FIELD_BYTES: u32 = 64;

fn report_delivery_charter() -> Box<[u8]> {
    // Two required delivery fields, each bounded to a short title or body.
    const FIELDS: u32 = 2;
    smith_agent_process_world::wire::edit_charter(&coding_charter(), |parts| {
        let limits = &smith_charter::CEILINGS;
        let mut contract = parts.contract.clone().into_parts();
        contract.change = None;
        parts.contract = smith_charter::Contract::new(limits, contract).expect("report contract");
        let mut fields = List::with_capacity(FIELDS);
        for name in [b"title".as_slice(), b"body"] {
            fields
                .push(
                    smith_charter::FieldRule::new(
                        limits,
                        smith_charter::FieldRuleParts { name: name.into(), max: DELIVERY_FIELD_BYTES },
                    )
                    .expect("delivery field"),
                )
                .expect("two delivery fields");
        }
        let mut tools = parts.tools.clone().into_parts();
        tools.deliver = Some(
            smith_charter::ChangeRule::new(limits, smith_charter::ChangeRuleParts { checks_must_pass: true, fields })
                .expect("checked delivery grant"),
        );
        parts.tools = smith_charter::Tools::new(limits, tools).expect("delivery tools");
    })
}

/// Delivery is an independent grant beside a report (domain/run.md, 3.1 and 8.4).
#[test]
fn a_report_run_offers_its_delivery_grant_and_continues() {
    const REPORT: &[u8] = b"The files were delivered.";
    let charter = report_delivery_charter();
    let mut world = World::new(38, &charter);
    world.standard_workspace();
    world.script(
        [
            ("write", r#"{"path":"result.txt","content":"delivered\n"}"#),
            ("deliver", r#"{"fields":{"title":"Design","body":"Keep the design."}}"#),
            ("finish", r#"{"form":"report","text":"The files were delivered.","fields":{}}"#),
        ]
        .into_iter()
        .map(|(name, arguments)| {
            smith_agent_world::fixture::calls(
                Box::new([api::Line::Call { name: name.as_bytes().into(), arguments: arguments.as_bytes().into() }]),
                1,
            )
        })
        .collect(),
    );
    assert!(world.settle());
    let queries = world.peer_queries();
    let first = queries.first().expect("report run reaches the model");
    let deliver = first
        .tools
        .iter()
        .find(|tool| tool.name.as_ref() == b"deliver")
        .expect("the independent delivery grant offers deliver beside report");
    let schema: serde_json::Value = serde_json::from_slice(&deliver.parameters).expect("delivery schema");
    assert_eq!(schema["properties"]["fields"]["required"], serde_json::json!(["title", "body"]));
    for name in ["title", "body"] {
        assert_eq!(schema["properties"]["fields"]["properties"][name]["maxLength"], DELIVERY_FIELD_BYTES);
    }
    let last = queries.last().expect("report turn");
    assert!(
        last.messages.iter().flat_map(|message| &message.parts).any(|part| {
            matches!(part, api::Part::ToolOutput { output, is_error: false, .. }
            if output.as_ref() == b"delivered\nreceipt directory=0 text=\"checked files kept\"")
        }),
        "delivery is answered before the report"
    );
    assert_eq!(world.checked_files(), [b"delivered\n".to_vec()]);
    let answer = world.answer().expect("host answer");
    let smith_channel::RunResult::Accepted(accepted) = answer.result() else { panic!("report accepted: {answer:?}") };
    let result =
        smith_charter::RunResult::decode(&smith_charter::CEILINGS, &mut skein_lib::Reader::new(accepted.result()))
            .expect("report result");
    assert_eq!(result.form(), &smith_charter::Form::Report);
    assert_eq!(result.text(), REPORT);
    world.assert_agent_clean();
}
