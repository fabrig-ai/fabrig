+++
id = "goal-value"
part = "goal"
kind = "interface"
needs = ["goal-wire"]
files = [
  "temper/crates/temper-engine-domain-forge/src/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/kinds.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/settings.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs",
  "temper/crates/temper-protocol-store/src/records/settings.rs",
  "temper/crates/temper-protocol-store/src/tests/settings.rs",
  "temper/crates/temper-engine-smith/src/kinds.rs",
  "temper/crates/temper-engine-smith/src/tests/kinds.rs",
  "temper/crates/temper-engine-domain/src/route/kinds.rs",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 75
+++

# Kind 11 as the forge's value: judged, kept, stored

## Goal

A goal template's work becomes a value the forge holds: the forge has kind 11,
its `TemplateGoal`, its judgement and its accessor, and the store and smith's
codec carry it. Nothing declares the kind to jig yet (`goal-kind` does), so no
change sets one until then; every reader that asks gets none.

Acceptance, by `temper/docs/design/protocol/configuration.md`, 13.2 (*A goal
template's work, kind 11*):

1. **The value,** exactly `temper/docs/plan/ui-p3f-design/notes/contracts.md`,
   section 3: `TemplateGoal` (with its derives, `Copy` and `Default` among them);
   `Kind::TemplateGoal` (11) in `number` and `of`; `Value::Goal(TemplateGoal)`;
   `Bounds::templates`. Each new item's comment cites 13.2, as its neighbours do.
2. **The judgement** (`settings.rs`, `judge`), as section 3 of the note has it:
   kind 11 at scope 0 alone, numbers 1 to `bounds.templates`, a `Value::Goal` or
   none; any other value refused `Value`.
3. **The bound filled:** the root's `bounds`
   (`temper/crates/temper-engine-domain/src/route/kinds.rs`) gives `templates:
   limits.templates`; that one field, nothing else of the root.
4. **The accessor** (`kinds.rs`, beside `project_questions`):
   `template_goal(template) -> Option<TemplateGoal>`, through `setting`, so a
   stock default (`goal-stock`'s, later) answers as a change does and a removal
   answers none.
5. **Kept as the deployment's:** `erase_project_settings` leaves kind 11, as it
   leaves kinds 1 and 3; `stored_bytes` (`lib.rs`) counts it as it counts kind
   3's charter.
6. **The store's codec** (`temper/crates/temper-protocol-store/src/records/settings.rs`):
   `Value::Goal` as `SettingValue::Goal` through the store's `TemplateGoal`, and
   back; `goal-wire`'s `Undecodable` stand-in goes.
7. **Smith's codec** (`temper/crates/temper-engine-smith/src/kinds.rs`): `encode`
   and `size` write and measure `Value::Goal` through the client wire's
   `v1::TemplateGoal`, as they do kind 7's `ProjectWays`. `decode` of kind 11 is
   `goal-kind`'s: leave its default arm.
8. **Compile-forced arms** for `Value::Goal` and `Kind::TemplateGoal` wherever a
   `match` lists them, in any crate of temper's tree (the forge's `kinds.rs`
   and `lib.rs`; the root's `route/kinds.rs`, `route/ways.rs`' `project_ways`,
   `route/approvals.rs`, `route/policy_translate.rs`; the ways crate's `ways.rs`
   `laid`, `laid_ways` and `own_ways`; the service's `clients/translation.rs`
   and `stock/kinds.rs`; whichever else the compiler names): each treats kind
   11 as kind 3, the deployment's other one-number value, is treated where kind
   3 is not the one asked for (excluded, no charter named, the value's size).
   One that would need another choice of behaviour: stop and ask.
   `goal-path` changes `route/ways.rs`' `start` and `inputs` beside you: touch
   `project_ways` alone there.
9. **Rows,** no new test function:
   - the forge's judgement tables (`temper-engine-domain-forge/src/tests/settings.rs`,
     the deployment kinds' table): kind 11 at a project's scope refused `Scope`;
     number 0 and number `templates + 1` refused `Number`; number 1 and number
     `templates` good with each of the four values; a removal and a reset good;
     a `Value::Charter` for kind 11 refused `Value`;
   - the forge's keep and restore table (`tests/kinds.rs`, the one that drives
     keep, replace, restore and propose over its kinds' values): a kind 11 row;
     and beside the questions' default test, or the nearest table there:
     `template_goal` none with none held, the held value once held, none once
     removed; a project's deletion leaving kind 11 held;
   - the store's settings round trip (`temper-protocol-store/src/tests/settings.rs`):
     a kind 11 row for each of `{false, false}` and `{true, true}`, beside kind
     7's;
   - smith's round-trip values table and its size test
     (`temper-engine-smith/src/tests/kinds.rs`), if they enumerate `Value`s: a row
     for `Value::Goal`, the size the encoding's length.
10. **Bites:** let `judge` take kind 11 at scope 1, see its row fail; let
    `erase_project_settings` erase kind 11, see the deletion row fail; restore
    each.
11. **Status:** delete the bullets naming `goal-value` in
    `temper/docs/design/protocol/configuration.status.md` (13.2) and
    `temper/docs/design/protocol/store.status.md` (2) (`notes/common.md`, *Status
    files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2: *A goal template's work,
  kind 11*, and, for the mechanism it shares, *Judged by the forge*, *Kept by
  the forge*, *Erased with their project*.
- `temper/docs/design/protocol/store.md`, section 2, family 20 `Setting` and
  *Family 20 also keeps kind 11*.
- `temper/docs/plan/ui-p3f-design/notes/contracts.md`, sections 1 to 3.
- `temper/docs/plan/ui-p3f-design/notes/common.md`.
- The code, kind 3 (the deployment's) and kind 7 (a record) end to end as the
  model: `temper-engine-domain-forge/src/settings.rs` (`Kind`, `ProjectWays`,
  `Value`, `Bounds`, `judge`, `Refusal`), `src/kinds.rs`
  (`erase_project_settings`, `project_questions`, `setting`, `value_state`),
  `src/lib.rs` (`stored_bytes`); `temper-protocol-store/src/records/settings.rs`;
  `temper-engine-smith/src/kinds.rs`.

## Owns and may change

- Owns: the forge's kind 11, its value, judgement, accessor and held size; the
  root's one `bounds` field; the store's and smith's codec arms; the rows; the
  two status bullets.
- Compile-forced only (acceptance 8): the arm a `match` on `Value` or the forge's
  `Kind` needs, each named in the report. Not the service's kind declarations,
  the wire's `settings_kind`, smith's `decode` and setting names, the root's
  reads (all `goal-kind`'s), the stock's defaults (`goal-stock`) or the path
  (`goal-path`, `goal-read`).
- A catch-up that meets kind 8, 9 or 10 (`notes/common.md`, *Beside this plan*)
  in these files keeps both.
- **The bound:** if the root's `Limits::templates` isn't jig's `core.templates`
  as the engine's limits give it, say which it is in the report.

## Steps

1. Read the inputs.
2. The value, judgement, accessor and size; their rows; the bites.
3. The store's and smith's codecs and their rows; the forced arms, by `heavy cargo
   check --tests` over the packages of step 4.
4. Runs: `heavy cargo nextest run -p temper-engine-domain-forge -p
   temper-protocol-store -p temper-engine-smith`; the drift test without writing
   (`notes/common.md`), to show no schema moved; `heavy cargo check --tests -p
   temper-engine-domain -p temper-engine-domain-ways -p temper-service -p
   temper-codecs-world -p temper-client-tools`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-engine-domain-forge -p temper-protocol-store -p temper-engine-smith
   --all-targets -- -D warnings`, with every other package acceptance 8 touched.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the forge's surface as written, every forced
arm by file, the rows and the bites, the runs' counts.
