//! Engine and worker documents (temper/docs/design/playground.md, 3).
use skein_lib::{List, Secret};
const DIGEST: &[u8] = b"6fb46f7a92742970166379ed5195e79c4493a7cc5664280c039cfd4095ba5faf";
const ENGINE: &[u8] = br#"{
"llm": {
"providers":[{"name":"fake-provider","number":1,"models":["fake"],"sources":{"keys":true}}],
"endpoints": [{
"name": "fake",
"number": 1,
"dialect": 3,
"provider": "fake-provider",
"address": {"socket": "127.0.0.1:7101"},
"transport": "plaintext",
"headers": [],
"connect_ms": 10000,
"handshake_ms": 10000
}],
"models": [{
"name": "fake",
"endpoint": "fake",
"window": 1048576,
"output": 512,
"prices": {"input": 1, "cached": 1, "output": 1, "unit": 1000000}
}]
},
"hosts": {
"listener": [{"socket": "127.0.0.1:7200"}]
},
"charters": [{
"number": 1, "instructions": "charters/chat.txt", "model": "fake", "alternatives": [],
"hosts": "workers", "waiting_ms": 10000, "resume": true,
"turns": 4, "time_ms": 30000, "reserve_turns": 1, "reserve_time_ms": 5000, "spend": 20,
"inspect": false, "modify": false, "shell": false, "agents": false, "call_timeout_ms": 10000
}],
"templates": [
{"number": 1, "name": "chat", "executor": {"charter": 1}, "contract": {"report": {"words": 32}}, "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none", "tracked": null, "conversation": true, "listed": false, "retention": "kept", "label": 1, "grace_ms": 5000, "llm": null},
{"number": 2, "name": "goal", "executor": "chosen", "contract": {"report": {"words": 32}}, "shallower": 1, "budget": "given", "deadline": "given", "tracked": {"priority": 1}, "conversation": false, "listed": true, "retention": "kept", "label": 2, "grace_ms": null, "llm": null}
],
"clients": {
},
"owners": [{"provider": 1, "subject": "owner-a"}],
"rules": {
"ceiling": {"tools": 0, "grants": [],
"delegation": {"kinds": [{"kind": "charter", "number": 1}], "tasks": 8, "depth": 8},
"budget": {"spend": 100, "deadline_unix_seconds": null}, "notes": 0, "note_resources": []},
"period_spend": 100, "minimum_run_spend": 1, "maximum_run_spend": 100,
"implies": [], "requirements": []
}
}"#;

pub(super) const WORKER_DOCUMENT: &[u8] = br#"{
"workers":[{
"engine":{"socket":"127.0.0.1:7200"},"name":"worker-a","secret":"worker.secret","slots":1,
"agent":{"program":"agent-program","configuration":"agent.json","arguments":["agent"]},
"limits":{"overrides":{"hub.slots":1,"hub.run_calls":1,"hub.inbox":8,"hub.held":8,"tasks.run_inbox":8,"tasks.result_bytes":512,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}}
}],"application":{}}
"#;

pub(super) const POLICIES: &[u8] = br#"[
  {
    "number": 1,
    "roles": [
      {
        "connector": 1,
        "permission": 1,
        "role": 0
      }
    ],
    "policy": {
      "templates": [],
      "escalation_role": 0,
      "period_spend": 100,
      "projections": [],
      "requirements": [],
      "ceiling": {
        "tools": 0,
        "grants": [],
        "delegation": {
          "kinds": [
            {
              "kind": "charter",
              "number": 1
            }
          ],
          "tasks": 8,
          "depth": 8
        },
        "budget": {
          "spend": 100,
          "deadline_unix_seconds": null
        },
        "notes": 0,
        "note_resources": []
      },
      "roles": [
        {
          "number": 0,
          "period_spend": 100,
          "requests": 511,
          "decides": 31,
          "authority": {
            "tools": 0,
            "grants": [],
            "delegation": {
              "kinds": [
                {
                  "kind": "charter",
                  "number": 1
                }
              ],
              "tasks": 8,
              "depth": 8
            },
            "budget": {
              "spend": 100,
              "deadline_unix_seconds": null
            },
            "notes": 0,
            "note_resources": []
          }
        }
      ]
    }
  }
]"#;

