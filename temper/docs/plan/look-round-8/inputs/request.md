# look-round-8: "Configure temper" shows temper's own way

At the user's word (2026-11-06, through the director): the "At scale, as temper builds itself" card and its reference land after `ui-settings`, which is on main, with the figures left as placeholders for the user. Land the look request `configure-temper-own-way` (`c980127128`, on `origin/look-requests`), released: its `Look-hold` no longer applies. The design is `temper/docs/design/draft/own-way.md`.

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Every commit carries `Look-request: configure-temper-own-way` directly above `Co-Authored-By:`.

## Decided (fabrig/ui, taking own-way.md's open questions)

- **The figures are placeholders:** section 6 of the reference shows its three boxes, Merges, Agent runs and Workers, each with "—", and under them "Figures to come." There is no "live figures" link, and no address of any machine: public code names none. The user fills these later. Do not use the request's numbers.
- **Where the reference lives:** a stock Markdown file in the binary, as the stock templates are (`own-way.md`, 3). Not a seeded note.
- **The pane:** the reference opens in the same document pane a chat's draft uses (`ui-chat-draft`, on main), in Split, with no chat made. On a phone, Draft.
- **The picture:** `skein/docs/design/draft/diagrams.md` isn't built and isn't planned. Draw the picture as the flow's words, an ordered list of the six steps with their captions, in the reference's own Markdown. Don't build flows. Say so in `own-way.status.md`.
- **The table's rows:** each row is a setting, or says "later" in its "Yours to set" cell. Workers and their slots are the deployment's root configuration: its cell says so ("The deployment's configuration") rather than promising a project setting. Check each row against `reference-code-production.md`, section 14 and `protocol/configuration.md`.
- **"Start from it":** the configuring chat with that first message, as a starter does. The configuring agent reads the same stock reference (`stock/configuring.txt`). "How PROJECT works" keeping temper's own values beside Hello's is the agent's words, not a new view.

## How the cut shapes it

Two to four tasks, the design first; then the card; then the pane and reference; then the agent's words. Screenshots beside the request's pages at 1440, 1000 and 420 wide in `~/src/fabrig/look-round-8/` on the builder, outside any worktree, the report saying which builder. The page still shows whole at 1366 by 680. New rules go under `/* ==== ui: settings ==== */`. A fresh project's Configure temper keeps working with Goals and drafts hidden.

## Size

One cut is at most 30 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/own-way.md`
- `temper/docs/design/draft/own-way.status.md`
- `temper/docs/design/draft/look-requests.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
