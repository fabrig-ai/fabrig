# ui-public-2: public reading's spend, look and sense

What temper/qa's `rung-public` walk found that's temper/ui's to fix. Public reading stays off, and titles aren't this plan's. In jig, then temper: two tasks, as the request asks.

## Parts and tasks

| Part | Task | Needs | What it makes true |
|---|---|---|---|
| `public-records` | `public-records` | | jig's public line has a goal's spend alone and no budget, and an ended task's phase says how it ended |
| `public-page` | `public-page` | `public-records` | one plain column in the client's look; "landed", "failed", "ended"; seven plain digits; empty pages say so; no forge link or address |

Critical path: `public-records`, then `public-page`.

## What the request left to find

- **Can the index know a repository is public? No, so the links go.** `Reached.private` is read in the list of what can be adopted, and no repository row keeps it. Keeping it would mean a new stored fact that goes stale when a repository changes visibility. The request's last branch applies: drop the links. `temper/docs/design/client/forge.md`, section 10, keeps the question open.
- **Where a commit is widened.** Forges' ids sit zero-padded in 32 bytes (`api::Commit`), which is the contract. The widening is the public view's `hex`, which printed all 64 digits into a link. With no link left, the view prints seven digits and nothing else prints a commit whole there.
- **What "landed" can rest on.** jig's record can say done, failed or cancelled, and names no landing. The signed-in Goals page already reads done as landed, so a goal's row does the same, with "ended" for a cancelled one. A task's row reads "done": a task isn't a landing.

## Decisions

Each is fabrig/ui's under the request's *Decide, don't ask*.

- **The forge addresses leave the records too,** not only the links: a repository's, a pull's and a run's. The request says "drop the links". A field no page reads would still hand a private repository's owner and name to anyone reading the record. This goes a step past the request's words, and `first-release.status.md` says so.
- **Public interfaces changed in place.** jig's and temper's `public-v1` keep their names and family versions: `PublicLine` loses `budget`, `spent` becomes optional, `PublicPhase::ended` goes, and temper's four records lose their addresses. Nothing stores them.
- **An ended row says how long ago** ("landed · 2 hours ago"), where the request says "for how long". "Landed for 2 hours" wouldn't read as sense to a stranger.
- **The executor stays on the task page's own row** and leaves the board, whose row the request spells out. So its words and the record's field keep a reader.
- **Priority is no longer drawn.** It still orders the board, and the record keeps it.
- **"No public projects yet."** applies the request's "empty pages say so" to the projects page.
- **A landing's member shows one commit:** landed, else candidate. Three nameless hashes meant nothing to a stranger. The record keeps `base`, which is harmless, though no page reads it now.
- **Lines' words get visible separators** across the public pages, not only on the change's line: "CI at its headPassed" was the same defect as "the forge789ada...".

## Overlaps

Task boundaries can't avoid these. All are additive, and each is named in `notes/common.md` or the spec:

- `temper/crates/temper-client-view/src/words.rs`: `long-words/composer-view`, `ui-proposal-card/card-states`, `ui-qa-small/three-lines`.
- `temper/crates/temper-client-shell-web/temper.css`: `ui-proposal-card/card-proof`. This plan's rules sit under their own `/* ==== ui: public ==== */` marker.
- `temper/crates/temper-client-view/src/look.rs`: new tokens may collide with another plan's, and its test would say so.
- `temper/docs/design/client/browser.md` and `browser.status.md`: `model-check/trial-close`. This plan changes 8.3 and 8.4 and adds its own status lines.

`public-records` touches temper only for the required field, in `temper-client-view`'s public view and its tests.

## Seen, and left

- `temper/crates/temper-client-tools/src/landing.rs`, `hex`, prints a commit's 64 digits to the terminal for `temper ctl`, so a 40-digit id shows 24 zeros there too. It isn't a link and isn't a stranger's page, so it isn't this plan's. It's worth its own small fix.
- **"Nothing has landed yet." can be untrue on a stock queue.** A project that lands by queue makes no landing records, so while a change waits there, the board's section shows its queue line and these words, even if earlier work landed by queue. This follows the request's ruling and its *Left out*, so the plan builds it as ruled. The user may want other words once a stock queue's Landings are filled.

## Not run in the plan

`browser_public.rs` is updated and compiled, not run: after-landing runs it. The look's proof is a throwaway render, at 320 to 1440 px, with screenshots at 1440 and 420 in the task's report.
