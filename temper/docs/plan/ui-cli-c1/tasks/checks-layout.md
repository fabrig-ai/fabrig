+++
id = "checks-layout"
part = "checks"
kind = "feature"
needs = []
files = ["tests/boundary/tests/cli.rs", "tests/boundary/Cargo.toml", ".config/gate/README.md"]
effort = "high"
minutes = 120
+++

## Goal

The command line's checks 1, 2 and 3 (`skein/docs/foundation/cli.md`,
9) run in `fabrig-boundary`'s focused tests, landing green: every
binary target is a package of its own at `<tree>/bin/<name>/main.rs`;
every binary's source is at most 30 lines; no workspace crate has
features. What doesn't follow them yet is an exception in the check's
source, naming the round or milestone that removes it, and an
exception that matches nothing fails.

Acceptance:

- `tests/boundary/tests/cli.rs` holds three tests,
  `every_binary_is_a_package_of_its_own` (check 1),
  `every_main_is_plumbing` (check 2) and
  `no_workspace_crate_has_features` (check 3), each with its exceptions
  table (`notes/contracts.md`, 3), each passing on the trunk.
- Every exception's `until` follows `notes/contracts.md`, 4: temper's
  binaries and features by the map; smith's, skein's, jig's and
  hearth's as their rounds.
- Each check fails, naming each, on a finding no exception names and on
  an exception that names nothing; a table test in the file holds both
  on made-up inputs, as text, without the trunk (Steps, 5).
- `.config/gate/README.md` names the checks as gate item 10.
- `heavy cargo nextest run -p fabrig-boundary --test cli` passes, and
  the whole package's focused tests do.

## Inputs

- `skein/docs/foundation/cli.md`, 2, 3 and 9 (checks 1 to 3, and the
  exceptions' rule).
- `tests/boundary/tests/fail_stop.rs`: how it reads `cargo metadata`'s
  bin targets and their sources (checks 1 and 2 read the same).
- `tests/boundary/tests/direction.rs`: how it reads the root's members
  and each member's manifest, its tables and dependency lines (check 3
  reads them the same way).
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 3 and 4 (binding),
  and `notes/common.md`.

## Owns and may change

- Owns the files in `files`; `cli.rs` is new.
- **The root's `Cargo.toml`, one move:** `tests/boundary/Cargo.toml`'s
  `syn = { version = "2", features = ["full", "visit"] }` goes to the
  root's `[workspace.dependencies]` as it stands, and the member says
  `syn.workspace = true`, so the check's own crate has no exception. If
  another member names `syn` and that changes its build, except
  `tests/boundary/Cargo.toml: syn` instead and say so.
- Share nothing with `fail_stop.rs` or `direction.rs` by editing them:
  each file of `tests/boundary/tests/` is its own crate, so copy the few
  helpers `cli.rs` needs.
- Not yours: any manifest's dependencies but the move above, any
  binary, any `main.rs`. C1 brings nothing into line (`notes/common.md`).

## Steps

1. **The exceptions' type and rule,** once for the three checks:

   ```rust
   /// What a check excepts, and the round or milestone that removes it
   /// (skein/docs/foundation/cli.md, 9).
   struct Exception { what: &'static str, until: &'static str }

   /// The findings no exception names, and the exceptions that name no
   /// finding, as one failure message; `None` when both are empty.
   fn judged(check: &str, findings: &[String], exceptions: &[Exception]) -> Option<String>;
   ```

   The message lists each unexcepted finding, then each stale exception
   with "remove it: the check's exceptions only shrink".
2. **Check 1,** from `cargo metadata --no-deps --format-version 1
   --locked`: for each bin target, a finding `PACKAGE TARGET` unless its
   source, from the workspace root, is `<tree>/bin/<TARGET>/main.rs` with
   `<tree>` one of `skein`, `smith`, `jig`, `temper`, `hearth`, its
   package is named `<TARGET>`, and the package has no `lib` target and
   no other `bin` target.
3. **Check 2,** from the same metadata: a finding `PACKAGE TARGET` for
   each bin target whose source has more than 30 lines.
4. **Check 3,** from the root's members and manifests: a finding
   `MANIFEST: [features]` for a member with a `[features]` table, and
   `MANIFEST: KEY` for each dependency line of a member, in any
   dependency table (normal, dev, build, target-specific, a dotted
   `[dependencies.KEY]` table), that sets `optional`, `default-features`
   or `features`; and `Cargo.toml: KEY` for each entry of the root's
   `[workspace.dependencies]` with a `path` that sets one. An external
   crate's entry in the root may set them. A line the check can't read
   is a failure, never a skip, as in `direction.rs`.
5. **The rule's table test,** `exceptions_only_shrink`: rows of
   (findings, exceptions, expected outcome) for a finding excepted, one
   not excepted, and an exception naming nothing.
6. **The exceptions:** run each check with an empty table, then write
   one exception per finding, `until` by contracts 4. Keep each table
   sorted by `what`. A target or a manifest under `temper/` that the map
   doesn't place goes to `cli C8` (a tool or fixture) and your report
   names it.
7. **The gate's README,** item 10 after 9, in its words: `**The
   command line's checks:** cargo nextest run -p fabrig-boundary,
   included in default. Rule: skein/docs/foundation/cli.md, 9, and
   tests/boundary/tests/cli.rs's module doc: the layout, main.rs's
   size and no features, with their exceptions.` (code spans as item 9
   has them).
8. `cargo fmt`; `heavy cargo clippy -p fabrig-boundary --all-targets --
   -D warnings`; `heavy cargo nextest run -p fabrig-boundary`. Bite:
   delete one exception of each check and see it fail naming that
   finding; add a made-up one and see it fail as stale; restore.
   Commit, push.

## Report

The three checks, each table's count of exceptions by `until`, what
`syn`'s move did, the tests and their counts, the bites and the
failures seen, and any target or manifest you placed by judgement.
