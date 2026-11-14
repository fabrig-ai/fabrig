# Repair after landing's non-live reds of 11-14

Draft, 2026-11-14. A manual non-live after-landing run at main 6bd49188cb (11-14 10:41Z, on w2) found 108 reds in four pairs: browser-jig 7 of 7, browser-skein 73 of 100, browser-temper 12 of 12 and process-temper 16 of 53. conformance-hearth was green. Each process red was repeated alone, by name. The run's logs are on w2, under `temper-worker/ci/after/runs/20261114T104149Z/`, and the by-name reruns are under its `rerun/`.

## 1. Browser worlds can't find their servers (92 reds)

- `jig/tests/client-browser/src/lib.rs` (`jig_test`), `temper/tests/client-browser/src/lib.rs` (`binary`), `skein/tests/browser/tests/suite/folds_away.rs` (`setup`) and skein-ui-web-tests' like it take `current_exe().parent().parent()` to be `target/debug`.
- Under the pinned nightly's cargo (1.101.0-nightly 2026-10-12, pinned by goal 1914), a test executable can sit deeper, under `target/debug/build/<pkg>/<hash>/out/`. The lookup then misses `target/debug/jig-test`, `skein-ui-example` and `temper-box`, which are built. Each test fails in about 10 ms.
- **Wanted:** one shared way for a world to find a sibling binary that doesn't depend on how deep the test executable sits. For example, cargo's `CARGO_BIN_EXE_<name>` where the binary is in the same package; otherwise, walk up to the target profile directory, or use `CARGO_TARGET_DIR` and the profile. Every world above uses it.

## 2. Process stories refused at serve (3 reds)

- `a_batch_of_three_lands_by_one_fast_forward` and `a_red_batch_is_narrowed_before_the_remaining_change_lands` are refused at serve with "application.landing.projects[0].batch: batch passes forge.landing_members". That check came from goal 2175, and these stories had never run before. Their configuration must agree with the check.
- `a_turn_landing_is_found_after_a_runner_cancellation_and_restarts` is refused with "application.limits.overrides.forge.resources_per_task: must agree with forge.landing_members + 1". The check is from goal 1787. Its configuration must agree too.
- Change a story's configuration, not the check, unless the check is wrong. In that case, say why.

## 3. "one real loop per thread" (6 reds)

- At `skein/testing/skein-world/src/end_to_end.rs:61` (`FindingScope::enter`), which came from Panics P3 (goal 2699, 8a6c8287db), these 6 fail:
  - `a_csv_change_passes_ci_and_…`;
  - `a_persons_plan_change_lands_…`;
  - `a_restart_with_another_box_sharing_its_cache_answers_ctl`;
  - `fake_backends::newer_build::a_stopped_fake_box_starts_with_new_files_at_the_same_paths`;
  - `fake_backends::walk::here_key::forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box`;
  - `fake_backends::walk::sign_in::a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create`.
- **Wanted:** these stories keep P3's finding scope without nesting a second real loop on one thread, or the scope allows their shape. Whichever fix is chosen keeps P3's intent.

## 4. Unattributed process reds (7)

Each failed alone, by name, with the same message:

- `an_unexpected_member_exit_is_reported_after_its_client_settles`: cluster.rs:977, "w1 exited unexpectedly: Some(Signal(9))". No OOM was seen.
- `a_box_signs_its_person_in_to_its_owned_forgejo_with_one_click`: forge_sign_in.rs:41, "DIRECTORY is signed in as party 1; this link is not spent".
- `startover::owned_setup_cuts_start_over_and_a_removal_cut_finishes_on_restart`: startover.rs:155, "cut at Prepare: Open this link in a browser on this machine."
- `a_forgejo_elsewhere_connected_as_its_admin_…`: forgejo_elsewhere.rs:522, "elsewhere prerequisite call 0 failed" (Code(1)).
- `tui_box::lending::temper_box_lends_its_pty_…`: lending.rs:332. The terminal loan doesn't settle in 60 s, with runnable waits in µs, so it's a hang, not load.
- `tui_box::program_login_takes_its_background_groups_terminal_before_claude_reads`: the real loop doesn't settle in 60 s. Same as the one above.
- `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge`: boxed.rs:651. The box ctl story runs past 230 s, with 42 s of CPU.

**Wanted:** for each one, the cause and a repair. If the cause lies outside the test, for example a product bug, fix the product. A red whose cause can't be found within reason is reported, not hidden: its evidence goes in the goal's result, and the test stays as it is.

## 5. Out

- Live stories, budgets and tier wiring.

## 6. Tests

Cheap: focused and fuzzy suites only; walk = false. As a red's repair, each test named above may run alone, by name, to see it green. Run no tier and no pair as a whole.
