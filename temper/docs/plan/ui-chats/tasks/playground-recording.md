+++
id = "playground-recording"
part = "playground-recording"
kind = "docs"
needs = ["tool-lines"]
files = [
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/journeys.status.md",
]
effort = "medium"
minutes = 45
+++

# The playground's recording walked anew

## Goal

`welcome-playground`'s walk now passes `NewChat`: walked anew on a fresh fake box and added, so that the print shows the new page.

Acceptance:

- `temper/docs/design/client/journeys/welcome-playground/` is a new recording, `02.html` `NewChat` in the Demo project, the playground started, its
  banner's one line above its main; its `README.md` says so, in `temper/docs/design/client/journeys.md`, 9's words.
- `heavy cargo run -p temper-client-journeys` reprints every journey and exits 0. `heavy cargo nextest run -p temper-client-journeys` passes.
- The report says which journeys were reprinted and which walked.
- `journeys.status.md`'s entry for `welcome-playground` goes.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`.
- `temper/docs/design/client/journeys.md`, 3.5, 8, 8.1, 9.
- `temper/docs/plan/ui-chats/inputs/request.md`, *Recording*. No `Look-request:` trailers.

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- Try `again` first: `heavy cargo run -p temper-client-journeys -- again welcome-playground`. Its acts may hold, since "Try the playground" is still the
  one act. If it refuses, walk it by hand: a fresh box, Chromium through CDP (a throwaway script, gone before the push, handling
  `Page.javascriptDialogOpening`), Alt+Shift+S to save, then `-- add welcome-playground FILE`.
- A box or Chromium that won't start on this builder is `Infra:`, in the report's first line.
- `project-landing` and `welcome-set-up` are reprinted only. If either is stale, say so and leave it to the trunk's end check.

## Steps

1. Walk, save, `again` or `add`. Read `02.html` in a render, beside the gallery's `chat-new-first-run-1`.
2. The README, the status.
3. **Check,** above. Remove every throwaway. Push.

## Report

As `notes/common.md`, *Your report*.
