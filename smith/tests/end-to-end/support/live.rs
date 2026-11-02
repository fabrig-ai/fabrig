//! Live account setup and command policy (smith/docs/design/testing.md, section 2.3).
//! Keeps only the guarded live root; smith owns all credential reads and writes.

use std::path::{Path, PathBuf};

pub const CHATGPT: &str =
    "no record in `$SMITH_LIVE/tokens/chatgpt`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login chatgpt`";
pub const CLAUDE: &str =
    "no record in `$SMITH_LIVE/programs/claude`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login claude`";
pub const KEY: &str = "`ANTHROPIC_API_KEY` is not set";

/// The refresh story's baseline deployment lead.
pub const REFRESH_LEAD_MS: u64 = 34_560_000;

/// The suite's durable user layer; each story requests only its own accounts.
pub struct Environment {
    pub root: PathBuf,
    pub refresh_lead: Option<u64>,
}

impl Environment {
    #[must_use]
    pub fn load(accounts: &[&str]) -> Self {
        let root = root(std::env::var_os("SMITH_LIVE").as_deref()).unwrap_or_else(|message| panic!("{message}"));
        let root = smith_bench::guard::write_path(&root).expect("guarded $SMITH_LIVE");
        let environment = Self { root, refresh_lead: None };
        let _configuration = environment.configuration();
        for account in accounts {
            let key = *account == "anthropic-key"
                && std::env::var_os("ANTHROPIC_API_KEY").is_some_and(|value| !value.is_empty());
            setup(&environment.root, account, key).unwrap_or_else(|message| panic!("{message}"));
        }
        environment
    }

    /// Read generation through the product's private record reader, without exposing a token.
    #[must_use]
    pub fn generation(&self) -> u64 {
        let _configuration = self.configuration();
        smith_local_shell::credentials::record_generation(&self.root.join("tokens/chatgpt"))
            .expect("a readable ChatGPT record")
            .expect("ChatGPT record exists")
    }

    /// Recheck every live-layer destination before a child can use it.
    #[must_use]
    pub fn configuration(&self) -> PathBuf {
        let mut configuration = self.root.join("config");
        for relative in [
            "config",
            "config/smith",
            "config/smith/config.toml",
            "host-id",
            "tokens",
            "tokens/chatgpt",
            "programs/claude",
        ] {
            let path = smith_bench::guard::write_path(&self.root.join(relative))
                .unwrap_or_else(|reason| panic!("$SMITH_LIVE/{relative}: {reason}"));
            if relative == "config" {
                configuration = path;
            }
        }
        configuration
    }
}

pub(super) fn root(value: Option<&std::ffi::OsStr>) -> Result<PathBuf, &'static str> {
    value.filter(|value| !value.is_empty()).map(PathBuf::from).ok_or("$SMITH_LIVE unset")
}

pub(super) fn setup(root: &Path, account: &str, key: bool) -> Result<(), &'static str> {
    let (path, message) = match account {
        "chatgpt" => ("tokens/chatgpt", CHATGPT),
        "claude" => ("programs/claude", CLAUDE),
        "anthropic-key" => return if key { Ok(()) } else { Err(KEY) },
        _ => panic!("unknown live account"),
    };
    std::fs::symlink_metadata(root.join(path)).map(|_| ()).map_err(|_| message)
}

/// Insert the refresh lead setting before the trailing prompt argument `-`.
pub fn with_refresh_lead(arguments: &mut Vec<std::ffi::OsString>, lead_ms: u64) {
    let at = arguments.len().checked_sub(1).expect("prompt argument");
    arguments.splice(at..at, ["-c".into(), format!("deployment.refresh_lead={lead_ms}").into()]);
}

/// Public command settings shared by live stories and their offline controls.
#[must_use]
pub fn arguments(account: &str, directory: &Path, change: bool, push: bool) -> Vec<std::ffi::OsString> {
    let mut arguments = vec![
        "exec".into(),
        "--json".into(),
        "-C".into(),
        directory.join("repo").into(),
        "--chat".into(),
        "chat".into(),
        "-c".into(),
        format!("endpoints.{}.account=\"{account}\"", if account == "chatgpt" { "openai" } else { "anthropic" }).into(),
        "-m".into(),
        if account == "chatgpt" { "openai/gpt-6-luna".into() } else { "anthropic/claude-haiku-5-5".into() },
    ];
    if account == "chatgpt" {
        arguments.extend(["--effort".into(), "low".into()]);
    }
    if change {
        arguments.push("--deliver".into());
    }
    let overrides = serde_json::json!({
        "instructions":"Follow the requested outcome exactly. Use tools and call finish when done.",
        "budget.turns":512, "budget.spend":25, "budget.seconds":3600,
        "deployment.memory":2_281_701_376_u64, "sub_agents":false,
        "workspace.directories":[{"name":"repo","path":directory.join("repo"),"writable":true,"git":change}],
        "workspace.conventions.guide":".smith-test/guide", "workspace.conventions.checks":".smith-test/check",
        "workspace.contract.form":if change {"change"} else {"report"},
        "workspace.contract.checks_must_pass":change,
        "workspace.contract.fields":if change {serde_json::json!([{"name":"title","max":256},{"name":"body","max":4096}])} else {serde_json::json!([])},
        "workspace.contract.max":4096,
        "workspace.delivery.pushes":push,
        "trace.capture":"everything",
        "environment.inherit":[], "environment.set.PATH":std::env::var("PATH").expect("PATH"),
        "environment.set.SMITH_TEST_ENVIRONMENT":"configured",
        "environment.set.HOME":directory.join("home"),
        "environment.set.XDG_STATE_HOME":directory.join("state"),
    });
    for (name, value) in overrides.as_object().expect("overrides") {
        let value: toml::Value = serde_json::from_value(value.clone()).expect("public TOML value");
        arguments.extend(["-c".into(), format!("{name}={value}").into()]);
    }
    if account == "claude" {
        arguments.extend(["-c".into(), "endpoints.anthropic.program=\"claude_code\"".into()]);
    }
    arguments.push("-".into());
    arguments
}
