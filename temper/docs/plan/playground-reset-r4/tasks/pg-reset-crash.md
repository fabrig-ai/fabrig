+++
id = "pg-reset-crash"
part = "playground"
kind = "feature"
needs = ["pg-reset"]
files = [
  "temper/tests/engine/domain/tests/suite/playground_crash.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs",
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
- a restart on a store holding a removed forge's `Repository` rows (and
  the rows that link to them) reconciles (Answers, 2026-11-01);
- the tests pass; nothing outside the files owned changes.

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

Owned: the new test file; the reconcile fix in
`temper-engine-domain-forge/src/reconcile.rs` and its test in
`src/tests/reconcile.rs` (Answers). Neighbours:
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
3. The reconcile fix (Answers) first, narrow: `heavy cargo nextest run
   -p temper-engine-domain-forge --lib -E 'test(reconcile::) |
   test(release::) | test(removal::)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world
   -p temper-engine-domain-forge --all-targets -- -D warnings`. Commit, push.

## Report

fab's sections; the cut points kept and any dropped, and why.

## Answers

2026-11-01. Q: the cuts after the removal's finish, `RemoveAccount`'s
decision and the row's erasure panic at restart (`reconcile.rs`,
`Repository -> Forge(1)`): route the repair to the engine's owners, or
extend this task? A: extend this task; the fault is reconcile's, not the
writers' order. Contracts D3 says the `Forge` row is erased once its last
kept act is answered, while "the forge's repositories' rows stay, ended
and readable, until their project's deletion releases them"; a removal
outside the playground (a start over, 6.4) leaves them with no deletion
at all. So a restored `Repository` row, and a row whose need is
`Need::Repository`, must resolve when its forge was removed, not only
when it is configured. In `reconcile.rs`: a private `removed_forge(forge)`,
true when `next_forge` is `Some(next)`, `forge < next` and the forge is
not configured (`configured_forge` false), the rule `demo.rs`'s
`forge_ended` holds for a removed forge, written in reconcile so
`pg-demo-out-engine`'s rewrite of `demo.rs` does not reach it;
`Need::Repository` resolves on `(configured_forge || removed_forge)` and
the repository retained; `Stored::Repository`'s need becomes
`Need::Repository(row.provider)` (its own row is retained, so its forge
decides). No new `Need` variant, so `domain.rs`'s key mapping stays as
it is (`RecordKind::Repository` maps by its row). `Need::Forge` stays as
it is for writers: a removed forge's writer is erased with it, and a
restored one is a writer bug still. Test in `tests/reconcile.rs`:
`a_removed_forges_repository_restores` (a `Forges { next: 2 }` row and a
`Repository` row of forge 1, no forge 1 configured: reconcile passes;
the same `Repository` row without the `Forges` row still panics). If,
once fixed, another missing link names a row the removal should have
erased (a writer, a client row), that is a question again, not a
widened fix. Keep all six cuts.
