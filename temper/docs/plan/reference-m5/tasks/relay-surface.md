+++
id = "relay-surface"
part = "relay"
kind = "interface"
needs = ["relay-types"]
files = ["temper/crates/temper-engine-domain-forge-change/src/boundary.rs", "temper/crates/temper-engine-domain-forge-change/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/limits.rs", "temper/crates/temper-protocol-store/schema/store-v11.schema", "temper/crates/temper-protocol-store/src/generated/v11.rs", "temper/crates/temper-protocol-store/golden/v11/**", "temper/crates/temper-protocol-store/src/records/change.rs", "temper/crates/temper-protocol-store/src/tests/**", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema", "temper/crates/temper-client-protocol-wire/src/generated/v1.rs", "temper/crates/temper-client-protocol-wire/golden/**", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-client-tools/src/change.rs", "temper/crates/temper-client-domain/src/forge/change.rs", "temper/crates/temper-client-domain/src/forge/words.rs"]
effort = "high"
minutes = 80
+++

# relay-surface: a change's questions on its record, its parameter, its hold

## Goal

The surface the relay's root and panel build on (contracts C1 to C4), over task
`relay-types`' pure types: the relay's record on the change's row and in the store,
parameter 14 decoded as the change is made, the foreman's charter kept on the row, and the
hold past the bound, `Hold::Questions`, carried to the store, the wire, `ctl` and the web
client's words. No
change takes a question yet: every change's route is `none` unless parameter 14 says
otherwise, and nothing writes parameter 14 until task `relay-up`; the procedure's step reads
none of it.

Acceptance:

- `temper-engine-domain-forge-change`'s `boundary.rs`: `Hold::Questions` after
  `Authority(..)`, documented as C1 says. Every compiler-forced arm in the crate's
  `domain.rs` treats `Hold::Questions` as `Hold::Repairs` is treated where the arms group the
  bounds (near line 310); no other behaviour changes.
- `temper-engine-domain-forge`'s `ChangeRow` gains `foreman_charter: u32` after
  `review_charter` and `questions: change::Questions` at its end, documented "(domain/forge.md,
  8.11)". `limits::valid` refuses `change_policy.questions` above `QUESTIONS_OPEN - 1`.
- The store (`store-v11.schema`, in place at version 11: `temper/docs/design/protocol/store.md`,
  section 4): C2's records and enums, `ChangeRow`'s two fields, and `ChangeHold`'s
  `questions` appended last. Regenerate with `heavy cargo run -p skein-codegen --
  temper/crates/temper-protocol-store/schema/store-v11.schema
  temper/crates/temper-protocol-store/src/generated`, keeping the regenerated
  `src/generated/v11.rs` and `golden/v11/**`; translate both ways in `records/change.rs`;
  a store bound that names `ChangeRow`'s fields (as `change_row_report` does) gains the held
  answer's bound, the deployment's message bytes. A row written before this change is not
  read: the version stays 11 as reference-m1's tasks kept it.
- The root (`route/forge_route.rs`): `const FOREMAN_CHARTER: u32 = 6;` beside where
  `review_charter: 4` is set; `start_change` decodes parameter 14 (`Number` 1 `Foreman`, 2
  `Up`; any other value, or another kind, fails the start as an unknown parameter does; absent,
  `Route::None`), sets `questions: Questions { route, taken: 0, open: [] }` and
  `foreman_charter: FOREMAN_CHARTER`. `hold_change_decision` maps `Hold::Questions` as
  `Hold::Repairs` is mapped (`tasks::Hold::Procedure`), with C1's words, the bound from
  `change_policy.questions` and the first open question's asker and number (none open: the
  words end after "taken").
- The wire (`client-wire-v1.schema`'s `enum Hold`): `questions` appended after
  `authority`, regenerated with `heavy cargo run -p skein-codegen --
  temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema
  temper/crates/temper-client-protocol-wire/src/generated`, keeping `golden/**`;
  `temper-service/src/family.rs` maps `change::Hold::Questions` to it; `ctl`'s
  `temper-client-tools/src/change.rs` prints it `questions`; the web client's
  `temper-client-domain` maps it to `ChangeWord::QuestionsSpent`, its words "its questions are
  spent", beside `RepairsSpent` in `forge/words.rs`'s table and lists.
- Every test and fixture that builds a `ChangeRow` compiles with the new fields at their
  defaults (`Questions::default()`, `foreman_charter: 6`) and passes.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 (its questions, a parameter) and 8.11 (the
  bound, the record); `temper/docs/design/protocol/configuration.md`, 13.2 ("The forge's
  charters are read when a change is made"); `temper/docs/design/protocol/store.md`, section
  2 (family 12, `Change`) and section 4; `temper/docs/design/client/README.md`, 4.6 (a hold
  past its questions).
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C1 to C4; task
  `relay-types` made C1's `Route`, C2's types and `Limits.questions`.
- `temper/docs/design/protocol/store.status.md`'s entry for the `Change` family's questions
  and `temper/docs/design/domain/forge.status.md`'s for 8.1, 8.11 stay (later tasks remove
  them); this task edits no design file.

## Owns and may change

Owns the files in the header, each hunk kept to the fields, variants and arms above.

- Readers that build these types, edited only to add the new fields or arms (the literal's
  new field, the compiler-forced arm): `temper/tests/engine/forge/src/changes.rs`,
  `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`,
  `temper/crates/temper-engine-domain-forge/src/tests.rs`, `src/tests/fixtures.rs`,
  `src/tests/limits.rs`, `temper/crates/temper-fake/src/tests/check.rs`,
  `temper/tests/engine/protocol-forge/**`, `temper/crates/temper-engine-domain/src/tests/documents.rs`,
  `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs`,
  `temper/tests/codecs/tests/support/store.rs`, `temper/crates/temper-service/src/tests/family.rs`
  (its hold table gains the `Questions` row), `temper/crates/temper-client-protocol-wire/src/tests/**`,
  `temper/crates/temper-client-domain/src/tests/change.rs` and
  `temper/crates/temper-client-view/src/tests/change.rs` (each hold table gains its row:
  `questions`, `QuestionsSpent`, "its questions are spent"). `rg 'ChangeRow \{|Hold::Repairs' temper`
  lists them; a reader not named here is edited the same way and
  named in the report.
- If the store schema's spelling for an enum variant carrying a value differs from C2's
  sketch, use the schema's (as `ChangeHold`'s `authority` is spelled) and say so. If the
  generated wire `Hold`'s new tag would renumber an existing variant, stop and ask.
- If `ChangeHold`'s store bound for the held answer needs a limit the forge's `Limits` does
  not hold, use `forge.client.op_bytes` as `change_row_report` does and say so.

## Steps

1. `Hold::Questions`; row fields; store schema and its regeneration; translations; the
   root's decode and constant; `limits::valid`; the wire's `Hold` and its three readers;
   fixtures.
2. Tests:
   - `temper-protocol-store`: the regenerated goldens, and a round trip of a `ChangeRow`
     whose `questions` holds one `Open` of each `Stands` (`Taken`, `Foreman { task: None }`,
     `Foreman { task: Some(9) }`, `Asked`, `Held { answer }`), route `Foreman`, `taken` 3,
     `foreman_charter` 6, beside the existing change row round trip.
   - `temper-engine-domain`'s `route/forge_route/tests.rs` (or the domain world's
     `forge_routes` suite where `start_change` is driven): `a_change_made_with_parameter_14_keeps_its_route`
     (1 gives `Foreman`, 2 `Up`, absent `None`, 3 fails the start as an unknown parameter);
     `a_hold_past_questions_names_its_bound`, the hold's words as C1 gives them.
   - `temper-engine-domain-forge`'s limits tests: `change_policy.questions` of
     `QUESTIONS_OPEN` is invalid, one below is valid.
3. Narrow: `heavy cargo nextest run -p temper-protocol-store`, `-p
   temper-engine-domain-forge-change`, `-p temper-engine-domain-forge`, `-p
   temper-client-protocol-wire`, `-p temper-client-domain --lib -E 'test(/change/)'`, `-p
   temper-client-view --lib -E 'test(/change/)'`, `-p temper-service --lib -E
   'test(/family/) | test(/limits/)'`, `-p temper-engine-domain --lib`, `-p
   temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`, `-p
   temper-engine-forge-world`, `-p temper-engine-protocol-forge-world` and `-p
   temper-codecs-world` (each one at a time, checking the count).
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge-change -p temper-engine-domain-forge -p temper-protocol-store
   -p temper-engine-domain -p temper-service -p temper-client-protocol-wire -p
   temper-client-tools -p temper-client-domain --all-targets -- -D warnings`; `heavy cargo
   check -p temper-fake --tests` and `-p temper-client-view --tests`. Commit ("Keep a
   change's questions on its record") and push.

## Report

As the implementer's text says, naming each reader edited outside the header's files.
