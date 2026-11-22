# Failure worlds: the room F7 made, measured

2026-11-22. temper's focused (`default`) and `fuzzy` suites, by layer and by affected binary, from the suites' own reports, before and after F7 of `temper/docs/design/draft/failure-worlds.execution.md`, section 1: the fuzzy copies of focused tests retired, and the real-loop and one-process stories cut to what only their tiers show, their other assertions held below (`temper/docs/design/testing/checks.md`, section 1). This same-machine pair supplies the measurements missing from the plan in `failure-worlds-budget.md`. After's focused end-to-end result is partial; the other layers are complete. No figure in `tools/tiers/budgets.toml` moves: the transfers at the end remain proposals for the user.

Only temper's tests change in F7, so only temper's packages are measured. The other four trees run the same code before and after.

## Method

- **Selection:** every package under `temper/` that `cargo metadata --no-deps --format-version 1` lists, through `tools/tiers/one-process default -p …` and `tools/tiers/one-process fuzzy -p …`, from the checkout's root, with `TEMPER_INTERNAL_BUNDLE` and `TEMPER_BUNDLE` both `none`. The hold is taken as the tool takes it, never bypassed. No slow profile, browser, walk or throwaway driver runs.
- **Figures:** each run's `results.json` (`tools/tiers/design.md`, 11.8): CPU is user plus system seconds of the `tests`, `fresh` and `alone` processes, waited-for descendants included, placed by the record's own `tree` and `layer`, and checked against `cpu-summary`'s totals. No test's CPU is apportioned from a shared process. The ceiling is `tools/tiers/budgets.toml` with its moves applied once (`tools/tiers/budgets.md`, sections 2 and 10), as `cpu-summary` gives it. Instruction counts stand beside CPU where the run counted.
- **Each run's record:** its commit, machine, lanes, wall, and every failed, starved, cancelled or unobserved test by name. A binary with one is **partial**, and its figure a lower bound.
- Both sides run back to back in one session on **hel1-19**, using `TEMPER_INTERNAL_BUNDLE=none TEMPER_BUNDLE=none tools/tiers/one-process --out target/<dir> --layers all default -p …` and the same command for `fuzzy`: `--layers all` retains real-loop and end-to-end, otherwise omitted from these profiles (`tools/tiers/design.md`, 11.9), without selecting a slow profile.
- Superseded: three hel1-8 runs on 2026-11-22, whose logs were lost when the session moved machines, observed all-layer CPU lower bounds of 841.292331 s (Before default, exit 100), 326.851411 s (Before fuzzy, exit 0, one starved test), and 844.956877 s (After default, exit 100); no After fuzzy ran there. The full pair below replaces them, not a cross-machine combination.

## Before

Measured at main's tip **`e0324a990e593d7947f694ab3da70b837686f961`**, on **hel1-19** (the identifying suffix of the observed hostname), on 2026-11-22. This replaces the unmeasured attempt at `ac898e16f970`: main's tip contains subsequent main work and is the correct baseline for F7.

At 18:39:50 UTC, `date -u`, hostname and the initial After head were recorded. Both `git merge-base --is-ancestor e0324a990e59 HEAD` and the corresponding check for `31d6999f2bc3` returned 0; `git diff --stat 31d6999f2bc3 HEAD` was empty. With a clean tree, the checkout switched to Before, ran default then fuzzy, and returned to the initial detached After head with `git switch --detach -`. The document was written only after all four runs.

Both sides' discovery selected the **same 132 packages**, every manifest under `temper/` from `cargo metadata --no-deps --format-version 1`; no package was added or removed. Package lists and metadata are in `target/failure-worlds-pair/{before,after}-packages.txt` and `{before,after}-metadata.json`. Before discovery started at 18:39:54 and returned at 18:39:55 UTC, exit 0; After discovery started and returned at 18:49:58 UTC, exit 0. The same Cargo target directory was reused for incremental rebuilds. No disk check or hold was bypassed.

Run directories are relative to the checkout root. Start/return times below are from `date -u` on 2026-11-22, not inferred from suite wall:

