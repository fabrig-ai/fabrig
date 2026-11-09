+++
id = "crash-rows"
part = "crash-rows"
kind = "feature"
needs = []
files = ["jig/testing/jig-test-store-files/src/**", "jig/testing/jig-core-kit/src/world.rs", "jig/testing/jig-core-kit/src/world/store.rs", "jig/testing/jig-core-kit/src/faults.rs", "jig/tests/core/tests/suite/faults.rs", "jig/tests/core/tests/suite/fuzzy/faults.rs", "jig/tests/store/src/**", "jig/tests/store/tests/suite/crashes.rs", "jig/tests/store/tests/suite/fuzzy/crashes.rs", "jig/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 240
+++

## Goal

The request's failing story: an engine killed at each point of a
decision's write (after each record, mid-frame, between families), and
killed again during the restart that follows, starts again from what was
durable, loading the last whole decision, never half of one, with no
panic and no refusal. Rows of the core world's crash table, over the
simulated disk's crash points, at the lowest world that holds both the
engine's store and its start's link pass. Each kill candidate is ruled
in or out by its row; one ruled in is fixed at the lowest tier that
shows it (request items 1 and 2).

Acceptance:

- The core world can run over its directory-backed store on skein's
  simulated disk without patch sets, and kill it at a chosen file
  operation counted from a chosen commit's submission or from the
  store's open (Steps, 1).
- The rows of Steps, 2 exist and pass. Each that failed on the trunk
  was committed first, alone, its failure quoted in the report, and its
  fix followed (Steps, 4); each that passed on the trunk rules its
  candidate out and stays as a guard. The report says which, per row.
- The referee holds after every row: the start reaches `Phase::Open`
  (`world.domain.ready()`), `start_failure()` is `None`, nothing
  panics, and the store's rows equal the rows after the last
  acknowledged commit, or after that and one whole uncertain one
  (`jig/docs/plan/store-crash-safe/notes/contracts.md`, 2).
- The **Not yet** entry `14: the core's world on the simulated disk: ...`
  in `jig/docs/design/protocol/store.status.md` is removed, its lines
  alone. That departs from the implementer's text, which closes
  `docs/design/` to you: the entry records this task's gap.

## Inputs

- `jig/docs/design/protocol/store.md`, 14 (*The store's world* and *The
  core's world on the simulated disk*), 6 (*Writes become operations*,
  *Group commit*), 10.4 (power loss).
- `jig/docs/design/domain/engine.md`, 5.1 (*One decision, one commit*,
  *Whole across families, through a kill*) and 6.
- `skein/docs/design/kv.md`, 3 (recovery), 4 (group commit), 5
  (snapshots); `skein/docs/design/kv.status.md`, its **Not yet** 9: the
  gaps in skein's own crash coverage that a kill row may reach.
- `jig/docs/plan/store-crash-safe/notes/contracts.md`, 2;
  `jig/docs/plan/store-crash-safe/notes/common.md`.
- `jig/docs/plan/store-crash-safe/inputs/request.md`, item 1.

## Owns and may change

- Owns the files in `files`.
- The harness (`jig-test-store-files`, ordinary Rust):
  - **A disk without patch sets.** Today `Store::build` picks the
    directory-backed store only when patch sets are given. Add an
    explicit choice (`Store::on_disk(limits, models)`, and a `Setup`
    field or constructor in `jig-core-kit`'s world that uses it, the
    name yours); every existing caller keeps what it picks now.
  - **A kill at an operation.** `Fault::Kill { from: KillFrom, operation:
    u32 }` with `KillFrom::Commit(u64)` (counted from that commit's
    submission to the component) and `KillFrom::Open` (counted from the
    store's open at a restart): the disk serves that many file
    operations of the store's lanes, then serves none, as
    `jig-store-world`'s `cut_at` does (`jig/tests/store/src/world.rs`,
    `cut_at` and its checks where operations are served), and the world
    marks the cut reached. A count of a commit's operations, as that
    world's `operation_count`, lets a row sweep them.
  - **A model of prefixes:** the rows after each submitted commit, kept
    from the commits' writes (as `jig-store-world`'s `Model` does), so
    the referee can compare.
- `jig-core-kit`: `Cut::Disk { from, operation }` beside `Submitted` and
  `Durable`, mapped to the store's fault, and its `reached_cut` the same
  way; nothing else of the kit's interface changes.
