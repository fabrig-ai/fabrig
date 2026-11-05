# worker-slots: a worker's workspaces can stay as they are between workstreams

One environment variable, off by default and never shipped on. Two tasks in temper, one for each way a clone is lost today.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `evict-keeps` | `evict-keeps` | | the request's change 1, and the switch from the shell to the cache: eviction keeps a clone the new workstream names |
| `restart-keeps` | `restart-keeps` | `evict-keeps` | the request's changes 2 and 3: a start keeps `workspaces`, a first hold looks, and a damaged clone heals |

Critical path: `evict-keeps`, then `restart-keeps`. There is no parallel work: the second builds on the first's switch and tests, and both write `hold.rs` and `reuse.rs`.

## What was decided

- **Cut by what loses a clone.** Eviction needs no new operation, so it goes first and alone. The restart needs one, and everything that speaks git's vocabulary.
- **The variable is a call, not a field.** `WorkerRuntime`, `Limits` and the component's `Settings` are written out in many fixtures, and several of those are fabrig/ui's. A method called after construction changes no literal, adds nothing to the configuration's reference, and leaves an unset deployment running today's code. So: `Service::reuse_workspaces`, called by the shell.
- **`Cached::Kept` tells eviction's reuse apart** from a workstream's own: it is another workstream's clone, which is what the design's reason for off-by-default turns on.
- **Only the least recently used idle workspace is considered,** as the request says: no search for a better one.
- **The look is a git invocation, not a file read.** Parsing `.git/config` by hand would be a second reader of git's format. `--git-dir` from the worker's root means no repository above the directory can answer for it, and a missing directory is an exit, not a refused spawn. Settled against real git 2.47.3 (the task says what it printed).
- **The remote is compared byte for byte with `remote.origin.url`.** `git clone` writes the URL it was given, and credentials travel in a header, so the bare remote is what is there. `origin-matches` holds that against `clone`'s own `argv`.
- **A look names only the spec's repositories,** as the request says, where eviction's `holds_exactly` knows the whole workspace. A directory beside them stays until a make; the status entry says so.
- **The restart's status gives the true reason for off-by-default.** The request names hooks, but `core.hooksPath=/dev/null` holds them off (`temper-worker-protocol-git/src/invocation.rs`). Tried with real git, under the same settings: a `post-checkout` hook did not run, and a filter driver in `.git/config` did. So the entry names what the worker's git still obeys.
- **No search for the workspace that holds a remote after a restart.** A workspace's first hold looks at that workspace alone, as the request has it. With several remotes, a workstream that comes first can make again a clone another would have wanted: wasteful, never wrong.

## Overlaps

No task touches fabrig/ui's files. Unavoidable, a line or two each: `temper-service/src/tests.rs`'s `mod` list; `temper/crates/temper/tests/suite/fake.rs`'s arm for the new command; and `ui-goal-page/plan-filter-out`'s globs, which take all of `temper/crates/**` and `temper/tests/**`.

## Design written

`temper/docs/design/domain/worker.status.md`, **Departs** on 5 and 9; `temper/docs/design/protocol/git.status.md`, **Departs** on 2 and 4.
