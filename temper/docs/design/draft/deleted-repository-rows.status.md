# deleted-repository-rows: status

2026-11-15. Where the design lags or departs from [deleted-repository-rows.md](deleted-repository-rows.md).

The design carries section 1 in `../protocol/store.md`, 4 (the job), jig's `protocol/store/jobs.md`, 1 and 3 (the counts) and `../protocol/binaries.md`, 2 (the `ran` line).

- **Design departs:** 1: "rows with a live owner stay" does not hold for a `Names` row: the job drops the entries naming a missing repository from any `Names` row, whatever its task, since a job at open reads temper's rows alone and cannot see whether a task is live (`../protocol/store.md`, 4). A live task that holds a resource there has a `Hold` naming it, which refuses the job. Decided by the goal's designer, 2026-11-15, as the draft lists the `Names` entries among what is dropped.
- **Design departs:** 3: the restart story runs in `temper-engine-service-component`'s tests, not in `temper/tests/engine/domain/tests/suite/reconcile.rs`: that world's store opens with no patch sets, so no job runs there. Decided by the goal's designer, 2026-11-15.
- **Open:** 1: whether the job also drops an `Application` row whose `Adopted` outcome names a deleted repository, which `project-delete-cascade.md`, 1 says a release leaves behind; until the user decides, the job refuses on one, naming it (`../protocol/store.md`, 4).
