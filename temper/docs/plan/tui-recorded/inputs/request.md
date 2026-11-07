# tui-recorded: the TUI signed in by the browser's approval, and its journeys recorded from the start

The spec is `temper/docs/design/draft/tui.md`, section 4.2 (the TUI's side) and section 5, carried out as `temper/docs/design/draft/tui.execution.md` says: the rest of M2, then M3. Read the draft whole, then both entries, which are this plan's input. It designs and builds together and lands whole (`tui.execution.md`, section 2). The TUI's first pass is one of v0.1.0's three critical pieces: this plan carries the priority mark.

On main already: M1's TUI (`temper/docs/design/client/README.md`, 6.11 to 6.16: its command, directory, host and seam, pages, theme and loop) and M2's approval in jig, on the engine, on the web's page and in `temper-client-domain`, whose design includes the TUI's side (`client/README.md`, 7.1). Build to those sections. Where they leave the TUI's side open, design it here.

## Part 1: the TUI's way through the approval (M2's rest)

- **Signed out, the sign-in screen asks for an approval,** through `temper-client-domain`'s native ask, and draws the code and the page's address, as 4.2's frame illustrates. The pasted link stays on the same screen as the other way in.
- **The tab, opened by the TUI's shell** (`xdg-open` on Linux, `open` on macOS) through the host's "a browser opened" ask. The address is made by the TUI from its own parts: the engine's address, the page's fixed path, and the code checked against the code's alphabet. It goes to the opener as one argument. The TUI opens no address the engine sent.
- **No browser to open** (no display, an SSH session, or the opener failing): the TUI prints the address and the code, to open in a browser that reaches the engine.
- **Refused, expired or used:** the TUI says which and offers to ask again. Once taken, it keeps the credential in its directory and draws the first screen.
- **Tests:** in temper's client world, a TUI's approval approved by a second client signed in as a browser is, then the first screen drawn (`temper/docs/design/domain/testing.md`, 6.3). The address the TUI opens is made from its own parts alone, as a table test whose rows include an engine answer carrying a hostile address or code. The opener gets exactly one argument. Each ending's lines. Goldens for the approval's frame beside M1's.

## Part 2: recorded (M3)

- **The TUI's journeys, beside `temper/docs/design/client/journeys.md`:** recorded at the host's seam in a build of its own on a fresh fake box; replayed through the shipped host; printed as frames, as text a row per line and as a page in the theme's colours, walked by the same keys; kept current by a test; recorded again by a command, with the TUI in a pseudo-terminal and the approval given in headless Chromium; the look asked for on the frames, through a theme overlay that stays empty on main, as the browser's `overlay.css` does (`tui.md`, section 5; `look-requests.md`).
- **One journey format or two** (`tui.md`, 8): take the default, sharing what fits, and say so in `tui.status.md`.
- **Redaction and the last scan are the browser's, with the same code** (`journeys.md`, 3.6 and 3.7). They gain the TUI's originals: the approval's secret, the credential the TUI takes, and the bearer every exchange carries. Additions only: what the browser's journeys redact and refuse stays as it is.
- **The first journey:** a fresh fake box, the browser signed in by its printed link, `temper` started, the approval's frame, the approval given, and the first screen. Committed with its README, its text frames and its pages. Recording it again is generation, not a walk: a task runs it.
- **Tests:** as the browser's journeys' (`journeys.md`, section 6), within their cost; the redaction and the scan shared with them, with a failing test first for each TUI original, in each place it can appear.

## Design

Write the TUI's side of signing in where `client/README.md`, 7.1 leaves it, and the TUI's journeys. A document that is the TUI's alone goes in `temper/docs/design/client/tui/` from the start (`temper/docs/design/README.md`, section 6: the client's documents are split by surface). Shared parts, `journeys.md`'s redaction and scan among them, stay where they are. Whether the TUI's journeys get a document of their own is the designer's choice. Then close the lags in `temper/docs/design/draft/tui.status.md` (4.2's TUI side, 5) and set M2's and M3's states in `tui.execution.md`, section 3. Never edit `tui.md`'s own text. Names are the designer's.

## Left out

Everything in `tui.md`, sections 3 and 7. macOS stays cheap: `open` is wired up; nothing more is asked there.

## Overlaps

- The browser's journeys, `temper-client-journeys`, and `journeys.md`'s redaction and scan are shared with whoever owns the web client: keep the change to the TUI's originals and the TUI's own journeys, named in `plan.md`, and leave the browser's journeys and their recordings alone.
- `temper-client-domain`: the smallest change the TUI's flow needs, named in `plan.md`.

## Walk

`walk = false` in the plan. M2's walk and the user's look at the first journey's frames follow the landing, outside the plan.

## For every task of this plan

- **A plan is complete when it lands.** Walks, render checks and throwaway drivers follow landing, outside the plan: don't ask for them, or run them. Recording a journey again is generation, which a task runs.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Recording flakes:** a re-record that stops on a moving screen or page is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are that task's to regenerate. Help pages a change grows past their bound are that task's to shorten, losing no facts.
- **A test's oracle or helper that a change's correct behaviour trips is that task's to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
