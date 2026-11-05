+++
id = "status-close"
part = "status-close"
kind = "docs"
needs = ["failed-producer-holds", "stock-after-asking", "own-question-parks", "asker-left"]
files = ["temper/docs/design/domain/forge.status.md", "jig/docs/design/domain/tasks.status.md"]
effort = "medium"
minutes = 5
+++

## Goal

This plan's **Not yet** entries leave the status files, now the code holds them.

Acceptance: the entries below are gone, `forge.status.md`'s `producer-failure` **Design
departs** stays, and the citation check passes.

## Inputs

- `DESIGN.md`, section 2.
- `temper/docs/plan/producer-failure/notes/common.md`.

## Owns and may change

Owns the `files` above, a departure from the implementer's text: this task edits
`docs/design/` status files, and no other design file.

Options: if a merged task's report left one of these open, keep that entry, and say so.

## Steps

Remove, from `temper/docs/design/domain/forge.status.md`'s **Not yet**: the entries
beginning `8.1, 8.4: a producer that delivered`, `8.11: the stock coding charter` and
`8.11: an asker that ends first`. Remove from `jig/docs/design/domain/tasks.status.md`:
`5.7: a run kept by its own question ...`, up to the start of `5.9`. Date both files as
today, `date -u`.

Run `python3 -I -B tools/citations/citations.py --base origin/main`. Commit, push.

## Report

What left, and anything kept.
