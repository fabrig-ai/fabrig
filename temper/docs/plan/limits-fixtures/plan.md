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

## Before and after

Measured on `debian-32gb-hel1-11`, 2026-11-07 UTC, from the plan base
`7ffd748466210b3b131113777ca14de89f592874` to integration tip
`f81186919ac77f2d6ef5ebef094fe026fc9c50e6`.
Full counts, every named probe/check, causal costs and both invariant checks
are in `temper/docs/plan/limits-fixtures/figures.md`.
Each percentage below is a sampled test’s net instruction change; negative
means fewer instructions. Intervening plans are included in the net result.

- `temper-engine-domain-world::suite`, `forge_routes`: floors -0.08% / -0.08%; original median -19.62%; advisor median -28.66%; resolution probe -2.66%.
- `temper-engine-domain-world::suite`, `root_routes`: floor -0.12%; median -9.27%.
- `temper-engine-domain-world::suite`, all: binary floor -0.12%; route medians above; named walking replay -0.65%.
- `temper-service`, `setup`: floor -0.42%; original median -59.63%; later-selected median -77.38%; request/keep-order probe -74.43%; shared setup probe -77.44%; named malformed-query check -83.14%.
- `temper-service`, `tests::stock`: floor -0.42%; median -0.41%; stock construction probe -79.02%.
- `temper-service`, all: setup/stock samples above; remainder floor -0.40%, median -30.62%; named fuzzy replay -34.13%.
- `temper-client-world::suite`, all: floor -0.38%; dismissal median -60.42%; recovery probe -7.67%.
- `temper-client-view`, all outside `tests::limits`: floor -0.53%; median -20.48%; words-fixture probe -16.73%.
- `temper::suite`, `startup`: floor -0.25%; median -8.33%, including child instructions.
- `temper::suite`, `forge`: floor -8.04%; median -8.79%, including child instructions.

All 68 available one-test traces passed. Six newly added checks have an
`unknown` base figure, with their adding commits recorded in `figures.md`.
Name comparison accounts for all 11 missing base names in other plans
(model-picker `50e7171ce1`, proposal revision `74918c6e56`); none was removed
by this plan. Nextest settings, tier budgets and every Cargo manifest are
unchanged. No whole-suite CPU saving is inferred from the samples.

### Causes not fixed

The R identifiers refer to the cause table in
`temper/docs/plan/limits-fixtures/figures.md`; its source figures give the
sampled cost and stop-rule calculation for each. Each remaining cause has
its required further work below.

- R1: **Loader relocations and process/harness floor**. Requires binary/loader/linker work; build settings are closed. Fixture changes cannot remove process startup.
- R2: **First typed engine/init configuration parse and validation**. A generated typed stock/projection checked against the shipped input, or parser/validator work, including skein Slice::answer allocation/copy. Per-process caching cannot remove the first call.
- R3: **Two startup document parses with different consumers**. Share the bounded parsed document through startup/setup interfaces while preserving depth/token/byte limits and diagnostic ordering.
- R4: **Remaining stock parse and init generation**. Further parser/generator work. Task whole-stock upper estimates are 0.42 and 0.017 CPU s; a generated-byte cache was not justified.
- R5: **Real first start, durable rows and cold restoration**. An independent configuration-dependent writer for complete policy/settings/start rows, counters and durable state, checked against the real two-commit script. A start mark alone is insufficient.
- R6: **Repeated admission and owned policy/configuration snapshots**. Share checked construction results or change snapshot/config ownership while retaining distinct admission checks and journal/restart lifetimes.
- R7: **Hosted store replacement and secret exchanges**. An in-memory hosted-store constructor over supplied maps, retaining driver state, wires, durable startup rules and secret exchanges.
- R8: **Sizing translations and generated bound validation**. Shared checked sizing/codec interfaces across application, commit/frame and store boundaries, or generator changes with schema/bound tests. Discarded stores no longer pay this cost.
- R9: **Reusable replacement store during a story restart**. A distinct consuming restart cut path preserving the retained driver destructor/oracle contract; the public reusable take_store must leave usable replacement storage.
- R10: **Real row encoding/decoding and temporary owned values**. Borrowed translation/writer interfaces that preserve encoded bytes, malformed/bound refusals and real codec execution. Typed-mirror bypass would weaken the world.
- R11: **Separate owning rows for history, mirror and inflight write**. A borrowed commit-encoding seam or durable-fork representation; mirror and replayable history still retain independent values.
- R12: **One-row restore/map requests and cursor/key copies**. A batching/paging algorithm preserving encoded bounds, lookahead cursor, first-row refusal and corruption/diagnostic order, with boundary tests.
- R13: **Actual story decisions, engine stepping and readback**. Profile admission/snapshot and page projection/stepping mechanisms with lower-tier stories; retain all inputs, durable commits, deliveries and assertions.
- R14: **Independent teardown and restart oracle**. Cheaper snapshot/paging and retained-configuration representation while preserving the independent restart and final-store oracle.
- R15: **Cold English construction, remaining ordering and allocations**. Already ordered vocabulary writers with a checked ordered-input path and a cheaper construction representation, retaining every word, form and hole.
- R16: **Owned nested Words clones and teardown**. Flatten loaded-book allocations or change caller/ownership/lifetime interfaces. View owns Words; removing its cache would rebuild the book for repeat clients.
- R17: **Remaining capacity/presence/maximum scans**. A trusted checked vocabulary summary/representation that still detects public book/connector mutation, missing vocabulary and overflow. Domain limits alone cannot key a safe cache.
- R18: **Whole-book validation and capacity arithmetic**. Fuse checked passes with preserved whole-load error precedence and bounded occurrence counters. Local limits arithmetic alone is below the stop rule.
- R19: **Semantic drawing and tree diff during real stories**. A different indexed traversal preserving keyed/role sibling matching, stable identities and exact patch behavior, checked in skein UI tests.
- R20: **Applying and independently checking drawn trees**. An incremental model/checking representation preserving the independent patch oracle and callback builds.
- R21: **Domain setup and semantic rendering that assertions consume**. A separate domain/rendering profile and mechanism change; retain differently bounded Views, domain facts and asserted text.
- R22: **Paired configurations, piece/config copies and idle bookkeeping**. Factor paired per-configuration construction or introduce borrowed codec/config views without losing consumed inputs. Reported individual upper estimates stay below 0.529 CPU s.
- R23: **Referee/default words, fake/component/Truth setup and engine limits**. An explicit words constructor or lazy/copy changes would add interfaces/state for sub-threshold savings. Preserve fakes and complete decision vocabulary. Domain fake construction is only 86,411 over three pairs.
- R24: **Required GitHub signing**. Cryptographic implementation work, or a separately justified fixture seam retaining wire authentication; a canned token would skip the signing path.
- R25: **Remaining Face, connection and local settling/manifest mechanics**. Connection settling must retain forge.above for assertions; keep required framing/routing and scenario time advancement. Connection-only estimated reach is 0.060 CPU s; local advance/manifest upper estimates 0.211/0.093.
- R26: **Process fixtures, measured turn bounds and scripts**. A cheaper equivalent bound encoder or byte-identical runtime-path templates; shared process fixture work belongs to model-catalogue-2/process-fixtures. Measure kernel file/exec cost separately while preserving process proof.
- R27: **Cheap rule checks and harness work**. Retain the behavior checked. No shared expensive fixture is reached; harness work is outside fixture repairs.
