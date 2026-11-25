//! Shared documents for the cluster restart and unit-stop stories (testing/process-tier.md).
use crate::{
    self as support,
    cluster::Cluster,
    ctl::{self, Call},
};
use serde_json::{Value, json};
use std::{fs, path::Path};

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
    let engine = compose_engine(engine, &agent);
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
    change_spend: u64,
}
impl Engine {
    /// Two stock runs fund production and leave room for its scripted turns after the model-priced reserve.
    #[must_use]
    pub fn change_spend(&self) -> u64 {
        self.change_spend
    }
}

/// Compose the edits both cluster boundaries lay over the rendered engine.
#[must_use]
pub fn compose_engine(mut engine: temper_playground::documents::Configuration, agent: &Value) -> Engine {
    engine.jig_over("people.words", 512);
    let turn = support::configuration::turn_bounds::turn_body(agent);
    support::configuration::turn_bounds::apply(&mut engine, turn);
    let providers = temper_agent_fixtures::providers::take(&mut engine).expect("fixture providers");
    let agents = temper_agent_fixtures::compose_setting(&mut engine, agent, &providers);
    support::configuration::cluster_agent_limits(&mut engine, &agents.figures);
    let configured = temper_real_loop_world::settings::configuration(&engine);
    let stock = temper_service::stock::defaults(&configured.llm, configured.limits.core.tasks.result_bytes);
    // The unit-stop referee reads the parent chat with its whole instructions.
    let chat = stock.charters.iter().find(|charter| charter.number == 1).expect("stock chat charter");
    engine.jig_over("documents.text_bytes", chat.policy.instructions.len());
    Engine { configuration: engine, agents, change_spend: 2 * stock.rules.maximum_run_spend }
}

/// Keep the current charters, overriding their provider, documents and hosts.
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
    for call in &calls[2..4] {
        let packet = ctl::json(call);
        let mut value = packet["value"].clone();
        support::provider_setup::use_provider(&mut value, 1);
        value["documents"] = json!([]);
        value["hosts"] = "workers".into();
        items.push(json!({"setting":packet["setting"],"set":value}));
    }
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
