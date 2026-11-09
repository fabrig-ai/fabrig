+++
id = "link-refusal"
part = "link-refusal"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/start.rs", "jig/crates/jig-domain-core/src/start/links.rs", "jig/crates/jig-domain-core/src/start/projections.rs", "jig/crates/jig-domain-core/src/start/script.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-core/src/tests/start.rs", "jig/crates/jig-domain-core/src/tests/start/**", "jig/crates/jig-domain-people/src/restore.rs", "jig/crates/jig-domain-people/src/domain/start.rs", "jig/crates/jig-domain-people/src/tests/start.rs", "jig/crates/jig-domain-tasks/src/stored.rs", "jig/crates/jig-domain-tasks/src/stored/failure.rs", "jig/crates/jig-domain-tasks/src/tests/startup/**", "jig/tests/core/tests/suite/start_links.rs", "jig/tests/core/tests/suite/switch_step.rs", "temper/crates/temper-engine-shell/src/serve/host.rs", "temper/tests/engine/domain/tests/suite/root_routes.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 150
+++

## Goal

A store whose records fail a link at start refuses the start, naming the
record and what it links to, instead of panicking: the deployment's
`Tasks(Live(199)) links missing People(Person(102))` becomes
`Action::Refused(Failure::Link { .. })`, and `temper serve` prints it as
its reason and ends. This is the request's item 2, its second half; it
holds whatever the cause of the dangling link is (`crash-rows` and
`writer-rows` hunt that).

Acceptance:

- `Failure::Link { record: Key, linked: Linked }` and `Linked` exist as
  `jig/docs/plan/store-crash-safe/notes/contracts.md`, 1 writes them, and
  every link the start checks refuses through them: the core's, the
  people child's and the tasks child's, as that contract lists. None of
  them panics.
- The first failing row (Steps, 1) is committed first, alone, and the
  report quotes its panic on the trunk.
- A refused start commits nothing and sweeps nothing: the store's rows
  after the refusal equal those before it.
- Every test that expected one of these panics now expects the refusal,
  naming the same record and link (Steps, 4).
- `temper serve`'s words for it are the contract's, pinned by a row of
  `start_failures_name_the_set_key_setting_and_measured_limit`.
- The **Not yet** entry `6: a link the start cannot make panics, ...` in
  `jig/docs/design/domain/engine.status.md` is removed, its lines alone.
  That departs from the implementer's text, which closes `docs/design/`
  to you: the entry records this task's gap.

## Inputs

- `jig/docs/design/domain/engine.md`, 6 (*Load, then link*: *It refuses
  a link it cannot make*, *A link that cannot be made*) and 16 (*A record
  whose link cannot be made*).
- `jig/docs/plan/store-crash-safe/notes/contracts.md`, 1.
- `jig/docs/plan/store-crash-safe/notes/common.md`.
- `jig/docs/plan/store-crash-safe/inputs/request.md`, item 2.

## Owns and may change

- Owns the files in `files`.
- Readers, call sites and compile-forced arms only (any arm the new
  variants force in an exhaustive `match`, each edited for that alone):
  `jig/testing/jig-test-domain/src/lib.rs`,
  `jig/testing/jig-test-service/src/stop.rs`,
  `jig/examples/ops/domain/src/restart.rs`,
  `temper/crates/temper-engine-domain/src/restart.rs`,
  `temper/crates/temper-service/src/stop.rs`. The temper lines are the
  one-tree rule's exception for a required variant: each names the
  variant and nothing else.
- Tests that expect a link panic, wherever they are (Steps, 4): the ones
  the cut found are `jig/crates/jig-domain-core/src/tests/start/{inbox,projections,links}.rs`,
  `jig/crates/jig-domain-core/src/tests/start.rs`,
  `jig/crates/jig-domain-people/src/tests/start.rs` (`SignIn(7) links
  missing person 99`), `jig/crates/jig-domain-tasks/src/tests/startup/detail.rs`
  (`missing charter 99`), `jig/tests/core/tests/suite/switch_step.rs`
  (`Tasks(Live(1)) links missing project 1`), and
  `temper/tests/engine/domain/tests/suite/root_routes.rs` (`identity
  index collision at Person(2)`, `links missing People(Person(999))`).
  Another you find (`rg 'links missing|missing charter|missing run charter|missing task|identity index collision'`
  over `jig/` and `temper/`) is yours the same way, named in the report.
- Pre-approved:
  - **The core's mapping:** `start_collect_failure` (`start/script.rs`)
    turns a child's `RestoreCause::Link` or `Charter` into
    `Failure::Link`; the irrefutable `let people::RestoreCause::Bound`
    there becomes a `match`.
  - **A link check that cannot return early** (one nested in a loop of
    `start_cross_links`): it records the first missing link in the
    script (`start.missing` or a field beside it) and the Link step
    refuses with it once the pass ends; every later check is skipped
    or ignored, never asserted.
  - **The people child's identity collision** is found at load (`fits`),
    not at link: refuse it there as `RestoreCause::Link` with `record`
    the second person's key and `target` the first's, through the same
    `restore_failure` path a bound takes.
  - **An assert at start that is not a link** (a duplicate key, which
    the store's map cannot hold; `first.rs`'s seed policy; a phase's
    order) stays an assertion; name each you leave in the report.
  - **`jig-domain-accounts`:** if its start asserts a link a store can
    trip (engine.md, 6: a name reserved by another source), give it the
    same path (`RestoreCause::Link`-like, mapped to `Failure::Link`) and
    name it; if none, say so.
- Not yours: the writers (`routing/`, `admission/`), the store, every
  figure, `jig/tests/core/tests/suite/faults.rs` and `restart.rs`
  (`crash-rows`).

## Steps

1. **The failing row, first, alone.** In `jig/tests/core/tests/suite/start_links.rs`,
   a table test `a_store_that_fails_a_link_refuses_the_start_naming_both`:
   rows that restart a world over a store holding a dangling link, each
   `(name, plant, record, linked)`: a live task whose requester is a
   person the store lacks (the deployment's row: `Tasks(Live(n))`,
   `Linked::Record(People(Person(m)))`); a task naming a project with no
   policy; a run proof's task missing; a projection naming a connector
   the configuration lacks. Plant each by saving the row straight into
   the world's store before the restart (`world.store` holds the rows;
   see how `start_links.rs` and `switch_step.rs` build theirs). Assert the
   start answers `Refused(Failure::Link { record, linked })` with the
   row's pair, `world.domain.ready()` is false, and the store's rows are
   unchanged. On the trunk the first row panics; write the rows against
   the contract's names, so the commit compiles only with Steps, 2's
   types: commit the types' declarations with the row (no behaviour),
   then the behaviour.
2. **The types and the core's refusals.** `Failure::Link` and `Linked`
   in `start.rs`; `Linked` exported from `lib.rs`'s `start` re-exports.
   Each `assert!` in `start/links.rs` (the cut counts fourteen and four
   `expect`s on a link) and the link asserts of `start/projections.rs`
   (header, project, connector, live goal) and `start/script.rs`
   (`census_position`, `start.missing`) refuses as the contract says.
   `missing_proof_link` and `missing_charter_link` keep their shapes and
   feed the same refusal.
3. **The children.** People: `RestoreCause::Link { target }`;
   `domain/start.rs`'s `link` sets the restore failure and makes
   `start_link` answer false, instead of asserting; the identity
   collision as pre-approved. Tasks: `RestoreCause::Link { target }` and
   `RestoreCause::Charter { charter, revision }`; `stored.rs`'s
   `live_link`, `retained_link` and `links`' charter asserts set the
   restore failure the way a bound does, and `Event::Check` then answers
   `RestoreRefused` as for a bound.
4. **The readers.** Each test that expected a link panic now drives the
   same store to the refusal and asserts `Failure::Link` with the same
   record and link (or the child's `RestoreCause::Link` in a child's own
   unit test). Keep each test's name where it still says what it pins;
   rename one that says "panics". temper's shell: `start_failure` gains
   the contract's words; one row of
   `start_failures_name_the_set_key_setting_and_measured_limit` pins
   them. temper's two `root_routes.rs` tests assert the refusal.
5. Break the rule once by hand (put one `assert!` back in `link_person`),
   see the deployment's row fail for that reason, restore, and name it in
   the report.
6. **Runs.** This changes what a start does on a dangling link, which
   every start reader sees:
   `heavy cargo nextest run -p jig-domain-core --lib`,
   `heavy cargo nextest run -p jig-domain-people --lib`,
   `heavy cargo nextest run -p jig-domain-tasks --lib`,
   `heavy cargo nextest run -p jig-core-world`,
   `heavy cargo nextest run -p temper-engine-shell --lib`,
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/root_routes::/)'`,
   and `heavy cargo check --tests` for each reader package you edited
   (`-p jig-test-domain -p jig-test-service -p jig-ops-domain -p temper-engine-domain -p temper-service`).
   Check each run's count.
7. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-domain-core -p jig-domain-people -p jig-domain-tasks -p jig-core-world -p temper-engine-shell --all-targets -- -D warnings`;
   remove the status entry; commit saying why (a store that fails a link
   says which records disagree and stops with a reason, so a damaged
   store is a refusal an operator can read, not a crash loop); the
   citations check; push.

## Report

`Done:` / `Failed:` / `Question:` first. Then: the failing row's panic
on the trunk; each link check moved, by file, and those left as
assertions with why; each reader test changed; the hand-broken rule and
the failure seen; the runs and their counts; your implementation and
check minutes apart.
