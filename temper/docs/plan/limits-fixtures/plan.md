# limits-fixtures: the plan

From `inputs/request.md`: 8.1's second question for temper's many small tests. Their place is right, so make them cheaper at the cause. What each spec says about a cause comes from reading the code on 2026-11-06. No test ran for this cut, since a call builds nothing, so each task profiles first and follows the profile.

## Parts and tasks

| Row of the request | Part | Task | Needs | Minutes |
|---|---|---|---|---:|
| `temper-service` all, `setup`, `tests::stock` | `service` | `service-fixture` | | 120 |
| `temper-client-view` | `view` | `view-words` | | 100 |
| `temper-engine-domain-world::suite` all, `forge_routes`, `root_routes` | `engine-world` | `domain-fixture` | | 120 |

More tasks follow with the next push: at most eight, as the request says.

**Critical path, so far:** any one of these, about 120 implementer-minutes.

## What was decided

- **No design is written.** This plan changes what tests and the product cost, not what they do. Its contract is `skein/docs/foundation/testing-strategy.md`, 8 and 8.1. If a task's product fix changes a public type that a design section names, `close-figures` writes that.
- **A fixture's change is in one task** (the request). `tests/fixtures.rs` is `service-fixture`'s, and the engine world's construction is `domain-fixture`'s.
- **Fixture names and signatures stay,** since other plans add tests that call them (`notes/common.md`).
- **Each task writes its own figures file,** so they don't conflict.

## Overlaps

- **`model-catalogue-2/stock-models`** changes `temper-service/src/tests/fixtures*` and `tests/stock*.rs`, and `service-fixture` owns these too. This couldn't be avoided: the fixture is the cause. A catch-up is expected, and the diff stays tight.
- **`model-catalogue-2/catalogue-clients`** and **`goal-delivery/goal-start`** change `temper-engine-domain/src/route/**`, and **`catalogue-store`**, **`model-surface`** and **`ui-model-next-turn/choice-next-turn`** change `jig-protocol-store`. `domain-fixture` changes those only where the profile puts a cause, and small.
- **`look-round-13`** and **`model-catalogue-2`** change `temper-client-view/src/words.rs`. `view-words` keeps to `Words::english` and `english_language`, and leaves the words themselves.
- **`limits-heavy`** owns `temper-client-view/src/tests/limits.rs`. `view-words` leaves it alone.
