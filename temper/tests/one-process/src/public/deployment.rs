//! The public goal's scripted producer (temper/docs/design/protocol/sign-in.md, 2.5).
use crate::fixtures;

pub(super) fn provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    };
    let grant = |kind| serde_json::json!({"connector":fixtures::FORGE_CONNECTOR,"kind":kind,"segments":["forge","127.0.0.1:3000"],"terminal":"open","last":"acme"});
    let delegate = serde_json::json!({"batch":[{
        "executor":{"kind":"procedure","connector":fixtures::FORGE_CONNECTOR,"code":fixtures::PROCEDURE_CODE},
        "tracked":{"priority":1},
        "spec":{"words":"@edit Public change title\nPRIVATE_SPEC_BODY","parameters":[
            {"name":fixtures::PARAMETER_NAMES[0],"kind":"resource","connector":fixtures::FORGE_CONNECTOR,"resource":fixtures::REPOSITORY_TAG},
            {"name":fixtures::PARAMETER_NAMES[1],"kind":"bytes","value":"main"},
            {"name":fixtures::PARAMETER_NAMES[2],"kind":"bytes","value":std::str::from_utf8(fixtures::MERGE_BRANCH).expect("branch")},
            {"name":10,"kind":"bytes","value":"PRIVATE_PARAMETER"}]},
        "contract":{"kind":"change","connector":fixtures::FORGE_CONNECTOR,"change_kind":fixtures::CHANGE_KIND,"words":fixtures::CHANGE_WORDS},
        "authority":{"tools":fixtures::PRODUCER_TOOLS,"grants":([3,4,11].map(grant)),"delegation":{"kinds":[{"kind":"agent","number":fixtures::AGENT_KIND}],"tasks":fixtures::PRODUCER_TASKS,"depth":fixtures::PRODUCER_DEPTH},"budget":{"spend":fixtures::PRODUCER_SPEND},"notes":fixtures::PRODUCER_NOTES}
    }]});
    let delegate = serde_json::to_vec(&delegate).expect("delegate JSON");
    let finish = call(b"finish", br#"{"form":"change","text":"PRIVATE_RESULT","fields":{"title":"PRIVATE_PULL_TITLE","body":"PRIVATE_PULL_BODY"}}"#);
    fixtures::scripted_provider(Box::new([
        Script {
            cue: b"@plan".as_slice().into(),
            turns: Box::new([call(b"delegate", &delegate), call(b"wait", b"{}"), call(b"wait", b"{}")]),
        },
        Script { cue: b"@edit".as_slice().into(), turns: Box::new([finish.clone()]) },
        Script {
            cue: b"Resolve the merge conflict".as_slice().into(),
            turns: Box::new([call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish]),
        },
    ]))
}
