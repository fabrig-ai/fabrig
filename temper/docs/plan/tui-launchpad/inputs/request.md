# tui-launchpad: the user's look request `tui-launchpad`, the TUI's launchpad, its commands and a chat

The user's first request for the TUI's look, on their branch `tui-requests` at `9b3942a123`, one commit on `main`: hand-made frames under `temper/docs/design/client/tui/journeys/launchpad/` (01 to 04, with a README saying they're hand-made), and the commit's message, which is the request (`temper/docs/design/draft/look-requests.md`, 2). Read the commit's message and frames whole first: this plan's `inputs/` holds a copy (`request-commit.txt`, `launchpad-01.txt` to `-04.txt`, `launchpad-README.md`), and the commit itself is `git show 9b3942a123` from `origin/tui-requests`, its `.html` pages among it. They are this plan's spec, with `temper/docs/design/draft/tui.md`, section 3 the direction they draw. Land it as `look-requests.md`, sections 3, 5 and 6 say, on the TUI: the frames' text into the TUI's view and words, any paint into its theme. **Every commit of the landing carries the trailer `Look-request: tui-launchpad`,** and the last carries the account (`look-requests.md`, 5).

## What it asks, in short

- **01, the launchpad,** in place of the first screen's two lines: a framed header with the project and the deployment's address; the status agent's brief in words, with who wrote it and when; where you left off; what needs you; the composer, "❯ Ask temper"; and the quiet line: "/ for commands", what needs you, the model and effort, and the link's state.
- **02, `/` typed:** the commands the launchpad offers, in the browser terminal's order and words, the first selected, in the quiet line's place. `/exit` closes temper. `/font` is the browser's alone.
- **03, a question asked:** a new chat, its answer still growing, read as `temper/docs/design/draft/chat-reading.md` says: a run of calls as one line in words with its time, under "▸"; the answer only growing, whole, its blocks drawn once, its end a small block after the last word; numbered lists numbered; the running line "✦ working" with no clock.
- **04, the same chat later:** the answer committed in place without moving, the person's reply, and the agent's confirmation under the box as a list: its question as the head, the recommended choice first and selected, "Tell it something else" last.
- **The states the message names and the frames don't show** (the dots before the first word, a call running, no project, nothing needing you, offline or signed out, the light theme, a narrow terminal), each as the message says.
- **What it replaces:** the first screen's two lines and "q to quit". Once the composer has the focus, `q` types, and Ctrl+C or `/exit` closes.

**The user asked that the chat render the stream as the web client's fixes say:** hold the build to `chat-reading.md` and to the browser's chat terminal (`temper/docs/design/client/README.md`, 3.5, *A chat's terminal*): its glyphs ❯ ✦ ▸, its running line, its commands' words, its quiet line, and a confirmation as a list under the box. Where the TUI and the terminal draw the same thing, they draw it alike (`tui.md`, 2).

## What it builds on

- **The composer is the TUI's way into Ask temper,** temper's own AI, which the web gained in `ui-ask-m1` (`temper/docs/design/draft/ask-temper.md`; `temper-client-domain`'s `Address::Ask`). A question typed there opens an Ask temper chat in the project, as the web's Ask temper does. Its views of ways (`ask-temper.md`, 6.2) come later, from another plan.
- **Engine facts the web already reads** are read the same way through the shared client root: the status agent's brief (the project's facts, `temper/docs/design/protocol/store.md`'s `ProjectFacts`), what needs the person, the chats, the chat's model and effort. Draw them; don't derive them again in the TUI.
- **Where you left off:** if the client root already holds what it names (the person's most recent chat or goal, and what it waits on), draw it from that. If it needs a fact the engine doesn't keep, split it off as `look-requests.md`, 6 says, with its account row and a status entry, and leave the section out of the launchpad until it exists. Don't add an engine record for it in this plan.
- **The TUI's design** is `client/README.md`, 6.11 to 6.16 and 7.1, and `client/tui/journeys.md`. Write the launchpad, the commands and the chat there, in place. Put the departures from `tui.md`, 4.3's first screen in `temper/docs/design/draft/tui.status.md`, as the user's request of 2026-11-08. The draft's own text doesn't change.

## Journeys

The hand-made frames stay the request's: no reprint will equal them. The committed `sign-in-approved` ends on the first screen, which this replaces, so record it again (generation). If a fake box can reach a chat, add a TUI journey of the launchpad and a chat beside it, recorded by `again`. If it can't, say so in the account.

## Left out

Ask temper's views of ways. A deployment at a name. Flags. The side pane and drafting (`tui.md`, 3).

## Overlaps

- **`temper-client-domain`** is shared with whoever owns the web client: the smallest change the TUI needs, named in `plan.md`. Ask temper's own plans (`ui-ask-drop`, `ui-ask-m3a`) are in flight or next beside this one: keep off their files.
- This plan starts from main after `tui-link-colours` has landed, so the TUI's theme already draws on the terminal's own colours. Draw the frames' paints in that theme.

## Walk

`walk = false`. The render check and a walk follow the landing.

## For every task of this plan

- **A plan is complete when it lands.** Walks, render checks and throwaway drivers follow landing, outside the plan: don't ask for them, or run them. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Recording flakes:** a re-record that stops on a moving screen or page is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are that task's to regenerate. Help pages a change grows past their bound are that task's to shorten, losing no facts.
- **A test's oracle or helper that a change's correct behaviour trips is that task's to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
