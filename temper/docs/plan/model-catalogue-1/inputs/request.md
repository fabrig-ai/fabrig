# model-catalogue-1: the stock models, their efforts, and a resolved default

Section 7's step 1 of `temper/docs/design/draft/model-catalogue.md`, alone: read the draft whole, build this step, and leave steps 2 and 3. The user took it, 2026-11-06, and reviews on waking. Where a choice is left open, take the draft's and note it in `plan.md` for that review. It needs no store change.

- **Seven stock models with efforts and a default effort.** The orders are Opus 5.5 then Sol 6.1, and for the configuring charter Opus 5.5, Astra 6, Sol 6.1. Terra's name is `gpt-6.1-terra`: OpenAI lists no `gpt-6-terra`.
- **`resolved_default` in `me`,** which `temper/docs/design/client/README.md`, 3.5 describes and no crate names yet. A byline and the picker read it, so a view doesn't guess. `first-usable-model` may have settled part of this: read how it landed, and build on it.
- **The picker's facts.** The look request `model-picker` is landed separately by fabrig/ui: this plan gives it what it needs, and takes its pages by catch-up. Models grouped by what pays, an effort row, on every chat, Configure temper included; a Settings Models tab.

## The user's additions, 2026-11-06

On a journey: "it's not possible to select the model for the configuring agent", and "we should make sure that we ever surface a model choice if it was actually fetched from the provider list (plus the stock ones we already know that exist), and if a provider/account is not available (e.g. chatgpt in my case) we should not surface any of its models, including the ones we know exist." The draft's section 3 is now that rule. For this step:

- **One offer in `me`, per project:** the models the person can pick now, each with its account, efforts and default effort, whether the picker shows it, and `resolved_default`, one of them. The clients draw from it and work nothing out. `choice::models` and "needs an account" beside a provider's models go.
- **A provider with no usable account** shows its name and "Connect a plan or key ›", and no model, stock ones included. An account that stops being usable keeps a line with its fix, and no model.
- **The configuring chat gets the picker** (the look `model-picker`, its page 05). `temper/docs/design/client/README.md`, 3.5 says today that it has no chooser: change it.
- **The configuring agent is told the offer alone,** so it can't propose a model no account can run. Its "Models by role" starter names only connected accounts, and goes with a single account.
- **A task given no choice with no account** says "No account can run this chat yet." rather than naming the charter's first, on top of `first-usable-model`'s wait's words. A task given a model still names it.

Reading a provider's list is still step 3, and its per-account narrowing is in section 3, ready for it. The look request `model-picker` holds these states in its message; `model-check-pane`, above it, isn't to land yet.

## What the plan must hold

- **The default model changes:** the stock order moves again, away from `gpt-6.1-sol` first. After-landing went red the last time init's order moved, in tests a fixture hid: `temper/crates/temper/tests/support/first_repository.rs`, `run_spend`; `forge_land.rs`, `cheapest_model_first`; `temper-init`'s tests; `stock_brief.rs`'s pinned sizes. Before readying, run `process-temper`'s two named in those and read every live-tier site by hand, as the queue's gate doesn't run them. Name each in the report.
- **Declaring efforts changes the start's largest-turn check** (`temper/docs/design/protocol/configuration.md`, 13.8), so measure it again and say what it comes to.
- **A Claude-only and a ChatGPT-only box** each still run with no model named, as `first-usable-model`'s stories have it, under the new order.
- **fabrig/qa's comparison** names `gpt-6.1-sol` and Opus by charter settings: no stock change should reach a stored charter, and say so if one does.

## Size

As it needs. fabrig/ui has priority on `temper-client-*`: keep hunks small and name each overlap. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). Formats change in place before v0.1.0. `walk = false`: the user reads it on a real box.
