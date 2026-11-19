//! One ctl answer releases two report delegates (testing/real-loop.md, the turn-landing setup's stories).
use crate::{Ran, argv, calls, fixtures, llm};
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};

pub const GOAL: u64 = 1;
pub const LATCH: u64 = 2;
pub const REPORTS: [u64; 2] = [3, 4];
pub const BEFORE: [usize; 2] = [4, 5];
pub const PERSON: usize = 6;
pub const RELEASE: usize = 7;
pub const AFTER: [usize; 2] = [8, 9];
pub const ENDED: usize = 10;
pub const LATCH_ENDED: usize = 11;

fn configure(value: &mut temper_playground::documents::Configuration, agent: &mut Value) {
    agent["profile"]["declared"]["calls_per_response"] = 2.into();
    value.jig_over("hub.run_calls", 2);
    fixtures::own_policy(value);
    for project in value["world_settings"]["policies"].as_array_mut().expect("policies") {
        let policy = &mut project["policy"];
        policy["period_spend"] = 1000.into();
        policy["roles"][0]["period_spend"] = 1000.into();
        let authority = json!({"tools":1,"grants":[],
                    "delegation":{"kinds":[{"kind":"charter","number":1},{"kind":"role","number":0}],"tasks":4,"depth":2},
                    "budget":{"spend":500,"deadline_unix_seconds":null},"notes":0,"note_resources":[]});
        policy["ceiling"] = authority.clone();
        policy["roles"][0]["authority"] = authority;
    }
    // One goal, its person latch and two single-run reports; one worker slot suffices.
    for (name, count) in [
        ("tasks.tasks", 4),
        ("tasks.project_tasks", 4),
        ("tasks.tree_tasks", 4),
        ("tasks.batch", 3),
        ("authority.batch", 3),
        ("tasks.delegates", 3),
        ("people.operations", 4),
        ("tasks.exemptions", 4),
        ("documents.chips", 4),
        ("documents.lines", 4),
        ("tasks.result_bytes", 128),
        ("tasks.authority_bytes", 512),
        ("people.inbox_tasks", 8),
        ("tasks.inbox_messages", 8),
    ] {
        value.jig_over(name, count);
    }
}

fn call(name: &[u8], arguments: &Value) -> Line {
    Line::Call { name: name.into(), arguments: serde_json::to_vec(arguments).expect("tool JSON").into() }
}

fn turn(lines: Vec<Line>) -> Turn {
    Turn { lines: lines.into(), finish: Finish::ToolCalls, tokens: 1 }
}

fn report(text: &str) -> Turn {
    turn(vec![call(b"finish", &json!({"form":"report","text":text,"fields":{}}))])
}

fn scripts() -> Box<[Script]> {
    let authority = |spend| {
        json!({"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},
        "budget":{"spend":spend},"notes":0})
    };
    let mut batch = vec![json!({
        "executor":{"kind":"person","person":1},
        "spec":{"words":"Release the turn landing latch"},
        "contract":{"kind":"report","words":64},"authority":authority(0)
    })];
    for cue in ["@turn-latch-x", "@turn-latch-y"] {
        batch.push(json!({
            "executor":{"kind":"agent","charter":1},"spec":{"words":cue},
            "contract":{"kind":"report","words":64},"authority":authority(64),
            "dependencies":[{"kind":"batch","number":0}]
        }));
    }
    Box::new([
        Script {
            cue: b"@turn-latch ".as_slice().into(),
            turns: Box::new([
                turn(vec![call(b"delegate", &json!({"batch":batch})), call(b"wait", &json!({}))]),
                report("Both reports ended."),
            ]),
        },
        Script { cue: b"@turn-latch-x".as_slice().into(), turns: Box::new([report("X ended.")]) },
        Script { cue: b"@turn-latch-y".as_slice().into(), turns: Box::new([report("Y ended.")]) },
    ])
}

#[must_use]
pub fn run() -> Ran {
    use calls::{D1, call, number, sign_in};
    crate::run_configured(
        || llm::scripted(scripts()),
        configure,
        || {
            vec![
                sign_in(D1),
                call(
                    D1,
                    argv![
                        "goal",
                        "start",
                        "--project",
                        "1",
                        "--charter",
                        "1",
                        "--budget",
                        "5.00",
                        "--deadline",
                        "2030-01-01T00:00Z",
                        "--key",
                        "latch",
                        "@turn-latch Release two reports\nPerson: 1;"
                    ],
                ),
                call(D1, argv!["task", "show", number(GOAL), "--until", "phase:idle", "--json"]),
                call(D1, argv!["task", "show", number(GOAL), "--tree", "--json"]),
                call(D1, argv!["task", "show", number(REPORTS[0]), "--json"]),
                call(D1, argv!["task", "show", number(REPORTS[1]), "--json"]),
                call(D1, argv!["task", "show", number(LATCH), "--json"]),
                call(D1, argv!["task", "answer", number(LATCH), "--rev", "0", "Ready together", "--key", "release"]),
                call(D1, argv!["task", "show", number(REPORTS[0]), "--until", "ended", "--json"]),
                call(D1, argv!["task", "show", number(REPORTS[1]), "--until", "ended", "--json"]),
                call(D1, argv!["task", "show", number(GOAL), "--until", "ended", "--json"]),
                call(D1, argv!["task", "show", number(LATCH), "--json"]),
            ]
            .into_iter()
            .map(|step| if matches!(step, calls::Step::Call { .. }) { step.succeeds() } else { step })
            .collect()
        },
    )
}
