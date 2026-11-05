+++
id = "chat-draft-journey"
part = "chat-draft-journey"
kind = "feature"
needs = ["rail-menu", "fake-document"]
files = [
  "temper/docs/design/client/journeys/chat-draft/**",
  "temper/docs/design/client/journeys.status.md",
  "temper/tools/journeys/src/**",
]
effort = "high"
minutes = 80
+++

# The journey `chat-draft`, recorded

## Goal

Walk a chat with a document on a fresh fake box, and keep the recording: `temper/docs/design/client/journeys.md`, 9, its row `chat-draft`.

Acceptance:

- The walk, on a fresh `temper box --fake-backends` built by `temper/scripts/bundle.sh --loopback --record`: a chat started, "+ Draft" pressed; two turns of the fake model (`fake-document`'s cues: the second by a message saying "Add what happens to a running goal."); Split; Draft; "Start goal…", started; the goal's page. Marked with Alt+Shift+M at each, saved with Alt+Shift+S.
- `heavy cargo run -p temper-client-journeys -- add chat-draft FILE`, whose first replay proves it; `journeys/chat-draft/README.md`, `recording`, and its `NN.html`.
- `journeys.status.md`'s `chat-draft` line goes.
- **If anything stops it, say what, and end with `Question:`** rather than working round it, as the request says: what stops it, the step, what you tried, your recommendation. Commit what you have first, and say so.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *Design and the cut*, *Recording*.
- `temper/docs/design/client/journeys.md`, 3.5, 8, 9.
- `temper/crates/temper/tests/fake_backends/walk/chat_document.rs`, for the numbers its scripts use.
- `temper/docs/design/client/journeys/project-landing/README.md`, on a walk's driver.

## Owns and may change

Owns, as `files` lists them.

May also change `temper/docs/design/client/journeys.md`, section 9's row, a word that the walk teaches.

Options, each pre-approved:

- **The driver:** none is kept (`journeys.status.md`), so a throwaway CDP driver, untracked and gone before the push, walks it in Chromium, handling `Page.javascriptDialogOpening` and, at 420 pixels, `Emulation.setDeviceMetricsOverride`.
- **A stale recording of another journey:** `again NAME` if its acts hold; a walked anew one is another's, and goes in `## Left`.
- **`named`:** if `tools/journeys`' tests hold a fixed list of journeys, add this one.
- **The fake's literal numbers** (`fake-document`) hold only in a fresh box's first chat: start from a fresh box.

## Steps

1. Build and walk it; add it; read the prints.
2. `heavy cargo nextest run -p temper-client-journeys`.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, naming the builder.

## Answers

2026-11-05 (fabrig/ui). Yes: the walk turns on Settings, How work runs, 'Chats can write drafts' first, as part of the recording; stock defaults stay. That setting decides + Draft (ruling 8, coord/ui/report/rulings.md).

2026-11-05 (fabrig/ui). Addition, yours now: the recording goes the whole way: turn on 'Chats can write drafts', then + Draft, two document turns, Split, Draft, Start goal…, the one sheet, the goal page and its Draft button back to the chat. If the fake box's model has no draft-writing script (temper-fake's backends.rs model_document; fake-document put two in temper-init's template), add the scripts there; those files are yours. Stock defaults stay. Don't stop for this.
