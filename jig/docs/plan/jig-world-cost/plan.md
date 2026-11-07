# jig-world-cost: the plan

jig's focused suite reads about 93 CPU s against its 30. This plan repairs what a profile of it found, in the order of `skein/docs/foundation/testing-strategy.md`, 8.1: does a test belong in this tier, then can it be made faster. No budget or build setting changes, and no story loses what it checks. It won't reach 30 alone and isn't asked to: what's left is read after landing.

## Parts and tasks

One task per item of `inputs/request.md`. Each stands alone and may land short.

| Item | Part and task | Needs | What it does |
|---|---|---|---|
| 1 | `focused-cuts` | | Five sweeps of `jig/tests/core` move under `fuzzy::`; focused keeps cuts chosen for why |
| 2 | `stock-room` | | One world at `Small`'s bound in focused; `Standard`'s drive in fuzzy |
| 3 | `observer-view` | | The core observer makes a store view only for a reader; `snapshot` as far as it's simple |
| 4 | `oracle-reads` | | `close` walks the store twice, not four times; `walk_map` asks for pages |
| 5 | `root-checks` | | The channel referee formats the roots where a grant can arrive; every step in fuzzy |

## Critical path

None: all five start at once and none needs another. `observer-view` is the longest, at about 75 minutes.

## Sweeps

Item 1's table. **No existing fuzzy test holds any of the five,** read at this cut:

- `jig/tests/conformance/tests/suite/fuzzy/deletion.rs` and `cuts.rs` cut with `Cut::AfterCommit`, durably only, so neither has a submission lost before it's durable. `deletion::run` makes no comparison with an uncut run, and `cuts.rs` doesn't restart twice or slow the pages.
- `scenarios.rs` sweeps the shared scenarios, none of which is one of these stories.
- `jig/tests/core/tests/suite/fuzzy/faults.rs`'s `random_host_worlds_draw_link_store_and_cold_restart_faults` fails one drawn next commit, with no sweep of a scenario's boundaries. `random_drawn_commit_cuts_preserve_whole_turns_and_atomic_effect_decisions` cuts one turn's commit.

So each sweep moves whole, keeping its name and every check.

| Test today (`jig/tests/core`) | Focused test now, and its chosen cuts | The whole sweep now |
|---|---|---|
| `deletion::a_deletion_cut_after_any_commit_ends_the_same` | `deletion::a_deletion_cut_around_its_decision_and_last_step_ends_the_same`. `Submitted(first)`: the decision is lost, and the requester asks again. `Durable(first)`: the decision is kept with no step done, and the restart finishes it before opening. The middle commit, durable: between two family steps. `Durable(last)`: the last step | moved: `fuzzy::deletion::a_deletion_cut_after_any_commit_ends_the_same` |
| `faults::every_startup_sign_in_chat_and_assignment_commit_restarts_on_either_side_of_durability` | `faults::the_sign_in_chat_and_claim_commits_restart_on_either_side_of_durability`. The commit saving the sign-in (an answer to a party) and the chat's live row, either side, in the worker world. The assignment's claim, either side, in both the worker and the engine worlds | moved: `fuzzy::faults::every_startup_sign_in_chat_and_assignment_commit_restarts_on_either_side_of_durability` |
| `faults::every_commit_of_a_turn_effect_and_terminal_survives_both_crash_cuts` | `faults::an_effect_and_its_terminal_survive_both_crash_cuts`. The commit saving the effect's outbox row, and the one saving the terminal, either side | moved: `fuzzy::faults::every_commit_of_a_turn_effect_and_terminal_survives_both_crash_cuts` |
| `faults::a_failed_commit_at_every_scenario_boundary_stops_before_any_later_output` | `faults::a_failed_effect_or_terminal_commit_stops_before_any_later_output`. The same two commits failed | moved: `fuzzy::faults::a_failed_commit_at_every_scenario_boundary_stops_before_any_later_output` |
| `decision_reads::decisions_keep_their_offered_fence_and_room_across_each_restart_cut` | `decision_reads::decisions_keep_their_offered_fence_and_room_across_a_restart_at_the_decision_and_the_turn`. A restart after the decision, for a message and for a proposal; a restart after the turn that answers it, for a message | moved: `fuzzy::decision_reads::decisions_keep_their_offered_fence_and_room_across_each_restart_cut`, all eight runs |

`jig/tests/store`'s cuts stay: testing.md, 9 places them in focused. The task reports any row that lands differently.

## What the channel's focused suite gives up

Item 5. **The rule stays:** `Rules::roots` and `a_grant_value_planted_in_a_root_s_state_is_caught` are unchanged, and `jig/docs/design/protocol/channel.md`, 12 holds it.

Focused formats a host's roots when a new grant shows on the wire, at that host's next root step, before the referee itself replaces a host's service, and at the story's end (`roots_at_end`).

