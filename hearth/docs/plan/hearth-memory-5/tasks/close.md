+++
id = "close"
part = "close"
kind = "docs"
needs = ["machine"]
files = ["hearth/docs/development/debt/D001-room-checks-and-max-out.md", "hearth/docs/design/domain/render.status.md", "hearth/docs/design/domain/testing.status.md"]
effort = "medium"
minutes = 10
+++

# D001, updated

## Goal

This round doesn't pay D001, so its file stays and says what's true after `machine`.

Acceptance:

- D001 names no form that's gone, its **Paid when** doesn't change, and its **Measure** and next step are current.
- Its other rows and the order held only by the design, resumed work before the next event, stay as they are.
- `python3 -I -B tools/citations/citations.py --all` stays at 0, and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-memory-5/notes/common.md`
- `hearth/docs/development/debt/README.md`

## Owns and may change

- **Owns:** D001, and nothing else but the two lines below.
- **`hearth/docs/design/domain/render.status.md` and `testing.status.md`:** only `## Steps`, 7 and 8. This departs from the implementer's text, which leaves design alone: the cut wrote the first entry for what this plan builds, the second lists the tests this plan adds, and this task is the plan's last.
- **If `## What` or **Measure** have moved** because another draft landed its own edit first, keep theirs and apply yours on top.

## Steps

1. **Measure:** the figure `grep '^max_out ' hearth/scripts/old-forms.txt` prints on the trunk.
2. **Checked:** today, at the trunk's commit, from `date -u` and `git rev-parse --short=10 HEAD`.
3. **Where:** take out `hearth-client-protocol-records/src/printer.rs`.
4. **`## What`:** the first bullet ends "`MAX_OUT` in five domains, and `MaxOut` and `MAX_OUT_*` in `hearth-protocol-builder`;". Recount the `.room()` calls in non-test step code with `git grep -n '\.room()' -- hearth/crates ':!hearth/crates/*/src/tests*'`, and give those figures for the 23 calls and the 21 lines.
5. **`### The machines and the domains`:** take out the two rows of `client.rs`'s `requests` and `events`. In the paragraph under the table:
   - its first sentence says that the queues beside git's machines, mmdebstrap's, the config decoder and the printer, these last in `hearth-protocol-builder/src/resolve.rs`, `indices.rs`, `read.rs` and `client.rs`, are uncapped;
   - the sentence on the orders' tests says they stand in `hearth/tests/git`, `hearth/tests/mmdebstrap`, `hearth/tests/config`, `hearth/tests/builder` and `hearth-client-protocol-records/src/tests/printer.rs`;
   - the sentence on `Client`'s queues goes.
6. **`## What pays it back`:** "Git, mmdebstrap, config and records are done; the service comes next." **`### What the service round must finish`:** add what `machine`'s commit message names, if anything.
7. **`render.status.md`:** take out the **Not yet** entry on the printer's orders, and set the file's date to today's.
8. **`testing.status.md`,** 9.1: "and the git, mmdebstrap and config worlds' order tests" becomes "and the git, mmdebstrap and config worlds' order tests and the printer's". Set the file's date to today's.
9. Run the citations check and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit and push.

## Report

The first line, the measure, the `.room()` figures, the citations and `outward` results.
