+++
id = "ask-remembering"
part = "remember"
kind = "feature"
needs = ["remember-surface"]
files = ["temper/crates/temper-service/src/stock/ask.txt", "temper/crates/temper-service/src/tests/stock.rs"]
effort = "medium"
minutes = 45
+++

## Goal

Put the *Remembering* part of Ask temper's instructions into `ask.txt`,
word for word as `temper/docs/design/draft/ask-temper.md`, section 8 gives
it, between *Showing* and *Your voice*, and hold the instructions to that
draft's 10 KB (`temper/docs/design/domain/agent.md`, 8.4, *Its
instructions*). The user reviews these words before landing: change none.

Acceptance:

- `temper/crates/temper-service/src/stock/ask.txt` holds, after the
  *Showing* part and before `## Your voice`, the draft's `## Remembering`
  heading, an empty line, its one paragraph exactly as the draft's
  section 8 has it (its `…` the Unicode ellipsis, its quotes straight), and
  an empty line; nothing else of the file changes.
- `ask_temper_is_charter_11_and_template_5` (`src/tests/stock.rs`): its
  heading list gains `## Remembering` before `## Your voice`; the assertion
  that the file has no Remembering part becomes one that its *Remembering*
  part, split from `## Remembering\n\n` to `## Your voice`, equals the
  draft's paragraph and its newline; *Showing*'s split ends at
  `## Remembering`; its bound becomes `words.len() <= 10_240`, its message
  "Ask temper's instructions fit ask-temper.md, 4.1's 10 KB".
- `asks_first_request_fits_both_shapes` (`src/tests/ask_bound.rs`) passes
  within 16,384 bytes with `remember` declared and these words.
- The engine domain world's whole-file check of the instructions
  (`temper/tests/engine/domain/tests/suite/configuring.rs:633`,
  `an_owners_asking_chat_proposes_and_hears_it_accepted`) and every other
  test that pins `ask.txt`'s words (this cut's survey:
  `temper-engine-smith/src/tests/see.rs:434`,
  `temper/tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs:112`
  and `:203`, `stock.rs:1815`) pass unchanged.
- Delete the **Not yet** entries `8.4: ask.txt has no *Remembering* part`
  in `temper/docs/design/domain/agent.status.md` and `13.4: charter 11's
  ask.txt has no *Remembering* part` in
  `temper/docs/design/protocol/configuration.status.md`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`.
- `temper/docs/design/draft/ask-temper.md`, section 8, its `## Remembering`
  paragraph, and 4.1's table (instructions, 10 KB);
  `temper/docs/design/domain/agent.md`, 8.4, *Its instructions*.
- `temper/crates/temper-service/src/tests/stock.rs`, lines 333 to 400.

## Owns and may change

- **Owns:** the two files in this spec's header.
- **Options, pre-approved:** none on the words. If the first request
  passes 16,384 bytes, report `Question:` with the parts the test prints
  on both shapes, the words in place and pushed.

## Steps

1. The paragraph; the stock test's rows and bound.
2. Break the words once (drop its last sentence) to see the row fail, and
   restore.
3. `heavy cargo nextest run -p temper-service --lib -E
   'test(/stock|ask_bound/)'`; `heavy cargo nextest run -p
   temper-engine-domain-world -E 'test(/asking|goal_delivery/)'`;
   `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/see/)'`;
   `heavy cargo clippy -p temper-service --all-targets -- -D warnings`.
   Commit and push.

## Report

As the implementer's text asks; give `ask.txt`'s size and the first
request's parts on both shapes.
