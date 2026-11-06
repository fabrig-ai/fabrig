# What `temper ctl` tells a person: five small things

_From temper/qa's walks, 2026-11-06, and temper/forge's backlog._

Each is from reading a walk, not reproduced: reproduce it first in the lowest tier, and say if one's wrong. They stand apart, so they can run side by side. **Drop from the bottom if this grows past eight tasks, and say what you dropped.**

## What to change

1. **`--key K` is refused with bare usage.** Keys are lowercase, digits and hyphens (`jig/key.rs`), but neither the refusal nor `temper help ctl` says so. Say so in both.

2. **`task show` contradicts itself on a task waiting for an account.** Its header says `Due to run` against `waiting on: an account`; `--json` has `waits_on: "nothing"`; and `move` refuses `not_waiting`. Make the header, the plain line and the JSON tell one story, and say what `move` should do then. `conditions.rs`'s list for `--until` stays as it is.

3. **`forge setup` on a box's own Forgejo ends "temper's forge user may not read the branch protection..."**, which a new person can't act on. Read why it's said on an owned forge, where temper made the user: either it isn't true there and goes, or temper can do what it asks.

4. **A task held after failed runs says only "out of tries"**, with the reason in the engine's log, so a person reading `task show` or `hold` can't tell why. Showing it means keeping the failure's text on the hold, which is a stored format. **That's a question first, with its cost:** what it stores, its bound, and who may read it, since a failure's text can hold what a run saw. Build it only on the answer. If the answer's no, the words say where to look.

5. **A settings item naming an effort its model doesn't offer gets no answer at all.** On a box whose efforts lack `xhigh`, `settings apply` with one template asking for it printed `the engine ... did not answer within 30 s`, a `change-settings` stayed kept and unanswered, and the engine worked on. The same item with `high` gave its revision at once. A person should get a refusal naming the effort and the model. Reproduce this at your base first: a plan beside this one changes effort in the configuration, and if it's gone, say so and drop it.

## Tests

The lowest tier that shows each, failing first. **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5).** No test reaches a real host.

## Out of scope

- `temper-client-*` crates other than `temper-client-tools` and `temper-client-shell-cli`: temper/ui owns what a person meets in a browser. `ctl-waits` is beside this, on `host.rs`'s deadline words and a kept act's answer: expect a catch-up.
- `wait-words`' three lines, just landed.
- Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

`walk = false`.
