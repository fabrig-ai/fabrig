+++
id = "charter-delivery"
part = "delivery-grant"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-protocol-charter/src/boundary.rs",
  "jig/crates/jig-protocol-charter/src/assignment.rs",
  "jig/crates/jig-protocol-charter/src/charter.rs",
  "jig/crates/jig-protocol-charter/src/tests.rs",
  "jig/docs/design/domain/hosts.status.md",
]
effort = "high"
minutes = 50
+++

# A run may deliver with no change among its outcomes

## Goal

`charter.rs` takes `deliver` from `outcome.change` alone, and `modify` only with it. Let a Finite run with a report take `deliver` where its policy's `modify` is set and its root says its workspace delivers. Its `finish` keeps its contract's forms. The mechanism names no application.

Acceptance:

- `Charter` has `delivers: bool` (`notes/contracts.md`), and `assignment` sets it `false`, so every run that exists today translates exactly as it did.
- The translation follows `notes/contracts.md` and this table, one test row each:

  | Contract | Ended by | `modify` | `delivers` | `deliver` | `grants.tools.modify` |
  |---|---|---|---|---|---|
  | Report | Result | yes | yes | title and body, checks must pass | yes |
  | Report | Result | yes | no | none | no |
  | Report | Result | no | yes | none | no |
  | Report | Requester | yes | yes | none | no |
  | Verdict | Result | yes | yes | none | no |
  | Change | Result | yes | no | the contract's | yes |
  | Change | Result | no | no | the contract's | no |
  | Change | Requester | yes | yes | none | no |

- A delivering report's fields are bounded by `CONTRACT_TEXT_MAX` and `CHANGE_BODY_MAX`, whatever the report's own `words`. A test holds both, with `words` small.
- Its `outcome` is unchanged: `outcome.change` none, `outcome.report` some, `outcome.failure` some, no verdict. A test holds that no `no_change` is offered.
- Encoded, it carries the grant: as `finite_mode_is_encoded_with_delivery` does, with a Report contract and `delivers`.
- `modify_needs_a_delivery` and `workspace_tools_follow_every_combination_of_run_policy` pass unchanged.

## Inputs

- `jig/docs/design/domain/hosts.md`, 8.
- `smith/docs/design/domain/run.md`, 7.1 and 8.4.
- `temper/docs/plan/goal-delivery/notes/common.md` and `notes/contracts.md`.

## Owns and may change

- **Owns:** `jig/crates/jig-protocol-charter/src/{boundary,assignment,charter,tests}.rs`.
- **May change:**
  - `temper/crates/temper-engine-smith/src/room.rs`, `measurement_charter`'s literal: `delivers: false`, nothing else. This is the producer's line in the other tree.
  - `jig/docs/design/domain/hosts.status.md`: remove the **Not yet** entry for section 8 that names `temper/docs/plan/goal-delivery/`, once this holds, and nothing else. This departs from the implementer's text, which closes `docs/design/`, because the status says what's built.
- **Not here:** `temper-engine-smith/src/start.rs`, which `goal-start` changes. Smith: if smith refuses this charter, stop and ask, since its design says it admits it.
- **Options:**
  - Clippy's `struct_excessive_bools` on `Charter`: a scoped `#[expect]` with a reason, as `RunPolicy` has.
  - `hosted.rs` builds a `Charter` through `assignment`: it stays false there, with no new part.

## Steps

1. Write the table's test first, in `tests.rs`, beside `modify_needs_a_delivery`, with the fixture's `delivers` set on the assembled charter. It fails to compile until the field exists: add the field and `assignment`'s `false`, then see the `yes` row fail for the rule's reason.
2. Change `charter.rs`, building the `ChangeSpec` there from `CONTRACT_CHECKS_MUST_PASS`, `CONTRACT_TEXT_MAX` and `CHANGE_BODY_MAX`, with the field names `title` and `body` that `assignment.rs`'s Change uses. Name them once.
3. Say in the module's comment and the field's what grants it, citing `domain/hosts.md`, section 8.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-charter --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-charter`; `heavy cargo check -p temper-engine-smith --tests`.
5. Commit and push.

## Report

The rule you broke by hand and the failure it gave, the test count, and any other literal the compiler sent you to.
