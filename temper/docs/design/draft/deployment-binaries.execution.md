# deployment-binaries: execution

2026-11-09. How [deployment-binaries.md](deployment-binaries.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone, and each walk comes after its milestone lands.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| M1, the client: `temper-client`, the TUI, `ctl`, their help, `--version`; dropped under option D of 5 | 4, 9.1 | temper/tui | — | none: step tests, and the draft's ctl story (10) run after landing | done: landed 2026-11-08 15:43Z (`8e46725ad7`), plan `client-binary`; the ctl story (10) runs after landing |
| M2, the engine: the service's split and `temper-engine`, with its web, `forge`, `versions`, `upgrade`, help and `--version` | 4, 9.2 | temper/ui | — | none: the draft's cluster story (10) run after landing | done: landed 2026-11-08 20:06Z (`b737a87c78`), plan `ui-engine-split`, but for `upgrade`, which waits on jig's rehearsal and going back (`../protocol/binaries.status.md`, 2); the cluster story (10) was written in M3 |
| M3, the worker: `temper-worker`, `program-login`, help, `--version`, and naming the agent's program | 4, 9.2 | temper/migration | M2's split of the service; M4's smith crate for reading the agent's document | none: the same cluster story | done: landed 2026-11-09 (`9d53e0556bcc`), pull request #150; the cluster story (10) on the four binaries is written and compiled, and runs after landing in the process tier |
| M4, the agent: `temper-agent` and its modes, and smith's crate for the agent's document without the agent service | 4, 9.3 | the agent-binary plan (smith's `docs/design/shell/README.md`, 2.6; `../protocol/binaries.md`, 3 and 4; `../protocol/setup/owned.md`, 6; `../protocol/setup/README.md`, 7) | — | none | done: built by plan `agent-binary`, all six ranked items, none dropped; carried onto main by plan `agent-binary-closure`, landed 2026-11-09 04:36Z (`e3fdb2177a`), which made the engine's and the agent's closure rows pass |
| M5, the box's `temper` links all four, the dependency rule's test, and the help's pages, starting from a commit's `examples.tar.gz` | 4, 9.4 to 9.6, 10 | temper/migration | M1 to M4 | none | not started |
| M6, the release job builds and publishes both sets | 2, 6, 7, 8 | whoever runs the site's release job | M1 to M5; the user's choice of names (5) for the client's file | a fresh Linux machine runs an engine, a worker and its agent from one commit's directory, by the help alone (10) | not started |

## 2. Order

- **M1, M2 and M4 start at once:** none needs another.
- **M3 builds on M2's split and M4's smith crate.**
- **M5 closes the code** after M1 to M4.
- **M6 publishes the engine, worker and agent once M5 lands,** and the client's file once the user has chosen its name.
- **After M6:** the deployment the site runs switches its engine and worker units to the set. That's temper/migration's, outside this file.
