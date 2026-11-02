//! Live account setup and command policy (smith/docs/design/testing.md, section 2.3).
//! Keeps only the guarded live root; smith owns all credential reads and writes.

use std::path::{Path, PathBuf};

pub const CHATGPT: &str =
    "no record in `$SMITH_LIVE/tokens/chatgpt`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login chatgpt`";
pub const CLAUDE: &str =
    "no record in `$SMITH_LIVE/programs/claude`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login claude`";
pub const KEY: &str = "`ANTHROPIC_API_KEY` is not set";

/// The refresh story's lead, in the settings' seconds: 400 days, past any access token's life.
pub const REFRESH_LEAD_SECONDS: u64 = 400 * 86_400;

/// The live check's hard bound, in seconds.
pub const CHECK_SECONDS: u64 = 30;

/// The live exec's hard bound, in seconds.
pub const EXEC_SECONDS: u64 = 260;

/// How far ahead an earliest refresh time must be for the refresh story to expect its record
/// kept: longer than its check and exec bounds together (testing.md, 2.3).
pub const EARLIEST_MARGIN_SECONDS: u64 = 300;

/// A record without a refresh token needs a new sign-in before the refresh story.
pub const NOT_REFRESHABLE: &str = "the record in `$SMITH_LIVE/tokens/chatgpt` holds no refresh token: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login chatgpt`";

/// The refresh story's case, from its record's refresh facts (testing.md, 2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshCase {
    /// No earliest refresh time, or one at or before now: the run refreshes at once.
    Refreshed,
    /// An earliest refresh time more than the margin ahead: the run keeps the record.
    Kept,
    /// An earliest refresh time ahead within the margin: wait this many whole seconds,
    /// which pass it, then choose again.
    Wait { seconds: u64 },
}

/// The suite's durable user layer; each story requests only its own accounts.
pub struct Environment {
    pub root: PathBuf,
    /// Override the deployment refresh lead in seconds.
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

    /// The `ChatGPT` record's refresh facts, through smith's own reader.
    #[must_use]
    pub fn refresh_facts(&self) -> smith_local_shell::credentials::RefreshFacts {
        let _configuration = self.configuration();
        smith_local_shell::credentials::refresh_facts(&self.root.join("tokens/chatgpt"))
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

/// The story's case for `facts` at `now`; a record without a refresh token is
/// `Err(NOT_REFRESHABLE)`, whatever its earliest refresh time.
pub fn refresh_case(
    facts: &smith_local_shell::credentials::RefreshFacts,
    now: skein_lib::Wall,
) -> Result<RefreshCase, &'static str> {
    if !facts.refreshable {
        return Err(NOT_REFRESHABLE);
    }
    let Some(earliest) = facts.earliest_refresh_at else {
        return Ok(RefreshCase::Refreshed);
    };
    let remaining = earliest.as_nanos().saturating_sub(now.as_nanos());
    if remaining == 0 {
        Ok(RefreshCase::Refreshed)
    } else if remaining > skein_lib::Duration::from_secs(EARLIEST_MARGIN_SECONDS).as_nanos() {
        Ok(RefreshCase::Kept)
    } else {
        let seconds =
            remaining.div_ceil(1_000_000_000).checked_add(1).expect("the wait is at most the margin plus one");
        Ok(RefreshCase::Wait { seconds })
    }
}

/// One line of the refresh facts before and after a run, in whole seconds from `now`.
#[must_use]
pub fn refresh_report(
    before: &smith_local_shell::credentials::RefreshFacts,
    after: &smith_local_shell::credentials::RefreshFacts,
    now: skein_lib::Wall,
) -> String {
    let seconds = |at: skein_lib::Wall| {
        i128::from(at.as_nanos())
            .checked_sub(i128::from(now.as_nanos()))
            .expect("the difference of two u64 values fits i128")
            .checked_div(1_000_000_000)
            .expect("nanoseconds per second is nonzero")
    };
    let facts = |record: &smith_local_shell::credentials::RefreshFacts| {
        let earliest = record.earliest_refresh_at.map_or_else(|| "none".into(), |at| format!("{}s", seconds(at)));
        format!(
            "generation={} refreshable={} expires_in={}s earliest_in={earliest}",
            record.generation,
            record.refreshable,
            seconds(record.expires_at),
        )
    };
    format!("record before: {}; after: {}", facts(before), facts(after))
}

pub(crate) fn root(value: Option<&std::ffi::OsStr>) -> Result<PathBuf, &'static str> {
    value.filter(|value| !value.is_empty()).map(PathBuf::from).ok_or("$SMITH_LIVE unset")
}

pub(crate) fn setup(root: &Path, account: &str, key: bool) -> Result<(), &'static str> {
    let (path, message) = match account {
        "chatgpt" => ("tokens/chatgpt", CHATGPT),
        "claude" => ("programs/claude", CLAUDE),
        "anthropic-key" => return if key { Ok(()) } else { Err(KEY) },
        _ => panic!("unknown live account"),
    };
    std::fs::symlink_metadata(root.join(path)).map(|_| ()).map_err(|_| message)
}

/// Insert `-c deployment.refresh_lead=<lead_seconds>` before the trailing prompt argument `-`.
pub fn with_refresh_lead(arguments: &mut Vec<std::ffi::OsString>, lead_seconds: u64) {
    let at = arguments.len().checked_sub(1).expect("prompt argument");
    arguments.splice(at..at, ["-c".into(), format!("deployment.refresh_lead={lead_seconds}").into()]);
}

/// Insert `-c trace.path=<path>` before the trailing prompt argument `-`.
pub fn with_trace(arguments: &mut Vec<std::ffi::OsString>, path: &Path) {
    let at = arguments.len().checked_sub(1).expect("prompt argument");
    let path = toml::Value::String(path.to_str().expect("trace path is UTF-8").into());
    arguments.splice(at..at, ["-c".into(), format!("trace.path={path}").into()]);
}

/// The rendered settings an observer needs to tell a program account (smith-bench's `Prepared::rendered`).
#[must_use]
pub fn rendered(account: &str) -> serde_json::Value {
    let (endpoint, source) = match account {
        "claude" => ("anthropic", "program"),
        "chatgpt" => ("openai", "sign_in"),
        "anthropic-key" => ("anthropic", "env"),
        _ => panic!("unknown live account"),
    };
    serde_json::json!({
        "endpoints": {endpoint: {"account": account}},
        "accounts": {account: {"source": source}},
    })
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
