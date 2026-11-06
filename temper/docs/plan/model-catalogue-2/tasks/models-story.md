+++
id = "models-story"
part = "page"
kind = "feature"
needs = ["models-add", "stock-models"]
files = ["temper/tests/client-browser/tests/browser_all/browser_models.rs", "temper/tests/client-browser/tests/browser_all/main.rs"]
effort = "medium"
minutes = 45
+++

# A browser story of the page

## Goal

A switch turned off and back, and a model added by name, with a reload between: written and compiled, not run. After-landing runs it, and a red comes back as a repair.

Acceptance:

- `a_switch_turned_off_and_back_holds_across_a_reload`: on Settings, Models, an owner turns a model off; the reload says "Off"; the composer's picker doesn't name it; All models does; turned back, it does again.
- `a_model_added_by_name_is_on_the_picker`: "Add a model by name", a name; its row reads "added by you"; the picker names it; a reload keeps it.
- Both are in the module `main.rs` names, in the shape of `browser_accounts.rs`, and `heavy cargo check -p` on `temper/tests/client-browser`'s package compiles.
- Neither runs: never `tools/tiers/run`, a `browser-*` profile or `scripts/browser.sh`. Say so in the report.

## Inputs

- `temper/tests/client-browser/tests/browser_all/browser_accounts.rs`, `browser_pages.rs`.
- `temper/docs/design/client/README.md`, 3.5, *The `Models` tab*.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. `temper/tests/client-browser/src/lib.rs`: a helper's line, if a helper's missing. The fake model and fixtures `process-fixtures` moved aren't yours.

Options, pre-approved: the budget's story list, if a tier's table names stories, gets its line; the story's name is yours within the shape above.

## Steps

`cargo fmt`; `heavy cargo clippy -p` on the package `--all-targets -- -D warnings`; the check above. Commit and push.

## Report

That it is compiled and not run.
