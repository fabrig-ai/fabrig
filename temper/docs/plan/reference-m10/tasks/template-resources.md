+++
id = "template-resources"
part = "template"
kind = "feature"
needs = []
files = ["temper/crates/temper-settings-form/src/**", "temper/crates/temper-client-tools/src/jig/settings_adapter.rs", "temper/crates/temper-client-tools/src/tests/jig/settings.rs"]
effort = "high"
minutes = 90
+++

# A template's resources in a settings file

## Goal

A settings file's template carries `resources`, the connectors' symbols a task filled from it
holds, as jig's format has them, so the reference's goal template for drafts can name the
forge's *its own goal branch* (`temper/docs/design/client/tools.md`, 5.3; jig's
`jig/docs/design/protocol/configuration.md`, 3.3). Today the form has no such member and every
template applied from a file names none.

Acceptance:

- `temper-settings-form`'s `view::Template` gains `pub resources: Box<[TemplateSymbol]>` (or the
  crate's list type its other list members use), first or last as the crate's other members
  suggest, with a new `view::TemplateSymbol { pub connector: u32, pub symbol: u32 }` (take
  `connector`'s and `symbol`'s integer types from `jig_client::contract::Symbol`, which they
  mirror).
- The settings file's form: a template's value takes an optional `"resources": [{"connector":
  C, "symbol": S}, …]`, left out `[]`; a row for it in `FIELDS` (`src/fields/jig.rs`, beside the
  template's `llm` row: its shape, a list bounded by the limit jig's `me` gives for a task's
  holdings where the form has it, otherwise by the bound jig's own decoder takes, and a line
  "The connectors' resources a task made from it holds, such as the forge's own goal branch.").
  Decoded bounded as the form's other lists are: past the bound, a pair twice, or a member of
  the wrong type, refused naming the member's path (`template.resources`).
- `temper-client-tools/src/jig/settings_adapter.rs`: `client_template` gives `c::Template` the
  file's resources (no longer an empty list), and `form_template` gives `n::Template` the
  template's, so `settings show --json` prints them and its output applies again unchanged.
- Rendering (`settings show` in text, `src/render.rs` or where a template's members are
  rendered) shows a template's resources, each as `connector C symbol S`, and nothing when
  there are none.
- Tests:
  - in `temper-settings-form` (its file tests, `src/settings_file_tests.rs` or beside them): a
    template with `"resources": [{"connector": 1, "symbol": 1}]` decodes to that list; one with
    none decodes to `[]`; a pair twice and a list past its bound are refused naming
    `template.resources`;
  - in `temper-client-tools/src/tests/jig/settings.rs`: a template applied from a file with
    resources reaches jig's `Template` with them, and one read from jig prints them in
    `settings show --json` and applies again to the same value.
- The focused tests of `temper-settings-form`, `temper-client-tools` and `temper` (whose
  `reference_settings` test decodes the reference's file) pass.

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/client/tools.md`, 5.3 (the settings file: a template's `resources`).
- `jig/docs/design/protocol/configuration.md`, 3.3 (a template's own resources).
- `temper/docs/design/domain/forge.md`, section 2 (the forge's symbol 1, *its own goal branch*).

## Owns and may change

- Owns `temper-settings-form` (its sources and tests), `settings_adapter.rs` and the client
  tools' settings tests, for what acceptance names.
- Every `view::Template { … }` literal elsewhere in temper (the reference's test, the
  configuring agent's tools, fixtures; found with `rg 'Template \{' temper/crates`): the new
  member only, `[]` (or the list type's empty), compiler-forced.
- `temper/docs/examples/reference/settings.json` is not yours: a later task names the goal
  branch in template 4.
- If jig's client contract's `Symbol` cannot carry a resource applied from a file (its settings
  change refuses a template's resources, or drops them), stop and ask, quoting what refuses.

## Steps

1. The view types and the form's row; the decoder; the adapter both ways; rendering.
2. The tests; run each narrowly (`heavy cargo nextest run -p PKG --lib -E 'test(=…)'`).
3. `cargo fmt`; `heavy cargo clippy -p temper-settings-form -p temper-client-tools --all-targets
   -- -D warnings`; their focused tests and `temper`'s; `heavy cargo check` on every other
   package whose `Template` literal you touched, with `--tests`; commit and push.

## Report

As the implementer's text asks.
