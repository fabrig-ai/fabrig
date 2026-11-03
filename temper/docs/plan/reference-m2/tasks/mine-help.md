+++
id = "mine-help"
part = "mine"
kind = "docs"
needs = ["mine-call"]
files = ["temper/docs/help/change.md"]
effort = "medium"
minutes = 25
+++

# mine-help: the change page names `branch`

## Goal

`temper help change` names `change`'s `branch` member beside `landing`, `after` and
`changes` (`temper/docs/design/domain/agent.md`, 4.3.1), within the concept page's bound
(8,192 bytes, `temper-docs`' `Kind::Concept`); the page is 8,177 bytes before.

Acceptance: `temper/docs/help/change.md` carries the replacement below byte for byte and
nothing else changes; it is then 8,191 bytes; `temper-docs`' tests and the help tests of its
readers pass.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 (`branch`).
- `temper/docs/plan/reference-m2/notes/common.md`.

## Owns and may change

Owns `temper/docs/help/change.md`: this task departs from the implementer's text, whose "Not
yours" otherwise keeps `docs/help/` from implementers, for this one replacement. In the
page's first bullet under `## What it is`, these four lines:

```
  With the control family (amend, cancel, release), a run names, too,
  `landing` (`goal`, a branch, or the default), `after` (the changes
  it waits for, by task number or a `name` from the same call) and
  `changes` (several at once, all or none).
```

become these four:

```
  With the control family, a run names, too, `landing` (`goal`, a
  branch, or the default), `branch` (`mine`: its goal branch, landed
  whole), `after` (what it waits for: task numbers, or names from the
  same call) and `changes` (several at once, all or none).
```

If the page on your base is no longer 8,177 bytes (another plan changed it), apply the
replacement all the same; if the result passes the bound, say so; if a help test refuses it
for its length, stop and ask with the page's size, and raise no bound.

## Steps

1. The replacement; `wc -c temper/docs/help/change.md`.
2. `heavy cargo nextest run -p temper-docs`, then `heavy cargo nextest run -p
   temper-engine-smith -E 'test(/docs/)'` and `heavy cargo nextest run -p temper -E
   'test(/help/)'` (check each selected some; if a filter selects none, run the package's
   tests whose names `rg 'help' <crate>/src` shows read the pages).
3. Commit ("help: change names branch mine") and push.

## Report

As the implementer's text says, with the page's size.
