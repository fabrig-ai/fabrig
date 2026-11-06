# sign-in-new-tab: a plan's sign-in moves to a new tab, and temper's page stays and waits

The user took this behaviour as offered, 2026-11-06. The spec is `temper/docs/design/draft/sign-in-new-tab.md`: read it whole. No compatibility is owed before v0.1.0. The look request `sign-in-new-tab` is landed separately, by fabrig/ui's plan: this plan is the behaviour, and takes that look by catch-up.

In short: the press opens a tab in its own turn (a tab opened after the answer is blocked as a pop-up), the answer points it at the address with its opener cut, and the page holds the sign-in with no `Return` and no `Request::Leave`, moving on from the `accounts` watch. The listener's fixed page says "Sign-in received. You can close this tab.", because it answers before the exchange ends and can't say "Signed in". Section 5 removes the departure for a sign-in and the "This browser isn't on temper's computer" entry, which is no longer needed.

## Check on the way, and report

- **The user's gesture.** The tab has to open while the browser still counts the press as a user gesture. Confirm from the code that `skein-ui-web`'s press reaches the host's turn close enough to the click on Firefox and Safari as well as Chromium, and test it in the browser tier on whichever of them that tier runs; say plainly what's only read and not run. If a queued press breaks the gesture, the fix is to make the press synchronous for this act, in skein first, and that's a question only if it's large.
- **A blocked pop-up.** If the browser blocks the tab anyway, the person isn't left waiting: say what they see and do, in the words the draft has, or the nearest thing.
- **The journeys.** The journey walks that sign in will meet a new tab, and the recorder has to keep what the shell did with it (`temper/docs/design/client/journeys.md`, 3.1 and 4.2). Name each journey that moves. Its words are for the look's owner, fabrig/ui: coordinate through the plan's overlaps.
- **Real and fake providers both.** Every path that signs in: an owned forge's and a plan's. If a path is left as it was, name it.

## Tests

Failing tests first, in the lowest tier that shows each: the press opens the tab in its own turn, before any answer; the answer points it, with its opener cut (`noopener`, or the nearest thing with the same effect); the page holds and moves on from the `accounts` watch, with no `Return` and no `Request::Leave`; the listener's page says the fixed words. No test reaches a real host.

## Size

temper's client, the web shell, and skein's web host only if the press needs it. fabrig/ui has priority, so its `temper-client-*` work comes first: keep hunks small and name each overlap. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`: the user tries it on a real box.
