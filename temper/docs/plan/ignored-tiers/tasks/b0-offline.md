+++
id = "b0-offline"
part = "b0-offline"
kind = "feature"
needs = []
files = [".config/nextest.toml", "temper/crates/temper/tests/proof_live.rs", "temper/scripts/proof-live.sh", "temper/crates/temper/tests/live.rs", "temper/crates/temper/tests/live_github.rs", "temper/crates/temper/tests/suite/forge_land.rs", "temper/docs/design/domain/testing.status.md"]
effort = "medium"
minutes = 50
+++

## Goal

B0's two offline driver proofs leave `#[ignore]` and join the process
tier. `scripts/proof-live.sh rung-b0-offline` keeps working by running
them there. The live tests that still need a person say in their
`#[ignore]` reason what only a person gives. You also write the
selection contract for both of this plan's process-tier members.

Acceptance:

- `temper/crates/temper/tests/proof_live.rs`:
  `rung_b0s_setup_reaches_account_import_with_the_fake_model_and_no_account`
  and `rung_b0s_walk_passes_offline_on_its_twin` have no `#[ignore]`.
  Their bodies are unchanged.
- `.config/nextest.toml` holds the selection contract of
  `notes/common.md`, word for word, with each changed filter's comment
  saying what it selects and citing
  `temper/docs/design/domain/testing.md`, 2.6.
- `heavy cargo nextest list -p temper --profile process-temper` lists the
  two `rung_b0s_` tests. `heavy cargo nextest list -p temper` (focused)
  lists neither, and still lists `proof_live`'s
  `support::forge_land::tests::` tests and `live_github_managed`'s
  `relay_managed::` tests as before. `heavy cargo nextest list -p temper
  --profile process-temper --run-ignored all` lists `managed_ready_client`
  (still ignored until `managed-sitting`) and no other
  `live_github_managed` test.
- `temper/scripts/proof-live.sh rung-b0-offline` runs
  `cargo nextest run -p temper --test proof_live --profile process-temper --no-fail-fast`
  behind `TEMPER_WRAPPER` as now, with `NEXTEST_DOUBLE_SPAWN=0`. It drops
  `--ignore-default-filter` and `--run-ignored only`, since the profile's
  default filter selects the two. Its comment says no live account is
  admitted and cites `domain/testing.md`, 2.6.
- A new focused test in `temper/crates/temper/tests/suite/forge_land.rs`,
  `the_offline_rung_runs_its_proofs_in_the_process_profile`, holds that
  command. It works like `the_launcher_runs_its_builds_through_the_wrapper_it_is_given`:
  `launcher(root, "rung-b0-offline", None)`, a `wrap` stub on `PATH`
  recording its arguments, `TEMPER_WRAPPER=wrap`, exit 0. It asserts the
  exact argument lines `cargo`, `nextest`, `run`, `-p`, `temper`, `--test`,
  `proof_live`, `--profile`, `process-temper`, `--no-fail-fast`. It must
  bite: put `live-temper` back in the script once, see it fail, restore.
- The `#[ignore]` reasons name what only a person gives (design
  `temper/docs/design/domain/testing.md`, 2.7, *What needs a person*):
  - `live.rs`, `a_real_model_answers_a_chat_and_pushes_a_small_change`:
    `"live: a person's fresh ChatGPT sign-in, through scripts/proof-live.sh"`;
  - `live.rs`, `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge`:
    `"live: a held ChatGPT account, TEMPER_LIVE_ACCOUNT; the after-landing tier, or scripts/proof-live.sh forge-land"`;
  - `live.rs`, `rung_b0_a_person_agrees_a_plan_before_the_change_lands`:
    `"live: a held ChatGPT account, on the rung's sol; scripts/proof-live.sh rung-b0"`;
  - `live.rs`, `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`:
    `"live: a held Claude Code login, TEMPER_LIVE_CLAUDE; the after-landing tier, or scripts/proof-live.sh forge-land-claude"`;
  - `live_github.rs`, `a_chat_lands_a_change_on_github_com_through_a_restart`:
    `"live: github.com, on the probe App's fixtures the owner holds; scripts/proof-live-github.sh"`.
- `temper/docs/design/domain/testing.status.md`: the bullet that begins
  `**Not yet:** 2.6: B0's two offline proofs are ignored` is replaced
  whole by:

  ```
  - **Not yet:** 2.6: B0's two offline proofs have no runtime result in
    profile `process-temper`, and no CPU figure against the tier's budgets
    (`tools/tiers/budgets.toml`): the plan `ignored-tiers` compiled and
    listed them, and the next after-landing run proves them (11).
  ```

  Keep the second new bullet (`managed_ready_client`), which is
  `managed-sitting`'s.
- `heavy cargo check -p temper --tests` passes, and
  `heavy cargo nextest run -p temper` (temper's focused tests, the
  profile's default filter) passes.

## Inputs

- `temper/docs/plan/ignored-tiers/notes/common.md`: every rule there
  holds, the selection contract above all.
- `temper/docs/design/domain/testing.md`, 2.6 (*Two drivers' offline
  proofs*), 2.7 (the ladder's B0 paragraph, and *What needs a
  person*), and 7 (the `proof_live.rs` line).
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.
- `tools/tiers/design.md`, 1 and 6 (a profile's package set is
  generated from its listing; `process-temper`'s stays `temper`, so
  `tools/tiers/packages.toml` doesn't change).

## Owns and may change

- Owns: the files in the header. In `live.rs` and `live_github.rs`, the
  `#[ignore = "..."]` strings alone. In `suite/forge_land.rs`, the new
  test alone, beside the launcher tests, reusing `launcher`.
- Not yours: `temper/crates/temper/tests/live_github_managed.rs` and
  `support/managed_client.rs` (`managed-sitting` owns them), and the
  bodies of `support/forge_land/rung.rs`.
- Options, each pre-approved:
  - If clippy or the compiler asks for a change in `proof_live.rs`
    beyond the two lines (an unused import, say), make it and name it.
  - If `cargo nextest list` shows a `rung_b0s_` test selected by another
    profile too (`fuzzy`, `live-temper`), the contract is wrong for it.
    Narrow that profile by the same `not (...)` clause and say so in the
    report. Don't widen any other filter.
  - If `tools/tiers/packages.toml` would change by its generator
    (`tools/tiers/design.md`, 6), regenerate it with that tool and say
    so. Never edit it by hand.

## Steps

1. Remove the two `#[ignore]` lines in `proof_live.rs`. Edit
   `.config/nextest.toml` to the contract, and edit `proof-live.sh`'s
   `rung-b0-offline` branch. Commit.
2. Write `the_offline_rung_runs_its_proofs_in_the_process_profile`. Run
   it alone (`heavy cargo nextest run -p temper --test suite -E
   'test(=the_offline_rung_runs_its_proofs_in_the_process_profile)'`,
   count 1), break it once as above, restore.
3. Change the five `#[ignore]` reasons and the status bullet.
4. `heavy cargo check -p temper --tests`; the three lists of the
   acceptance. Never run `proof_live`'s tests, the `process` binary or a
   live test.
5. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper`.
6. Commit, run the citations check (`notes/common.md`), push.

## Report

The implementer's report, with: the three lists' lines for the
`rung_b0s_` tests and `managed_ready_client`; the new test's time and
what breaking it showed; the reasons as written; any option taken.
