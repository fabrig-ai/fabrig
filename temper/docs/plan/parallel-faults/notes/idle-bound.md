# parallel-faults: the idle bound (the request's item 5)

No task: reading doesn't settle it, so the request says to leave it. Read from the code on 2026-11-06. Nothing here was run or measured, and the failing deployment's documents weren't read.

## What the code says

- `idle` is per model and required by a deployment's `agent.json` (`smith/crates/smith-agent-service/src/profile/quantities.rs`, `smith/crates/smith-agent-shell/src/config/endpoints.rs`).
- Smith's local presets are 300 s for OpenAI and 120 s for Anthropic (`smith/crates/smith-local-shell/src/presets/openai.rs`, `presets.rs`). `temper init`'s template is 30 s for every model (`temper/crates/temper-init/template/agent.json`), and the help calls its values illustrative (`temper/docs/help/deployment.md`).
- The timer arms at the response head, before any body event. Only output-carrying events re-arm it (`skein/crates/skein-llm-connection/src/deadlines.rs`, `skein/crates/skein-llm/src/client.rs`). Pings don't. So a 200 with no event, a stream of pings and a stall partway all read `timed out during response idle (HTTP 200)` (`smith/crates/smith-events/src/words.rs`).
- A retry resends the same request, four attempts in all, with about 1.4 s of backoff at most (`smith/crates/smith-domain-session/src/limits.rs`).

## What the arithmetic allows

19 runs in 73 minutes is about 230 s a run. If all four attempts of a run hit the bound, a run costs 480 s at 120 s and 1,200 s at 300 s, both over the figure seen. At 30 s it costs 120 s, leaving about 110 s for sending and the head. So 30 s fits and the presets don't, but this doesn't prove it: runs may have failed in other phases, or had fewer attempts.

If the deployment used 30 s, raising smith's presets changes nothing, because they don't reach it. The figure would be in the deployment's `agent.json` and temper's template.

The bound alone doesn't explain why a run at 823 KB worked where 456 KB failed. Nothing here says whether size is the cause at all.

## What would settle it

1. The `idle` value in the replay deployment's `agent.json`.
2. The rerun with `trace` set in `agent.json`, reading each attempt's `response.completed` (`smith/docs/design/protocol/events.md`): `duration_ms - first_byte_ms` close to `idle` means the timer was never re-armed, and `largest_gap_ms` says how quiet a stream that did speak ran. Compare the two failing tasks with the run at 823 KB.

A new figure then comes from those gaps and a sourced figure for a comparable client. The repository records none, and none is guessed here.
