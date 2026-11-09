# tui-box-polish: typing from the first frame, plain words in the box, `/theme` as a list, `/quit`, and a notice area

Five changes the user asked for, 2026-11-09, after a night of `temper box` with the TUI in front (emacs vterm, 150 by 39, a brand-new box). The TUI's design is `temper/docs/design/client/README.md`, 6.11 to 6.17. Departures from `temper/docs/design/draft/tui.md` and `surfaces.md`, 11 go in their status files as the user's requests of 2026-11-09. Drafts' own text doesn't change.

## 1. What's typed in the box shows at once, from the first frame, and stays through the sign-in

Today a key typed on the welcome before the TUI has signed in goes nowhere visible, and is then thrown away. In the user's recording, "a" at 3.6 s was lost; the sign-in landed at 6.6 s; "f" at 10.3 s showed. As `origin/main` reads: while the link is `SignedOut`, `Host::typed` (`temper-client-shell-tui/src/host.rs`) tells `Told::Typed` to the sign-in page's link field, but during the file sign-in the view draws the welcome and its box, whose value is `command_words`. `Tui::set_link` then empties that field when the link goes live (6.14, "sign-in transitions clear the value").
- **Whenever the box is what's drawn, typing goes to the box,** and the sign-in never clears it. The paste field takes keys only when the sign-in page with its field is what's drawn.
- **Enter before the sign-in lands** keeps the words, and the notice area (5) says "Still signing in". Nothing is sent or run until the person presses Enter again once signed in.

## 2. The person's words in the box are the terminal's own ink

"❯ fa" is painted in the accent across the mark, the words and the cursor's cell (6.15's focus paint). **The accent goes on "❯" alone;** the words and the cursor's cell are the terminal's default ink. Lists keep their own active-item rule. The goldens that draw typed text in the box change with it (`foot-commands-80x24-dark.txt` among them).

## 3. `/theme` opens a list: Auto, Light and Dark

Today `/theme` flips silently, and on the terminal's own ground the flip shows as almost nothing.
- **`/theme` opens a command's list, as `/model`'s does.** Its head is "Theme. Auto follows your terminal." Then "Auto" with what it reads now ("light, as your terminal says"), "Light" and "Dark", with the current one selected. The command's description is "Auto, light or dark".
- **Taking one draws in it at once and says so in the notice area:** "Theme: auto, light as your terminal says."
- **Auto is today's start:** `COLORFGBG`, else the terminal's answer to OSC 11, else dark. Where neither says, the list says so: "Auto   dark: your terminal doesn't say". (emacs vterm never answers OSC 11.)
- **The choice is kept in the TUI's small store,** so the next run starts in it, Auto by default.
- The browser terminal's `/theme` is temper/ui's and stays as it is.

## 4. `/quit` closes temper in the TUI

The user typed `/quit` and got "Unknown command". **In the TUI, the command that closes temper is `/quit`,** listed with today's description and its question while work runs. `/exit` still works, unlisted, as the same command. The browser terminal's `/exit` stays. If the commands' words are shared with the browser's terminal (`temper-client-view`'s `shared`), give the TUI its own word, and leave the browser's unchanged.

## 5. A notice area at the right end of the quiet line

One-line notices go to the right end of the quiet line, the row with "● Live", not into the conversation.
- **What goes there:** `/theme`'s (3), "Still signing in" (1), and an unknown command's: "Unknown command /qiut · /help lists them", which today is a note after the conversation.
- **It's the place the box's state already uses** in front ("STEP restarting", "STEP gave up": 6.17), made general. A notice shows over the state for its time, then the state is back.
- **A notice stays 6 seconds,** and a newer one replaces it, timed by the host's dues, as replays need.
- **Notes of more than a line stay in the conversation:** `/help`'s table and `/log`.
- **In a narrow window** the notice goes before the quiet line's own words.

## Tests and journeys

Goldens for the box with typed words, `/theme`'s list (with and without a terminal answer), a notice over the quiet line at two sizes, and the welcome with words typed before the sign-in. A host test of keys typed during the file sign-in, kept and drawn, with Enter before it lands. The store keeping the theme across runs. Journeys whose frames change are recorded again (generation).

## Overlaps

- temper/migration's F1 goal changes `host.rs`'s local sign-in on "not ready". finding-469's repair is in fab. Expect catch-ups in `host.rs`.
- The commands' shared words in `temper-client-view` are shared with temper/ui: change only the TUI's, named in `plan.md`.

## Walk

`walk = false`.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, or a tier's CPU figure. A slow-tier test whose expected output the change makes stale is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit just outside the file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** that a change makes stale are that task's to regenerate.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