pub(super) fn engine_document() -> super::Configuration {
    let document: serde_json::Value = serde_json::from_slice(&document(ENGINE)).expect("engine JSON");
    let mut world = super::Configuration::stock(document);
    world.limits.root.transcript_memory = 1_152_921_504_606_846_976;
    for (name, value) in [
        ("tasks.inbox_bytes", 4096_u64),
        ("tasks.run_inbox", 8_u64),
        ("tasks.result_bytes", 32_u64),
        ("people.amendment_bytes", 64_u64),
        ("core.policy_bytes", 512_u64),
        ("tasks.spec_bytes", 128_u64),
        ("tasks.message_bytes", 512_u64),
        ("tasks.hold_bytes", 64_u64),
        ("tasks.authority_bytes", 64_u64),
        ("people.words", 512_u64),
        ("core.call_answer_bytes", 2048_u64),
        ("notes.recalled", 1_u64),
        ("people.operations", 2_u64),
        ("tasks.tasks", 2_u64),
        ("tasks.exemptions", 2_u64),
        ("tasks.funders", 2_u64),
        ("tasks.project_tasks", 1_u64),
        ("tasks.tree_tasks", 1_u64),
        ("tasks.project_goals", 1_u64),
        ("tasks.batch", 1_u64),
        ("tasks.dependencies", 1_u64),
        ("tasks.delegates", 1_u64),
        ("tasks.charters", 1_u64),
        ("people.people", 1_u64),
        ("people.projects", 2_u64),
        ("people.sign_ins", 2_u64),
        ("people.holdings", 1_u64),
        ("people.inbox_tasks", 4_u64),
        ("people.pending", 1_u64),
        ("people.waiters", 1_u64),
        ("fleet.attempts", 1_u64),
        ("fleet.calls", 1_u64),
        ("fleet.turns", 8_u64),
        ("brief.briefs", 1_u64),
        ("brief.sections", 8_u64),
        ("tasks.charter_documents", 0_u64),
        ("notes.entries_per_scope", 1_u64),
        ("notes.party_entries", 1_u64),
        ("views.runs", 1_u64),
        ("core.load_slots", 1_u64),
        ("core.load_bytes", 8192_u64),
        ("core.resume_bytes", 4096_u64),
        ("core.run_bytes", 4096_u64),
        ("views.text_bytes", 1024_u64),
        ("core.templates", 2_u64),
        ("documents.rows", 1_u64),
        ("documents.lines", 1_u64),
        ("documents.chips", 1_u64),
        ("documents.text_bytes", 128_u64),
    ] {
        world.jig_over(name, value);
    }
    world
}
fn document(engine: &[u8]) -> Vec<u8> {
    let mut document = b"{\"engine\":".to_vec();
    document.extend_from_slice(engine);
    document.extend_from_slice(
        br#", "application":{"server":{"listener":{"socket":"127.0.0.1:7100"}},"credential":"app.secret","connectors":[
        {"prefix":["one"],"resources":[]},{"prefix":["two"],"resources":[]}]},
        "service":{"components":["accounts","workers","clients"]}}"#,
    );
    document
}
pub(super) fn files(worker: bool) -> temper_service::settings::Files {
    let inputs: &[(&[u8], &[u8])] = if worker {
        &[(b"worker.secret", b"worker-secret"), (b"agent.json", b"{}")]
    } else {
        &[
            (b"charters/chat.txt", b"Reply to the person."),
            (b"secrets/worker-a.digest", DIGEST),
            (b"app.secret", b"application-token"),
        ]
    };
    let mut entries = List::with_capacity(u32::try_from(inputs.len()).expect("fixture files"));
    for (name, bytes) in inputs {
        entries.push(((*name).into(), Secret::new((*bytes).into()))).expect("fixture files");
    }
    temper_service::settings::Files { entries }
}
pub(super) fn endpoints() -> smith_protocol_channel::Endpoints {
    let mut entries = List::with_capacity(1);
    entries
        .push(smith_protocol_channel::Endpoint { name: b"fake".as_slice().into(), number: 0, dialect: 3 })
        .expect("one endpoint");
    smith_protocol_channel::Endpoints::new(entries)
}
