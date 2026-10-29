//! Landing dependencies and the words delivered to hosted agents.
use skein_fake_llm_domain::api::{self, Script};
use std::net::SocketAddr;
use temper_real_loop_world::{
    Ran, argv,
    calls::{self, D1},
    forge, landing,
};

// Creating an unrelated root first shifts the parent's delegates by one.
const OUTSIDE_TASK: u64 = 1;
const SHIFTED_REPORT_TASK: u64 = 3;
const FIRST_PARENT_CALL: usize = 1;
const PARENT_CALL_AFTER_OUTSIDE: usize = 3;

fn call_turn(name: &[u8], arguments: &serde_json::Value) -> api::Turn {
    api::Turn {
        lines: Box::new([api::Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(arguments).expect("tool JSON").into(),
        }]),
        finish: api::Finish::ToolCalls,
        tokens: 20,
    }
}

fn report_turn() -> api::Turn {
    call_turn(b"finish", &serde_json::json!({"form":"report","text":"Reported.","fields":{}}))
}

fn refusal_scripts(target: u64, dependency: u64, cancel: bool) -> Box<[Script]> {
    use serde_json::json;
    let report = |words| {
        json!({
            "executor":{"kind":"agent","charter":1},
            "spec":{"words":words,"parameters":[]},
            "contract":{"kind":"report","words":128},
            "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},
                "budget":{"spend":10},"notes":0}
        })
    };
    let mut turns = vec![call_turn(b"delegate", &json!({"batch":[report("@qa-report"),report("@qa-prerequisite")]}))];
    if cancel {
        turns.push(call_turn(b"cancel", &json!({"target":dependency,"reason":"cancel before amendment"})));
    }
    turns.extend([
        call_turn(
            b"amend",
            &json!({"target":target,"revision":0,
            "amendment":{"dependencies":[dependency],"reason":"new prerequisite"}}),
        ),
        call_turn(b"wait", &json!({})),
        report_turn(),
    ]);
    if cancel {
        // The cancelled delegate's result can wake the wait before the report's arrives: finish again once it has.
        turns.push(report_turn());
    }
    Box::new([
        Script { cue: b"@qa-parent".as_slice().into(), turns: turns.into() },
        Script { cue: b"@qa-report".as_slice().into(), turns: Box::new([report_turn()]) },
        Script { cue: b"@qa-prerequisite".as_slice().into(), turns: Box::new([report_turn()]) },
        Script { cue: b"@qa-outside".as_slice().into(), turns: Box::new([call_turn(b"wait", &json!({}))]) },
    ])
}

fn refusal_fixture() -> forge::Fixture {
    let mut fixture = landing::kept::fixture();
    fixture.configure = |value, agent, forge| {
        (landing::kept::fixture().configure)(value, agent, forge);
        // The kept fixture grants delegate + amend (5); cancel adds its bit (8).
        for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
            project["policy"]["ceiling"]["tools"] = 13.into();
            project["policy"]["roles"][0]["authority"]["tools"] = 13.into();
            // The unrelated root and the amender each keep a 64-unit chat purse.
            let policy = &mut project["policy"];
            for path in ["/ceiling/budget/spend", "/period_spend", "/roles/0/period_spend"] {
                let spend = policy.pointer_mut(path).expect("fixture spend member");
                *spend = spend.as_u64().expect("fixture spend").max(128).into();
            }
        }
    };
    fixture
}

fn refusal_story(scripts: fn(SocketAddr) -> Box<[Script]>, outside: bool) -> Ran {
    temper_real_loop_world::run_forge(refusal_fixture(), scripts, || {
        let mut steps = vec![calls::sign_in(D1)];
        if outside {
            // Sign-in is call zero; the outside chat is call one.
            const OUTSIDE_CHAT: usize = 1;
            steps.extend([
                calls::call(D1, argv!["chat", "start", "--project", "1", "--key", "qa-outside", "@qa-outside"]),
                calls::call(
                    D1,
                    argv!["task", "show", calls::task(OUTSIDE_CHAT), "--until", "phase:idle", "--timeout", "5"],
                ),
            ]);
        }
        let parent_call = if outside { PARENT_CALL_AFTER_OUTSIDE } else { FIRST_PARENT_CALL };
        steps.extend([
            calls::call(D1, argv!["chat", "start", "--project", "1", "--key", "qa-parent", "@qa-parent"]),
            calls::call(D1, argv!["task", "show", calls::task(parent_call), "--until", "ended", "--timeout", "5"]),
        ]);
        steps
    })
}

fn refusal_words(ran: &Ran) -> String {
    ran.llm()
        .iter()
        .filter_map(|record| {
            if let skein_fake_peers::llm::Observation::Query { query, .. } = record { Some(query) } else { None }
        })
        .flat_map(|query| query.messages.iter())
        .flat_map(|message| message.parts.iter())
        .filter_map(|part| {
            if let api::Part::ToolOutput { output, .. } = part {
                let text = String::from_utf8_lossy(output);
                text.contains("Refused").then(|| text.into_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn unreachable_dependency_refusal_names_the_task_to_the_agent() {
    // jig's domain/tasks.md 7; engine.md 7.3: a new dependency must be in the amender's reach.
    // Outside root 1, parent 2, report delegate 3, prerequisite delegate 4.
    let ran = refusal_story(|_| refusal_scripts(SHIFTED_REPORT_TASK, OUTSIDE_TASK, false), true);
    let words = refusal_words(&ran);
    assert!(
        words.contains('1')
            && (words.to_ascii_lowercase().contains("depend") || words.to_ascii_lowercase().contains("reach")),
        "the agent must hear which dependency is outside its reach: {words:?}"
    );
}
