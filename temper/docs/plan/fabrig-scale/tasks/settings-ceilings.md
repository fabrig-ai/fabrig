+++
id = "settings-ceilings"
part = "jig-rows"
kind = "migration"
needs = ["tasks-pieces"]
files = ["jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/src/generated/settings_v1.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/golden/settings/**", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/tests/**"]
effort = "medium"
minutes = 75
+++

# settings-v1's verdicts and approvals hold a 256 KiB result, priced as tasks-v9's

## Goal

The settings family stores a template's contract as the tasks family does, and maps its field
rules' names and an approval's state from `tasks.result_bytes` (`codec/settings.rs`, `limits`).
At contracts K2's `result_bytes` 262,144, settings-v1 refuses: `FieldRule.name` is
`bytes 4096`, `Approval.state` `bytes 65536`, so `codec::settings::limits` is `None` and with it
`codec::limits` for jig's `standard` at fabrig's figures. A plain raise does not fit: settings-v1
keeps `Verdict.fields` a `list 512 FieldRule`, which at a 512 KiB name prices a verdict at
about 268 MB, and a `Proposal`'s 128 items holding templates of 8 such verdicts past `u32`.
So settings-v1 takes tasks-v9's shape for these fields (`jig/docs/design/protocol/store.md`,
5.1: a collection the domain bounds by its bytes in all is one piece), in place
(`notes/common.md`: no new version; an older layout is refused at open).

Acceptance:

- `jig/crates/jig-protocol-store/schema/settings-v1.schema`, these fields and no others:

  | Record.field | Was | Becomes |
  |---|---|---|
  | `FieldRule.name` | `bytes 4096` | `bytes 524288` |
  | `Verdict.fields` | `list 512 FieldRule` | `bytes 1048576 pieces FieldRule` |
  | `Approval.state` | `bytes 65536` | `bytes 524288` |

  (the same ceilings tasks-v9 takes for its `FieldRule`, `Verdict` and `Approval` in
  `store-ceilings`).
- `codec/settings.rs`: `limits` sets `verdict_fields` as `codec/tasks.rs` does,
  `min(VERDICT_FIELDS, result_bytes) × (FieldRule(EMPTY) + 4) + result_bytes`, checked, overflow
  `None`; `field_rule_name` and `approval_state` stay `result_bytes`. `encode_verdict` and
  `decode_verdict` write and read the fields as pieces, after `codec/tasks/pieces.rs`'s
  `rules` and `read_rules` (settings' own wire types; a decoded verdict past `VERDICT_FIELDS`
  rules or `result_bytes` of names is refused as the tasks family refuses it). No other mapping
  changes.
- At small's core limits with `tasks` at K2's figures (`result_bytes` 262,144,
  `message_bytes` 262,144, `spec_bytes` 131,072, `contract_choices` 4, `charters` 12,
  `executor_kinds` 16, `authority_grants` 16, `authority_segments` 8, `hold_segments` 8, the
  rest as `store-ceilings`'s `the_codecs_hold_fabrig_scale` sets them) and `authority`'s
  `grants` 16, `executors` 16, `segments` 8, `codec::settings::limits` is `Some`. If it is still
  `None` and the refusing ceiling is another settings-v1 field mapped from a K2 figure, raise
  that ceiling the least that holds the figure and list it in the report; if skein refuses the
  schema (`SchemaError::Large`), stop and ask with its words; never lower a figure.
- Generated code, layouts and goldens by the writer (`notes/common.md`:
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world --test drift`);
  commit `settings_v1.rs`, `layouts.rs` and any `golden/settings/` file the writer changed
  (say which; a `.bin` holding a verdict changes with its shape). A diff in another family's
  generated file means the tree is stale: say so.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2.
- `jig/docs/design/protocol/store.md`, 5.1 (pieces; a family's limits made from the domain's,
  never past its ceilings).
- `jig/crates/jig-protocol-store/src/codec/tasks.rs` (`limits`, `verdict_fields`) and
  `codec/tasks/pieces.rs` (`rules`, `read_rules`): the shape to follow.

## Owns and may change

- The files above. `jig-protocol-store`'s tests: a test that pins a settings ceiling these
  change takes the new figure; a new test (below).
- No configured limit or profile; no other family's schema or codec; not
  `jig/tests/store-codecs/tests/suite/profiles.rs` (`store-ceilings`'s).

## Steps

1. The schema; run the writer; `git diff --stat` shows only the files above (the codec will
   not compile until step 2: write both, then run the writer).
2. `codec/settings.rs` as above.
3. Tests in `jig/crates/jig-protocol-store/src/tests/codec/` (a new module beside
   `settings_proposals.rs`, `settings_scale.rs`):
   - `settings_hold_fabrig_scale`: the limits above give `Some`;
   - `a_verdict_round_trips_as_pieces`: a template whose verdict holds three field rules, one
     name of 262,144 bytes, encodes and decodes equal at those limits;
   - `settings_refuse_a_result_past_their_ceiling`: `result_bytes` 524,289 gives `None`.
4. Narrow: `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/settings_scale::/)'`;
   then `heavy cargo nextest run -p jig-store-codecs-world --test drift`, the packages
   `-p jig-protocol-store`, `-p jig-store-codecs-world`, `-p jig-store-world`,
   `-p jig-core-world`, and `heavy cargo check -p temper-service -p temper-twin --tests`.
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`; `heavy cargo clippy
   -p jig-protocol-store --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks, with every generated or golden file the writer changed and
any ceiling raised beyond the table, with the figure that asked for it.
