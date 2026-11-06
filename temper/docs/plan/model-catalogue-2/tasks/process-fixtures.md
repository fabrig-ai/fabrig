+++
id = "process-fixtures"
part = "stock"
kind = "migration"
needs = ["stock-models"]
files = ["temper/crates/temper/tests/support/**", "temper/crates/temper/tests/fixtures/**", "temper/tests/client-browser/src/**", "temper/tests/system/src/**", "temper/tests/real-loop/tests/fixtures/**"]
effort = "medium"
minutes = 80
+++

# The slow tiers' documents take no models

## Goal

The process, browser and live tiers write their own documents. Bring each in line with `stock-models`, so that after-landing finds a repair rather than a refused start on every story. None of them runs here: compile them, and read each writer's output against `a_documents_models_are_refused_at_their_paths`.

Acceptance:

- `rg -n 'llm.*models|"models"' ` over the header's files finds only a fake LLM's own lists (the `llm.json` files and `temper-fake`'s), each left as it is.
- Each deployment document has providers with their `endpoint`, numbered so that the stock's rows it wants are defaults.
- A fake LLM's list in each fixture holds the stock's names its world's chats run on, or the first the charter takes, by provider. `demo_document` keeps its scripted model.
- `retain_live_model`, `configure_live_model` and `forge_land/rung.rs`: a run that needs a larger output raises its agent endpoint's `limits` in the document, and sets its model's window and output by an owner's `temper ctl` change of settings after the start, as a person would.
- `heavy cargo check -p temper --tests` and `-p` on each of the tests' packages compile. No slow-tier story runs: say so in the report. `T/tests/support/sign_in.rs:509`, which read an agent model, reads what it needs from the stock's name.

## Inputs

- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 5, 6 and 7.
- `temper/docs/design/protocol/configuration.md`, 13.8.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files: `T/tests/support/{configuration,forge_land,forge_land/rung,sign_in}.rs`, the JSON at `T/tests/fixtures/cluster/engine.json`, `T/tests/support/github/engine.json`, `temper/tests/client-browser/src/{configuration,lib}.rs`, `temper/tests/system/src/composed/process.rs`. `T` is `temper/crates/temper`.

Options, each pre-approved:

- Fixture JSON that a focused test also reads, already changed by `stock-models`: leave it.
- If a story's fake LLM can't take a stock name without rewriting the story, give the fake that name with the story's own script, and report the story.

## Steps

The writers, then the JSON, then the checks. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo check` as above. Commit and push.

## Report

Each story you read as likely to differ after landing, by name and why, so that after-landing's repair starts there.