- `target/failure-worlds-before-default-20261122T1839` — started **18:39:57 UTC**, returned **18:47:51 UTC**, exit **0**.
- `target/failure-worlds-before-fuzzy-20261122T1847` — started **18:47:54 UTC**, returned **18:49:53 UTC**, exit **0**.

Both used **four lanes** (`run.json`). Nextest's JSON listing is captured in each `command-output.txt`; `plan.json` retains the selected names per binary. `results.json` records the Before commit and `selected_layers = ["step", "domain", "system", "machine", "protocol", "io", "simulated", "real-loop", "end-to-end", "checks"]`. All four runs counted instructions (`counting = {"counted": true, "why": null}`). Suite wall is first process start through last process end, matching `run.json`, not build/discovery/hold time. Before default has 157 `tests` processes; Before fuzzy has 27. Neither has `fresh`, `alone`, reference or empty processes.

| Suite | Selected / passed tests | Suite wall s | Exit status | Instructions | temper CPU s, all layers | temper CPU s, replaying layers only |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Focused (`default`) | 15632 / 15632 | 267.789188 | 0 | 4539043816393 | 651.332322 | 504.083931 |
| `fuzzy` | 143 / 143 | 115.219796 | 0 | 1608556751771 | 249.458050 | 159.712656 |

Replaying totals exclude `real-loop` and `end-to-end`. For each of the four directories, independent sums of user plus system seconds, both per layer and in total, agree with `tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --json` on that directory (all four summaries exited 0). Instruction sums agree too. Ceilings and room below use those summaries; room is ceiling minus CPU, not main's held budget or a retirement credit. CPU and wall are rounded to six decimals, instruction and test counts are exact. No process means no summary row: absence contributes no CPU to a sum but is not a measured zero.

### Focused (`default`)

| Layer | CPU s | Ceiling s | Room s | Instructions |
| --- | ---: | ---: | ---: | ---: |
| step | 163.032626 | 254.600000 | 91.567374 | 1483611276918 |
| domain | 147.807995 | 142.800000 | -5.007995 | 749234369554 |
| system | 58.982337 | 111.000000 | 52.017663 | 351564882805 |
| machine | 1.665197 | 4.000000 | 2.334803 | 13935277365 |
| protocol | 9.518380 | 9.000000 | -0.518380 | 66764418661 |
| io | no process | none | unmeasured | not counted: no process |
| simulated | 123.077396 | 470.000000 | 346.922604 | 866344268121 |
| real-loop | 84.817032 | 76.700000 | -8.117032 | 629479844536 |
| end-to-end | 62.431359 | 36.900000 | -25.531359 | 378109478433 |

### Fuzzy

| Layer | CPU s | Ceiling s | Room s | Instructions |
| --- | ---: | ---: | ---: | ---: |
| step | 24.546697 | 20.600000 | -3.946697 | 254061797945 |
| domain | 87.076119 | 85.400000 | -1.676119 | 433464785871 |
| system | 10.692076 | 13.000000 | 2.307924 | 73060796570 |
| machine | 0.369603 | 2.000000 | 1.630397 | 2132190540 |
| protocol | 0.140504 | 2.000000 | 1.859496 | 560729369 |
| io | no process | none | unmeasured | not counted: no process |
| simulated | 36.887657 | 196.000000 | 159.112343 | 273089069561 |
| real-loop | 89.745394 | 183.000000 | 93.254606 | 572187381915 |
| end-to-end | no process | 1.000000 | unmeasured | not counted: no process |

### Affected binaries and completeness

Every selected Before test passed. **Failed, crashed, starved, cancelled and unobserved selected tests: none** (`unobserved = []`, `cancelled = null`, every process exit 0, no signal or leak). No Before binary is partial. The zero-selected fuzzy library has no process, not an incomplete execution. Slow-profile tests were not selected and are not claimed as observations.

