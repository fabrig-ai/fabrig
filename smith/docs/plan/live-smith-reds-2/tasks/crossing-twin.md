+++
id = "crossing-twin"
part = "crossing"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/chat_crossing.rs", "smith/tests/end-to-end/end_to_end.rs", "smith/crates/smith-local-protocol/src/store.rs", "smith/crates/smith-host-domain/src/values.rs", "smith/crates/smith-host-domain/src/channel.rs", "smith/crates/smith-host-protocol/src/translate.rs", "smith/crates/smith-host-protocol/src/component.rs", "smith/crates/smith-host-protocol/src/tests.rs", "smith/crates/smith-protocol-translate/src/transcript.rs", "smith/crates/smith-protocol-translate/src/turn.rs", "smith/crates/smith-local-domain/src/agent.rs", "smith/crates/smith-local-service/src/host_io.rs"]
effort = "high"
minutes = 240
+++
# A fake-backed exec chat crosses Direct, program, Direct through the product

## Goal

In the whole after-landing run at main 46e6e70d9, the chat crossing's second leg, the first run
on Claude (a `program` account, Claude Code 2.1.312) of a chat begun on ChatGPT (Direct), failed
about 17 s in, before its run started:

```
account=claude invocation=2 step=Exec  failed=live exec succeeds
exit=Code(1) forced=No live_pids=0  error=none  end=Failed reason=Agent
session=2 run.completed status=Failed turns=0 failure.class=Agent ended=Signal(9)
session=2 session.started / session=2 notice kind=unknown / session=2 session.ended exit=1 answered=true
```

`class=Agent ended=Signal(9)` is smith's own spawned `smith-agent` ending without an answer,
seen by the host as signal 9 (`smith/crates/smith-local-domain/src/domain.rs`, `end.rs`;
`smith/crates/smith-local-service/src/stream.rs`); Claude Code's own failures are `Program`
(smith/docs/design/protocol/programs.md, 5.6). No `run.started` reached the agent's trace. Whether
smith killed it (a host fault: `Rules`, `TooLarge` or `Exited`, then SIGKILL after the 2 s grace)
or something outside did is unknown; `report-lines` makes the next live red say which.

