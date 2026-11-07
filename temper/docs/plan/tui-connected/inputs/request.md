# tui-connected: the TUI's M1, a full-screen terminal client signed in by the box's pasted link and drawing its first screen live

The spec is `temper/docs/design/draft/tui.md`, sections 4.1, 4.3 and 4.4, carried out as `temper/docs/design/draft/tui.execution.md`, M1 says: read the draft whole, then M1's entry, which is this plan's input. It designs and builds together and lands whole (`tui.execution.md`, section 2). The TUI's first pass is one of v0.1.0's three critical pieces: this plan carries the priority mark.

## In short

- **skein's full-screen shell,** beside its text shell (skein's `ui.md`, section 16; `ui-text.md`): a renderer, step code, that lays skein-ui's tree out on a grid of cells and places regions by their names (`ui.md`, section 8); and a terminal, ordinary Rust, that takes raw keys, the alternate screen, resizes and colour, and leaves the terminal as it found it on every way out, a panic's included.
- **The TUI's view and host on temper's client root:** its pages as skein-ui's tree beside `temper-client-view`, with the same words where the same thing is said; a host with one seam shaped as the browser's (`temper/docs/design/client/journeys.md`, section 2): reports in (the start, a due, a key, a resize, the transport's), what the host asks back (exchanges, writes, a browser opened, closing), and the frame drawn. M3 records and replays at that seam, so shape it for that now, "a browser opened" among the asks though nothing asks it yet.
- **Its theme,** given by the application to the shell: colours, glyphs and borders for its tokens and regions, dark and light, from the browser's chat terminal (`temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*).
- **The command:** bare `temper` runs the TUI when stdin and stdout are a terminal and prints today's usage otherwise; `temper ADDRESS` names the clients' listener, loopback alone; with none, the address last signed in to, else `http://127.0.0.1:8367`. Its private directory under `$XDG_STATE_HOME/temper/tui` (else `~/.local/state/temper/tui`), the TUI's alone. `q` or Ctrl+C closes without signing out.
- **The sign-in screen takes the box's printed link, pasted into its field,** and signs in by it as `temper open` does (`temper/docs/design/client/README.md`, section 7). The link is never an argument. M2 adds the browser's approval to the same screen later; leave room for it, build none of it.
- **The first screen, live,** as 4.3's illustrative frame draws it: connected to the address as the party, its projects from `me`, `q to quit`, and the link's state in the status line. The engine stopped and started again shows in the status line and loses nothing; signed out elsewhere returns to the sign-in screen.

## Design

Write the TUI's part of `temper/docs/design/client/README.md` (its crates in section 2, its host, its terminal, its directory; 6.9's "later full-screen terminal" becomes it) and of `temper/docs/design/protocol/README.md`, section 4, and `temper/docs/help/reference/commands.md` (bare `temper` and `temper ADDRESS`); skein's full-screen shell beside `ui-text.md`, as a document or a section of `ui.md`: the designer's choice. Then close those lags in `temper/docs/design/draft/tui.status.md`, and set M1's state in `tui.execution.md`, section 3. Never edit `tui.md`'s own text.

**Its section 8 defaults hold** (`tui.execution.md`, preamble): bare `temper` as the draft's default says, and say in `tui.status.md` that the default was taken. Names, crate boundaries and whether the full-screen shell is one crate or two are the designer's (`tui.md`, 4.4).

## Tests

As M1's entry lists, each cheap: the renderer's frames for the first screen at two sizes, light and dark, as text goldens; the host's seam in the client's joint world, a TUI signed in by a link drawing `me` (`temper/docs/design/domain/testing.md`, 6.3); the engine stopped and started again under it; the terminal left as found on each way out (a fake terminal records what was written, no real tty). A new case of one behaviour is a row of an existing table test. Nothing opens a real terminal, a browser or a real engine in a focused or fuzzy test.

## Left out

M2's approval (jig's, the web's page, the TUI's flow through it), M3's recording, and everything in `tui.md`, sections 3 and 7. macOS stays cheap: the terminal compiles and basically works there; nothing more is asked.

## Overlaps

- `temper-client-domain` is shared with whoever owns the web client: a change there is its smallest that the TUI needs, named in `plan.md`. `tui-approval` is cut beside this plan, adding the approval to jig, the web's page and `temper-client-domain`: keep this plan off jig's sign-in and off the web's pages.
- A look round of the browser's chat terminal is in flight (`look-round-17`): take the terminal's palette from main as it stands, and expect a catch-up.
- If an interface in skein's `fail-stop`, `heap`, `io`, `json`, `lib`, `scratch`, `shell`, `sim` or `world` changes, name it at the top of `plan.md`.

## Walk

`walk = false` in the plan: M1's walk (`tui.execution.md`, M1) follows the landing, outside the plan.

## For every task of this plan

- **A plan is complete when it lands.** Walks, render checks and throwaway drivers follow landing, outside the plan: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are that task's to regenerate. Help pages a change grows past their bound are that task's to shorten, losing no facts.
- **A test's oracle or helper that a change's correct behaviour trips is that task's to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
