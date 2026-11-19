//! Fixture providers committed through ctl (protocol/configuration/providers.md, 3).
use std::path::Path;

/// Settings edits for the fixture's providers, excluding the box's constant provider.
/// Unmentioned providers keep their stock settings and model references.
#[must_use]
pub fn items(wanted: &serde_json::Value) -> Vec<serde_json::Value> {
    let providers =
        temper_agent_fixtures::providers::without_scripted(&temper_agent_fixtures::providers::values(wanted));
    let rendered = temper_service_shell::providers::render(&providers);
    let rendered: serde_json::Value = serde_json::from_slice(&rendered).expect("provider list");
    rendered["providers"]
        .as_array()
        .expect("providers")
        .iter()
        .map(|provider| serde_json::json!({"setting":{"kind":"provider","number":provider["number"]},"set":provider}))
        .collect()
}

/// Bind a single-provider story to a model the stock charter already offers.
pub fn use_provider(charter: &mut serde_json::Value, provider: u16) {
    let model = std::iter::once(&charter["model"])
        .chain(charter["alternatives"].as_array().expect("charter alternatives"))
        .find(|model| model["provider"] == provider)
        .expect("stock model for the fixture provider")
        .clone();
    charter["model"] = model;
    charter["alternatives"] = serde_json::json!([]);
}

/// Commit a fake box's providers and agent figures before its first task-bearing start.
/// The no-task sitting changes settings through ctl, never the store's files.
pub(super) fn commit(directory: &Path, flags: &[String]) {
    use super::ctl::{Call, Step};
    let config = flags.windows(2).find(|pair| pair[0] == "--config").expect("explicit config directory");
    let data = flags.windows(2).find(|pair| pair[0] == "--data").expect("explicit data directory");
    let wanted = temper_agent_fixtures::providers::read(Path::new(&data[1]));
    let agents =
        std::fs::read(Path::new(&data[1]).join("agents.setting")).ok().map(|bytes| super::agents_value(&bytes));
    let file = Path::new(&config[1]).join("fixture-providers.json");
    if file.exists() {
        return;
    }
    let client = directory.join("provider-setup-client");
    temper_shell::directories::make_private(&client, skein_shell::effective_user()).expect("provider setup client");
    let outcome = super::boxed::run_ctl(
        directory,
        flags.to_vec(),
        &client,
        vec![
            Step::Run(Call::sign_in()),
            Step::Run(Call::new(&["settings", "show", "--json"])),
            Step::Then(Box::new(move |calls| {
                let current = calls.last().expect("settings inventory");
                let inventory = super::ctl::json(current);
                let mut items = items(&wanted);
                if let Some(agents) = agents {
                    items.push(serde_json::json!({"setting":{"kind":"agents"},"set":agents}));
                }
                super::write_private(
                    &file,
                    &serde_json::to_vec(&serde_json::json!({"items":items})).expect("provider edits"),
                );
                super::ctl::settings_change(current, &file, &format!("fixture-providers-{}", inventory["newest"]))
            })),
            Step::Run(Call::new(&["sign-out"])),
        ],
        super::boxed::BOX_DEADLINE,
        |_| (),
    );
    for call in &outcome.calls {
        assert_eq!(
            call.exit,
            Some(skein_io::kernel::Exit::Code(0)),
            "provider setup: {}",
            String::from_utf8_lossy(&call.stderr)
        );
    }
    assert_eq!(outcome.calls.len(), 4, "provider setup committed and signed out");
    assert_eq!(outcome.exit, Some(skein_io::kernel::Exit::Code(0)), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(outcome.left, Some(Vec::new()), "provider setup drained");
}
