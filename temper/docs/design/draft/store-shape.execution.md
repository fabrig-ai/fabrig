# store-shape: execution

2026-11-13. How [store-shape.md](store-shape.md) gets built, and how a deployment's store, dev.fabrig.ai's first, migrates to it in place (its section 10). It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. Each milestone reaches a deployment as an ordinary upgrade; none exports the store or starts one afresh.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| S1, the census: counts per keyspace and per layout in jig's component, kept as commits apply and rebuilt at recovery; `temper store census`; the deployment's document showing them | 8, 10.1 | temper/dev | none | after landing, on a staging deployment: the census matches a count of the snapshot by key prefix, and a put and an erase move it | landed; its walk not run |
| S2: no head-only walk remains; a store behind the build is refused by name, naming bridge build `3d1c717d379e` | 10.1 | temper/dev | none | none | landed, goal 2652 |
| S3, generations by hard links: opening's new generation links the previous one's snapshot and sealed segments and copies only the newest segment and the secrets' store; skein-io's link | 5, 10.1 | temper/dev | none | after landing, on a staging deployment: an upgrade's new generation shares the snapshot's file with the previous one, and going back before the new one's first commit opens the previous one at its commit | landed; its walk after landing not yet run |
| S4, layout ids: defaults in the codec's schemas; a value's head as its value type's layout id, the families' broad roots resolved by key; the store's registry; derived plans at read; written changes as written jobs; jobs at open; the open's check; the frozen-text rule dropped from skein's codec design | 3, 4, 10.1, 10.4 | temper/dev | none | after landing, on a staging deployment: a store from before S4 is refused by name, naming bridge build `3d1c717d379e`; a field with a default reaches a store at the floor with no patch and no walk, and an older value reads with the default | landed, goal 2826; its walk after landing not yet run |
| S5, milestones as attempts: jig's tasks child's attempt and hold records; the milestones' readers moved to them (escalations and the task record, history and activity pages, goals' projections and temper's feed, the clients' wire with its version moved, the task page, the TUI and `temper ctl`) | 6, 10.1 | temper/dev | S4 | after landing, on a staging deployment: the milestones' keyspace is empty, and each task's page shows its attempts as it showed its lifecycle | landed, goal 3361; its walk after landing |
| S6, retention: each keyspace's horizon declared by its owner, as `store-shape.md`, section 6 sets them; the domain's timers erasing what has passed it, in pages; an ended task's summary written when it ends; turns' files dropped 14 days after their task ends | 6, 10.1 | temper/dev | S4 | after landing, on a staging deployment with its clock moved on: a task past its horizon keeps its summary and loses its full record, and a turns file goes 14 days after its task ended | landed, goal 3362; its walk after landing not yet run |
| S7, history in task logs: skein-kv's payload logs generalized, with compression; jig's history keyspaces moved to a log per task, read by page; new turns in the task's log | 2, 5, 6, 10.1 | temper/dev | S4, S5, S6 | after landing, on a staging deployment: the map holds no history keyspace, a task's page reads its history from its log, and dropping a task removes its one file | landed, goal 3523; its walk after landing not yet run |
| S8, hot records apart from cold: live tasks, goals' projections and temper's change records keyed as a part written once and a part that changes | 7, 10.1 | temper/dev | S4 | after landing, on a staging deployment: the census shows the log's writes per hour fallen, and every record reads back as before | landed, goal 3363; its walk after landing not yet run |

## 2. Order

- **S1 and S3 first,** in parallel: each needs nothing, and S3 makes the upgrades that bring the rest cheaper. S2 leaves no machinery. S1 reaches a deployment before the others' upgrades where it can, so each is measured.
- **S4 after them,** or beside them: from it on, a derived change rewrites nothing, and the jobs' runner exists.
- **S5 and S6 after S4,** in parallel.
- **S7 after S5 and S6.**
- **S8 after S4,** at any point.
- **Section 11's open questions** are the named milestones' to settle as they design.
