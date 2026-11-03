+++
id = "relay-types"
part = "relay"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge-change/src/lib.rs", "temper/crates/temper-engine-domain-forge-change/src/tests.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/limits/names.rs", "temper/crates/temper-service/src/tests/limits.rs"]
effort = "medium"
minutes = 35
+++

# relay-types: a change's question route, its relay's record and its bound, as types

## Goal

The pure types the relay's tasks share (contracts C1 and C2) exist in the change
procedure's crate, and the change policy's limits carry the question bound, 4 on `small`,
named `forge.change_policy.questions`. Nothing reads them yet: tasks `relay-surface`,
`relay-policy` and `relay-up` build on them.

Acceptance:

- `temper-engine-domain-forge-change/src/boundary.rs` holds C1's `Route` and C2's
  `QUESTIONS_OPEN`, `QUESTION_LINE`, `Questions`, `Open` and `Stands` exactly as
  `notes/contracts.md` writes them (names, fields, derives, doc comments), after `Limits`;
  `lib.rs` exports them as it exports the crate's other boundary types.
- `Limits.questions: u32` after `updates`, documented "Questions a change takes from its
  making or its last release (domain/forge.md, 8.11)."
- `temper-service/src/limits.rs`'s small `change_policy`: `questions: 4`, commented "The
  reference's bound (draft/reference-code-production.md, section 8)." as its siblings are
  commented; `limits/names.rs` takes `forge.change_policy.questions` as it takes
  `forge.change_policy.repairs` (a `u32`, `Refused::Range` past it).
- Every `temper_engine_domain_forge_change::Limits { .. }` literal in the tree gains
  `questions: 4` and nothing else.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11 ("Bounded"; "On its record and its panel");
  `temper/docs/design/protocol/configuration.md`, section 7 (overrides by dotted name).
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C1 and C2.
- This task edits no design file.

## Owns and may change

Owns the files in the header.

- The literals of `Limits`, each gaining `questions: 4` only:
  `temper-engine-domain-forge/src/tests/limits.rs`, `src/tests/verification.rs`,
  `src/tests.rs`, `temper-fake/src/tests/check.rs`,
  `temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper-protocol-store/src/tests/fixtures.rs`, `temper/tests/engine/forge/src/world.rs`,
  `temper/tests/codecs/tests/support/store.rs`. `rg 'forge_change::Limits \{' temper` lists
  them; one not listed is edited the same way and named.
- If `temper-service`'s limits have a test table of every override name (or a help page
  listing them that a test reads), add `forge.change_policy.questions` to it beside
  `repairs` and name the file; a `docs/help/` page so read may be edited for that one line
  (a stated departure from the implementer's text).

## Steps

1. The types, the field, the stock value, the override name, the literals.
2. Tests:
   - in `temper-engine-domain-forge-change/src/tests.rs`: `a_changes_questions_default_to_none`
     (`Questions::default()` has route `None`, `taken` 0, no open question) and
     `the_open_bound_leaves_room_for_the_one_past_the_bound` (`QUESTIONS_OPEN > 4`).
   - in `temper-service`'s limits tests (`src/tests/limits.rs`, or where `repairs`' name is
     tested): `forge.change_policy.questions` overrides the bound, and a value past `u32` is
     refused as `repairs`' is.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-forge-change`, then `heavy cargo
   nextest run -p temper-service --lib -E 'test(/limits/)'` (check the counts).
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge-change -p temper-service --all-targets -- -D warnings`; `heavy
   cargo check --tests` for `-p temper-engine-domain-forge`, `-p temper-fake`, `-p
   temper-engine-domain`, `-p temper-protocol-store`, `-p temper-engine-forge-world` and `-p
   temper-codecs-world`, one at a time. Commit ("Type a change's question route and bound")
   and push.

## Report

As the implementer's text says.