| Binary | Focused CPU s | Focused selected tests | Focused instructions | Fuzzy CPU s | Fuzzy selected tests | Fuzzy instructions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `temper-engine-domain-world::suite` | 129.735074 | 1363 | 646362200142 | 37.993650 | 39 | 203885124411 |
| `temper-engine-forge-world::suite` | 7.099534 | 358 | 37677587963 | 7.061634 | 10 | 41187641560 |
| `temper-worker-domain-world::suite` | 0.254904 | 76 | 1389110500 | 0.404189 | 3 | 2617748639 |
| `temper-system-world::suite` | 12.000933 | 44 | 101376578580 | 1.390507 | 1 | 7386889556 |
| `temper-one-process-world::suite` | 78.853159 | 64 | 532507761252 | 36.775320 | 9 | 272134298912 |
| `temper-one-process-world` (library tests) | 0.640810 | 24 | 5742578464 | no process | 0 | not counted |
| `temper-real-loop-world::suite` | 43.824011 | 31 | 287306063900 | 89.745394 | 18 | 572187381915 |

Each Before real-loop `fuzzy::` test is listed in full, including stories retired or renamed at After. These are individual **wall times**, not apportioned CPU; every row passed.

| Fuzzy test in `temper-real-loop-world::suite` | Wall s |
| --- | ---: |
| `fuzzy::case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | 8.539170 |
| `fuzzy::case_4_4::the_pace_question_reaches_the_person_and_its_answer_is_kept` | 4.007378 |
| `fuzzy::case_4_5_limited::a_limited_account_holds_its_work_until_its_reset` | 3.924474 |
| `fuzzy::case_4_5_refused::a_refused_key_holds_its_work_until_it_is_replaced` | 4.207800 |
| `fuzzy::case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | 4.795330 |
| `fuzzy::case_5_1::a_held_director_keeps_the_owners_question_and_its_fresh_run_reads_the_new_rules` | 4.278432 |
| `fuzzy::case_5_2::a_stopped_owner_holds_its_delegates_and_its_release_resumes_them_and_its_news_waits_for_its_release_without_a_park_or_a_retired_charter` | 15.342253 |
| `fuzzy::case_5_4::a_new_direction_cancels_x3_under_its_closed_pull_and_bs_landing_news_waits_for_its_release_with_an_overlap_wake_and_a_closing_comment` | 10.617147 |
| `fuzzy::case_5_5::an_owners_answered_plan_starts_a_sub_owner_under_a_deadline_that_cannot_delegate_without_the_deadline_firing_or_a_fresh_run_only_at_the_milestones_end` | 4.133929 |
| `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | 11.170064 |
| `fuzzy::case_6_5::a_budget_question_climbs_two_owners_to_the_persons_inbox_and_a_raised_investigation_hands_its_report_to_an_advisor_without_the_director_asking_the_person` | 4.861616 |
| `fuzzy::case_6_6::a_kit_change_lands_before_the_apps_early_repin_without_relay_without_the_a3_a4_epilogue` | 7.402652 |
| `fuzzy::case_6_7::a_sub_owner_past_its_deadline_is_held_at_its_next_run_and_escalated_through_its_owner_to_the_director_who_splits_it_by_amendment_and_release_without_a_grace_for_its_delegates` | 4.780936 |
| `fuzzy::case_6_8::m1s_end_message_starts_m2_beside_a_tester_without_forge_effects_whose_test_lands_only_through_a1s_change` | 5.137227 |
| `fuzzy::green_light::a_green_light_in_the_inbox_lets_the_change_land` | 6.994810 |
| `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | 3.304419 |
| `fuzzy::program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | 3.183070 |
| `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | 8.492994 |

## After

Measured at **`fef26ce96c4c8cd1725ed58d65ac06eeb3208209`**, on the same **hel1-19**, in the same session and target directory, immediately after Before. Its tree is identical to `31d6999f2bc376736517b01c782a7dc10cdfe536`: no code or Markdown difference above that previously measured goal tip. It includes the duplicate retirements, the two library referee controls, the composed-system holders, the worker hosting repair and the reconciled twelve-story real-loop library.

