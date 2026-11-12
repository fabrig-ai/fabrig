//! Temper's test table and environment capture
//! (`temper/docs/design/cli/README.md`, 11).

use skein_shell::{Set, TestTable, Variable, Way};
use std::{ffi::OsString, os::unix::ffi::OsStringExt};

pub const BOX_PARENT_TEST: Variable = Variable {
    name: "TEMPER_TEST_BOX_PARENT_TEST",
    set: Set::Test,
    form: "DIR",
    line: "Run the box parent-death story's child with this scratch directory; the process test sets it.",
    absent: "Run the parent side of the story.",
    notes: "",
    way: Way::Reads,
};

pub const BUNDLE: Variable = Variable {
    name: "TEMPER_TEST_BUNDLE",
    set: Set::Test,
    form: "DIR",
    line: "Locate the built browser bundle; the browser tier sets it.",
    absent: "The browser world refuses to run without a bundle.",
    notes: "Relative paths are resolved from temper's tree; the directory must hold index.html.",
    way: Way::Reads,
};

pub const E4_CAPTURE: Variable = Variable {
    name: "TEMPER_TEST_E4_CAPTURE",
    set: Set::Test,
    form: "SOCKET",
    line: "Connect the Forgejo-elsewhere story to its private capture witness; the story's parent sets it.",
    absent: "Start the capture listener and re-execute the story as a child.",
    notes: "An abstract Unix socket name, not credential bytes.",
    way: Way::Reads,
};

pub const E4_LIFE: Variable = Variable {
    name: "TEMPER_TEST_E4_LIFE",
    set: Set::Test,
    form: "DIR",
    line: "Run one engine life in the Forgejo-elsewhere story's scratch directory; the story sets it for its child.",
    absent: "Run the complete story, including capture and both engine lives.",
    notes: "",
    way: Way::Reads,
};

pub const E4_RESTART: Variable = Variable {
    name: "TEMPER_TEST_E4_RESTART",
    set: Set::Test,
    form: "1",
    line: "Select the restarted engine life in the Forgejo-elsewhere child; the story sets it.",
    absent: "Run the first engine life.",
    notes: "Any nonempty value selects the restart.",
    way: Way::Reads,
};

pub const FORGEJO: Variable = Variable {
    name: "TEMPER_TEST_FORGEJO",
    set: Set::Test,
    form: "FILE",
    line: "Locate pinned Forgejo for owned-forge support, real checkout delivery and the git transcripts tool; CI or a developer sets it.",
    absent: "The tests and tool that need pinned Forgejo refuse to run.",
    notes: "",
    way: Way::Reads,
};

pub const FORGEJO_RUNNER: Variable = Variable {
    name: "TEMPER_TEST_FORGEJO_RUNNER",
    set: Set::Test,
    form: "FILE",
    line: "Locate the pinned Forgejo runner for owned-forge support; CI or a developer sets it.",
    absent: "The owned-forge tests that need the runner refuse to run.",
    notes: "",
    way: Way::Reads,
};

pub const FORGE_PARENT_HELPER: Variable = Variable {
    name: "TEMPER_TEST_FORGE_PARENT_HELPER",
    set: Set::Test,
    form: "1",
    line: "Run the startup suite's inherited-listener helper; its fake Forgejo script sets it.",
    absent: "The helper test does nothing.",
    notes: "Any nonempty value selects the helper.",
    way: Way::Reads,
};

pub const LIVE_ACCOUNT: Variable = Variable {
    name: "TEMPER_TEST_LIVE_ACCOUNT",
    set: Set::Test,
    form: "FILE",
    line: "Name a held ChatGPT account record for a live story; a developer sets it for proof-live.sh.",
    absent: "The landing story signs in interactively; rung-b0 requires a held account.",
    notes: "A regular 0600 file, moved into a fresh store by account import; a kept store cannot import it again.",
    way: Way::Reads,
};

