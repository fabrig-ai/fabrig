# note-parts: a note written in parts

At the user's idea (2026-11-05, through fabrig/qa and fabrig/ui): jig's notes as an LLM's working area, for chats' drafts and for configuring. `note` writes a body whole on every call, and a call's payload on a box is about 10,662 bytes at init's 8,192-token output (`temper/docs/design/protocol/configuration.md`, 13.8). So `notes.body_bytes`' 65,536 never reaches the agent: a draft stops being rewritable at about 10.6 KB, and a 23 KB settings value (finding-330) can't be built at all. There is no supported deployment: formats, protocols and tools change freely.

This trunk is based on `ui-chat-draft`'s, for `note-answer`, `note-room` and `note-scope`. **It lands after `ui-chat-draft`.**

## The change

- **`note` takes a line range,** one optional form beside today's whole body: `first` (1-based) and `remove` (a count of lines, 0 to insert), with `revision`, the revision read, required. The body's text replaces those lines. An append is `first` = lines + 1, `remove` 0. Without `first`, the call is as today. One form only: no named sections, since a settings value isn't Markdown.
- **Refusals an agent can act on:** a stale revision says the current revision; a range past the end says the line count; a result past `notes.body_bytes` says so. Each leaves the note as it was.
- **The answer stays `note-answer`'s:** `lines`, `added`, `removed`, `first`, computed the same way.
- **Stored whole:** a revision stores the resulting body, so a reader, a client and `recall` see no parts.
- **Reading back:** `recall` returns the whole body, which costs input, not output. Add a line range to `recall` only if a walk shows it's needed.
- **The stock instructions** (`temper/crates/temper-service/src/stock/chat.txt`, and the configuring agent's): write a first version whole; change a document by lines; never rewrite it whole past a few KB; a long document is built in parts.

Tool schema, jig's contract and wire, `jig/docs/design/domain/engine.md` (notes, *A body's bound*), `jig/docs/design/draft/notes-memory.md` and its status, and temper's `protocol/configuration.md`, 13.8 say the same.

## The proof

Pure line-range tests beside `line_change`, including a body with no final newline and an empty one; world tests for each refusal; then, through `temper ctl` on a fake box, a scripted model builds a 30 KB note in several calls of under 10 KB each, its pane's read whole, its revisions in order. Name what isn't proved on a real model.

## Not this plan

`propose_settings` naming a note: fabrig/qa's, under finding-330. It needs `first`/`remove` from here. Note kinds and the working scope of `notes-memory.md` stay held.

## How the cut shapes it

Two to four tasks, jig first, the design first. A change to a format, a protocol or a public interface needs no `Question:`: decide, and note it in `plan.md`. hearth uses no jig notes that I know of: check, and tell fabrig/hearth only if it does.

## Size

One cut is at most 45 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
