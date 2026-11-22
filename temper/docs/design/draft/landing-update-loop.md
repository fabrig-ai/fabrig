# A stale update replans instead of spinning

Draft, 2026-11-16, on the user's word. On 11-16, a landing asked the forge to rebase its pull request onto main. Every try answered "outcome failed" on the old base: eight in one minute, then all 80 of its updates in a minute, then "update pull request: head changed". It came straight back to the same loop after each release, which left a goal's fix unable to land (P66).

## 1. Why it spins

- `change_write` sends a change's `Effect::Prepare` as a forge rebase, and `Effect::Update` as a merge. Both carry `Condition::Update { head, base }` (`temper/crates/temper-engine-domain-forge/src/domain.rs`).
- The forge client checks both that the pull request's head is `head` and that its base commit is `base`. Either mismatch answers `Stale` before anything is sent, and `Stale`'s words are "head changed" (`skein/crates/skein-forge-client/src/outbox/answer.rs`; `temper/crates/temper-engine-domain-forge-vocabulary/src/effect_words.rs`). So a moved base reads as a changed head.
- `change_outcome` turns an update's `Timeout`, `Unavailable` and `Stale` alike into no result, and asks for another decision. `preparing` then calls `prepare_due` at once with the same pinned base (`State::Preparing.main`), counting one more update each time, with no wait (`temper/crates/temper-engine-domain-forge-change/src/domain.rs`). A base pin that the pull request no longer matches therefore spends every update in seconds.
- The history's "updated" line prints the pin, not the base the pull request has, so the words show the stale base.
- A real conflict maps to `Conflict` and reaches a merge job and a resolution. A `Stale` never does.
- Newer landings prepare through a round branch and a worker job, as `temper/docs/design/domain/landing.md` describes. The forge-rebase path remains for landings without a round branch, and for changes already preparing.

## 2. Wanted

- **A stale update replans:** `Stale` is told apart from a lost write, in `change_outcome` and `step_read` and in the policy's `preparing` and `prepare_due`. On `Stale`, the change reads its pull request's head and base afresh. If its preparation is obsolete, it drops it and makes a fresh one against the current base. It never resends against the old pin.
- **A lost write is recovered** against the requested pin, as today.
- **No spin:** where fresh reads still disagree with the plan (the forge's view lagging, say), the change waits with a bounded backoff before trying again, and that wait spends no update. The update count counts sends, not refusals before sending.
- **Honest words:** a stale refusal says which of the head or the base moved, from the client's check. The history's update lines print the base the forge reports alongside the pin.
- **A conflict goes to its job:** a legacy rebase preparation that meets a real conflict goes through `conflict_due` to the merge job, then to a resolution for what's left, as a merge update does.
- Revise `temper/docs/design/domain/forge/updates.md`, 1, which allows counted retries while saying they never spin, and the change's update words, with their status files.

## 3. Out

- The round-branch prepare path's own recovery (`landing-prepare-recovery.md`).

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- In the forge client's recovery tests (`skein/crates/skein-forge-client/src/tests/recovery.rs`): a stale base and a stale head are named apart.
- In the change policy's tests (`temper/crates/temper-engine-domain-forge-change/src/tests/turn.rs`): with an unchanged head and an advanced base, a stale answer replans once against the new base and spends no update. Repeated stale reads back off and stay within the bound. A release and a restart don't restart the spin.
- In the forge engine's update story (`temper/tests/engine/forge/tests/suite/update_lost.rs`): the history shows the right bases, and an explicit conflict reaches the merge job and then a resolution.

Run only the new and affected tests, by name.
