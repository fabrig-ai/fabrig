//! Scripted acts and the checks after the new build opens.
use super::{STATE,golden,fixtures,stories,referee,CoreRecord,Record};
pub(super) fn provider() -> skein_fake_peers::llm::Peer {
    let mut scripts = golden::llm_scripts();
    let parent = scripts["asked"].as_array_mut().expect("asked").iter_mut().find(|s| s["cue"] == "Golden coding").expect("coding parent");
    let delegate = &mut parent["turns"][0]["lines"][1]["arguments"]["batch"][0];
    delegate["executor"] = serde_json::json!({"kind":"agent", "charter":1});
    delegate["contract"] = serde_json::json!({"kind":"report", "words":128});
    delegate["spec"]["parameters"] = serde_json::json!([]);
    delegate["authority"]["tools"] = 579.into();
    delegate["authority"]["grants"].as_array_mut().expect("grants").push(serde_json::json!({"connector":1,"kind":8,"segments":["forge"],"terminal":"open","last":""}));
    let coding = scripts["cued"].as_array_mut().expect("cued").iter_mut().find(|s| s["cue"] == "## Task\n\nGolden coding in flight").expect("coding script");
    coding["turns"][0]["lines"][0] = serde_json::json!({"call":"read_task", "arguments":{"task":5,"field":"specification"}});
    coding["turns"][0]["lines"].as_array_mut().expect("lines").insert(0, serde_json::json!({"text":"upgrade-tail-marker"}));
    coding["turns"].as_array_mut().expect("coding turns").insert(1, serde_json::json!({"lines":[{"call":"effect_forge", "arguments":{"repository":{"forge":1,"repository":1},"resource":{"kind":"repository"},"write":{"kind":"create_issue","key":"upgrade-write","title":"Upgrade once","body":"kept across starts"}}}],"finish":"tool_calls","tokens":7}));
    if STATE.with(|s| s.borrow().upgrade) {
        coding["turns"] = serde_json::json!([{"lines":[{"call":"finish","arguments":{"form":"report","text":"Fresh after upgrade","fields":{}}}],"finish":"tool_calls","tokens":1}]);
    }
    if STATE.with(|s| s.borrow().upgrade) {
        scripts["asked"][0]["turns"] = serde_json::json!([{"lines":[{"text":"Paused chat resumed"},{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":2}]);
    }
    let document = temper_fake::llm::Document::decode(&serde_json::to_vec(&scripts).expect("scripts")).expect("LLM scripts");
    temper_fake::llm::peer(temper_fake::llm::configuration(&document, temper_playground::documents::KEY, temper_playground::documents::seeds(fixtures::SEED).llm)).expect("peer")
}

pub(super) fn forge_peer(transport: &temper_fake_git_hosted::Typed) -> skein_fake_peers::forge::Peer {
    let document = temper_fake::forge::Document::decode(temper_playground::stories::A.forge).expect("forge document");
    let seed = temper_playground::documents::seeds(fixtures::SEED).forge;
    let (_, mut table) = document.seed(seed);
    let table = STATE.with(|s| {
        let s = s.borrow();
        let mut tokens = skein_lib::List::with_capacity(table.tokens.len() + u32::try_from(s.tokens.len()).expect("tokens"));
        for token in table.tokens.into_boxed() { tokens.push(token).expect("token room"); }
        for token in &s.tokens { tokens.push(token.clone()).expect("token room"); }
        table.tokens = tokens;
        let mut users = skein_lib::List::with_capacity(table.users.len() + u32::try_from(s.users.len()).expect("users"));
        for user in table.users.into_boxed() { users.push(user).expect("user room"); }
        for user in &s.users { users.push(user.clone()).expect("user room"); }
        table.users = users;
        table
    });
    let mut peer = skein_fake_peers::forge::Peer::new(document.listen(), temper_fake::limits::peer(), table, temper_fake::forge::git_limits(), transport.forge.clone(), seed).expect("forge peer").without_records();
    let token = temper_fake::credential::credential(seed, temper_fake::credential::ADMIN_TOKEN);
    temper_fake::forge::setup(peer.setup_mut(), &token);
    peer
}

pub(super) fn mutations(ledger: &[skein_fake_forge_domain::Observation]) -> Vec<skein_fake_forge_domain::Observation> {
    ledger.iter().filter(|o| !matches!(o,skein_fake_forge_domain::Observation::Refused {..} | skein_fake_forge_domain::Observation::Rejected {..})).cloned().collect()
}

pub(super) fn continued(store: &referee::Store) {
    let (task, old_attempt) = STATE.with(|s| (s.borrow().task,s.borrow().attempt));
    let start = store.agent_records.iter().find_map(|row| match row {
        referee::AgentRecord::Start {task:t,attempt,input:Some(input),..} if *t == task && *attempt > old_attempt => Some(input),
        _ => None,
    }).expect("mid-run task gets a new attempt");
    assert!(start.transcript().is_empty(),"fresh attempt, not the closed conversation");
    let charter = smith_charter::Charter::decode(&smith_charter::CEILINGS,&mut skein_lib::Reader::new(start.charter())).expect("charter");
    assert!(charter.brief().as_slice().iter().any(|section| skein_lib::bytes::find(section.body(), b"upgrade-tail-marker").is_some()),"fresh brief carries the closed window's tail");
    assert!(!store.rows.values().any(|row| matches!(row, temper_engine_domain::Record::Forge {row} if matches!(row.as_ref(),temper_engine_domain_forge::Stored::Flight {task:t,..} if *t == task))),"crossed claim's forge write settled");
    let new = store.rows.values().find_map(|row| match row {
        temper_engine_domain::Record::Core(Record::Tasks(jig_domain_tasks::Stored::Live(task) | jig_domain_tasks::Stored::Ended(task))) if task.spec.words.as_ref() == b"After upgrade" => Some(task.number),
        _ => None,
    }).expect("new chat allocated");
    STATE.with(|s| {
        let s = s.borrow();
        let maps = s.before.as_ref().expect("before");
        let key = jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment);
        let encoded = jig_protocol_store::keys::encode(&key,64).expect("header key");
        let Record::Core(CoreRecord::Deployment(header)) = jig_protocol_store::codec::decode(&key,maps.store.get(encoded.as_ref()).expect("header"),&stories::playground::codec(),fixtures::store_names()).expect("header decodes") else {panic!("header")};
        assert!(new > header.tasks,"new task above every kept task");
        assert!(start.activation() > header.runs,"new attempt above every kept attempt");
        let resumed=store.rows.values().find_map(|r| match r {
            temper_engine_domain::Record::Core(Record::Core(CoreRecord::Turn(turn))) if turn.task == 1 && skein_lib::bytes::find(&turn.transcript,b"Paused chat resumed").is_some() => Some(turn),
            _ => None,
        }).expect("paused chat's resumed turn");
        assert!(resumed.read.is_some_and(|number|number > header.messages),"new message above every kept message");
    });
}
