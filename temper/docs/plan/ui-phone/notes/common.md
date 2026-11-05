# ui-phone: the rules every task names

The request is `temper/docs/plan/ui-phone/inputs/request.md`; read its section for your task. The design this plan cut is `temper/docs/design/client/browser.md`, 2.5 (*The page covers the display*) and skein's `skein/docs/design/ui-web.md`, 4.3. The target is `temper/docs/design/client/greenfield/`: `mock.css`, its last `@media (max-width: 760px)` block for a phone; `app.js`, which draws each page (`chatPage()`, `phonePage()`). Where this note and the implementer's text differ, this note wins.

## Decide, don't ask

The user is asleep and takes the plan's judgement. A stored format, a protocol or a public interface may change freely, with no compatibility and no `Question:`. A small edit your task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in your report, not parked. A `Question:` is only for what truly blocks the task.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit the `docs/design/` status lines its spec names as it closes them, and a design line a small edit needs; say which in your report. A draft's own text stays as it is.
- **`docs/help/` and generated files:** `docs/help/reference/*`, prints, goldens and counted baselines your change makes stale are yours to regenerate in the same task. A help page a change grows past its bound is yours to shorten, losing no fact and never raising the bound.
- **A throwaway render script** is allowed in any task, untracked and gone before the push; the sweep's lives outside any worktree (`notes/sweep.md`).
- **Commits carry no `Look-request:` trailer.**

## What stays as it is

- **Landed and measured equal, don't touch:** the top bar (padding `14px 16px 10px 20px`, the star, 17px name, menu at least 220 wide), four tabs at 12.5px and 48 tall, `.main`'s padding `8px 20px 112px`, headlines 25px, ledes 15.5px, the welcome's whole narrow set.
- **A look request already made wins over the mock.** `temper-session-foot`'s `8px 16px 14px` at 760 px and under is the look request `configure-temper-name`'s (commit `70e66541af`), where the mock's `.convo-foot` says `8px 12px 14px`; it stays.
- **Not built:** `phone.html`, `mock-toggle`, `mock-panel`. No push notifications.
- **No new breakpoint:** 760, as the mock has it. Existing 600, 430 and 850 rules stay unless they fight the mock; one that does is changed where it stands and named in your report.
- **Others' rules:** the tab bar's rule is `ui-goals`' and `ui-settings`', the Away chip `ui-away`'s, the CTA and the project's count `ui-needs`', the chat's frame `ui-chats`'. This plan checks them and fixes only a miss.
- **`ui-chat-draft` and `ui-away` run beside this plan.** Their pages keep their own phone rules: a chat's draft pane (`.seg`, `.doc`), the goal sheet from a chat, Chats' Drafts, the rail's menu; the Away chip, Home's decided-for-you, Settings' Away. Don't touch them. What of theirs isn't on your base yet is named "not there yet" in your report, not built.
- **A page the mock draws that doesn't exist** is reported, not built.

## `temper.css`

- A rule a page's phone look lacks goes in that page's own place, under its `/* ==== ui: AREA ==== */` marker. A rule of the phone's own goes under `/* ==== ui: phone ==== */`. Never at the end of the file.
- Remove a replaced rule where it stands, rather than overriding it.
- **The terminal's goldens don't move:** a browser-only rule doesn't reach them. If one moves, you changed the view: say why, or undo it.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`:** on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.

## Tests

None new but what a fix needs. The sweep is the check (`notes/sweep.md`). Run the focused tests of what you changed: for a stylesheet, `heavy cargo nextest run -p temper-client-gallery`, and `-p temper-client-view` if you touched the view.

## Recording flakes

If a change stales a journey, record it again (`temper/docs/design/client/journeys.md`, 8.1) in that task. A re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`: use CDP's `Emulation.setDeviceMetricsOverride`, or the media queries lie. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver. The mock's Google Fonts link needs the network; the client's own fonts are what's compared, and a difference that is only the fallback font isn't one. Screenshots go in `~/src/fabrig/ui-phone/` on your builder, outside any worktree, never committed; your report names the builder.

## Your report

As the implementer's text, with: the builder; the status lines you closed; each small edit outside your file list; the options you took; what you checked and found equal, in a line per check, so the proof task and fabrig/ui can read what was measured.
