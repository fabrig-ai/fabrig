+++
id = "reference-tiers"
part = "goal-tier"
kind = "docs"
needs = ["goal-tier-config"]
files = ["temper/docs/examples/reference/landing.json", "temper/crates/temper-service/src/tests/reference_landing.rs"]
effort = "medium"
minutes = 40
+++

# The reference's landing member names main's tier and g's tier

## Goal

`temper/docs/examples/reference/landing.json` gains project 1's repository entry with two
tiers, main's `slow` and g's `goal`, as `temper/docs/design/protocol/configuration.md`, 13.9
(the tiers) has them, and its test checks they parse.

Acceptance:

- Project 1's object in `landing.json` gains, after its `rules`, this member exactly (the
  walk fills `forge` and `repository` as it fills the rules' host and name):

  ```json
  "gates": [
    {
      "forge": 1,
      "repository": 1,
      "gates": [],
      "tiers": [
        {"name": "slow", "workflow": "tier-slow.yml", "ignored": [], "artifact": "failing", "format": "lines"},
        {"name": "goal", "workflow": "tier-goal.yml", "ignored": [], "artifact": "failing", "format": "lines", "branch": "goal", "span_ms": 1200000}
      ]
    }
  ]
  ```

  indented as the file is; nothing else in the file changes.
- `temper/crates/temper-service/src/tests/reference_landing.rs`'s
  `the_reference_landing_member_parses` asserts, beside what it checks, that project 1 has one
  repository entry with no gate and the two tiers: `slow` following its landing branch
  (`Follows::Landing`) with no span, and `goal` following `Follows::Goals` with a span of
  1,200,000 ms, both `lines`, artifact `failing`.
- `heavy cargo nextest run -p temper-service --lib -E 'test(/reference_landing/)'` passes.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`; `notes/contracts.md`, section 4.
- `temper/docs/design/protocol/configuration.md`, section 3 (the `gates` entry, `branch`,
  `span_ms`) and 13.9 (the tiers).

## Owns and may change

- Owns the two files of this spec's header for what acceptance names.
- If the parse refuses the entry because repository 1 of forge 1 is not adopted in the test's
  document, give the test's document what the rules' check gives it (its fixtures), not the
  JSON; if that cannot be done in the test, stop and ask.

## Steps

1. The JSON; check it with `python3 -I -c 'import json,sys; json.load(open(sys.argv[1]))' temper/docs/examples/reference/landing.json`.
2. The test's assertions; run it as acceptance says.
3. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; commit and
   push.

## Report

As the implementer's text asks.
