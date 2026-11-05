+++
id = "needs-proof"
part = "needs-proof"
kind = "docs"
needs = ["home-card", "journey-again", "stock-sentence"]
files = [
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 90
+++

# The proof: rendered against the mock, and one live walk

## Goal

Show that the client looks and works as the mock does on Needs you and Home's needs: each page rendered
against the mock's at three widths, its open states, and one live walk in which a goal hears the words of
the choice a person pressed on Home. Fix what the comparison finds, and close the design's status.

Acceptance:

- **Rendered:** each gallery state of `needs-*`, `home-morning-card`, `home-answered`, `home-one-need`
  against the same state of `needs.html` and `home.html`, at 1440, 1000 and 420, plus the open states (saw
  open, reply open, toast showing), compared as `temper/docs/design/draft/look-requests.md`, 4 does:
  each element's computed style and box, then the screenshots for what styles miss. Screenshots under
  `~/src/fabrig/ui-needs/` on your builder. No `Look-request:` trailers.
- **Each difference is a row** in your report: fixed, or kept with its reason. Expected, kept: the fonts'
  source; "Work carries on" for the mock's "Goals carry on"; no `$` in money; `.dots` as a figure, not
  `role=img`; the mock's scenario details the client isn't given (`README.md`, 11).
- **The walk,** with `temper-fake`: a scripted goal asks its requester in the line form; the person's
  browser shows Home's card with its choices; the person presses the recommended one; the toast and its
  "Undo" show, and the answer goes at 5 seconds; the goal's next turn holds those words. Saved as
  screenshots and the log lines that show it, and the report says which builder.
- **An Undo in the same walk:** a second question, answered, undone inside 5 seconds: nothing reaches the
  goal, and the card is back.
- **Status:** each 3.5 entry this plan wrote in `README.status.md`, 7.2's in `browser.status.md`, and 8.2's
  in `agent.status.md`, is gone if built, or says what is left. Each file's date is today's.
- **No `Inbox` statement left wrong:** read `README.md`, `browser.md`, `journeys.md`, `mockups/STATUS.md`
  by `rg -n 'Inbox|inbox'`, for any that says the browser's page is a list with filters. The cut changed
  those it found; a further one is corrected here, and named in the report.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, *Render proofs*.
- `temper/docs/design/draft/look-requests.md`, 4.
- `temper/docs/design/client/README.md`, 3.5 and 5.3; `temper/docs/design/domain/agent.md`, 8.2.
- `temper/docs/design/domain/testing.md`, on `temper-fake` and its scripted turns;
  `temper/crates/temper-fake/src/tests/demo.rs`; `temper/crates/temper/tests/fake_backends/walk/`.

## Owns and may change

Owns `files`. May also change, for a difference the comparison finds:
`temper/crates/temper-client-shell-web/temper.css`, `temper/crates/temper-client-view/src/pages/`
`{ask,needs,home}.rs`, `src/look.rs`, `src/words.rs`, the gallery's rows and fixtures, with their tests; a
statement of `Inbox` in the design files named above.

Options, each with its default:

- The render and walk scripts are throwaway: untracked, gone before the push.
- If `temper box --fake-backends` gives no way to script a goal's turn in the line form, drive it as
  `temper/tests/one-process` or `temper/tests/client` plays a story's scripted turns, with the browser's
  acts sent through the same public events, and say so. The goal hearing the words is what counts.
- If the walk stops on the sign-in fault `temper/docs/design/client/journeys.status.md` names, start from
  the playground's model, or a key, and say so.
- A difference that would need a new fact of a protocol: keep it, and name it in the report.

## Steps

1. Build the bundle and the gallery. Render the mock's and the client's states by CDP, `Emulation.
   setDeviceMetricsOverride` for 420, handling `Page.javascriptDialogOpening`.
2. Compare, fix, re-render what you fixed, and write the table.
3. The walk, then its Undo.
4. Status and the design's statements; the citations check; `heavy cargo nextest run -p
   temper-client-view --lib` and `-p temper-client-gallery` if the view moved. fmt and clippy for what moved.
   Commit and push.

## Report

As the implementer's text asks, with: the builder; the screenshots' directory; the table of differences;
the walk's log lines, with the words the goal heard.
