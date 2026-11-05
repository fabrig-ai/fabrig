+++
id = "close"
part = "close"
kind = "docs"
needs = ["machine"]
files = ["hearth/docs/development/debt/D001-room-checks-and-max-out.md", "hearth/docs/design/domain/render.status.md"]
effort = "medium"
minutes = 10
+++

# D001, updated

## Goal

This round doesn't pay D001, so its file stays and says what's true after `machine`.

Acceptance:

- D001 names no form that's gone, its **Paid when** doesn't change, and its **Measure** and next step are current.
- Its `tail.rs` row and the order held only by the design, resumed work before the next event, stay as they are.
- `python3 -I -B tools/citations/citations.py --all` stays at 0, and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-memory-3/notes/common.md`
- `hearth/docs/development/debt/README.md`

## Owns and may change

- **Owns:** D001, and nothing else but the line below.
- **`hearth/docs/design/domain/render.status.md`:** only `## Steps`, 7. This departs from the implementer's text, which leaves design alone: the cut wrote that entry for
  what this plan builds, and this task is the plan's last.
- **If `## What` or **Measure** have moved** because another draft landed its own edit first, keep theirs and apply yours on top.

## Steps

1. **Measure:** the figure `grep '^max_out ' hearth/scripts/old-forms.txt` prints on the trunk.
2. **Checked:** today, at the trunk's commit, from `date -u` and `git rev-parse --short=10 HEAD`.
3. **Where:** take out `hearth-protocol-mmdebstrap`.
4. **`## What`:** take "mmdebstrap" out of "the mmdebstrap, config and records machines". Recount the `.room()` calls in non-test step code with
   `git grep -n '\.room()' -- hearth/crates ':!hearth/crates/*/src/tests*'`, and give that figure for the 25.
5. **`### The machines and the domains`:** take out the rows of `resolve.rs`'s `requests`, `seen` and `pending`, and `indices.rs`'s `demands` and `found`. Change the paragraph
   under the table to say that the queues beside git's machines and mmdebstrap's, these last in `resolve.rs` and `indices.rs`, are uncapped, that the machines' states
   keep the orders in `hearth/docs/design/domain/render.md`, 8.5, and that their tests stand in `hearth/tests/git` and `hearth/tests/mmdebstrap`. Keep the `tail.rs` row, with
   whichever test it names.
6. **`## What pays it back`:** "Git and mmdebstrap are done; config and records come next, then the service." **`### What the service round must finish`:** add what `machine`'s
   commit message names, if anything.
7. **`render.status.md`:** take out the **Not yet** entry on 8.5's mmdebstrap readers, and set the file's date to today's.
8. Run the citations check and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit and push.

## Report

The first line, the measure, the `.room()` figure, the citations and `outward` results.
