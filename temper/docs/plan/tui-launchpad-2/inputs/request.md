# tui-launchpad-2: the launchpad and a chat, refined from the user's walk on a real box, and the look request `tui-chat-title-below`

The user walked the launchpad on their real box, 2026-11-08 (emacs vterm, 150 by 39, a solarized-dark background), and asked for four changes and one look request. The launchpad and the chat are `temper/docs/design/client/README.md`, 6.13 to 6.16. Departures from `temper/docs/design/draft/tui.md` go in `tui.status.md` as the user's requests of 2026-11-08; the draft's text doesn't change.

## 1. The left-off chats, reached by the arrows

Up and Down do nothing on the launchpad today (6.14, *Nothing to press*). **While the box is empty, they move through the chats under "Where you left off", and Enter opens the selected one.** The selection shows as any list's active item does (its mark and accent). With words in the box, the arrows stay the box's.

## 2. `/effort` on the launchpad

Offer `/effort` wherever `/model` is offered and the chosen model has efforts, as the browser's chat terminal does wherever its picker is. Today `offered` gives it in a chat alone. The launchpad's choice applies to the chat its box starts.

## 3. The person's words read apart from the agent's, on any ground

On the user's dark-solarized terminal, the band under "❯ hello" is the dark theme's fixed `sel` (#2A2825). It reads as black there, and the words on it are in the terminal's default ink, the same as the agent's. Now that the TUI draws on the terminal's own background, the band can't be a fixed colour:
- **Where the terminal answered its background** (OSC 11), the band is that background moved one step toward the ink: a small, fixed step in lightness, toward lighter on a dark ground and darker on a light one.
- **Where it didn't**, there's no band. The person's words are in the dim colour, after an accent "❯", as Claude Code draws its prompts.
- **The active item of a list** keeps its mark and accent; its band follows the same rule.

## 4. A Markdown bullet is "•", never "✦"

The model's "- **How temper works**:" items show as "✦ How temper works:". "✦" is the agent's own mark. A bullet is drawn as "•". Check every list marker the reader draws, nested ones too, and leave numbered lists numbered.

## 5. The look request `tui-chat-title-below`

The user's request on `tui-requests` at `35b3899829`. This plan's `inputs/` holds its message (`request-commit-chat-title-below.txt`) and its change to `launchpad-chat/04` (`chat-title-below-04.diff`). Land it as `temper/docs/design/draft/look-requests.md`, sections 3, 5 and 6 say. **Every commit for it carries the trailer `Look-request: tui-chat-title-below`,** and its last carries the account.

In short: the chat's top line moves into the rule above the box, aligned right, two dashes from the edge, freeing the first row for the conversation. It reads: the project, " / ", the title cut to at most 20 characters at a word with "…", the chat's number, " on ", and the engine's place. The place is "box" for a temper box at its default port (8367), "box:PORT" for a box at another port, and HOST:PORT otherwise. The TUI knows it's on a box from the link file the box writes (and, later, from the box starting it). The message names the states the frame doesn't draw.

## Journeys

`launchpad-chat` is recorded again (generation). Its frames change with items 3 to 5.

## Overlaps

- finding-465's repair (a fast chat losing its opening question) is in flight in `post-landing-probes`, in the TUI's chat. Expect a catch-up.
- `tui-native-secret` is in flight and touches none of the TUI's crates.
- `tui-box-front` (`temper box` with the TUI in front) follows `tui-native-secret` and will draw the place in the box's header. The place's words are this plan's: put them in one function both can call.
- `temper-client-view`'s `shared` module is shared with whoever owns the web client: change it only where the browser's chat terminal should change too, and name each change in `plan.md`.

## Walk

`walk = false`. The walk follows the landing.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run. A slow-tier test whose expected output the change makes stale is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Recording flakes:** a re-record that stops on a moving screen is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait and say so.
- **Generated files** (goldens, prints, help references) that a change makes stale are that task's to regenerate.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