The earlier After run at `9f9c7462852131328fbb55452ffdcace568dd471`, hel1-18, 2026-11-22, is superseded by this same-machine pair; its all-layer totals were default **687.486916 s** and fuzzy **225.069200 s**, for reference only.

- `target/failure-worlds-after-default-20261122T1850` — started **18:50:01 UTC**, returned **18:54:12 UTC**, exit **100**.
- `target/failure-worlds-after-fuzzy-20261122T1854` — started **18:54:18 UTC**, returned **18:55:24 UTC**, exit **0**.

Both used **four lanes**, the same 132 packages and `selected_layers` as Before. After default has 157 `tests` processes and one automatic `alone` diagnostic; After fuzzy has 26 `tests` processes. There are no `fresh`, reference or empty processes. The nextest listings have 28 engine-domain, three one-process and twelve real-loop fuzzy tests: no engine-domain `fuzzy::removal::` or `fuzzy::reset::`, no four fixed-cut one-process `fuzzy::upgrade::power_lost_at_*` wrappers, and no real-loop 4.4, 4.5, 5.5, 6.5 or 6.7 story.

**Partial convention:** `≥` is an observed lower bound; the corresponding room is an upper bound (`≤`). After default end-to-end is partial, so its all-layer total and instruction total are lower bounds too. The differences in those rows are bounds against a complete Before, not exact savings. All other reported layers and all seven affected binaries are complete. Every difference is **After − Before**, in CPU s and instructions; a negative sign denotes a reduction. Test-count differences are default −25 selected / −26 passed, fuzzy −23 selected / −23 passed.

| Suite | Selected / passed tests | Suite wall s | Exit status | Instructions | temper CPU s, all layers | temper CPU s, replaying layers only | Difference from Before |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Focused (`default`) | 15607 / 15606 | 182.341691 | 100 | ≥4557298456335 | ≥634.795607 | 489.316339 | all ≥−16.536715 s; replaying −14.767592 s; instructions ≥+18254639942 |
| `fuzzy` | 120 / 120 | 61.756951 | 0 | 1286720502805 | 199.967302 | 143.397013 | all −49.490748 s; replaying −16.315643 s; instructions −321836248966 |

### Focused (`default`)

| Layer | CPU s | Ceiling s | Room s | Instructions | Difference from Before; relevant work |
| --- | ---: | ---: | ---: | ---: | --- |
| step | 157.136331 | 254.600000 | 97.463669 | 1479263353983 | −5.896295 s; −4347922935 instructions; two referee controls moved here, worker hosting repair |
| domain | 145.071410 | 142.800000 | -2.271410 | 762289383549 | −2.736585 s; +13055013995 instructions; engine-domain/forge holders added for cut or retired stories |
| system | 53.672602 | 111.000000 | 57.327398 | 360232993113 | −5.309735 s; +8668110308 instructions; composed chat/merge and story holders added |
| machine | 1.577677 | 4.000000 | 2.422323 | 13935271739 | −0.087520 s; −5626 instructions; no F7 move attributed |
| protocol | 9.555147 | 9.000000 | -0.555147 | 66764456703 | +0.036767 s; +38042 instructions; no F7 move attributed |
| io | no process | none | unmeasured | not counted: no process | no process on either side, no numeric difference; no F7 move attributed |
| simulated | 122.303172 | 470.000000 | 347.696828 | 866343485922 | −0.774224 s; −782199 instructions; two supplied-value referee controls moved out |
| real-loop | 85.427049 | 76.700000 | -8.727049 | 624722060366 | +0.610017 s; −4757784170 instructions; stories cut or retired; focused helper users retained |
| end-to-end (**partial**) | ≥60.052219 | 36.900000 | ≤−23.152219 | ≥383747450960 | ≥−2.379140 s; ≥+5637972527 instructions; no F7 move attributed |

### Fuzzy

