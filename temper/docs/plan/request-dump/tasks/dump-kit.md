+++
id = "dump-kit"
part = "dump-kit"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-llm/src/client.rs",
  "skein/crates/skein-llm/src/client/tests.rs",
  "skein/crates/skein-llm-connection/src/**",
  "skein/crates/skein-fake-llm-protocol/src/provider.rs",
  "skein/crates/skein-fake-peers/src/llm.rs",
  "skein/tests/fake-peers/tests/suite/llm.rs",
  "smith/crates/smith-protocol-llm/src/component.rs",
  "smith/crates/smith-protocol-program/src/backend.rs",
  "smith/crates/smith-protocol-program/src/backend/**",
  "smith/crates/smith-protocol-program/src/mcp.rs",
  "smith/crates/smith-protocol-program/src/lib.rs",
  "smith/crates/smith-agent-service/src/**",
  "smith/crates/smith-agent-shell/src/**",
  "smith/tests/agent-process/**",
]
effort = "high"
minutes = 120
+++

# The kit: an agent writes each model request's rendered body, and what a program was given, to a private directory

## Goal

The rendered body of each API request, those very bytes and not a re-rendering, and what a Claude Code child was given, reach a new shell writer in `smith-agent-shell`, when its host asks; unasked, nothing changes but one branch. No skein or smith name of temper. temper turns it on in a later task (`dump-worker`).

**One departure from how tasks are cut:** this branch crosses two trees, skein and smith, because the request sizes the whole at one or two tasks and it comes out whole. Keep the kit's rule inside it: **skein's commit first, smith's after it, each saying why.**

Acceptance:

1. With `Agent::dump_to`, each API request writes one file, before it's sent: its bytes equal what the fake provider read, for an OpenAI-style account with a bearer and Anthropic with a key; the fake's credential is in no file; the file is 0600 in a directory made 0700, and no `.dump-*` part stays.
2. A gzip endpoint's kept body is the body before compression, and equals what the fake read after inflating it (`skein-llm`'s test).
3. A Claude Code run under `Agent::dump_to` writes `claude-code-argv.txt`, `-instructions.md`, `-tools.json`, `-stdin.json` and `-init.json`; the argv file's arguments equal what the fake program was started with, `--effort` and `--model` among them; no file holds `SMITH_MCP_SECRET`, no file is `mcp.json`.
4. A run's `start.txt` names its budget's turns and spend and its tools granted.
5. A directory that can't be made, and one open to group or others, each leave the run answering as it would, with one line on the agent's error output per failed write.
6. A small injected bound lets the first file through, stops the second, logs one line, and the run answers. The line is also in `dump.log`.
7. Without `dump_to`, `Client::take_kept_body` and `Component::pop_body` give `None`, and `run` and `run_decoded` keep their signatures.

## Inputs

