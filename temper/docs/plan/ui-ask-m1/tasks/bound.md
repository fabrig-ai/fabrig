+++
id = "bound"
part = "stock"
kind = "feature"
needs = ["stock", "mechanics"]
files = ["temper/crates/temper-service/src/tests/ask_bound.rs", "temper/crates/temper-service/src/tests.rs", "temper/crates/temper-service/Cargo.toml", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 90
+++

# ui-ask-m1: what temper gives Ask temper's first request, held to 16 KB

## Goal

A step test in `temper-service` holds what temper gives a first request of charter 11,
as the stock has it, to at most 16,384 bytes on both shapes: its instructions, smith's
mechanics with no workspace, its declarations and the brief at a first message, with
no preference kept. Claude Code's own part is not temper's and is not measured.

Acceptance:

- `temper-service/src/tests/ask_bound.rs` (a `mod` line in `src/tests.rs`), one table
  test of two rows, `chatgpt` and `claude_code`, each measuring, for a run of stock
  charter 11 at a chat's first message (the brief the engine gathers then, its task
  section holding the person's first words, a line of about 100 bytes):
  - `chatgpt`: what smith sends a direct backend as the run's system text and tool
    declarations, rendered as smith renders them for that provider (the request's
    instructions and its `tools` array, without the conversation's message);
  - `claude_code`: the instructions file smith writes for Claude Code
    (`--system-prompt-file`'s content) and the MCP server's `tools/list` result, as
    `smith-protocol-program` renders them;
  and asserting each at most 16,384 bytes, the bound named as a constant with a doc
  line citing `temper/docs/design/domain/agent.md`, 8.4. Use the helpers
  `tests/stock_brief.rs` uses to start smith's domain for a stock charter (it admits a
  run and checks its host part), and smith's own renderers where they are public; a
  renderer that is not public is measured through the nearest public measure smith
  offers (`run::brief_room` and the host part's parts), and the report says which.
- It bites: with the declarations of every host tool in place of the asking three (as
  a charter 1,023's offer), the row fails for the bound; report the figure seen.
- The report gives both rows' bytes and each part's (instructions, mechanics,
  declarations, brief).
- Status (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built): delete from `temper/docs/design/domain/agent.status.md`
  the **Not yet** "8.4: no step test holds what temper gives an asking charter's first
  request to 16,384 bytes on both shapes ..." bullet, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 2, 4 and 5.
- `temper/docs/design/domain/agent.md`, 8.4 ("What temper gives its first request");
  4.3 ("The host part's budget"); `temper/docs/design/draft/ask-temper.md`, 4.1 (the
  parts and their bounds).
- `smith/docs/design/protocol/programs.md`, 5.1 and 5.3 (what Claude Code is given);
  `smith/docs/design/domain/run.md`, 3.3 (the system text, no workspace).

## Owns and may change

- Owns the files in the header.
- `temper-service/Cargo.toml`: a dev-dependency line on a smith crate whose renderer the
  test needs (`smith-protocol-program`, `smith-protocol-llm`), pre-approved.
- Pre-approved: if a smith renderer the test needs is private, do not open it; measure
  as the acceptance says and say so. If a row passes 16,384 bytes as built, do not
  raise the bound or trim `ask.txt`: report `Question:` with the parts' figures.

## Steps

1. The table; run it; with every host tool declared, see it fail for the bound.
2. `heavy cargo nextest run -p temper-service --lib -E 'test(/ask_bound/)'`; `cargo fmt`;
   `heavy cargo clippy -p temper-service --all-targets -- -D warnings`.
3. Commit and push.

## Report

As the implementer's text says, with the figures above.
