# worker-slots: a worker's workspaces can stay as they are between workstreams

One environment variable, off by default and never shipped on. Two tasks in temper, one for each way a clone is lost today.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `evict-keeps` | `evict-keeps` | | the request's change 1, and the switch from the shell to the cache: eviction keeps a clone the new workstream names |

Critical path: `evict-keeps`, then the restart task, which is still being written.

## What was decided

- **Cut by what loses a clone.** Eviction needs no new operation, so it goes first and alone. The restart needs one, and everything that speaks git's vocabulary.
- **The variable is a call, not a field.** `WorkerRuntime`, `Limits` and the component's `Settings` are written out in many fixtures, and several of those are fabrig/ui's. A method called after construction changes no literal, adds nothing to the configuration's reference, and leaves an unset deployment running today's code. So: `Service::reuse_workspaces`, called by the shell.
- **`Cached::Kept` tells eviction's reuse apart** from a workstream's own: it is another workstream's clone, which is what the design's reason for off-by-default turns on.
- **Only the least recently used idle workspace is considered,** as the request says: no search for a better one.

## Overlaps

No task touches fabrig/ui's files. Unavoidable, a line or two each: `temper-service/src/tests.rs`'s `mod` list, and `ui-goal-page/plan-filter-out`'s globs, which take all of `temper/crates/**`.

## Design written

`temper/docs/design/domain/worker.status.md`, **Departs** on 5 and 9; `temper/docs/design/protocol/git.status.md`, **Departs** on 2 and 4.
