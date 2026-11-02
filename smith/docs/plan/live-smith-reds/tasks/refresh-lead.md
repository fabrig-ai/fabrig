+++
id = "refresh-lead"
part = "refresh-lead"
kind = "feature"
needs = ["live-report"]
files = ["smith/tests/end-to-end/support/live.rs", "smith/tests/end-to-end/live.rs", "smith/tests/end-to-end/live_configuration.rs"]
effort = "medium"
minutes = 40
+++
# The refresh story raises its lead in milliseconds

## Goal

`smith::live a_run_refreshes_its_grant_at_the_issuer` is red after landing: its run completes but
the ChatGPT record's generation does not rise (`smith/tests/end-to-end/live.rs:34`). The story
means to raise `deployment.refresh_lead` past any access token's life ("400 days": 34_560_000 is
400 x 86_400) so its run refreshes at once (smith/docs/design/testing.md, 2.3). But smith's
deployment durations are whole milliseconds (smith/docs/design/protocol/limits.md, the paragraph
after the limits tables: "durations in whole milliseconds"; `smith/crates/smith-agent-shell/src/config.rs`
`duration(millis, ..)`), so the story raises the lead to 9.6 hours, and a lent record whose access
token has longer to live is not refreshed. Make the story's lead 400 days in milliseconds, through
one named constant that an offline test resolves the way the product does.

Acceptance:

- `smith/tests/end-to-end/support/live.rs` holds the lead and the argument splice:
  ```rust
  /// The refresh story's lead, in the settings' milliseconds: 400 days, past any access token's life.
  pub const REFRESH_LEAD_MS: u64 = 400 * 24 * 60 * 60 * 1000;

  /// Insert `-c deployment.refresh_lead=<lead_ms>` before the trailing prompt argument `-`.
  pub fn with_refresh_lead(arguments: &mut Vec<std::ffi::OsString>, lead_ms: u64);
  ```
  and `Environment::refresh_lead` is documented as milliseconds.
- `live.rs`: the refresh story sets `Some(live::REFRESH_LEAD_MS)`; `run` calls
  `live::with_refresh_lead` instead of its inline splice. Nothing else in `live.rs` changes.
- A new offline test in `smith/tests/end-to-end/live_configuration.rs`,
  `the_refresh_story_lead_outlives_an_access_token`: it parses
  `live::arguments("chatgpt", directory, true, false)` with `with_refresh_lead(.., REFRESH_LEAD_MS)`
  and resolves them as `assert_account` does, and asserts the resolved policy's `refresh_lead`
  is `Duration::from_secs(400 * 86_400)`. Run once with the constant at the story's old
  `34_560_000` first: red (9.6 hours); then green with `REFRESH_LEAD_MS`.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds/notes/common.md`.
- `smith/docs/design/testing.md`, 2.3 (the refresh story); `smith/docs/design/protocol/limits.md`,
  the `refresh_lead` row and the units paragraph after the tables.
- `smith/tests/end-to-end/live_configuration.rs` `assert_account` (the resolution to model on);
  `smith/crates/smith-local-shell/src/credentials/product.rs` (the product reads
  `resolved.agent.profile.policy.refresh_lead`).

## Owns and may change

- `smith/tests/end-to-end/support/live.rs`: the constant, the function, the field's doc.
- `smith/tests/end-to-end/live.rs`: the two lines named above only (`live-report` owns the rest
  of its shape and has landed before you start).
- `smith/tests/end-to-end/live_configuration.rs`: the new test, and a shared helper factored from
  `assert_account` if the new test needs the same resolution.
- Pre-approved:
  - If `settings::resolve::select` does not apply the `deployment.*` overrides to the agent
    profile, resolve the way the product does before it builds credential limits (follow
    `product.rs` back to where `resolved` is made) and say which call you used.
  - If a validation the product runs after resolution would refuse a 400-day lead, stop and ask:
    the lead's value is then a question, not yours to lower.
  - If 400 days in milliseconds overflows any conversion on the way (it should not: it is
    3.5e16 ns), stop and ask.

## Steps

1. Write the offline test with the constant at `34_560_000`; run
   `heavy cargo nextest run -p smith --test live_configuration -E 'test(=the_refresh_story_lead_outlives_an_access_token)'`:
   red; note what it saw.
2. Set `REFRESH_LEAD_MS`, add `with_refresh_lead`, wire `live.rs`; rerun: green.
3. `heavy cargo check -p smith --tests` (the `live` binary compiles; never run it).
4. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings`;
   `heavy cargo nextest run -p smith --test live_configuration`.
5. Commit, push.

## Report

The red of step 1 (the resolved lead it saw), the files changed, the tests run (selected,
passed), and implementation and check minutes apart.
