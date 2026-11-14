# store-shape: execution

2026-11-13. How [store-shape.md](store-shape.md) gets built, and how a deployment's store, dev.fabrig.ai's first, migrates to it in place (its section 10). It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. Each milestone reaches a deployment as an ordinary upgrade; none exports the store or starts one afresh.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| S1, the census: counts per keyspace and per layout in jig's component, kept as commits apply and rebuilt at recovery; `temper store census`; the deployment's document showing them | 8, 10.1 | temper/dev | none | after landing, on a staging deployment: the census matches a count of the snapshot by key prefix, and a put and an erase move it | landed; its walk not run |
| S2, identity walks rewrite the head alone: where a patch leaves a value type's old bytes valid, the walk rewrites the value's version and puts it, with no decode, machine or encode | 10.1 | temper/dev | none | after landing, on a staging deployment: an upgrade whose patch moves the tasks family walks its milestones in seconds, and every value reads back as before | landed, goal 2652; its walk after landing |
| S3, generations by hard links: opening's new generation links the previous one's snapshot and sealed segments and copies only the newest segment and the secrets' store; skein-io's link | 5, 10.1 | temper/dev | none | after landing, on a staging deployment: an upgrade's new generation shares the snapshot's file with the previous one, and going back before the new one's first commit opens the previous one at its commit | landed; its walk after landing not yet run |
| S4, layout ids: defaults in the codec's schemas; a value's head as its value type's layout id, the families' broad roots resolved by key; the store's registry, seeded from the kept texts with no value rewritten; derived plans at read; written changes as registered functions; jobs at open; the open's check; the frozen-text rule dropped from skein's codec design; what becomes of patch files in goals in flight | 3, 4, 10.1, 10.4 | temper/dev | none | after landing, on a staging deployment: a store from before S4 opens with its snapshot untouched and the registry added; a later field with a default reaches it with no patch and no walk, and an older value reads with the default | not started |
| S5, milestones as attempts: jig's tasks child's attempt and hold records; the milestones' readers moved to them (escalations and the task record, history and activity pages, goals' projections and temper's feed, the clients' wire with its version moved, the task page, the TUI and `temper ctl`); a job at open folding each task's milestones into its attempts and holds and erasing them | 6, 10.1 | temper/dev | S4 | after landing, on a staging deployment: after the upgrade the milestones' keyspace is empty, and each task's page shows its attempts as it showed its lifecycle | not started |
| S6, retention: each keyspace's horizon declared by its owner, as `store-shape.md`, section 6 sets them; the domain's timers erasing what has passed it, in pages; an ended task's summary written when it ends, and by a job at open for tasks already ended; turns' files dropped 14 days after their task ends | 6, 10.1 | temper/dev | S4 | after landing, on a staging deployment with its clock moved on: a task past its horizon keeps its summary and loses its full record, and a turns file goes 14 days after its task ended | not started |
| S7, history in task logs: skein-kv's payload logs generalized, with compression; jig's history keyspaces moved to a log per task, read by page; new turns in the task's log; a job at open moving each task's history out of the map | 2, 5, 6, 10.1 | temper/dev | S4, S5, S6 | after landing, on a staging deployment: after the upgrade the map holds no history keyspace, a task's page reads its history from its log, and dropping a task removes its one file | not started |
| S8, hot records apart from cold: live tasks, goals' projections and temper's change records keyed as a part written once and a part that changes, re-keyed by a job at open | 7, 10.1 | temper/dev | S4 | after landing, on a staging deployment: the census shows the log's writes per hour fallen, and every record reads back as before | not started |

## 2. Order

- **S1, S2 and S3 first,** in parallel: each needs nothing, and S2 and S3 make the upgrades that bring the rest cheaper. S1 reaches a deployment before the others' upgrades where it can, so each is measured.
- **S4 after them,** or beside them: from it on, a derived change rewrites nothing, and the jobs at open that S5 to S8 need exist.
- **S5 and S6 after S4,** in parallel.
- **S7 after S5 and S6.**
- **S8 after S4,** at any point.
- **Section 11's open questions** are the named milestones' to settle as they design.
