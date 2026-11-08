# ignored-tiers: tests run by hand that can run unattended join a tier

testing-strategy 8 says after landing runs every test except focused, fuzzy and `live-smith`, and that a check the suites can make becomes a test. Twelve of temper's ignored tests are run by hand: through a script, by a person, or as a measurement. Six of them need no person at all. This plan gives those six a place in a tier. It also writes down, for the other six, why each still needs a person.

## What's wanted

**1. Six tests run unattended, each in the tier that fits what it checks (8.1). Its `#[ignore]` goes.**

| Test | File | Today |
|---|---|---|
| `managed_ready_client` | `temper/crates/temper/tests/live_github_managed.rs` | "fake sitting only", through `scripts/proof-live-github-managed.sh` |
| `rung_b0s_setup_reaches_account_import_with_the_fake_model_and_no_account` | `temper/crates/temper/tests/proof_live.rs` | offline B0 driver proof, through `proof-live.sh rung-b0-offline` |
| `rung_b0s_walk_passes_offline_on_its_twin` | `temper/crates/temper/tests/proof_live.rs` | as above |
| `story_a_at_its_high_water` | `temper/tests/one-process/tests/memory.rs` | a measurement, by hand (`temper/docs/development/playground-memory.md`) |
| `story_as_world_at_its_first_step_and_start` | `temper/tests/playground/tests/memory.rs` | as above |
| `two_change_hint_request_profile` | `temper/tests/engine/domain/tests/suite/forge_routes/hints.rs` | an opt-in before-and-after request profile |

- **What a script sets up, the test sets up itself:** the environment, the fake sitting and the offline twin. The script keeps working, by running the test.
- **A measurement becomes a check with a bound.** The bound is the figure `playground-memory.md` records, with the margin the doc gives, or 25% where it gives none. The doc says the test now holds the bound, and the bound is named in the test.
- **The request profile asserts what its profile shows,** for example that the second change sends no more requests than the first. If it only prints, it says so in a comment and makes the smallest assertion it can.
- **Each test is selected by one tier,** through `.config/nextest.toml`'s profiles: `process-temper` for a test that starts the binary and its processes, or the suite whose crate it already sits in. Never `live-temper` for a test that needs no real model.

**2. Six tests still need a person, and each says why.**

- `live.rs`: `a_real_model_answers_a_chat_and_pushes_a_small_change`, `rung_b0_a_person_agrees_a_plan_before_the_change_lands`, `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge`, `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`
- `live_github.rs`: `a_chat_lands_a_change_on_github_com_through_a_restart`
- `live_github_managed.rs`: `managed_relay`

For each, judge whether a scripted person on the cheapest stock model could run it unattended in `live-temper` without a real account's sign-in. Where it could, and it fits in a task, make it so. Where it can't, its `#[ignore]` reason names what only a person can give (a sign-in, an account, github.com), and `temper/docs/design/domain/testing.md` lists it once among the tests run by hand.

## Limits

- No figure in `tools/tiers/budgets.toml` changes.
- The new tier members are written and compiled, not run: `cargo check --tests`, and `cargo nextest list` under each profile to show that it selects them.
- `walk = false`. Effort medium. Two or three tasks.

## Sources

- `temper/crates/temper/tests/live_github_managed.rs`
- `temper/crates/temper/tests/proof_live.rs`
- `temper/tests/one-process/tests/memory.rs`
- `temper/tests/playground/tests/memory.rs`
- `temper/docs/development/playground-memory.md`
- `temper/scripts/proof-live.sh`
- `temper/scripts/proof-live-github-managed.sh`
- `.config/nextest.toml`
- `temper/crates/temper/tests/live.rs`
- `temper/crates/temper/tests/live_github.rs`

Read by section: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** Never `tools/tiers/run`, nor a browser, process or live tier or one of their stories: a story that joins one is written and compiled, not run. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
