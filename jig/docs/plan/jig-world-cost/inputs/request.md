# jig-world-cost: what makes jig's focused worlds cost what they do

jig's focused suite reads about 93 CPU s against its 30 (`skein/docs/foundation/testing-strategy.md`, 8). This plan repairs what a profile of it found, in 8.1's order: does a test belong in this tier, then can it be made faster. **No budget changes, no build setting changes, and no story loses what it checks.** It won't reach 30 alone, and isn't asked to: other plans take other causes, and what's left is read after landing.

## What's wanted

Five tasks at most, one per numbered item. Each item stands alone, so one that proves harder than it reads doesn't hold the others: say so in its report and let it land short.

**1. Sweeps in the focused suite (8.1, question 1).** `jig/docs/design/domain/testing.md`, 5 draws the line: "the focused suite crashes after a few chosen commits and checks the story still ends as it should; the fuzzy suite sweeps every commit of the scenario", and 9 says the same. These focused tests of `jig/tests/core` sweep:

| Test | File | Share of a full focused run |
|---|---|---|
| `a_deletion_cut_after_any_commit_ends_the_same` | `tests/suite/deletion.rs` | 1.6 CPU s |
| `every_startup_sign_in_chat_and_assignment_commit_restarts_on_either_side_of_durability` | `tests/suite/faults.rs` | 0.4 |
| `every_commit_of_a_turn_effect_and_terminal_survives_both_crash_cuts` | `tests/suite/faults.rs` | 0.3 |
| `decisions_keep_their_offered_fence_and_room_across_each_restart_cut` | `tests/suite/decision_reads.rs` | 0.2 |
| `a_failed_commit_at_every_scenario_boundary_stops_before_any_later_output` | `tests/suite/faults.rs` | 0.15 |

Each becomes a few chosen cuts, named for why they're chosen (testing.md, 10: "the commits around an effect, a claim and an answer are the likeliest to show a bug"). The whole sweep lives in fuzzy: name, per test, the fuzzy test that already holds it (`jig/tests/conformance/tests/suite/fuzzy/deletion.rs`, `cuts.rs`, `scenarios.rs` are the first to read), or move the sweep under `fuzzy::` if none does. A table in `plan.md` says which, per test. A test's name follows what it now checks. `jig/tests/store`'s cuts stay: testing.md, 9 places them in focused.

**2. `stock_profiles_hold_a_day_of_deciding_requests` (`jig/tests/core/tests/suite/settings.rs`, 1.7 CPU s).** It drives 10,000 requests through two profiles, 8,000 of them through `Standard`. What it checks is that each stock profile's room holds a day of a stated size. Decide how little shows that: most likely the arithmetic for both profiles as a table test on the stock figures, with one world at `Small`'s bound and its restart. Whatever else `Standard`'s drive shows, if anything, goes to a fuzzy test. Say which, and why, in the report.

**3. The core observer's view (`jig/testing/jig-core-kit/src/conformance.rs`; `jig/docs/development/debt/D034-world-replay-cost.md`).** A median core-world test spends 18% of its instructions in `Observer::durable` and 11% in `store_view`, built 91 times in one short story: for every core event in `inbound` (38 times), at each `submitted` (26) and at each new commit (26). Only `durable` and `fresh_mark` skip when nothing was applied. Build a view only when something will read it, or keep one until the rows change. D034 warns that `Got` changes the rows outside commits, so `applied` alone doesn't name their state. Then, as far as it's simple, `jig_conformance::core::Observer::durable_at`'s whole `snapshot`. D034's keyed row feed is **not** required. Update D034, `jig/docs/design/domain/testing.md`, 10, and `testing.status.md` to what's true after. These kits serve the client-protocol, conformance and ops worlds too, so keep every adopter's behaviour.

**4. The restart oracle's reads (`jig/testing/jig-core-kit/src/oracle.rs`, `jig/testing/jig-test-store-files/src/store/readback.rs`, `jig/crates/jig-protocol-store/src/pages/reader.rs`, `retained.rs`).** `World::close` is 44% of a median core-world test and 29% of a client-protocol one. **"Every story ends with a cold restart" (`jig/docs/design/domain/engine.md`, 15) stays, and no story opts out:** this is its cost, not its place. By reading:
- `close` reads the whole store four times, `retained()` and `readback()` on each side of the restart; a short story makes 64 `retained_page` calls, 13% of the test.
- `walk_map` issues one `skein_kv` load per row, with `max: Page { rows: 1, .. }`, then asks again past that row.
Reuse what a read already holds, ask the store for a page rather than a row where its bounds, order and cursors stay exactly as they are, and take the oracle's pages as large as the limits allow. `jig-protocol-store` is product code: its step tests and every bound stay, and a change there must keep each byte a reader sees. If a page load can't keep them, leave `walk_map` alone and say why.

**5. The channel referee's root check (`jig/testing/jig-channel-kit/src/referee.rs`, `observe_roots`).** A median `jig-channel-world` test spends 21% of its instructions on `format!("{:?}", Roots(..))` of every root, 44 times in one story, so that `Rules::roots` can look for a credential grant in the text. Keep the rule and `a_grant_value_planted_in_a_root_s_state_is_caught`. Make it cheaper by what can actually change: format only what can hold a grant, or check where a grant can newly arrive and at the story's end, with `fuzzy::channel` keeping every step if that's what's given up in focused. Say in `plan.md` what focused gives up, if anything, and where it's still held.

## Limits

- **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5), and is complete when it lands.** No slow tier, walk or throwaway driver, and no measuring of tests or processes: the queue's record is read after landing. `walk = false`. Effort medium.
- **Stay out:** `skein/testing/skein-heap` and `skein/testing/skein-world/src/stream.rs` are another plan's. `jig/tests/ops/src/joint`, `jig/testing/jig-test-service` and `jig/testing/jig-client-kit/src/joint` have a repair under way in another plan. `Cargo.toml`'s profiles, `tools/tiers/budgets.toml`, nextest's configuration.
- A test moves between tiers only under item 1's and 2's rules: what it checks decides its place, never its cost.
- **Overlaps:** `model-catalogue-2` changes `jig-domain-core`'s documents and may touch `oracle.rs`'s `durable`. Expect a catch-up, and keep this plan's edits to `oracle.rs` in one task.

## Sources

Too large for a source, so read by name: `jig/testing/jig-channel-kit/src/referee.rs` (`observe_roots`, `Rules::roots`, `Roots`); `jig/testing/jig-conformance/src/core.rs` (`durable_at`); `jig/tests/core/tests/suite/deletion.rs` and `settings.rs` (the tests above); `jig/docs/design/domain/testing.md`, 5, 9 and 10; `jig/docs/design/domain/engine.md`, 15.

- `jig/testing/jig-core-kit/src/conformance.rs`
- `jig/testing/jig-core-kit/src/oracle.rs`
- `jig/testing/jig-test-store-files/src/store/readback.rs`
- `jig/crates/jig-protocol-store/src/pages/reader.rs`
- `jig/crates/jig-protocol-store/src/retained.rs`
- `jig/tests/core/tests/suite/faults.rs`
- `jig/docs/development/debt/D034-world-replay-cost.md`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **A helper the change makes unused goes in the same task.**
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
