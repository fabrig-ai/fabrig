+++
id = "phone-sweep-gallery"
part = "phone-sweep-gallery"
kind = "feature"
needs = ["phone-sweep-mock"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/task-pages.css",
]
effort = "high"
minutes = 90
+++

# The sweep: every gallery state at 360, 390 and 420

## Goal

Beyond the mock's pages: every gallery state passes the sweep's four checks (`temper/docs/plan/ui-phone/notes/sweep.md`) at 360, 390 and 420. Nothing past the right edge, the last control reachable above the tab bar, every tab and primary button at least 44 tall, an open switcher and an open sheet fully on screen.

Acceptance:

- **The sweep exits clean** over every name in `temper/tools/gallery/names.txt` at the three widths, or each remaining failure is named in your report: the mock draws it so; or `ui-chat-draft` or `ui-away` own it; or it's under `## Left`.
- **The counts before and after,** per check and per width, in your report.
- `heavy cargo nextest run -p temper-client-gallery` passes. The terminal's goldens don't move.

## Inputs

- `temper/docs/plan/ui-phone/notes/sweep.md`, all of it; `temper/docs/plan/ui-phone/notes/common.md`.
- `temper/docs/plan/ui-phone/inputs/request.md`, section 3, *Beyond the mock's pages*.
- `temper/docs/design/client/browser.md`, 2.5.

## Owns and may change

Owns the `files` above.

- **Where a fix goes:** a page's rule under its marker; one that holds for every page at 760 px and under, as the primaries' 44 might, under `/* ==== ui: phone ==== */`.
- **Known already, on paper:** `.temper-main .temper-flow-primary` has a `min-height` of 32.67px on the old connection pages, those outside `.temper-settings` (`temper.css`, *Priority is assigned by the view*). If the sweep confirms it, the phone's rule lifts it to 44; the wide rule stays.
- **Reader files, a line or two each, where the sheet alone can't:** a page's view in `temper/crates/temper-client-view/src/pages/` and `look.rs`, its `tests/look.rs`, and the goldens and prints that follow, regenerated.
- **An old page the mock doesn't draw** (`accounts-*`, `forges-*`, `github-*`, `connect-*`, `adoption*`, `repositories*`, `builders*`, `inbox-*`) gets the sweep's four checks and no other look work.
- **More than a rule's or a small view's change** goes under `## Left`, with the states it covers.

## Steps

1. **The script:** extend what `phone-sweep-mock` left at `~/src/fabrig/ui-phone/sweep/`, or write it from the note.
2. **Run, group by cause, fix,** smallest first; run again whole after each group.
3. **Check:** the implementer's text; `cargo fmt`; commit and push.

## Report

As `notes/common.md`, Your report: the counts before and after, the causes fixed, the failures named.
