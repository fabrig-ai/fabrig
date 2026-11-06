# ui-rename-focus: the rules every task names

The request is `temper/docs/plan/ui-rename-focus/inputs/request.md`: read all of it, it is short, and its *For every task of this plan* binds you and is not repeated here. Its causes are a reader's: reproduce before you fix.

## Departures from the implementer's text

- **Design text:** this plan's design lines are written, in the sections your spec cites. Your task closes its **Not yet** entry and the index line your spec names, in its last commit. A fix that makes another design line wrong edits that line and names it in your report. Nothing else under `docs/design/`, but the generated prints.
- **Decide, don't ask,** as the request says. A `Question:` is only for what truly blocks your task.
- **A proof's screenshots** stay outside the repository, in `~/src/fabrig/ui-rename-focus/<your task>/`; your report names them. The render script is throwaway, untracked and gone before the push.
- **A box and Chromium run directly,** outside `heavy`: they build nothing. Every build and test still goes through `heavy`.

## What crosses the two tasks

`focus-target` makes these true, and `rename-row` builds on them with no code of its own:

- **Where a closing form leaves the focus:** in `skein-ui-web`, a removal that takes the focus stops at the nearest remaining `Role::Disclosure`, and `Op::Focus` on a disclosure focuses its `<summary>`. No state and no focus request in temper's domain or view: the form's removal does it. A page that asks the focus for its own field (`new_chat.rs`, `configure.rs`) still takes it, later in the same apply.
- **What draws no ring:** `skein-ui.css` draws no outline on `[tabindex="-1"]` but a menu's item or a tree's. A sheet of temper's that names `:focus-visible` with a higher specificity can still draw one, and `rename-row` looks for that.
- **Why the summary:** the request allows it beside "the control that opened it". A focus request on "Rename NAME" would need one-shot state in the domain and arbitration with `Builder::focus`'s at most one per tree; the removal's rule also serves every other form inside a disclosure.
