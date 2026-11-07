+++
id = "large-argument"
part = "argument"
kind = "feature"
needs = []
files = ["smith/tests/agent-process/**", "smith/tests/fake-claude/src/**", "smith/crates/smith-domain-run/src/host.rs", "smith/crates/smith-protocol-program/src/backend/conversation.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs"]
effort = "high"
minutes = 90
+++

# A large argument through a program, into smith's MCP server

## Goal

An argument of 64 to 128 KB through a program into smith's MCP server has never run; the largest seen is about 10 KB. This task runs one, with the fake program in the focused suite, says what limit each size meets first, and writes the same story for real Claude Code, compiled, not run.

Acceptance:

- A table test in `smith/tests/agent-process/tests/suite/program_tools.rs` sends a host tool's argument of each row's size through the fake program, the relay and smith's MCP server, and asserts for each row that the argument reaches the host whole, byte for byte, or that the call is refused for the limit the row names.
- Each refused row's name for its limit is the one its failure shows: a comment on the table gives the limit each size meets first.
- A row at a declared payload of 131,072 passes at 65,536 bytes and at the largest argument that payload admits. Where a limit stops it, the fix is below (Options), and the row stays.
- The live story compiles with `heavy cargo nextest run -p smith-agent-process-world --test live --no-run`, and is not run.
- The table's comment says what it costs in CPU, measured, and the `smith-agent-process-world` focused suite stays within its budget.

## Inputs

- `temper/docs/plan/call-room/notes/common.md`.
- `temper/docs/design/draft/goal-as-designer.md`, 6.
- `smith/docs/design/protocol/programs.md`, 5.3 and 5.5.
- `smith/docs/design/protocol/limits.md`, 2.2 (`tool_payload`) and 3.1.

## Owns and may change

Owns `smith/tests/agent-process/**` and `smith/tests/fake-claude/src/**`.

Options, each pre-approved:

