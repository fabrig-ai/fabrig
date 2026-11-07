+++
id = "flows-close"
part = "temper-flow"
kind = "docs"
needs = ["scene-height", "flow-cases", "temper-flow", "temper-flow-look", "stock-flows"]
files = [
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 10
+++

# The status files say what's built

## Goal

Every **Not yet** this plan wrote for its design goes, since the code now does what the design says. What's still left stays, said plainly.

Acceptance:

- In `skein/docs/design/ui.status.md`, the sentences on 3.3, 3.5, 10.1, 11.5, 17 and 11.2 are gone, and the entry's first sentence, on section 3, stands.
- In `skein/docs/design/ui-web.status.md`, the **Not yet** entry on 8 and 12 is gone.
- In `temper/docs/design/client/README.status.md`, the **Not yet** entry on 3.5, *a chat's document*, is gone. A new **Not yet**, in its place: `3.5, a chat's document: temper/ui's after-landing look at a flow, in a chat and in its document.`
- In `temper/docs/design/client/browser.status.md`, the **Not yet** entry on 2.5, *a flow's look*, is gone, and in `temper/docs/design/domain/agent.status.md`, the one on 8.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new broken.

## Inputs

- `DESIGN.md`, sections 2 and 4.
- This plan's commits, by `git log` on your base, for anything built otherwise than its design says.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Departs from the implementer's text: `docs/design/` is closed to you but for these five status files, each only for these entries. No draft changes, and no case. `skein/docs/design/draft/diagrams.status.md` stays as it is.

Options, each pre-approved:

- If a commit says it built something otherwise than the design says, write a **Departs** entry in that design's status file, saying what and why, and name it in your report.
- If a sentence you must remove has changed its neighbours' wording in a merge, remove only what these entries say.

## Steps

1. The five files.
2. Update each file's date line to the day you run, by `date -u`.
3. The citations check. No build runs. Commit and push.

## Report

The implementer's report, with any **Departs** you wrote.
