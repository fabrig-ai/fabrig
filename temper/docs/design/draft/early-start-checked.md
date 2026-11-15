# An early start begins from checked work

Draft, 2026-11-15, on the user's word. On 11-14, a goal cut two changes `after` a sibling change into the same goal branch. Their producers started early, seconds before the sibling's producer asked a question with its work saved unchecked. So both started from that unchecked save. For an hour they met 349 and 429 delivery-check failures that were the sibling's, asked their goal about them, and worked on a base that kept moving until the sibling landed (P51). Nothing told them that their tree held a live sibling's unchecked work. Meanwhile the stock designer charter tells designers that a change cut `after` others "starts once both land".

## 1. Start early only from checked work

- **Today:**
  - `producer_start` (`temper/crates/temper-engine-domain-forge-change/src/domain.rs`) starts a change's producer early once every pending same-repository, same-landing-branch dependency has a head and isn't itself kept by a dependency.
  - The head is the dependency's branch tip as the forge last read it (`landing_route::pending`, `temper/crates/temper-engine-domain/src/route/forge_route/landing_route.rs`; `temper/docs/design/domain/forge/change/questions.md`, 1, *Early starts*). That includes an unchecked save before a question, which the worker pushes as "Unchecked: work in progress, saved before a question".
  - A dependency kept only by its own open question doesn't block.
- **Wanted:**
  - A dependency's head counts for an early start only once its work is checked: its producer's result, or a delivery whose checks passed (the worker's `Landed.checked`, already on the wire and in the delivery history). An unchecked save, or any push made after the last checked one, never counts.
  - The change keeps its dependency's last checked commit on its record, and the early start merges that commit, never a later branch tip. This is a stored format change: move its version and bring its patch, by the upgrade rules.
  - Keep `producer_start` and the forge's own early-start predicate (`early_base`, `temper/crates/temper-engine-domain-forge/src/domain.rs`) in agreement, and keep the checkout's selection pinned to the recorded commit (`temper/docs/design/domain/worker/checkouts.md`).
  - Everything else in *Early starts* stays: once only, a conflict refuses the start, and the next producer merges the landing tip first. The landing still waits for every dependency to be done. `kept-change-answers.md`'s rule, which holds an answer only for a dependency the producer didn't start from, is unchanged.
  - Revise `questions.md`, 1, *Early starts*, to say a pushed head counts once it is checked, and why.

## 2. An early producer knows what it started from

- **Today:** `production_words` and `produce_words` (`temper/crates/temper-engine-domain/src/route/forge_route.rs`) give the producer the change's words and generic delivery instructions. The change records each dependency it started from, with its head (`StartedFrom`, `temper/crates/temper-engine-domain-forge-change/src/boundary.rs`), but the producer isn't told about them.
- **Wanted:**
  - The facts go in code. A producer that started early gets one short fixed line per dependency in its required words, as `[This task: N]` is given: `[Started from change N at <commit>, not landed]`. The lines hold through amendments and resumes, and fit the smallest brief budget, cut with a marker rather than dropped.
  - The rule for what to do with those facts goes in the charters (section 3), not in generated words.

## 3. The charters say what happens

- **Today:** the stock designer charter (`temper/docs/charters/stock/designer.md`, its line on starts and its worked example) says a change cut `after` others starts once they have landed. The implementer and coordinator charters say nothing of early starts.
- **Wanted:**
  - **Designer:** a change cut `after` others may start early from their checked work, and lands only after them. A spec whose work needs a dependency's landed result, not just its checked work, says what to check first, as the charter already says for work that "rests on something only A's real result shows". Fix both places that promise a start at landing.
  - **Implementer** (`temper/docs/charters/stock/implementer.md`): a failure inside work your task started from, named by its `[Started from …]` line, belongs to that change. If your delivery still passes, leave it and name it in your result. If it refuses your delivery, save and ask, naming the change and the commit. Don't fix it, and don't record it as a red.
  - **Coordinator** (`temper/docs/charters/stock/coordinator.md`): such a failure is the live dependency's, never a red to record. Answer that the change waits for the dependency to land and then merges g's tip, or, where its delivery passes, leaves it under `## Left`.
  - Keep the charters' assembly and size tests passing.

## 4. Out

- Dropping early starts.
- Telling a running producer when its dependency pushes again.
- An `after` that waits for landing, for a change that mustn't start early. Add it only if early starts from checked work still prove costly.

## 5. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The pure start tests (`temper/crates/temper-engine-domain-forge-change/src/tests.rs`, near the start cases):
  - a dependency whose head is an unchecked save, its producer asking: no early start;
  - a checked head: an early start from that commit;
  - a later unchecked push after a checked one: the checked commit is used.
- Stories in `temper/tests/engine/domain/tests/suite/forge_routes/early_start.rs`: the same cases end to end, one across a restart, and the producer's required words carrying a `[Started from change N at <commit>, not landed]` line per dependency, through an amendment and a tight budget.
- The checkout selection tests (`temper/crates/temper-engine-domain-forge/src/tests/early.rs`) pin the recorded commit.
- `kept-change-answers.md`'s focused cases in `questions.rs` still pass, their fixtures given checked heads.
- The stored format's patch: its focused upgrade test.
- The stock charter tests (`temper/crates/temper-engine-service-stock/src/tests/stock.rs`) pass with the revised designer, implementer and coordinator lines.

Run only the new and affected tests, by name.
