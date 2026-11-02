+++
id = "crossing-program"
part = "crossing-program"
kind = "feature"
needs = ["claude-shapes"]
files = ["smith/tests/agent-process/src/program.rs", "smith/tests/agent-process/src/program/**", "smith/tests/agent-process/tests/suite/program_events.rs", "smith/tests/agent-process/tests/suite/program_resume.rs", "smith/tests/agent-process/tests/live/program.rs", "smith/crates/smith-protocol-program/src/backend/compact.rs", "smith/crates/smith-protocol-program/src/backend/compact_tests.rs"]
effort = "high"
minutes = 80
+++
# The chat crossing's Claude leg, checked below the live story

## Goal

`smith::live a_chat_crosses_from_chatgpt_to_claude_then_back` runs a ChatGPT-started chat on
Claude Code (resumed), then on ChatGPT again; in one run its exec failed, in another a leg's file
was wrong. Its Claude leg relies on two things no test checks today:

1. **`run.started.resumed` on a program run.** The live story expects `resumed == false` on a
   fresh Claude run and `true` on a carried-over one (smith/docs/design/domain/session.md, 8.2;
   `resumed` is `transcript.is_some()` in `smith/crates/smith-domain/src/route.rs`). The
   agent-process world reads a program run's events (`a_program_run_s_events_name_its_backend`)
   but never asserts `resumed`, and its carried-over runs (`program_resume.rs`) capture no events.
2. **The summary session's init on the real program.** A conversation carried over from another
   backend first runs a summary session: `compact::arguments` drops `--mcp-config`,
   `--permission-prompt-tool` and `--json-schema` and keeps `--strict-mcp-config`, `--tools ""`,
   `--max-turns 1`; `compact::checked_init` refuses its init unless `tools` and `mcp_servers` are
   both empty arrays, failing the run ("summary program offered tools"). No live probe runs that
   argv (every probe passes `--mcp-config`), and `compact::checked_init` has no unit test.

Add the tests that pin both, red before a fix where smith is wrong, and a live probe of the
summary session's init on Claude Code 2.1.312 for the after-landing pair.

Acceptance:

- `program_events.rs` `a_program_run_s_events_name_its_backend` also asserts exactly one
  `RunStarted` with `resumed == false`.
- `program_resume.rs` `a_conversation_is_carried_over_from_the_direct_backend` also asserts the
  program run's `RunStarted` has `resumed == true`, its events captured as `Case::Events` captures
  them (give the window case events: a new `Case` or a flag on `run_with_window`, your choice).
- `smith-protocol-program --lib`: `compact_tests.rs` (a `#[cfg(test)] #[path]` module of
  `compact.rs`, as the backend's other `*_tests.rs` are) with
  `a_summary_init_with_no_tools_and_no_servers_passes` (the recorded init keys of
  `stream_tests.rs` `init_line`, `tools: []`, `mcp_servers: []`) and
  `a_summary_init_with_a_tool_or_a_server_fails`.
- `smith/tests/agent-process/tests/live/program.rs`: `a_live_summary_session_offers_no_tools`, an
  `Options` flag that starts `claude` with the summary argv (no `--mcp-config`, with
  `--strict-mcp-config`, `--tools ""`, `--max-turns 1`, as `compact::arguments` builds it), and
  asserts init's `tools` and `mcp_servers` are `[]`, its message naming the tools and servers it
  saw. Checked by compilation only.
- If assertion 1 or 2 is red, the cause is fixed in smith with a unit test in the crate that holds
  it (see Pre-approved), and both go green.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds/notes/common.md`.
- `smith/docs/design/protocol/programs.md`, 5.8 (a conversation carried over) and 8 (the live
  suite checks what the fake assumes); `smith/docs/design/domain/session.md`, 8.2;
  `smith/docs/design/protocol/events.md`, `run.started`'s row.
- `smith/tests/agent-process/src/program.rs` (`Case`, `configuration`, `make_agent`'s trace for
  `Case::Events`, `run_with_window`).

## Owns and may change

- The files in this spec's header. `claude-shapes` has landed: keep its two probes as they are.
- Pre-approved:
  - If a fresh program run reports `resumed == true`, or a carried-over one `false`, find where
    `resumed` is set for a program run (`smith-domain` `route.rs`, `smith-domain-run`
    `facts.rs`/`run.rs`, `smith-protocol-events`, `smith-agent-service`), fix it there with a unit
    test in that crate, and list the crate; if the fix needs a change in more than one crate or
    in a channel or event wire form, stop and ask.
  - If carried-over program runs have no `RunStarted` at all, that is a question: ask.
  - If the fake `claude`'s summary session does not answer the window case's events the way the
    run needs, change only `smith/tests/agent-process/src/**` and say so.

## Steps

1. Add the two world assertions; run them
   (`heavy cargo nextest run -p smith-agent-process-world --test suite -E 'test(/a_program_run_s_events_name_its_backend|a_conversation_is_carried_over_from_the_direct_backend/)'`);
   note red or green.
2. Add `compact_tests.rs`; `heavy cargo nextest run -p smith-protocol-program --lib -E 'test(/compact_tests::/)'`.
3. Fix any red from step 1 as Pre-approved; rerun.
4. Add the live probe; `heavy cargo check -p smith-agent-process-world --tests` (never run it).
5. `cargo fmt`; `heavy cargo clippy -p smith-agent-process-world -p smith-protocol-program --all-targets -- -D warnings`
   (and any crate fixed in step 3); `heavy cargo nextest run -p smith-agent-process-world`;
   `heavy cargo nextest run -p smith-protocol-program`; the fixed crate's tests if any.
6. Commit, push.

## Report

Step 1's result for each assertion (red or green, what it saw), any fix and its crate, the tests
run (selected, passed), and implementation and check minutes apart.