- Pre-approved:
  - **A case the core world cannot reach cheaply** (a snapshot needs the
    log past `SNAPSHOT_FLOOR`, 64 KiB, of commits; several commits in
    one skein-kv batch need them queued below the component): its row
    goes in `jig/tests/store/tests/suite/crashes.rs`, whose world makes
    snapshots (`patching.rs` sets one up) and holds several commits
    (`enqueue`, `settle`), with a commit pair that saves a person row
    and a task row naming it, and that world's referee; its `Model`
    grows to several uncertain commits, each a whole prefix. Say which
    rows moved and why. The core world's rows then cover the commit,
    the families and the double kill.
  - **Sweeps:** a row that sweeps every operation of a long commit or
    restart in the focused suite past its budget keeps its chosen cuts
    (each record's write, the frame's middle, the sync, the rename, the
    directory sync) in `faults.rs`, and the whole sweep goes under
    `jig/tests/core/tests/suite/fuzzy/faults.rs`
    (`jig/docs/design/domain/testing.md`, 9 says which).
  - **A fault a row shows below jig** is reproduced first as a row of
    `skein/tests/kv/tests/suite/store.rs` or `disk.rs`, then fixed in
    `skein/crates/skein-kv/src/**`; one in `jig-protocol-store` as a row
    of `jig/tests/store`, then fixed in
    `jig/crates/jig-protocol-store/src/**`. Both are yours for that
    fix. A fix that changes a stored format, a frame's layout or
    skein's `io`, `sim` or `world` interface is a `Question:`, with the
    failing rows pushed.
  - **A row that fails on a dangling link** (the start panics or refuses
    `Failure::Link` with no kill having split anything: the rows are a
    whole prefix but a link fails) shows a writer, not a kill: report it
    under `## Left` with the row; `writer-rows` owns writers.
- Not yours: `jig-domain-core`, the children, the writers,
  `jig/tests/core/tests/suite/start_links.rs` and `switch_step.rs`
  (`link-refusal`), every figure of a shipped profile.

## Steps

1. **The harness**, as `Owns and may change` says. A unit test in
   `jig-test-store-files` (its `src/tests.rs`) pins the kill: a commit
   killed at operation 0 is not durable after the restart, and one
   killed past its count is.
2. **The rows,** a table test in `jig/tests/core/tests/suite/faults.rs`,
   `a_kill_at_each_file_operation_of_a_decision_starts_from_a_whole_one`,
   its rows `(name, scenario, chosen commit, cuts)`, each run from its
   own world and seed:
   - **a task and the person task it links:** the commit that saves a
     live task whose executor is a person and the task that asked for
     it (find a story that makes an agent's or a procedure's person
     task: `rg 'PersonAddress::Person' jig/tests/core`), killed at each
     operation of that commit;
   - **a person and a task naming it:** `Boot::Caller` signs in a caller
     and makes its initial task; the commit that saves the `Person` row
     and the one that saves the task requested by that person, each
     killed at each operation;
   - **killed twice:** the first row's kill at a mid-frame operation,
     then a second kill at each operation counted from the restart's
     open (`KillFrom::Open`: recovery's repair snapshot, the start's own
     commits), then the next start;
   - **twice, then working:** after the first row's kill and a clean
     restart, the next decision's commit killed at each operation;
   - **a snapshot racing commits**, and **several commits in flight**,
     here or in `jig/tests/store` as pre-approved.
   After each kill, `world.restart(seed, ..)` and the referee
   (Acceptance). On the trunk, each row either passes (its candidate
   ruled out) or fails (ruled in).
3. A row of `random_commits_and_crashes_keep_the_referees_promises`
   (`jig/tests/store/tests/suite/fuzzy/crashes.rs`) that alternates a
   person commit and a task commit naming it, and kills twice in a row
   at seeded operations.
4. **A row that failed:** commit it first, alone, with its failure;
   reproduce it at the lowest tier (pre-approved), commit that row,
   then the fix. Break the fix once by hand, see the lowest row fail
   for the fault's reason, restore, and name it in the report.
5. **Runs.** `heavy cargo nextest run -p jig-test-store-files --lib`,
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/^faults::/) | test(/^switch_step::/) | test(/^restart::/)'`,
   `heavy cargo nextest run -p jig-core-world --test suite --profile fuzzy -E 'test(/^fuzzy::faults::/)'`,
   `heavy cargo nextest run -p jig-store-world` if you touched it; with a
   fix below jig, also `heavy cargo nextest run -p skein-kv -p skein-kv-tests`
   or `-p jig-protocol-store -p jig-store-world`, and every package that
   reads the changed crate's behaviour (`heavy cargo check -p <reader>
   --tests` across trees). Check each run's count. The kit's change is
   new API only, so its readers need only compile:
   `heavy cargo check -p jig-core-world --tests`.
6. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` (and
   `skein/scripts/old_forms.py` with a skein fix);
   `heavy cargo clippy -p jig-test-store-files -p jig-core-kit -p jig-core-world --all-targets -- -D warnings`
   (adding each package you fixed); remove the status entry; commit
   saying why (a kill at any point of a decision's write starts again
   from a whole decision, proven at the world that holds the store and
   the start); the citations check; push.

## Report

`Done:` / `Failed:` / `Question:` first. Then: per row, its candidate
ruled in or out, and the failure quoted for one ruled in; the fix and
its lowest row; rows moved to `jig/tests/store` and why; the
hand-broken rule; the runs and their counts, with each new test's time;
your implementation and check minutes apart.
