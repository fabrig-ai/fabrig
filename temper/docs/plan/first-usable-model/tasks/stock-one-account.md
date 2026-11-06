+++
id = "stock-one-account"
part = "stock-one-account"
kind = "feature"
needs = ["first-usable-choice"]
files = [
  "temper/crates/temper-service/src/tests/first_model.rs",
  "temper/crates/temper-service/src/tests.rs",
]
effort = "high"
minutes = 80
+++

# A box as `temper init` writes it, with one account of each kind, does its work

## Goal

A fixture's model order hid this fault. Hold the rule on the stock settings: the `engine.llm` `temper init` writes,
in its order, and `stock::defaults`, never a kit that moves a model first.

Acceptance, each a story in `temper/crates/temper-service/src/tests/first_model.rs`, with no real host:

- **Claude alone,** a program login usable on the worker: a chat (charter 1) and a goal's coding task (charter 2)
  each reach a claim whose assignment's main is `claude-opus-5-5`, and the Task document's `llm.main` says so
  before the claim.
- **ChatGPT alone,** a source of the OpenAI provider `temper init` writes: the same two, on `gpt-6.1-sol`.
- **A named `llm` with no account for it:** a chat naming `gpt-6.1-sol` on the Claude-alone box waits, its document's
  `run_wait` `Account { account: None, source: None, why: Empty }`, and its `llm.main` the name it was given.
- **ChatGPT added later:** a chat that ran on Claude keeps its account, then an OpenAI account is added: its next
  run's main is still `claude-opus-5-5`, on the same account.
- **No usable account:** a declared login not signed in: `Account { account: Some(_), source: Some(_), why: Host }`.
  No account at all: `why: Empty`, `llm.main` the charter's first.
- A guard: the stories read the generated document's model list, and fail by name if `gpt-6.1-sol` isn't its first.

## Inputs

- `temper/docs/plan/first-usable-model/notes/common.md`, `temper/docs/plan/first-usable-model/notes/contracts.md`.
- `jig/docs/design/domain/accounts.md`, 4.3.
- `temper/docs/design/domain/testing.md`, for the tier.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-service/src/tests/fixtures.rs` and `fixtures/*`: a new helper only, leaving existing callers.
- Any test in `temper/` whose expected model moved because of `first-usable-choice`, reported under `## Left` by that task:
  the expected value alone, with the reason in your report. A story whose fixture needs its old model picks it by
  naming `llm`.

Options, each pre-approved:

- **Home:** `temper-service/src/tests` has `temper-init` and `temper-engine-domain-world` in its `Cargo.toml`. If a story
  can't reach a claim there, the next home is `temper/tests/engine/domain` (`direct::Driver`, `protocol::{hello_at, up}`),
  with its `stock_config` swapped for init's document. Say which, and why.
- **The ChatGPT account:** a key on that provider if the world can't sign in; `MoveIn` if a sign-in is what you need
  (`temper/tests/engine/domain/tests/suite/accounts_restart.rs`). `sources.keys` is `true` in init's document.
- **Cost:** over its CPU budget, the stories share a world per kind of account rather than one per story.

## Steps

1. Reuse `tests/stock.rs`' `generated()` and `engine()`, and `fixtures.rs`' `claimed_chat` path (`Link::Hello`, then `LinkUp::Logins`).
   A program login needs all of: `DeclareProgram` so `accounts::programs::issued` finds its name; `LinkUp::Logins` with
   `LoginState::Usable`; the hello's `bounds.models` naming the model and `bounds.program_endpoints` its endpoint. That
   fixture's `logins` and its `small` model bound need replacing for init's document.
2. A helper for the box: `fn stock_box(account: OneAccount) -> ...`, with `enum OneAccount { Claude, ChatGpt, None, Declared }`.
3. The stories. They need no fix beyond `first-usable-choice`, so see each fail by breaking the rule once, in `next_llm`:
   take the charter's first always, and read the failure.

Tests, through `heavy`: `-p temper-service --lib -E 'test(first_model)'`, then the package. Then fmt, `old_forms.py` for temper, clippy, commit and push.

## Report

As the implementer's text says; CPU seconds of the new stories; the rule you broke and what failed.

## Answers

2026-11-06 (fabrig/temper). A correction from the user's recordings: both hung chats were started from Configure temper, template 3, and the chat that answered named claude-opus-5-5 by the chooser. So no chat with no model named has answered on that box. Cover both, in a Claude-only box on a fresh init's order, with no model named: template 1 and template 3. Say if they differ.