- **The fake's own line bound** (`smith/tests/fake-claude/src/fake.rs`, `FILE_BYTES`, 65,536, read by `mcp.rs` and `server.rs`) stops a row past it, and it's the fake's, not the product's. Give the fake's MCP input its own constant large enough for the largest row, rather than moving `FILE_BYTES`, whose `intake` other stories' heaps count. Say so in the report; the table's comment doesn't count it as the limit a size meets.
- **A limit in smith's own path that doesn't follow `tool_payload`** stops a row at a declared 131,072 (a fixed constant in `smith/crates/smith-protocol-program/src/mcp.rs` or `relay.rs`, or `smith/crates/smith-agent-service/src/service/program.rs`): make it follow the payload, in that file, and name it in the report. You may change those three files for this alone.
- **A limit that needs a generated codec's ceiling to move** is a `Question:` (`notes/common.md`).
- **The derivation refuses the row's declarations** in the world's tiny profile: raise what that world declares for its row (its `file_bytes`, its endpoint's window and output, as `skein-llm` requires: an output of at least `tool_payload` × 6 / 8 plus framing). A world may specialise a derived limit (`smith/docs/design/protocol/limits.md`, 2.2).
- **`program_world`'s `Case`** gains one variant carrying the row's payload. Its other cases keep their figures.

## Steps

The world, in `smith/tests/agent-process/src/program_world.rs`, its `configuration(case)` deriving this case's limits from `crate::profile()` with the row's declarations:

```rust
pub enum Case {
    // ...
    /// Host tool calls at a declared `tool_payload`, for the large-argument table.
    Payload { tool_payload: u32 },
}
```

The table, beside `a_nested_host_input_is_told_and_answered` and reusing its `told_result` and `accepted`. The sizes are the argument's whole JSON input; the refused rows' expected values are yours to find, by running them first:

```rust
#[test]
fn a_large_host_argument_reaches_the_host_or_names_its_limit() {
    // Each size's first limit: <what the rows show>. Costs <n> ms of CPU, measured.
    for (declared, argument, reaches) in [
        (10_662, 10_000, Reaches::Whole),
        (10_662, 65_536, Reaches::Refused(/* found */)),
        (131_072, 65_536, Reaches::Whole),
        (131_072, 131_072, Reaches::Whole),
        (131_072, 131_073, Reaches::Refused(/* found */)),
    ] { /* a Step::Call of "lookup" with {"text": <argument bytes>}, then finish() */ }
}
```

If the row at 131,072 can't be sized exactly because of the JSON around the text, keep the rows' meaning: the largest input the payload admits, and one byte past it.

The live story, in `smith/tests/agent-process/tests/live/program.rs`, beside `a_live_result_contract_lists_and_calls_structured_output`, reusing `Live`, `Options` and `prompt_server`'s form of MCP fixture: a Python server whose `echo` tool writes the length and a hash of its `text` to a file in the probe's home. It asks real Claude Code to call the tool with a named text repeated to about 65,536 bytes, then again to about 131,072, smaller first, so a failure says which size. It asserts each length reached the server at least its size less 5%, whole, and that no tool result was an error. Say in the story's comment that after-landing runs it, and that a model's own output cap is one limit it may name. Write it, compile it, don't run it.

Each pinned rule breaks once by hand: a refused row's limit raised shows the row failing for that reason; report what you saw.

`cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-agent-process-world -p smith-fake-claude --all-targets -- -D warnings`, naming any other package you changed; `heavy cargo nextest run -p smith-agent-process-world -p smith-fake-claude`, plus `smith-protocol-program` and `smith-agent-service` if you changed them. Commit, push.

## Report

The table's rows as they ended, with the limit each size meets first, by name and file. Whether a product limit had to move, and which. The table's CPU cost. That the live story was compiled and not run, and what it would name if the model's own output stops it.

## Answers

2026-11-07. **Extend this task to raise `HostInput`'s cap and repair program input admission, or lower the acceptance to 65,536?** Extend it; the acceptance stays at 131,072. The request asks for 64 to 128 KB, and the rows fail on two limits in smith's own path, not on a generated codec's ceiling, so neither is a `Question:` (`notes/common.md`). The branch's checkpoint, `83c2a34adf`, stands: go on from it. You may also change the three files named here, for this alone:

- **`smith/crates/smith-domain-run/src/host.rs`:** `HostInput::CAPACITY` goes to 1,048,576, the generated ceiling of channel `host_ask_input` and transcript `call_input`. It is a cap on the vocabulary: `host_input_bytes` stays the per-deployment receiving bound, and no heap is sized from the constant. Say in its doc line that it matches those ceilings. Leave `HostAnswer::CAPACITY` as it is: this world holds its replies at 512 bytes, and temper renders to that constant.
- **`smith/crates/smith-protocol-program/src/backend/conversation.rs`:** admit a tool input against `translation.client.input`, not `.answer`, at `assistant` (line 177) and at the structured output (line 216), which also becomes a transcript call. Then an input past the transcript's `call_input` is a `Rejection::Limit` of `Bound::Input`, "tool input bytes", rather than a run that can't send its record. `Turns::new`'s raw bound stays on `.answer`. Hold it with a row in that crate's lowest-tier test: an input past `client.input` and within `client.answer` names `Bound::Input`. If an existing fixture's input is past its own `client.input`, raise that fixture's figure and report it. Don't change what a test expects.
- **`temper/crates/temper-engine-smith/src/tests/settings.rs`,** `a_large_settings_decoder_error_is_bounded_host_feedback`: this test sizes its key from `HostInput::CAPACITY`, and `temper-engine-smith/src/json.rs`, line 21, refuses past 65,536. Size the key from 65,536 less 128, so the input and the expected answer stay as they are. Change nothing else in temper.

The row `(10_662, 65_536)` is refused when the run ends with `Bound::Input`'s failure and no host call arrives. Name in the table's comment the bound that failure shows. The run's report read 65,532, where 10,662 × 6 is 63,972, so check which `tool_payload` that row derives from.

Leave these for the plan's later tasks, and list them under `## Left`, since this task's acceptance doesn't need them: `temper-engine-smith/src/json.rs`'s fixed 65,536 stops any temper tool's input past it, and `smith-domain-run/src/limits.rs` derives `host_reply_bytes` from `tool_payload`, so a deployment declaring past 65,536 is refused by `valid_host_tools` against `HostAnswer::CAPACITY` unless its replies are set lower.

Beside the Steps' checks, run `heavy cargo nextest run -p smith-domain-run -p smith-protocol-llm -p smith-protocol-program -p smith-agent-service -p temper-engine-smith` and clippy for those packages, then `heavy cargo nextest run -p temper-service --lib -E 'test(each_invalid_call_names_its_problem)'`. Report the heap ceilings as unmoved, or measure before and after if one moves.
