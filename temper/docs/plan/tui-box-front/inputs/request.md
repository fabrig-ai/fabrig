# tui-box-front: `temper box` in a terminal, with the TUI in front, the user's look request `tui-box-front`

The user's settled shape, 2026-11-08, on `tui-requests` at `ceb1192d9a`: hand-made frames under `temper/docs/design/client/tui/journeys/box-front/` (01 to 05, with a README saying they're hand-made), and the commit's message, which is the request. This plan's `inputs/` holds a copy (`request-commit.txt`, `box-front-01.txt` to `-05.txt`, `box-front-README.md`). Read them whole first: they are this plan's spec. The box's design is `temper/docs/design/protocol/setup.md`, section 7, and the TUI's is `temper/docs/design/client/README.md`, 6.11 to 6.16 and 7.1. Land it as `temper/docs/design/draft/look-requests.md`, sections 3, 5 and 6 say. **Every commit carries the trailer `Look-request: tui-box-front`,** and the last carries the account. Departures from `temper/docs/design/draft/tui.md` go in `tui.status.md` as the user's decision of 2026-11-08; the draft's text doesn't change.

## What it asks, in short

- **In a terminal, `temper box` runs its services and the TUI in front.** `temper box --no-tui` (the user's name) runs the box as today, and so does a box whose stdin or stdout isn't a terminal. The box's printed block goes to its logs, and a second terminal for `temper` is no longer needed.
- **No starting screen:** the box hands out its sign-in link before temper is up and holds the TUI's connection until it is, as it does the browser's. The welcome is the first frame.
- **Signed in with nothing to read or paste:** the box hands the TUI a one-time secret through an inherited pipe or descriptor at once. Never argv, the environment or a file. It builds on `tui-native-secret`'s local link, which is on main when this starts.
- **01, the welcome:** the printed link drawn whole, never opened by the TUI, what it does, and the browser's welcome steps, none started. The link's secret is redacted in any journey, as today.
- **02, following the browser:** the TUI follows the welcome's steps read-only (signed in, Welcome, Forge, Model, Ready). Setup never runs in the TUI: the forge is chosen and set up in the browser. The step under way is the first step not yet done, as `welcome::decision` computes it, not the step the browser's tab shows (that lives in the tab's address), so a person who clicks back to an earlier step isn't followed.
- **"✓ Signed in as the owner" stays, and the work it needs is this plan's** (the user, 2026-11-08). Today a client can't see that another client signed in: `temper-client-domain`'s welcome reads forges, accounts, the playground and `me`, and jig's `protocol/client.status.md` defers the person's list of sign-ins. Build the smallest fact the TUI needs, that the box's printed link has been used by a browser, as a read the client root watches. It need not be the list of sign-ins. Design it in jig (`protocol/client.md`, section 8) or temper, wherever the printed link's state lives, and name the protocol change at the top of `plan.md`.
- **03, the playground:** the TUI works with it as with any project, and says it's the playground: "playground" in the header and on the quiet line, and the browser banner's words in the header's box.
- **04, quitting:** Ctrl+C while work runs asks first, as a list under the box: "Stop the box? … stops with it.", with "Stop the box" and "Don't quit". With nothing running, it stops the box at once, draining it as the box's first termination signal does today.
- **05, a failing step:** the step on the quiet line ("forge restarting"), and `/log` showing the box's recent log lines. A step failing for good: the box gives up naming the step, and the TUI says so and stays open for `/log`.
- **The place** reads "box" at the default port and "box:PORT" at another. The words are `tui-launchpad-2`'s (`tui-chat-title-below`): call its function if it's on main. If it isn't yet, draw the header's place as today and leave the rule to that plan.
- **Left out:** a box over SSH (the person's tunnel), and every flag but `--no-tui`.

## What isn't the look's

Almost all of it is a feature: the box starting the TUI and handing it a sign-in, the TUI reading the box's steps and log, following the welcome's steps, and quitting stopping the box. Design those in the box's and the TUI's documents, in place. Where the TUI reads the welcome's steps or the box's state, read what the engine or the box already says; a new record for it is this plan's, named in `plan.md`.

## Journeys

Record a TUI journey of the box with the TUI in front, on a fresh fake box, through the welcome to a project or the playground, if the recorder can drive `temper box` as it drives `temper`. If it can't, say why in the account, and leave a status entry.

## Overlaps

- **`temper box` and its block are temper/ui's** (temper's seam 5): keep the box's `--no-tui` and non-terminal behaviour exactly as today, and name every change to the box's code and help in `plan.md`. Slow-tier box stories that read its terminal output are updated and compiled, not run.
- `tui-launchpad-2` is in flight in the TUI's launchpad and chat: expect a catch-up, and keep this plan's views (the welcome, the steps, the playground header, the quit question, `/log`) in files of their own.

## Walk

`walk = false`. The walk follows the landing.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run. A slow-tier test whose expected output the change makes stale (the box's block, a walk's expectations) is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Recording flakes:** a re-record that stops on a moving screen is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait and say so.
- **Generated files** that a change makes stale are that task's to regenerate. Help pages a change grows past their bound are that task's to shorten, losing no facts.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