**Given up in focused:** seeing a grant's value that a root holds only between those checks. That means a root that takes a value up later than its next step and has dropped it before the story's end, or a host killed while holding one, since a kill isn't a check.

**Where it's still held:** `fuzzy::channel`, at each root step:

- `drawn_channel_worlds_keep_the_referee_s_rules`: `Chat`, `Progress`, `Call`, `HeldCommit`, `MidFrame` and `Restart`, under drawn faults.
- `a_replaced_grant_s_roots_are_checked_at_every_step`, new: `Key { replaced: true }` and `ApplicationGeneration`, the stories where a grant arrives mid-run. `drawn` doesn't draw those, which is why it's added.

**Not held at each step in any tier:** the stories `drawn` doesn't draw and the new test doesn't name, such as `Facts` or the refusals. Their grants arrive by the same `Assignment` and `Granted` frames the held stories use, so what's lost is the same transient case, on those stories alone. The arrival and end checks still run on them.

## Decisions

- **Item 2's table test already exists.** `profiles_retain_a_day_of_deciding_answers_with_fivefold_room` (`jig/crates/jig-protocol-config/src/tests/profile.rs`) holds the arithmetic for both profiles. Adding another would repeat it, in a directory another plan is changing.
- **Item 2's fuzzy test has a reason.** What only `Standard`'s drive shows is that its whole room of answers stays under `REQUEST_HEAP_CEILING`. `Small`'s world holds a quarter of those rows, so by reading the code, not by measuring, it wouldn't come near a ceiling recorded over both workloads.
- **Item 3 builds on read and keeps nothing.** The request allows keeping a view until the rows change. But a view borrows the store, `Got` changes rows outside commits, and an owned copy would cost more than it saves. So `inbound` and `submitted` take a closure, and the kit's own signatures don't change, which keeps the stay-out files out of it.
- **Item 3's `submitted` reads one commit.** The core reads only `store.pending.back()`, and the rows only for two kinds of erase, so it takes that commit and a closure. `Store.named` goes with it.
- **Item 4's oracle pages are already as large as the limits allow.** `component_limits` sets `page.rows` to 2, and `retained_page` clamps to it. Raising that would change what paging every story exercises, so it stays. The saving there comes from walking half as often, and in these worlds a load holds at most two rows. Production's readers gain more from `walk_map` than the worlds do.
- **Item 4's page load keeps every byte, by reading `skein_kv`'s `Map::page`:** it refuses only when the first row doesn't fit, and it names the first key it didn't return. The per-row steps stay as they are, and `a_walk_by_pages_matches_a_walk_by_rows` holds it against today's walk. `load_map` keeps its one-row meaning for `transcripts/`.
- **Item 5 can't format less:** `fmt_roots` is in `jig/testing/jig-test-service`, in the stay-out list. So it checks where a grant can newly arrive.
- **Design written at this cut:** `jig/docs/design/domain/testing.md`, 5 (an adapter's view) and `jig/docs/design/protocol/channel.md`, 12 (when each suite looks for a grant's value), each with a **Not yet** entry its task removes. `observer-view` also writes `testing.md`, 10 and D034, since their words depend on how far `snapshot` goes. That's a stated departure in its spec, as is `root-checks`' one status edit.
- **`testing.md`, 10's open question on how many cuts stays open:** the request doesn't close it, and this plan answers it for five tests only.
- **Effort is medium throughout, as the request says.** `walk = false` for every slice.
- **Nothing here is measured.** No spec names a slow tier, a timer or a CPU figure. What this plan saved is read from the queue's record after landing.
- **The moved sweeps add to fuzzy's CPU.** The request's shares of the five sweeps and `Standard`'s drive come to about 4 CPU s, against a fuzzy budget of 2 minutes. Whether that fits is read after landing too.

## Overlaps

**Inside this plan,** two pairs share a crate, on different files, so neither waits:

- `focused-cuts` and `stock-room` in `jig-core-world`.
- `observer-view` and `oracle-reads` in `jig-core-kit`: `conformance.rs` and `oracle.rs`.

**With drafts beside this one:**

- `jig/testing/jig-core-kit/src/oracle.rs`: `model-catalogue-2` may touch `durable`. `oracle-reads` is this plan's only task editing the file, and stays out of that function. Expect a catch-up.
- `jig/docs/design/protocol/channel.status.md`: `model-catalogue-2/models-proof` also edits it. This plan adds and then removes one line.
- `jig/crates/jig-protocol-store/src/tests/`: `model-catalogue-2/catalogue-store` owns `settings.rs` and `models.rs`. `oracle-reads` adds to `readers.rs` alone.
- `jig/tests/core/tests/suite/main.rs`: drafts adding stories may add `mod` lines. `focused-cuts` adds two, inside `mod fuzzy`.
- `limits-worlds`, `limits-fixtures`, `limits-heavy` and `browser-cap` are cuts in flight, and their files are unknown at this cut.
