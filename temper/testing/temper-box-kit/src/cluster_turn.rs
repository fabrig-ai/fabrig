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

/// The checked-in engine template with the story's listener and ready facts supplied directly.
#[must_use]
pub fn engine_document(bindings: &[(&str, &str)]) -> Value {
    support::configuration::engine_template(&fixture().join("engine").join(temper_config_shell::FILE), true, bindings)
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
    bind_addresses(cluster, addresses);
    // Source reads and preparation use the engine's git, found on PATH (config.md, 6.5).
    cluster.git_on_path("engine", root);
    for name in ["engine", "w1"] {
        cluster.prepare(name);
    }

    let engine_config = cluster.dir("engine").join("config");
    let engine_data = cluster.dir("engine").join("data");
    let engine = support::limit_settings::configuration(&engine_config);
    let agent = temper_agent_fixtures::at(&engine, &engine_data);
    let engine = compose_engine(engine, agent);
    support::write_private(&engine_config.join("landing-settings.json"), &engine.landing);
    engine.agents.lay(&engine_data);
    support::limit_settings::remember(&engine_config, &engine.configuration);
    cluster.bind("story.change_spend", &engine.change_spend().to_string());
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
    engine.configuration
}

/// Fix the cluster-turn listeners and peer captures before rendering any member.
pub fn bind_addresses(cluster: &mut Cluster, addresses: &[std::net::SocketAddr; 5]) {
    for (name, address) in ["clients", "workers", "webhooks", "forge", "llm"].iter().zip(addresses) {
        cluster.bind(&format!("story.{name}"), &address.to_string());
    }
    cluster.bind("proc.forge.ready.address", &addresses[3].to_string());
    cluster.bind("proc.llm.ready.address", &addresses[4].to_string());
}

/// The engine's configured figures and startup edits for this turn.
pub struct Engine {
    pub configuration: temper_playground::documents::Configuration,
    pub agents: temper_agent_fixtures::Setting,
    pub landing: Vec<u8>,
}
impl Engine {
    /// Two stock runs fund production and leave room after the model-priced reserve.
    #[must_use]
    pub fn change_spend(&self) -> u64 {
        let configured = temper_real_loop_world::settings::configuration(&self.configuration);
        let stock = temper_service::stock::defaults(&configured.llm, configured.limits.core.tasks.result_bytes);
        2 * stock.rules.maximum_run_spend
    }
}

/// Compose the edits both cluster boundaries lay over the rendered engine.
#[must_use]
pub fn compose_engine(mut engine: temper_playground::documents::Configuration, mut agent: Value) -> Engine {
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
    engine.jig_over("hub.event_bytes", prepare.result);
    engine.temper_over("checkout.message_bytes", payload);
    let turn = support::configuration::turn_bounds::turn_body(&agent);
    support::configuration::turn_bounds::apply(&mut engine, turn);
    let landing = serde_json::to_vec(&json!({"items":[
            {"setting":{"kind":"project-landing","project":1},"set":{"by":"turn","batch":1}},
            {"setting":{"kind":"project-gates","project":1},"set":{"repositories":[{"forge":1,"repository":1,"gates":[{"name":"gate","workflow":"gate.yml","ignored":[],"blocking":true,"base":true}],"tiers":[]}]}}
    ]})).expect("landing settings");
    let providers = temper_agent_fixtures::providers::take(&mut engine).expect("fixture providers");
    let agents = temper_agent_fixtures::compose_setting(&mut engine, &agent, &providers);
    Engine { configuration: engine, agents, landing }
}

/// Leave seven forty-eighths of the period for landing tasks, not the owner's pool.
/// Copy current whole settings so the fixture pins no stock charter document.
#[must_use]
pub fn settings(root: &Path, calls: &[support::Ended]) -> Call {
    let providers = temper_agent_fixtures::providers::read(&root.join("engine/data"));
    settings_with(root, calls, &providers, support::write_private)
}

/// Compose the settings FILE from the captured replies and supplied provider copy.
pub fn settings_with(
    root: &Path,
    calls: &[support::Ended],
    providers: &Value,
    write: impl FnOnce(&Path, &[u8]),
) -> Call {
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
            support::provider_setup::use_provider(&mut value, 1);
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
    items.extend(support::provider_setup::items(providers));
    let path = root.join("turn-settings.json");
    write(&path, &serde_json::to_vec(&json!({"items":items})).expect("settings JSON"));
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
