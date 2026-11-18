# A producer that started early catches up once its dependency lands squashed

Draft, 2026-11-16, on the user's word. On 11-15, a change started early from its sibling's checked commit. The sibling then landed into the goal branch squashed, as a new commit with the same tree, and its own branch was deleted. From then on, the early producer could neither deliver nor save: "delivery second parent is not reachable from the branch's remote". Its held workspace still had the sibling's original commit as a parent, and nothing reachable on the forge contained it any more (P61).

## 1. What happens

- An early start merges each dependency's recorded checked commit into the landing tip, with the current head first and the dependency second, and records `StartedFrom` (`temper/crates/temper-engine-domain-forge-change/src/domain.rs`, `producer_start`; `temper/crates/temper-engine-domain-forge/src/items.rs`, `workspace_item`; `temper/crates/temper-worker-domain-checkout/src/hold.rs`).
- A delivery, and an agent's `save`, read HEAD and MERGE_HEAD and fetch the remote's branches. They refuse a parent no remote ref reaches (`temper/crates/temper-worker-protocol-git/src/snapshot.rs`; `temper/docs/design/protocol/git.md`).
- A landing squashes a change into one commit (`temper/docs/design/domain/landing.md`), and a landed change's branch is then deleted (`temper/docs/design/domain/forge/branches.md`). The dependency's commit is no longer reachable, though its work, in the same tree, is.
- "The next producer merges the landing tip first" (`temper/docs/design/domain/forge/change/questions.md`, 1) covers a fresh run. A workspace already held, with its pin frozen, gets no such catch-up, so every later delivery checks the same lost parent. Early-start-checked doesn't touch this: it chooses where an early start begins, not what happens after the dependency lands.

## 2. Wanted

- **A refused delivery or save catches up:** when one is refused for an unreachable parent that is a dependency in `StartedFrom`, and that dependency has landed, the change makes a fresh hold. That hold keeps the producer's work and merges the landing tip into it, with the current work first and the reachable tip second, so delivery stays a fast-forward. The run then delivers from it.
  - Checks run on the resulting tree.
  - A conflict goes to the producer to settle, as at any catch-up.
  - The pin is moved explicitly for this, and no other way.
- **Other refusals stay:** an unreachable parent that isn't a landed dependency, or that is a dependency not yet landed, is refused as today.
- **Early starts stay as built:** only a checked commit starts one, and unchecked saves stay unchecked.
- Revise `questions.md`, 1, `temper/docs/design/domain/worker/checkouts.md` (reconciliation, the pin, delivery and save) and `git.md`'s delivery section, with their status files.

## 3. Out

- How landings squash.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- Worker checkout delivery tests (`temper/tests/worker/checkout/tests/fakes/delivery.rs`):
  - a dependency landed squashed, its branch gone: a delivery and a save each catch up and succeed, with the producer's edits kept;
  - a landing that kept the ancestry needs no catch-up;
  - a catch-up conflict is left for the producer;
  - an unrelated unreachable parent is still refused.
- Engine early-start stories (`temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs`): the squash-and-resume stories run through the worker's guard rather than scripting past it. Early-start-checked's cases still pass.

Run only the new and affected tests, by name.
