+++
id = "fix-look"
part = "fix-look"
kind = "feature"
needs = ["walk"]
files = [
  "temper/docs/plan/ui-final-walk/findings.md",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/task-pages.css",
  "temper/crates/temper-client-shell-web/tests/suite/**",
  "temper/crates/temper-client-view/src/**",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 90
+++

# The first walk's small findings, in the sheets and the view

## Goal

Fix each row of `findings.md` whose owner is `fix-look`, each with a test red first, and fill in its `state`. The rows are the first walk's: read them, as this spec was written before they were.

Acceptance:

- Every `fix-look` row has a `state`: its commit and the test that was red first; `not reproduced`, with what you did; or `large, moved`, with its reproduction below the table and its owner changed to `fabrig/ui`.
- Each fix is its own commit, and says the finding's id.
- The packages you changed pass; goldens, the gallery's lists and the journeys' prints are regenerated.
- No row you own is left, and none has been explained away: a row you think wrong says why, and its pair is looked at again.
- **With no `fix-look` row:** commit "No findings for this task." in `findings.md`, and report `Done:`. That commit releases the next task.

## Inputs

- `temper/docs/plan/ui-final-walk/findings.md` and `report.md`, from `walk`; its screenshots, if your builder is `report.md`'s, and otherwise the driver.
- `temper/docs/plan/ui-final-walk/notes/common.md` (*A fix*) and `notes/decisions.md`.
- `temper/docs/design/client/greenfield/mock.css` and `app.js`, for the row's selector and markup.
- `temper/docs/design/client/browser.md`, 2.5, for how the sheets are laid out; `temper/docs/design/client/README.md`, the section of the page in question.

## Owns and may change

Owns, as `files` lists them.

May also change, for a fix's fallout alone: `temper/crates/temper-client-shell-terminal/src/tests/**` goldens regenerated; `temper/docs/design/client/journeys/**` prints regenerated, and a recording by `again`; `temper/docs/help/**` and `docs/help/reference/*`, where a word they quote changed; a design line a fix makes wrong (`notes/common.md`); `temper/crates/temper-client-view/src/limits.rs`, a derived count raised by what you add, which isn't a budget.

Options, each pre-approved:

- **A row whose fix proves to be in the domain:** change its owner to `fix-flow`, say why in `state`, and leave it. `fix-flow` runs next.
- **A look rule the terminal's print also reads:** the view stays and the sheet hides it in the browser (`notes/decisions.md`, *Chats*).
- **A rule the mock and a look request differ on:** the look request wins (`temper/docs/design/client/greenfield/README.md`).
- **A check that's a screenshot alone** (a rule's whole effect is its picture): the test still pins the rule, and a pair proves the picture. Take the pair with a throwaway script, untracked and gone before the push, and save it in `~/src/fabrig/ui-final-walk/fix-look/`.

## Steps

1. Read the rows; group them by page; start with what's most visible.
2. Each: the test, seen red; the fix; the test green; its row's `state`; commit.
3. The generated files, once at the end.
4. **Check,** the implementer's text: `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`, `temper-client-journeys`. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

As `notes/common.md`, Your report, and: a line per row, its state.
