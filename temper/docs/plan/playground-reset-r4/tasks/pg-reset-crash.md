+++
id = "pg-reset-crash"
part = "playground"
kind = "feature"
needs = ["pg-reset"]
files = [
  "temper/tests/engine/domain/tests/suite/playground_crash.rs",
]
effort = "high"
minutes = 45
+++

## Goal

The engine's world for the playground's reset cut by a crash (draft
`temper/docs/design/draft/playground-reset.md`, 8, "The engine's
world", and 6.2, "Cut by a restart"): a reset crashed after each of its
kept acts is decided, or after its answer is saved, resumes at the next
start to the same end as an uncut reset; and a playground found running
at a start (D5) is reset the same way. This is the reset's half of the
world's crash tests (the forge removal's and the project deletion's
halves are the engine and jig designers').

Acceptance:

- for each cut point below, a world that starts the playground, opens a
  chat in its project, sends `EndPlayground`, then crashes its store at
  the cut (`Store::fork` of the durable commits, or `Store::crash`,
  as the world's restart tests cut it) and starts again on what was
  durable, reaches the same end as the uncut reset of
  `suite/playground_reset.rs`: no `Playground`, `Kept` or playground
  `Forge` row, no deployment account of the scripted provider, no
  project `Playground` in `me`, the chat ended; and a `StartPlayground`
  after it is offered;
- the cut points: after the decision that reads the row resetting;
  after `RemoveProject`'s answer is saved by jig but before the
  connector erased its row; after the step-2 decision (the forge
  removing, its `RemoveForge` and `BoxRemove` rows kept); after the
  removal's finish; after `RemoveAccount`'s decision; after the row's
  erasure (nothing to resume);
- a world restarted with the row `Running` and no reset asked (D5) ends
  reset, with no `EndPlayground` sent;
- each act is decided once: jig answers a resent `RemoveProject` or
  `RemoveAccount` from its retained answer (the same key), never a
  second deletion;
- the tests pass; nothing outside the test file changes.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D4, D5).
- `temper/docs/design/draft/playground-reset.md`, 6.2 and 8.
- `temper/docs/design/domain/testing.md` (the engine's world, its
  store cuts).
- `temper/docs/plan/playground-reset-r4/tasks/pg-reset.md` and its
  merged report; `temper/tests/engine/domain/tests/suite/playground_reset.rs`
  (its config and assertions, reused).
- The world's cut and restart helpers: `temper/tests/engine/domain/src/store.rs`
  (`crash`, `fork`), and a suite test that restarts on a forked store
  (`-E 'test(restart) | test(fork)'` lists them).

## Owns and may change

Owned: the new test file. Neighbours:
`temper/tests/engine/domain/tests/suite/main.rs`, one `mod` line.
Pre-approved: if the world cannot hold a commit between jig's answer
and the connector's erasure (one decision), drop that cut point and
say so; if a helper the cuts share would be useful to `suite/playground_reset.rs`
too, keep it in this file (read only there).

## Steps

1. `suite/playground_crash.rs`: one `#[test]` per cut point, named
   `a_reset_cut_after_<point>_ends_whole` (e.g.
   `a_reset_cut_after_its_decision_ends_whole`), sharing a `fn cut_at(
   point: Point) -> Driver` and a `fn assert_reset(d: &Driver)`; and
   `a_running_playground_found_at_a_start_is_reset`.
2. Narrow run: `heavy cargo nextest run -p temper-engine-domain-world
   --test suite -E 'test(playground_crash::)'`; `fuzzy` for one cut if
   the world's fuzzy profile runs the suite with seeds.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world
   --all-targets -- -D warnings`. Commit, push.

## Report

fab's sections; the cut points kept and any dropped, and why.
