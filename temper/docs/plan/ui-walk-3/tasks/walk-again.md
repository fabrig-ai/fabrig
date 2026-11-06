+++
id = "walk-again"
part = "walk-again"
kind = "docs"
needs = ["fix-flow"]
files = [
  "temper/docs/plan/ui-walk-3/report.md",
  "temper/docs/plan/ui-walk-3/findings.md",
  "temper/docs/plan/ui-walk-3/additions.md",
  "temper/docs/plan/ui-walk-3/real-check.md",
  "temper/docs/plan/ui-walk-3/driver/**",
]
effort = "high"
minutes = 75
+++

# The walk once more, only over what the fixes touched: the plan's result

## Goal

From your base, the trunk with every fix, walk again each state `fix-look` and `fix-flow` touched, at each width they name, and no other. Then write the plan's result. This is the page the user reads.

Acceptance:

- **`report.md` is the request's *The result*:** the table, every state at both widths, as `notes/walk.md` says, in the form of `walk-2/report.md`; `additions.md`'s rows folded in, and a state walked again takes its new row and pair. Each row names which walk's commit and folder it comes from. A short opening above the table: how far the client matches the mock, in plain words, and what's still different and why.
- **Compared with the second walk,** under three headings of those names: *What's fixed*, *What's new*, *What regressed*. Each line a finding's id, or a state's, with its evidence. W3, W6, A2, A3, A4 and R1 are each in one of them, or say that they're still open. A4 says its score.
- **`findings.md`:** `additions.md`'s and `real-check.md`'s rows folded in, in one table. Every row whose `state` claims a fix has been seen fixed in this walk, or its `state` says it wasn't and it has a new row. `additions.md` and `real-check.md` are then removed, their reproductions and steps carried across.
- **The list for `fabrig/ui`:** each `large` finding, as its own line under a heading of that name in `report.md`, with its reproduction. Don't plan them.
- **`owed` lines,** in a section of their own: what `chat-reading`, `sign-in-new-tab` and `model-check` still owe, each naming its draft and section. Not findings.
- **The real check:** R1's verdict in a section of its own, with its limits. Run it again, as `temper/docs/plan/ui-walk-3/tasks/real-login.md` says, only if a fix touched the command or the fetch.
- **New small findings:** fix one here, with a test red first, if it's a word, a link or a look rule that fits in a commit, and walk that state again. Otherwise it's a `B` row, named on the list.
- On the builder: `~/src/fabrig/ui-walk-3/index.html`, the contact page, linking each state's pair, with `report.md` copied beside it. `report.md` names each builder and folder it draws on (`walk`, `additions`, `real-login`, this one), and names `~/src/fabrig/ui-walk-3/` as where the screenshots are.
- Every box and Chromium you started is stopped.

## Inputs

- `temper/docs/plan/ui-walk-3/inputs/request.md`, *The walk*, *The result*.
- `temper/docs/plan/ui-walk-3/notes/common.md`, `notes/walk.md`, `notes/decisions.md`.
- `temper/docs/plan/ui-walk-3/driver/README.md`; `report.md`, `findings.md`, `additions.md`, `real-check.md`; `walk-2/report.md`, `walk-2/findings.md`.
- `git log` of `fix-look`'s and `fix-flow`'s commits, and their rows' `state`, which name the states each fix touched. A fix whose states you can't tell from its row is walked by the page it changed.

## Owns and may change

Owns, as `files` lists them.

May also change: for a new small finding only, what `fix-look` may (`temper/docs/plan/ui-walk-3/tasks/fix-look.md`, Owns and may change); and `temper/crates/temper-fake/src/backends.rs`, where a fix moved what a script needs.

Options, each pre-approved:

- **No fix landed:** nothing to walk again. Write the result from what `walk`, `additions` and `real-login` found, and say so at its head.
- **The driver stopped on a fixed page:** a name a fix changed is the driver's to follow; change it, and say so.
- **Your builder isn't `walk`'s:** its pairs aren't on yours. Link them by builder and folder, as the second walk's report does, and make your own contact page of what you hold. Move nothing between builders.
- **A fix touched W6's brief:** the state needs its 31 minutes by the clock. **A fix touched anything A4 presses:** the ten repeats again.
- **More small findings than fit:** the walk and its fixes had their chance, so what doesn't fit goes on the list as `small`, and you end.

## Steps

1. Read the fixes; write down the states they touched.
2. Build and copy; walk them at both widths.
3. Fixes that fit, if any, then walk those states again.
4. `report.md`, `findings.md`, the contact page, on a clean look at each.
5. If you changed code: **Check,** the implementer's text. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As `notes/common.md`, Your report, and: the counts of each verdict against the second walk's, the list's lines, and the builder and folder of the contact page.