The carried-over leg is pinned only one leg at a time, in the agent-process world
(`smith/tests/agent-process/tests/suite/program_resume.rs`,
`a_conversation_is_carried_over_from_the_direct_backend` and `..._to_the_direct_backend`). No
offline story runs the product across the three legs: `smith exec --chat` on a Direct account,
then on a `program` account (the product spawning `smith-agent`, which compacts the transcript in a
tool-free summary session and opens the program's new window), then on the Direct account again.
Write that story against the fake provider and the fake `claude`: the lowest world that holds the
host, the spawned agent and a carried transcript together.

Acceptance:

- `smith/tests/end-to-end/chat_crossing.rs`, a module of the `end_to_end` binary, holds
  `an_exec_chat_crosses_from_direct_to_a_program_and_back`: three `smith exec --json --chat chat`
  invocations in one scratch, each exit 0 with an accepted answer; the program leg's agent trace
  (`<trace>.agent`) holds `run.started` with `resumed` true, the fake's observations show a
  summary `-p` invocation without `--mcp-config` before the window's with it; the third leg's
  `run.started` (in the product's trace or stream, as a Direct run records) has `resumed` true.
- If the story is red, the red is fixed in the one smith crate that holds it, with the story
  green after; if it is green, the report says so, and that the leg-2 kill is not reproduced
  offline.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds-2/notes/common.md` (read it first).
- `smith/docs/design/protocol/programs.md`, 5.8 (carrying over) and 8 (the fake's stories: "a
  conversation carried over from the direct backend and back").
- `smith/docs/design/domain/session.md`, 8.2.
- `smith/docs/design/protocol/events.md`, 2 ("Where it goes": a program run's records are the
  spawned agent's, in the trace with `.agent` appended).
- `smith/docs/design/testing.md`, 2.3 (the crossing story, "Each run's own records").

## Owns and may change

- `smith/tests/end-to-end/chat_crossing.rs` (new): the story and its private helpers.
- `smith/tests/end-to-end/end_to_end.rs`: one line, `mod chat_crossing;`, among the module lines
  (where `cargo fmt` puts it).
- Read, never change: `support/mod.rs`, `support/exec.rs`, `program_runs.rs` and
  `program_runs/records.rs` (other tasks edit some of them). Copy what you need of their helpers
  (`signed_in`'s fake-`claude` login, `records::agent` and `records::read`, the fake provider
  settings of `support::exec::fake_provider_settings`) into your module, or call them where
  they are public.
- Pre-approved:
  - The accounts: the Direct legs on an `env` account against the fake provider, as
    `support::exec::exec_with_provider` runs one (`fake_provider_settings`'s `codex` endpoint and
    account, `Reply::Accepted`); the program leg on a `program` account on the fake `claude`
    (`[programs] claude_code=<fake-claude>`, `provider='anthropic'`), chosen per leg with
    `-c endpoints.<name>.account=...` and `-m` as `support::live::arguments` does.
  - The fake `claude`'s story scripts the summary turn and the window's turn, as
    `program_resume.rs`'s carried-over test does through `program_world`; read the fake's
    `Story`, `Turn` and `Step` in `smith/tests/fake-claude/src/story.rs`.
  - If a leg's settings cannot be expressed through `-c` and one user layer, write the user layer
    afresh between legs (the chat's files stay in the state directory) and say so.
  - If the story is red in smith's code, fix it in the one crate the red points to, with a unit
    or world test there red before and green after, and say which; if the fix would need a
    second crate, a protocol or a stored format, stop: commit the red story, push, and ask.
  - If the product's fake-provider harness cannot serve a run after a program run in one scratch
    without a change to `support/`, stop and ask with what is missing.

## Steps

1. Read the inputs, `program_runs.rs` (`signed_in`, `a_program_exec_traces_its_run_to_the_agent_file`),
   `support/exec.rs` (`fake_provider_settings`, `exec_with_provider`, `run`), `program_resume.rs`
   (the two carried-over tests) and the fake `claude`'s story types.
2. Write the story; run it:
   `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(=chat_crossing::an_exec_chat_crosses_from_direct_to_a_program_and_back)'`.
   Note its first result for the report (green, or the red's assertion and what the leg's
   report or standard error says).
3. If red: find the cause, fix it per "Pre-approved", rerun green.
4. `cargo fmt`; `heavy cargo clippy -p smith --features end-to-end --all-targets -- -D warnings`
   (and every crate you changed).
5. `heavy cargo nextest run -p smith --features end-to-end --test end_to_end`; if you changed a
   crate, its package's tests and its direct dependents' (`AGENTS.md`, 6).
6. Commit, push.

## Report

The story's first result, any fix (crate, test, red and green), the files changed, the test runs
(selected, passed), the story's wall time, and implementation and check minutes apart.

## Answers

2026-11-02. Question (run crossing-twin-1): authorize the cross-crate repair that keeps a carried
transcript's endpoint through the host's forwarding, or leave the red for a later plan?

Answer: **the repair is authorized in this task**; the red story stays as committed (`24037da5c2`)
and goes green here. The red is a departure from the design, not a new decision:
smith/docs/design/protocol/transcript.md, 4 says a window on an endpoint "the agent's configuration
does not name" is carried over, not refused, and 8 says turns go back on resume as they were told.
Today the host decodes the stored window with the leg's configuration (`smith-local-protocol`
`store.rs`), loses the unconfigured endpoint's name (`endpoint: None`), and then cannot re-encode
it for Start (`smith-host-protocol` `translate.rs` through `encode_turn`), so the spawned agent is
never started and is killed as `Agent`/`Signal(9)`. No stored format, wire protocol or design
changes: the agent receives the v6 turn bytes the store already holds, under the existing channel
schema, and resolves the names itself against its own configuration, as it already does.

- **Shape, in order of preference.** (1) The host forwards the saved window's turn bodies verbatim:
  the store returns them with the decoded transcript, the host's Start carries them beside the
  decoded `Transcript` (which the host keeps for its own checks), and `encode_start` puts them in
  the Start's transcript piece instead of re-encoding the decoded turns. (2) Only if (1) cannot hold
  (the host changes the transcript between the store and Start, so the saved bytes are no longer
  the history it sends), the decoded turn keeps its saved endpoint name when the configuration does
  not resolve it, and `encode_turn` writes that name back; say in the report why (1) did not hold.
  Never a substitute endpoint, an extra model or grant, or a configuration workaround (no adding the
  old endpoint to the leg's agent configuration).
- **Lowest world first, red before, green after:** a unit test in the lowest crate the shape
  touches (for (1), `smith-host-protocol`'s `tests.rs`: `encode_start` of a resumed Start whose
  saved window ran on an endpoint the given `Endpoints` does not name succeeds, and the frame's
  transcript decodes to the saved bodies byte for byte; for (2), `smith-protocol-translate`:
  `decode_transcript` then `encode_turn` with such `Endpoints` gives back the saved name), then the
  product story `an_exec_chat_crosses_from_direct_to_a_program_and_back` green, with legs 2 and 3
  and the summary/window ordering checked.
- **Owns and may change, widened:** the crates on that path, `smith-local-protocol`,
  `smith-host-domain`, `smith-host-protocol`, `smith-protocol-translate`, `smith-local-domain`,
  `smith-local-service` (the frontmatter's `files` list the likely files), and the construction
  sites and tests in other crates that a changed type forces to compile, nothing more. Not
  `smith-transcript`'s codec, not `smith-channel`'s generated schema, not any `docs/design/` (what
  the design's status file should say goes in the report).
- **Checks:** fmt; clippy with `-D warnings` on every changed package; each changed package's tests
  and its direct dependents' (AGENTS.md, 6), one command at a time; the whole `end_to_end` binary.
  If a stored format, the channel schema or a second protocol turns out to need changing, stop and
  ask, as before.
