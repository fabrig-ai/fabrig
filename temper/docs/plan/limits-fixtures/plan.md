# limits-fixtures: the plan

From `inputs/request.md`: 8.1's second question for temper's many small tests. Their place is right, so make them cheaper at the cause. What each spec says about a cause comes from reading the code on 2026-11-06. No test ran for this cut, since a call builds nothing, so each task profiles first and follows the profile.

## Parts and tasks

| Row of the request | Part | Task | Needs | Minutes |
|---|---|---|---|---:|
| `temper-service` all, `setup`, `tests::stock` | `service` | `service-fixture` | | 120 |
| `setup`, beyond its fixture (item 2) | `service` | `setup-steps` | `service-fixture` | 90 |
| `temper-client-view` | `view` | `view-words` | | 100 |
| `temper-engine-domain-world::suite` all, `forge_routes`, `root_routes` | `engine-world` | `domain-fixture` | | 120 |
| `forge_routes`, beyond its fixture (item 2) | `engine-world` | `forge-routes-steps` | `domain-fixture` | 120 |
| `temper-client-world::suite` | `client-world` | `client-world` | `view-words`, `domain-fixture` | 90 |
| `temper::suite` `startup`, `forge` | `temper-suite` | `suite-startup` | | 75 |
| Item 3, before and after | `figures` | `close-figures` | all seven | 60 |

**Eight tasks, 775 implementer-minutes.**

**Critical path:** `domain-fixture`, then `forge-routes-steps`, then `close-figures`: about 300 implementer-minutes. `client-world` also waits on `domain-fixture` and `view-words`, about 210 before the close.

## What was decided

- **No design is written.** This plan changes what tests and the product cost, not what they do. Its contract is `skein/docs/foundation/testing-strategy.md`, 8 and 8.1. If a task's product fix changes a public type that a design section names, `close-figures` writes that line.
- **A fixture's change is in one task** (the request). `temper-service/src/tests/fixtures.rs` is `service-fixture`'s, the engine world's construction is `domain-fixture`'s, and `Words::english` and `Phrasebook::load` are `view-words`'.
- **A set's stepping is a task apart, after its fixture** (`setup-steps`, `forge-routes-steps`). The request's item 2 asks what they do beyond the fixture, and that can only be read once the fixture is cheap.
- **`client-world` waits for `view-words` and `domain-fixture`.** It builds on both: its world makes the same book and runs the same first start, so profiling it earlier would chase the same causes twice.
- **Fixture names and signatures stay,** since other plans add tests that call them (`notes/common.md`).
- **Each task writes its own figures file,** so they don't conflict, and `close-figures` measures every sampled test at the base and the tip on one machine. A task's own before starts from what earlier tasks saved, so a row's before and after can only be read there.
- **`suite-startup` is the smallest row, and may find nothing worth fixing.** It runs the real binary, much of its cost is the kernel's, and callgrind can't see that. It's in the plan because the row is in the request, with a stop rule of about 0.3 CPU s.
- **No test goes,** so `Nothing is lost` needs no holders in this file. `close-figures` compares the test names at the base and the tip, and names any that went.

## Overlaps

The request expects catch-ups in these crates. These overlaps couldn't be avoided, because the fixture or code is the cause:

- **`model-catalogue-2/stock-models`** changes `temper-service/src/tests/fixtures*`, `tests/stock*.rs` and `src/startup*`. `service-fixture` owns the first two, and `suite-startup` may change the last, small.
- **`model-catalogue-2/catalogue-clients`** and **`goal-delivery/goal-start`** change `temper-engine-domain/src/route/**`. **`catalogue-store`**, **`model-surface`** and **`ui-model-next-turn/choice-next-turn`** change `jig-protocol-store`. `domain-fixture` and `forge-routes-steps` change those only where the profile puts a cause, and small.
- **`look-round-13`** and **`model-catalogue-2`** change `temper-client-view/src/words.rs`. `view-words` keeps to `Words::english` and `english_language`, and leaves the words themselves.
- **`model-catalogue-2/process-fixtures`** changes `temper/crates/temper/tests/support/**`. `suite-startup` keeps off it unless a line there saves something.
- **`model-catalogue-2/config-models`** changes `jig-protocol-config`. `service-fixture` may add `Clone` there, and `suite-startup` may make a small change.

These were kept apart by the tasks' boundaries:

- **`limits-heavy`** owns `temper-client-view/src/tests/limits.rs`, `setup_malformed_queries_are_bad_requests`, `box_listeners`, `forge_land`, `fake_backends` and `fabrig_scale`. No task here takes them.
- **`limits-worlds`** profiles the same engine and store from the real loop, so it may find the same causes. `forge-routes-steps` reads its trunk before fixing a shared cause.
- **`goal-delivery`** changes `forge_routes/goal_delivery.rs`, and **`look-round-13`** changes `temper/tests/client/tests/suite/**`. `forge-routes-steps` and `client-world` leave them.
