use jig_channel_world::both_ways::{self, Case, Grant, Run};
use skein_fake_llm_domain::api::{Finish, Line, Script, Scripts, Turn};
use smith_fake_claude::{End, Story};

fn call(name: &[u8], arguments: &[u8]) -> Turn {
    Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    }
}
fn scripts(turns: Vec<Turn>) -> Scripts {
    Scripts {
        cued: Box::new([Script { cue: Box::new([]), turns: turns.into_boxed_slice() }]),
        asked: Box::new([]),
        otherwise: Box::new([]),
    }
}
fn finish() -> Turn {
    call(b"finish", br#"{"form":"report","text":"Hello","fields":{}}"#)
}
fn answered(run: &Run) {
    assert!(
        run.terminals.iter().any(|answer| matches!(&answer.end,
        jig_domain_tasks::End::Finished { result: jig_domain_tasks::TaskResult::Report { words }, .. }
        if words.as_ref() == b"Hello")),
        "terminals {:?}; {}",
        run.terminals,
        run.diagnostics
    );
}
#[test]
fn a_key_routes_a_claude_model_directly_without_starting_claude() {
    let run = both_ways::run(Case::Key, &Story::default(), scripts(vec![call(b"wait", b"{}"), finish()]));
    assert_eq!(run.assignments.len(), 1, "{:?}; {}", run.assignments, run.diagnostics);
    assert_eq!(run.assignments[0].endpoint, b"direct");
    assert_eq!(run.assignments[0].grant, Grant::Key);
    assert_eq!(run.program_spawns, 0, "no attempted spawn, even of the missing executable");
    assert!(run.observations.is_empty(), "no Claude invocation, including auth status");
    assert_eq!(
        run.credentials,
        vec![(skein_llm::Provider::Anthropic, Some(skein_llm::Scheme::Key), Some(0)); 2],
        "{}",
        run.diagnostics
    );
    assert_eq!(run.requests.len(), 2, "{}", run.diagnostics);
    assert_eq!(run.request_heads, 2);
    let messages = run.requests[1]["messages"].as_array().expect("messages");
    let blocks: Vec<_> =
        messages.iter().flat_map(|message| message["content"].as_array().expect("message blocks")).collect();
    let call =
        blocks.iter().find(|block| block["type"] == "tool_use" && block["name"] == "wait").expect("wait was called");
    let result = blocks
        .iter()
        .find(|block| block["type"] == "tool_result" && block["tool_use_id"] == call["id"])
        .expect("matching tool result reaches the next request");
    assert_ne!(result["is_error"], true);
    assert_eq!(result["content"], "Messages arrived. Read them before continuing.");
    assert!(messages.iter().any(|message| message["content"].to_string().contains("The tool ran")));
    assert_eq!(run.assignments[0].model, b"fake");
    answered(&run);
}
#[test]
fn a_login_routes_the_same_claude_model_through_the_program() {
    let story = Story {
        turns: vec![smith_fake_claude::Turn {
            steps: vec![],
            end: End::Success {
                structured_output: Some(serde_json::json!({"outcome":{"form":"report","text":"Hello","fields":{}}})),
            },
        }],
        ..Story::default()
    };
    let run = both_ways::run(Case::Login, &story, scripts(vec![finish()]));
    assert_eq!(run.assignments.len(), 1, "{:?}; {}", run.assignments, run.diagnostics);
    assert_eq!(run.assignments[0].endpoint, b"fake");
    assert_eq!(run.assignments[0].grant, Grant::Login);
    assert!(run.observations.iter().any(|row| row.arguments.first().is_some_and(|arg| arg == "-p")), "{run:?}");
    assert_eq!(run.request_heads, 0, "the peer sees no request, including a model list");
    assert_eq!(run.requests.len(), 0);
    assert_eq!(run.credentials.len(), 0);
    assert_eq!(run.assignments[0].model, b"fake");
    answered(&run);
}

fn says(text: &[u8]) -> Turn {
    Turn { lines: Box::new([Line::Text { text: text.into() }]), finish: Finish::Stop, tokens: 1 }
}
fn crossed(run: &Run, first: Grant, second: Grant) {
    assert_eq!(run.assignments.len(), 2, "{}", run.diagnostics);
    let [before, after] = run.assignments.as_slice() else { unreachable!("two assignments") };
    assert_eq!(before.task, after.task);
    assert_ne!(before.attempt, after.attempt);
    assert_ne!(
        before.source, after.source,
        "{:?}; logins {:?}; {}",
        run.assignments, run.login_states, run.diagnostics
    );
    assert_eq!((&before.grant, &after.grant), (&first, &second));
    for assignment in [before, after] {
        assert_eq!(
            assignment.endpoint,
            if assignment.grant == Grant::Key { b"direct".as_slice() } else { b"fake".as_slice() }
        );
        assert_eq!(assignment.model, b"fake");
    }
    assert_eq!(run.sources.len(), 2, "one billing account, two ordered sources");
    let program =
        run.sources.iter().find(|(_, _, _, kind)| *kind == jig_domain_accounts::Kind::Program).expect("program source");
    let key = run.sources.iter().find(|(_, _, _, kind)| *kind == jig_domain_accounts::Kind::Key).expect("key source");
    assert_eq!(program.0, key.0, "one account");
    assert!(program.2 < key.2, "login stays before key");
    assert_eq!(run.terminals.len(), 2);
    assert_eq!(run.terminals[0].end, jig_domain_tasks::End::Parked);
}
#[test]
fn a_login_then_key_carries_the_conversation_through_a_direct_summary() {
    let story = Story {
        turns: vec![smith_fake_claude::Turn::new(vec![
            smith_fake_claude::Step::Text("committed plain answer".into()),
            smith_fake_claude::Step::Call { tool: "wait".into(), input: serde_json::json!({}) },
        ])],
        ..Story::default()
    };
    let run = both_ways::run(
        Case::LoginToKey,
        &story,
        Scripts {
            asked: Box::new([Script {
                cue: b"Summarise the work so far.".as_slice().into(),
                turns: Box::new([says(b"direct summary")]),
            }]),
            cued: Box::new([]),
            otherwise: Box::new([finish()]),
        },
    );
    crossed(&run, Grant::Login, Grant::Key);
    assert!(run.login_states.contains(&jig_protocol_worker_wire::LoginState::Usable));
    assert_eq!(run.login_states.last(), Some(&jig_protocol_worker_wire::LoginState::NotSignedIn));
    assert_eq!(run.requests.len(), 2, "summary before ordinary completion: {}", run.diagnostics);
    assert_eq!(run.requests[0]["tool_choice"]["type"], "none");
    assert!(run.requests[0]["messages"].to_string().contains("committed plain answer"));
    assert!(!run.requests[0]["messages"].to_string().contains("direct summary"));
    assert!(run.requests[1]["messages"].to_string().contains("direct summary"));
    assert!(run.requests[1]["messages"].to_string().contains("Continue on the other source"));
    assert_eq!(run.credentials, vec![(skein_llm::Provider::Anthropic, Some(skein_llm::Scheme::Key), Some(0)); 2]);
    assert_eq!(
        run.observations.iter().filter(|row| row.arguments.first().is_some_and(|arg| arg == "-p")).count(),
        1,
        "only the first run uses Claude"
    );
    answered(&run);
}
#[test]
fn a_key_then_login_carries_the_conversation_through_a_program_summary() {
    let story = Story {
        summary: "program summary".into(),
        turns: vec![smith_fake_claude::Turn {
            steps: vec![],
            end: End::Success {
                structured_output: Some(serde_json::json!({"outcome":{"form":"report","text":"Hello","fields":{}}})),
            },
        }],
        ..Story::default()
    };
    let mut first = call(b"wait", b"{}");
    first.lines = vec![Line::Text { text: b"committed direct answer".as_slice().into() }, first.lines[0].clone()]
        .into_boxed_slice();
    let run = both_ways::run(Case::KeyToLogin, &story, scripts(vec![first]));
    crossed(&run, Grant::Key, Grant::Login);
    assert!(run.login_states.contains(&jig_protocol_worker_wire::LoginState::NotSignedIn));
    assert_eq!(run.login_states.last(), Some(&jig_protocol_worker_wire::LoginState::Usable));
    assert_eq!(run.requests.len(), 1, "only the first run uses the direct peer");
    let print: Vec<_> =
        run.observations.iter().filter(|row| row.arguments.first().is_some_and(|arg| arg == "-p")).collect();
    assert_eq!(print.len(), 2, "summary then ordinary program completion");
    assert!(!print[0].arguments.iter().any(|arg| arg == "--mcp-config"));
    assert!(print[1].arguments.iter().any(|arg| arg == "--mcp-config"));
    assert!(print.iter().all(|row| !row.arguments.iter().any(|arg| arg == "--resume")));
    assert_eq!(run.received.len(), 2);
    assert!(run.received[0]["message"]["content"].as_str().expect("summary input").contains("committed direct answer"));
    assert!(run.received[1]["message"]["content"].to_string().contains("program summary"));
    assert!(run.received[1]["message"]["content"].to_string().contains("Continue on the other source"));
    answered(&run);
}
