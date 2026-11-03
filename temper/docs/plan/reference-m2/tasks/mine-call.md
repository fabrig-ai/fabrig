+++
id = "mine-call"
part = "mine"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs"]
effort = "medium"
minutes = 45
+++

# mine-call: `change` takes `"branch": "mine"`, decoded and counted as a plan's member

## Goal

`change`'s call carries the `branch` member of `temper/docs/design/domain/agent.md`, 4.3.1:
`"mine"` decodes to `ChangeCall.branch: Some(ChangeBranch::Mine)`, the plan's declaration
offers it, and the root counts it as a plan's member (refused without the control family).
What `mine` makes (the change on its requester's held branch) is a later task's: this one
carries the shape only, so the root does nothing more with it yet.

Acceptance:
- `{"words":"a","branch":"mine"}` decodes to one `ChangeCall` with `branch:
  Some(engine::ChangeBranch::Mine)`; so does each object of `changes` naming it;
  `"branch":"theirs"` and `"branch":""` are `Problem::Range(b"branch")`; `"branch":1` is
  the type problem `text` gives.
- `PLAN_CHANGE`'s schema names `branch` exactly as contract C1 has it; the stock
  declaration does not.
- A run without the control family calling `change` with `"branch":"mine"` is refused with
  C1's words; the existing story asserting the old words asserts the new ones.
- Every test of `temper-engine-smith` and the `forge_routes` stories pass.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1: the `branch` bullet and "Who may set them".
- `temper/docs/plan/reference-m2/notes/contracts.md`, C1 (the names, the schema text, the
  refusal's words).
- `temper/docs/plan/reference-m2/notes/common.md`.
- `temper/docs/design/domain/agent.status.md`: its **Not yet** for 4.3.1's `branch` stays
  (the change it makes is a later task's); leave it.

## Owns and may change

Owns the files in the header.

- `boundary.rs`: `ChangeBranch` and `ChangeCall.branch` as C1 has them; `ChangeCall::words`
  sets `branch: None`.
- `calls.rs`: `fn change` decodes `branch` after `landing`, the same way `landing` is
  decoded (`value.field(b"branch")?`, then `text(b"branch")?`); `PLAN_CHANGE`'s schema gains
  C1's `branch` property after `landing`, nothing else of it changes.
- **Reader files, struct literals only** (the field the compiler asks for, `branch: None`,
  or `..ChangeCall::words(...)` where the literal already uses it):
  `temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs` and any other
  literal `heavy cargo check -p temper-engine-domain-world --tests` names.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`, `change_batch` only: add
  `|| change.branch.is_some()` to the single change's `plan` test and put `branch` in the
  refusal's words, exactly C1's. No other hunk in this file (other plans change it).
- `temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs`: the assertion of the
  old refusal words (near line 113) takes C1's words; add one story beside it,
  `a_branch_without_the_control_family_is_a_plans_member_refused`, the same shape with
  `{"words":"Small fix","branch":"mine"}`.
- If the schema's bytes are counted against a declaration budget (agent.md 4.3, the host
  part's budget) and a test of it fails on the new length: keep C1's text and report the
  budget test's numbers under `## Left`; do not shorten other descriptions.

## Steps

1. The type and the decoder, as above.
2. Tests in `temper-engine-smith/src/tests.rs`, beside `change_decodes_a_plan_member_each`:
   `change_decodes_branch_mine` (the single object and one inside `changes`) and
   `change_refuses_a_branch_other_than_mine` (`"theirs"`, `""`, `1`). Extend the test that
   asserts `PLAN_CHANGE`'s members, if one does, with `branch`.
3. The root's `plan` line and words, and the story, as above.
4. Narrow: `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/change_/)'`; then
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::change_tool::/)'` (check each selected some).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-smith -p temper-engine-domain-world --all-targets --
   -D warnings`; `heavy cargo nextest run -p temper-engine-smith -p temper-engine-domain`.
   Commit ("change: take branch mine, a plan's member") and push.

## Report

As the implementer's text says.
