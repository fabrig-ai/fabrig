+++
id = "models-help"
part = "help"
kind = "docs"
needs = ["stock-models", "models-add"]
files = ["temper/docs/help/account.md", "temper/docs/help/deployment.md", "temper/docs/help/box.md", "temper/docs/help/playground.md", "temper/docs/help/charter.md", "temper/docs/help/reference/ctl-model.md", "temper/crates/temper/src/tests/help.rs"]
effort = "medium"
minutes = 45
+++

# The help says how to offer another model

## Goal

"Offering another model" stops editing two files. This task departs from the implementer's text: `docs/help/` is yours here. It's Markdown read by tests.

Acceptance:

- `temper help deployment`, *Offering another model*: on Settings, Models, "Add a model by name", a provider and a name; prices are estimated until set; its first run shows whether the provider serves it; to set its prices, window or efforts, tell Configure temper; a task keeps the model it has; no restart. A local model's provider: its `endpoint` in `engine.llm.providers`. A larger output than an agent's endpoint's limits: raise them in `CONFIG/agent.json` and restart, then change the model.
- `temper help account`: "The providers are the configuration's and the models are settings", pointing to it.
- `box.md`, `playground.md` and `charter.md` lose whatever says the models are the document's; `box.md`'s "init writes Anthropic's provider, endpoint and model" becomes its provider and endpoint.
- `account_and_deployment_help_explain_offering_another_model`, in `temper/crates/temper/src/tests/help.rs`, pins the new text: the heading, the page's name, "Add a model by name", and no JSON example of a model; it keeps the rest of its checks.
- Each page keeps its byte bound; a page past it is shortened, losing no fact, and the bound isn't raised.
- `temper/docs/help/reference/*` regenerated if stale. `temper-docs`'s and `temper`'s help tests pass.

## Inputs

- `temper/docs/design/draft/model-catalogue.md`, 4 and 6.
- `temper/docs/design/client/README.md`, 3.5, *The `Models` tab*.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. `temper-docs`' bounds aren't yours.

Options, pre-approved: a page's sentence on models outside those listed, found by `rg -n 'agent.json|engine.llm|models' temper/docs/help --glob '!reference/*'`, is brought in line by a one-line edit and named in the report.

## Steps

`heavy cargo nextest run -p temper-docs`, then `heavy cargo nextest run -p temper --lib -E 'test(=account_and_deployment_help_explain_offering_another_model)'` and the rest of its help tests. Break the pin once by hand. `cargo fmt`. Commit and push.

## Report

Each page's size against its bound.