pub const LIVE_ANTHROPIC_KEY: Variable = Variable {
    name: "TEMPER_TEST_LIVE_ANTHROPIC_KEY",
    set: Set::Test,
    form: "FILE",
    line: "Name the Anthropic key file for forge-land-claude-key; the after-landing tier or a developer for proof-live.sh sets it.",
    absent: "The key landing story skips, saying why.",
    notes: "A nonempty regular 0600 file of the test's user, not a link; passed to account add-key by path, never read, printed or copied by the story.",
    way: Way::Reads,
};

pub const LIVE_CLAUDE: Variable = Variable {
    name: "TEMPER_TEST_LIVE_CLAUDE",
    set: Set::Test,
    form: "DIR",
    line: "Name a held Claude Code login for forge-land-claude; a developer sets it for proof-live.sh.",
    absent: "The Claude landing story refuses to run.",
    notes: "A nonempty 0700 directory, not a link, whose basename is a valid source name; only for forge-land-claude.",
    way: Way::Reads,
};

pub const LIVE_GITHUB_FIXTURES: Variable = Variable {
    name: "TEMPER_TEST_LIVE_GITHUB_FIXTURES",
    set: Set::Test,
    form: "DIR",
    line: "Locate the live GitHub story's fixtures; proof-live-github.sh sets it.",
    absent: "The live GitHub story refuses to run.",
    notes: "The directory holds app.json with the App and owner installation's public fixture.",
    way: Way::Reads,
};

pub const LIVE_GITHUB_KEEP: Variable = Variable {
    name: "TEMPER_TEST_LIVE_GITHUB_KEEP",
    set: Set::Test,
    form: "1",
    line: "Keep the live GitHub story's scratch state on exit; a developer sets it for proof-live-github.sh.",
    absent: "Remove the scratch state on exit.",
    notes: "Only 1 keeps the state.",
    way: Way::Reads,
};

pub const LIVE_GITHUB_OWNER: Variable = Variable {
    name: "TEMPER_TEST_LIVE_GITHUB_OWNER",
    set: Set::Test,
    form: "OWNER",
    line: "Select the live GitHub story's owner; proof-live-github.sh sets it.",
    absent: "The live GitHub story refuses to run.",
    notes: "The story accepts only the probe owner, fabrig-ai.",
    way: Way::Reads,
};

pub const LIVE_KEEP: Variable = Variable {
    name: "TEMPER_TEST_LIVE_KEEP",
    set: Set::Test,
    form: "DIR",
    line: "Select where proof-live.sh retains redacted notices; a developer may set it.",
    absent: "Use the live-evidence directory under the host's scratch area, or beside the task directory for rung-b0.",
    notes: "The retained directory and notice are private; the transient client bearer is redacted.",
    way: Way::Reads,
};

pub const LIVE_NOTICE: Variable = Variable {
    name: "TEMPER_TEST_LIVE_NOTICE",
    set: Set::Test,
    form: "FILE",
    line: "Append live notices to the private notice file; proof-live.sh creates it and sets the path.",
    absent: "Print notices to stderr only.",
    notes: "The transient notice may contain the full sign-in link; the retained copy is redacted.",
    way: Way::Reads,
};

pub const LIVE_STORE: Variable = Variable {
    name: "TEMPER_TEST_LIVE_STORE",
    set: Set::Test,
    form: "DIR",
    line: "Select the live stories' persistent deployment stores; a developer or proof-live.sh sets it.",
    absent: "The launcher chooses its live-store directory, or a fresh task-adjacent directory for rung-b0; the stories require the launcher's value.",
    notes: "Landing stories use a subdirectory per story; rung-b0 uses the directory itself. A held account needs a fresh store.",
    way: Way::Reads,
};

pub const MANAGED_CLIENT: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_CLIENT",
    set: Set::Test,
    form: "sign-in|adopt",
    line: "Select the managed sitting's fake client role; the sitting or proof-live-github-managed.py sets it.",
    absent: "Do not run a fake client role.",
    notes: "An unknown role is refused.",
    way: Way::Reads,
};

