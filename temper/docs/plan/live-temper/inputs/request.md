# live-temper: two shelf-login live stories after every landing (the user's decision, 11-02)

The user decided (11-02):

1. Run the two live stories that need only shelf logins after each landing, under fab's `--live`
   with `--run-ignored` for just those two:
   - `live::a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` (temper/crates/temper/tests/live.rs ~39-45)
   - `live::a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` (live.rs ~54-58)
2. Delete harness step tests that duplicate focused ones (the user counted 42).
3. Keep every other live test that needs a person (a sitting, github.com), ignored.

## The temper change (this plan)

- **Select just the two.** The `live-temper` profile (`.config/nextest.toml` ~159) selects
  `binary(=live) | binary(=live_github)`; under `--run-ignored only` that would also run the three
  person-only stories (`a_real_model_answers_a_chat_and_pushes_a_small_change`,
  `rung_b0_a_person_agrees_a_plan_before_the_change_lands`, `live_github::a_chat_lands_a_change_on_github_com_through_a_restart`).
  Narrow the profile's default-filter to the two stories (or a new profile fab runs), and give
  `tools/tiers/packages.toml` ~46 its binary (`live`, today `[]`).
- **One fresh store per story.** Both read `TEMPER_LIVE_STORE` and a store that isn't empty counts as
  kept (support/forge_land.rs ~769-773): run in one invocation, the second story reuses the first's
  deployment. Each story takes its own fresh store under the one fab sets (e.g. a per-story
  subdirectory), or the profile runs them apart; say which in testing.md.
- **The cheap model for Claude.** The Claude story hard-codes `"sonnet"` (forge_land.rs ~1134); the
  user's rule for automatic live runs is the cheapest models (11-01: luna, claude-haiku-5-5). Use
  `claude-haiku-5-5` there (ChatGPT's is gpt-6-luna already, ~522, ~1133).
- **Without a person:** with `TEMPER_LIVE_ACCOUNT` set the ChatGPT story needs no sitting
  (forge_land.rs ~784, ~1064-1072); the Claude one needs `TEMPER_LIVE_CLAUDE` and `claude` on PATH
  (live.rs ~57, forge_land.rs ~741-750). Keep both runnable by the owner as today.
- **The 42 duplicates: none remain.** The count came from the after-landing tiers
  audit at de07a0b2c; 1425451fb7 (in eb6c40aaf, 11-01) since moved those harness tests into
  `suite/support_tests.rs`. Nothing to delete; the design text says so.
- **Kept, ignored:** the three person-only stories above, `live_github_managed`'s two ignored ones and
  `proof_live`'s three (offline driver proofs): unchanged.
- Tests: the profile selects exactly the two (a nextest `--list` count in the report); design text
  (temper/docs/design/domain/testing.md's live tier section, its status file) says what runs after
  landing and what stays the owner's.

## Out of scope

fab's side (its LIVE table, a fresh `TEMPER_LIVE_STORE` per run, `--run-ignored only` for the pair)
is fab's own work in the agents repository, after this plan lands: not this plan's.
