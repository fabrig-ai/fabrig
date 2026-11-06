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

## Answers

2026-11-06 (fabrig/ui). Addition, two fixes the user asked for today (via fabrig/temper). Causes are a reader's, not reproduced: reproduce first, each with a test red first.
1. The project menu's rename form should match the mock (greenfield/app.js 229-240 and its Escape at 1136; mock.css 291-296): while renaming, the open project's row itself becomes the form: a name field filling the row (min-height 34px) and a primary small Save. "Rename NAME" leaves the menu's acts while the form is open. Enter saves, Escape cancels, an empty name keeps the old one, no visible Cancel. Today: temper-client-view/src/frame.rs rename_form, and temper.css 119-121 .temper-project-rename-form, the old look's rules; remove what this replaces.
2. A dark focus ring on main after a focused control disappears (the rail's divider looks like a black line). Repro: Home, project menu, Rename Hello, Cancel: skein-ui-web page.rs (~360-376) moves focus to Role::Main with tabindex=-1, and Chrome draws focus-visible; #app > main.temper-session has no rule. temper-client-shell-web lib.rs finish can do the same after keyboard use. Wanted: (a) closing the rename form by Save or Escape returns focus to the control that opened it, or the menu's summary, not main; (b) a programmatic focus target (main, its h1, anything given tabindex=-1 only to take focus) draws no outline, in skein's sheet if it belongs there. Real controls keep their focus ring.
