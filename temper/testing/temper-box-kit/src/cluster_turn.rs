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

    let mut engine = support::limit_settings::configuration(&cluster.dir("engine").join("config"));
    let agent = temper_agent_fixtures::at(&engine, &cluster.dir("engine").join("data"));
    engine.jig_over("people.words", 512);
    let turn = support::configuration::turn_bounds::turn_body(&agent);
    support::configuration::turn_bounds::apply(&mut engine, turn);
    temper_agent_fixtures::apply(&mut engine, &cluster.dir("engine").join("data"), &agent);
    support::limit_settings::remember(&cluster.dir("engine").join("config"), &engine);
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

/// Keep the current charters, overriding their provider, documents and hosts.
#[must_use]
pub fn settings(root: &Path, calls: &[support::Ended]) -> Call {
    let mut items = Vec::new();
    for call in &calls[2..4] {
        let packet = ctl::json(call);
        let mut value = packet["value"].clone();
        support::provider_setup::use_provider(&mut value, 1);
        value["documents"] = json!([]);
        value["hosts"] = "workers".into();
        items.push(json!({"setting":packet["setting"],"set":value}));
    }
    let providers = temper_agent_fixtures::providers::read(&root.join("engine/data"));
    items.extend(support::provider_setup::items(&providers));
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
