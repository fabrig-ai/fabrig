# ui-rename-focus: the plan

The request: `inputs/request.md`. Two tasks, one after the other, 125 implementer-minutes.

| part, task | what | needs | minutes |
|---|---|---|---|
| `focus-target` | skein: a removal in a disclosure leaves the focus on its summary; a `tabindex="-1"` target draws no ring | none | 50 |
| `rename-row` | temper: the open project's row is the form; Save on an empty or unchanged name closes; the end-to-end proof | `focus-target` | 75 |

**Critical path:** `focus-target`, then `rename-row`. They run in series because `rename-row`'s acceptance is the request's repro, which needs skein's fix, and a missing generic mechanism goes into skein before its consumer.

## What was decided

- **The summary, not "Rename NAME":** the request allows either. See `notes/common.md`, *What crosses the two tasks*. A page that asks the focus for its own field (a new chat, Configure) keeps taking it, as today.
- **Cancel stays in the tree, hidden in the browser:** skein's Escape presses a bound, enabled node naming it from the tree, and a text field takes no `Press` (`skein/crates/skein-ui/src/binding.rs`). The terminal has no Escape of its own (skein's `ui.md`, 6.1), so it keeps showing it. So: no skein change, no new word, and a sheet rule on `[aria-keyshortcuts=Escape]`.
- **Spaces alone are an empty name,** and a name is sent without the spaces at its ends, as the mock's `trim()`.
- **The focus guards:** the form's `b.focus(0)` meets four unguarded `b.focus` calls (`pages/chat.rs`, `document.rs`, `settings.rs`, `ask.rs`), a panic by the builder's at most one per tree. The request doesn't name it; the form opens there, so `rename-row` tests and guards them. This is read from the code, not reproduced: the spec has it red first.

## What differs from the mock, on purpose

Recorded in `temper/docs/design/client/README.status.md`:

- The menu stays open after the form closes; the mock draws it closed. The page part holds a disclosure's fold, and the view has no way to fold one. The switcher's disclosure doesn't fold away, so it stays open until the person folds it.
- The field doesn't select its name on opening (the mock's `i.select()`), and no toast says "Renamed". The request names neither; the first needs a skein mechanism.
- The field's hidden label stays "Rename" where the mock's says "Project name": no one sees it, and a new word isn't worth it here.

## Overlaps with the drafts beside this one

- **`ui-final-walk`:** `fix-look` is merged into its trunk and changed two lines of `temper.css`, far from these rules; it has not touched `frame.rs`. `rename-row` keeps off `tests/suite/main.rs`'s end, where that plan appends, with a file of its own and one `mod` line. `fix-flow` is still open and its globs are `temper-client-domain/src/**` and `temper-client-shell-web/src/**`: `rename-row` edits `pages/switcher.rs` and its test in the first, which could not be avoided, and nothing in the second.
- **Generated:** `temper/docs/design/client/journeys/skein-ui.css` is rewritten by `focus-target`. On a catch-up, reprint it rather than merge.
- **`debt-25`, `box-link-first`:** no file in common.
