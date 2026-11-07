# tui-approval: the TUI's M2 without the TUI's screens, a terminal's sign-in approved by a signed-in browser, in jig, on the engine and on the web's page

The spec is `temper/docs/design/draft/tui.md`, section 4.2, carried out as `temper/docs/design/draft/tui.execution.md`, M2 says: read the draft whole, then M2's entry, which is this plan's input. It designs and builds together and lands whole (`tui.execution.md`, section 2). The TUI's first pass is one of v0.1.0's three critical pieces: this plan carries the priority mark.

**This plan is M2 less the TUI's own flow.** `tui-connected` (M1) builds the TUI beside it, and M2's entry says the TUI's flow alone needs M1. So this plan builds everything the TUI will call, up to and including `temper-client-domain`; the TUI's screens for it follow M1's landing in a plan of their own. Design the whole of 4.2 here, the TUI's side among it, so that the follow-up builds to a design already landed.

## In short

- **jig's third built-in way of signing in** (jig's `client/domain/contract.md`, section 2; `domain/people.md`, section 3; `protocol/client.md`, section 8): a sign-in approved by a signed-in browser, for that browser's own party. Enabled where the printed link is, by configuration and on a loopback listener alone. Asked, approved, refused, expired and taken, as 4.2's flow, steps 1 to 5, says: the ask's answer holds a secret, a short code and an expiry; approving makes nothing; taking with the secret makes the sign-in and draws its credential, handed out once in that answer. An approval never taken expires with nothing made.
- **A browser approves, nothing else:** the approval is sent with the browser's cookie under jig's browser guards (jig's `protocol/browser.md`, 12.2). A bearer client, `temper ctl` among them, is refused, so that no agent driving `temper ctl` as the owner can approve a terminal.
- **Bounds:** a code of a few characters, none that look alike; a few approvals waiting at once per deployment; each spent by its first approval or refusal; the sign-in it makes counts as any does (jig's `protocol/client.md`, section 8, *Full*).
- **On the engine:** temper's configuration enables it where it enables the printed link, `temper box` and `temper init`'s templates among them.
- **The web's approval page,** the web client's (section 8's default): "Sign in a terminal?", the code, when it was asked, "Approve only a code your own terminal shows.", Approve and Refuse. Opening it approves nothing. Its code comes in its address on a box; opened with no code, it asks for the code to be typed, and never lists the approvals that wait. After the take, "Signed in. You can close this tab."; refused, expired or used each say which; the browser signed out says to open the box's printed link first, then the approval's address again (`temper/docs/design/client/browser.md`, 2.3 and 2.4). Its fixed path and the code's place in its address are designed here, since the TUI makes that address itself from the engine's address, the path and the code checked against the code's alphabet.
- **`temper-client-domain`'s half:** asking for an approval, taking it with the secret, and the outcomes, as the TUI will call them, with the secret and the credential never in a print, a log or a document. The browser's approving is the web client's.

## Design

Write jig's third way in the three jig sections above, the web's page in `temper/docs/design/client/README.md` and `client/browser.md`, 2.3 and 2.4, and the TUI's signing in, with its own lines, in the TUI's part of `client/README.md` (`tui-connected` writes that part: add a section of your own beside it, and expect a catch-up). Then close those lags in `temper/docs/design/draft/tui.status.md`, and set M2's state in `tui.execution.md`, section 3, as "jig, the engine and the web landed; the TUI's flow follows". Never edit `tui.md`'s own text.

**Section 8's defaults hold:** the code eight characters, ten minutes; the page the web client's. Say in `tui.status.md` that they were taken. Names are the designer's.

## Tests

As M2's entry lists, less the TUI's flow: in jig's worlds, the approval's ends, a code used twice, a secret never in a print or a log, an approval giving the approving party's sign-in alone, one from a client that isn't a browser refused, one never taken making no sign-in; in temper's client world, a client's approval approved by a second client signed in as a browser is, the first then signed in (`temper/docs/design/domain/testing.md`, 6.3); the browser's page in the gallery's states (`client/browser.md`, section 7). **This is a sign-in, so its tests carry the plan:** a failing test first for each refusal. Test data is fake. A new case of one behaviour is a row of an existing table test.

## Left out

The TUI's flow: opening the tab, drawing the code and the page's address, printing them when no browser opens, the address made from its own parts, its lines for refused, expired or used, and asking again. Also everything in `tui.md`, section 7: a deployment at a name, Google, the code typed there, the web's list of sign-ins.

## Overlaps

- The web's pages and `temper-client-domain` are shared with whoever owns the web client: keep the change to the approval page and the client root's half, named in `plan.md`. `tui-connected` is cut beside this plan, adding the TUI's crates, skein's full-screen shell and a small part of `temper-client-domain`: keep off them.
- `per-wake` is in flight in jig: name any overlap in `plan.md`.
- `temper.css`: put new rules under a marker comment of this plan's, `/* ==== tui: approval ==== */`, beside the sign-in's rules, never at the end of the file. The gallery's `names.txt` and `tests.rs` conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted.

## Walk

`walk = false` in the plan: M2's walk (`tui.execution.md`, M2) follows the TUI's flow, outside the plan.

## For every task of this plan

- **A plan is complete when it lands.** Walks, render checks and throwaway drivers follow landing, outside the plan: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are that task's to regenerate. Help pages a change grows past their bound are that task's to shorten, losing no facts. A wire change that makes a browser journey's recording stale re-records it in that task (`temper-client-journeys again` is generation, not a walk).
- **A test's oracle or helper that a change's correct behaviour trips is that task's to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
