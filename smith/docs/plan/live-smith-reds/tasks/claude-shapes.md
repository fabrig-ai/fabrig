+++
id = "claude-shapes"
part = "claude-shapes"
kind = "feature"
needs = []
files = ["smith/tests/fake-claude/src/stream.rs", "smith/tests/fake-claude/tests/suite/**", "smith/tests/agent-process/tests/live/program.rs", "smith/crates/smith-protocol-program/src/backend/stream_tests.rs", "smith/crates/smith-protocol-program/src/backend/turns_tests.rs"]
effort = "high"
minutes = 60
+++
# The fake `claude` and the live probes follow Claude Code 2.1.312

## Goal

Claude Code 2.1.312, the release on the after-landing machine, sends two shapes that smith's fake
`claude` does not send and two live probes refuse. Make the fake send them, and bring the two live
probes' expectations to them, so both probes pass after landing:

- `smith-agent-process-world::live program::a_live_server_with_a_non_object_schema_lists_no_tools`
  failed at `smith/tests/agent-process/tests/live/program.rs:388`: init's `mcp_servers` was
  `[{"name":"smith","source":"dynamic","status":"connected"}]`, the probe expects
  `[{"name":"smith","status":"connected"}]`.
- `smith-agent-process-world::live program::a_live_thinking_turn_has_the_fakes_shape` failed at
  `program.rs:478`, `assert!(estimated_tokens.is_number())` on a `thinking_delta`.

What 2.1.312 sends, read from its shipped code (the lead read it; you never run `claude`):

- **init:** each server entry is `{name, status}` plus `source` when the server has one; a server
  given by `--mcp-config` has `"source":"dynamic"`. `claude_code_version` is `"2.1.312"`.
- **thinking:** consecutive `content_block_delta` frames of one block may be coalesced into one
  `{"type":"thinking_delta","thinking":<the deltas' text joined>,"estimated_tokens":<number or
  null>}`; `estimated_tokens` is `null` when none of the joined deltas carried one. A single,
  uncoalesced delta keeps the API's own shape (a number, as the fake sends today). The
  `system/thinking_tokens` record is unchanged: `estimated_tokens` and `estimated_tokens_delta`
  numbers, sent only when a delta carried a number or non-empty thinking text.

smith's production decoding already reads init's servers by field name and ignores thinking
deltas (`smith/crates/smith-protocol-program/src/backend/stream.rs` `checked_init`,
`measures.rs`); this task changes no production code. If you find that production code refuses
either shape, stop at that point: commit the failing test, push, and ask.

Acceptance:

- The fake's init names the configured server `{"name":"smith","status":...,"source":"dynamic"}`
  and reports `claude_code_version` `"2.1.312"`; its thinking turn sends one coalesced
  `thinking_delta` whose `estimated_tokens` is `null`, besides the number-bearing one it sends
  today.
