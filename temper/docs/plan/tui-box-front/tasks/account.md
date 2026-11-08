+++
id = "account"
part = "account"
kind = "docs"
needs = ["world-front", "box-help", "tui-redact", "front-lines"]
files = [
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/tui/journeys.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/STATUS.md",
  "jig/docs/design/protocol/client.status.md",
  "jig/docs/design/STATUS.md",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "medium"
minutes = 45
+++

# tui-box-front: the status entries close, and the landing's account

## Goal

The plan's last task. Check every row of the plan's account note against
the trunk, close what the plan's **Not yet** entries still say where the
trunk shows it built, and write the landing's account in this task's commit
message (`temper/docs/design/draft/look-requests.md`, sections 3, step 7,
and 5).

Acceptance:

1. **The rows:** each row of `temper/docs/plan/tui-box-front/notes/account.md`
   checked against the trunk this task starts from (the code, the tests,
   the design): the commit message's table has one row per row of the note,
   `| Item | Result |`, written as it stands: landed; differs, and why; or
   deferred, to whom and where it is tracked. A row whose result isn't the
   note's expected one says so and why. The render check's row says it
   follows landing and names its owner, temper/tui's after-landing check;
   compilation and the goldens establish no rendered look.
2. **The status files** (a departure from the implementer's text, which
   keeps `docs/design/` closed): every **Not yet** bullet naming
   `tui-box-front` in `client/README.status.md`, `client/tui/journeys.status.md`,
   `protocol/setup.status.md`, `protocol/README.status.md`, jig's
   `protocol/client.status.md` and
   skein's `shell.status.md` that a task left behind is deleted where the
   trunk shows it built, or narrowed to what stays, its row then deferred
   to temper/tui; `journeys.status.md`'s **Not yet** for 8.3 and the
   journey `box-welcome` stays, as the account's journey row says, and
   `README.status.md`'s for the place ("box"), which waits for
   `tui-launchpad-2`'s `place`, stays unless `place` is on the trunk and
   the front draws it. Each
   file's date line becomes the day it runs (`date -u`). The trees'
   `docs/design/STATUS.md` summaries change only where they name these gaps.
   No design document's text changes.
3. `python3 -I -B tools/citations/citations.py --base origin/main` reports
   nothing new.

## Inputs

- `temper/docs/design/draft/look-requests.md`, 3 (step 7) and 5.
- `temper/docs/plan/tui-box-front/notes/account.md`, `notes/common.md`,
  `plan.md`; each task's report on the trunk's history (`git log` of
  `tui-box-front/*` merges), for what each left.
- `DESIGN.md`, 2 (status files: gaps as they stand, no history).

## Owns and may change

- Owns: the status files above, only as acceptance 2 says.
- Read, never change: any design document's text, the drafts, the code and
  tests. A gap you find in the code is a row (deferred, to temper/tui, and
  tracked by a **Not yet** you leave in the status file that names it), not
  a fix.

## Steps

1. Read each row's place on the trunk; note its result.
2. The status edits.
3. Citations.
4. Commit with the account's table in the message (its subject
   `tui-box-front: the landing's account`) and the trailer
   `Look-request: tui-box-front`; push.

## Report

As the implementer's text says, with the account's table.
