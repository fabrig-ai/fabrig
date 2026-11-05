+++
id = "phone-sweep-mock"
part = "phone-sweep-mock"
kind = "feature"
needs = ["phone-chat"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 75
+++

# The sweep: every page the mock draws, against its phone rules

## Goal

For every page the mock draws, at 390 and 420, its phone rules landed. The request's section 3's table, each row measured on the client and on the mock's own page, each miss fixed in that page's own place. A page that doesn't exist is reported, not built.

Acceptance:

- **A table in your report:** one line per row and width, the client's measure beside the mock's. Measure the mock too, rather than trust the row: where the request's number and the mock's differ, the mock wins and you say so.
- **Every row is equal, or named:** by a look request (name it), by a plan beside this one (not there yet), or under `## Left`.
- `heavy cargo nextest run -p temper-client-gallery` passes. The terminal's goldens don't move.

## The rows and where to measure them

The gallery's states, all in `temper/tools/gallery/names.txt`:

| Mock's rule | Client's states |
|---|---|
| `.tabbar`: Goals + Chats with goals on, else Chats + Settings; a Needs count in signal colour | `goals-under-way`, `settings-goals-off`, `home-one-need`; the links' rule on every framed state |
| `.phonebar`: `position: sticky`, `z-index: 20`, stays at the top on scroll | `home-at-work`, `goals-more`, `settings`, scrolled. `.away-chip`: `ui-away`'s, not there yet unless on your base |
| `.needs-full` hidden, `.needs-cta`, `.go` 48 tall, "Answer, one at a time" or "Answer"; `.hide-narrow` | `home-one-need`, `home-needs-several` |
| `.ask { padding: 18px }`, `.choices` stacked, `.btn` 48 tall, `.quiet` 44 | `needs-one`, `needs-proposal`, `needs-reply-open`; Home's, if a Home state draws one |
| `.flow-q { font-size: 23px }`; `.flow .choices .btn` 54 tall; `.pair` | `needs-one`, `needs-funds`, `needs-held`, `needs-plain-words` |
| `.goal-row { padding: 12px 0 }`, `.line` 15px; `.goal-head h1` 23px | `goals-under-way`, `goals-more`, `goal-running` |
| `.two { gap: 32px }`; `.col, .col.wide { gap: 32px }`; `.intro { gap: 10px }` | `home-at-work`, `home-morning`, `goal-running`, `goal-landed` |
| `.decision { padding: 12px 0 }`, `.why` hidden, `.late` 8em | Home's decided-for-you: `ui-away`'s, not there yet unless on your base; the sheet's rule stays |
| `.setting` one column | `settings`, `settings-goals`, `settings-refused` |
| `.choices-grid, .fields` one column; `.first-project-row` stacked, button 48 tall; `.choice, .panel, .away-note { padding: 18px }` | `welcome-forge-choose`, `welcome-model-choose`, `welcome-forge-details`, `welcome-ready-none`, `welcome-ready-copies` |

The client's classes aren't the mock's: read the state's print and the view for what draws each rule (`temper-client-view/src/look.rs`, `NAMES`; `temper.css`, each page's marker). The mock's pages, for its side: `home.html`, `needs.html`, `goals.html`, `goal.html`, `chats.html`, `changes.html`, `settings.html`, `welcome.html`, with its scenarios (`notes/sweep.md`, The driver).

## Inputs

- `temper/docs/plan/ui-phone/inputs/request.md`, section 3 and Decided.
- `temper/docs/design/client/browser.md`, 2.5.
- `temper/docs/plan/ui-phone/notes/common.md`, `temper/docs/plan/ui-phone/notes/sweep.md`.

## Owns and may change

Owns the `files` above: each fix under the page's own marker.

- **Reader files, a line or two each, where the sheet alone can't:** the page's view in `temper/crates/temper-client-view/src/pages/` and `look.rs`, its `tests/look.rs`, and the goldens and prints that follow, regenerated. Say why in your report.
- **"The welcome's whole narrow set" is measured equal already:** a row of it measures only to confirm. Fix it only if it's since moved.
- **More than a rule's or a small view's change** goes under `## Left`, with the row and the screenshot's path.
- **A throwaway table in the script** stays outside the tree, with the sweep.

## Steps

1. **The script,** `~/src/fabrig/ui-phone/sweep/`, with the table's rows as its data.
2. **Measure,** then **fix,** smallest first, and measure again.
3. **Check:** the implementer's text; `cargo fmt`; commit and push. If nothing missed, push nothing and say so in the first line.

## Report

As `notes/common.md`, Your report: the table, the builder, the script's path.