pub const MANAGED_EVIDENCE: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_EVIDENCE",
    set: Set::Test,
    form: "FILE",
    line: "Select the managed relay's redacted evidence file; the sitting or proof-live-github-managed.py sets it.",
    absent: "The managed relay refuses to run.",
    notes: "The relay creates a new 0600 file.",
    way: Way::Reads,
};

pub const MANAGED_MODE: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_MODE",
    set: Set::Test,
    form: "fake|live",
    line: "Select the managed relay's transport; the sitting or proof-live-github-managed.py sets it.",
    absent: "The managed relay refuses to run.",
    notes: "Fake mode requires an upstream socket address; live mode uses the live transport.",
    way: Way::Reads,
};

pub const MANAGED_PERSON: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_PERSON",
    set: Set::Test,
    form: "LOGIN",
    line: "Name the person making the GitHub App in the managed sitting; a developer supplies it to the launcher, which sets it for the relay.",
    absent: "The live launcher and managed relay refuse to run.",
    notes: "The fake sitting supplies its own person; the relay requires a nonempty login.",
    way: Way::Reads,
};

pub const MANAGED_PREFIX: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_PREFIX",
    set: Set::Test,
    form: "PREFIX",
    line: "Limit the managed relay's branch namespace to this sitting; the sitting or proof-live-github-managed.py sets it.",
    absent: "The managed relay refuses to run.",
    notes: "",
    way: Way::Reads,
};

pub const MANAGED_SCRATCH: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_SCRATCH",
    set: Set::Test,
    form: "DIR",
    line: "Locate the managed relay's scratch directory; the sitting or proof-live-github-managed.py sets it.",
    absent: "The managed relay refuses to run.",
    notes: "The relay writes relay.jsonl and relay-ready.json here.",
    way: Way::Reads,
};

pub const MANAGED_UPSTREAM: Variable = Variable {
    name: "TEMPER_TEST_MANAGED_UPSTREAM",
    set: Set::Test,
    form: "IP:PORT",
    line: "Locate the managed relay's fake upstream; the sitting or proof-live-github-managed.py sets it.",
    absent: "Fake mode refuses to run; live mode needs no upstream.",
    notes: "Parsed as a socket address in fake mode.",
    way: Way::Reads,
};

pub const PLAYGROUND: Variable = Variable {
    name: "TEMPER_TEST_PLAYGROUND",
    set: Set::Test,
    form: "DIR",
    line: "Locate the built playground for the browser story; the browser tier sets it.",
    absent: "The playground browser story refuses to run.",
    notes: "The story canonicalizes the path before serving it.",
    way: Way::Reads,
};

pub const RATE: Variable = Variable {
    name: "TEMPER_TEST_RATE",
    set: Set::Test,
    form: "1",
    line: "Print the installation's HTTP routes and costs from the forge protocol's rate suite; a developer sets it.",
    absent: "The suite prints nothing.",
    notes: "Any nonempty value enables printing.",
    way: Way::Reads,
};

pub const RECORD_RELEASE: Variable = Variable {
    name: "TEMPER_TEST_RECORD_RELEASE",
    set: Set::Test,
    form: "FILE",
    line: "Write the confirm tests' release card tree recording to this path; a developer sets it.",
    absent: "Compare the release card tree against its checked recording.",
    notes: "",
    way: Way::Reads,
};

pub const SITTING_WAIT_SECS: Variable = Variable {
    name: "TEMPER_TEST_SITTING_WAIT_SECS",
    set: Set::Test,
    form: "SECONDS",
    line: "Delay the fake managed sitting before reading the adopted-card confirmation; a developer sets it for proof-live-github-managed.py.",
    absent: "Do not delay.",
    notes: "A nonnegative integer; ignored in live mode.",
    way: Way::Reads,
};

pub const WRAPPER: Variable = Variable {
    name: "TEMPER_TEST_WRAPPER",
    set: Set::Test,
    form: "COMMAND",
    line: "Wrap builds in temper's proof scripts and real-loop fixture regeneration; a developer sets it.",
    absent: "Run the build commands directly.",
    notes: "One command name, not shell words.",
    way: Way::Reads,
};

