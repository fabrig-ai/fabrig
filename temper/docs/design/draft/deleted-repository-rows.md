# A store with a deleted project's leftovers starts again

Draft, 2026-11-15, on the user's word. This is the stop-gap; `project-delete-cascade.md` is the root fix.

On staging, 11-15, a person deleted two projects with `temper ctl project delete`. The deletion finished in the background after the next restart. It removed repository 20's `Repository` record but left records naming it, a `PullState` (pull 45) among them. From then on every start refused, on the old build and on main alike: `restart refused during link: forge PullState … links missing Repository`. The strict check has been on main since f26b0a0c90 (11-14), and the design calls such a link a writer's bug, refused before the switch (`temper/docs/design/domain/root.md`). The staging engine crash-looped until it was stopped. Its refusing store and a stopped backup are kept.

## 1. A one-shot job drops forge rows whose repository is gone

- **Today:** `temper-store` is a registered store at version 25. A change to what it holds goes through defaults, former names or an owner-written open job, not a new schema version (`temper/docs/design/protocol/store.md`). Open jobs may erase whole keys, read across ranges in bounded pages, and run once per store before the root restores, with a durable completion marker (jig's `docs/design/protocol/store/jobs.md`). Jobs report rows scanned and commits, not records dropped.
- **Wanted:**
  - A named open job, `deleted-repository-forge-rows`, is appended to temper's patch set after the existing jobs (`temper/crates/temper-protocol-store/src/patch_set.rs`, dispatched from `src/jobs.rs`). It runs once per store, before the root restores.
  - It erases the forge rows that name a repository with no stored `Repository` record. The list of families is explicit and covers those a project's release leaves behind: `BranchHead`, `PullState`, `Ci`, `Tier`, `Verification`, `LandingLog`, `Receipt`, the client's live rows for that repository, and its `Names` entries. Paired rows go together. The job judges by the stored `Repository`'s presence, never by the deployment's configuration.
  - It keeps what the link check already exempts, such as an ended forge's history and closing issue projections. Rows with a live owner stay, as do changes, issues, holds, entries, proposed effects and anything else that may stand for a pending outward effect. For any of these that names a missing repository, the job refuses the upgrade and names the record, rather than dropping it.
  - The upgrade's output names the job and its counts: the total dropped and the count per family. Add the smallest count field the job runner needs to carry them: `jig-protocol-store`'s job answer and `Ran` event, forwarded through the engine's notes to `temper/crates/temper-engine-shell/src/lines.rs` and `src/upgrade/report.rs`.
  - The link check stays strict for everything else, after the job too.
  - Document the job in `store.md` and jig's `jobs.md`, and the counts in the upgrade report's description.

## 2. Out

- Making `project delete` take its records with it: `project-delete-cascade.md`. Until it lands, projects aren't deleted on a deployment.
- Changing the link check's rules.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The store's job tests (`temper/crates/temper-protocol-store/src/tests/jobs.rs`):
  - orphan rows of each listed family are dropped, with exact counts;
  - rows of a repository that still exists stay;
  - paging across many rows;
  - the job completes once and doesn't run again;
  - a change or hold naming a missing repository refuses, naming it.
- jig's job tests (`jig/crates/jig-protocol-store/src/tests/phase/jobs.rs`): the count travels in the job's answer.
- The upgrade report's tests: the job's name and counts are printed.
- A restart story (`temper/tests/engine/domain/tests/suite/reconcile.rs`): a store holding a `PullState` whose repository is gone starts after the job. A store with some other missing link still refuses.

Run only the new and affected tests, by name.
