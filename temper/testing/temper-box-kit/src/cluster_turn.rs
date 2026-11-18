//! Shared documents for the two process boundaries (testing/process-tier.md, landing by turn on the fake forge).
use crate::{
    self as support,
    cluster::Cluster,
    ctl::{self, Call},
};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub const CI_MS: u32 = 10_000;
pub const CONTENT: &[u8] = b"Saved work\nDelivered work\n";

#[must_use]
pub fn fixture() -> std::path::PathBuf {
    skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/cluster-turn")
}

#[must_use]
pub fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("story document")).expect("story JSON")
}

pub fn write(path: &Path, value: &Value) {
    support::write_private(path, &serde_json::to_vec(value).expect("story JSON"));
}

/// Render once, retaining these documents across both kinds of restart.
pub fn prepare(
    cluster: &mut Cluster,
    root: &Path,
    addresses: &[std::net::SocketAddr; 5],
) -> temper_playground::documents::Configuration {
    for (name, address) in ["clients", "workers", "webhooks", "forge", "llm"].iter().zip(addresses) {
        cluster.bind(&format!("story.{name}"), &address.to_string());
    }
    cluster.bind("proc.forge.ready.address", &addresses[3].to_string());
    cluster.bind("proc.llm.ready.address", &addresses[4].to_string());
    // Source reads and preparation use the engine's git, found on PATH (config.md, 6.5).
    cluster.git_on_path("engine", root);
    for name in ["engine", "w1"] {
        cluster.prepare(name);
    }
    let engine_path = cluster.dir("engine").join("config/temper.json");
    let worker_path = cluster.dir("w1").join("config/temper.json");

    let mut engine = support::limit_settings::configuration(&cluster.dir("engine").join("config"));
    let mut worker = temper_playground::documents::Configuration::from_jig(read(&worker_path));
    let mut agent = temper_agent_fixtures::read(&cluster.dir("engine").join("config"));
    // domain/worker/jobs.md, kind 1: the engine declares preparation only when
    // its spec and result fit. The worker and the agent must carry that result.
    let prepare = temper_service::limits::prepare_bound();
    for (name, bytes) in [
        ("tasks.spec_bytes", prepare.spec),
        ("tasks.message_bytes", prepare.result),
        ("tasks.result_bytes", prepare.result),
    ] {
        engine.jig_over(name, bytes);
    }
    engine.jig_over("people.words", 512);
    engine.jig_over("tasks.run_inbox", 8);
    engine.jig_over("tasks.inbox_bytes", prepare.result * 8);
    let payload = prepare.result + 2; // the result and its JSON string quotes
    agent["profile"]["declared"]["tool_payload"] = payload.into();
    engine.jig_over("hub.outcome_bytes", payload);
    engine.temper_over("forge.client.op_bytes", prepare.result);
    engine.temper_over("tool_payload", payload);
    worker.jig_over("tasks.result_bytes", prepare.result);
    worker.jig_over("hub.outcome_bytes", payload);
    worker.jig_over("hub.event_bytes", prepare.result);
    worker.temper_over("checkout.message_bytes", payload);
    let turn = support::configuration::turn_bounds::turn_body(&agent);
    support::configuration::turn_bounds::apply(&mut engine, turn);
    support::configuration::turn_bounds::apply(&mut worker, turn);
    write(
        &cluster.dir("engine").join("config/landing-settings.json"),
        &json!({"items":[
            {"setting":{"kind":"project-landing","project":1},"set":{"by":"turn","batch":1}},
            {"setting":{"kind":"project-gates","project":1},"set":{"repositories":[{"forge":1,"repository":1,"gates":[{"name":"gate","workflow":"gate.yml","ignored":[],"blocking":true,"base":true}],"tiers":[]}]}}
        ]}),
    );
    write(&engine_path, &engine);
    // Stock charters need a model-priced reserve, not the small cluster fixture's spend.
    // Two runs fund production and leave room for its scripted turns after the reserve.
    let configured = temper_real_loop_world::settings::configuration(&engine);
    let stock = temper_service::stock::defaults(&configured.llm, configured.limits.core.tasks.result_bytes);
    cluster.bind("story.change_spend", &(2 * stock.rules.maximum_run_spend).to_string());
    for name in ["forge", "llm"] {
        cluster.prepare(name);
    }
    // The operator installs the LLM's trust root before the engine's first start.
    support::write_private(&cluster.dir("llm").join("root.der"), skein_tls_kit::pki::ROOT);
    // The person uses the fake's fixed fixture identity, through a private
    // header file rather than a credential in curl's argument vector.
    let token = temper_fake::credential::credential(17, temper_fake::credential::ADMIN_TOKEN);
    support::write_private(
        &root.join("person.header"),
        format!("Authorization: token {}\n", std::str::from_utf8(&token).expect("fixture token")).as_bytes(),
    );
    engine
}

