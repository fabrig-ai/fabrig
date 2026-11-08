+++
id = "classes-value"
part = "classes"
kind = "interface"
needs = ["reach-kit", "reach-wire"]
files = [
  "temper/crates/temper-protocol-reaches/**",
  "temper/crates/temper-engine-domain-forge/src/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/kinds.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs",
  "temper/crates/temper-engine-domain-forge/Cargo.toml",
  "temper/crates/temper-protocol-store/src/records/settings.rs",
  "temper/crates/temper-protocol-store/src/tests/settings.rs",
  "temper/crates/temper-protocol-store/Cargo.toml",
  "Cargo.lock",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 90
+++

# Kind 8's value: the kit's records, the forge's value, stock and judgement

## Goal

A project's classes become a value the forge holds: the kit converts tables and reaches to and from their records, the forge has `Value::Classes` for kind 8 with its stock (Agents review's table), judges an item of it by the kit's `check`, erases it with its project and answers `project_classes`, and the store keeps it. Nothing declares kind 8 to jig yet (`classes-kind` does), so no project sets one until then; every reader of a project's classes already gets the stock.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.2 and 13.4, and `temper/docs/design/domain/forge.md`, 5.10:

1. **The kit's records,** `temper/docs/plan/ui-reaches/notes/contracts.md`, section 3, exactly: the kit gains `temper-client-protocol-wire` as its second dependency and `classes_from`, `classes_record`, `reach_from`, `reach_record`, `subject_from`, `subject_record` and `largest`, `pub` at its root. `largest()` passes `check`, and its record encodes.
2. **The forge's value,** section 3 of the note: `Kind::ProjectClasses` numbered 8 (`number`, `of`); `Value::Classes(temper_protocol_reaches::Classes)`; `project_stock(8)`, Agents review's table measured as its record's encoding, as kind 7's is measured; `Domain::project_classes(project)` beside `project_questions`, the held kind 8 number 1 of that project, else the stock.
3. **Its judgement** (`settings.rs`, `judge`): kind 8 of a project's scope alone, number 1 alone; a `Value::Classes` whose table `check` refuses is refused by a new `Refusal::Classes(temper_protocol_reaches::Refusal)`, its `words` one static line per kit refusal saying the fault ("an item of a kind that always waits", "a path set's pattern that is not well formed", and so on, the forge's words beside the others'); a removal or a reset is good; any other value is `Refusal::Value`.
4. **Erased with its project:** `erase_project_settings` takes kind 8 beside 2, 6 and 7.
5. **Its size held:** `stored_bytes` (`lib.rs`) counts a held table's boxed items, words, sets, names and patterns, as it counts a rule's segments.
6. **The store's codec** (`temper/crates/temper-protocol-store/src/records/settings.rs`): `Value::Classes` encodes as `SettingValue::Classes` through the kit's record, and decodes back, a table `check` refuses being `Undecodable`; `reach-wire`'s stand-in arm goes.
7. **Compile-forced arms** for `Value::Classes` and `Kind::ProjectClasses` wherever a `match` lists them (the survey found, at least: the forge's `kinds.rs` `project_questions`, `setting_charter`, `value_state`; the root's `route/kinds.rs` `way_refusal` and `approval_for_projects`, `route/ways.rs` `project_ways`; the ways crate's `ways.rs` `laid`, `laid_ways`, `own_ways`; smith's `kinds.rs` `encode` and `size`): each treats kind 8 as kind 7 is treated where kind 7 is not the one asked for (excluded, no charter named, the value's size), and smith's `encode` and `size` write and measure the table by `classes_record`. Smith's `decode` of kind 8 is `classes-kind`'s: leave its default arm.
8. **Rows,** no new test function:
   - the kit: in its tests, a table of record round trips: each way's stock table, `largest()`, a table with sets and worded items; a reach of each reason, `reach_from` refusing a set reason numbered 0 and 7; `subject_from` and `subject_record` inverse for every kind and a set;
   - the forge's project-kind judgement table (`temper/crates/temper-engine-domain-forge/src/tests/settings.rs`, about line 400): kind 8 at scope 0 refused `Scope`, number 2 refused `Number`, the stock table good, a table `check` refuses (an item of `money`) refused `Classes`, a removal and a reset good, a `Value::Ways` refused `Value`;
   - the forge's `project_values` table (`tests/kinds.rs`, about line 738), which drives deletion and keep, replace, restore and propose: a kind 8 row; and beside the questions' default test (about line 783), `project_classes` answering the stock with none held and the held table once held, as rows of that test if it is a table, else of the nearest table there;
   - the store's settings round trip (`temper/crates/temper-protocol-store/src/tests/settings.rs`, about line 32): a kind 8 row with a table holding a set and a worded item, beside kind 7's.
9. **Bites:** break `judge`'s check (accept every table), see the `money` row fail; break `erase_project_settings` (drop 8), see the deletion row fail; restore each.
10. Runs, counts in the report: `heavy cargo nextest run -p temper-protocol-reaches -p temper-engine-domain-forge -p temper-protocol-store`; `heavy cargo nextest run -p temper-protocol-reaches --profile fuzzy`; the drift test (`notes/common.md`, *Generated files*) without writing, to show no schema moved; `heavy cargo check --tests -p temper-engine-domain -p temper-engine-domain-ways -p temper-engine-smith -p temper-service -p temper-codecs-world`; `heavy cargo clippy -p temper-protocol-reaches -p temper-engine-domain-forge -p temper-protocol-store --all-targets -- -D warnings`, and for every other package acceptance 7 touched.
11. Status: delete the **Not yet** entry naming `ui-reaches/classes-value` in `temper/docs/design/protocol/configuration.status.md`.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`; `temper/docs/plan/ui-reaches/notes/contracts.md`, sections 1 to 3.
- `temper/docs/design/protocol/configuration.md`, 13.2 (the kinds' table, *A project's classes' value*, *Judged by the forge*, *Erased with their project*) and 13.4 (kind 8's stock row, *A project's kinds 6, 7 and 8 have no row in the stock*).
- `temper/docs/design/domain/forge.md`, 5.10, *The table*, *The stock tables*.
- The code, kind 7 end to end as the model: `temper-engine-domain-forge/src/settings.rs` (`Kind`, `ProjectWays`, `Value`, `project_stock`, `judge`, `Refusal`), `src/kinds.rs` (`erase_project_settings`, `project_questions`), `src/lib.rs` (`stored_bytes`); `temper-protocol-store/src/records/settings.rs`.

## Owns and may change

Owns the kit's `record` module and its manifest line; the forge's `Kind::ProjectClasses`, `Value::Classes`, `project_stock(8)`, `project_classes`, `Refusal::Classes`, kind 8's judgement and erasure and its held size; the store's codec arms; the rows; the status entry.

May change, compile-forced only (acceptance 7): the arm a `match` on `Value` or the forge's `Kind` needs, in any crate of temper's tree, each named in the report. Not the service's kind declarations, the wire's `settings_kind`, smith's `decode` and setting names, the root's reads (all `classes-kind`'s), or the stock ways (`classes-ways`').

Options, each decided:

- **`project_stock(8)` measures** the stock table's record through `v1::ProjectClasses::new(&v1::CEILINGS, ..)` and `measure()`, as kind 7's does; if the generated constructor takes another shape, follow it and say so.
- **The forge's `Value` derives** a trait the kit's `Classes` lacks: add nothing to the kit but what the contract has; if `Value` needs `Ord` or `Copy`, stop and ask.
- **A forced arm in the root or the ways crate** that would need a choice of behaviour beyond "as kind 7 when kind 7 isn't the one asked for": ask.

## Steps

1. Read the Inputs; the kit's `record` module and its rows.
2. The forge's value, stock, accessor, judgement, erasure and size; its rows; the bites.
3. The store's codec and its row; the forced arms (acceptance 7), by `heavy cargo check -p PKG --tests` over the packages of acceptance 10.
4. The runs of acceptance 10; the status entry; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: the conversions and the forge's surface as written, `Refusal::Classes`'s words, every forced arm by file, the rows and the bites, the runs' counts.
