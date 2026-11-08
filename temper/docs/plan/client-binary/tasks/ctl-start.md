+++
id = "ctl-start"
part = "ctl"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-client-shell-cli/src/start.rs",
  "temper/crates/temper-client-shell-cli/src/start/**",
  "temper/crates/temper-client-shell-cli/src/lib.rs",
  "temper/crates/temper-client-shell-cli/Cargo.toml",
  "temper/crates/temper/src/ctl.rs",
  "Cargo.lock",
]
effort = "medium"
minutes = 40
+++

# `ctl`'s start moves into its shell, for both binaries

## Goal

`temper ctl`'s entry, today `temper/crates/temper/src/ctl.rs`, moves verbatim
into `temper-client-shell-cli` as the module `start`, so that the box's `temper`
and the deployment's `temper-client` run one function for `ctl`
(`temper/docs/design/client/README.md`, section 2, *The clients' start, one for
two binaries*). No behaviour of `ctl` changes: its words, its lines (`temper ctl:
…`), its exits.

Acceptance:

1. **`temper_client_shell_cli::start`** (`notes/contracts.md`, section 8):
   `pub fn parse` and `pub fn run`, bodies as `ctl.rs` has them; the two helpers
   they read from `temper`'s root (`super::path`, bytes to a `Path`, and
   `super::close`, each descriptor closed by `skein_shell::close_startup_fd`)
   become private functions of the module. Module doc: "`ctl`'s start, which
   `temper ctl` and `temper-client ctl` run (temper/docs/design/client/tools.md,
   8.1; temper/docs/design/client/README.md, section 2)." `lib.rs` gains `pub mod
   start;`.
2. **`temper` runs it:** `temper/crates/temper/src/ctl.rs` holds only `pub use
   temper_client_shell_cli::start::run;` and `pub(crate) use
   temper_client_shell_cli::start::parse;` under its module doc; `lib.rs`'
   `ctl::run(&arguments, &Clock::new())` and the tests' `crate::ctl::parse`
   (`temper/crates/temper/src/tests/help.rs`) compile unchanged.
3. **Its tests:** `temper`'s tests reading `ctl::parse` pass unchanged
   (`tests::help::…`, which runs every documented `ctl` call through `parse`);
   no new test (a move, `notes/common.md`).

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`; `notes/contracts.md`,
  section 8.
- Design: `temper/docs/design/client/README.md`, section 2 (the crate tree's
  `temper-client-shell-cli` line and *The clients' start, one for two binaries*);
  `temper/docs/design/client/tools.md`, 8.1.
- The code: `temper/crates/temper/src/ctl.rs` (all 97 lines), `lib.rs`' `close`
  (about line 590) and `path`; `temper/crates/temper-client-shell-cli/src/lib.rs`
  and its `Cargo.toml` (it has `skein-shell`, `skein-io`, `skein-lib` and
  `temper-client-tools` already); `temper/crates/temper/src/tests/help.rs`, lines
  about 390 to 440 (`crate::ctl::parse`).

## Owns and may change

Owns `start.rs` (new) and the `pub mod start;` line of
`temper-client-shell-cli`, a dependency line in its `Cargo.toml` if one is
missing, and `temper/crates/temper/src/ctl.rs`. Nothing else of `temper`: not
`lib.rs` (another task of this plan and the in-flight `tui-box-front` change
it).

Options, each decided:

- **A clippy finding the move surfaces** under the shell crate's lints (the same
  `ordinary` variant as `temper`'s): reshape the moved line where the fix is
  local, else a scoped `#[expect(…, reason = "…")]`, named in the report.
- **A test of `parse` inside the shell:** none is required; if you add one, it is
  a row, and it bites.

## Steps

1. Move the module (acceptance 1 and 2). `heavy cargo check -p
   temper-client-shell-cli -p temper --tests` until it builds.
2. `heavy cargo nextest run -p temper --lib -E 'test(/tests::help::/)'`.
3. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-shell-cli -p temper --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-shell-cli -p temper`.
4. Commit ("ctl's start moves into temper-client-shell-cli, for both binaries")
   and push.

## Report

`Done:`, `Failed:` or `Question:` first, then the files changed, the runs and
their counts, any expectation added, anything left.
