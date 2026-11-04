+++
id = "d001-close"
part = "jig-core-steps"
kind = "docs"
needs = ["core-prepare-agent"]
files = [
  "jig/docs/development/debt/D001-roots-sequence-core-steps.md",
  "jig/docs/development/debt/D002-core-untyped-continuations.md",
  "jig/docs/development/debt/D004-ops-root-jig-wiring.md",
  "jig/docs/development/debt/README.md",
]
effort = "medium"
minutes = 20
+++

# jig D001: what stands, then the item goes

## Goal

D001's **Paid when** is rewritten to what stands, the measure is confirmed at 39, and the item and its row go in this
branch. The fixed answers left go to D004's text.

Acceptance:

- `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` prints 39, and the report says so.
- `jig/docs/development/debt/D001-roots-sequence-core-steps.md` and its row in `jig/docs/development/debt/README.md`
  are gone; temper's, smith's and skein's own D001 are untouched.
- D004's **What** says the fixed answers are its own.
- `tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-8/notes/common.md`, `temper/docs/plan/debt-8/inputs/request.md`, item 2.
- `jig/docs/development/debt/README.md`, the rule at its top.
- `jig/docs/design/domain/root.md`, 3 and 14.

## Owns and may change

- Owns: the four files in this spec's header, nothing else. debt-7's `service-items` also edits the README, in D005's
  row: if a merge meets it, keep both.
- D002 and D004 change only as the steps say. Reviews under `jig/docs/development/debt/reviews/` stay.

## Steps

1. Commit the rewrite first, so the history holds it: D001's **Paid when** reads
   `a `Now` value stays when any root does real work with it; none is answered by every root with a fixed core event.`;
   its **Measure** reads 39 at your base's commit; its **Checked** is today's.
   Its `## What` says what stands: temper does real work with `Now::Activate` (its byte budget and read room),
   `Now::BriefCorePlanned`, `Now::CompleteBrief` and `Now::WorkspaceRequest`.
2. Check the rewritten **Paid when** holds by reading those four arms in
   `temper/crates/temper-engine-domain/src/route/now.rs`. If it does not, stop and ask.
3. `jig/docs/development/debt/D004-ops-root-jig-wiring.md`: its **What** gains one bullet, in its form: `ops`'s and the
   test root's fixed answers to `Now::BriefCorePlanned`, and `ops`'s to `Now::WorkspaceRequest`, are this item's: they
   build a plan from the core's limits and answer with empty holds. Its line `with the same `Now` arms (D001)` loses
   the citation. `## What pays it back` drops `After D001: `, with the sentence's first word capitalised.
   Its **Checked** is today's.
4. `jig/docs/development/debt/D002-core-untyped-continuations.md`: `Goes with D001 and helps` loses D001, the rest
   kept. No other word changes.
5. Delete the file and its row, and fix what `python3 -I -B tools/citations/citations.py --base origin/main` lists.
   `rg -n 'D001-roots' jig temper skein smith tools` reads nothing outside `docs/plan/`.
6. Commit (`D001: what stands, the item goes`, saying why) and push. No code changes, so no tests run.

## Report

The measure, the base commit, and what each of the four arms does.
