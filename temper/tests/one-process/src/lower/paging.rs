//! Template paging on the simulated native transport (temper/docs/design/domain/testing.md, 2.4).
use serde_json::{Value, json};
fn turn(name: &[u8], value: &Value) -> skein_fake_llm_domain::api::Turn {
    use skein_fake_llm_domain::api::{Finish, Line, Turn};
    Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(value).expect("tool JSON").into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    }
}
pub(super) fn scripts() -> Box<[skein_fake_llm_domain::api::Script]> {
    use skein_fake_llm_domain::api::Script;
    let member = |words, executor, dependencies| {
        json!({
            "executor":executor, "spec":{"words":words,"parameters":[]},
            "contract":{"kind":"report","words":128},
            "authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":40},"notes":0},
            "dependencies":dependencies
        })
    };
    let agent = json!({"kind":"agent","charter":1});
    let delegates = json!({"batch":[
        member("@bounds-dependency", agent.clone(), json!([])),
        member("@bounds-ended", agent.clone(), json!([{"kind":"batch","number":0}]))
    ]});
    let live = json!({"batch":[
        member("Bounds person", json!({"kind":"role","role":0}), json!([])),
        member("Bounds waiting one", agent.clone(), json!([{"kind":"batch","number":0}]))
    ]});
    let last = json!({"batch":[member("Bounds waiting last", agent, json!([{"kind":"existing","number":4}]))]});
    Box::new([
        Script {
            cue: b"@bounds-root".as_slice().into(),
            // The three live delegates keep both reports from being the last
            // result, so the root's default wake policy leaves it parked.
            turns: Box::new([
                turn(b"delegate", &delegates),
                turn(b"delegate", &live),
                turn(b"delegate", &last),
                turn(b"wait", &json!({})),
            ]),
        },
        Script {
            cue: b"@bounds-dependency".as_slice().into(),
            turns: Box::new([turn(b"finish", &json!({"form":"report","text":"Dependency ended","fields":{}}))]),
        },
        Script {
            cue: b"@bounds-ended".as_slice().into(),
            turns: Box::new([turn(b"finish", &json!({"form":"report","text":"Dependent ended","fields":{}}))]),
        },
    ])
}
