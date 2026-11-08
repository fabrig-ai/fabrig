# tui-link-colours: the TUI signs in to a local box with nothing to paste, and draws on the terminal's own colours

Two changes the user asked for, 2026-11-07, after using the TUI's first pass on their laptop in emacs vterm on Linux. They follow `temper/docs/design/draft/tui.md`'s first pass (sections 4 and 5), as built in `temper/docs/design/client/README.md`, 6.11 to 6.16 and 7.1, and `temper/docs/design/client/tui/journeys.md`. **The draft's own text doesn't change.** Each departure from it goes in `temper/docs/design/draft/tui.status.md`, under **Design departs**, as the user's decision of 2026-11-07, in the landing that designs it. This plan designs and builds together and lands whole. It carries the priority mark.

## Part 1: signed in to a local box with nothing to paste

What happened: a second `temper box` fell back from 8367 to 38683, bare `temper` went to 8367, and it refused the pasted link for 38683, as `tui.md`, 4.1 says.

- **The engine keeps its live link at rest:** while the printed link is enabled, the running engine keeps its current printed link (address and secret together) in a private file, 0600 in a 0700 directory. It rewrites the file at each new link, as the startup hears each `Link`, and removes it at exit. The secret is otherwise held only in jig's memory and on the box's stdout (jig's `protocol/client.md`, section 8, *The printed link*). The file never goes in the configuration directory, which holds no secret by design.
- **Where:** under `$XDG_RUNTIME_DIR` on Linux (a tmpfs, never backed up), under the data directory on macOS (macOS stays cheap). Its name is keyed by the deployment's data directory, resolved as `temper box` resolves it: `TEMPER_DATA`, then the XDG default. Flags are left out, by the user's word: the command line's flags are designed with the user later.
- **The TUI finds it:** a fresh `temper` with no kept sign-in, or one whose sign-in has expired, resolves the data directory the same way. If the file is there and the engine at its address answers, the TUI signs in by it with no interaction and draws the first screen. Otherwise it shows the sign-in screen as today: the pasted link, the browser's approval. Address and secret come from the same file, written by the engine that holds the listener. A restart forgets links, and a spent link is dead, so a stale file grants nothing: the TUI falls back, saying nothing alarming.
- **Expired is not signed out:** after a sign-in expires, the TUI signs in again from the file on its own. After an explicit sign-out, here or elsewhere, it never does: only a person's act signs it in again. Design how the TUI tells the two apart from what the engine answers, and keep that in its directory if it must.
- **The link is one-time:** using it for the TUI must not leave the person's browser with no way in. Check what jig does after a printed link is spent, and keep a way in for the browser, the next link printed and kept as today.
- **The box names the TUI's command** beside its printed link, `temper http://HOST:PORT`, for a terminal that finds no file. That block's place and words are the box's: keep the change to one line there.
- **The risk, accepted by the user, 2026-11-07:** a live owner link, one-time and live at most 2 hours, at rest and readable by any process running as the person, the box's own agents among them. Those can already read the TUI's credential, so this adds little. Record it as accepted in the design, with the way back: the file holds the address alone, and the sign-in takes one step.
- **Tests:** the file written 0600 at each new link and removed at exit; its path resolved alike by the engine and the TUI; in the client's joint world (`temper/docs/design/domain/testing.md`, 6.3), a fresh TUI signed in from the file with no key pressed; a stale file's dead link falling back to the sign-in screen; an expired sign-in renewed from the file; a signed-out TUI not renewed; the secret never in a print, a log or a frame. A failing test first for each refusal.

## Part 2: the terminal's own colours

What happened: in emacs vterm with solarized-dark, the TUI paints the theme's `bg`, #191816, over every cell (`client/README.md`, 6.15, the base's background), a dark slab with vterm's ground at its edges. Claude Code leaves the background and body text to the terminal's defaults, and colours only its accents.

- **The base in the terminal's defaults:** SGR 49 for the background and 39 for the text, with the palette kept for the accent, dim, faint and good colours.
- **Defaults to settle, then refine on the frames:**
  - The focus shown by the accent and its marker, with no background of its own, as Claude Code does. Reverse video is the fallback if the marker alone can't show it.
  - Frames and goldens keep "the terminal's default" as a colour of its own. The HTML print draws it with the theme's stand-in.
  - Dark or light from `COLORFGBG`, else an OSC 11 query at start with a short timeout, else dark. The query's answer is never read as a key, and a recording keeps the scheme among its start's facts.
- **Regenerate** the TUI's goldens, and record `sign-in-approved` again (generation, not a walk: a task runs it).

## Design

Write both in `client/README.md`, 6.11 to 6.16 and 7.1, where they stand. Section 6 of `temper/docs/design/README.md` puts the TUI's documents in `client/tui/`, but moving the existing sections is a later cleanup's: extend them in place. Add to `client/tui/journeys.md` what the colours change in the prints. Then put the departures in `tui.status.md`: 4.1 (the local box's file), and 4.4 with section 5 (the terminal's colours, and how frames draw them). Names are the designer's.

## Left out

Flags for any of it. A deployment at a name. Anything in `tui.md`, sections 3 and 7.

## Overlaps

- `temper box`'s printed block is the box owner's: one line beside the link, named in `plan.md`.
- The browser's journeys, `temper-client-journeys`, and `journeys.md`'s redaction and scan are shared with whoever owns the web client: change none of them unless the file's secret needs a new original, and then add only.
- skein's full-screen shell (`skein/docs/design/ui-screen.md`) gains the default colour and, if it's skein's, the OSC 11 query. If an interface in skein's `fail-stop`, `heap`, `io`, `json`, `lib`, `scratch`, `shell`, `sim` or `world` changes, name it at the top of `plan.md`.

## Walk

`walk = false` in the plan. The walk follows the landing, outside the plan.

## For every task of this plan

- **A plan is complete when it lands.** Walks, render checks and throwaway drivers follow landing, outside the plan: don't ask for them, or run them. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Recording flakes:** a re-record that stops on a moving screen or page is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are that task's to regenerate. Help pages a change grows past their bound are that task's to shorten, losing no facts.
- **A test's oracle or helper that a change's correct behaviour trips is that task's to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