| Layer | CPU s | Ceiling s | Room s | Instructions | Difference from Before; relevant work |
| --- | ---: | ---: | ---: | ---: | --- |
| step | 24.741794 | 20.600000 | -4.141794 | 254066171078 | +0.195097 s; +4373133 instructions; moved referee controls are focused, not fuzzy |
| domain | 83.351985 | 85.400000 | 2.048015 | 420958019110 | −3.724134 s; −12506766761 instructions; eleven duplicate removal/reset imports retired; lower holders added |
| system | 10.659799 | 13.000000 | 2.340201 | 73062406786 | −0.032277 s; +1610216 instructions; new focused holders replace two upper fuzzy wrappers |
| machine | 0.351104 | 2.000000 | 1.648896 | 2132190652 | −0.018499 s; +112 instructions; no F7 move attributed |
| protocol | 0.106902 | 2.000000 | 1.893098 | 560729557 | −0.033602 s; +188 instructions; no F7 move attributed |
| io | no process | none | unmeasured | not counted: no process | no process on either side, no numeric difference; no F7 move attributed |
| simulated | 24.185429 | 196.000000 | 171.814571 | 179027438272 | −12.702228 s; −94061631289 instructions; four fixed-cut upgrade and two chat/merge wrappers retired |
| real-loop | 56.570289 | 183.000000 | 126.429711 | 356913547350 | −33.175105 s (56.570289 − 89.745394); −215273834565 instructions; twelve survivors cut to their seams; other stories retired |
| end-to-end | no process | 1.000000 | unmeasured | not counted: no process | no process on either side, no numeric difference; zero selected, not a timed observation |

The non-gate summaries warn on default domain/protocol instruction ceilings and real-loop/end-to-end CPU ceilings, and fuzzy step/domain/system instruction ceilings. These observations give no permission to raise a budget. Positive CPU room does not imply instruction room. The largest absolute layer CPU differences are fuzzy real-loop **−33.175105 s**, fuzzy simulated **−12.702228 s**, and default step **−5.896295 s**. A same-machine run controls machine choice, not load; for example, default domain CPU fell while its instructions rose. Whole-process differences do not isolate the marginal cost of any individual assertion.

### Affected binaries and completeness

All seven affected binaries completed their selected tests successfully on both sides. **After default's `temper-box::suite` is partial**: `box_listeners::a_fresh_box_holds_serves_clients_port` failed in-suite (wall **0.289820 s**) and in its one automatic alone diagnostic (wall **0.234191 s**, exit 101). Both report the assertion at `temper/bin/temper-box/tests/suite/box_listeners.rs:109`: “the configured clients port stays reserved”, `Some(false)` instead of `Some(true)`. This is **not a flake**. The failure log is `target/failure-worlds-after-default-20261122T1850/command-output.txt` (in-suite line 299, alone line 1511); `results.json` retains the in-suite output and `alone = "failed"`. No further retry or code repair was made in this documents-only measurement. The requester authorized continuing the four-run pair and recording partial results without stopping between runs.

The partial box binary contributes **≥40.221114 CPU s**, **≥325059675152 instructions**, including the alone process (**0.131119 CPU s**, **1357571930 instructions**), to the end-to-end lower bound. Its 227 selected tests had 226 passes and one failure; the diagnostic is additional, not counted as another selected test. **Crashed, starved, cancelled and unobserved selected tests: none** in the pair (`unobserved = []`, `cancelled = null`, no signals or leaks). After fuzzy's 120 tests all passed. No other binary is partial. The load-sensitive `turn_landing::a_failing_member_turns_the_dispatched_top_red` from the superseded hel1-8 runs passed in both default runs here.

