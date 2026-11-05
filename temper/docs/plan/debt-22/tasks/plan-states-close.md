+++
id = "plan-states-close"
part = "forgejo-states"
kind = "docs"
needs = ["answer-measure", "small-plan-states", "turn-states", "contents-states", "actions-states", "writes-states", "reads-states"]
files = ["skein/docs/development/debt/D021-forgejo-plan-states.md", "skein/docs/design/forge.status.md"]
effort = "medium"
minutes = 10
+++

# D021 goes, and forge's status says what's left

## Goal

The item's **Paid when** holds, so it goes, with what the status says of it.

Acceptance:

- `temper/docs/plan/debt-22/notes/forgejo-plans.md`'s searches A, B and C read nothing, but for the note's exemptions, each named in your report.
- `skein/docs/development/debt/D021-forgejo-plan-states.md` is deleted.
- `skein/docs/design/forge.status.md`'s **Departs** on 4.2 reads, whole:

  > - **Departs:** 4.2: `skein/docs/foundation/programming-model.md`, 5.4's one handler per cell: the plans under `skein-forge/src/forgejo/plans` hold their states, and keep many cells as arms of one `match`; no reason is recorded.

  Its date becomes today's. No other entry changes.
- `python3 -I -B tools/citations/citations.py --all` reads 0, and `git grep -n 'D021-forgejo-plan-states' -- skein` finds nothing.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`, with this departure: this task edits the status file above, in the words above.
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, "The searches"
- `DESIGN.md`, 2, 4

## Owns and may change

- **Owns:** the files in the header. No code.
- **If a search reads more than the exemptions,** stop and ask: the item stays until it doesn't.

## Steps

1. The searches; the two files.
2. `python3 -I -B tools/debt/debt.py | rg D021` shows no skein row: jig's own D021 may stay. The citations check. Commit and push.

## Report

The common note's items; each search's count.
