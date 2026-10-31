+++
id = "cl-tidy"
part = "client"
kind = "migration"
needs = ["cl-gallery", "cl-world-playground", "cl-world-project", "cl-tools", "pg-gone"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/crates/temper-client-tools/**",
  "temper/crates/temper-client-shell-*/**",
  "temper/tests/client/**",
  "temper/tests/client-browser/**",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "medium"
minutes = 60
+++

## Goal

Remove what the client part kept only as a bridge or a shim, and the
words nobody says any more, so the client holds no demo halves,
retirement, setup strip or configured-slot words (draft 6.3, "What
goes"); then run the browser tier once on the finished client.

Acceptance:

- `rg -n 'Halves|halves|retired_demo|RetiredDemo|other_model_usable|listed::setup|SetupBoth|SetupForge|SetupModel|DemoBoth|DemoModel|DemoForge|SteppedAside|next_number|Elsewhere' temper/crates/temper-client-* temper/tools/gallery temper/tests/client`
  finds nothing but what a comment of the report names as kept with a
  reason.
- `family::demo` and `forge::demo::DemoState` (the client's) are gone;
  the playground's names in the client domain read `playground` where
  they are public (`Domain::demo` gone, `Domain::playground` stays).
- The focused tests of every client package, `temper-client-journeys`
  included, pass; the old-forms counts of `temper/` fall or stay.
- The browser tier run once (below), its result in the report.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- The reports of `cl-play-domain`, `cl-play-view`, `cl-welcome` (the
  shims and their readers) as their commits carry them, and
  `temper/docs/plan/playground-reset-r2/tasks/cl-play-domain.md`'s shim
  list.
- `temper/docs/design/draft/playground-reset.md`, 6.3.
- `temper/docs/design/client/journeys.md`, 6 and 8.

## Owns and may change

Owns the files in the header (no other client task runs now). The
renames it makes are inside the client crates and their readers listed;
a rename outside them is forced (scope.md).

## Steps

1. Delete the shims (`cl-play-domain.md`'s list) once `rg` shows no
   reader; delete the `Word` variants of V/words.rs and
   V/accounts/words.rs no code says, with their phrases, ids and lists.
2. Rename the client's public `demo` names that now mean the playground
   (`demo_answered` → `playground_answered`, `demo_running` →
   `playground_running`, …), with their readers, where the rename is
   mechanical; leave names the journeys' recordings carry in an event's
   Debug text (`Pages(Playground)`, `Forge(..)`), or record them again
   (`cl-surface.md`, step 13).
3. Reprint the journeys; they must not go stale here.
4. **The browser tier, once:** `tools/tiers/run browser-temper` (the
   root configuration's profile; `temper/tests/client-browser`). A
   failure from a ctl argv or a page this run changed is fixed in
   `temper/tests/client-browser/**` (its helpers, e.g.
   `start_playground_chat`, its configuration); any other failure is
   reported with its log and the machine, not fixed. Starved is a
   warning (testing-strategy.md, 8.1).
5. Checks: `cargo fmt`; `heavy cargo clippy` over every changed package
   `--all-targets -- -D warnings`; one `heavy cargo nextest run` over the
   client packages and `temper-client-journeys`; old forms with
   `--write` where they fell.
6. Commit and push.

## Report

- What was deleted and renamed; anything kept and why.
- The browser tier's result: passed, starved, or each failure with its
  log path.
