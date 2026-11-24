//! The public goal's scripted producer (temper/docs/design/protocol/sign-in.md, 2.5).
use crate::fixtures;
use std::cell::Cell;

pub(super) const PRIVATE_PROJECT: &str = "PRIVATE_PROJECT_NAME";
pub(super) const FORGE_WEB: &str = "https://private-forge.example.invalid";
pub(super) const CHAT_REPLY: &[u8] = b"PRIVATE_CHAT_REPLY";
pub(super) const OTHER_TASK_TITLE: &str = "PRIVATE_UNTRACKED_TASK_TITLE";
pub(super) const SCRIPTED_REPORT: &str = "PRIVATE_SCRIPTED_REPORT";
pub(super) const REVIEW_TITLE: &str = "PRIVATE_REVIEW_TITLE";
pub(super) const REVIEW_REPORT: &str = "PRIVATE_REVIEW_REPORT";
const REVIEW_TASK: &[u8] = b"Review this change at the named head";

thread_local! { pub(super) static WITH_REVIEW: Cell<bool> = const { Cell::new(false) }; }

pub(super) fn configure(value: &mut temper_playground::documents::Configuration) {
    if !WITH_REVIEW.with(Cell::get) {
        return;
    }
    value["world_settings"]["landing"]["deployment"] = serde_json::json!([{
        "pattern": {"segments": ["forge", "127.0.0.1:3000"], "last": "open", "value": "acme"},
        "enforce": true, "ci": false, "up_to_date": false,
        "gates": [{"number": 1, "blocking": true, "freshness": "clean"}], "approvals": []
    }]);
    // The review can overlap the conversation, the change, the producer and the turn.
    for (name, count) in
        [("tasks.tasks", 6), ("tasks.project_tasks", 6), ("tasks.tree_tasks", 8), ("people.inbox_tasks", 12)]
    {
        value.jig_over(name, count);
    }
}

pub(super) fn give_private_sources(given: &mut temper_service::stock::Given) {
    if !WITH_REVIEW.with(Cell::get) {
        return;
    }
    let temper_service::stock::Given::Own { jig, forge, .. } = given else { panic!("public forge defaults") };
    // Leave part of the owner's period for the unrelated conversation. Its first chat
    // needs the change's allowance and enough for its own turns.
    jig.templates[0].budget = jig_domain_people::TemplateBudget::AsAllowed { cap: Some(2 * fixtures::PRODUCER_SPEND) };
    // Use the world charter that the change is permitted to delegate to.
    let review = forge.iter_mut().find(|row| (row.kind, row.scope, row.number) == (3, 0, 2)).expect("review charter");
    review.value.value = temper_engine_domain_forge::settings::Value::Charter(u32::from(fixtures::AGENT_KIND));
}

pub(super) fn provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    let call = |name: &[u8], arguments: &[u8]| Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: 1,
    };
    let grant = |kind| serde_json::json!({"connector":fixtures::FORGE_CONNECTOR,"kind":kind,"segments":["forge","127.0.0.1:3000"],"terminal":"open","last":"acme"});
    // The same change delegates two reviews, its resolution, and its landing work.
    let delegated_tasks = if WITH_REVIEW.with(Cell::get) { 4 } else { fixtures::PRODUCER_TASKS };
    let delegate = serde_json::json!({"batch":[{
        "executor":{"kind":"procedure","connector":fixtures::FORGE_CONNECTOR,"code":fixtures::PROCEDURE_CODE},
        "tracked":{"priority":1},
        "spec":{"words":"@edit Public change title\nPRIVATE_SPEC_BODY","parameters":[
            {"name":fixtures::PARAMETER_NAMES[0],"kind":"resource","connector":fixtures::FORGE_CONNECTOR,"resource":fixtures::REPOSITORY_TAG},
            {"name":fixtures::PARAMETER_NAMES[1],"kind":"bytes","value":"main"},
            {"name":fixtures::PARAMETER_NAMES[2],"kind":"bytes","value":std::str::from_utf8(fixtures::MERGE_BRANCH).expect("branch")},
            {"name":10,"kind":"bytes","value":"PRIVATE_PARAMETER"}]},
        "contract":{"kind":"change","connector":fixtures::FORGE_CONNECTOR,"change_kind":fixtures::CHANGE_KIND,"words":fixtures::CHANGE_WORDS},
        "authority":{"tools":fixtures::PRODUCER_TOOLS,"grants":([3,4,11].map(grant)),"delegation":{"kinds":[{"kind":"agent","number":fixtures::AGENT_KIND}],"tasks":delegated_tasks,"depth":fixtures::PRODUCER_DEPTH},"budget":{"spend":fixtures::PRODUCER_SPEND},"notes":fixtures::PRODUCER_NOTES}
    }]});
    let delegate = serde_json::to_vec(&delegate).expect("delegate JSON");
    let finish = call(b"finish", br#"{"form":"change","text":"PRIVATE_RESULT","fields":{"title":"PRIVATE_PULL_TITLE","body":"PRIVATE_PULL_BODY"}}"#);
    let mut plan = call(b"delegate", &delegate);
    let mut lines = plan.lines.into_vec();
    lines.insert(0, Line::Text { text: CHAT_REPLY.into() });
    plan.lines = lines.into_boxed_slice();
    let review = serde_json::to_vec(&serde_json::json!({
        "form": "verdict", "label": "1", "text": format!("{REVIEW_TITLE}\n{REVIEW_REPORT}"),
        "fields": {}, "items": []
    }))
    .expect("review JSON");
    let report = serde_json::to_vec(&serde_json::json!({
        "form": "report", "text": SCRIPTED_REPORT, "fields": {}
    }))
    .expect("report JSON");
    fixtures::scripted_provider(Box::new([
        Script {
            cue: b"@plan".as_slice().into(),
            turns: Box::new([plan, call(b"wait", b"{}"), call(b"wait", b"{}"), call(b"wait", b"{}")]),
        },
        Script { cue: OTHER_TASK_TITLE.as_bytes().into(), turns: Box::new([call(b"finish", &report)]) },
        Script { cue: b"@edit".as_slice().into(), turns: Box::new([finish.clone()]) },
        Script { cue: REVIEW_TASK.into(), turns: Box::new([call(b"finish", &review), call(b"finish", &review)]) },
        Script {
            cue: b"Resolve the merge conflict".as_slice().into(),
            turns: Box::new([call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish]),
        },
    ]))
}
