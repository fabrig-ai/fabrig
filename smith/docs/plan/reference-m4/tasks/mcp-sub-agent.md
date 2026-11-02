+++
id = "mcp-sub-agent"
part = "mcp-sub-agent"
kind = "feature"
needs = []
files = ["smith/crates/smith-protocol-program/src/backend/files.rs", "smith/crates/smith-protocol-program/src/backend/conversation.rs", "smith/crates/smith-protocol-program/src/mcp/json.rs", "smith/crates/smith-protocol-program/src/backend/conversation_tests.rs", "smith/crates/smith-protocol-program/src/backend/builtin_tests.rs"]
effort = "high"
minutes = 60
+++
# The program's MCP server serves `sub_agent`, inspect only

## Goal

On a program backend, smith's `sub_agent` reaches the program through the agent's own MCP
server. Make `smith-protocol-program` list `sub_agent` when the conversation's prompt serves it,
with its `tools` narrowed to `inspect`, and decode its calls as the direct backend does, a call
asking for modify or shell answered invalid naming `tools`: contract C2 of
`smith/docs/plan/reference-m4/notes/contracts.md`. Nothing offers it to a program's prompt yet
(smith-domain withholds it until task `program-children`), so no world changes.

Acceptance:

- With `llm::Served::SubAgent` in the prompt, `Backend::tools()` holds `sub_agent` with C2's
  description and schema (enum `["inspect"]`), and the MCP `tools/list` answer carries it; without
  it, the listing is as today.
- A program's `mcp__smith__sub_agent` call with `"tools":["inspect"]` decodes to
  `run::Ask::SubAgent` with inspect alone; one with `"modify"` or `"shell"` decodes to
  `Invalid { problem: BadValue { field: b"tools" } }`; one missing `brief` to the direct
  decoder's `Missing`.
- A `system/init` that lists `mcp__smith__sub_agent` is admitted when the prompt serves it.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first) and
  `smith/docs/plan/reference-m4/notes/contracts.md`, C2.
- `smith/docs/design/protocol/programs.md`, 5.3 (the agent's server; object schemas, no top-level
  combinator) and 5.10 ("Served by the agent's own server", "Inspect only").
- `smith/docs/design/domain/run.md`, 5.3 (the `sub_agent` call).

## Owns and may change

- `smith/crates/smith-protocol-program/src/backend/files.rs`: `tools()` lists
  `protocol::ToolKind::SubAgent` as C2 says, instead of dropping it with the owned tools.
- `smith/crates/smith-protocol-program/src/mcp/json.rs`: a helper writing the narrowed schema
  and description, if you build them there (as `json::outcome_schema` is for `finish`).
- `smith/crates/smith-protocol-program/src/backend/conversation.rs`: `decode` drops its early
  `UnknownTool` for `sub_agent` and narrows the decoded ask as C2 says.
- Tests: `backend/conversation_tests.rs` (`the_unoffered_sub_agent_tool_is_not_executed`, at
  about line 70, now offers it, so it changes: see Steps) and `backend/builtin_tests.rs` (new
  listing and init tests, beside the ones that call `backend.tools()`).
- Neighbour reads only: `smith-protocol-llm/src/tools.rs` (`sub_agent`, the direct schema) and
  `decode.rs` (how `protocol::decode` decodes `sub_agent` and refuses nesting). Do not change them.
- Pre-approved:
  - If the narrowed schema is simpler built whole in the program crate than by replacing the enum
    in the direct schema's bytes, build it whole, keeping every other property and bound of the
    direct schema byte for byte, and add a test that the two differ only in the enum.
  - If `stream::checked_init` needs a change to admit the listed tool, make it in
    `backend/stream.rs` and say so.

## Steps

1. Read the inputs, `files.rs` `tools()`, `conversation.rs` `decode` (about line 515) and the
   direct `sub_agent` schema.
2. Tests first, run red:
   - `builtin_tests.rs`: `a_served_sub_agent_is_listed_inspect_only`: an active backend whose
     prompt serves `SubAgent`; `backend.tools()` holds one `sub_agent` whose description ends
     with C2's sentence and whose schema parses as an object whose
     `properties.tools.items.enum` is `["inspect"]`, `required` `["brief","tools"]`, and no
     top-level `oneOf`/`allOf`/`anyOf`. `an_unserved_sub_agent_is_not_listed`: the same without
     `SubAgent` lists none.
   - `builtin_tests.rs`: `an_init_listing_a_served_sub_agent_is_admitted`: modelled on the later-init
     tests there, an init whose MCP tools include `mcp__smith__sub_agent` passes when the prompt
     serves it, and fails the start when it does not.
   - `conversation_tests.rs`: rewrite `the_unoffered_sub_agent_tool_is_not_executed` as
     `a_served_sub_agent_call_decodes_inspect_only`: with `SubAgent` served, a `tool_use` of
     `mcp__smith__sub_agent` with `{"brief":"Read lib.rs","tools":["inspect"]}` is
     `Decoded::Served { ask: run::Ask::SubAgent { .. } }` with inspect alone; add
     `a_sub_agent_asking_to_write_is_invalid_naming_tools` (`["inspect","modify"]`, and
     `["shell"]`) and `a_sub_agent_call_without_a_brief_is_missing_it`.
   `heavy cargo nextest run -p smith-protocol-program --lib -E 'test(/sub_agent/)'`.
3. Make them green in `files.rs`, `conversation.rs` (and `json.rs` if you build there).
4. `cargo fmt`; `heavy cargo clippy -p smith-protocol-program --all-targets -- -D warnings`.
5. Focused tests: `heavy cargo nextest run -p smith-protocol-program`; then, since no program
   prompt serves `sub_agent` yet, the readers must be unchanged:
   `heavy cargo nextest run -p smith-agent-process-world`.
6. Commit, push.

## Report

The red runs of step 2, the schema and description as served (paste the schema), which
pre-approved option you took, the test runs (selected, passed), and implementation and check
minutes apart.
