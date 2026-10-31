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
            .map(|(name, arguments)| api::Turn {
                lines: Box::new([api::Line::Call { name: name.into(), arguments: arguments.into() }]),
                finish: api::Finish::ToolCalls,
                tokens: 1,
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