| Binary | Focused CPU s | Focused selected tests | Focused instructions | Fuzzy CPU s | Fuzzy selected tests | Fuzzy instructions | Difference from Before; relevant work |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `temper-engine-domain-world::suite` | 128.164929 | 1400 | 659422193572 | 35.351431 | 28 | 191382863425 | default −1.570145 s / +13059993430 instructions; fuzzy −2.642219 s / −12502260986 instructions; duplicate fuzzy removal/reset registrations retired, focused story holders added |
| `temper-engine-forge-world::suite` | 6.445421 | 359 | 37670106988 | 7.026264 | 10 | 41176381470 | default −0.654113 s / −7480975 instructions; fuzzy −0.035370 s / −11260090 instructions; focused story holders added |
| `temper-worker-domain-world::suite` | 0.230667 | 76 | 1391550146 | 0.344227 | 3 | 2623415683 | default −0.024237 s / +2439646 instructions; fuzzy −0.059962 s / +5667044 instructions; hosting repair is in the worker library; no retirement credited here |
| `temper-system-world::suite` | 12.667945 | 66 | 110045754996 | 1.436213 | 1 | 7388354232 | default +0.667012 s / +8669176416 instructions; fuzzy +0.045706 s / +1464676 instructions; focused chat/merge and story holders added |
| `temper-one-process-world::suite` | 77.758274 | 62 | 532510047156 | 24.076599 | 3 | 178072667508 | default −1.094885 s / +2285904 instructions; fuzzy −12.698721 s / −94061631404 instructions; two focused controls moved out, four upgrade and two chat/merge fuzzy wrappers retired |
| `temper-one-process-world` (library tests) | 0.677823 | 26 | 5742705467 | no process | 0 | not counted | default +0.037013 s / +127003 instructions; fuzzy no process on either side, no numeric difference; two focused referee controls moved in |
| `temper-real-loop-world::suite` | 45.213694 | 31 | 282955828854 | 56.570289 | 12 | 356913547350 | default +1.389683 s / −4350235046 instructions; fuzzy −33.175105 s / −215273834565 instructions; five requested retirements plus 6.7 retired, nine scripted survivors cut; backend and twins retained |

Across all binaries, the largest absolute CPU differences are fuzzy `temper-real-loop-world::suite` **−33.175105 s**, fuzzy `temper-one-process-world::suite` **−12.698721 s**, and default `temper-client-world::suite` **−5.959518 s** (instructions **−1069130**, no F7 move attributed). Within the seven affected binaries, third is fuzzy engine-domain **−2.642219 s** instead.

The After real-loop library has twelve fuzzy stories: nine scripted (4.1/4.3 combined, 4.6, 5.1, 5.2, 5.4, 6.4, 6.6, 6.8, kept-delegate landing), the program backend and two recorded twins. These are their individual **wall times**, not apportioned CPU; all passed.

