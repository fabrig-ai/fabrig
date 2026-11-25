# A full forge outbox is a wait, not a hold

Draft, 2026-11-15, on the user's word. On 11-15, eight changes were reopened at once. Two of them could not open their pull requests: the forge's outbox, 8 entries for the whole deployment, was full. Each was held for a person, with words saying to raise the limit and restart. The outbox drained by itself within minutes, but nothing retried, and both stayed held until a person released them (P57).

## 1. A write that finds no room waits for it

- **Today:**
  - A change's effects (open, merge, update, reopen) go through `emit_change_effect`, and the forge's other writes (comments, closes, an agent's reopen) go through `enqueue` and `enqueue_release` (`temper/crates/temper-engine-domain-forge/src/domain.rs`). Each refuses when `entries` are all taken.
  - The refusal becomes `ResourceRefused` with cause `Limit`, and the root turns it into a task hold, `Hold::ResourceUnavailable`, with the words "reduce the named work or raise this limit in the deployment configuration, restart, then release" (`temper/crates/temper-engine-domain-forge-vocabulary/src/refusal.rs`; `temper/crates/temper-engine-domain/src/route/forge_route.rs`).
  - Release cleanup's comments and closes report `ReleaseFailed` instead. Goal projections already wait, as goal branches do, in order (`goal_branch.rs`).
  - The refused write takes no entry. Held entries do count against the bound.
  - `temper/docs/design/domain/forge/effects.md`, 1, says a requester finding the outbox full "waits or is refused by its own rule".
- **Wanted:**
  - A write that finds the outbox full waits. Its intent is kept durably, or the change keeps a state it can replay, so no request is lost. It is admitted in order when an entry leaves: first come first served, with no newcomer overtaking a waiter. When it is admitted, it checks its facts and authority again, and journals before it sends, as now. This is the same shape as goal branches' wait.
  - A wait is no failure. It counts against no allowance and no stall, it survives a restart, and a cancel ends it.
  - A hold stays only for a write that can never fit. A single write over `forge.client.op_bytes` is already told apart from a full outbox (`TooLarge` against `Busy`, `skein/crates/skein-forge-client/src/outbox.rs`).
  - Release cleanup's comments and closes wait the same way, rather than failing.
  - The change's panel says "Waiting for room in the forge outbox" while it waits, and clears that once admitted. The wait's reason is typed through the forge panel, the client's wire and its words. The client's wire moves its version unbridged, by the upgrade rules.
  - Revise `temper/docs/design/domain/forge/effects.md`, 1, and `temper/docs/design/domain/forge/change/README.md`, 2, and `temper/docs/design/domain/forge/change/panel.md`, 1, with their status files: a full outbox is a wait; say its order, what happens on restart and cancel, and the refusal for a write that can never fit.

## 2. Out

- The figure itself: `forge.entries` is 8 in both profiles, taken from the world fixture, and `forge.client.entries` must stay at least as large. It's the user's to set.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- In the forge domain's tests (`temper/crates/temper-engine-domain-forge/src/tests/refusals.rs`, replacing the cases that expect a full-outbox refusal):
  - a full outbox, two waiters, one entry freed: the first waiter is admitted, with no hold and no duplicate;
  - the same across a restart;
  - a waiter cancelled while it waits;
  - a write too large to ever fit still refuses.
- The same wait for a merge, an update, a comment, a close and a reopen.
- The panel's tests: the wait shows, and clears on admission.
- The engine's held-change story (`temper/tests/engine/domain/tests/suite/forge_routes/held_change.rs`): eight changes opening at once with two over the bound all open, with no person.

Run only the new and affected tests, by name.
