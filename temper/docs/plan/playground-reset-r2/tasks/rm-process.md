+++
id = "rm-process"
part = "forge-remove"
kind = "feature"
needs = ["rm-service", "bx-process", "cl-tools"]
files = [
  "temper/crates/temper/tests/support/start_over.rs",
  "temper/crates/temper/tests/process.rs",
]
effort = "high"
minutes = 90
+++

## Goal

The process tier (`temper` package, `process` binary, profile
`process-temper`) walks a set up cut short and started over (draft 6.4,
8): under `temper box`, an owned Forgejo's set up is killed with the box
mid-way; the next box start lists the forge unfinished; `temper ctl
forge remove N` removes it: its group stopped, `<data>/box/groups/N`,
`started/N` and its directory gone, the cache kept; a new set up makes
forge `N + 1`, whose fetch takes Forgejo's binaries from the cache, and
which connects with its first repository. And a removal cut by the box
killed after its answer ends whole at the next start.

Acceptance (process tier, new tests in `temper/crates/temper/tests/process.rs`
calling the new support module):

- `a_set_up_killed_is_found_unfinished_and_started_over_from_the_cache`:
  run one: `forge setup` (owned Forgejo), the box killed once the set
  up's `prepare` step is reported (stderr line), so before the first
  repository; run two: `forge list --json` shows forge 1 with
  `finished: false`; `forge remove 1` answers removing; then the
  forge list holds no forge 1, the directory `<data>/forge-1/` (the
  entry's `directory`) and `box/groups/1`, `box/started/1` are gone, the
  cache directory and its Forgejo binary are as before (same inode or
  sha256); `forge setup` again makes forge 2, its fetch step reports no
  download (the cache hit's line), and forge 2 lists `finished: true`.
- `a_removal_cut_by_the_box_ends_whole_at_the_next_start`: a forge
  removed and the box killed right after the `removing` answer, before
  `removed N` is seen; the next box start resends `remove N`; the
  group's files and directory are gone and the forge's secrets are
  erased (the store's application secrets hold no number of forge N:
  check through the engine's `forge list` and a `temper` admin read
  that already exists, or by the secrets file's absence of the forge's
  rows if a test helper reads it today; never by reading a secret's
  value).
- Each test's wall time measured and within the `process-temper` tier's
  budget (`tools/tiers/run`; the tier's per-test hard limit): say the
  figures in the report. Over budget is a question, never a move.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 3 and 5.
- `temper/docs/design/draft/playground-reset.md`, 4, 6.4 and 8.
- `temper/docs/design/protocol/setup.md`, 6.6 (the cache) and 7.1.
- `skein/docs/foundation/testing-strategy.md`, section 1 and 8.1 (tier
  budgets, starved).
- `AGENTS.md`, sections 5 and 6 (the tiers through `tools/tiers/run`).
- The support code: `temper/crates/temper/tests/support/boxed.rs`
  (`run_ctl`, `run_ctl_with_exits`, `echo_steps`), `support/owned.rs`,
  `support/forgejo.rs`, `support/ctl.rs` (`Step`, `Call`),
  `tests/process.rs` (`a_first_run_answers_on_the_demo` and the owned
  Forgejo tests near line 1783: the patterns to follow), and bx-process's
  process test of a group started from its file.
- The ctl: cl-tools' `forge remove FORGE`, `forge list --json`'s
  `finished`, `forge setup`.

## Owns and may change

Owns the files in the header. May change `temper/crates/temper/tests/process.rs`
(the new tests, and the old ones this part's change breaks: any that
assumed a forge cannot be removed or numbered by the document), and the
support modules under `temper/crates/temper/tests/support/` (helpers, a
kill-at-a-line step if `boxed.rs` lacks one: add it there).
`notes/scope.md` applies. Production code: none. A failure the tier
shows that is not this test's is reported with its log, as AGENTS.md 6
says.

## Steps

1. `support/start_over.rs`: the steps of each run (sign in, `forge
   setup`, kill at the step's line; `forge list`, `forge remove`, the
   files' checks, `forge setup` again), and the cache's fingerprint.
2. The two tests in `process.rs`.
3. Then, one at a time:
   - `heavy cargo nextest run -p temper --test process -E 'test(/started_over|removal_cut/)' --profile process-temper`
     (or through `tools/tiers/run` as `AGENTS.md` says for the tier);
   - `heavy cargo nextest run -p temper` (focused);
   - `heavy cargo clippy -p temper --all-targets -- -D warnings`;
     `cargo fmt`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit and push.

## Report

The standard report, plus: each test's wall time and the tier's
budget; how the box is killed at the step; how the cache hit is seen.
GitHub's Go on after a restart (draft 8) is not this task's: fr-process
covers the kept GitHub on the fake backends.
