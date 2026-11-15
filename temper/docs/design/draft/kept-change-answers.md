# A kept change passes its producer's answers

Draft, 2026-11-15, on the user's word. A change waiting on a dependency is "kept" by it. While kept, it holds every answer meant for its early-started producer until the dependency lands. A producer that asks a question therefore stops for as long as the dependency takes, which defeats the early start. On 11-14 this stopped S4's 2960, tier-controls' 2958 and 2962, and S5's 3393 and 3394, each for up to 1.5 hours. A second, smaller defect hid the cause: a task waiting on its own open question shows "Waiting for news".

## 1. Answers reach a kept change's producer

- **Today:**
  - `apply` (`temper/crates/temper-engine-domain/src/route/forge_route/relay.rs`) marks the relay as kept whenever the change's own `waiting_on` is nonempty.
  - `relay` and `next_act` (`temper/crates/temper-engine-domain-forge-change/src/relay.rs`) then store the answer as `Stands::Held` instead of sending it, and refuse to send it while kept.
  - That follows `temper/docs/design/domain/forge/change/questions.md`, 1, which holds answers until every dependency is done or removed. Its own *Early starts* paragraph, which lets the producer work beside its dependencies, says otherwise.
- **Wanted:**
  - An answer to a kept change's producer is delivered at once, as for a change that isn't kept.
  - An answer is held only while the producer has no message room. Keep that condition apart from keptness: `apply` also treats zero room as kept today, so removing the guards alone could lose an answer.
  - Answers already held are delivered when room exists, a restart included, with no new stored schema (`Stands::Held` already keeps them).
  - The landing gate stays: nothing merges before its dependencies land (change `README.md`, 2, step 7; jig `docs/design/domain/connectors.md`, 4.1).
  - Revise `questions.md`, 1, to say so, and change the test that pins the old contract (`temper/tests/engine/domain/tests/suite/forge_routes/questions.rs`, near the "add a dependency with the answer" case) on purpose, saying why.

## 2. A task waiting on its own question says so

- **Today:** jig's wait projection (`jig/crates/jig-domain-core/src/runs/waits.rs`) checks an idle task's dependencies and delegates, and otherwise falls back to "news". It never checks the task's own unanswered question to its requester, which keeps it from running (`jig/crates/jig-domain-tasks/src/run/ends.rs`).
- **Wanted:** such a task's card names its wait, for example "Waiting for an answer to its question N", in jig's English words (`jig/crates/jig-client/src/words/english.rs`). Wherever a wire carries the wait, it moves its version by the upgrade rules.

## 3. Out

- How kept tasks start runs: jig's keep for a task's own open question stays as it is.
- The relay of a goal's words to a producer whose question is already answered (P49), a separate finding.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- In `temper/tests/engine/domain/tests/suite/forge_routes/questions.rs`, a focused story:
  1. A change already kept by a dependency has an early producer, which asks a question and parks.
  2. A person's words add activity but start no attempt.
  3. The requester answers the change's question.
  4. The answer reaches the producer's original question, and the producer starts a new attempt while the dependency is still pending.
  5. Nothing merges until the dependency lands.
  Repeat it across a restart with an answer already held.
- In `temper/crates/temper-engine-domain-forge-change/src/tests/relay.rs`, assert delivery despite dependencies, and keep the zero-room save-and-retry case.
- A focused test of the wait projection: a task waiting on its own question names that wait, not news.

Run only the new and affected tests, by name.