/// Temper's test set, in byte order of its variable names.
pub const TESTS: TestTable = TestTable {
    tree: "temper",
    variables: &[
        BOX_PARENT_TEST,
        BUNDLE,
        E4_CAPTURE,
        E4_LIFE,
        E4_RESTART,
        FORGEJO,
        FORGEJO_RUNNER,
        FORGE_PARENT_HELPER,
        LIVE_ACCOUNT,
        LIVE_ANTHROPIC_KEY,
        LIVE_CLAUDE,
        LIVE_GITHUB_FIXTURES,
        LIVE_GITHUB_KEEP,
        LIVE_GITHUB_OWNER,
        LIVE_KEEP,
        LIVE_NOTICE,
        LIVE_STORE,
        MANAGED_CLIENT,
        MANAGED_EVIDENCE,
        MANAGED_MODE,
        MANAGED_PERSON,
        MANAGED_PREFIX,
        MANAGED_SCRATCH,
        MANAGED_UPSTREAM,
        PLAYGROUND,
        RATE,
        RECORD_RELEASE,
        SITTING_WAIT_SECS,
        WRAPPER,
    ],
};

/// The host's temporary directory, outside temper's test set.
pub const TMPDIR: Variable = Variable {
    name: "TMPDIR",
    set: Set::Public,
    form: "DIR",
    line: "Select the host's temporary directory for test scratch; the host or test launcher sets it.",
    absent: "Use the host's default temporary directory.",
    notes: "",
    way: Way::Reads,
};

pub const SKEIN_CODEGEN_WRITE: Variable = Variable {
    name: "SKEIN_CODEGEN_WRITE",
    set: Set::Test,
    form: "1",
    line: "Rewrite checked documents and goldens; a developer sets it.",
    absent: "Compare the generated bytes with the checked file.",
    notes: "Only 1 enables rewriting. Skein's round declares this in skein's test table.",
    way: Way::Reads,
};

pub const SKEIN_TEST_CHROMIUM: Variable = Variable {
    name: "SKEIN_TEST_CHROMIUM",
    set: Set::Test,
    form: "FILE",
    line: "Locate Chromium for browser tests; a developer or the browser tier sets it.",
    absent: "Find chromium on PATH, refusing to run if it is missing.",
    notes: "The named path must be a file. Skein's round declares this in skein's test table.",
    way: Way::Reads,
};

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

#[expect(
    clippy::disallowed_methods,
    reason = "a test harness: temper's test table captures the process environment (temper/docs/design/cli/README.md, 11.2)"
)]
fn capture() -> Box<[EnvironmentEntry]> {
    skein_shell::environment()
}

fn value_in(variable: &Variable, pairs: &[EnvironmentEntry]) -> Option<OsString> {
    variable.value(pairs).map(|bytes| OsString::from_vec(bytes.to_vec()))
}

/// A declared variable's value in the process's environment: none
/// when it's absent or empty.
#[must_use]
pub fn value(variable: &Variable) -> Option<OsString> {
    value_in(variable, &capture())
}

/// The process's environment, as `skein_shell::environment` returns it,
/// for a test that hands a child the environment it was given.
#[must_use]
#[expect(clippy::type_complexity, reason = "the capture's byte pairs are the interface of cli/README.md, 11.2")]
pub fn environment() -> Box<[(Box<[u8]>, Box<[u8]>)]> {
    capture()
}

fn rewrite_in(pairs: &[EnvironmentEntry]) -> bool {
    value_in(&SKEIN_CODEGEN_WRITE, pairs).is_some_and(|value| value == "1")
}

/// Whether the rewrite switch is on: `SKEIN_CODEGEN_WRITE` is `1`.
#[must_use]
pub fn rewrite() -> bool {
    rewrite_in(&capture())
}

#[cfg(test)]
mod tests;
