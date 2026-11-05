+++
id = "chats-base"
part = "chats-base"
kind = "migration"
needs = ["chat-state"]
files = []
effort = "medium"
minutes = 60
+++

# What ui-chats has by now

## Goal

Every task that draws builds on `ui-chats`' byline, conversation and tool lines. This task merges `origin/ui-chats/integration` into this plan's trunk, so that they do, as far as they've merged by now.

Acceptance:

- Your branch holds `origin/ui-chats/integration`'s tip at the time of your run, by one merge commit, `git merge --no-ff`, whose second parent it is. If `origin/main` already holds `ui-chats`' landing, say so and merge `origin/main` instead.
- Conflicts are resolved keeping both sides' meaning: this plan's jig, skein and temper engine work stays, and `ui-chats`' client stays as it is there. Where a generated file conflicts, regenerate it (`notes/common.md`).
- `cargo fmt`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery`; `heavy cargo check --tests -p temper-client-world`.
- **Your report's first section says which of `chat-byline`, `conversation-turns`, `tool-lines`, `chat-stories`, `playground-recording` and `proof` your base now holds.** (`git log origin/ui-chats/integration --oneline | rg ...`, or `fab status` isn't yours: read their merge commits.) A later task reads that.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`.
- `temper/docs/plan/ui-chat-draft/plan.md`, *How it sits on ui-chats*.
- `temper/docs/plan/ui-chats/plan.md`.

## Owns and may change

Owns nothing, and may change whatever the merge names, to settle it.

Options, each pre-approved:

- **`ui-chats`' tasks not merged yet:** that's no failure, and you don't wait: merge what's there, and name what isn't. Each later task of this plan has its own option for what it finds.
- **A test of `ui-chats`' already red on `origin/ui-chats/integration`:** `## Left`, with its log, not yours.
- **Budgets:** the merge adds no tests of its own.

## Steps

1. `git fetch origin`, the merge.
2. The conflicts; the checks above; push.

## Report

As `notes/common.md`, Your report, with the list above.
