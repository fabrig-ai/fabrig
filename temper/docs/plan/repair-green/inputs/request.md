# repair-green: a green head ends a CI repair

From the first real goal, 2026-11-05. A change's CI went red, a repair began and was held; CI was then re-run by hand on the same head and passed. The change stayed `repairing` with `CI passed`, and releasing the held repair made a fresh repair task, as `temper/docs/design/domain/forge.md`, 8.4 says it does today: "asks for a fresh repair with the same reason and current forge facts". Its words read `Repair CI at <head>: `, no check named, because nothing was failing.

This is a design change, the owner's decision, not a bug against the design. `first-goal-fixes` found 8.4 has no such rule, and carried the empty name's words only.

## The rule

A CI repair's reason is CI failed at the head. When CI at the change's head has passed:

1. A repair not running (due, retrying, held) ends without an agent's work, and the change goes on from where it is, as if no repair were needed. No fresh repair is made, on a release or otherwise, for a green head.
2. A repair run in progress finishes as a run does; don't interrupt it. Its delivery, or its finding nothing to do, is handled as today. Settle by 8.4 whether a run's brief needs a word for it.
3. A push moves the head, so the rule asks about the new head alone: a green result on an old head changes nothing.
4. A repair for another reason, a review's request or a conflict, isn't touched.
5. Whatever the task's end reads says that CI passed at the head, not a failure.

Write it into `forge.md`, 8.4, in its words, and name any section it disagrees with. If a concrete case makes this rule wrong or much larger than it reads, say so in a `Question:`, with a recommendation.

## Tests

A failing test first, in `temper/tests/engine`'s forge world or lower: red, repair held, green at the same head, the repair ended and the change going on; released after green, no fresh repair; a run in progress left to finish; green on an old head after a push, no change; a review repair untouched.

## Size

One or two tasks in temper, none in jig or skein unless the reading shows it. `brief-failure` is working in `temper-engine-domain-forge`, and `first-goal-fixes` has just landed there: expect catch-ups. fabrig/ui has priority: leave its files alone. Formats change in place before v0.1.0. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Walk

`walk = false`: the world's story is the proof, and it comes up on a deployment only when someone re-runs CI by hand.