/// Leave seven forty-eighths of the period for landing tasks, not the owner's pool.
/// Copy current whole settings so the fixture pins no stock charter document.
#[must_use]
pub fn settings(root: &Path, calls: &[support::Ended]) -> Call {
    let mut items = Vec::new();
    for call in &calls[2..5] {
        let packet = ctl::json(call);
        let mut value = packet["value"].clone();
        if packet["setting"]["kind"] == "policy" {
            let period = value["period_spend"].as_u64().expect("policy period");
            let owner = value["roles"]
                .as_array_mut()
                .expect("roles")
                .iter_mut()
                .find(|role| role["number"] == 0)
                .expect("owner role");
            owner["period_spend"] = (period / 48 * 41).into();
        } else {
            value["documents"] = json!([]);
            value["hosts"] = "workers".into();
        }
        items.push(json!({"setting":packet["setting"],"set":value}));
    }
    // Turn's test gate replaces the stock review; there is no second judgement
    // path in these stories. The branch pattern covers this story's repository.
    items.push(json!({"setting":{"kind":"landing-rule","project":1,"number":1},"set":{
        "pattern":{"segments":["6272616e6368"],"last":{"open":""}},
        "enforce":true,"ci":true,"up_to_date":true,"gates":[],"approvals":[]}}));
    let path = root.join("turn-settings.json");
    write(&path, &json!({"items":items}));
    ctl::settings_change(&calls[1], &path, "turn-settings")
}

/// The first push is save's. Every later push waits while delivery.hold exists.
/// Only this worker invokes the wrapper; setup and the fake forge use fake git directly.
/// Strip shell-added exports so fake git receives git.md section 5's whole environment.
#[must_use]
pub fn wrapper(root: &Path, git: &Path) -> String {
    format!(
        r#"#!/bin/sh
set -eu
for arg do
  if [ "$arg" = push ] && [ -f '{root}/delivery.hold' ]; then
    if mkdir '{root}/saved-push' 2>/dev/null; then
      :
    else
      : > '{root}/delivery.entered'
      while [ -f '{root}/delivery.hold' ]; do sleep 0.02; done
    fi
  fi
done
exec /usr/bin/env -u PWD -u SHLVL -u _ '{git}' "$@"
"#,
        root = root.display(),
        git = git.display()
    )
}

/// A proving panel counts only once the dispatched run is known. Reading later
/// is a finding, never a successful restart outside the intended cut.
pub fn proof_cut(panel: &Value) -> Result<Option<u64>, String> {
    match panel["phase"].as_str().ok_or("landing phase")? {
        "waiting" | "preparing" => Ok(None),
        "proving" => {
            let proof = &panel["members"][0]["gates"][0]["proof"];
            if proof.get("passed").is_some() || proof.get("failed").is_some() {
                return Err(format!("missed proving before kill: pace.ci_ms={CI_MS}; {panel}"));
            }
            Ok(proof["dispatched"]["run"].as_u64())
        }
        phase => Err(format!("read landing past proving before kill ({phase}): pace.ci_ms={CI_MS}; {panel}")),
    }
}
