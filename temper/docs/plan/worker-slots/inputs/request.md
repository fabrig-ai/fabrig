# worker-slots: a worker's workspaces can stay as they are between workstreams

At the user's request (2026-11-05): a deployment that runs one trusted person's work on a machine it builds on needs a workspace to behave as a long-lived build directory, so that a new piece of work rebuilds only what changed. Reasonably hack-ish is fine, behind one environment variable, **off by default, and never a shipped default**. fabrig names no tool, machine or session of whoever runs its development (`AGENTS.md`, 1), so nothing here does; a build's target, its cache and its limits are the deployment's own tooling, outside this repository.

## The change

One environment variable read at the worker's start, say `TEMPER_WORKSPACE_REUSE`, unset meaning today's behaviour exactly. When it is set:

1. **Eviction keeps the clone.** In `temper/crates/temper-worker-domain-checkout/src/cache.rs`, `Cache::hold`: when no workspace is free and the least recently used idle one holds exactly the repositories the spec names (`holds_exactly`, by directory and remote), and no repository starts saved, keep its `disk` and answer `Cached::Reused`, or a new `Cached` if the facts need telling apart. `hold.rs` then goes straight to fetch and checkout, unchanged files keeping their mtimes. Anything else evicts as today. A workspace whose disk is `Unknown` is made again, as today.
2. **A restart keeps them.** The cache is in memory, so after a worker's restart each workspace answers `Cached::New`, and `Op::Make` renames it to trash. With the variable set, the first hold on a workspace looks at what its directory holds: where each repository the spec names is there, a git repository, from that remote, treat it as `Reused`; otherwise `Make`, as today. Settle how to compare the remote by what a clone actually writes (no credentials are on disk, worker.md, 5). A clone a killed git left damaged must heal: its next operation that breaks spoils it, and the next prepare makes it again, as today. Prove that with a stale `index.lock`.
3. **A workspace's path stays the same** across eviction and a restart: `ROOT/workspaces/<id>/<repository>`. Things outside key on it, so hold it with a test, and say in the report whether `<id>` depends on anything but the order workspaces are made in. If it can differ after a restart, make it not, with the variable set.

`Command::Clean` and the saved start's empty directory stay as they are.

## Tests

A checkout-world story per change, both with the variable set and unset (`temper/tests/worker/checkout`): a new workstream reuses the evicted clone and a changed file is the only one touched; other remotes evict as today; a restart reuses, and a damaged clone heals. Default paths stay byte for byte what they are.

## Design

`temper/docs/design/domain/worker.status.md`, Design departs against section 5's "a cached checkout is reused only" and "a new workstream ... made again": what the variable does, and why it's off by default: a run can write its `.git`, whose hooks the worker's next git command would run across workstreams, which is acceptable only for one trusted person. No help page. Check `root` may be an absolute path outside the data directory, as `protocol/configuration.md` reads, and say so in the report.

## Beside it

fabrig/ui has priority: leave its files alone, and expect catch-ups. No test reaches a real host. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Size

One or two tasks in temper. No skein, jig or smith change. A stored format, a protocol or a public interface change is a `Question:`.
