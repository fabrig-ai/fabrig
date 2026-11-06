+++
id = "spend-template"
part = "template"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/first_repository.rs",
  "temper/crates/temper/tests/suite/startup.rs",
]
effort = "medium"
minutes = 20
+++

# The process stories' purse reads init's template again

## Goal

`first_repository::run_spend` parses `temper-init/template/temper.json` unfilled, and since `8a8a1268fd` that template's `engine.clients.provider` is the whole-string placeholder `"$provider"`, which `temper_service::startup::engine` refuses: `init engine settings: a problem at clients.provider`. Every story that scripts a first repository dies there, before its box starts. This is that helper's fault alone: the box's own path fills the template (`temper-init/src/template.rs`, `documents`).

Acceptance:

1. `run_spend` parses again, with the template's provider read as init writes it when a box has no sign-in: JSON `null` (`temper/docs/design/protocol/sign-in.md`, 9.9: "the document has `clients.provider` ... `null`").
2. A focused test in `temper/crates/temper/tests/suite/startup.rs`, `a_story_purse_is_priced_from_the_init_template`, is shown red without the fix, with the message above, and passes with it.
3. No process story runs in this task: `a_csv_change_...` and `playground_setup_...` still need `restart-handover`, which runs them.

## Inputs

- `temper/docs/plan/ui-restart-red/notes/common.md`
- `temper/docs/plan/ui-restart-red/inputs/request.md`, *What a foreman read* item 1
- `temper/docs/design/protocol/setup.md`, 8.3 (the template's whole-string placeholders); `temper/docs/design/protocol/sign-in.md`, 9.9

The other readers of the template were surveyed for this spec and need nothing. Each reads a member `$provider` doesn't touch, or the filled document: `temper-init/src/tests/documents.rs`, `temper-playground/src/tests/stories.rs`, `temper/src/tests/reference_settings.rs`, `temper-fake/src/oauth/tests.rs`, `temper-fake/src/tests/demo.rs`, `temper/tests/suite/fake_opening.rs`, `temper/tests/suite/startup.rs` (`bounded_template` fills it; `turn_bound_document` reads limits), `temper/tests/engine/domain/src/forge_routes.rs`, `temper/tests/engine/domain/tests/fixtures/stock.rs`. Only `run_spend` hands the unfilled template to `startup::engine`.

## Owns and may change

- `temper/crates/temper/tests/support/first_repository.rs`: `run_spend` alone, and its visibility.
- `temper/crates/temper/tests/suite/startup.rs`: the one new test, at the end of the file.

**Options:**

- If your base's `run_spend` already reads `temper-init/template/bare.json` (`model-catalogue-1/stock-seven` makes that change, and may land first): the cause is gone. Leave its body as it is, add the test and its visibility, and say so in the report. The test then can't be shown red by the fix; show it red by pointing `run_spend` at an unfilled `"$provider"` once, by hand.
- If `startup::engine` then refuses something else in the unfilled template: treat it the same way, as init would fill it with no sign-in, and name it in the report.

## Steps

1. In `run_spend`, after the truncations and before `to_vec`:

   ```rust
   // Init fills this; with no sign-in it writes null (protocol/sign-in.md, 9.9).
   template["engine"]["clients"]["provider"] = Value::Null;
   ```

   Keep the existing comment on the demo's socket. Make `run_spend` `pub`, so the suite can name it.

2. At the end of `suite/startup.rs`:

   ```rust
   #[test]
   fn a_story_purse_is_priced_from_the_init_template() {
       assert!(support::first_repository::run_spend(5).is_power_of_two());
   }
   ```

3. Run it without the fix, then with it: `heavy cargo nextest run -p temper --test suite -E 'test(=startup::a_story_purse_is_priced_from_the_init_template)'`. Check the name by its count: a run that selected nothing proves nothing.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

`Done:` first. The red's message and the pass, the branch and commit, and your implementation minutes and check minutes apart.
