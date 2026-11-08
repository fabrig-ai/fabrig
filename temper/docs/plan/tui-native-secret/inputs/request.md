# tui-native-secret: the link file holds a one-time secret of its own, so the TUI's sign-in never spends the box's printed link

The user's pick, 2026-11-08: today, a `temper` started before the person opens the box's printed link signs in through the link file and spends that very link (`tui-link-colours`; `temper/docs/design/client/README.md`, 6.12 and 6.14). The browser then gets "used", the box prints again, and the person goes back to the terminal for the new link, every time. The user weighed a reusable link that never expires, and no link at all on loopback, and set both aside: any page the browser visits can reach 127.0.0.1, and so can other accounts on the machine. They picked this instead. This plan carries the priority mark, as a repair of the first pass's sign-in.

## The change

- **Two one-time secrets live at once:** the printed link's, as today, and a second one for native clients on the machine, kept only in the engine's link file. Both are the same kind of secret: one-time, for the deployment's owner, with the printed link's lifetime. Spending one leaves the other live.
- **The file holds the file's own secret, never the printed link's.** Everything else about the file stays as built: 0600 in a 0700 directory, where it lives, rewritten at each new secret, removed at exit. When the TUI spends the file's secret, the engine draws the next and rewrites the file, so a second `temper` signs in the same way. The printed link isn't touched, reprinted or spent by the TUI.
- **jig's "one link is live at a time"** (jig's `protocol/client.md`, section 8, *The printed link*) becomes the printed link's plus the file's. Write that in jig's design, with the bounds: at most those two live, each spent once, each expiring as the printed link does. A restart forgets both.
- **The TUI's side doesn't change** beyond reading the file's secret where it read the link: the `sign-in` note, expired against signed out, and the fallback to the sign-in screen all stay.
- **No new exposure:** the file's secret is as private as the file. The printed link's exposure is today's.

## Tests

In jig's worlds: two secrets live together, each spent once, spending one leaves the other, a restart forgets both, and the file's secret never in a print or a log. In temper's client world: a fresh TUI signs in from the file and the printed link still signs a browser in afterwards, in either order. The file's contents are the file's secret alone. A failing test first for each.

## Design and status

jig's `protocol/client.md`, section 8; `temper/docs/design/client/README.md`, 6.12 and 6.14; `temper/docs/design/protocol/setup.md`, 7.2 if the box's block says anything of it. Departures from `temper/docs/design/draft/tui.md` go in `tui.status.md` as the user's decision of 2026-11-08; the draft's text doesn't change.

## Overlaps

finding-463's repair (the no-project launchpad's box) is in flight in `post-landing-probes`, in `temper-client-shell-tui`'s `host.rs` and `temper-client-tui`'s state and foot: keep off them. `tui-box-starts` follows this plan and builds on it.

## Walk

`walk = false`. The walk follows the landing.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run. A slow-tier test whose expected output the change makes stale (the box's block, a walk's expectations) is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** that a change makes stale are that task's to regenerate.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
