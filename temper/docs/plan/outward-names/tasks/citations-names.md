+++
id = "citations-names"
part = "citations"
kind = "docs"
needs = []
files = ["tools/citations/README.md"]
effort = "medium"
minutes = 10
+++

# tools/citations' README stops naming the pipeline

## Goal

fabrig names nothing of whoever runs its development (`AGENTS.md`, 1). The citation checker's
README calls a plan's files "fab's records of a change". It says "a runner's records of a
change" instead (`inputs/request.md`, How), and nothing else changes. `tools/citations/citations.py`
already holds no name: its inventory entry is the last task's to drop.

Acceptance:

- `tools/citations/README.md` holds none of the lab's names; the only changed words are
  ~l.27's "fab's records of a change" → "a runner's records of a change" (rewrapped within its
  list item if the line grows past its neighbours').
- The check of `notes/common.md` ("Checking your files are clean"), with the entries of
  `tools/citations/README.md` and `tools/citations/citations.py` deleted, passes (1 selected);
  the inventory is then restored and not committed.
- `python3 -I -B tools/citations/test_citations.py` passes; `python3 -I -B
  tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/notes/contracts.md`, 3: the wording.
- `temper/docs/plan/outward-names/inputs/request.md`: the request; its last How item is this
  task.

## Owns and may change

- **`tools/citations/README.md`**, the one phrase. If, at your base, the line holds a further
  name, it is changed by `notes/contracts.md`, 3, and the report says so.
- Nothing else: not `citations.py`, `allow.txt` or the tests; not `tests/boundary/outward.txt`.
- **Departures from the implementer's text:** no Rust changes, so no `cargo fmt`, clippy or
  nextest of a tree's package; `tools/` is no tree's, so no old-forms run. The one heavy command
  is the check above.

## Steps

1. The phrase. `git diff` shows one line (or one list item, rewrapped).
2. The check, as `notes/common.md` says; restore the inventory.
3. Commit; `test_citations.py`; `citations.py --all`; push.

## Report

As the implementer's text says, and: the line before and after; the check's selected count; the
two Python checks' last lines; anything under `## Left`.
