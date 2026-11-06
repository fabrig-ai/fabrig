+++
id = "trial-close"
part = "trial-close"
kind = "docs"
needs = ["trial-worlds"]
files = [
  "temper/docs/design/client/README.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/draft/model-check.status.md",
  "temper/docs/design/domain/trial.status.md",
  "temper/docs/design/STATUS.md",
  "temper/docs/help/**",
]
effort = "medium"
minutes = 40
+++

# The design says what stands

## Goal

The design catches up with what's built. This is the one place the implementer's text's `docs/design/` and `docs/help/` closure
doesn't hold: this task's work is design text, from what the trunk holds and `temper/docs/design/domain/trial.md`.

Acceptance:

- `temper/docs/design/client/README.md`, 3.5, *The try*: says what feeds it (`../domain/trial.md`, 6.1 to 6.4), that "Try again"
  runs another trial past a host's stop, that it's not a chat, and that a stop's why is the run's ending or the wait.
  *Home*'s line and the accounts page's row cite `../domain/trial.md`, 6.2 and 6.3.
- `temper/docs/design/client/README.status.md`: its **Not yet** on *The try*, closed; a gap left is said, and only that. The `Open`
  section's *What feeds the try* goes.
- `temper/docs/design/client/browser.md`, 7.2: the gallery's table has `accounts-try-stopped` and `home-try-stopped`, and the
  trial's own states on `welcome-ready-*`.
- `temper/docs/design/client/mockups/STATUS.md` and `temper/docs/design/STATUS.md`: a line this makes false.
- `temper/docs/design/draft/model-check.status.md`: **Design lags** holds only what stands. **Design departs** holds what `trial.md`
  decided against the draft, each with why and who decided: *Leaving* cancels nothing and a restart cancels it; a second account's
  trial may answer as another; a deployment's accounts aren't tried; "first committed text" is the first committed turn.
- `temper/docs/design/domain/trial.status.md`: 2 to 7 closed, or what's left. The trial.md text says what the code does: a line that
  isn't so, you change here, and say which in your report.
- `temper/docs/help/**`: whichever page says a model's account is usable only after its check, or the welcome stops on it: fixed in
  its bound, by the help page's rules. None: say so.
- `temper/docs/help/settings.md` and `charter.md`: the stock's charter count and list take charter 10, as a trial's and
  never offered: "Stock floors are six charters" becomes seven, while its rules still offer six charter executors and
  `tasks.executor_kinds` stays nine.
- `python3 -I -B tools/citations/citations.py --base origin/main`: clean.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `notes/client.md`.
- `DESIGN.md`, whole.
- `temper/docs/design/domain/trial.md`, whole; `temper/docs/design/draft/model-check.md`, whole.
- What the plan landed: `git log --first-parent model-check/integration`.

## Owns and may change

Owns, as `files` lists them, in small hunks: `chat-reading-c/reading-close` is editing `README.md`, `browser.md`, `mockups/STATUS.md`,
`STATUS.md`. Nothing else: no code. A test that reads a help page (`temper/docs/help/**` are read by tests) is run, and its
expectation changed if your page's words moved.

Options, each pre-approved:

- **`README.md`, 3.5 no longer has the look round's lines:** `notes/client.md` applies, and it was merged in; if it isn't on your
  base even so, take the note's merge.
- **A draft's section the design now carries whole:** say so in the status; the draft isn't yours to delete, and neither is its text.

## Steps

Read, then edit. Run `heavy cargo nextest run -p temper-docs` if you changed a help page, and the citations.

## Report

As the implementer's text asks, with the design's lines you changed and why.