- `temper/docs/plan/request-dump/notes/contracts.md`: every signature and file form here is binding.
- `temper/docs/plan/request-dump/notes/common.md`.
- `temper/docs/design/draft/dogfood-hacks.md`, 2.13.
- `skein/docs/design/llm.status.md`, `skein/docs/design/llm-connection.status.md`, `smith/docs/design/protocol/agent.status.md`, `smith/docs/design/protocol/programs.status.md`: the departures this builds, and what your comments cite.
- `smith/docs/design/protocol/agent.md`, 6 (standard error is the agent's log); `smith/docs/design/protocol/programs.md`, 5.1 (the command and its files).

## Owns and may change

**Owns:** the `files` above. Inside them, in particular:

- `skein-llm/src/client.rs`: `MeasuredRequest` and `Client` each gain one field, and their one constructor each sets it (`measure_request`, `prepare_measured`).
- `skein-llm-connection/src/component.rs`, `boundary.rs` and `lib.rs`: `Body`, `keep_bodies`, `pop_body`.
- `smith-agent-service/src/service/` and a new `dump.rs` beside `service/program.rs`; `service/stages.rs`'s `start_ready`, which sees the charter before it moves into the domain.
- `smith-agent-shell/src/dump.rs` (new), `agent_shell.rs`, `lib.rs`.
- `smith/tests/agent-process/tests/suite/dump.rs` (new) with its `mod` line, and what `src/world.rs` and `src/program_world.rs` need to pass a `Dumps` to the `Agent` they build.

**The survey found no other reader to change:** the additions are new methods and types. `Below`, `Measure`, the connection's `Event` and `provider::Event` stay as they are, so no exhaustive `match` elsewhere moves; please keep it so. `Client` and `MeasuredRequest` are built in one place each; so are `Backend`, `Stages` and `Agent`.

**Options, each with its default:**

- If a memory test's figure moves by the new fields' size: where it's derived, nothing; where it's a literal, change that literal by exactly that size and say so in the report. If it moves by more, you've kept something while unasked: fix that.
- If the two `prepare_*` paths in the connection make one take-point awkward, take the body at each: every prepared call's body must be taken, the waiting path's included.
- `std::fs::File` is banned by the shell's `clippy.toml`: where naming it can't be avoided, use a scoped `#[expect(clippy::disallowed_types, reason = "...")]`, the reason being that a dump's file is written whole and closed within a pass, outside hosted io.
- `Case` in `program_world.rs` crosses as JSON into the spawned world: if a directory fits badly there, pass it however the world already passes such facts. Default: a new field or variant.

## Steps

Read `notes/contracts.md` first. It holds every signature below.

**skein, first commit.**

1. `prepare_measured`: once `rendered` exists and before gzip, when asked, keep `rendered.clone()`. This is the seam nearest the wire whose bytes are readable, and it's what the report's proof rests on: `translate::encode_measured_request` and `anthropic::encode_measured_request` take the prompt and limits, and never `input.credential` or `input.endpoint`.
2. `skein-llm-connection`: `measure_start` asks with `keep_body` when `keep_bodies` was called. Every prepared client's body is taken into a queue of `Body` that `pop_body` drains, on both the immediate path (`admit_policy`) and the waiting one (`prepare_waiting`).
3. The fake: `provider::requested` clears `server.body` soon after decoding, so with `keep_body` it copies the body first, after inflating. `Peer::keeping_bodies` asks each connection's server, and `bodies` holds what they read; off by default, since the worlds count memory.
4. Tests:
   - `skein-llm/src/client/tests.rs`: `a_prepared_call_keeps_its_rendered_body_only_when_asked`, identity rows comparing to what `down` writes after the head and showing no credential in it, and a gzip row comparing to the inflated upload.
   - `skein-llm-connection/src/tests.rs`: `an_owner_that_asks_takes_each_call_s_body`, on both paths; and unasked, none.
   - `skein/tests/fake-peers/tests/suite/llm.rs`: a body kept as read, with one in gzip.

**smith, second commit.**

5. `smith-protocol-llm`: `keep_bodies` and `pop_body`, delegating.
6. `smith-protocol-program`: a `Given` queue on `Backend`, present only when asked. Push:
   - `Arguments` in `launch`, from the `launch_spec` it just built, so a compaction's and a resume's count;
   - `Instructions` and `Tools` in `store_instructions`, the second from an accessor you add to the MCP server for its `tool_list`;
   - `Input` in `transport.rs` where `finished` holds, with the whole of `outgoing`, since every line goes through there (`start.rs`, `recovery.rs`, `stop.rs`, `conversation.rs`);
   - `Init` where a `system/init` line is read, the first one (`stream::init`'s caller) and `later_init`.
7. `smith-agent-service`: `keep_dumps` asks the LLM, each program backend and each child backend started later (`start_child`), mirroring `service/program.rs`'s `pop_measure`. `start_ready` records identity and activation and, when asked, queues `Started`. `pop_dump` follows the contract's order.
8. `smith-agent-shell`: `dump.rs` is ordinary Rust, `#[cfg(unix)]` as `program` and `relay` are, so that `agent_shell.rs`'s `not(unix)` build still compiles: the directory, file, name, bound and line rules of the contract. In `Host::iterate` for `Agent`, after `service::iterate`:

   ```rust
   if let Some(dumps) = &mut self.dumps {
       while let Some(dump) = self.service.pop_dump() {
           dumps.write(dump, self.errors.as_mut());
       }
   }
   ```

   `Agent::dump_to` calls `keep_dumps`. Say in `agent_shell.rs`'s module doc what it keeps.
9. Tests in `smith/tests/agent-process/tests/suite/dump.rs`, in a `skein_scratch::Scratch` directory, on the real filesystem, with the simulated kernel for the rest:
   - `a_request_s_file_holds_the_bytes_the_provider_read`: rows for an OpenAI-style bearer and an Anthropic key, with `World::private_grant`, a `Peer` keeping bodies, and a script of two turns. Each `*-openai.json` or `*-anthropic.json`, in order, equals `bodies()[n]`; `PRIVATE_GRANT` is in none of the directory's files; modes 0600 and 0700; no `.dump-*`; the names sort in request order.
   - `a_run_s_start_file_says_what_it_was_given`.
   - `a_directory_that_cannot_be_made_leaves_the_run_answering`: its path under a regular file; the answer is `Accepted`; `World::errors` holds one `request dump` line per failed write.
   - `a_directory_open_to_others_is_refused`: 0755; no file in it; the run answers.
   - `the_bound_stops_the_writing_with_one_line`: a bound between the first file's size and the first two's.
   - `a_claude_code_run_keeps_what_it_was_given`, in the program world: each of the five files; argv compared to `Run::invocations`; the stdin file holds the prompt's text; no `SMITH_MCP_SECRET` in any file; no `mcp.json`.
10. For each new test, break its rule once by hand and see it fail, as your text says.
11. `cargo fmt`, `old_forms.py` for skein and smith, and `heavy cargo clippy -p skein-llm -p skein-llm-connection -p skein-fake-llm-protocol -p skein-fake-peers -p smith-protocol-llm -p smith-protocol-program -p smith-agent-service -p smith-agent-shell -p smith-agent-process-world -p skein-fake-peers-tests --all-targets -- -D warnings`. Then the focused tests of those packages. For dependents, additive changes need only `heavy cargo check --tests` of `smith-local-service`, `smith-inline-agent`, `temper-fake`, `temper-playground`, `temper-service` and `temper`.
12. Narrow runs: `heavy cargo nextest run -p smith-agent-process-world --test suite -E 'test(/dump::/)'`; `-p skein-llm --lib`; `-p skein-llm-connection --lib`; `-p skein-fake-peers-tests` with your test's name.
13. Push `request-dump/dump-kit`.

## Report

Beside what your text asks:

- **How the seam proves no credential reaches a file:** the functions a body and each `Given` come from, and what they don't take.
- Which file a gzip endpoint's body is, and that it's before compression.
- Any request or input line you found that doesn't pass through these points, in plain words: a gap there is the user's to know.

## Answers

2026-11-05 (fabrig/temper). Leave start.txt out, and its 40 lines, if you can still do so cheaply: smith's trace already gives each run's system text, tools and effort. If it's already written and tested, keep it and say so. The design's entry is now dogfood-hacks.md 2.13, not 2.10: cite that.