- fake-claude's suite asserts both, red before the fake changes and green after.
- smith-protocol-program's unit tests show the decoder takes both 2.1.312 shapes.
- The two live probes expect the 2.1.312 shapes, and say the JSON type they saw when they fail.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds/notes/common.md` (read it first).
- `smith/docs/design/protocol/programs.md`, 5.5 (the stream, told as smith's) and 8 (the fake
  and the live suite: "the fake sends what a real run was recorded sending").

## Owns and may change

- `smith/tests/fake-claude/src/stream.rs`: `init` and the thinking block's deltas.
- `smith/tests/fake-claude/tests/suite/**`: every assertion of init's `mcp_servers` (today in
  `mcp.rs`, `relay.rs`, `turns.rs`, `binary.rs`) and the thinking sequence in `stream.rs`.
- `smith/tests/agent-process/tests/live/program.rs`: the two probes named above, and nothing else
  of its behaviour.
- `smith/crates/smith-protocol-program/src/backend/stream_tests.rs` and `turns_tests.rs`: new
  tests, and the `init_line` fixture's `claude_code_version`.
- Pre-approved:
  - If a fake-claude story with `Init.extra_servers` (`smith/tests/fake-claude/src/story.rs`)
    needs a `source`, leave extra servers without one (2.1.312 omits it when a server has none).
  - If another test in `smith-agent-process-world`, `smith-hosts-world` or `smith`'s
    `end_to_end` binary counts the fake's stream records and goes red from the added delta,
    update that count in that test only and list it in the report; if its meaning would change,
    ask.
  - If adding the coalesced delta to every thinking turn breaks more than such counts, send it
    only on a story option (a `Step::Think` field or a `Story` flag, default off) and turn it on
    in the sequence test and in a test of the agent-process world; say which you did.

## Steps

1. Read the inputs; run fake-claude's `stream.rs` and `mcp.rs` tests to see them green.
2. Change the suite's assertions first and run them red:
   - `mcp.rs` `init_reports_the_configured_server_connected_with_its_tools` and
     `one_non_object_schema_hides_every_server_tool_at_init` expect
     `[{"name":"smith","status":"connected","source":"dynamic"}]`; `relay.rs` likewise (with
     `"failed"` where it expects failed); `turns.rs` where it names the configured server;
   - `stream.rs` `a_thinking_call_uses_the_recorded_block_sequence` lists the coalesced delta as
     one more `("stream_event", "content_block_delta")` row inside the thinking block, before the
     signature delta, with no `system/thinking_tokens` row for it, and asserts that delta's
     `delta.estimated_tokens` is `null` and the earlier one's is `THINKING_TOKENS / 2`;
   - `init_has_the_recorded_keys_and_thirteen_capabilities` keeps its key list (`source` is a key
     of a server entry, not of init) and asserts `claude_code_version` is `"2.1.312"`.
   `heavy cargo nextest run -p smith-fake-claude --test suite -E 'test(/init_|thinking_call|relay|non_object/)'`
   shows them red; note the failures for the report.
3. Change `smith/tests/fake-claude/src/stream.rs`: the configured server entry gains
   `"source":"dynamic"`; `claude_code_version` is `"2.1.312"`; the thinking block's deltas gain
   `{"type":"thinking_delta","thinking":"","estimated_tokens":null}` after the number-bearing
   delta and before the signature delta. Rerun step 2's tests green.
4. smith-protocol-program, `--lib`:
   - `stream_tests.rs`: `a_2_1_312_init_names_its_server_source`: `init_line` with the server entry
     `{"name":"smith","status":"connected","source":"dynamic"}` passes `checked_init`; the same
     line with a second server still fails. Set `init_line`'s `claude_code_version` to
     `"2.1.312"`.
   - `turns_tests.rs`: `a_coalesced_thinking_delta_without_an_estimate_is_ignored`: model on
     `thinking_and_a_call_are_one_completion_with_final_usage` and its `thinking_turn` helper, with
     a `thinking_delta` carrying `"thinking":"private reasoning","estimated_tokens":null`; the turn
     is still one completion with the final usage.
   These describe what the decoder already does; if either is red, stop and ask (see Goal).
5. `smith/tests/agent-process/tests/live/program.rs`:
   - `a_live_server_with_a_non_object_schema_lists_no_tools`: expect
     `json!([{"name":"smith","status":"connected","source":"dynamic"}])`;
   - `a_live_thinking_turn_has_the_fakes_shape`: a `thinking_delta`'s `estimated_tokens`, when
     present, is a number or `null`; the `system/thinking_tokens` record's stays a number. Each
     such assertion's message names the JSON type it saw (`null`, `string`, ...), never the
     thinking text.
   Check it with `heavy cargo check -p smith-agent-process-world --tests`; never run it.
6. `cargo fmt`; `heavy cargo clippy -p smith-fake-claude -p smith-protocol-program -p smith-agent-process-world --all-targets -- -D warnings`.
7. The fake's readers' tests, one command at a time:
   `heavy cargo nextest run -p smith-fake-claude`, `heavy cargo nextest run -p smith-protocol-program`,
   `heavy cargo nextest run -p smith-agent-process-world`, `heavy cargo nextest run -p smith-hosts-world`,
   `heavy cargo nextest run -p smith --test end_to_end`.
   temper and jig link the fake but assert neither shape; their trunk check covers them.
8. Commit (the tests and the fake can be one commit or two), push.

## Report

The red runs of step 2 (test, assertion), the files changed, every count or option from
"Pre-approved" you used, the five test runs (selected, passed), and implementation and check
minutes apart.
