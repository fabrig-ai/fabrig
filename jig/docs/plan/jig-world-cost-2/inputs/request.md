# jig-world-cost-2: the second round of what jig's focused worlds cost

jig's focused suite reads about 80 CPU s against its 30 (`skein/docs/foundation/testing-strategy.md`, 8), after `jig-world-cost` and `skein-world-costs`. This plan repairs what a new profile of `main` found, in 8.1's order. **No budget changes, no build setting changes, and no story loses what it checks.** It won't reach 30 alone, and isn't asked to.

## What's wanted

Five tasks at most, one per numbered item, each able to land short: if one proves harder than it reads, say so in its report and stop. **Pace: at most two tasks at once, by `needs`:** items 1, 3 and 5 in one chain, in that order, and 2 then 4 in the other.

**1. Setup that every test of a world pays (8: "A fixture many tests share is cheap to build, or a constant").** Shares of a median test's instructions: `jig_core_kit`'s `World::new` 24% of a core-world test, `Story::configured` 15% of a client-protocol one, and in a channel one `jig_client_kit::test::fixture_with` 8.7% and `jig_channel_kit::fixtures::engine_configuration` 7.8%. Find what they do repeatedly or expensively for what the test then reads: building a document to parse it again, a configuration copied, a startup step a fixture could hand over. Fix the cause. A world still starts the way the design says, and a story that tests startup still runs it.

**2. `naming` formats every row (`jig/testing/jig-conformance/src/core/naming.rs`).** `naming` opens with `row: format!("{key:?}")`, on every durable row at every new commit: 8.8% of a median core-world test, nearly all of it `Key`'s `Debug`. Keep what's needed to name a row, a typed key or a borrow, and format it when a failure prints it. The referee's messages stay as readable as they are.

**3. The store world's sweeps (8.1, question 1).** `jig/docs/design/domain/testing.md`, 5: "the focused suite crashes after a few chosen commits and checks the story still ends as it should; the fuzzy suite sweeps every commit of the scenario", and 9: "the focused suite runs a few cuts and seeds per scenario, the fuzzy suite sweeps". 9 places the store's generation, patch, check and frame scenarios in focused, which stays; what moves is sweeping every cut of them. In `jig/tests/store/tests/suite`:

| Test | CPU s in a full focused run |
|---|---|
| `upgrades::patching::drop_copy_and_switch_recover_after_every_mutation` | 0.74 |
| `upgrades::check::cuts_across_check_and_switch_recover_whole_maps` | 0.43 |
| `generations::strays_cut_anywhere_are_gone_at_the_next_start` | 0.42 |
| `patching::every_patch_write_cut_recovers_all_records_and_no_stray` | 0.39 |
| `generations::a_first_start_cut_anywhere_is_whole_at_the_next_start` | 0.15 |
| `upgrades::frames::every_file_cut_in_a_patched_read_preserves_all_files` | 0.13 |
| `pending_switch::each_cut_around_the_switch_leaves_one_generation_whole` | 0.11 |

Each keeps a few cuts, named for why they're chosen, in focused. The whole sweep goes under `fuzzy::` in the same world, unless a fuzzy test already holds it: say which, per test, in a table in `plan.md`. Rows of one table test stay one test. `jig-world-cost`'s `focused-cuts` did the same for the core world, and its names are the pattern. Update testing.md, 9 where it reads otherwise.

**4. The protocol world's polling (`jig/testing/jig-client-protocol-kit/src/net.rs`).** `Net::advance` is 15% of a median client-protocol test, and `check` 10%. `Net::advance` walks every socket at every step, and `check` feeds `socket.engine.received` and `socket.response` into their record readers each time. The joint fuzzy world pays too: 186,412 `Net::advance` calls in one story, measured before `ui-fuzzy-joint`. Do the work when a socket's bytes changed, and read only what's new. Every cut, every record check and the order of events stay as they are.

**5. Asking admission again and again (`jig/crates/jig-domain-core/src/settings/charters.rs`, `jig/testing/jig-test-domain/src/lib.rs`).** `Core::context_ready` is 14% of a median channel test: 468 calls in one story, each a full `run_admission`, from `take_due` by `continue_ready` and `due_ready` by `Domain::quiescent`, nearly 1,000 calls each, on tasks whose answer hasn't changed. This is product code and a real engine pays it as well. Make the common case cheap: cheaper checks first, or a stored answer **only if** every input that changes it visibly clears it. If that can't be shown from the code, leave it and say why: a stale admission is a bug, and this saving isn't worth one.

## Limits

- **A plan runs the focused and fuzzy suites alone (`AGENTS.md`, 5), and is complete when it lands.** No slow tier, walk or throwaway driver, and no measuring of tests or processes: the queue's record is read after landing. `walk = false`. Effort medium.
- **Stay out:** `Cargo.toml`'s profiles, `tools/tiers/budgets.toml`, nextest's configuration, how test binaries link. `skein/testing/skein-heap` and `skein/testing/skein-world/src/stream.rs` have just been repaired. `jig/docs/development/debt/D034-world-replay-cost.md`'s row feed isn't asked for.
- A test moves between tiers only under item 3's rule: what it checks decides its place, never its cost.
- **"Every story ends with a cold restart" (`jig/docs/design/domain/engine.md`, 15) stays,** and no story opts out.

## Sources

Read by name where too large: `jig/testing/jig-core-kit/src/world/constructors.rs` (`World::new`, `empty`); `jig/testing/jig-client-protocol-kit/src/client.rs` (`Story::configured`, `configured_peers`); `jig/testing/jig-client-kit/src/test.rs` (`fixture_with`); `jig/testing/jig-channel-kit/src/fixtures.rs` (`engine_configuration`); `jig/crates/jig-domain-core/src/settings/charters.rs` (`context_ready`, `take_due`, `due_ready`); `jig/docs/design/domain/testing.md`, 5 and 9; `jig/tests/store/tests/suite` (the tests above).

- `jig/testing/jig-conformance/src/core/naming.rs`
- `jig/testing/jig-client-protocol-kit/src/net.rs`
- `jig/testing/jig-core-kit/src/world/constructors.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **A helper the change makes unused goes in the same task.**
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
