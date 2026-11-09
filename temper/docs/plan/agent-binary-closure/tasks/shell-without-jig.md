+++
id = "shell-without-jig"
part = "agent-closure"
kind = "migration"
needs = []
files = ["temper/crates/temper-shell/**", "temper/crates/temper-engine-shell/src/lib.rs", "temper/crates/temper-engine-shell/src/forge.rs", "temper/crates/temper-engine-shell/src/forge_sign_in.rs", "temper/crates/temper-engine-shell/src/forge_sign_in/tests.rs", "temper/crates/temper-engine-shell/src/serve.rs", "temper/crates/temper/src/directories.rs"]
effort = "medium"
minutes = 45
+++

## Goal

`temper-shell` stops linking jig: its one jig use, the method
`Directories::jig()` (`temper/crates/temper-shell/src/directories.rs`),
moves out to its callers as `temper-engine-shell`'s `jig_directories`, and
the dependency on `jig-protocol-config` leaves its manifest. Then
`temper-agent`, which links `temper-shell`, links no jig crate (it links
sixteen today), and `tests/boundary/tests/closure.rs`'s agent row holds
without a change to the row. `temper-shell` is the shared crate of
temper/ui's and temper/tui's binaries; this plan's `plan.md` names the
change for them.

Acceptance:

- `temper-shell`'s manifest names no jig crate, and `cargo tree --locked
  --offline -p temper-shell -e normal` lists no `jig-` package.
- `temper_engine_shell::jig_directories` exists as `notes/contracts.md`,
  section 2 gives it, and `temper::directories` re-exports it.
- Every former `.jig()` on a `temper-shell` `Directories` (35 lines, in
  `temper-engine-shell` and the box's `temper`, its `src/` and `tests/`)
  calls `jig_directories(&…)` instead; nothing else at those lines
  changes.
- `heavy cargo nextest run -p fabrig-boundary --test closure` passes, both
  tests, with the row as your base has it.
- `heavy cargo nextest run -p fabrig-boundary --test binaries` passes or
  fails only on the smith crates task `launcher-crate` removes (list them
  under `## Left`).
- Your **Not yet** entry is deleted (`notes/common.md`): in
  `temper/docs/design/protocol/README.status.md`, the one beginning
  "**Not yet:** 4.3, 4.4, 5: `temper-shell` links `jig-protocol-config`",
  just above the **Departs** entries on `temper-client`. Leave every
  other entry.

## Inputs

- `temper/docs/design/protocol/README.md`, 4.3 (*It links*) and section
  5's rows `crates/temper-engine-shell` and `crates/temper-shell`.
- `temper/docs/design/protocol/configuration.md`, section 12: the three
  directories.
- `temper/docs/plan/agent-binary-closure/notes/common.md` and
  `notes/contracts.md`, section 2.

## Owns and may change

- Owns the files in `files`, and the status entry named above.
- Reader files, call sites only (`x.jig()` becomes `jig_directories(&x)`,
  plus the `use` line that brings `jig_directories` in):
  `temper/crates/temper/src/box_command.rs`, `fake.rs`, `fake/check.rs`,
  `fake/check/tests.rs`, `fake_backends.rs`, `fake_backends/import.rs`,
  `fake_backends/tests.rs`, `init/tests.rs`, `program_login.rs`,
  `tests/box_kinds.rs`, `worker_alone.rs`; and
  `temper/crates/temper/tests/suite/configuration.rs`,
  `tests/suite/forge_land.rs`, `tests/support/live.rs`, which call
  `temper::directories::jig_directories`.
- `Cargo.lock`: as the build leaves it (`temper-shell`'s entry loses
  `jig-protocol-config`).
- Pre-approved:
  - In the box's files, import `jig_directories` from
    `crate::directories` (or `temper::directories` in its `tests/`), not
    from `temper_engine_shell` directly, so the box's code reads its own
    `directories` module as it does for `program_environment`.
  - Where a call site's expression would exceed the line width, let
    `cargo fmt` reflow it; no other edit there.
  - If a call site's receiver is not a `temper-shell` `Directories`
    (`rg` finds the 35 lines; read each), leave it and say so.
  - If `temper-shell`'s own tests used `.jig()`, they move to
    `temper-engine-shell`'s tests unchanged but for the call; none do on
    the trunk as this spec was cut.
- Not yours: `temper-fetch-shell`, `temper-launcher`, `temper-agent`, the
  box's `claude_code.rs`, `agent.rs` and `src/tests.rs`, `Cargo.toml` at
  the root, and both boundary tests' rows (tasks `launcher-crate` and
  `agent-version`); every other `docs/design/` text; `docs/help/`.

## Steps

1. **The function,** in `temper-engine-shell`'s `lib.rs`, at its root,
   as the contract gives it; `lib.rs` already imports
   `directories::Directories` and `jig_protocol_config as config`, so its
   return type may read `config::Directories`. Its doc cites
   `temper/docs/design/protocol/README.md`, section 5.
2. **The method leaves.** Delete `Directories::jig` from
   `temper-shell/src/directories.rs`, and `jig-protocol-config` from its
   manifest's `[dependencies]`. `heavy cargo check -p temper-shell --tests`
   passes.
3. **The callers.** In `temper-engine-shell`: `forge.rs`,
   `forge_sign_in.rs`, `forge_sign_in/tests.rs` and `serve.rs` call
   `crate::jig_directories` (or `super::`, as the file's other crate-root
   calls do). The box's `directories.rs` gains `pub use
   temper_engine_shell::jig_directories;`, and its call sites import it
   from there. Compile with `heavy cargo check -p temper-engine-shell -p
   temper --tests` while errors last.
4. **Every reader of `temper-shell` still builds:** `heavy cargo check -p
   temper-shell -p temper-engine-shell -p temper -p temper-engine -p
   temper-client -p temper-client-shell-tui -p temper-agent --tests`.
5. **Narrow runs** (a signature moves; no meaning changes):
   - `heavy cargo nextest run -p fabrig-boundary --test closure`, and
     `--test binaries` (acceptance);
   - `heavy cargo nextest run -p temper-shell`;
   - `heavy cargo nextest run -p temper-engine-shell --lib -E 'test(/forge_sign_in::/)'`;
   - `heavy cargo nextest run -p temper --lib -E 'test(/box_kinds/) | test(/fake::check::/) | test(/fake_backends::/) | test(/init::tests::/)'`;
   - `heavy cargo nextest run -p temper --test suite -E 'test(/^configuration::/) | test(/^forge_land::/)'`.
   Check each run's count; one that selected nothing proves nothing.
6. Delete your status entry. `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-shell -p temper-engine-shell -p temper --all-targets -- -D warnings`;
   commit; `python3 -I -B tools/citations/citations.py --base origin/main`
   (it reads `HEAD`). The commit says why (`temper-shell`, every binary's
   start, links skein's crates and `temper-docs` alone, so the agent links
   no jig crate); push.

## Report

As the implementer's text has it, and: the call sites changed per file;
any `.jig()` you left and why; the counts of each run in Steps, 5; and
both boundary tests' outcomes, with any smith lines of `binaries.rs`
under `## Left`.
