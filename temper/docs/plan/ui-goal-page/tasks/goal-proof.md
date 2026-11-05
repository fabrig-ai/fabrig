+++
id = "goal-proof"
part = "goal-proof"
kind = "feature"
needs = ["goal-browser"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/start_goal.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# The proof, beside the mock

## Goal

For every page and state, a render beside the mock, what differs repaired, and a box walk. The status lines close.

Departure from the implementer's text: a throwaway render script stays untracked and is gone before the push. The screenshots are never committed.

Acceptance:

- **Renders,** at 1440, 1000 and 420 wide, beside the mock's `goal.html` in the same scenario, for each `goal-*` and `start-goal*` state of `temper/docs/design/client/browser.md`, 7.2, and each open state (the disclosure, the dialog, the sheet, the amendment), screenshots in `~/src/fabrig/ui-goal-page/` on your builder.
  Your report names the builder and says, per state, what differs from the mock and why it stands. What differs and doesn't stand is repaired here, in `temper.css`, `look.rs`, or the view's words, with the test it moves.
- **Phone width:** `.acts` keep `.btn.small`'s height, wrapping as drawn; the bands' buttons stack (`.choices`); the side follows the report by `.two`'s wrap, also at 1000 wide.
- **A box walk** on `temper box --fake-backends`, through a real browser: start a goal from Goals' sheet; see planning; then its report and tasks; Pause, Resume; message it; Stop it. If the fake model's script can't show a state, say which: `goal-browser`'s report names them.
- `temper/docs/design/client/README.status.md`'s *3.5, 3.6, 5.8, 5.11, 5.14, 10* line and any `browser.status.md` line left of this plan's are gone, or say exactly what's still lacking.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports none broken.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, Render proofs; `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8, 5.11, 5.14; `browser.md`, 2.5, 7.2.
- `temper/docs/design/client/greenfield/`; `temper/docs/plan/ui-goals/tasks/goals-proof.md`, how the last proof ran.

## Owns and may change

Owns, as `files` lists them. May change a neighbouring test when a repair moves its words or its look. Options, each pre-approved:

- **Fonts:** the client's own, not the mock's Google link.
- **A difference the design already states** (task rows are links, no "earlier reports", no budget field, the Stop dialog as a dialog, not `confirm()`): it stands; name it once.
- **Not in the mock:** the toast's refusal, `.sheet-error` when refused: compare to the mock's look of its neighbours.
- **A repair past the look:** a behaviour that's wrong gets its test and a fix if it's small; otherwise list it under `## Left`, with its file.
- **Budgets and `$`:** the mock's `$31 of $40` is "31 of 40 spent" here.

## Steps

1. Render the gallery's states and the mock's pages, by CDP, at three widths.
2. Compare, repair, re-render. Remove your script.
3. The box walk.
4. `heavy cargo nextest run -p temper-client-view -p temper-client-gallery`, and the status.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: the builder, the screenshots' path, per state what differs and why it stands, the walk's steps, what the script can't show, and your implementation minutes and check minutes apart.
