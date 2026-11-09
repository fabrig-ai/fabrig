+++
id = "launcher-crate"
part = "engine-closure"
kind = "migration"
needs = []
files = ["temper/crates/temper-launcher/**", "temper/crates/temper-fetch-shell/**", "tests/boundary/tests/closure.rs", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/README.status.md"]
effort = "high"
minutes = 60
+++

## Goal

Claude Code's launcher leaves `temper-fetch-shell` for a crate of its
own, `temper-launcher`, so that `temper-fetch-shell`, which the forge's
setup links, links no smith crate, and `temper-engine`'s closure no longer
holds the nine smith crates its row in `tests/boundary/tests/binaries.rs`
excludes (`smith-agent-document`, `smith-agent-profile`, `smith-events`,
`smith-host-domain` and `smith-protocol-{channel,events,llm,machine,program}`),
all of which come through `launcher.rs`'s two calls to
`smith_agent_document::program::private_directory`. The code moves; no
row gains an exception.

Acceptance:

- `temper/crates/temper-launcher` exists, a workspace member under the
  `binary` lint profile, holding `temper-fetch-shell/src/launcher.rs`'s
  code and tests at its crate root, with the public surface
  `notes/contracts.md`, section 1 gives, bodies unchanged.
- `temper-fetch-shell` holds `pub mod fetch` alone, and its manifest
  names no smith crate and no dependency its remaining code doesn't use.
- `temper-agent` and the box's `temper` call `temper_launcher::...` where
  they called `temper_fetch_shell::launcher::...`, and link
  `temper-launcher` in place of `temper-fetch-shell`.
- `heavy cargo nextest run -p fabrig-boundary --test binaries` passes,
  both tests: `deployment_binaries_hold_their_closures` and
  `closure_matching`.
- `closure.rs`'s `temper-agent` row allows `"temper-launcher"`
  (`notes/contracts.md`, section 3). `heavy cargo nextest run -p
  fabrig-boundary --test closure`: `the_check_names_what_a_row_excludes`
  passes, and `each_binary_links_what_its_row_allows` either passes or
  fails naming only `temper-agent excludes jig-…` lines, which are task
  `shell-without-jig`'s (list them under `## Left`).
- The forge setup's process-boundary scan still holds the launcher's
  exemptions at its new path (Steps, 4).
- Your **Not yet** entries are deleted (`notes/common.md`): in
  `temper/docs/design/protocol/README.status.md`, the one beginning
  "**Not yet:** 4.1 to 4.4, 5: `temper-launcher`"; in
  `temper/docs/design/protocol/setup.status.md`, the one beginning
  "**Not yet:** 6.6, 11: `temper-launcher`"; in
  `temper/docs/design/README.status.md`, the one beginning "**Not yet:**
  9: `temper-launcher`". Leave every other entry, including the
  `temper-agent --version` one right after yours in
  `protocol/README.status.md`.

## Inputs

- `temper/docs/design/protocol/setup.md`, 6.6, the bullet *Its launcher*,
  and section 11: the crates' tree, `temper-fetch-shell` and
  `temper-launcher`, and the bullets *The agent's binary links no forge
  setup* and *The forge's setup links no launcher*.
- `temper/docs/design/protocol/README.md`, 4.1 (*What it links is held by
  a test*), 4.3 (*Its arguments*, item 1; *It links*), 4.4's table, and
  section 5's rows `crates/temper`, `crates/temper-agent`,
  `crates/temper-fetch-shell` and `crates/temper-launcher`.
- `tools/lints/design.md`, section 5: `--set`.
- `temper/docs/plan/agent-binary-closure/notes/common.md` and
  `notes/contracts.md`, sections 1 and 3.

## Owns and may change

- Owns the files in `files`; the three status files only for the entries
  named in the acceptance.
- Reader and neighbour files, each only as said:
  - `Cargo.toml` (root): a `workspace.members` line
    `"temper/crates/temper-launcher"` beside `temper-fetch-shell`'s, and a
    `[workspace.dependencies]` line `temper-launcher = { path =
    "temper/crates/temper-launcher" }` beside `temper-fetch-shell`'s.
  - `Cargo.lock`: as the build leaves it.
  - `temper/crates/temper-agent/Cargo.toml`: `temper-fetch-shell.workspace
    = true` becomes `temper-launcher.workspace = true` in
    `[dependencies]`; nothing else (task `agent-version` edits this file
    after you).
  - `temper/crates/temper-agent/src/main.rs` and `src/lib.rs`: the paths
    `temper_fetch_shell::launcher::X` become `temper_launcher::X`.
  - `temper/crates/temper/Cargo.toml`: `temper-fetch-shell.workspace =
    true` becomes `temper-launcher.workspace = true`, if no code of the
    package then names `temper_fetch_shell`; otherwise add the
    `temper-launcher` line and keep the other, and say why.
  - `temper/crates/temper/src/claude_code.rs`, `src/agent.rs`,
    `src/tests.rs`: the same path edits (`tests.rs`'s
    `the_launchers_io_is_the_small_profiles` reads `temper_launcher::IO`).
  - `temper/crates/temper-forge-setup-shell/tests/suite/boundary.rs`: the
    path `temper-fetch-shell/src/launcher.rs` becomes
    `temper-launcher/src/lib.rs`, in `forbidden_at` and in the rows of
    `the_scan_catches_each_form_and_passes_other_process_modules`; the
    comment above the branch stays true as it is.
- Pre-approved:
  - If a dependency line of `temper-fetch-shell` is unused once the
    launcher has left (`skein-json`, `skein-forge` and
    `smith-agent-document` should be), drop it; if `fetch.rs` uses one
    that the contract's list for `temper-launcher` omits, or the launcher
    needs one it doesn't list, follow the code and say so in the report.
  - If `temper-fetch-shell`'s `[dev-dependencies]` (`serde_json`) served
    only the launcher's tests, move it to `temper-launcher`.
  - Make the move with `git mv temper/crates/temper-fetch-shell/src/launcher.rs
    temper/crates/temper-launcher/src/lib.rs`, so the file's history
    follows; its inline `mod tests` moves with it. A module doc's
    citation, `(protocol/setup.md, 6.6)`, becomes
    `(temper/docs/design/protocol/setup.md, 6.6)`.
- Not yours: `temper-shell`, `temper-engine-shell` and the box's other
  files (task `shell-without-jig`); `temper-agent`'s build script,
  `--version` and tests (task `agent-version`); `binaries.rs`; every
  `docs/design/` text but your three entries; `docs/help/`.

## Steps

1. **The crate.** `temper/crates/temper-launcher/Cargo.toml`: `[package]`
   as `temper-fetch-shell`'s, with `name = "temper-launcher"` and
   `description = "Claude Code's launcher: its exec, its links and the
   substitution of \"fetched\""`; the dependencies of
   `notes/contracts.md`, section 1. Its `clippy.toml` a copy of
   `temper-fetch-shell`'s (the `ordinary` variant). Add the two root
   `Cargo.toml` lines, then give it the profile:
   `python3 -I -B tools/lints/lints.py --set binary temper/crates/temper-launcher`,
   and check `python3 -I -B tools/lints/lints.py temper` finds nothing.
2. **The move.** `git mv` the launcher (above); its `use temper_fetch::...`
   stays, and `crate::fetch::claude_code` becomes
   `temper_fetch_shell::fetch::claude_code`. `temper-fetch-shell`'s
   `lib.rs` keeps `pub mod fetch;` and its module doc says what is left:
   the pinned fetch's host (setup.md, 6.6 and 11). Its manifest's
   `description` becomes `"Pinned binary fetch host"`; drop its unused
   lines.
3. **The readers,** as `Owns and may change` lists them. Then
   `cargo tree --locked --offline -p temper-engine -e normal -i
   smith-agent-document` must answer `did not match any packages`, and `cargo tree -p temper-fetch-shell -e normal` must name no
   smith crate.
4. **The process scan.** The forge setup's scan
   (`temper-forge-setup-shell/tests/suite/boundary.rs`) walks every temper
   crate's `src/`, so it now reads `temper-launcher/src/lib.rs`: its
   exemption follows the file. Run
   `heavy cargo nextest run -p temper-forge-setup-shell --test suite -E 'test(/^boundary::/) | test(/^fetch::/)'`;
   `process_and_file_locks_stay_in_their_declared_modules` and
   `the_scan_catches_each_form_and_passes_other_process_modules` pass.
5. **The row.** Add `"temper-launcher"` to `closure.rs`'s row, after
   `"temper-docs"` (`notes/contracts.md`, section 3), and run both
   boundary binaries (acceptance).
6. **Narrow runs:**
   - `heavy cargo nextest run -p temper-launcher --lib` (the three moved
     tests);
   - `heavy cargo nextest run -p temper-agent` (its parser table and the
     suite's mode table, whose `claude` rows run the launcher);
   - `heavy cargo nextest run -p temper --lib -E 'test(=tests::the_launchers_io_is_the_small_profiles)'`;
   - `heavy cargo nextest run -p temper --test suite -E 'test(/^agent_modes::/)'`;
   - `heavy cargo check -p temper-engine -p temper-engine-shell --tests`.
   Check each run's count; one that selected nothing proves nothing.
7. Delete your three status entries. `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-launcher -p temper-fetch-shell -p temper-agent -p temper -p temper-forge-setup-shell -p fabrig-boundary --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main` after
   committing (it reads `HEAD`). Commit saying why (the launcher's smith
   reads leave the forge setup's fetch host, so the engine's closure holds
   its row); push.

## Report

As the implementer's text has it, and: the dependency lines dropped from
`temper-fetch-shell` and those `temper-launcher` took; whether the box
kept a `temper-fetch-shell` line, and why; the `cargo tree` answers of
Steps, 3; the counts of each run in Steps, 4 to 6; and
`each_binary_links_what_its_row_allows`'s outcome, with any `jig-…`
lines under `## Left`.
