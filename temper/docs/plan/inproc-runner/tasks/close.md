+++
id = "close"
part = "close"
kind = "docs"
needs = ["repairs"]
files = ["tools/tiers/design.status.md"]
effort = "medium"
minutes = 45
+++

# inproc-runner: the tiers design's status says what is built

## Goal

With every part of the plan merged, `tools/tiers/design.status.md` says where the
one-process mode lags or departs from `tools/tiers/design.md`, sections 10.1 to 10.3
and 11, as the code now stands, and nothing else of this plan's.

Acceptance:

- The two **Not yet** entries this plan's cut wrote deleted: `11.1 to 11.8, 11.10, and
  10.3's one-process directory: ...` and `10.1, 10.2 and 11.9: ...`.
- Each place where the code (`tools/tiers/one_process.py`, `tools/tiers/cpu.py`'s
  one-process functions, `tools/tiers/one-process`, `test_one_process.py`) does
  otherwise than those sections say becomes a **Departs** entry, by section, with the
  reason the code's comments or its commits give (`git log` on those files); one with
  no recorded reason says so. What is designed and not built is a **Not yet** entry.
  Compare by reading the sections against the code: the refusals (11.1), the listing's
  arguments (11.2), the filtersets (11.3), the command line, lanes, splitting and groups
  (11.4), the environment's variables (11.5), the outcomes (11.6), the cancellation
  (11.7), the files and kept times (11.8), the command (11.9) and the suite's rows
  (11.10).
- The entries of the file not this plan's stay as they are; the date is today's.
- This departs from the implementer's text, which closes `docs/design/` and its kin:
  the task writes this status file and nothing else. The design's text stays as it
  is; what it should say otherwise goes in the report.

## Inputs

- `DESIGN.md`, 2 (status files).
- `tools/tiers/design.md`, 10.1 to 10.3 and 11.
- `temper/docs/plan/inproc-runner/notes/common.md`.

## Owns and may change

- Owns `tools/tiers/design.status.md` alone. No code changes: a gap is recorded, not
  closed.

## Steps

1. Read 10.1 to 10.3 and 11 beside the code; list the gaps.
2. The entries.
3. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text gives it, with each gap found and the entry it became.
