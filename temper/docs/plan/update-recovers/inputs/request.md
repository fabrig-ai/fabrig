# update-recovers: a lost answer to an update doesn't hold a change for good

From temper/ui's trial of the process tier at three threads, 2026-11-06: `a_turn_landing_is_found_after_a_runner_cancellation_and_restarts` failed with `update pull request: uncertain: it may or may not have been made`, and no landing progress in 60 s. Alone, and at one thread, it passes. Its scheduler figures showed no starvation, and no port or cache cause. The trial's logs are outside this repository, so the words here are what you have.

**From reading, not run; reproduce first, and say if it's wrong:**

- An update, `Write::Update` (`POST R/pulls/<number>/update?style=merge|rebase`, `skein/docs/design/forge.md`), is `unrecoverable` (`temper/docs/design/domain/forge.md`, the table of effects; `skein/docs/design/forge-client.md`, 7.5).
- A forge call's deadline and a write's lifetime are `forge.client.lifetime_ms`, 10 s on the stock. A forge's update does its git work inside the call, so on a loaded machine its answer can come after that, and a restart can lose it as well.
- So the answer is lost, the entry is `Outcome::Uncertain`, and it's looked for by a comparison showing the head holds the base. Not found once the lifetime has passed, it is `Outcome::Held` (`skein/crates/skein-forge-client/src/outbox/retry.rs`, `not_found`), and the client never sends it again by itself. The change waits for a person's `Remake`, even if the forge's own work finishes a moment later.

So a slow machine can stop a goal until someone comes, which a person running goals unattended won't expect. It isn't slowness: it stays held.

**What's wanted.** An update that was lost and isn't found is checked and made again, as a write that isn't `unrecoverable` is, up to `write_attempts`, and a person meets a hold only for what really needs one. An update is safe to repeat, as far as reading shows: its proof is that the head holds the base, a second update of a head that holds it changes nothing that matters, and a merge is conditional on its exact head, so a late copy can't land something unchecked. **Check that holds for both styles, and say what a late copy can cost:** a rewritten head under a running CI, I'd expect, and no more. If it can do worse, say so and take the nearest thing.

**Which class it becomes is yours to settle** against jig's `domain/connectors.md`, 4.3, which asks a class to say what a late copy may do, and against its section 15, which asks the connector's world to show the class holds. Say why, and move `forge-client.md`, 7.4 and 7.5 and `forge.md`'s table with it.

Also, in this order:

1. **What an update that fails for good does.** `Outcome::Failed(Timeout)` after `write_attempts` reaches the change's machine where `Held` did today. Say what it does there, and make sure it neither spins nor stops silently: a person is told why, if one is needed at all.
2. **A held update already stored:** formats change in place before v0.1.0, so no migration; say if there's anything to clear.
3. **A forge's real time for an update isn't known.** `forge.client.lifetime_ms` stays: name in your report what a lifetime that's too short still costs (a repeated update, a CI run) so that it can be measured after landing.

Failing tests first, in the lowest tier: `skein-forge-client`'s, beside `tests/recovery.rs`, and temper's change machine for item 1. Keep the words of a hold for what stays unrecoverable. **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5):** no process or browser story runs here; temper/ui re-runs its three-thread trial after landing, and that's the proof. A generic mechanism goes in skein first, and skein names no temper. temper/ui owns `temper-client-*`: stay out of it, and a stored format or protocol change is a question first, so its owner can be told. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`.
