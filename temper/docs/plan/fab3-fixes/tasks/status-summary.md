+++
id = "status-summary"
part = "fixes"
kind = "docs"
needs = []
files = ["temper/docs/design/STATUS.md"]
effort = "medium"
minutes = 15
+++

# temper's STATUS.md summary drops the connected Forgejo's closed lag

## Goal

`temper/docs/design/STATUS.md`'s summary paragraph (~27-30) says a connected Forgejo, its connect
with its bot's tokens and a forge's Replace lag, citing six documents; their status files and the
code have them, and the one gap left is `client/tools.status.md`'s 11.3, the real loop's story of
a connected Forgejo. The summary says that gap alone.

Acceptance:

- The clause "a connected Forgejo, its connect with its bot's tokens and a forge's Replace lag
  (`protocol/forge.md`, `protocol/setup.md`, `protocol/credentials.md`, `client/README.md`,
  `client/forge.md`, `client/tools.md`)" is gone; the sentence names the real loop's story of a
  connected Forgejo as a lag, citing `client/tools.md`. Suggested text, from "and lags in":
  "and lags in a removal without the box (`domain/forge.md`), a project's Delete
  (`client/README.md`) and the real loop's story of a connected Forgejo (`client/tools.md`);
  with `application.machine.github` null, GitHub's remotes come out malformed
  (`protocol/configuration.md`)."
- Nothing else in the paragraph moves but its line wrapping, which follows the paragraph's
  (about 70 columns); the date line is today's (`date -u +%F`).
- `client/forge.status.md`, `client/tools.status.md` and the other status files stay as they
  are: check that none of `protocol/forge.status.md`, `protocol/setup.status.md`,
  `protocol/credentials.status.md`, `client/README.status.md` or `client/forge.status.md` lists a
  gap of a connected Forgejo, its tokens' connect or a Replace. If one does, keep it in the
  summary in the same clause, citing it, and say so in the report.
- The citations check is clean for `STATUS.md`.

## Inputs

- `temper/docs/plan/fab3-fixes/notes/common.md`.
- `temper/docs/plan/fab3-fixes/inputs/request.md`, finding 2.
- `DESIGN.md`, 2 (a status file and the tree's `STATUS.md` summary).
- `temper/docs/design/client/tools.status.md`, its **Not yet** 11.3 entry (read only).

## Owns and may change

`temper/docs/design/STATUS.md` alone, its summary paragraph and date line. This departs from the
implementer's text ("Not yours: any `docs/design/`"): this task is that one edit. No other file.

## Steps

1. Edit the paragraph as above.
2. From the repository root, `python3 -I -B tools/citations/citations.py --base origin/main`;
   fix what it lists in `STATUS.md`. No build or test reads `STATUS.md`
   (`.config/gate/read-by-tests.txt`), so nothing else runs.
3. One commit saying why; push.

## Report

As the implementer's text says, and the paragraph's new sentence as written.
