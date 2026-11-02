# reference-m4: the shapes that cross tasks

Binding for every task that names this note. Design: `smith/docs/design/protocol/programs.md`,
5.10; `smith/docs/design/domain/run.md`, 5.3.

## C1. A child's conversation ends: `Close`

- `smith_domain::Lower` gains, after `Yield`:

  ```rust
  /// The child conversation `owner` has ended and asks nothing more: a backend
  /// that keeps a process for it ends it (protocol/programs.md, 5.10). Sent once,
  /// after the terminal of the conversation's last `Complete`; nothing answers it.
  Close { owner: Token },
  ```

  `owner` is the token that conversation's `Lower::Complete`s carried as `owner`.
- smith-domain sends it exactly once for each child conversation whose session ran on a program
  backend (its `Spec.backend` was `session::record::Backend::Program { .. }`, set in
  `smith-domain/src/translate.rs`), when that session ends (`session::Request::Ended`), whatever
  its end. Never for main, a compaction, or a direct session.
- `smith_protocol_llm::FromDomain` gains, after `Yield`:

  ```rust
  /// The owner's child conversation ended (smith_domain::Lower::Close). A backend
  /// that keeps a process for it ends it; no terminal is owed.
  Close { owner: Token },
  ```

  The direct component (`smith-protocol-llm/src/component.rs`) ignores it, as it ignores
  `Yield`. `smith-agent-service/src/service/request.rs` maps `Lower::Close` to it, and
  `service/program.rs` `complete` routes it: to the program backend that keeps `owner`'s
  conversation (from task `program-children` on), and nowhere before that.

## C2. The program's `sub_agent` tool

- **Listed** by the agent's MCP server (`smith-protocol-program/src/backend/files.rs`, `tools()`)
  exactly when the conversation's `prompt.served` contains `llm::Served::SubAgent`, with:
  - `name`: `sub_agent`;
  - `description`: the direct backend's (`smith-protocol-llm/src/tools.rs`, `sub_agent`), then
    ` On this backend a sub-agent reads only: its tools are ["inspect"]. Several asked in one message run side by side.`
  - `inputSchema`: the direct backend's schema with its `tools` items' enum
    `["inspect","modify","shell"]` replaced by `["inspect"]`, nothing else changed (an object
    schema with no top-level combinator, as 5.3 requires).
- **Decoded** (`smith-protocol-program/src/backend/conversation.rs`, `decode`) by
  `protocol::decode` as on the direct backend, giving `run::Ask::SubAgent { .. }`; an ask whose
  families include modify or shell is instead
  `llm::Decoded::Invalid { problem: llm::Problem::BadValue { field: b"tools" } }`, so no child
  opens and the model reads `invalid field value tools`.
- `system/init` lists it as `mcp__smith__sub_agent`; `checked_init` already compares the server's
  names with `tools()`.

## C3. Children on the program, in the service

- `smith-agent-service/src/service/program.rs` keeps main's `Backend` and, beside it, one
  `smith_protocol_program::Backend` per live child conversation on the program, keyed by the
  child's `owner`. A child's `Complete` goes to the program when `prompt.asker` is `Child` and
  main's backend `holds` its grant; it creates that child's backend on its first `Complete`.
  `Cancel`, `Yield` and `Close` go to the backend that keeps `owner`.
- A child's backend is `Backend::new(first, settings, &entropy)` set up as main's is (directory,
  workspace, mounts, the same grant, turn limit and run time), with a token range and an entropy
  of its own, so its private run directory, MCP secret and session differ from main's.
- On `Close`, the child's backend is closed (`Backend::close`) and dropped once `closed()`.

## C4. The fake's children

- A child's script in a fake `claude` story is a `cued` script whose cue is a word that only that
  child's brief holds (the brief is the child's `--append-system-prompt-file`), so main and each
  child pick their own script from the one story file (`smith/tests/fake-claude/src/print.rs`).
- Main asks children with `Step::Call { tool: "sub_agent", input }`, several in one message with
  `Step::Together`, each input `{"brief": ..., "tools": ["inspect"], ...}`.
- Each fake process writes its own `Observation { arguments, .. }`; a test tells a child's from
  main's by its `--append-system-prompt-file` content or its `--model`.
