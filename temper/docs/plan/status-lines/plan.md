# status-lines: the plan

From `inputs/request.md` (the director, 2026-11-03, from the debt backfill): about 23 design gaps
the code shows and no status file records, one entry each in the status file named, and the
stale design texts of jig 7. Docs only. One task per tree, skein's split in two by file; none
needs another, so all five go out in the first push. `notes/common.md` holds the entry's form,
the label rule the citation checker enforces, and the check-the-code-first rule.

## Parts and tasks

1. **jig** (slice 1). `tasks/jig-lines.md`, high, ~50 minutes: jig 1 to 6 in
   `domain/engine.status.md` (1, 3, 4, 6), `client/domain/contract.status.md` (1),
   `client/domain/client.status.md` (2, D009) and `domain/testing.status.md` (5, only if
   `testing.md`, 3 asks it).
2. **skein** (slice 2), two tasks, one writer per file:
   - `tasks/skein-machines.md`, high, ~45 minutes: kv (5.4, D017; kv.md 9 re-checked), io (5.4,
     D018; io.md 5, D044), channel (5.4, D019), forge-connection (5.4 and 8.1/8.4, D020; 7.2
     re-checked), forge (5.4, D021; 4.1, D010).
   - `tasks/skein-rest.md`, high, ~50 minutes: oauth (D022), llm (D023; `verbosity` re-checked),
     llm-connection (D023), examples (D024), lib (D025), forge-setup (7, D031), testing (6,
     D027, D028), shell (7.5, D045), fake-forge and fake-llm (testing-strategy.md 4, settled
     with D043), and `STATUS.md`: seven documents leave its "have no gap" list, one sentence
     names the departures.
3. **smith** (slice 3). `tasks/smith-lines.md`, high, ~35 minutes: session (1, D004),
   testing (2, D017), benchmarks (6, D010), host (8, D018, the designer decides), and
   `STATUS.md` (session.md leaves the "have no gap" list); runs `smith-bench`'s tests, since
   smith's design tree is read by its guard checks.
4. **temper** (slice 4). `tasks/temper-lines.md`, high, ~45 minutes: reconcile.rs without a
   caller in `protocol/README.status.md` and `protocol/setup.status.md` (D006); temper-service
   as a copy of jig's reference service in `protocol/README.status.md`, 6, 7 (D001); temper-07's
   `not_installed` in `client/README.status.md`, 4.7 (D008); temper-12 checked against
   `domain/forge.status.md`'s merge-queue entry; and `protocol/configuration.md`, 5's
   `hello_bounds` sentence corrected (jig 7).

## What was decided

- **jig 7's second text is dropped:** `jig/docs/design/protocol/llm.md`, 5.6 no longer describes
  the JIGM/SKOT moved-record reader; commit `6532420160` (2026-11-02) removed it. The
  `hello_bounds` sentence stands (configuration.md, line 535, on main and on fabrig-scale's
  trunk); the code checks the agent's `bounds` directly (`temper-service/src/startup/checks.rs`,
  `admission`).
- **temper 2's file:** `temper/docs/design/protocol/README.status.md`, 6, 7, not jig's
  `service.status.md`. jig's code keeps to its design; temper's design is what is silent on the
  copy.
- **temper-13's crowded installations** gets no entry of its own: the comment is inside
  `reconcile.rs`, which temper 1's entry records as having no product caller.
- **temper-12** is expected to be covered by `domain/forge.status.md`'s existing merge-queue
  entry; the task writes an entry only if it is not.
- **The 5.4 lines** are labelled by each crate's own document's section, and cite
  `skein/docs/foundation/programming-model.md`, 5.4 as a path: the citation checker reads a
  status file's `N.N:` labels against its own document.
- **Debt items:** each entry cites the item the request names, in
  `jig/docs/design/examples.status.md`'s form ("the code's side is `<path>`"); jig 1 and 3 to 6
  name none and get none. The debt items themselves are not edited.
- **`STATUS.md`** changes only where a "have no gap" list names a document that now has a gap
  (skein, smith); jig's and temper's summaries stay.
- The "not re-checked" items (skein 9's three, jig 5's condition, temper 3's three) are checked
  by their tasks against the code; what they drop is in their reports.

## Critical path

Five independent docs tasks, the longest ~50 minutes (`jig-lines`, `skein-rest`), ~100 with the
buffer, then fab's end check. No task needs another.

## Overlaps

- `public-reading-build/public-status` (not merged) changes `jig/docs/design/domain/engine.status.md`'s
  **Not yet:** bullet and `jig/docs/design/client/domain/contract.status.md`'s last line, and
  owns `skein/docs/design/STATUS.md`. `jig-lines` appends to engine's **Departs:** bullet only,
  and puts contract's new **Departs:** bullet before its first bullet, so no hunk is shared.
  `skein-rest` edits one sentence of skein's `STATUS.md` and adds one; if public-status changes
  the same "have no gap" sentence (ui-web.md is in it), the second to land meets a conflict
  there. Not avoidable: the sentence is false once the entries land.
- `fabrig-scale` (its trunk and its open `init-standard`, `measure-box`) changes
  `temper/docs/design/protocol/setup.status.md`'s first **Not yet:** line; `temper-lines`
  appends to its **Departs:** bullet only. fabrig-scale rewrites much of
  `temper/docs/design/protocol/configuration.md`, but no hunk within 60 lines of the
  `hello_bounds` sentence.
- No other draft's listed files are touched.
