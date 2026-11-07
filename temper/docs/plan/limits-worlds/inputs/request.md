# limits-worlds: the real loop and the worlds on it, within the suites' CPU limits

temper's fuzzy suite has a limit of 120 CPU s and reads about 174, and its focused suite a limit of 30 and reads about 336 (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). The worlds that run whole deployments are most of the first and a quarter of the second. This plan is 8.1's repair for them. Other plans take the fixtures of the many small tests, and the other heavy tests.

Medians of three gate runs near `50053b79a2`, CPU s, user plus system with descendants:

| Suite | World | Tests | CPU s |
|---|---|---:|---:|
| fuzzy | `temper-real-loop-world` | 22 | 85 |
| fuzzy | `temper-conformance-world` | 15 | 44 |
| fuzzy | `temper-engine-domain-world` | 26 | 26 |
| fuzzy | `temper-one-process-world` | 4 | 10 |
| focused | `temper-one-process-world` | 49 | 29 |
| focused | `temper-real-loop-world` | 241 | 30 |
| focused | `temper-system-world` | 42 | 8 |
| focused | `temper-playground-world` | 6 | 5 |
| focused | `temper-conformance-world` | 29 | 3.5 |

Largest in fuzzy: `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` 8.7, `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` 8.2, `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` 7.6, `fuzzy::green_light::a_green_light_in_the_inbox_lets_the_change_land` 7.4, `fuzzy::case_5_4::...` 6.8, `fuzzy::case_6_4::...` 6.7. Largest in focused: `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` 4.7, `composed::a_long_chat_recovers_from_unusable_resume_history` 4.5, `github_repair::...` 3.8 and 3.3, `m15::whole_run_and_renewal_scan_every_fake_issued_token_byte` 3.1, `github_land::a_chats_change_lands_on_connected_github` 3.0.

## What's wanted, in this order

**1. What do these tests belong to?** Strategy 8: fuzzy tests "look for what no scenario names". `temper/tests/real-loop/tests/suite/fuzzy/` is mostly the reference case (`temper/docs/design/domain/testing.md`, 5.3) and `repair`, `green_light`, `landing` and `program_landing`: scripted stories, with no seed, in the fuzzy suite because they were "past the guide". 8.1 says that isn't a reason. And 2.4 keeps the real loop to "one end-to-end path per feature, and each behaviour and corner case tested in the lower world that owns it, never here". So, for each story, one row in `plan.md`: what it checks that no other test holds, by reading. A story whose behaviours are all held elsewhere goes, naming the holders. A story that alone holds a behaviour stays where it is for now, unless the behaviour can have a lower story first, at a small cost: **none moves into the focused suite in this plan,** and the report lists what stays with its numbers, for the user. If 5.3's table or 6.1's guides change as a result, change them in the same task. The process tier also runs this case (`temper/docs/design/protocol/README.md`, 15.6): read what it holds before calling anything lost.

**2. Why does one story cost seconds?** Profile `repair`, `case_6_4`, `stories::a_ci_repair_is_crashed_at_every_live_commit`, a focused real-loop story, and `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte`. These worlds share an engine, a store, a fake forge and a fake LLM, so one cause may cost in all of them: repeated encoding, a scan, setup rebuilt per restart, a poll where an event would do, git run where a fixture would do. Fix the largest causes the profiles show, largest first, and stop when the next one's gain is under about 2% of a story.

**3. A crash at every live commit.** `temper-conformance-world`'s `fuzzy::stories` and `fuzzy::shared` crash a story at each commit and replay. If each replay repeats work that a shared prefix could hold, that's a cause. They keep checking every commit: no sampling.

**4. What's left.** A table in the report: each world's tests before and after, in instruction counts for the ones profiled, and the causes found and not fixed.

Effort as each task needs; at most about eight tasks. `walk = false`.

## Sources

Read by section, too large to be sources: `temper/docs/design/domain/testing.md`, 2.4, 5.3 and 6.1; `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.

- `temper/tests/real-loop/Cargo.toml`
- `temper/tests/real-loop/tests/suite/fuzzy/repair.rs`
- `temper/tests/real-loop/tests/suite/fuzzy/case_6_4.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver. A new story in a slow tier is written and compiled, not run.
- **8.1's order, always** (`skein/docs/foundation/testing-strategy.md`, 8.1): does the test belong in this suite; if it does, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. What a test checks decides its place, never its cost. Moving a test to escape a budget is raising it.
- **Find the cause with a profile before changing anything** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. If a profile shows a cost no test can avoid, say so in the report, and leave it.
- **Nothing is lost.** A test that goes names what holds its behaviour now, by test name, in `plan.md`, and that holder exists and passes first. A test past its guide isn't merged with another or named an exception to make room (`temper/docs/design/domain/testing.md`, 6.1).
- **A product change is fine where the profile puts the cause,** at the size the cause needs, with its own focused test, and no change of behaviour. A cause in skein, smith or jig is fixed there if small and clearly theirs; otherwise report it.
- **Decide, don't ask.** A stored format or protocol may change freely, with no compatibility. A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
