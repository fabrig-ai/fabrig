+++
id = "change-call-llm"
part = "change-temper"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_branch.rs"]
effort = "high"
minutes = 80
+++

# change-call-llm: `change` decodes `producer` and `llm`

## Goal

`change` decodes the two members of contract C4, a producer's charter and an LLM choice per
role, into `ChangeCall`, and offers them in its plan schema, only to a run whose authority
holds jig's control family. The root counts them as a plan's members (refused without the
control family, as `landing` is) and, until task `change-root` builds them, refuses a change
naming either. A call naming neither behaves exactly as today.

Acceptance:

- In `temper/crates/temper-engine-domain/src/boundary.rs` (exported as `ChangeCall` is):

  ```rust
  pub struct ChangeCall {
      // M1's members, unchanged, then:
      pub producer: Option<u32>,
      pub llm: ChangeLlm,
  }
  /// A change's LLM choices by role (domain/agent.md, 4.3.1; domain/forge.md, 8.1).
  #[derive(Clone, Default, PartialEq, Eq, Debug)]
  pub struct ChangeLlm {
      pub producer: Option<jig_domain_tasks::LlmChoice>,
      pub review: Option<jig_domain_tasks::LlmChoice>,
  }
  ```

  (`jig_domain_tasks` as the file already names it, e.g. `tasks::LlmChoice`.)
  `ChangeCall::words` gives `producer: None, llm: ChangeLlm::default()`.
- `temper-engine-smith`'s `change` decode (in `calls.rs`, reached from `call_configured`, which
  passes its `models: &[engine::Model]` down to it), for each change, the single one or each
  of `changes`:
  - `producer`: an integer from 1 to `u32::MAX`, else `Problem::Range(b"producer")` (a
    non-integer as `small` reports it).
  - `llm`: an object whose members are `producer` and `review`, each optional; any other
    member is `Problem::Range(b"llm")`. Each is jig's form by name (jig's
    `protocol/configuration.md`, 3.1): `{"main": {"model": NAME, "effort": NAME},
    "sub_agents": [{"model": NAME, "effort": NAME}, ...]}`, `main` required, `effort`
    optional (none: `ModelEffort.effort` `None`), `sub_agents` optional, at most
    `tasks::MODELS` items (else `Problem::TooLarge`). A model's name is looked up among
    `models` by `name`, byte for byte, giving its index in that slice
    (`Problem::Range(b"model")` when none has it); an effort's name among that model's
    `efforts`, giving its index (`Problem::Range(b"effort")` when it has none so named). The
    choice's `account` is `None`. Effort and charter fit are the root's (task `change-root`),
    not the decode's.
- The plan schema (`PLAN_CHANGE`) gains, between `after` and `changes`, byte for byte:

  ```json
  "producer":{"type":"integer","minimum":1,"description":"The charter its producer, repairs and resolutions run under; left out, the forge's change charter."},
  "llm":{"type":"object","description":"LLM choices by role, producer and review: each {main: {model, effort}, sub_agents: [{model, effort}]}, by configured names; a role left out takes its charter's default."},
  ```

  Today's schema (no control) is unchanged, byte for byte.
- The root, in `forge_route.rs`'s `change_batch` alone: a single change naming `producer` or a
  non-default `llm` is a plan's (the `plan` test), so a run without the control family is
  refused with the words, which become `change's landing, name, after, changes, producer and
  llm are a plan's: this task's authority lacks the control family (amend, cancel,
  release).`; then any change naming either is refused
  `ChangeRefused::Refusal(tasks::Refusal::Spec)`, before any delegate is built.
- Every existing story and test passes, edited only for the new fields and the words.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1: the bullets `producer` and `llm` and "Who may
  set them".
- `jig/docs/design/protocol/configuration.md`, 3.1: the `llm` form (its paragraph on a
  template's members), names by configuration.
- `temper/docs/design/domain/agent.status.md`: its **Not yet** entry "4.3, 4.3.1: `change`'s
  `producer` and `llm`" stays (task `change-root` removes it); this task edits no design file.
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C4.

## Owns and may change

Owns the files in the header.

- `boundary.rs`: the two members, `ChangeLlm`, `ChangeCall::words`; no other hunk.
- `calls.rs`: the decode (a helper per member, e.g. `change_llm(value, models)`; match
  `crate::json::Value::Object` to list `llm`'s members), `PLAN_CHANGE`; `change` gains the
  `models` argument from `call_configured`.
- `forge_route.rs`: `change_batch`'s `plan` test, its words, and the refusal above; no other
  hunk (other plans change this file).
- Tests: `temper-engine-smith/src/tests.rs` (the `change` decode tests near line 1379 and the
  plan schema at line 1495), the literal in `goal_branch.rs` (line 99) and the words in
  `change_tool.rs`. If `rg 'ChangeCall \{|are a plan' temper/crates temper/tests` finds a
  reader not listed, edit it the same way and name it in the report.
- Options: if `engine::Model` in `temper-engine-smith` does not carry `name` and `efforts`
  (jig's `jig-protocol-engine-clients` `Model` does), take them from the type it is and say
  so. If smith's schema check refuses the `llm` member's shape, keep its `type` and
  `description` alone (as written above it has no more) and say so.

## Steps

1. Types, decode, schema, the root's hunk, the literals.
2. Tests in `temper/crates/temper-engine-smith/src/tests.rs`, beside the `change` ones, with
   models `[{name: "sol", efforts: ["medium", "high"]}, {name: "haiku", efforts: []}]` given to
   `call_configured`:
   - `change_decodes_a_producer_and_llm_by_role`: `{"words":"a","producer":6,"llm":{"producer":
     {"main":{"model":"sol","effort":"high"},"sub_agents":[{"model":"haiku"}]},"review":{"main":
     {"model":"sol","effort":"medium"}}}}` gives `producer: Some(6)` and the choices `0.1 1`
     and `0.0`, account `None`; in `changes`, each change keeps its own.
   - `change_refuses_a_producer_or_llm_out_of_range`: `producer` 0, `-1`, `"6"`; `llm` with
     member `foreman`; a model `"opus"`; an effort `"max"`; `main` missing; 65 sub-agents:
     each its `Problem`.
   - The plan-schema byte test updated; the no-control schema test unchanged.
3. A story in `temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs` beside the
   control-family refusal: a run without control naming `producer` is refused with the new
   words; a run with control naming `llm` is refused `Spec`, and no delegate is made.
4. Narrow: `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/change_/)'` (check
   the count includes the new ones); `heavy cargo nextest run -p temper-engine-domain-world
   --test suite -E 'test(/forge_routes::/)'`; `heavy cargo nextest run -p temper-engine-domain
   --lib`; since the declaration is measured there, `heavy cargo nextest run -p temper-service
   --lib -E 'test(/stock_brief/)'`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-smith -p temper-engine-domain -p temper-engine-domain-world --all-targets --
   -D warnings`; `heavy cargo check -p temper-service --tests`. Commit ("Decode change's
   producer and LLM choices") and push.

## Report

As the implementer's text says, with the plan schema's byte count before and after.
