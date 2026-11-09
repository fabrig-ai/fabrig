+++
id = "agent-version"
part = "agent-commit"
kind = "migration"
needs = ["launcher-crate"]
files = ["temper/crates/temper-agent/Cargo.toml", "temper/crates/temper-agent/build/**", "temper/crates/temper-agent/src/lib.rs", "temper/crates/temper-agent/src/tests.rs", "temper/docs/design/release.status.md"]
effort = "medium"
minutes = 25
+++

## Goal

`temper-agent --version` takes its VERSION and COMMIT from
`temper-version`, as `temper/docs/design/release.md`, section 3 has every
binary do, in place of its own build script (`build/main.rs`,
`build/commit.rs`, `TEMPER_AGENT_COMMIT`), which goes. The agent's
closure row allows `temper-version`, a binary's package's identity crate
with no dependency of its own.

Acceptance:

- `temper-agent`'s manifest has no `build` key and names
  `temper-version.workspace = true` among its `[dependencies]`; its
  `build/` directory is gone.
- `temper_agent::version()` returns `temper_version::line(NAME)` and a
  newline, and its doc cites `temper/docs/design/release.md`, section 3.
- `closure.rs`'s `temper-agent` row allows `"temper-version"`
  (`notes/contracts.md`, section 3) and nothing else is added.
- `heavy cargo nextest run -p temper-agent` passes: its parser table, the
  version line's test (Steps, 2) and the suite's mode table, whose
  `version` row compares the built binary's stdout with
  `temper_agent::version()`.
- `heavy cargo nextest run -p fabrig-boundary --test closure`:
  `the_check_names_what_a_row_excludes` passes, and
  `each_binary_links_what_its_row_allows` passes or names only
  `temper-agent excludes jig-…` lines, which are task
  `shell-without-jig`'s (list them under `## Left`); it never names
  `temper-version`.
- Your **Not yet** entries are deleted (`notes/common.md`): in
  `temper/docs/design/protocol/README.status.md`, the one beginning
  "**Not yet:** 4.1, 4.3 to 4.4, 5: `temper-agent --version`"; in
  `temper/docs/design/release.status.md`, the last one, beginning
  "**Not yet:** 3: `temper-agent --version`". Leave every other entry.

## Inputs

- `temper/docs/design/release.md`, section 3, *Both are
  `temper-version`'s*.
- `temper/docs/design/protocol/README.md`, 4.3 (*Its arguments*, item 3;
  *It links*), 4.4's table and section 5's rows `crates/temper-agent`
  and `crates/temper-version`.
- `temper/crates/temper-engine/src/main.rs` and its suite's `version`
  check (`temper/crates/temper-engine/tests/suite/main.rs`): how a binary
  reads `temper-version` today.
- `temper/docs/plan/agent-binary-closure/notes/common.md` and
  `notes/contracts.md`, section 3.

## Owns and may change

- Owns the files in `files`, and the two status entries named above.
- Neighbour files, each only as said:
  - `tests/boundary/tests/closure.rs`: `"temper-version"` appended to the
    agent row's `temper` list, after `"temper-fetch-shell"`.
  - `Cargo.lock`: as the build leaves it.
- Pre-approved:
  - `src/tests.rs` loses `#[path = "../build/commit.rs"] mod commit;` and
    the table test `the_commit_word_is_twelve_hex_digits_dirty_or_unknown`:
    the commit's word is `temper-version`'s, and its own tests hold it
    (`temper/crates/temper-version/tests/commit.rs`). Say so in the
    report; it's no loss of a rule.
  - If `temper/crates/temper-agent/tests/suite/modes.rs` reads
    `TEMPER_AGENT_COMMIT` or the build script, it reads `temper_version`
    instead; it reads neither on the trunk as this spec was cut.
- Not yours: `temper-launcher`, `temper-fetch-shell`, `temper-shell`;
  `temper-client`'s own build script (`TEMPER_CLIENT_COMMIT`), a recorded
  departure of another milestone; `temper-version`; every other
  `docs/design/` text; `docs/help/`.

## Steps

1. **The manifest and the code.** Drop `build = "build/main.rs"`, `git rm
   -r temper/crates/temper-agent/build`, add `temper-version.workspace =
   true` to `[dependencies]`. `version()`:

   ```rust
   /// The binary's version line (temper/docs/design/release.md, section 3).
   #[must_use]
   pub fn version() -> String {
       format!("{}\n", temper_version::line(NAME))
   }
   ```

2. **The test.** `the_version_line_names_the_binary_its_version_and_commit`
   asserts the line is `temper-agent VERSION (COMMIT)\n` with
   `temper_version::VERSION` and `temper_version::COMMIT`, and that COMMIT
   is twelve hex digits, with or without `-dirty`, or `unknown`, as the
   engine's suite checks it. Break it once by hand (print `NAME` twice, or
   drop the newline), see it fail for that reason, restore, and say which
   in the report.
3. **The row.** Append `"temper-version"` to `closure.rs`'s agent row.
   `cargo tree --locked --offline -p temper-agent -e normal --prefix none
   --format {p}` then lists `temper-version` and, of temper's crates, only
   the row's.
4. **Narrow runs:** `heavy cargo nextest run -p temper-agent`;
   `heavy cargo nextest run -p fabrig-boundary --test closure`. Check each
   run's count.
5. Delete your two status entries. `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-agent -p fabrig-boundary --all-targets -- -D warnings`;
   commit; `python3 -I -B tools/citations/citations.py --base origin/main`
   (it reads `HEAD`). The commit says why (every binary's COMMIT is
   `temper-version`'s, release.md 3); push.

## Report

As the implementer's text has it, and: the bite of Steps, 2; the `cargo
tree` list of Steps, 3; the counts of the runs in Steps, 4; and the
closure test's outcome, with any `jig-…` lines under `## Left`.
