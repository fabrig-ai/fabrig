+++
id = "close"
part = "close"
kind = "docs"
needs = ["machine"]
files = ["hearth/docs/development/debt/D001-room-checks-and-max-out.md", "hearth/docs/design/domain/render.md"]
effort = "medium"
minutes = 15
+++

# D001, updated

## Goal

This round starts D001 and doesn't pay it, so its file stays and says what's true after `machine`.

Acceptance:

- D001 names no form that's gone, and its **Paid when** doesn't change.
- `python3 -I -B tools/citations/citations.py --all` stays at 0, and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-memory-2/notes/common.md`
- `hearth/docs/development/debt/README.md`

## Owns and may change

- **Owns:** D001, and nothing else but the line below.
- **`hearth/docs/design/domain/render.md`:** only if `service-orders`' commit says `file_told` before `told` is an order, and then one bullet in 8.5, in that
  section's voice, naming what keeps it. This departs from the implementer's text, which leaves design alone: the task's facts come from a test, and
  a design line is all that's left to carry them.
- **If `## What` or **Measure** have moved** because another draft landed its own edit of them first (`debt-14/ratchet` changes both), keep theirs and
  apply yours on top.

## Steps

1. **Measure:** the figure `grep '^max_out ' hearth/scripts/old-forms.txt` prints on the trunk.
2. **Checked:** today, at the trunk's commit, from `date -u` and `git rev-parse --short=10 HEAD`.
3. **Where:** take out `hearth-protocol-git`.
4. **`## What`:** take "git" out of "the git, mmdebstrap, config and records machines". Recount the `.room()` calls in non-test step code with
   `git grep -n '\.room()' -- hearth/crates ':!hearth/crates/*/src/tests*'`, and give that figure for the 26.
5. **`## What each room check keeps`:** take out the three rows of `hearth-protocol-git/src/tail.rs`. Change the paragraph on the capacity-1 queues so it
   says they're uncapped, and that the tests stand in `hearth/tests/git`.
6. **`### The service`:** read `service-orders`' commit message on the trunk. Where its table of bites names a different test from the orders table
   here, take its name. If its row 4 had no bite, say so in that row. Rewrite the sentence on `file_told` by what its row 6 found: nothing failed, or
   the test that did. If the commit reports a difference between queue sizes, stop: it's a `Question:`, with the figures.
7. **`## What pays it back`:** "from the machines up" stands. Add that git is done and mmdebstrap, config and records come next, then the service.
8. Run the citations check and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit and push.

## Report

The first line, the measure, the two figures, the citations and `outward` results.
