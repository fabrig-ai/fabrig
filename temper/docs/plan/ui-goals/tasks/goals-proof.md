+++
id = "goals-proof"
part = "goals-proof"
kind = "docs"
needs = ["box-goal"]
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# The proof beside the mock

## Goal

Every page and state of this plan is shown beside the mock, what differs is fixed or named, and the status files say so.

Acceptance, screenshots in `~/src/fabrig/ui-goals/` on your builder, outside any worktree, never committed:

- **Renders beside `goals.html`, at 1440, 1000 and 420 wide:** each gallery state of this plan, each filter, and the switcher open.
- **The hello project's empty list** against the mock's with the project switched.
- **A box walk,** `temper box --fake-backends`, in a browser: a chat's goal appears in the rail and the list, with its report's headline.
- What differs from the mock in layout, type, spacing or colour is fixed in `temper.css` or the view, and a test where a test can hold it. What the plan left out, or the client's own fonts, is named, not fixed.
- `temper/docs/design/client/README.status.md` and `browser.status.md` no longer say Goals is not yet, unless something is left, named by section.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md` (Render proofs), `notes/contracts.md`.
- `temper/docs/design/client/browser.md`, 2.5, 7.
- `temper/docs/plan/ui-goals/inputs/request.md`, How the cut shapes it.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-view/src/look.rs`, its `tests/look.rs`, the gallery's prints and the terminal's goldens where a fix moves them, by regenerating.

Options, each pre-approved:

- A throwaway render script, untracked, gone before the push.
- A difference that takes more than a rule's or a word's change, or belongs to another plan's page, goes in your report under `## Left`, with its screenshot's path.

## Steps

1. **Print,** then drive Chromium: 420 px by CDP's `Emulation.setDeviceMetricsOverride`, and handle `Page.javascriptDialogOpening`.
2. **Compare and fix,** smallest first.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: the builder; the screenshots' directory; a table of each state and width, same or different and why.