| Fuzzy test in `temper-real-loop-world::suite` | Wall s |
| --- | ---: |
| `fuzzy::case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | 5.268239 |
| `fuzzy::case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | 2.987101 |
| `fuzzy::case_5_1::released_runs_read_the_pushed_guide_without_the_dropped_rule` | 2.607451 |
| `fuzzy::case_5_2::a_stopped_owners_reviewed_change_lands_and_its_news_arrives_after_release` | 4.797161 |
| `fuzzy::case_5_4::a_pushed_change_is_cancelled_under_its_closed_pull_and_an_overlapping_landing_wakes_b_after_release` | 7.578931 |
| `fuzzy::case_6_4::a_red_tip_is_repaired_before_a_waiting_change_updates_and_lands` | 5.773821 |
| `fuzzy::case_6_6::kit_merges_before_apps_first_push_and_both_land_green_with_the_pin` | 4.473719 |
| `fuzzy::case_6_8::a_testers_test_lands_only_through_its_owners_change_without_tester_forge_writes` | 4.430263 |
| `fuzzy::green_light::a_green_light_in_the_inbox_lets_the_change_land` | 9.779328 |
| `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | 3.273574 |
| `fuzzy::program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | 2.962510 |
| `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | 7.789804 |

## The moves for the user

Use `failure-worlds-budget.md`, *The moves proposed*: for baseline M, retirement credit D, replacement L, checks H, sweeps S and net transfer T, the balance is **ceiling + T − (M − D + L + H + S)**. The complete layer measurements supply M = Before and the combined expression M − D + L + H + S = After, not separately measured D/L/H/S. Thus the measured post-transfer balance is **ceiling + T − After**. For partial end-to-end this is only an upper bound. Historical candidate ceilings in the budget note are not today's ceilings: this arithmetic starts from the current summaries, with existing moves applied once, then applies only the proposals below.

A candidate is supported here only when its donor and receiver are complete on both sides and its `cpu_s` is no greater than the donor's measured freed CPU, **Before − After**. A partial donor or receiver would be **not certified: partial**; none of these four candidate layers is partial in this pair. Receiver growth is the signed **After − Before**, not an apportioned marginal replacement cost. Positive After room alone is not funding. These are measured-funding proposals, not a guarantee about main's unknown held budgets or a causal credit for all CPU fluctuations.

| Suite | from → to | Candidate cpu_s | Donor freed CPU s (Before − After) | Receiver growth CPU s (After − Before) | Decision |
| --- | --- | ---: | ---: | ---: | --- |
| default | `temper.real-loop` → `temper.domain` | 12 | −0.610017 | −2.736585 | **Not supported**, no move proposed: donor grew, so no positive reduced figure is supported; cut stories have lower holders but yielded no measured CPU saving in this suite. |
| default | `temper.simulated` → `temper.step` | 0.1 | +0.774224 | −5.896295 | **Supported at 0.1**, not reduced: the two supplied-value referee controls moved, and 0.1 ≤ 0.774224. |
| fuzzy | `temper.real-loop` → `temper.domain` | 20 | +33.175105 | −3.724134 | **Supported at 20**, not reduced: survivors were cut and stories retired; 20 ≤ 33.175105. |
| fuzzy | `temper.simulated` → `temper.system` | 2 | +12.702228 | −0.032277 | **Supported at 2**, not reduced: chat/merge holders replaced wrappers; 2 ≤ 12.702228. |

Every proposed direction is strictly downward in `tools/tiers/layers.toml`: step, domain and system precede simulated and real-loop. The proposals preserve each suite's sum of ceilings (`tools/tiers/budgets.md`, 10). No instruction transfer is proposed. Duplicate retirement within one layer frees that layer, not a separate inter-layer allowance. Receiver growth includes all work in the layer, not just the lower holders, and does not claim that any one retired test paid for any one added assertion.

| Suite / candidate layer | Current ceiling s | Before room s | After room s, no new move | Net proposed T s | Proposed ceiling s | After room s, with proposals |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| default / real-loop | 76.700000 | -8.117032 | -8.727049 | 0 | 76.700000 | -8.727049 |
| default / domain | 142.800000 | -5.007995 | -2.271410 | 0 | 142.800000 | -2.271410 |
| default / simulated | 470.000000 | 346.922604 | 347.696828 | -0.1 | 469.900000 | 347.596828 |
| default / step | 254.600000 | 91.567374 | 97.463669 | +0.1 | 254.700000 | 97.563669 |
| fuzzy / real-loop | 183.000000 | 93.254606 | 126.429711 | -20 | 163.000000 | 106.429711 |
| fuzzy / domain | 85.400000 | -1.676119 | 2.048015 | +20 | 105.400000 | 22.048015 |
| fuzzy / simulated | 196.000000 | 159.112343 | 171.814571 | -2 | 194.000000 | 169.814571 |
| fuzzy / system | 13.000000 | 2.307924 | 2.340201 | +2 | 15.000000 | 4.340201 |

**For the user — proposed moves (`cpu_s`):**

- `default temper.simulated → temper.step 0.1`
- `fuzzy temper.real-loop → temper.domain 20`
- `fuzzy temper.simulated → temper.system 2`
- No default real-loop → domain move is supported by this pair.

**Still over the CPU ceilings after those proposals:** default domain **2.271410 s**, protocol **0.555147 s**, real-loop **8.727049 s**, end-to-end **at least 23.152219 s** (partial); fuzzy step **4.141794 s**. These are also the CPU overruns before the proposals, since none of the three supported transfers changes an over-ceiling layer. Instruction ceilings remain exceeded by default domain **120889383549**, default protocol **16364456703**, fuzzy step **78366171078**, fuzzy domain **54358019110**, and fuzzy system **1362406786** instructions. No budget, held-budget, reference, ledger, code, test or configuration file was changed.
