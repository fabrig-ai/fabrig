# worker-reply-bound: an engine sizes a run's reads by its worker's reply bound

A goal on a real deployment read a task's words through `read_task` 128 bytes a call: reading one 30 KB spec took about 2 minutes 40 seconds of calls. An engine with no local agent reads a task's words in pieces of the deployment's `documents.text_bytes`, because it is never told its workers' agents' host reply bound (smith's `host_reply_bytes`, the agent's `tool_payload`: 10,662 at `temper init`'s figures). A larger page would reach a run past that bound as TooLarge at every offset (`temper/docs/design/domain/routing.md`, 2; `../../domain/agent/README.md`, 8). **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing test first:** at the lowest tier holding the engine and a remote worker, a run on a worker whose agents' reply bound is larger than the deployment's `documents.text_bytes` reads a task's words in pages as large as that bound allows, and a run on a worker with a smaller bound reads within it. Rows of the existing reads table.

2. **The worker says its bound:** each worker names its agents' host reply bound in its hello on jig's worker link, as a byte count. jig carries it without naming smith or temper. The engine keeps it per worker, and sizes each read of a run by the worker running that attempt, as a local agent's reads are sized today. A worker that names none keeps today's page. This is a protocol change of jig's link: formats change in place before v0.1.0, with no version bump.

3. **The design says so:** jig's worker link where the hello is written, and `../../domain/routing.md` 2 and `../../domain/agent/README.md` 8 where the page is.

## Not in this plan

A charter's words, and any figure: the bound is the worker's own.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test; no test reaches a real host or a real LLM. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** jig never names temper, and smith names neither. `walk = false`.
