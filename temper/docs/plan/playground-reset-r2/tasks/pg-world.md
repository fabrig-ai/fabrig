+++
id = "pg-world"
part = "playground"
kind = "feature"
needs = ["pg-gone"]
files = [
  "temper/tests/engine/domain/src/reset.rs",
  "temper/tests/engine/domain/tests/suite/fuzzy/playground_reset.rs",
]
effort = "high"
minutes = 75
+++

## Goal

The engine's world proves the playground's reset, and the project
deletion and forge removal inside it, end whole however a crash cuts
them (draft 8, "The engine's world"): a started playground with a chat
in its project is reset once uncut, then again once per durable commit
with the store crashed right after that commit, the root rebuilt and
started, and run to its end; every run ends as the uncut one, and
nothing of the playground's forge or project is read afterwards but
what draft sections 4 and 5 keep (spend in the totals; nothing else).

Acceptance (fuzzy profile):

- `fuzzy::playground_reset::a_reset_crashed_after_each_commit_ends_as_uncut`:
  for every commit `k` of the uncut run, the cut run ends with no
  `Playground`, `Forge`, `KeptAct` row, no row of the deleted project in
  jig's families but its spend (contracts.md, 1), no scripted account,
  the chat erased; the rows of both stores equal the uncut run's; each
  reset step's and kept act's deployment act answered once by jig under
  its key (retained answers).
- `fuzzy::playground_reset::a_running_playground_crashed_is_reset_at_start`:
  a playground started, crashed after each commit of its start, then
  started again: the start resets it (draft 9's first default) and it
  ends as the uncut reset; `StartPlayground` then makes a forge numbered
  past every number given.
- Both report their seed on failure and replay from it.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 1 and 2.
- `temper/docs/design/draft/playground-reset.md`, 4, 5, 6.2, 8 and 9.
- `skein/docs/foundation/testing-strategy.md`, section 1 and its fuzzy
  sections.
- The world: `temper/tests/engine/domain/src/removal.rs` (rm-world's
  crash-and-restart helpers: reuse them), `tests/suite/playground_start.rs`
  and `tests/suite/playground_reset.rs` (pg-start's and pg-reset's
  helpers), `src/direct.rs`, `src/store.rs`.

## Owns and may change

Owns the files in the header. May change the world's `src/lib.rs` and
`tests/suite/main.rs` (one `mod` line each), `src/removal.rs` (to share
a helper: generalize, keep its callers passing), and move a helper from
the suite files into `src/reset.rs`. `notes/scope.md` applies. No
production code: a fault the sweep finds is reported as the first line,
`Question:`, with the commit number and the rows that differ, unless it
is a one-line slip in `reset.rs`/`kept.rs`/`remove.rs`.

## Steps

1. `src/reset.rs`: `story(seed)` (pg-start's world start, a chat in the
   playground's project), `reset(&mut ..)` (`EndPlayground` as the
   deployment owner), `serve` (the service's answers to `RemoveGroup`/
   `EraseSecrets`, from `removal.rs`), `run_to_end`, `crash_after`, and
   `rows` (both stores' rows, spend ledgers set apart).
2. The two fuzzy tests; measure each one's wall time and keep it within
   the fuzzy profile's per-test budget (say the figures).
3. Then, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite --profile fuzzy -E 'test(/playground_reset/)'`;
   - `heavy cargo nextest run -p temper-engine-domain-world`;
   - `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit and push.

## Report

The standard report, plus: the uncut runs' commit counts; the tests'
wall times; every difference found before they passed and where it was
fixed.
