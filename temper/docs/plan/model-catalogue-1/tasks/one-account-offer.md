+++
id = "one-account-offer"
part = "one-account-offer"
kind = "feature"
needs = ["stock-seven", "offer-engine"]
files = [
  "temper/crates/temper-service/src/tests/first_model.rs",
]
effort = "high"
minutes = 70
+++

# On a box as `temper init` writes it, `me` offers what its account can run

## Goal

The meeting point of the stock and the offer: the real settings, the real core, one kind of account at a time. The
user's box is Claude-only: it must show no ChatGPT model anywhere, and a default that runs.

Acceptance, each a story in `temper-service/src/tests/first_model.rs` on its `stock_box`:

1. Claude alone: each project's `models` holds Opus, Sonnet and Fable with that account and their default efforts, and
   no OpenAI model. Previews: template 1 and template 3 Opus at `high`. A chat with no model named runs on Opus at `high`.
2. A ChatGPT plan alone: Sol, Astra, Terra and Luna. Previews: template 1 Sol at `high`, template 3 Astra at `high`. A chat
   and a configuring chat run on those, each admitted within its charter's spend.
3. Both: Opus on both previews, as the user named it first; every model offered.
4. None: `models` empty, no previews; a chat with no model named waits, its document's `llm` none and its `run_wait`
   `Account { None, None, Empty }`. A chat named `gpt-6.1-sol` waits and still names it.
5. A login that lapses: its models leave the offer, `me` is marked, and a task keeping that account still names its model.
6. `me` at init's limits, with seven models and three previews on each project: within its client bound through the
   wire's translation.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`.
- `jig/docs/design/client/domain/contract.md`, 5.1; `temper/docs/design/protocol/configuration.md`, 13.4.

## Owns and may change

- Owns `first_model.rs`.
- May change, each only for this: process walks under `temper/crates/temper/tests/fake_backends/walk/` and
  `temper/crates/temper/tests/support/` that a run below shows reaching `chat start --model` before the offer holds the
  model. Keep it to one helper polling `model list`. `chat_document.rs` and `note_parts.rs` name `demo` straight after
  `playground start`, the likeliest.
- A real gap the stories find in `Core::offer` or the stock is a repair of that crate: fix it, and report it.

## Steps

1. The stories, broken once each to see them bite.
2. Narrow tests, then the implementer's text's closing checks over `temper-service`.
3. The three process tests of `temper/docs/plan/model-catalogue-1/notes/stock-readers.md`, each alone, and the same six
   walks as `stock-seven`, again now the offer is real. Commit and push.

## Report

As the implementer's text says; each process test and walk with its result.
