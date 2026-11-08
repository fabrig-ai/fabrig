+++
id = "path-stroke"
part = "stroke"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 20
+++

# No outline on a path's marks and labels

## Goal

The request's task 1 (`temper/docs/plan/look-round-24/inputs/request.md`). Every mark of a path's drawing, a stop's hit area, a star, a code square, main's bar and each SVG label, takes skein's default `stroke: currentColor` (`skein/crates/skein-ui-bundle/skein-ui.css`, about line 25: `[data-drawn] > svg :is(g, line, polyline, path, rect, circle, text) { stroke: currentColor; fill: none; }`), a 1 px ink outline that makes the labels look bold. The request's page has `stroke: none` on them. One rule of temper's, scoped to the path, gives skein's stroke way; skein's rule stays as it is.

Acceptance, by `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways* (its last sentences, written by this cut):

1. **One rule** in `temper/crates/temper-client-shell-web/temper.css`, under `/* ==== ui: chats ==== */`, directly after the comment `/* ask-temper-ways: judgement above the dotted authority boundary, code below. */` (about line 1488), before the path's other rules:

   ```css
   .temper-ways-path[data-drawn] > svg :is(g, line, polyline, path, rect, circle, text) { stroke: none; }
   ```

   Its specificity, (0,2,2), outweighs skein's rule, (0,1,2), whatever the order, and is outweighed by every rule of the path's that sets a stroke, each `#app > .temper-session ...`, (1,2,0) or more: `.temper-ways-base`, `.temper-ways-rise`, `.temper-ways-auth`, `.temper-ways-person`, `.temper-ways-halo`, `.temper-ways-caret` and `.temper-ways-stop:focus-visible .temper-ways-hit`. No other rule changes; `skein-ui.css` is not touched.
2. **Every stroke the path keeps is its own:** read each rule of the path under `ui: chats` and of the flows beside it (`.temper-flow*`, `[data-drawn] > svg .skein-ui-flow-*`): none loses a stroke it sets to the new rule, since the new rule names `.temper-ways-path` alone and a flow's figure has no such class. Say so in the report, rule by rule.
3. **Status:** delete the **Not yet** entry naming `path-stroke` in `temper/docs/design/client/browser.status.md` (the first entry, under 2.5). No other design line.
4. **No test:** the rule is a rendered fact the render check after landing measures; a focused test of the stylesheet's text would pin a selector's spelling, not the cascade. Run the tests that read `temper.css` so that none breaks: `heavy cargo nextest run -p temper-client-shell-web` and `-p temper-client-view` (its look tests read the stylesheet's class names), with their counts in the report.

One commit, with the trailer `Look-request: ask-temper-ways` and the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 1.
- `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways*.
- skein's `skein/docs/design/ui-web.md`, 12 (what `skein-ui.css` gives every mark), read only.

## Owns and may change

Owns the one rule and the status entry. Not `skein-ui.css`, not the path's other rules, not the opening's rules under `ui: settings`, not the view.

Options, each decided:

- **A test reading `temper.css` fails on the new selector** (one that lists selectors or attribute names it allows, say): add what it needs for this selector alone, keeping what it checks for everything else, and name it in the report.
- **Another rule of the path's sets a stroke at a specificity under (0,2,2)** (step 2 finds one): leave it, and raise neither; say which and where in the report, as a finding for temper/ui.

## Steps

1. Read the request's task 1, the Inputs and the path's rules under `ui: chats`; add the rule (acceptance 1).
2. Read every stroke rule (acceptance 2).
3. The status entry (acceptance 3); the runs (acceptance 4); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| A path's marks and labels: no outline (`stroke: none`, the request's) | landed (`temper.css`, `ui: chats`, `.temper-ways-path[data-drawn] > svg :is(...)`) |
| The path's own strokes (line, boundary, ring, halo, caret, a focused stop) | kept: each outweighs the new rule |
| skein's default stroke | stays as it is, outside the path |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): focused tests establish no rendered equality |

## Report

The implementer's report: the rule as written, each stroke rule read and its weight, the runs' counts, and the account as committed.
