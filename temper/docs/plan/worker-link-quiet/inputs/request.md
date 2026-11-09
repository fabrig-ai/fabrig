# worker-link-quiet: one run leaves a remote worker's link quiet until it drops

On a real deployment at fabrig `a66fcfb9f7`, which already holds `worker-relinks` and `worker-reply-bound`, there is one engine and one remote worker on another host, reached through a TCP forward. One task, a change's producer on a Codex program, makes the link go quiet each time it's released. The producer is being resumed onto its kept tree after earlier lost runs. Its specification is about 8 KB.

What happens:
- About 15 s after the release, the worker logs `lost w1 ... silence` and the engine logs `worker lost worker=w1 channel=N: stream`.
- The worker relinks at once, and the link goes quiet again, about every 2 min 16 s.
- After eight losses the task holds with `retry allowance exhausted: lost runs`.
- Every run in flight on that worker is lost with each loss.
- With the task held, the link stays up for hours. Three releases gave three bursts of eight.

During the quiet, both processes sit idle in their io_uring waits: the engine's main thread is in `skein_shell::drive`, and the worker's thread is in `io_cqring_wait`. Both TCP sockets have empty send and receive queues, and the worker's agent processes are alive. So nothing at all crosses the link for the whole silence span, heartbeats included, while the engine has a run to give that worker. The evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing story first,** at the lowest tier that holds the engine and a remote worker's link. A run's start or resumption carries the largest instructions, specification, transcript window and tree the stock settings admit, and then one past each. The link must stay live, with heartbeats flowing. The run either starts or fails with its reason named. Never silence. Find where it goes quiet. Candidates:
   - a frame the engine can't send (past the worker's frame cap, or the link's credit) blocks that channel's writer, heartbeats included, with no error;
   - the engine's output waits on credit the worker never grants (the worker's entrance, `engine_cap(...).frames + 3`), so neither side moves;
   - the start or resumption passes a bound, is dropped without a fault, and the link then waits for an answer that never comes;
   - the worker drops a too-large frame without faulting the link.
2. **The fix:** a frame that can't be sent fails its run with a line naming the bound and the size, on the side that refused it. The link keeps its heartbeat. One run's failure never costs the link, nor the other runs on it.
3. **The design says so:** `temper/docs/design/domain/worker.md` and the worker link's protocol design, with their status files.

## Not in this plan

Any figure. Making runs survive a link loss is another milestone's work.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
