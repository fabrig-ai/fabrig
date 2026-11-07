+++
id = "jig-room"
part = "profile"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/profile/standard.rs", "jig/crates/jig-protocol-config/src/tests/profile.rs", "jig/crates/jig-protocol-charter/src/**", "jig/crates/jig-protocol-worker-agents/src/tests/route.rs", "jig/testing/jig-test-service/src/tests.rs"]
effort = "high"
minutes = 120
+++

# jig's standard holds a whole spec, and a change's body follows its words

## Goal

jig's `standard` profile refuses a `change` of five changes whatever its bytes, and holds a charter to 65,536 and an outcome to 4,096. A change's `body` is held to 3,584 by a constant. This task moves the three figures in `standard` and makes the body follow the contract's words, so a report is a setting from then on.

Acceptance:

- `standard`'s `authority.batch` is 32, `hub.charter_bytes` 262,144, `hub.outcome_bytes` 131,072. `small`'s figures don't move, and neither does `temper/docs/help/reference/limits-engine.md`, which prints small.
- `CHANGE_BODY_MAX` is gone: `rg -n CHANGE_BODY_MAX jig temper smith -g '*.rs'` finds nothing.
- A change contract's `body` has a `max` of its `words`; its title, failure and `no_change` keep `words.min(CONTRACT_TEXT_MAX)`. A report-only run that delivers has a `title` of `CONTRACT_TEXT_MAX` and a `body` of the larger of its contract's words and `CONTRACT_TEXT_MAX`.
- `a_change_at_its_body_bound_encodes_within_the_hubs_outcome` holds rows at an outcome of 4,096, 10,662 and 131,072, each filled to the outcome and encoded within it with no field lost.
- Focused and fuzzy tests of each changed package and of the readers below pass.

## Inputs

- `temper/docs/plan/call-room/notes/common.md`; `temper/docs/plan/call-room/notes/contracts.md`, 3.
- `jig/docs/design/domain/hosts.md`, 8; `jig/docs/design/protocol/configuration.md`, 6.
- `temper/docs/design/protocol/configuration.md`, 13.8, *A batch* and *A brief*; `temper/docs/design/domain/agent.md`, 4.4.

## Owns and may change

Owns the header's files.

Readers and neighbours, the edit allowed in each:

- `temper/crates/temper-engine-smith/src/room.rs`: `measurement_charter`'s literal takes the new field and its `max`es (`notes/contracts.md`, 3). `temper/crates/temper-engine-smith/src/tests/room.rs` takes what that measures.
- `temper/crates/temper-service/src/tests/stock_brief.rs`, `temper/crates/temper-service/src/limits.rs`: a stock charter's contract now prints a body bound of five digits where it printed four, so `every_stock_charter_admits_its_whole_maximal_brief`'s `non_brief` and `remaining` pins move by a few bytes. Regenerate those pins, name them old and new. `root.brief.brief_bytes`, 355,965, moves only if the fit assertion fails, and then by what it needs, named in the report. `temper init`'s T doesn't move.
- `temper/tests/engine/domain/tests/suite/fabrig_scale.rs`, `a_goal_tree_takes_384_tasks`: its batch size is chosen from `limits.authority.batch` and each coordinator is funded for a batch of four. Hold that story's batch at four: `.min(4)` on its `batch_size` line, with its comment saying so. Its rule is the tree's 384 tasks, not the batch.
- `temper/tests/one-process/src/lower.rs`, `temper/tests/engine/domain/src/store.rs` and `src/forge_routes.rs` set `authority.batch` by literal: they stay.
- The store pins that follow a stored denial's bound: `jig/crates/jig-protocol-config/src/tests/profile.rs` (`standard_holds_fabrig_scale`, its page of 100,867,375, its value of 100,865,296), `jig/testing/jig-test-service/src/tests.rs` (`both_jig_profiles_have_usable_store_edges`: value, commit 901,830,282, operations 303,350). Regenerate each, old and new in the report. `jig/tests/store-codecs` and `jig/tests/core`'s `settings_memory.rs` take a standard-derived record: run them, and change them only for the figure.
- The literal `Charter { .. }` sites the compiler names, in any file, for the new field: `words: 0` in a site with no change or report under test.

Options, each pre-approved:

- **The batch's cost is more than bounds.** `stored_finding_limit` goes from 92 to 300, under its codec ceiling of 1,024, and `needed_within`'s aggregate from 144 grants and 153 executors to 512 and 544. If a store envelope check refuses `standard` for it, or a figure moves by more than those explain, move `authority.batch` to `temper/docs/examples/reference/limits.json`'s `engine.limits.overrides` as 32, leave `standard` at 4, and say so in the report with the figures. Anything past a codec ceiling is a `Question:`.
- **`Charter.words`** may take another shape where it reads better (a `delivery_words`), as long as `charter()` gets the contract's words without a new parameter on `assignment`.
- **A body larger than the agent's outcome** isn't refused at admission: `outcome::is_valid` checks minimum fields only, and the aggregate is judged at each `finish` and `deliver`. Leave that.
- **`smith_host_protocol::fixture::small()` can't encode the row at 131,072:** give that row a fixture whose `field_text` holds it; `smith_channel::CEILINGS` does.

## Steps

The figures, in `standard.rs`:

```rust
pub(super) const AUTHORITY: authority::Limits = authority::Limits { batch: 32, /* as it is */ ..small::AUTHORITY };
pub(super) const HUB: host::Limits = host::Limits {
    charter_bytes: 262_144,
    outcome_bytes: 131_072,
    /* as it is */
    ..small::HUB
};
```

`standard_holds_fabrig_scale` pins the three. In `jig-protocol-charter`, take `notes/contracts.md`, 3: `assignment.rs` sets `words` from `tasks::Contract::{Report, Change}` before `contract()` consumes it. Module docs cite `jig/docs/design/domain/hosts.md`, 8 as they do.

Tests, in `jig/crates/jig-protocol-charter/src/tests.rs`: `contract_text_is_bounded_and_change_checks_are_required` drops its pin of 3,584 and holds a body's `max` at a large `words` (131,070) and at a smaller one; `delivery_follows_contract_policy_and_workspace` and `delivering_report_keeps_its_outcome_and_independent_delivery_bounds` hold a delivery's body at the contract's words, and at 1,024 where the words are smaller. They're existing tests: no new test binary. Break each rule once by hand; report it.

A heap ceiling that moves: measure before and after, and say why (`notes/common.md`).

Readers' tests, since this changes what shared items mean: `heavy cargo nextest run -p jig-protocol-config -p jig-protocol-charter -p jig-protocol-worker-agents -p jig-domain-authority -p jig-test-service -p jig-store-codecs-world -p jig-core-world -p temper-engine-smith -p temper-service -p temper-engine-domain-world`, with `--profile fuzzy` for the packages you changed. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`, and temper's; `heavy cargo clippy -p jig-protocol-config -p jig-protocol-charter -p jig-protocol-worker-agents -p temper-engine-smith --all-targets -- -D warnings`. Commit, push.

## Report

What the batch cost in bounds, by figure, old and new, and whether it stayed in `standard`. Every pin you regenerated. Any heap ceiling that moved, before and after. What a stock charter's `non_brief` and `remaining` are now.
