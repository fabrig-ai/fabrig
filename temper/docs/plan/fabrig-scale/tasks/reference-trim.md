+++
id = "reference-trim"
part = "reference"
kind = "feature"
needs = ["stock-scale"]
files = ["temper/docs/examples/reference/limits.json", "temper/docs/examples/reference/agent.json", "temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper/tests/support/owned.rs", "temper/crates/temper/tests/suite/startup.rs", "temper/docs/design/protocol/configuration.status.md", "temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs"]
effort = "high"
minutes = 120
+++

# The reference sets only what the box does not hold

## Goal

With init's box at fabrig's scale, the reference's files set what still differs from it, and
nothing that would lower it (`temper/docs/design/protocol/configuration.md`, 13.9, "The profile,
whole"): `limits.json` holds `tasks.tree_tasks` 1,024 alone; the reference sets nothing of the
agent, so `agent.json` goes; the settings file's charters take the stock's run and its template
the box's result bound; charter 5 makes a family's changes within one call's payload. The
draft's "Done when" asks the reference's steps with no change beyond its charters and template.

Acceptance:

- `temper/docs/examples/reference/limits.json` is exactly
  `{"engine": {"limits": {"overrides": {"tasks.tree_tasks": 1024}}}}` (formatted as the file is,
  two-space JSON, ending with a line end); `temper/docs/examples/reference/agent.json` is
  deleted.
- `settings.json` (check it with `python3 -I -c 'import json,sys; json.load(open(sys.argv[1]))' <file>`
  after each edit):
  - charters 2, 5, 6, 7 and 8: `"turns": 800` and `"time": 28800000000000` (8 hours, in the
    file's unit, as its 1,800 s are `1800000000000`), every other member as it is;
  - template 4's report `words`: init's box's `tasks.result_bytes` (read it from
    `temper/crates/temper-init/template/temper.json`'s engine overrides, where `init-standard`
    set it to the agent's message);
  - charter 5's instructions: the paragraph that is exactly
    `A \`change\` call makes at most one changes. Make a larger batch in several calls, a later call naming an earlier one's changes in "after" by the task numbers its answer gave. Where a call is refused for its size, make one change to a call.`
    becomes exactly
    `A \`change\` call carries what one call's payload holds, about ten kilobytes of words. Make a larger batch in several calls, a later call naming an earlier one's changes in "after" by the task numbers its answer gave. Where a call is refused for its size, make fewer changes to a call.`
    Nothing else of the file.
- `temper/crates/temper/tests/support/owned.rs`: `bounded_reference` merges `limits.json` alone
  over init's `temper.json` and takes init's `agent.json` as it is; its callers in
  `tests/suite/startup.rs` (`the_reference_profile_passes_startup_admission`) pass, the
  admission `Ok`.
- `temper/crates/temper/src/tests/reference_settings.rs`: `TURNS` (64) and `INIT_TURNS` (16)
  give way to init's agent's `max_turns`, read from `temper-init`'s `template/agent.json`
  (`include_bytes!` and `serde_json`, as the file reads the others); charters 2, 5, 6, 7 and 8
  assert 800 turns and 28,800 s; the fit test asserts the charters within `tasks.charters` and
  the stock's, their turns within init's `max_turns`, template 4's report within init's
  `tasks.result_bytes`, and charter 5's instructions contain `A \`change\` call carries what one
  call's payload holds`. The assertion that the reference's `max_turns` is 64 and init's 16
  (near 220-221) goes with `agent.json`.
- `configuration.status.md`: from its first **Not yet** entry, the clause "13.9: the reference's
  files set what the box's profiles now hold (...)." removed.
- `heavy cargo nextest run -p temper --lib -E 'test(/tests::reference_settings::/)'` and
  `heavy cargo nextest run -p temper --test suite -E 'test(/the_reference_profile_passes_startup_admission|checked_in_deployment_profiles_pass_startup_admission/)'`
  pass, each selecting its tests; then the other readers of the files:
  `-p temper-service --lib -E 'test(/reference_landing/)'`, `-p temper-client-tools --lib
  -E 'test(/settings/)'`, `-p temper-init`, and the engine domain world's
  `forge_routes::goal_branch` (`-p temper-engine-domain-world`).

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K5 and K6.
- `temper/docs/design/protocol/configuration.md`, 13.9 ("The profile, whole", its table and the
  paragraph after it; "Checked as it is read") and 13.4 (the table of the box's limits).

## Owns and may change

- The files of the header, for what acceptance names.
- Readers that name `agent.json` of the reference (`rg -n 'reference/agent.json'`): a test that
  reads it to merge it stops doing so, its assertion on what it set removed; each listed.
- `temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs`: the worker hello's
  offered bounds only (see Answers).
- `docs/examples/reference/landing.json` and the repository's `.temper/` are not yours.
- If init's admission refuses the merged document (a limit `tasks.tree_tasks` 1,024 ties to,
  the least that passes), add that limit to `limits.json` at the least value that passes and
  say so; never lower a figure of init's.

## Steps

1. The two JSON files; the owned support and the startup test; run them.
2. The settings file; the settings test; run it.
3. The other readers, one package at a time.
4. The status line; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper --all-targets -- -D warnings`; the citations check; commit and push.

## Report

As the implementer's text asks.

## Answers

### 2026-11-04: may this task repair the goal-branch fixture, or does a separate task?

This task repairs it: acceptance names the test, and a separate task would only park this one.
The header now holds `goal_branch.rs`, for one line.

The likely cause is `stock-scale`'s, not the settings you changed; read from the code, not yet
run. `reference_template_four_starts_on_the_configured_forge` gives its
worker `jig_test_profile::fixtures::bounds()`, whose `time` is 3,600 s. The hello is checked
against every configured charter (`jig_domain_fleet::needs::shortfall`, from `channel.rs`), and
the stock's charters 2, 4 and 6 now run 28,800 s, so the hello gets `AgentShortfall::Time`, no
worker joins and no run is assigned: the `answers=[], assignments=[]` of your log. The test's
charters come from the stock, not from the reference's settings.

The repair, as `stock-scale`'s answer treats a fixture (what an agent offers, not a stock
figure), after `hello.bounds.models = ...` in that test:

```rust
hello.bounds.time = Duration::from_secs(28_800); // The stock's longest charter.
```

- First merge `origin/fabrig-scale/stock-scale`: if the test already passes, change nothing here.
- `jig_test_profile`'s `bounds()` stays; jig names no stock.
- If it still fails, read the hello's refusal: another offered bound may rise the same way, the
  figure the refusal names, listed in the report. A failure no offered bound repairs is a
  question; no stock, budget or timeout moves.
- The `problems.rs` lint is `stock-scale`'s: leave it under `## Left` unless the merge mended it,
  and finish `heavy cargo clippy -p temper --all-targets` as far as it goes.
